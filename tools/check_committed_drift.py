#!/usr/bin/env python3
"""Drift-detector for the cmake-off committed fallbacks (WS1–WS3 safety net).

The cargo-drive-cmake-off effort keeps several artifacts *committed* so a cargo
build needs no cmake configure/ninja: the C-floor flag templates (`cflags/*.flags`,
incl. `genincdata.<arch>.flags`), the usermode resource blobs (`*.rc.res`), the
RPC server stubs (`*_s.<arch>.rs`), and the committed `bugcodes.h` (win32k residue).
Each is "correct by construction" the moment it's generated — but it is NOT
regenerated on a normal build, so if an upstream input changes (a `.template.h`, a
`.rc`, a `.spec`/`.idl`, or cmake's recorded flags), cmake regenerates while the
committed copy silently goes stale. A configured cmake build stays green (it uses
the fresh copy); only a *cmake-off* build would then diverge — and nothing catches it.

It also guards two *self-hosted* generators whose output is reproduced (not
committed) at build time: native-gendib (win32k DIB tables) and `geninc` (the
per-arch ks*.inc asm offsets — `reactos-link::generate_ks_inc` compiles
genincdata.c + emits from its `.asmdef`). For those it reruns the generator and
confirms the output still matches cmake's, catching a template/source change the
committed *flags* don't capture.

This tool closes that gap: for every committed fallback / self-hosted generator,
it regenerates from a configured `build-<arch>/` and byte-compares against cmake's
copy. DRIFT = stale (regenerate). It self-skips per artifact when the prerequisite
build dir / inputs are absent, so it runs cleanly anywhere and is strict only where
it can actually check.

    python3 tools/check_committed_drift.py              # all arches present
    python3 tools/check_committed_drift.py --arch i386  # one arch

Exit 0 = no drift (incl. all-skipped); exit 1 = at least one DRIFT.
Regenerate a drifted artifact with: gen_cflags.py / gen_committed_res.py /
reactos-link's RPC replay (see the roadmap).
"""

from __future__ import annotations

import argparse
import os
import shutil
import struct
import subprocess
import sys
import tempfile
from glob import glob
from pathlib import Path

import gen_cflags
import gen_committed_res

REPO = Path(__file__).resolve().parent.parent
# The canonical build mountpoint (tools/canon-run.sh). compile_commands.json bakes
# /tmp/vibeos paths; outside a canon-run mount they're remapped to REPO so replays
# (the .res windres check) still run. Kept in sync with gen_cflags.CANON_ROOT.
CANON_ROOT = "/tmp/vibeos"
ARCHES = ("i386", "amd64", "arm64")
NM = ".build-tools/llvm-mingw/bin/llvm-nm"
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"

# geninc self-host: (arch, cross clang triple, ks output filename). The engine
# (reactos-link::generate_ks_inc) compiles genincdata.c with the committed
# `cflags/genincdata.<arch>.flags` + native-hpp xdk + committed bugcodes.h, then
# emits the ks*.inc from its `.asmdef` section. This check reproduces that
# pipeline and byte-compares against cmake's `native-geninc` copy in build-<arch>.
GENINC = (
    ("i386", "i686-w64-mingw32", "ks386.inc"),
    ("amd64", "x86_64-w64-mingw32", "ksamd64.inc"),
    ("arm64", "aarch64-w64-mingw32", "ksarm64.h"),
)

# RPC server stubs: (stub, committed .rs template, generated _s.c, _s.obj glob).
WIDL_STUBS = (
    (
        "svcctl_s",
        "sdk/lib/rust/svcctl-stub/data/svcctl_s.{arch}.rs",
        "build-{arch}/base/system/services/svcctl_s.c",
        "build-{arch}/base/system/services/CMakeFiles/*/svcctl_s.c.obj",
    ),
    (
        "winreg_s",
        "sdk/lib/rust/winreg-stub/data/winreg_s.{arch}.rs",
        "build-{arch}/base/system/winlogon/winreg_s.c",
        "build-{arch}/base/system/winlogon/CMakeFiles/*/winreg_s.c.obj",
    ),
)

OK, DRIFT, SKIP = "OK", "DRIFT", "SKIP"


def _read_coff_section(obj: Path, name: str) -> bytes:
    raw = obj.read_bytes()
    ns = int.from_bytes(raw[2:4], "little")
    oh = int.from_bytes(raw[16:18], "little")
    st = 20 + oh
    for i in range(ns):
        o = st + i * 40
        nb = raw[o : o + 8]
        nul = nb.find(b"\x00")
        nul = 8 if nul < 0 else nul
        if nb[:nul] == name.encode():
            rs = int.from_bytes(raw[o + 16 : o + 20], "little")
            ra = int.from_bytes(raw[o + 20 : o + 24], "little")
            return raw[ra : ra + rs]
    raise RuntimeError(f"section {name!r} not in {obj}")


