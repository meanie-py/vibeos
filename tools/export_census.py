#!/usr/bin/env python3
"""For every export a DLL declares, say what actually backs it — Rust, C, or neither.

WHY THIS EXISTS
---------------
"How far along is msvcrt?" was answered from prose for months, and the prose was wrong twice
in two days. Once because the headline counted only the Wine-derived `.c` and quietly skipped
`reactos/`, and once because the build tree was three days stale, so three landed waves still
read as C. Both times the claim was confident and specific and nobody could see the error
without redoing the measurement by hand — which is the definition of a missing tool.

So: no prose, no allowlist, no source scanning. The `.spec` says what the DLL exports; the
build tree says who defines each symbol; the linker's own command line says in what ORDER it
would pick. Everything else is derived.

WHAT "BACKS AN EXPORT" MEANS
----------------------------
Link order, and nothing else:

  1. the MODULE's own objects (`CMakeFiles/<name>.dir/*.obj`) — always linked, never
     lazily, so they win outright. This is where the `va_start` trampolines live: stable Rust
     cannot define a C-variadic function, so those shells are C on purpose and forever.
  2. any archive inside `--whole-archive` — every member is linked whether referenced or
     not, so it beats every lazy archive after it. This is how the Rust cohab wins.
  3. the remaining archives, in command-line order, each pulled member-by-member and only
     to satisfy a still-undefined symbol.

A cohab flip is exactly "the Rust archive got to a name before msvcrt_static did", and
`--allow-multiple-definition` means the loser is silent. That silence is the whole reason to
ask the binaries instead of the sources.

WHAT IS NOT A PORT CANDIDATE
----------------------------
Four buckets are counted separately because reporting them as "still C" overstates the work
left, and folding them into "done" overstates the progress:

  * `stub`   — spec2def generates a body that returns failure. There is no implementation
               in any language to port.
  * `forward`— `@ cdecl __threadid() kernel32.GetCurrentThreadId`. The export table names
               another DLL; no code is emitted here at all.
  * a module object — the `...` trampolines above.
  * an archive that is not this module's own C — `libcrtmath.a`, `libdbgrpt.a`. Real code
               and real port candidates, but they belong to a DIFFERENT lane with a
               different set of consumers. An IMPORT library lands here too and is not a
               port candidate at all: msvcrt's `memmove` resolves through `libntdll.a`, so
               the DLL's exported `memmove` is a six-byte `jmp *__imp_memmove` into
               ntdll.dll — where it is already Rust.

STALENESS
---------
A census computed from an old build tree is not a weaker answer, it is a wrong one that looks
identical to a right one. If any workspace `.rs` is newer than the DLL, this exits non-zero
and tells you to rebuild rather than printing numbers.

USAGE
    export_census.py --module msvcrt --arch amd64
    export_census.py --module msvcrt --arch amd64 --detail C          # list the still-C ones
    export_census.py --module msvcrt --arch all --json
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Each entry names a DLL whose exports we can account for. `spec` and `dll` are enough to ask
# the question; `own_c` names the archive holding the module's OWN C bodies, which is the only
# thing that distinguishes "not ported yet" from "belongs to another lane".
MODULES = {
    "msvcrt": {
        "spec": "dll/win32/msvcrt/msvcrt.spec",
        "dll": "dll/win32/msvcrt/msvcrt.dll",
        # No DLL links msvcrt_static since 2026-10-04; kept so a relink would be named
        # rather than misfiled as some other archive.
        "own_c": "libmsvcrt_static.a",
        "rust": "libmsvcrt_crt_cargo.a",
        # spec2def generates this object (Rust since 2026-10-06) and puts NOTHING but
        # stub bodies in it. An export backed by it is a stub whatever its spec line
        # says — msvcrt's `_cprintf_p`
        # family reaches it through `@ stub _cprintf_p(str)`, whose argument list makes
        # the directive scan below see a normal entry.
        "stubs_obj": "msvcrt_stubs.rs.obj",
    },
}
# The i386-only legacy CRTs link the same archive (msvcrt_rust_cohab.cmake); their own
# behaviour is in it too, under CRTDLL_/MSVCRT20_ names their specs map.
for _legacy in ("msvcrt20", "msvcrt40", "crtdll"):
    MODULES[_legacy] = {
        **MODULES["msvcrt"],
        "spec": f"dll/win32/{_legacy}/{_legacy}.spec",
        "dll": f"dll/win32/{_legacy}/{_legacy}.dll",
        "stubs_obj": f"{_legacy}_stubs.rs.obj",
    }

# The arches a `-arch=` spec filter can name, and which build each one belongs to. spec2def
# resolves these; the list mirrors its own, and a name absent here is a spec typo rather than
# something to pass through silently.
ARCH_TAGS = {
    "i386": {"i386", "win32"},
    "amd64": {"x86_64", "win64"},
    "arm64": {"arm64", "win64"},
}

# Tags that are real spec2def arches but no build here. `arm` is 32-bit ARM, which this tree
# does not target; msvcrt.spec carries a full set of `-arch=arm` C++ ABI thunks for it.
# Listed rather than tolerated silently, because the alternative to naming them is accepting
# any tag at all — and a typo'd one would drop its export from every arch's census with
# nothing to notice.
UNBUILT_ARCH_TAGS = {"arm"}


def _nm() -> Path:
    nm = ROOT / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm"
    if not nm.is_file():
        sys.exit(f"export_census: llvm-nm not found at {nm}")
    return nm


def _spellings(name: str) -> set[str]:
    """One symbol, every spelling it can be written as across arches.

    i386 PE prepends an underscore and `__stdcall` appends `@<arg bytes>`; amd64/arm64 do
    neither. Same helper, same reason, as verify_cohab_coverage.py's.
    """
    out = {name, name.split("@")[0]}
    if name.startswith("_"):
        out |= {n[1:] for n in list(out)}
    return out


def parse_spec(path: Path, arch: str) -> dict[str, tuple[str, str]]:
    """`export name -> (kind, backing symbol)` for one arch.

    Three things here are easy to get wrong, and each one silently inflates or deflates the
    census rather than failing:

      * `extern` is the ONLY directive whose export name and backing symbol differ —
        `@ extern _mbctype MSVCRT_mbctype`. For every other directive the trailing word, when
        present, is the implementation name and the leading word is the export.
      * the backing symbol comes AFTER the argument list: `@ cdecl time(ptr) MSVCRT_time`.
        A regex that stops at the first word finds `time`, does not find it in any archive,
        and files a real Rust export under "unresolved". That inflated one run by 22.
      * `-arch=` restricts a line to some builds. Ignoring it counts i386's whole x87 and
        port-IO surface against amd64, which inflated another run by 27.
    """
    exports: dict[str, tuple[str, str]] = {}
    for raw in path.read_text().splitlines():
        line = raw.split("#")[0].rstrip()
        m = re.match(
            r"^\s*(?:@|\d+)\s+(\S+)((?:\s+-\S+)*)\s+(\S+?)\s*(\(.*\))?(?:\s+(\S+))?\s*$", line
        )
        if not m:
            continue
        directive, flags, name, _args, second = m.groups()
        am = re.search(r"-arch=(\S+)", flags or "")
        if am:
            want = ARCH_TAGS[arch]
            tags = am.group(1).split(",")
            # spec2def accepts NEGATED tags — `-arch=!i386` is "every arch but i386" — and
            # msvcrt.spec uses them for the C++ ABI thunk families. Reading `!i386` as a
            # positive tag inverts the answer for that whole group.
            yes = {t for t in tags if not t.startswith("!")}
            no = {t[1:] for t in tags if t.startswith("!")}
            known = set().union(*ARCH_TAGS.values()) | UNBUILT_ARCH_TAGS
            unknown = (yes | no) - known
            if unknown:
                sys.exit(f"export_census: {path.name}: unknown -arch= tag(s) {sorted(unknown)}")
            if no & want:
                continue
            if yes and not (yes & want):
                continue
        backing = second or name
        kind = directive
        if "." in backing:
            kind = "forward"  # `kernel32.GetLogicalDrives` — the export table names another DLL
        exports[name] = (kind, backing)
    return exports


def defined_in(nm: Path, path: Path) -> dict[str, str]:
    """`symbol -> member name` for every GLOBAL definition in an archive or object.

    Lowercase nm codes are file-local: they cannot satisfy anyone else's reference and cannot
    be what an export resolves to, so they are not answers to this question.
    """
    out: dict[str, str] = {}
    r = subprocess.run(
        [str(nm), "--print-file-name", "--defined-only", str(path)],
        capture_output=True,
        text=True,
    )
    for ln in r.stdout.splitlines():
        m = re.match(r"^(.*?):\s*([0-9a-fA-F]*)\s+(\w)\s+(\S+)$", ln)
        if not m:
            continue
        who, _addr, kindc, sym = m.groups()
        if not kindc.isupper() or sym == "@feat.00":
            continue
        member = Path(who.split(":")[-1] or who).name
        out.setdefault(sym, member)
        # i386 `__stdcall` decorates the DEFINITION (`__adj_fdiv_m16i@4`) while the spec names
        # the bare symbol, so key the undecorated spelling too, or every stdcall export reads
        # UNRESOLVED (12 of msvcrt's did).
        if re.fullmatch(r"[^@]+@\d+", sym):
            out.setdefault(sym.split("@")[0], member)
    return out


def _dll_block(build: Path, mod: dict) -> str:
    ninja = build / "build.ninja"
    if not ninja.is_file():
        sys.exit(f"export_census: {ninja} missing.\n  fix: ./vibeos-cli build --arch <a>")
    text = ninja.read_text()
    at = text.find(f"build {mod['dll']}:")
    if at < 0:
        sys.exit(f"export_census: no rule building {mod['dll']} in {ninja}")
    return text[at : text.find("\nbuild ", at + 1)]


def module_objects(build: Path, mod: dict) -> list[Path]:
    """The objects linked DIRECTLY into the DLL: the build statement's explicit inputs.

    Not a glob of the target's `CMakeFiles/*.dir`. A deleted source leaves its object behind
    there, and the glob counted msvcrt's five retired `rust_*_shim.c.obj` as live providers
    of 113 exports for weeks after the shims became va_entry! thunks in the Rust archive.
    """
    statement = _dll_block(build, mod).split("\n", 1)[0]
    explicit = statement.split(":", 1)[1].split("|", 1)[0].split()[1:]  # [0] is the rule
    return [build / t for t in explicit if t.endswith(".obj")]


def link_inputs(build: Path, mod: dict) -> tuple[list[Path], list[Path]]:
    """`(whole_archive, lazy_archives)` straight out of build.ninja, in command-line order.

    Read from the generator's own output rather than restated here, because a hand-kept list
    of link inputs is a second source of truth that drifts the first time someone adds a
    library — and the drift shows up as an export mysteriously "unresolved". `LINK_FLAGS`
    matters too: msvcrt's `memmove` comes from a compiler-rt builtins archive that appears
    only there, so a tool reading LINK_LIBRARIES alone reports it as backed by nothing.
    """
    block = _dll_block(build, mod)
    whole, lazy, in_whole = [], [], False
    for var in ("LINK_LIBRARIES", "LINK_FLAGS"):
        m = re.search(rf"^\s*{var} = (.*)$", block, re.M)
        if not m:
            continue
        for tok in m.group(1).split():
            if tok.endswith("--whole-archive"):
                in_whole = True
                continue
            if tok.endswith("--no-whole-archive"):
                in_whole = False
                continue
            if not tok.endswith(".a"):
                continue
            p = Path(tok) if Path(tok).is_absolute() else build / tok
            if p.is_file():
                (whole if in_whole else lazy).append(p)
    return whole, lazy


def _require_fresh(build: Path, mod: dict, arch: str) -> None:
    dll = build / mod["dll"]
    if not dll.is_file():
        sys.exit(f"export_census: {dll} missing.\n  fix: ./vibeos-cli build --arch {arch}")
    newest, newest_rel = 0.0, ""
    for src in (ROOT / "sdk" / "lib" / "rust").rglob("*.rs"):
        if "/target" in str(src):
            continue
        t = src.stat().st_mtime
        if t > newest:
            newest, newest_rel = t, str(src.relative_to(ROOT))
    if newest > dll.stat().st_mtime:
        sys.exit(
            f"export_census: build-{arch} is STALE — {newest_rel} is newer than "
            f"{mod['dll']}. A census off an old tree is not a weaker answer, it is a wrong "
            f"one that looks exactly like a right one; three landed waves once read as "
            f"still-C this way.\n  fix: ./vibeos-cli build --arch {arch}"
        )


def census(module: str, arch: str) -> dict:
    mod = MODULES[module]
    build = ROOT / f"build-{arch}"
    if not build.is_dir():
        sys.exit(
            f"export_census: no build-{arch} in THIS worktree — generated artifacts never "
            f"come from a sibling's.\n  fix: ./vibeos-cli build --arch {arch}"
        )
    _require_fresh(build, mod, arch)
    nm = _nm()

    whole, lazy = link_inputs(build, mod)
    # Resolution order, most-preferred first. Module objects are not lazy and not an archive,
    # so they precede everything; --whole-archive members precede every lazy archive.
    providers: list[tuple[str, dict[str, str]]] = []
    mod_objs: dict[str, str] = {}
    for obj in module_objects(build, mod):
        for sym, _member in defined_in(nm, obj).items():
            mod_objs.setdefault(sym, obj.name)
    providers.append(("module-obj", mod_objs))
    for a in whole + lazy:
        providers.append((a.name, defined_in(nm, a)))

    exports = parse_spec(ROOT / mod["spec"], arch)
    buckets: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for name, (kind, backing) in sorted(exports.items()):
        if kind in ("stub", "forward"):
            buckets[kind].append((name, ""))
            continue
        cands = _spellings(backing) | _spellings("_" + backing) | _spellings("MSVCRT_" + backing)
        for who, table in providers:
            hit = next((c for c in cands if c in table), None)
            if hit is None:
                continue
            if who == mod["rust"]:
                label = "rust"
            elif who == mod["own_c"]:
                label = f"C: {table[hit]}"
            elif who == "module-obj":
                label = (
                    "stub" if table[hit] == mod.get("stubs_obj") else f"module-obj: {table[hit]}"
                )
            else:
                label = f"archive: {who}"
            buckets[label].append((name, table[hit]))
            break
        else:
            buckets["UNRESOLVED"].append((name, backing))
    return {"module": module, "arch": arch, "total": len(exports), "buckets": buckets}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--module", default="msvcrt", choices=sorted(MODULES))
    ap.add_argument("--arch", default="all", choices=["i386", "amd64", "arm64", "all"])
    ap.add_argument("--detail", help="list the exports in every bucket whose label contains this")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    arches = ["i386", "amd64", "arm64"] if args.arch == "all" else [args.arch]
    arches = [a for a in arches if (ROOT / f"build-{a}").is_dir()]
    if not arches:
        sys.exit(
            "export_census: no build-<arch> in this worktree.\n"
            "  fix: ./vibeos-cli build --arch i386"
        )

    out = []
    for arch in arches:
        r = census(args.module, arch)
        out.append(r)
        if args.json:
            continue
        print(f"=== {r['module']} [{arch}] — {r['total']} exports ===")
        for label in sorted(r["buckets"], key=lambda k: (-len(r["buckets"][k]), k)):
            print(f"{len(r['buckets'][label]):6d}  {label}")
        if args.detail:
            for label in sorted(r["buckets"]):
                if args.detail not in label:
                    continue
                print(f"  --- {label} ---")
                for name, member in r["buckets"][label]:
                    print(f"      {name}" + (f"  <- {member}" if member else ""))
        print()

    if args.json:
        print(
            json.dumps(
                [{**r, "buckets": {k: v for k, v in r["buckets"].items()}} for r in out], indent=2
            )
        )
    # An UNRESOLVED export means the tool failed to account for something, not that the DLL
    # is broken — it links. Loud rather than a quiet row, so a parsing regression cannot pass
    # as a finding about the code.
    if any("UNRESOLVED" in r["buckets"] for r in out):
        print(
            "export_census: some exports could not be attributed — see UNRESOLVED", file=sys.stderr
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
