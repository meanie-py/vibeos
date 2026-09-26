#!/usr/bin/env python3
"""Every GLOBAL symbol a cohab-flipped C file exports must already be resolved before it.

WHY THIS EXISTS
---------------
A cohab flip works by first-match: the Rust archive and the module's own objects link ahead of
the C static library, so each symbol they define wins and the C body goes dead in place. That
has a precondition nothing currently checks — **everything that links first must, between
them, define EVERY global symbol of the flipped C object that anything references.** Miss one
and lld pulls the C object in to satisfy it; that object also defines every OTHER symbol in
the TU, so the flip collapses into duplicate definitions, or (with
`--allow-multiple-definition`) into member-order roulette that varies with link order.

The failure that actually motivated this is quieter and worse. If the Rust defines a symbol
under the WRONG NAME, everything still builds and boots — Rust exports a symbol nothing
imports, the C copy stays live, and the "flip" is a no-op. Three of those shipped in one file
(msvcrt math.c) and none is visible in the C source:

  * `MSVCRT__rotl64` / `MSVCRT__rotr64` translated under a bare `_rotl64` / `_rotr64`.
  * `math_error` must be EXPORTED as `_invoke_matherr`, because the module's CMakeLists
    carries `-Dmath_error=_invoke_matherr`. The source never spells the symbol it defines.
  * The eight `__stdcall` FDIV stubs need `#[export_name = "__adj_fdiv_m16i@4"]`; on i386 PE
    `#[no_mangle]` emits the name verbatim and drops both the leading underscore and the `@N`.

So the oracle is the OBJECT FILE, not the source. `llvm-nm --defined-only` on the C object is
ground truth for what the TU exports, after every `-D`, every `#ifdef`, and every ABI
decoration the compiler applied.

WHY IT COMPARES BINARIES AND NOT RUST SOURCE
--------------------------------------------
Parsing the Rust twin for `fn` names was the first attempt, and it reported 81 findings that
were all false, because two whole provider categories are invisible from that one file:

  * a SIBLING file in the same archive — cnt-rust's difftest-pinned `cabi/math.rs` owns `abs`,
    `labs`, `_finite`, `_isnan` and friends, which is precisely why math.rs gates those off
    with `#[cfg(any())]`;
  * a C SHIM TU compiled into the msvcrt MODULE target rather than into msvcrt_static —
    `rust_file_varargs_shim.c` holds file.c's twenty `va_start` trampolines, because stable
    Rust cannot define a C-variadic function. Module objects resolve before archive members,
    so those definitions win.

Asking the binaries gets both for free and cannot drift: the union below is, by construction,
what lld sees before it reaches the static library.

WHAT IS AND IS NOT A FINDING
----------------------------
Only GLOBAL symbols (uppercase nm codes) count. A file-local symbol (`d`, `t`, `b`) cannot
collide and cannot pull the object in.

A global symbol that NOTHING REFERENCES is not a finding either, and that distinction is
load-bearing rather than a convenience. math.c defines `asm_sqrt`, `asm_sqrtf` and `x87_asin`,
and the same CMake `-D` list retargets all three to their libm counterparts — which rewrites
every CALL SITE but not the `__ASM_GLOBAL_FUNC` label, because `#name` stringifies before
macro expansion. They are defined-but-dead. No undefined reference to them exists anywhere, so
the linker never goes looking, so nothing need define them. Rather than allowlist that by hand
(an allowlist rots, and each entry is a claim nobody rechecks), this tool asks the binaries: a
symbol is "referenced" if it is undefined in ANY object of the module, or named in the
module's `.spec` export table.

FLEET ISOLATION
---------------
Generated artifacts come from THIS worktree or the tool hard-fails. A `build-*` directory is
gitignored, so a fresh worktree lacks one, and falling back to a sibling's would check this
tree's Rust against another lane's objects at another commit. A verdict that depends on which
siblings happen to exist is not a gate.

USAGE
    verify_cohab_coverage.py --arch i386            # all flipped modules
    verify_cohab_coverage.py --check                # exit 1 on any finding
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Cohab families this tool knows how to pair up. Each entry says where the flipped Rust
# modules are declared, how a module name maps to its C object, and what links ahead of it.
FAMILIES = [
    {
        "name": "msvcrt",
        # `pub mod X;` here IS the flip, so this file is also the list of what to check.
        "mod_rs": "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/mod.rs",
        "rust_dir": "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt",
        "obj_dir": "dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir",
        "obj_root": "dll/win32/msvcrt",
        "spec": "dll/win32/msvcrt/msvcrt.spec",
        "archive": "dll/win32/msvcrt/libmsvcrt_crt_cargo.a",
        # Which objects are LIVE is read from ninja, never from which files sit in a
        # directory: the DLL target's link line, and the static library's own archive
        # command for the flipped C objects. See _link_inputs.
        "link_target": "msvcrt",
        "static_lib": "dll/win32/msvcrt/libmsvcrt_static.a",
        # Sources whose edits must be reflected in the archive before a verdict means
        # anything. The crate list mirrors the CMakeLists GLOB that drives the rebuild.
        "archive_sources": [
            "sdk/lib/rust/msvcrt-crt-rust/src",
            "sdk/lib/rust/msvcrt-crt-cargo/src",
            "sdk/lib/rust/cnt-rust/src",
        ],
        # Modules whose C twin is not `<mod>.c.obj`, and the mapping is not derivable:
        # `reactos/misc.c` collides with msvcrt's own `misc.c` so its Rust module had to be
        # named differently, and `system.cpp` is not even a `.c`.
        "objects": {
            "reactos_misc": "reactos/misc.c.obj",
            "reactos_system": "reactos/system.cpp.obj",
        },
        # Rust modules with NO C original anywhere — nothing to compare them against, so
        # "no object" is the truth about them rather than a lookup that failed. Each needs a
        # reason, because the alternative reading of a missing object is a typo.
        "no_c_twin": {
            "cpp_rtti": "the RTTI half of cpp.c; cpp.c.obj carries both halves' symbols",
            "seh_i386": "i386 SEH substrate written directly in Rust, no C original",
        },
    },
]


def _nm() -> Path:
    nm = ROOT / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm"
    if not nm.is_file():
        sys.exit(f"verify_cohab_coverage: llvm-nm not found at {nm}")
    return nm


def _symbols(nm: Path, obj: Path, mode: str) -> list[tuple[str, str]]:
    out = subprocess.run(
        [str(nm), f"--{mode}-only", str(obj)], capture_output=True, text=True
    ).stdout
    rows = []
    for line in out.splitlines():
        parts = line.split()
        if not parts or parts[-1] == "@feat.00":
            continue
        rows.append((parts[-2] if len(parts) > 1 else "?", parts[-1]))
    return rows


def _spellings(name: str) -> set[str]:
    """One symbol, every spelling it can be written as across arches.

    i386 PE prepends an underscore and `__stdcall` appends `@<arg bytes>`; amd64/arm64 do
    neither. Comparing the decorated form against an undecorated one is how a real match gets
    reported as missing.
    """
    out = {name, name.split("@")[0]}
    if name.startswith("_"):
        out |= {n[1:] for n in list(out)}
    return out


def _require_fresh(build: Path, fam: dict, arch: str) -> None:
    """Refuse to judge a stale archive.

    This is not defensive padding — it is the difference between a gate and a noise source.
    Run against archives three days older than the tree, this tool reported 669 findings, all
    of them staleness: it had "proved" that errno.c, file.c and time.c — landed, booting,
    fully-flipped modules — leave hundreds of symbols to the C. A verdict computed from a
    stale binary is worse than no verdict, because it looks like the same output as a real
    one. Fail loud instead (the smoke harness refuses stale media for exactly this reason).
    """
    archive = build / fam["archive"]
    if not archive.is_file():
        sys.exit(
            f"verify_cohab_coverage: {archive} missing — cannot tell what the Rust side "
            f"defines.\n  fix: ./vibeos-cli build --arch {arch}"
        )
    newest, newest_path = 0.0, None
    for root in fam["archive_sources"]:
        for rs in (ROOT / root).rglob("*.rs"):
            mt = rs.stat().st_mtime
            if mt > newest:
                newest, newest_path = mt, rs
    if newest > archive.stat().st_mtime:
        sys.exit(
            f"verify_cohab_coverage: {archive.name} for {arch} is OLDER than "
            f"{newest_path.relative_to(ROOT)} — every finding would be staleness, not a "
            f"missing symbol.\n  fix: ./vibeos-cli build --arch {arch}"
        )


def _ninja_last_command(build: Path, target: str) -> str:
    """The last command ninja would run for `target` -- a link or an archive step."""
    try:
        res = subprocess.run(
            [
                "bash",
                str(ROOT / "tools" / "canon-run.sh"),
                "ninja",
                "-C",
                str(build),
                "-t",
                "commands",
                target,
            ],
            capture_output=True,
            text=True,
            timeout=300,
            cwd=ROOT,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        sys.exit(f"verify_cohab_coverage: could not ask ninja for {target}: {exc}")
    lines = [ln for ln in res.stdout.splitlines() if ln.strip()]
    if res.returncode != 0 or not lines:
        sys.exit(
            f"verify_cohab_coverage: `ninja -t commands {target}` failed in {build} "
            f"(exit {res.returncode}): {res.stderr.strip()[:1000]}\n"
            f"  fix: ./vibeos-cli build --arch <arch>"
        )
    return lines[-1]


def _link_inputs(build: Path, fam: dict) -> tuple[list[Path], list[Path]]:
    """(objects linked DIRECTLY into the module, members of the flipped static library).

    From ninja, not from a directory glob. Globbing `CMakeFiles/*.dir` made the verdict
    depend on which files happened to exist: when msvcrt's varargs shims moved from
    msvcrt.dir to msvcrt_rust_shims.dir, every build dir configured before the move kept
    stale copies in msvcrt.dir, and the glob counted them as live providers alongside the
    real ones -- harmless while identical, a false "provided" the moment the shim changed.
    A stale C twin in the static library's directory posed as live the same way.
    """

    def objs(cmd: str) -> list[Path]:
        return [build / t for t in cmd.split() if t.endswith(".obj")]

    return objs(_ninja_last_command(build, fam["link_target"])), objs(
        _ninja_last_command(build, fam["static_lib"])
    )


def _providers(build: Path, fam: dict, nm: Path, direct: list[Path]) -> set[str]:
    """Every global symbol that RESOLVES BEFORE the flipped static library does."""
    out: set[str] = set()
    sources = []
    archive = build / fam["archive"]
    if archive.is_file():
        sources.append(archive)
    sources += direct
    for src in sources:
        for kind, name in _symbols(nm, src, "defined"):
            if kind.isupper():
                out |= _spellings(name)
    return out


def _referenced(fam: dict, nm: Path, objects: list[Path]) -> set[str]:
    """Everything the module still needs from outside some TU, plus its export table."""
    out: set[str] = set()
    for obj in objects:
        for _kind, name in _symbols(nm, obj, "undefined"):
            out |= _spellings(name)
    spec = ROOT / fam["spec"]
    if spec.is_file():
        text = spec.read_text()
        for m in re.finditer(r"^\s*(?:@|\d+)\s+\S+(?:\s+-\S+)*\s+(\w+)", text, re.M):
            out |= {m.group(1), "_" + m.group(1)}
        # `extern` is the one directive whose EXPORT NAME and BACKING SYMBOL differ:
        # `@ extern _mbctype MSVCRT_mbctype` exports `_mbctype` from a symbol named
        # `MSVCRT_mbctype`. The pass above captures only the first name, so a data export
        # whose Rust twin is a MANGLED private static looked unreferenced and passed —
        # which is precisely the wrong-name failure this file exists to catch, in the one
        # shape it could not see. That shipped: `_mbctype` was a second, permanently-zero
        # array while `_setmbcp` wrote the Rust one.
        for m in re.finditer(r"^\s*(?:@|\d+)\s+extern(?:\s+-\S+)*\s+\w+\s+(\w+)", text, re.M):
            out |= _spellings(m.group(1))
    return out


def _flipped_modules(fam: dict) -> list[str]:
    text = (ROOT / fam["mod_rs"]).read_text()
    return [
        m.group(1)
        for line in text.splitlines()
        if not line.lstrip().startswith("//")
        for m in [re.match(r"\s*pub mod (\w+);", line)]
        if m
    ]


def check_family(fam: dict, arch: str, nm: Path) -> list[str]:
    build = ROOT / f"build-{arch}"
    obj_dir = build / fam["obj_dir"]
    if not obj_dir.is_dir():
        sys.exit(
            f"verify_cohab_coverage: {obj_dir} missing — generated objects must come from "
            f"THIS worktree, never a sibling's.\n  fix: ./vibeos-cli build --arch {arch}"
        )

    _require_fresh(build, fam, arch)
    direct, members = _link_inputs(build, fam)
    live_members = set(members)
    providers = _providers(build, fam, nm, direct)
    if not providers:
        sys.exit(
            f"verify_cohab_coverage: nothing links ahead of {fam['obj_dir']} in build-{arch} "
            f"— with no Rust archive and no module objects every symbol would look missing, "
            f"which is a false verdict, not a finding.\n  fix: ./vibeos-cli build --arch {arch}"
        )
    referenced = _referenced(fam, nm, direct + members)

    findings = []
    for mod in _flipped_modules(fam):
        if mod in fam.get("no_c_twin", {}):
            continue
        obj_rel = fam.get("objects", {}).get(mod, f"{mod}.c.obj")
        obj = obj_dir / obj_rel
        # A LIVE member of the static library, not merely a file on disk: a twin whose
        # source moved leaves its old object behind, and that one links into nothing.
        if obj not in live_members:
            # Two very different reasons an object can be absent, and skipping both alike is
            # how a flipped module gets checked against NOTHING and still reports clean. If
            # the C source exists, this arch simply does not compile it (`except_i386.c` on
            # arm64) and there is nothing to check. If it does not, the module name does not
            # name any C file in the tree — a rename or a typo — and its coverage is zero.
            src = ROOT / fam["obj_root"] / obj_rel.removesuffix(".obj")
            if src.is_file():
                continue
            findings.append(
                f"{fam['rust_dir']}/{mod}.rs: [{arch}] flipped, but no C object and no C "
                f"source at {fam['obj_root']}/{obj_rel.removesuffix('.obj')} — this module "
                f"is being checked against nothing. Add it to the family's `objects` map if "
                f"its twin is named differently, or to `no_c_twin` if it has none"
            )
            continue
        for kind, raw in _symbols(nm, obj, "defined"):
            if not kind.isupper():
                continue  # file-local: cannot collide, cannot pull the object
            cand = _spellings(raw)
            if cand & providers:
                continue
            if not (cand & referenced):
                continue  # defined-but-dead in the C too; nothing goes looking for it
            findings.append(
                f"{fam['rust_dir']}/{mod}.rs: [{arch}] nothing that links ahead of "
                f"{Path(fam['obj_dir']).name} defines `{raw}`, which {obj_rel} exports and "
                f"something references — the C object gets pulled back in and duplicates "
                f"every other symbol in this file"
            )
    return findings


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--arch", action="append", choices=["i386", "amd64", "arm64"])
    ap.add_argument("--check", action="store_true", help="exit 1 on any finding")
    args = ap.parse_args()

    nm = _nm()
    arches = args.arch or [a for a in ("i386", "amd64", "arm64") if (ROOT / f"build-{a}").is_dir()]
    if not arches:
        sys.exit(
            "verify_cohab_coverage: no build-<arch> in this worktree.\n"
            "  fix: ./vibeos-cli build --arch i386"
        )

    findings, pairs = [], 0
    for arch in arches:
        for fam in FAMILIES:
            if not (ROOT / f"build-{arch}" / fam["obj_dir"]).is_dir():
                continue
            pairs += 1
            findings += check_family(fam, arch, nm)

    for f in findings:
        print(f)
    # Always report coverage: "0 findings" over 0 pairs is indistinguishable from a clean
    # sweep, and that exact green has shipped before (verify_c_signatures).
    print(
        f"verify_cohab_coverage: {len(findings)} finding(s); "
        f"{pairs} family/arch pair(s) compared against their C objects",
        file=sys.stderr,
    )
    if args.check and not pairs:
        # As a gate stage, "I compared nothing" must not read as "I found nothing". Rename
        # a build directory, move msvcrt_static's obj dir, add an arch whose layout differs
        # — and every one of those silently retires this check while still printing green,
        # forever, with no one to notice. Refusing here is what makes the green mean
        # something (same reason _require_fresh refuses a stale archive).
        print(
            "verify_cohab_coverage: --check with 0 pairs compared — the build layout moved "
            "or no arch was built, so this run proves nothing.\n"
            "  fix: ./vibeos-cli build --arch i386",
            file=sys.stderr,
        )
        return 1
    return 1 if (findings and args.check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
