#!/usr/bin/env python3
"""Build the Tier-2 ucrt difftest REFERENCE SET: genuine Microsoft C, exports renamed ref_*.

Tier 1 (`sdk/lib/rust/ucrt-difftest`) grades our Rust against the genuine UCRT C on the
HOST, and can only reach TUs that include nothing but public UCRT headers. Everything
interesting reaches `corecrt_internal.h`, which includes <windows.h>, and the host's
`long` is 64 bits where Windows' is 32 -- so ~188 public declarations (`strtol`, `atol`,
`__timezone`, ...) cannot be graded there at all, silently.

Tier 2 grades IN THE GUEST instead, against binaries the normal build already produced:

    test exe = [ ucrt objects, exports renamed ref_* ]  <- genuine Microsoft C
             + [ ucrtbase import lib ]                  <- OUR shipping DLL

`ref_strtol` is Microsoft's body; `strtol` resolves through the import to whatever
ucrtbase.dll actually ships. Nothing is recompiled, so there are no header hacks and no
ABI mismatch -- both sides are Windows.

Two properties make this work, and both are load-bearing:

  * `llvm-objcopy --redefine-syms` renames UNDEFINED REFERENCES as well as definitions,
    so the reference set calls its own renamed copies and stays isolated from the DLL.
  * Only EXPORTS are renamed. Internals (`__acrt_locale_changed_data`, ...) keep their
    names and resolve among the privately-linked objects; nothing collides with the
    import because ucrtbase does not export its internals.

WHY THE OBJECT LIST COMES FROM THE LINK LINE, NOT A GLOB: `ucrt.dir` also holds objects
that are no longer built into anything. `sdk/lib/ucrt_reactos/` supplies four files
(matherr, GetStringTypeW, MultiByteToWideChar, WideCharToMultiByte) that REPLACE their
`sdk/lib/ucrt/` namesakes, and ninja never deletes the objects of the copies it stopped
using. Globbing the directory picks up both and the reference set fails to link with
duplicate symbols. The set that makes the shipping DLL is the set the DLL links.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools"))

SPEC = REPO / "dll" / "win32" / "ucrtbase" / "ucrtbase.spec"
#: Object directory of the `ucrt` target -- the genuine Microsoft C. Objects OUTSIDE it on
#: ucrtbase's link line (ucrtbase's own printf.c/stubs.c, and every .a) are ours and are
#: deliberately excluded: the reference side must be Microsoft's code and nothing else.
UCRT_OBJ_MARKER = "CMakeFiles/ucrt.dir/"


class RefsetError(RuntimeError):
    """Raised instead of degrading to a worse reference set."""


def _tool(name: str) -> str:
    """Pinned llvm-mingw tool, else PATH. Hard-fails rather than silently skipping a step."""
    pinned = REPO / ".build-tools" / "llvm-mingw" / "bin" / name
    if pinned.is_file():
        return str(pinned)
    found = shutil.which(name)
    if not found:
        raise RefsetError(
            f"{name} not found (looked in {pinned} and on PATH). The reference set cannot "
            f"be built without it; refusing to emit a partial one."
        )
    return found


def link_line(build: Path) -> str:
    """ucrtbase's real link command, straight from ninja.

    This is the authority for which objects make the shipping DLL. Reconstructing it by
    hand -- globbing the object directory -- inherits stale objects ninja left behind.
    """
    if not build.is_dir():
        raise RefsetError(f"{build} does not exist -- fix: ./vibeos-cli build --arch <arch>")
    try:
        res = subprocess.run(
            [
                "bash",
                str(REPO / "tools" / "canon-run.sh"),
                "ninja",
                "-C",
                str(build),
                "-t",
                "commands",
                "ucrtbase",
            ],
            capture_output=True,
            text=True,
            timeout=300,
            cwd=REPO,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise RefsetError(f"could not ask ninja for ucrtbase's link line: {exc}") from exc
    if res.returncode != 0:
        raise RefsetError(
            f"ninja -t commands ucrtbase failed in {build} (exit {res.returncode}):\n"
            f"{res.stderr.strip()[:2000]}"
        )
    lines = [ln for ln in res.stdout.splitlines() if ln.strip()]
    if not lines:
        raise RefsetError(f"ninja produced no commands for ucrtbase in {build}")
    return lines[-1]


def ucrt_objects(build: Path) -> list[Path]:
    """The genuine-Microsoft-C objects on ucrtbase's link line, in link order."""
    objs: list[Path] = []
    seen: set[str] = set()
    for tok in link_line(build).split():
        if tok.endswith(".obj") and UCRT_OBJ_MARKER in tok and tok not in seen:
            seen.add(tok)
            p = build / tok if not Path(tok).is_absolute() else Path(tok)
            if not p.is_file():
                raise RefsetError(f"link line names {tok} but it is not on disk under {build}")
            objs.append(p)
    if not objs:
        raise RefsetError(
            f"no {UCRT_OBJ_MARKER} objects on ucrtbase's link line in {build} -- the target "
            f"layout changed and this tool is now looking in the wrong place"
        )
    return objs


