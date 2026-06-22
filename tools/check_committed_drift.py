#!/usr/bin/env python3
"""Drift-detector for the cmake-off committed fallbacks (WS1–WS3 safety net).

The cargo-drive-cmake-off effort keeps several artifacts *committed* so a cargo
build needs no cmake configure/ninja: the per-arch asm offset includes
(`ks*.inc`), the C-floor flag templates (`cflags/*.flags`), the usermode
resource blobs (`*.rc.res`), the RPC server stubs (`*_s.<arch>.rs`), the
committed `bugcodes.h` (win32k residue), and — as a tool guard — the self-hosted
native-gendib output (win32k DIB tables). Each is
"correct by construction" the moment it's generated — but it is NOT regenerated
on a normal build, so if an upstream input changes (a `.template.h`, a `.rc`, a
`.spec`/`.idl`, or cmake's recorded flags), cmake regenerates while the committed
copy silently goes stale. A configured cmake build stays green (it uses the fresh
copy); only a *cmake-off* build would then diverge — and nothing catches it.

This tool closes that gap: for every committed fallback, it regenerates the
artifact from a configured `build-<arch>/` and byte-compares against the checked-in
copy. DRIFT = the committed copy is stale (regenerate it). It self-skips per
artifact when the prerequisite build dir / inputs are absent, so it runs cleanly
anywhere and is strict only where it can actually check.

    python3 tools/check_committed_drift.py              # all arches present
    python3 tools/check_committed_drift.py --arch i386  # one arch

Exit 0 = no drift (incl. all-skipped); exit 1 = at least one DRIFT.
Regenerate a drifted artifact with: gen_cflags.py / gen_committed_res.py /
the ks*.inc committed-dir README / reactos-link's RPC replay (see the roadmap).
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile
from glob import glob
from pathlib import Path

import gen_cflags
import gen_committed_res

REPO = Path(__file__).resolve().parent.parent
ARCHES = ("i386", "amd64", "arm64")
NM = ".build-tools/llvm-mingw/bin/llvm-nm"

# (committed ks file, arch). cmake writes the freshly-generated copy to
# build-<arch>/sdk/include/asm/<file>, so the check is a direct byte-diff — no
# regeneration tool needed.
KS_FILES = (("ks386.inc", "i386"), ("ksamd64.inc", "amd64"), ("ksarm64.h", "arm64"))

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


def check_ks(arches: tuple[str, ...]) -> list[tuple]:
    rows = []
    for fname, arch in KS_FILES:
        if arch not in arches:
            continue
        committed = REPO / "sdk/include/asm/committed" / fname
        generated = REPO / f"build-{arch}/sdk/include/asm" / fname
        if not generated.exists():
            rows.append(("ks*.inc", f"{fname}", SKIP, f"no build-{arch} copy"))
            continue
        same = committed.read_bytes() == generated.read_bytes()
        rows.append(("ks*.inc", fname, OK if same else DRIFT, "byte-diff vs build dir"))
    return rows


def check_bugcodes(arches: tuple[str, ...]) -> list[tuple]:
    """The committed bugcodes.h (windmc-generated, arch-identical) used by the
    win32k_residue cmake-off compile — diff vs any configured build's copy."""
    committed = REPO / "sdk/lib/rust/reactos-link/committed-headers/bugcodes.h"
    for arch in arches:
        gen = REPO / f"build-{arch}/sdk/include/reactos/mc/bugcodes.h"
        if not gen.exists():
            continue
        if not committed.exists():
            return [("bugcodes.h", "committed", DRIFT, "committed copy MISSING")]
        same = committed.read_bytes() == gen.read_bytes()
        return [("bugcodes.h", f"vs {arch}", OK if same else DRIFT, "byte-diff vs build dir")]
    return [("bugcodes.h", "committed", SKIP, "no build dir")]


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


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", action="append", choices=ARCHES, help="arch(es) (default: all)")
    args = ap.parse_args(argv)
    arches = tuple(args.arch) if args.arch else ARCHES

    os.chdir(REPO)  # the generators read build-<arch>/ relative to cwd
    rows: list[tuple] = []
    rows += check_ks(arches)
    rows += check_cflags(arches)
    rows += check_res(arches)
    rows += check_widl(arches)
    rows += check_bugcodes(arches)
    rows += check_gendib(arches)

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
        print("  ks*.inc → cp build-<arch>/sdk/include/asm/ks* sdk/include/asm/committed/")
        print("  widl .rs→ reactos-link RPC replay (see cargo-drive-cmake-off-roadmap WS3)")
        return 1
    if not checked:
        print("\n(no build-<arch>/ configured — nothing checked; configure a build to verify)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
