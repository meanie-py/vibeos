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


def link_line(build: Path, target: str = "ucrtbase") -> str:
    """A target's real link command (ucrtbase's by default), straight from ninja.

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
                target,
            ],
            capture_output=True,
            text=True,
            timeout=300,
            cwd=REPO,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise RefsetError(f"could not ask ninja for {target}'s link line: {exc}") from exc
    if res.returncode != 0:
        raise RefsetError(
            f"ninja -t commands {target} failed in {build} (exit {res.returncode}):\n"
            f"{res.stderr.strip()[:2000]}"
        )
    lines = [ln for ln in res.stdout.splitlines() if ln.strip()]
    if not lines:
        raise RefsetError(f"ninja produced no commands for {target} in {build}")
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


#: Our ucrtbase Rust archive, for the reference side's MISSING INTERNALS.
UCRT_RUST_ARCHIVE = Path("dll/win32/ucrtbase/libcrt_shared_cargo.a")

#: COFF/linker bookkeeping, not code: renaming these breaks the object.
_BOOKKEEPING = (".refptr.", "@feat.", ".weak.", "__imp_")

# Non-exported internals tools/payloads/ucrtdiff/ucrtdiff.c calls by plain name.
HARNESS_INTERNALS = ("__acrt_initialize",)


def undefined_symbols(objs: list[Path]) -> set[str]:
    """Every symbol the objects reference but do not themselves define."""
    nm = _tool("llvm-nm")
    res = subprocess.run(
        [nm, "--undefined-only", *(str(o) for o in objs)],
        capture_output=True,
        text=True,
        timeout=300,
    )
    if res.returncode != 0:
        raise RefsetError(f"llvm-nm --undefined-only failed: {res.stderr.strip()[:1000]}")
    out = set()
    for line in res.stdout.splitlines():
        parts = line.split()
        if len(parts) >= 2 and parts[-2] in ("U", "w", "v"):
            out.add(parts[-1])
    return out - defined_symbols(objs)


def build_internals_set(arch: str, out: Path) -> tuple[int, int]:
    """Our Rust's copies of the INTERNALS the reference set calls but no longer has.

    A takeover deletes a Microsoft file whose non-exported internals other Microsoft
    files still call: after the locale takeover, `strnicol.cpp` calls
    `__acrt_CompareStringA`, the `_LocaleUpdate` hooks, the locale statics. In the
    shipping DLL those calls land on our Rust. On the reference side nothing answered
    them, and the harness stopped linking.

    So the reference side gets those internals from our Rust archive -- and ONLY those.
    Every symbol the archive DEFINES is renamed `ours_*` except the ones the reference
    objects need, so nothing else can collide with, or shadow, Microsoft's code (or the
    DLL's plain names the harness calls). Every export the archive CALLS is renamed to
    the reference instance's `ref_*` copy where Microsoft's C defines one, so our
    internals run inside the reference CRT instance, not across into the DLL's.

    During a takeover check the deleted C is back on the link line, the reference set
    defines those internals itself, and nothing here is needed: the archive is linked
    as an archive, so no member is pulled.
    """
    build = REPO / f"build-{arch}"
    src = build / UCRT_RUST_ARCHIVE
    if not src.is_file():
        raise RefsetError(f"{src} not found -- fix: ./vibeos-cli build --arch {arch}")
    ref = out.parent / "libucrt_ref.a"
    if not ref.is_file():
        raise RefsetError(f"{ref} not found -- build the ucrt side first")

    ref_defined = defined_symbols([ref])
    needed_by_ref = undefined_symbols([ref])
    rust_defined = defined_symbols([src])
    rust_undefined = undefined_symbols([src])

    # Exports the reference set defines, as `ref_*`: settled per arch exactly as
    # rename_rules does, by reading the renamed names back out of the reference set.
    ref_exports = {d[len("ref_") :] for d in ref_defined if d.startswith("ref_")}
    ref_exports |= {"_" + d[len("_ref_") :] for d in ref_defined if d.startswith("_ref_")}

    def bookkeeping(sym: str) -> bool:
        return sym.startswith(_BOOKKEEPING) or sym.startswith("$")

    # Function EXPORTS stay out even when the reference set calls them by plain name
    # (an earlier takeover removed Microsoft's copy): the import library resolves
    # those to the shipping DLL, as it always has, and a local definition here would
    # also shadow the harness's own plain-name calls. DATA exports are the exception
    # -- the import library carries only their `__imp_` slot, so a plain data
    # reference from Microsoft's C has nothing else to bind to.
    from export_census import parse_spec

    exports = set(parse_spec(SPEC, arch))
    data_exports = {
        line.split()[-1]
        for line in SPEC.read_text(errors="replace").splitlines()
        if line.startswith("@") and " extern " in f" {line} "
    }
    underscored = sum(1 for e in exports if f"_{e}" in rust_defined) > sum(
        1 for e in exports if e in rust_defined
    )
    dec = "_" if underscored else ""
    function_exports = {f"{dec}{e}" for e in exports - data_exports}

    needed = {
        s
        for s in (needed_by_ref & rust_defined)
        if not bookkeeping(s) and s not in function_exports
    }
    # Internals the HARNESS calls by plain name to drive the reference instance
    # (`init_ref_crt`'s `__acrt_initialize`). Once their C is deleted no reference
    # object asks for them, so the rule above would rename them away -- but the
    # reference instance still has to be started, now by our copy.
    needed |= {
        f"{dec}{s}"
        for s in HARNESS_INTERNALS
        if f"{dec}{s}" in rust_defined and f"{dec}{s}" not in ref_defined
    }
    rules = []
    for sym in sorted(rust_defined):
        if sym in needed or bookkeeping(sym):
            continue
        rules.append((sym, f"ours_{sym}"))
    for sym in sorted(rust_undefined):
        if bookkeeping(sym) or sym not in ref_exports:
            continue
        # i386 decorates: `_strlen` -> `_ref_strlen`.
        to = (
            f"_ref_{sym[1:]}"
            if sym.startswith("_") and f"_ref_{sym[1:]}" in ref_defined
            else f"ref_{sym}"
        )
        if to in ref_defined:
            rules.append((sym, to))

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

    # Function exports a takeover REMOVED from the reference set. Left alone, Microsoft's
    # plain-name calls to them bind through the import library to the shipping DLL: after
    # the invalid_parameter takeover, all 174 of the reference side's
    # `_invalid_parameter_noinfo` calls reported to the DLL's handler, so every row's iph
    # count was charged to the wrong side. And the harness's own `ref_X` calls to one
    # (`ref__set_invalid_parameter_handler` to install the counter) stopped linking. Both
    # belong to the reference INSTANCE, which for a taken-over export is now our Rust
    # copy: rebind the reference objects' calls to `ours_X`, and give the harness
    # `ref_X` as an alias of it (build.sh reads the alias file). None of these exports is
    # graded -- the harness's UCRTDIFF_REF_HAS_X guards read the reference archive, which
    # still lacks them.
    ours_defined = {b for _, b in rules}
    reroute = sorted(
        s
        for s in needed_by_ref
        if s in function_exports and s not in ref_defined and f"ours_{s}" in ours_defined
    )
    if reroute:
        ref_map = out.parent / f".{ref.stem}-reroute.txt"
        ref_map.write_text("".join(f"{s} ours_{s}\n" for s in reroute))
        tmp = ref.with_suffix(".rerouted.a")
        res = subprocess.run(
            [objcopy, f"--redefine-syms={ref_map}", str(ref), str(tmp)],
            capture_output=True,
            text=True,
            timeout=600,
        )
        if res.returncode != 0:
            raise RefsetError(f"llvm-objcopy failed on {ref}: {res.stderr.strip()[:500]}")
        tmp.replace(ref)
    aliases = sorted(
        (f"{dec}ref_{e}", f"ours_{dec}{e}")
        for e in exports - data_exports
        if f"{dec}{e}" not in ref_exports and f"ours_{dec}{e}" in ours_defined
    )
    (out.parent / f"{out.stem}.refalias").write_text("".join(f"{a}={b}\n" for a, b in aliases))
    return len(needed), len(rules)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    ap.add_argument(
        "--side",
        default="ucrt",
        choices=["ucrt", "msvcrt-rust", "ucrt-internals"],
        help="ucrt: genuine Microsoft C -> ref_* (default). "
        "msvcrt-rust: our existing msvcrt Rust bodies -> rust_*. "
        "ucrt-internals: our ucrtbase Rust, only for internals the reference set lacks",
    )
    ap.add_argument(
        "--out", type=Path, help="output archive (default: build-<arch>/libucrt_<side>.a)"
    )
    args = ap.parse_args(argv)

    default_name = {
        "ucrt": "libucrt_ref.a",
        "msvcrt-rust": "libucrt_rust.a",
        "ucrt-internals": "libucrt_ours.a",
    }[args.side]
    out = args.out or (REPO / f"build-{args.arch}" / default_name)
    try:
        if args.side == "ucrt":
            nobj, nsym = build_refset(args.arch, out)
        elif args.side == "ucrt-internals":
            nneed, nsym = build_internals_set(args.arch, out)
            print(
                f"ucrt-tier2-refset {args.arch} [ucrt-internals]: {nneed} internal(s) the "
                f"reference set lacks, {nsym} symbols renamed -> {out}"
            )
            return 0
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