def defined_symbols(objs: list[Path]) -> set[str]:
    """Every globally-defined symbol across the reference objects."""
    nm = _tool("llvm-nm")
    res = subprocess.run(
        [nm, "--defined-only", *(str(o) for o in objs)],
        capture_output=True,
        text=True,
        timeout=300,
    )
    if res.returncode != 0:
        raise RefsetError(f"llvm-nm failed over the ucrt objects: {res.stderr.strip()[:1000]}")
    out = set()
    for line in res.stdout.splitlines():
        parts = line.split()
        if len(parts) >= 3:
            out.add(parts[2])
    return out


def rename_rules(arch: str, defined: set[str], prefix: str = "ref_") -> list[tuple[str, str]]:
    """(from, to) for every export genuine C actually DEFINES, decoration settled per arch.

    An export that genuine C does not define is skipped: it comes from vcruntime, crtmath,
    ucrtbase's own stubs or our Rust, and there is no Microsoft body here to grade against.

    THE DECORATION IS SETTLED ONCE FOR THE WHOLE ARCH, never per symbol. i386 decorates
    cdecl with a leading underscore and the other arches do not -- but deciding per export
    by trying `_name` and falling back to `name` silently corrupts the map, because the two
    conventions OVERLAP: `tolower` and `_tolower` are both exports, so on amd64 (undecorated)
    the symbol `_tolower` belongs to export `_tolower`, and a per-symbol probe hands it to
    export `tolower` instead -- stealing one export's body and leaving the other unrenamed.
    Nine amd64 exports and nine arm64 exports land in that overlap. Counting which
    convention the objects actually use, then applying only that one, cannot collide.
    """
    from export_census import parse_spec

    exports = sorted(set(parse_spec(SPEC, arch)))
    if not exports:
        raise RefsetError(f"{SPEC} declares no exports for {arch}")

    n_under = sum(1 for e in exports if f"_{e}" in defined)
    n_plain = sum(1 for e in exports if e in defined)
    if max(n_under, n_plain) == 0:
        raise RefsetError(
            f"not one of the {len(exports)} {arch} exports is defined by the ucrt objects -- "
            f"the object set is wrong; refusing to emit an empty reference set"
        )
    underscored = n_under > n_plain
    dec = "_" if underscored else ""

    rules = [(f"{dec}{e}", f"{dec}{prefix}{e}") for e in exports if f"{dec}{e}" in defined]
    # The map is a rename, so two exports must never claim one symbol, and one export must
    # never claim two. objcopy would silently apply whichever rule it read last.
    froms = [a for a, _ in rules]
    if len(set(froms)) != len(froms):
        dupes = sorted({f for f in froms if froms.count(f) > 1})[:10]
        raise RefsetError(f"{arch}: rename map is ambiguous, symbols claimed twice: {dupes}")
    return rules


def build_refset(arch: str, out: Path) -> tuple[int, int]:
    """Rename and archive. Returns (objects, renamed exports)."""
    build = REPO / f"build-{arch}"
    objs = ucrt_objects(build)
    rules = rename_rules(arch, defined_symbols(objs))

    objcopy, ar = _tool("llvm-objcopy"), _tool("llvm-ar")
    work = out.parent / f".{out.stem}-objs"
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)

    mapfile = work / "redefine.txt"
    mapfile.write_text("".join(f"{a} {b}\n" for a, b in rules))

    renamed: list[Path] = []
    for i, o in enumerate(objs):
        # Index-prefixed: link order is preserved and same-named objects from different
        # source directories cannot collide in one flat directory.
        dst = work / f"{i:04d}_{o.name}"
        res = subprocess.run(
            [objcopy, f"--redefine-syms={mapfile}", str(o), str(dst)],
            capture_output=True,
            text=True,
            timeout=120,
        )
        if res.returncode != 0:
            raise RefsetError(f"llvm-objcopy failed on {o}: {res.stderr.strip()[:500]}")
        renamed.append(dst)

    out.parent.mkdir(parents=True, exist_ok=True)
    if out.exists():
        out.unlink()
    res = subprocess.run(
        [ar, "crs", str(out), *(str(p) for p in renamed)],
        capture_output=True,
        text=True,
        timeout=300,
    )
    if res.returncode != 0:
        raise RefsetError(f"llvm-ar failed building {out}: {res.stderr.strip()[:500]}")
    return len(objs), len(rules)