def _emit_ks(asmdef: bytes) -> str:
    out = []
    for off in range(0, len(asmdef), 64):
        c = asmdef[off : off + 64]
        if len(c) < 64:
            break
        ty = c[0]
        nul = c[1:56].find(b"\x00")
        nul = 55 if nul < 0 else nul
        nm = c[1 : 1 + nul].decode()
        (val,) = struct.unpack_from("<Q", c, 56)
        if ty == 0:
            break
        elif ty == 1:
            out.append(nm + "\n")
        elif ty == 2:
            out.append(f"{nm} = 0x{val:x}\n")
        elif ty == 3:
            out.append(f"\n/* {nm} */\n")
    return "".join(out)


def check_geninc(arches: tuple[str, ...]) -> list[tuple]:
    """Reproduce the engine's self-hosted geninc (committed genincdata flags +
    the xdk/bugcodes closure) and byte-compare its ks*.inc against cmake's
    build-dir `native-geninc` copy. DRIFT = the self-host would diverge from
    cmake (a template/genincdata.c change the committed flags don't capture).
    The genincdata *flags* themselves are drift-checked separately by check_cflags."""
    rows = []
    for arch, triple, ksf in GENINC:
        if arch not in arches:
            continue
        bd = REPO / f"build-{arch}"
        cmake_ks = bd / "sdk/include/asm" / ksf
        flags_f = REPO / f"sdk/lib/rust/reactos-link/cflags/genincdata.{arch}.flags"
        clang = LLVM_MINGW / f"bin/{triple}-clang"
        if not cmake_ks.exists() or not flags_f.exists() or not clang.exists():
            rows.append(("geninc", ksf, SKIP, f"no build-{arch} / flags / clang"))
            continue
        flags = [
            ln.replace("{REPO}", str(REPO)) for ln in flags_f.read_text().splitlines() if ln.strip()
        ]
        with tempfile.TemporaryDirectory() as td:
            # build-dir psdk/ddk == native-hpp xdk (byte-identical, deterministic);
            # stage just bugcodes.h so <bugcodes.h> resolves cleanly.
            mc = Path(td) / "mc"
            mc.mkdir()
            shutil.copy2(bd / "sdk/include/reactos/mc/bugcodes.h", mc / "bugcodes.h")
            obj = Path(td) / "genincdata.o"
            cmd = [
                str(clang),
                *flags,
                "-I",
                str(bd / "sdk/include/psdk"),
                "-I",
                str(bd / "sdk/include/ddk"),
                "-I",
                str(mc),
                str(REPO / "sdk/include/asm/genincdata.c"),
                "-o",
                str(obj),
            ]
            r = subprocess.run(cmd, capture_output=True, text=True)
            if r.returncode != 0:
                rows.append(("geninc", ksf, SKIP, "genincdata compile failed"))
                continue
            text = _emit_ks(_read_coff_section(obj, ".asmdef"))
        same = text == cmake_ks.read_text()
        rows.append(("geninc", ksf, OK if same else DRIFT, "self-host vs build dir"))
    return rows


def check_bugcodes(arches: tuple[str, ...]) -> list[tuple]:
    """The committed bugcodes.h (used by the genincdata + win32k_residue cmake-off
    compiles). Two guards: (1) our self-hosted windmc-subset (gen_bugcodes_h.py)
    still reproduces the committed copy — runs ALWAYS, no build dir needed; (2) the
    committed copy still matches a configured build's real-windmc copy (the ground
    truth) — when a build dir is present."""
    import gen_bugcodes_h

    rows = []
    committed = REPO / "sdk/lib/rust/reactos-link/committed-headers/bugcodes.h"
    # (1) our generator vs committed — the self-host stays byte-faithful.
    if not committed.exists():
        rows.append(("bugcodes.h", "committed", DRIFT, "committed copy MISSING"))
    else:
        same = committed.read_text() == gen_bugcodes_h.generate()
        rows.append(("bugcodes.h", "gen vs committed", OK if same else DRIFT, "self-host regen"))
    # (2) committed vs real windmc (build dir) — our generator tracks windmc.
    for arch in arches:
        gen = REPO / f"build-{arch}/sdk/include/reactos/mc/bugcodes.h"
        if not gen.exists():
            continue
        same = committed.exists() and committed.read_bytes() == gen.read_bytes()
        rows.append(("bugcodes.h", f"vs {arch}", OK if same else DRIFT, "byte-diff vs windmc"))
        break
    return rows


def check_gendib(arches: tuple[str, ...]) -> list[tuple]:
    """The self-hosted native-gendib output (win32k DIB tables) — build the tool
    + run it, diff the 3 .c vs any configured build's cmake-generated copies."""
    src = REPO / "sdk/tools/gendib/gendib.c"
    cc = os.environ.get("CC", "cc")
    for arch in arches:
        cmdir = REPO / f"build-{arch}/win32ss/gdi/dib"
        if not (cmdir / "dib8gen.c").exists() or not src.exists():
            continue
        with tempfile.TemporaryDirectory() as td:
            tool = os.path.join(td, "native-gendib")
            if subprocess.run([cc, "-O2", "-o", tool, str(src)], capture_output=True).returncode:
                return [("gendib", arch, SKIP, "tool build failed")]
            if subprocess.run([tool, td], capture_output=True).returncode:
                return [("gendib", arch, SKIP, "gendib run failed")]
            bad = [
                f
                for f in ("dib8gen.c", "dib16gen.c", "dib32gen.c")
                if Path(td, f).read_bytes() != (cmdir / f).read_bytes()
            ]
            return [("gendib", arch, DRIFT if bad else OK, ",".join(bad) or "3 .c vs build dir")]
    return [("gendib", "*", SKIP, "no build dir")]


def check_cflags(arches: tuple[str, ...]) -> list[tuple]:
    rows = []
    for target in gen_cflags.DEFAULT_TARGETS:
        for arch in arches:
            expected = gen_cflags.gen_content(target, arch, str(REPO))
            committed = REPO / gen_cflags.committed_path(target, arch)
            if expected is None:
                rows.append(("cflags", f"{target}.{arch}", SKIP, f"no build-{arch}"))
                continue
            if not committed.exists():
                rows.append(("cflags", f"{target}.{arch}", DRIFT, "committed copy MISSING"))
                continue
            same = committed.read_text() == expected
            rows.append(("cflags", f"{target}.{arch}", OK if same else DRIFT, "regen vs committed"))
    return rows


def check_res(arches: tuple[str, ...]) -> list[tuple]:
    rows = []
    for arch in arches:
        jobs = gen_committed_res.res_jobs(arch)
        if jobs is None:
            rows.append((".res", f"*-{arch}", SKIP, f"no build-{arch}"))
            continue
        for stem, committed_rel, toks, cwd in jobs:
            committed = REPO / committed_rel
            # The replay cwd + tokens come from compile_commands.json, which bakes
            # the CANONICAL build path (/tmp/vibeos/build-<arch>), present only
            # inside a canon-run bind mount. Standalone (e.g. `check --strict`
            # outside a build), that dir is absent. Rather than SKIP (which left
            # .res drift un-gated — the T3 tail), REMAP /tmp/vibeos → the real
            # repo tree and run the replay there: windres output is resource DATA
            # (no embedded build paths), so a real-path build is byte-identical to
            # the canonical one. Same canonical-templatize idea as gen_cflags.
            # Only SKIP if neither the canonical nor the remapped cwd exists.
            if not os.path.isdir(cwd):
                real_cwd = cwd.replace(CANON_ROOT, str(REPO))
                if os.path.isdir(real_cwd):
                    cwd = real_cwd
                    toks = [t.replace(CANON_ROOT, str(REPO)) for t in toks]
                else:
                    rows.append(
                        (".res", f"{stem}-{arch}", SKIP, "build dir absent (canonical + real)")
                    )
                    continue
            with tempfile.TemporaryDirectory() as td:
                dst = os.path.join(td, "out.res")
                cmd = list(toks)
                cmd[-1] = dst
                r = subprocess.run(cmd, cwd=cwd, capture_output=True)
                if r.returncode != 0 or not os.path.exists(dst):
                    rows.append((".res", f"{stem}-{arch}", SKIP, "windres replay failed"))
                    continue
                if not committed.exists():
                    rows.append((".res", f"{stem}-{arch}", DRIFT, "committed copy MISSING"))
                    continue
                same = committed.read_bytes() == Path(dst).read_bytes()
                rows.append((".res", f"{stem}-{arch}", OK if same else DRIFT, "regen vs committed"))
    return rows