#: The msvcrt Rust archive. 799 of the 999 exports ucrtbase still answers with
#: Microsoft's C already have a body HERE -- written and graded against MSVCRT --
#: but it never reaches ucrtbase's link line, deliberately: letting it through
#: under --allow-multiple-definition would silently replace genuine UCRT bodies
#: with MSVCRT-semantics ones (ucrtbase is _MSVCR_VER 140, msvcrt is 0).
#:
#: Renaming those bodies to rust_* gives the harness a THIRD side, so it can ask
#: "is our existing Rust body UCRT-correct?" without flipping anything and
#: without deleting the C that is its only oracle.
MSVCRT_RUST_ARCHIVE = Path("dll/win32/msvcrt/libmsvcrt_crt_cargo.a")


def build_rustset(arch: str, out: Path) -> tuple[int, int]:
    """Rename the msvcrt Rust archive's ucrtbase exports to rust_*.

    Unlike the ucrt side this renames the ARCHIVE in place rather than object by
    object -- llvm-objcopy handles an archive directly, and there is no stale-object
    hazard here because the archive is a single build product.
    """
    build = REPO / f"build-{arch}"
    src = build / MSVCRT_RUST_ARCHIVE
    if not src.is_file():
        raise RefsetError(f"{src} not found -- fix: ./vibeos-cli build --arch {arch}")
    mine = defined_symbols([src])
    rules = rename_rules(arch, mine, prefix="rust_")

    # The two private sides must share NOTHING. Both archives carry internals that
    # are not ucrtbase exports, so no export-driven rule covers them -- and where the
    # names coincide (`__invoke_matherr`, `__p___mb_cur_max`) the link silently keeps
    # ONE under --allow-multiple-definition. The reference side would then call a Rust
    # helper and the oracle would be grading our code against itself. Renaming every
    # collision on the RUST side keeps Microsoft's objects untouched, which is what the
    # reference must be.
    ref = out.parent / "libucrt_ref.a"
    if ref.is_file():
        claimed = {a for a, _ in rules}
        collisions = sorted((mine & defined_symbols([ref])) - claimed)
        # COFF/linker bookkeeping, not code: renaming these breaks the object.
        collisions = [c for c in collisions if not c.startswith((".refptr.", "@feat."))]
        rules += [(c, f"rust_collide_{c.lstrip('_')}") for c in collisions]

    objcopy = _tool("llvm-objcopy")
    out.parent.mkdir(parents=True, exist_ok=True)
    mapfile = out.parent / f".{out.stem}-redefine.txt"
    mapfile.write_text("".join(f"{a} {b}\n" for a, b in rules))
    if out.exists():
        out.unlink()
    res = subprocess.run(
        [objcopy, f"--redefine-syms={mapfile}", str(src), str(out)],
        capture_output=True,
        text=True,
        timeout=600,
    )
    if res.returncode != 0:
        raise RefsetError(f"llvm-objcopy failed on {src}: {res.stderr.strip()[:500]}")
    return 1, len(rules)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    ap.add_argument(
        "--side",
        default="ucrt",
        choices=["ucrt", "msvcrt-rust"],
        help="ucrt: genuine Microsoft C -> ref_* (default). "
        "msvcrt-rust: our existing msvcrt Rust bodies -> rust_*",
    )
    ap.add_argument(
        "--out", type=Path, help="output archive (default: build-<arch>/libucrt_<side>.a)"
    )
    args = ap.parse_args(argv)

    default_name = {"ucrt": "libucrt_ref.a", "msvcrt-rust": "libucrt_rust.a"}[args.side]
    out = args.out or (REPO / f"build-{args.arch}" / default_name)
    try:
        if args.side == "ucrt":
            nobj, nsym = build_refset(args.arch, out)
        else:
            nobj, nsym = build_rustset(args.arch, out)
    except RefsetError as exc:
        print(f"ucrt-tier2-refset: FAIL ({args.arch}): {exc}", file=sys.stderr)
        return 1
    prefix = "ref_" if args.side == "ucrt" else "rust_"
    print(
        f"ucrt-tier2-refset {args.arch} [{args.side}]: {nobj} object(s), "
        f"{nsym} exports renamed {prefix}* -> {out}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