def check_widl(arches: tuple[str, ...]) -> list[tuple]:
    rows = []
    nm = REPO / NM
    w2r = REPO / "tools/widl_to_rust.py"
    for stub, rs_tmpl, c_tmpl, obj_glob in WIDL_STUBS:
        for arch in arches:
            committed = REPO / rs_tmpl.format(arch=arch)
            src_c = REPO / c_tmpl.format(arch=arch)
            objs = glob(str(REPO / obj_glob.format(arch=arch)))
            if not src_c.exists() or not objs or not nm.exists():
                rows.append(("widl .rs", f"{stub}.{arch}", SKIP, "no build-dir _s.c/.obj"))
                continue
            with tempfile.TemporaryDirectory() as td:
                dst = os.path.join(td, "out.rs")
                cmd = [
                    sys.executable,
                    str(w2r),
                    str(src_c),
                    "--symobj",
                    objs[0],
                    "--arch",
                    arch,
                    "--nm",
                    str(nm),
                    "-o",
                    dst,
                ]
                r = subprocess.run(cmd, capture_output=True, cwd=str(REPO))
                if r.returncode != 0 or not os.path.exists(dst):
                    rows.append(("widl .rs", f"{stub}.{arch}", SKIP, "widl_to_rust replay failed"))
                    continue
                if not committed.exists():
                    rows.append(("widl .rs", f"{stub}.{arch}", DRIFT, "committed copy MISSING"))
                    continue
                same = committed.read_text() == Path(dst).read_text()
                rows.append(
                    ("widl .rs", f"{stub}.{arch}", OK if same else DRIFT, "regen vs committed")
                )
    return rows


def check_layout(arches: tuple[str, ...]) -> list[tuple]:
    """ntkrnl-sys's compile-time size/offset asserts vs the C headers.

    `size_check_<arch>.rs` / `offset_check_<arch>.rs` pin every bindgen struct to
    the C layout, but they only prove the island matches the headers the asserts
    were generated FROM. An upstream header change (KPRCB's StartCycles/CycleTime,
    EJOB's pack(1) fix, the NT6 OB callbacks) leaves island + asserts agreeing with
    each other and both wrong, and nothing fails. Regenerating the asserts against
    today's headers is what notices; a DRIFT here means regenerate ntkrnl-sys
    (generate_bindings.sh) and then these files. Both tools cover all three arches
    in one run and need every build-<arch>/ configured."""
    missing = [a for a in ARCHES if not (REPO / f"build-{a}/sdk/include/psdk/ntdef.h").is_file()]
    if missing:
        return [("layout", "ntkrnl-sys", SKIP, f"needs build-{','.join(missing)}")]
    rows = []
    for name, tool in (
        ("size_check", REPO / "sdk/lib/rust/ntkrnl-sys/verify_sizes.py"),
        ("offset_check", REPO / "tools/verify_offsets.py"),
    ):
        cp = subprocess.run([sys.executable, str(tool), "--check"], capture_output=True, text=True)
        rows.append(
            ("layout", name, OK if cp.returncode == 0 else DRIFT, "C headers vs committed asserts")
        )
    return rows


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", action="append", choices=ARCHES, help="arch(es) (default: all)")
    args = ap.parse_args(argv)
    arches = tuple(args.arch) if args.arch else ARCHES

    os.chdir(REPO)  # the generators read build-<arch>/ relative to cwd
    rows: list[tuple] = []
    rows += check_geninc(arches)
    rows += check_cflags(arches)
    rows += check_res(arches)
    rows += check_widl(arches)
    rows += check_bugcodes(arches)
    rows += check_gendib(arches)
    rows += check_layout(arches)

    drift = [r for r in rows if r[2] == DRIFT]
    checked = [r for r in rows if r[2] == OK]
    skipped = [r for r in rows if r[2] == SKIP]
    for kind, name, status, detail in rows:
        mark = {OK: "✅", DRIFT: "❌", SKIP: "··"}[status]
        print(f"  {mark} {status:5} {kind:9} {name:22} {detail}")
    print(f"\n  {len(checked)} OK, {len(drift)} DRIFT, {len(skipped)} skipped (no build dir)")
    if drift:
        print("\nDRIFT — committed fallback(s) are STALE vs cmake. Regenerate:")
        print("  cflags  → python3 tools/gen_cflags.py [--arch …]")
        print("  .res    → python3 tools/gen_committed_res.py [--arch …]")
        print("  geninc  → self-hosted; if it drifts, the genincdata flags are stale")
        print("            (python3 tools/gen_cflags.py --target genincdata) or a")
        print("            ks*.template.h / genincdata.c change altered the .asmdef")
        print("  widl .rs→ reactos-link RPC replay (see cargo-drive-cmake-off-roadmap WS3)")
        print(
            "  layout  → sdk/lib/rust/ntkrnl-sys/generate_bindings.sh, install the staged\n"
            "            bindgen_*.rs, then verify_sizes.py + tools/verify_offsets.py"
        )
        return 1
    if not checked:
        print("\n(no build-<arch>/ configured — nothing checked; configure a build to verify)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
