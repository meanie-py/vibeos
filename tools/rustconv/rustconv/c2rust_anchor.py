"""c2rust behavior anchors — feed a compiler-exact transpilation into the prompt.

The mechanical LLM pass silently STUBS/drops hard function bodies and turns
function-like macros (e.g. `RosSymAllocMem`, `DPRINT`) into phantom externs. c2rust
transpiles compiler input mechanically, providing a structural checklist the LLM
must preserve. Its output still needs coverage and semantic checks. We don't ship
c2rust's raw output (it's ugly: libc casts, generated
names, `.expect("non-null function pointer")`); we hand it to the model as an anchor and
let it restyle the surface while preserving every statement/branch/call.

This module produces, for one C file, a `{function_name: rust_body}` map. Library
callers can request best-effort mode; the CLI uses strict mode and checks every
selected function before payment. An anchor guides translation; it does not prove
that a model preserved the anchored behavior.

Validated recipe (2026-06-07): c2rust chokes on a bare compile_commands.json path and
on the mingw cross `command` (multi-job), but the *direct-source* form works:
    c2rust transpile --output-dir DIR SOURCE.c -- <--target + -D + -I from the db entry>
Output lands at DIR/src/<stem>.rs.
"""

from __future__ import annotations

import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from .config import REACTOS_ROOT
from .function_coverage import rust_definition_bodies
from .wide_seal import _render_units

# Library preference when no arch is specified. The CLI always pins its requested arch.
_DB_BUILD_DIRS = ["build-amd64", "build-arm64", "build-i386"]

# Builds run at a CANONICAL bind-mount path (tools/canon-run.sh, default
# /tmp/vibeos) so ccache/sccache keys match across worktrees. Every path in
# compile_commands.json is therefore spelled /tmp/vibeos/... — a path that only
# resolves INSIDE a live build's mount namespace. c2rust runs outside it, so
# every consumed path must be mapped back onto the real checkout. The mount is a
# bind of exactly this tree, so the rewrite is a plain prefix swap.
_CANON_ROOT = os.environ.get("VIBEOS_CANON", "/tmp/vibeos").rstrip("/")


def _decanon(s: str) -> str:
    """Map canonical-build paths back onto this checkout (no-op if absent).

    Substring, not prefix: the flags arrive attached (`-I/tmp/vibeos/sdk/include`),
    so anchoring at the start of the token would miss every include path.
    """
    if not _CANON_ROOT:
        return s
    return s.replace(_CANON_ROOT, str(REACTOS_ROOT))


def _find_db_entries(c_file: Path, arch: str | None = None) -> list[tuple[dict, Path]]:
    """Every (entry, build_dir) for c_file, pinned arch first, then the other arches.

    More than one is useful because a c2rust failure can be a property of the ARCH's
    flags rather than of the file: c2rust 0.22.1 panics in convert_type.rs whenever a
    pointer is formed to a function type carrying a calling-convention attribute, and
    on i386 `WINAPI` IS such an attribute while on amd64 it is nothing. msvcrt's
    except.c fails on the i386 db and transpiles cleanly on the amd64 one.
    """
    cf = str(c_file.resolve())
    order = (
        _DB_BUILD_DIRS
        if arch is None
        else ([f"build-{arch}"] + [b for b in _DB_BUILD_DIRS if b != f"build-{arch}"])
    )
    out: list[tuple[dict, Path]] = []
    for bd in order:
        db_path = REACTOS_ROOT / bd / "compile_commands.json"
        if not db_path.exists():
            continue
        try:
            with open(db_path) as f:
                db = json.load(f)
        except Exception:
            continue
        for e in db:
            if str(Path(_decanon(e["file"])).resolve()) == cf:
                out.append((e, REACTOS_ROOT / bd))
                break
    if not out:
        # A FLIPPED .c has no db entry of its own. Borrow exactly what the call seal borrows
        # (macro_authority): a surviving sibling TU's flags with this file substituted in, or,
        # for ntoskrnl, which compiles no C through CMake at all, the command synthesized from
        # the derived kernel profile. win32k's job.c (never built as C here) is the first case.
        from .macro_authority import _derived_kernel_entry, _sibling_target_entry

        for bd in order:
            arch = bd.removeprefix("build-")
            argv = _sibling_target_entry(REACTOS_ROOT, arch, c_file) or _derived_kernel_entry(
                REACTOS_ROOT, arch, c_file
            )
            if argv is not None:
                entry = {"file": cf, "arguments": argv, "directory": str(REACTOS_ROOT / bd)}
                out.append((entry, REACTOS_ROOT / bd))
    return out


def _find_db_entry(c_file: Path, arch: str | None = None) -> tuple[dict, Path] | None:
    """Return (entry, build_dir) for c_file from the first build db that has it.

    A PINNED arch is honoured exactly — no sibling build is consulted. Only
    `_transpile_text` falls back, and only after the pinned arch's db has actually
    failed, and it says so when it does.
    """
    cf = str(c_file.resolve())
    for bd in [f"build-{arch}"] if arch is not None else _DB_BUILD_DIRS:
        db_path = REACTOS_ROOT / bd / "compile_commands.json"
        if not db_path.exists():
            continue
        try:
            with open(db_path) as f:
                db = json.load(f)
        except Exception:
            continue
        for e in db:
            if str(Path(_decanon(e["file"])).resolve()) == cf:
                return e, (REACTOS_ROOT / bd)
    return None


#: Function ATTRIBUTES that c2rust 0.22.1's Clang AST exporter cannot map. Each one
#: makes the exporter drop the TYPE of the declaration it sits on -- "Missing type
#: <id> for node: AstNode { tag: TagFunctionDecl ... }" followed by "Exported Clang
#: AST was invalid" -- and one such decl anywhere in the include closure kills the
#: whole transpile. They come from msvcrt's PRIVATE headers, so every msvcrt TU
#: inherits them and the failure looked like "c2rust refuses the SDK include
#: closure". It is not the closure: it is exactly these three macros.
#:
#: Measured 2026-09-13 on dll/win32/msvcrt: 32 invalid-AST sites -> 26 with
#: __WINE_MALLOC neutralised -> 0 with the two format attrs as well, and the TU
#: transpiled. Across the module: 2/37 before, 35/37 after. The two stragglers
#: (except.c, cpp.c) fail on the SEPARATE, already-documented `__label__` class --
#: they use wine's __TRY/__EXCEPT_CTX, which under _USE_DUMMY_PSEH=1 opens a block
#: with a GCC local-label declaration that c2rust does not support.
#:
#: Neutralising them is safe for the ANCHOR because they carry no semantics we
#: translate: `malloc` and `format(printf/scanf)` are optimiser and -Wformat hints,
#: not part of the ABI or the body. The real build is untouched -- these -D only
#: reach c2rust. They work because the macros are `#ifndef`-guarded in
#: dll/win32/msvcrt/include/corecrt.h (that guard is the one C change this needed).
_INTRINSIC_PREFIX = "__rustconv_intrin_"


def _unprefix_intrinsics(text: str) -> str:
    """Undo the _ATTR_NEUTRALIZERS intrinsic renames in c2rust's output."""
    return text.replace(_INTRINSIC_PREFIX, "")


_ATTR_NEUTRALIZERS = (
    "-D__WINE_MALLOC=",
    "-D__WINE_CRT_PRINTF_ATTR(a,b)=",
    "-D__WINE_CRT_SCANF_ATTR(a,b)=",
    # GCC LOCAL LABELS -- the second, separate blocker. c2rust cannot export a
    # `__label__` declaration: it becomes a TagDeclStmt whose only child is missing,
    # clang's AST is rejected, and the transpile dies with a CDeclId panic. Under
    # _USE_DUMMY_PSEH (the tree's global C compile mode) sdk/lib/pseh/include/pseh/pseh2.h
    # opens every _SEH2_TRY / __TRY block with one. That branch is INERT -- the except
    # arm is `if (_SEH2_Volatile0 || (0 && filter))`, never entered -- so the label only
    # ever serves _SEH2_LEAVE, and these label-free replacements are behaviour-identical
    # for the anchor. A TU that really uses _SEH2_LEAVE gets "use of undeclared label"
    # and fails LOUDLY rather than being anchored against control flow it does not have.
    # The guard that lets these win is the `#ifndef _SEH2_TRY` in pseh2.h.
    #
    # MSVC INTERLOCKED / BIT-SCAN INTRINSICS -- the third blocker. Under -fms-extensions clang
    # treats `_InterlockedOr`, `_BitScanReverse` & co. as language builtins, c2rust 0.22 has no
    # lowering for them, and it DROPS every function that calls one ("Unimplemented builtin
    # _InterlockedOr"). That made five of ntoskrnl/ps/job.c's functions unanchorable, and
    # most of the new pciidex/atapi stack. Renaming the intrinsic turns its declaration into an
    # ordinary function. The new name must be one NO header macro maps back: renaming to the
    # plain `InterlockedOr` worked for job.c, which calls the plain name, but atapi calls
    # `_InterlockedOr` directly, and `_InterlockedOr -> InterlockedOr -> _InterlockedOr`
    # (xdk's `#define InterlockedOr _InterlockedOr`) stops back at the builtin. So it gets a
    # private prefix, and `_unprefix_intrinsics` turns the anchor back into the plain call the
    # Rust uses (ntkrnl_sys::interlocked).
    *(
        f"-D_Interlocked{op}{width}={_INTRINSIC_PREFIX}Interlocked{op}{width}"
        for op in (
            "Exchange",
            "ExchangeAdd",
            "CompareExchange",
            "CompareExchangePointer",
            "ExchangePointer",
            "Increment",
            "Decrement",
            "Or",
            "And",
            "Xor",
        )
        for width in ("", "8", "16", "64")
    ),
    *(
        f"-D_BitScan{d}{w}={_INTRINSIC_PREFIX}BitScan{d}{w}"
        for d in ("Forward", "Reverse")
        for w in ("", "64")
    ),
    # The assertion trap and the barrier, which every checked-build NT_VERIFY/NT_ASSERT and
    # every DMA descriptor publish reach: `DbgRaiseAssertionFailure` is `__int2c` on x86 and
    # `__break(0xf001)` on arm64, and arm64's KeMemoryBarrier-style publishes use `__dsb`.
    # These keep their own spelling after the round trip.
    *(f"-D{name}={_INTRINSIC_PREFIX}{name}" for name in ("__int2c", "__break", "__dsb")),
)

#: PSEH overrides, tried in order. Both are behaviour-identical to the inert dummy
#: branch; they differ only in which C they FAIL to compile, and each one's failure is
#: loud.
#:
#: 1. `labelled` keeps the `__seh2_scope_end__:` labels and drops only the `__label__`
#:    DECLARATIONS, so the labels become function-scoped. `_SEH2_LEAVE`'s goto still has
#:    a target. It compiles exactly when the function has ONE _SEH2_TRY chain; a second
#:    one in the same function is a duplicate-label error.
#: 2. `labelfree` drops the labels too. It handles any number of TRY blocks, and a
#:    `_SEH2_LEAVE` anywhere in the TU is `goto` to a label that does not exist —
#:    c2rust says "Goto target label not found" rather than anchoring against control
#:    flow the C does not have.
#:
#: Trying (1) first and falling back to (2) means a file is only rejected when NEITHER
#: is legal, which so far means "several TRY blocks in one function AND a LEAVE".
#: Measured 2026-09-19 over the 54 SEH-using TUs in the i386 compile db: 50 anchor.
_PSEH_VARIANTS = (
    (
        "-D_SEH2_TRY={",
        "-D_SEH2_FINALLY=__seh2_scope_end__:; } if (1) {",
        "-D_SEH2_EXCEPT(...)=__seh2_scope_end__:; } if (_SEH2_Volatile0 || (0 && (__VA_ARGS__))) {",
        "-D_SEH2_END=__seh2_scope_end__:; }",
    ),
    (
        "-D_SEH2_TRY={",
        "-D_SEH2_FINALLY=} if (1) {",
        "-D_SEH2_EXCEPT(...)=} if (_SEH2_Volatile0 || (0 && (__VA_ARGS__))) {",
        "-D_SEH2_END=}",
    ),
)


def _extract_clang_flags(entry: dict) -> list[str]:
    """Pull --target, -D, -I, -isystem/-include from the db command; drop the
    multi-job triggers (-o/-c/-M*) that make clang tooling reject the command."""
    toks = entry.get("arguments") or shlex.split(entry.get("command", ""))
    keep: list[str] = []
    with_value = {
        "-D",
        "-U",
        "-I",
        "-isystem",
        "-include",
        "-imacros",
        "-idirafter",
        "-iquote",
        "-isysroot",
        "--sysroot",
        "--target",
        "-target",
        "-x",
    }
    semantic = {
        "-nostdinc",
        "-nostdinc++",
        "-fms-extensions",
        "-fms-compatibility",
        "-fshort-wchar",
        "-fshort-enums",
        "-funsigned-char",
        "-fsigned-char",
        "-mms-bitfields",
        "-mlong-double-64",
        "-ffreestanding",
    }
    i = 1
    while i < len(toks):
        token = toks[i]
        if token in with_value:
            if i + 1 >= len(toks):
                raise C2RustError(f"compile command has no argument after {token}")
            keep.extend((token, _decanon(toks[i + 1])))
            i += 2
            continue
        if token in semantic or token.startswith(
            (
                "-D",
                "-U",
                "-I",
                # Attached forms too: the arm64 db spells `-isystem/…/sdk/include/crt`, and
                # dropping it under `-nostdinc` left <stdio.h> unresolvable on that arch.
                "-isystem",
                "-idirafter",
                "-iquote",
                "--target=",
                "--sysroot=",
                "-std=",
                "-fpack-struct=",
                "-fms-compatibility-version=",
            )
        ):
            keep.append(_decanon(token))
        i += 1
    return keep


def _targets_i386(flags: list[str]) -> bool:
    """True when this compile command builds 32-bit x86, where __stdcall/__thiscall are
    real ABI and must never be erased."""
    for i, tok in enumerate(flags):
        val = (
            tok.split("=", 1)[1]
            if tok.startswith("--target=")
            else (flags[i + 1] if tok in ("--target", "-target") and i + 1 < len(flags) else "")
        )
        if val and re.match(r"^(i[3-6]86|x86)-", val):
            return True
    return False


def _parse_functions(rs_text: str) -> dict[str, str]:
    """Extract actual bodies; foreign declarations cannot borrow the next item's brace."""
    return rust_definition_bodies(rs_text)


class C2RustError(RuntimeError):
    """c2rust was explicitly requested (--c2rust / strict=True) but could not produce
    an anchor transpilation. Raised instead of silently degrading to a pure-LLM
    translation: a requested-but-failed compiler anchor is a hard error, never a
    dumb fallback (see AGENTS.md — "Fail loud, never silently degrade")."""


class CompiledOut(C2RustError):
    """Required functions that NO built arch's preprocessor keeps.

    Not a c2rust failure: c2rust ran and, with unused functions preserved, lists every
    definition the compiler saw. A name absent on every arch sits under a condition the
    build never sets -- partlist.c's `DumpPartitionTable` under `#ifdef
    DUMP_PARTITION_TABLE`, which only diskpart.h mentions, commented out. The caller drops
    these from the worklist (they are in no object we link) and says so."""

    def __init__(self, c_file: Path, names: set[str]):
        self.names = names
        super().__init__(f"{c_file}: compiled out on every built arch: " + ", ".join(sorted(names)))


# Transpile is the slow step (~seconds); a file's anchors AND preamble both need it.
# Cache the raw .rs text per (path, mtime) so the second consumer is free.
_TRANSPILE_CACHE: dict[tuple[str, float], str | None] = {}

# The lines in c2rust/clang output that actually say WHY it gave up. Ordered by how
# specific they are — a panic message beats a generic "error:".
_C2RUST_REASON_PATTERNS = (
    # c2rust ICE. NB the real line is `thread 'main' (3123720) panicked at <path>:L:C:` —
    # the pid in parens sits between the quoted name and "panicked", so do NOT anchor
    # ".*panicked" straight onto the closing quote.
    re.compile(r"^thread\b.*panicked at .*$", re.M),
    re.compile(r"^.*\bUnsupported\b.*$", re.M),  # explicit "unsupported node/type"
    re.compile(r"^.*\bfatal error: .*$", re.M),  # missing header, etc.
    re.compile(r"^.*\berror: .*$", re.M),  # clang parse error in OUR flags
)


def _c2rust_reason(stderr: str, stdout: str) -> str:
    """Pull the most specific explanation out of c2rust's output.

    Without this the caller could only say "(AST unsupported)" — a guess. c2rust fails for
    at least three different reasons (an internal panic, a genuinely unsupported construct,
    and a clang parse error caused by the flags WE pass it), and they need different fixes.
    Reporting them as one opaque message is exactly the silent-degrade smell the strict mode
    exists to avoid, one level up.
    """
    blob = f"{stderr}\n{stdout}"
    for pat in _C2RUST_REASON_PATTERNS:
        m = pat.search(blob)
        if m:
            line = m.group(0).strip()
            if "panicked at" in line:
                following = blob[m.end() :].strip().splitlines()
                if following and not following[0].startswith("note:"):
                    line += " " + following[0].strip()
            return line[:600]
    tail = [ln for ln in blob.splitlines() if ln.strip()][-1:]
    return tail[0].strip()[:300] if tail else "(no output — c2rust said nothing)"


def _transpile_text(
    c_file: Path,
    timeout: int = 180,
    verbose: bool = True,
    strict: bool = False,
    arch: str | None = None,
) -> str | None:
    """Run c2rust on c_file and return the raw transpiled .rs text.

    strict=False (library default): best-effort — returns None on any failure.
    strict=True (what --c2rust passes): raises C2RustError on ANY failure (missing
      binary, no compile_commands entry, timeout, or no .rs produced). When the caller
      explicitly asked for the compiler anchor, silently falling back to a worse
      pure-LLM translation is never acceptable — fail loud so the cause gets fixed.
    Cached: anchors and preamble extraction share one transpile."""

    def _fail(msg: str) -> None:
        if strict:
            raise C2RustError(msg)
        if verbose:
            print(f"  [c2rust] {msg} — skipping", file=sys.stderr)

    try:
        key = (f"{c_file.resolve()}:{arch}", c_file.stat().st_mtime)
    except OSError as e:
        _fail(f"cannot stat {c_file}: {e}")
        return None
    if key in _TRANSPILE_CACHE:
        cached = _TRANSPILE_CACHE[key]
        if cached is None:
            _fail(f"c2rust previously failed for {c_file.name}")
        return cached
    result: str | None = None
    if shutil.which("c2rust") is None:
        _TRANSPILE_CACHE[key] = None
        _fail("binary not on PATH (install c2rust, or pass --no-c2rust to translate pure-LLM)")
        return None
    candidates = _find_db_entries(c_file, arch)
    if not candidates:
        _TRANSPILE_CACHE[key] = None
        _fail(
            f"no compile_commands entry for {c_file.name} "
            "(build the arch first so the compile DB exists, or pass --no-c2rust)"
        )
        return None
    attempts: list[str] = []
    for attempt_no, (entry, build_dir) in enumerate(candidates):
        result = _transpile_one(
            c_file, entry, build_dir, timeout=timeout, attempts=attempts, strict=strict
        )
        if result is not None:
            if attempt_no and verbose:
                # NOT a silent degrade: say which db actually produced the anchor and
                # why the asked-for one could not. An anchor from another arch is still
                # a compiler-exact structural checklist, but arch-specific code in it
                # (pointer widths, calling conventions) is that arch's, so the caller
                # has to know.
                print(
                    f"  [c2rust] {c_file.name}: anchored from {build_dir.name}, not the "
                    f"requested {candidates[0][1].name} — {attempts[0]}",
                    file=sys.stderr,
                )
            _TRANSPILE_CACHE[key] = result
            return result
    _TRANSPILE_CACHE[key] = None
    _fail(
        f"no .rs produced for {c_file.name} — this file cannot be anchored on any built "
        f"arch; pass --no-c2rust to translate it pure-LLM as a deliberate choice.\n    "
        + "\n    ".join(attempts)
    )
    return None


def _transpile_one(
    c_file: Path,
    entry: dict,
    build_dir: Path,
    timeout: int,
    attempts: list[str],
    strict: bool,
) -> str | None:
    """One c2rust run against one build db. Appends a `<db>: <reason>` line to
    `attempts` on failure and returns None; returns the raw .rs text on success."""
    flags = _extract_clang_flags(entry)
    # __thiscall is the third and last c2rust blocker in the msvcrt closure: forming a
    # pointer to a function whose type carries a CALLING-CONVENTION attribute panics
    # convert_type.rs ("ctype parameter must be a function instead of Attributed(...)"),
    # and cxx.h's DEFINE_CXX_DATA takes the address of every __thiscall dtor. On a
    # 64-bit target the compiler ignores the keyword outright — there is only one
    # convention — so erasing it there is a no-op, not a semantic edit. On i386 it is
    # real, so we never touch it: that file anchors from a 64-bit db instead (the caller
    # is told), or not at all.
    extra = [] if _targets_i386(flags) else ["-D__thiscall="]
    for variant in _PSEH_VARIANTS:
        result = _transpile_attempt(
            c_file, build_dir, flags, [*extra, *variant], timeout, attempts, strict
        )
        if result is not None:
            return result
    return None


def _transpile_attempt(
    c_file: Path,
    build_dir: Path,
    flags: list[str],
    extra: list[str],
    timeout: int,
    attempts: list[str],
    strict: bool,
) -> str | None:
    """One c2rust invocation. Appends a `<db>: <reason>` line to `attempts` on failure."""
    result: str | None = None
    out_dir = Path(tempfile.mkdtemp(prefix="c2rust_anchor_"))
    try:
        cmd = [
            "c2rust",
            "transpile",
            "--output-dir",
            str(out_dir),
            str(c_file.resolve()),
            # c2rust prunes a `static` function nothing calls. Liveness is per ARCH:
            # setuplib's IsAcpiComputer is called only under `#ifndef _M_AMD64`, so the amd64
            # anchor lacked a body the i386 and arm64 builds compile. Every definition the
            # preprocessor kept is a body we translate, so keep them all.
            "--preserve-unused-functions",
            "--",
            # BEFORE the db flags, so a -D of the same name in the db still wins.
            *_ATTR_NEUTRALIZERS,
            *extra,
            *flags,
        ]
        try:
            proc = subprocess.run(
                cmd, cwd=str(build_dir), capture_output=True, text=True, timeout=timeout
            )
        except subprocess.TimeoutExpired:
            attempts.append(f"{build_dir.name}: timeout after {timeout}s")
            return None
        # SURFACE c2rust's own diagnosis. This used to say only "(AST unsupported)",
        # which is a GUESS — the real reason (a clang parse error in OUR flags, a c2rust
        # panic, an unsupported construct) was captured and thrown away, so every failure
        # looked identical and un-actionable. Failing loud means saying WHY.
        rs = out_dir / "src" / f"{c_file.stem}.rs"
        if not rs.exists():
            alt = out_dir / f"{c_file.stem}.rs"  # some layouts drop it at the top level
            rs = alt if alt.exists() else rs
        # Best-effort library callers may still use a partial .rs; strict callers never do.
        if not rs.exists() or (strict and proc.returncode != 0):
            attempts.append(
                f"{build_dir.name}: exit {proc.returncode}: "
                f"{_c2rust_reason(proc.stderr, proc.stdout)}"
            )
            return None
        result = _unprefix_intrinsics(rs.read_text(errors="replace"))
    finally:
        shutil.rmtree(out_dir, ignore_errors=True)
    return result


def build_anchor_map(
    c_file: Path,
    timeout: int = 180,
    verbose: bool = True,
    strict: bool = False,
    required_names: set[str] | None = None,
    arch: str | None = None,
) -> dict[str, str]:
    """Transpile c_file with c2rust and return {fn_name: rust_body}. strict=True raises
    C2RustError on failure (no dumb fallback); strict=False returns {} best-effort."""
    text = _transpile_text(c_file, timeout=timeout, verbose=verbose, strict=strict, arch=arch)
    if text is None:
        return {}
    fns = _parse_functions(text)
    # Only an error when functions were EXPECTED: a TU of file-scope data alone (setuplib's
    # fileqsup.c) correctly transpiles to no functions, and has only a preamble to translate.
    if strict and not fns and required_names:
        raise C2RustError(f"{c_file}: c2rust output contains no extracted function definitions")
    missing = (required_names or set()) - set(fns)
    if strict and missing:
        found, absent = _anchor_elsewhere(c_file, missing, timeout)
        for name, body in found.items():
            fns[name] = body
        if verbose and found:
            print(
                f"  [c2rust] {c_file.name}: {len(found)} anchor(s) from another arch, where "
                f"the requested one does not compile them: {', '.join(sorted(found))}",
                file=sys.stderr,
            )
        if absent == missing - set(found) and absent and not _ANCHOR_FAILURES.get(c_file):
            raise CompiledOut(c_file, absent)
        missing = missing - set(found)
    if strict and missing:
        raise C2RustError(
            f"{c_file}: missing required function anchors: " + ", ".join(sorted(missing))
        )
    if verbose:
        print(f"  [c2rust] anchors: {len(fns)} fn(s) from {c_file.name}", file=sys.stderr)
    return fns


_ANCHOR_FAILURES: dict[Path, list[str]] = {}


def _anchor_elsewhere(
    c_file: Path, names: set[str], timeout: int
) -> tuple[dict[str, str], set[str]]:
    """Look for `names` in every built arch's transpile: (bodies found, names in none).

    A name is reported absent only when every db settles it. A db whose c2rust run fails
    (arm64 panics on setuplib's partlist.c) is asked the narrower question directly: does
    the identifier survive its PREPROCESSOR at all? Anything unsettled is recorded in
    _ANCHOR_FAILURES and the caller keeps the hard error.
    """
    found: dict[str, str] = {}
    failures: list[str] = []
    for entry, build_dir in _find_db_entries(c_file):
        attempts: list[str] = []
        text = _transpile_one(c_file, entry, build_dir, timeout, attempts, strict=True)
        if text is None:
            kept = _preprocessor_keeps(c_file, entry, build_dir, names, timeout)
            if kept is None or kept - set(found):
                failures.extend(attempts)
            continue
        fns = _parse_functions(text)
        for n in names - set(found):
            if n in fns:
                found[n] = fns[n]
    _ANCHOR_FAILURES[c_file] = failures
    return found, names - set(found)


def _preprocessor_keeps(
    c_file: Path, entry: dict, build_dir: Path, names: set[str], timeout: int
) -> set[str] | None:
    """The subset of `names` still spelled anywhere after `clang -E`, or None if it failed."""
    flags = _extract_clang_flags(entry)
    extra = [] if _targets_i386(flags) else ["-D__thiscall="]
    # The db's OWN compiler: arm64 is llvm-mingw, whose CRT headers come with the driver
    # rather than from an -I, so a system clang cannot even find <stdio.h> there.
    toks = entry.get("arguments") or shlex.split(entry.get("command", ""))
    cc = _decanon(toks[0]) if toks else "clang"
    try:
        proc = subprocess.run(
            [cc, "-E", "-P", *_ATTR_NEUTRALIZERS, *extra, *flags, str(c_file.resolve())],
            cwd=str(build_dir),
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if proc.returncode != 0:
        return None
    return {n for n in names if re.search(rf"\b{re.escape(n)}\b", proc.stdout)}


# File-scope DATA tables c2rust transcribes compiler-exact: the lookup tables / S-boxes /
# dispatch tables / quirk lists the LLM preamble pass FABRICATES or truncates. We take ONLY
# `static [mut]` items and `const` items with an AGGREGATE initializer (`[`/`{`) — i.e. real
# data. We deliberately EXCLUDE scalar `const NAME: _ENUM = 5;` items: c2rust floods those from
# every header enum it touched (PCIBus/PowerAction…, hundreds per file), they are NOT the file's
# own data, and they COLLIDE with the enums bindgen/ntkrnl_sys already define. Struct/type/union
# defs are likewise excluded — drivers get those exact from bindgen `types_*.rs`; only DATA is
# fabrication-prone. Scalar #define consts stay the harvest_consts/LLM-preamble job (low risk).
_DATA_SIG = re.compile(r"^(?:pub\s+)?(static(?:\s+mut)?|const)\s+[A-Za-z_]\w*\s*:", re.M)
_DATA_NAME = re.compile(r"^(?:pub\s+)?(?:static(?:\s+mut)?|const)\s+([A-Za-z_]\w*)\s*:")


def _is_own_data(name: str, c_text: str, own_fns: set[str]) -> bool:
    """Is a c2rust file-scope item really THIS file's data?

    Two things c2rust emits at file scope are not:
      * a header's DATA when the TU sees definitions -- atapi.h includes <initguid.h>, so
        every GUID in wdmguid.h/ntddstor.h/wmidata.h (416 of them) became a `pub static`
        in each of 15 modules. Only names the .c itself spells are kept.
      * a function-typed forward declaration, `static IO_COMPLETION_ROUTINE Foo;`, which
        c2rust turns into `static Foo: IO_COMPLETION_ROUTINE = None;` beside the real
        `fn Foo` -- E0428, and a NULL where the function should be.
    """
    if name in own_fns:
        return False
    return re.search(rf"\b{re.escape(name)}\b", c_text) is not None


_ANON_TYPE_RE = re.compile(r"\bC2RustUnnamed\w*")


def _with_anonymous_types(text: str, items: list[str]) -> list[str]:
    """Prepend the definitions of the anonymous types the kept data items are typed with.

    A file-scope table of an anonymous struct (`static const struct { USHORT VendorID; ... }
    AhciControllerList[]`) is `static AhciControllerList: [C2RustUnnamed_158; N]` in c2rust's
    output, and the harvest kept the static without its type: pciidex had 90 "cannot find
    struct C2RustUnnamed_*" errors. A named type comes from the island; an anonymous one
    exists only here, so it travels with its table (recursively, for nested ones).
    """
    wanted: list[str] = []
    pending = sorted(set(_ANON_TYPE_RE.findall("\n".join(items))))
    seen: set[str] = set()
    while pending:
        name = pending.pop()
        if name in seen:
            continue
        seen.add(name)
        m = re.search(
            rf"^((?:#\[[^\]]*\]\n)*)pub (?:struct|union) {re.escape(name)} \{{", text, re.M
        )
        if m is None:
            continue
        depth, i = 0, text.index("{", m.end() - 1)
        while i < len(text):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        block = text[m.start() : i + 1]
        wanted.append(block)
        pending += [n for n in _ANON_TYPE_RE.findall(block) if n not in seen]
    return [*reversed(wanted), *items] if wanted else items


def build_preamble(
    c_file: Path,
    timeout: int = 180,
    verbose: bool = True,
    strict: bool = False,
    arch: str | None = None,
) -> str:
    """Return c2rust's file-scope DATA tables (`static [mut]` + aggregate `const`) as exact
    Rust text, to prepend deterministically instead of LLM-translating the preamble (which
    fabricates table values). Excludes header enum-variant scalar consts. strict=True raises
    C2RustError on failure (no dumb fallback); strict=False returns '' best-effort."""
    text = _transpile_text(c_file, timeout=timeout, verbose=verbose, strict=strict, arch=arch)
    if text is None:
        return ""
    lines = text.splitlines()
    c_text = c_file.read_text(errors="replace")
    own_fns = set(re.findall(r"^([A-Za-z_]\w*)\s*\(", c_text, re.M))
    out: list[str] = []
    i, n = 0, len(lines)
    while i < n:
        m = _DATA_SIG.match(lines[i])
        if m:
            start = i
            # an item runs until brackets balance and the line ends in `;` (handles both
            # one-line scalars and multi-line array/struct initialisers).
            depth = 0
            j = i
            while j < n:
                depth += lines[j].count("[") - lines[j].count("]")
                depth += lines[j].count("{") - lines[j].count("}")
                if depth <= 0 and lines[j].rstrip().endswith(";"):
                    break
                j += 1
            block = "\n".join(lines[start : j + 1])
            # keep `static*` always; keep `const` ONLY if it carries an aggregate (table/struct),
            # never a bare scalar enum-variant const.
            named = _DATA_NAME.match(lines[start])
            if (
                named is not None
                and (m.group(1).startswith("static") or ("[" in block or "{" in block))
                and _is_own_data(named.group(1), c_text, own_fns)
            ):
                out.append(block)
            i = j + 1
            continue
        i += 1
    out = _with_anonymous_types(text, out)
    if verbose:
        print(f"  [c2rust] preamble: {len(out)} data item(s) from {c_file.name}", file=sys.stderr)
    return hoist_wide_temporaries("\n\n".join(out))


# c2rust spells a file-scope `L"Arial,186"` as `transmute::<[u8; 20], [c_ushort; 10]>(*b"A\0r\0…")
# .as_ptr()`: a pointer into a TEMPORARY, which const-eval rejects in a static's final value
# ("encountered dangling pointer"). setuplib's mui.c font tables were 22 such errors.
_WIDE_TEMP_RE = re.compile(
    r"::core::mem::transmute::<\s*\[u8;\s*(\d+)\]\s*,\s*\[::core::ffi::c_ushort;\s*(\d+)\]\s*,?\s*>"
    r"\(\s*\*b\"((?:[^\"\\]|\\.)*)\"\s*,?\s*\)\s*\.as_ptr\(\)"
)
_C2R_WSTR_PREFIX = "RUSTCONV_C2R_WSTR_"


def _byte_string(body: str) -> bytes:
    r"""Decode a Rust `b"..."` body (c2rust emits \0, \xNN, \n, \t, \r, \\, \", \')."""
    out = bytearray()
    i = 0
    simple = {"0": 0, "n": 10, "t": 9, "r": 13, "\\": 92, '"': 34, "'": 39}
    while i < len(body):
        c = body[i]
        if c != "\\":
            out += c.encode()
            i += 1
            continue
        e = body[i + 1]
        if e == "x":
            out.append(int(body[i + 2 : i + 4], 16))
            i += 4
        elif e in simple:
            out.append(simple[e])
            i += 2
        else:
            raise C2RustError(f"unhandled byte-string escape \\{e} in c2rust output")
    return bytes(out)


def hoist_wide_temporaries(text: str) -> str:
    """Give each transmuted wide temporary a named `[u16; N]` static, pointed to by name.

    The static is named by content hash, so a string used twice is emitted once; the prefix
    differs from wide_seal's `RUSTCONV_WSTR_` so the two generators can never define one
    name twice in a module.
    """
    import hashlib

    statics: dict[str, str] = {}

    def repl(m: re.Match[str]) -> str:
        raw = _byte_string(m.group(3))
        if len(raw) != int(m.group(1)) or len(raw) != 2 * int(m.group(2)):
            raise C2RustError(f"c2rust wide literal length mismatch: {m.group(0)[:120]}")
        units = tuple(int.from_bytes(raw[k : k + 2], "little") for k in range(0, len(raw), 2))
        name = _C2R_WSTR_PREFIX + hashlib.sha256(raw).hexdigest()[:12].upper()
        statics[name] = f"pub static {name}: [u16; {len(units)}] = {_render_units(units)};"
        return f"{name}.as_ptr()"

    body = _WIDE_TEMP_RE.sub(repl, text)
    if not statics:
        return text
    return "\n".join(statics[k] for k in sorted(statics)) + "\n\n" + body


# --------------------------------------------------------------------------------------
# types a TU defines for itself
# --------------------------------------------------------------------------------------

_TYPE_ITEM_RE = re.compile(r"(?m)^((?:#\[[^\n]*\]\n)*)pub (struct|union|type) ([A-Za-z_]\w*)\b")


def _type_items(text: str) -> dict[str, str]:
    """Every top-level `pub struct|union|type` in c2rust output, attributes included."""
    out: dict[str, str] = {}
    for m in _TYPE_ITEM_RE.finditer(text):
        kind, name = m.group(2), m.group(3)
        if kind == "type":
            end = text.find(";", m.end())
        else:
            open_idx = text.find("{", m.end())
            semi = text.find(";", m.end())
            if open_idx < 0 or (0 <= semi < open_idx):
                end = semi
            else:
                depth, end = 0, open_idx
                while end < len(text):
                    depth += {"{": 1, "}": -1}.get(text[end], 0)
                    if depth == 0:
                        break
                    end += 1
        if end < 0:
            continue
        out.setdefault(name, text[m.start() : end + 1])
    return out


def local_types(
    c_file: Path, names: set[str], arches: tuple[str, ...] = ("i386", "amd64", "arm64")
) -> tuple[dict[str, str], dict[str, str]]:
    """Exact Rust for types the C FILE ITSELF defines (a `typedef struct` in the .c, or one
    declared inside a function body), from c2rust's translation of that file.

    The bindgen island only sees headers, so partinfo.c's `MBR_PARTITION_TYPE` and bootsup.c's
    function-local `DISK_GEOMETRY_EX_INTERNAL` were defined nowhere, and the per-function
    translation cannot define a type it only uses. c2rust lays them out with the C compiler
    and keeps C's typedef names (`UCHAR`, `PCSTR`), which the island defines identically.

    A requested name is taken only if the C file's own text mentions it, plus the struct a
    requested typedef aliases (`MBR_PARTITION_TYPE = _MBR_PARTITION_TYPE`). Each arch is
    transpiled; an item whose text differs across arches is returned per arch in the second
    dict as `name -> "arch:text\\0arch:text..."` -- never one arch's layout for all three.
    Returns (agreed items, arch-dependent items).
    """
    c_text = c_file.read_text(errors="replace")
    wanted = {n for n in names if re.search(rf"\b{re.escape(n)}\b", c_text)}
    if not wanted:
        return {}, {}
    per_arch: dict[str, dict[str, str]] = {}
    for arch in arches:
        text = _transpile_text(c_file, verbose=False, arch=arch)
        if text is None:
            continue
        items = _type_items(text)
        picked: dict[str, str] = {}
        for name in wanted:
            item = items.get(name)
            if item is None:
                continue
            picked[name] = item
            alias = re.fullmatch(r"(?s).*pub type \w+ = (\w+);", item)
            target = alias.group(1) if alias else None
            if target and target in items and re.search(rf"\b{re.escape(target)}\b", c_text):
                picked.setdefault(target, items[target])
        per_arch[arch] = picked
    agreed: dict[str, str] = {}
    differs: dict[str, str] = {}
    for name in sorted(set().union(*(p.keys() for p in per_arch.values())) if per_arch else ()):
        texts = {a: p[name] for a, p in per_arch.items() if name in p}
        if len(set(texts.values())) == 1 and len(texts) == len(per_arch):
            agreed[name] = next(iter(texts.values()))
        else:
            differs[name] = "\0".join(f"{a}:{t}" for a, t in sorted(texts.items()))
    return agreed, differs


_EXTERN_FN_RE = re.compile(r"(?m)^    (?:pub )?fn ([A-Za-z_]\w*)\(")


def extern_signatures(
    c_file: Path, names: set[str], arches: tuple[str, ...] = ("i386", "amd64", "arm64")
) -> dict[str, str]:
    """`fn NAME(..) -> T;` for C functions the file DECLARES (a header outside the island),
    from c2rust's `extern` block, when every arch's text agrees.

    The calling convention is NOT taken from here: c2rust writes `extern "C"` even for a
    stdcall function on i386. `sdk_facts.function_abis` supplies it.
    """
    per_arch: dict[str, dict[str, str]] = {}
    for arch in arches:
        text = _transpile_text(c_file, verbose=False, arch=arch)
        if text is None:
            continue
        found: dict[str, str] = {}
        for m in _EXTERN_FN_RE.finditer(text):
            if m.group(1) not in names:
                continue
            end = text.find(";\n", m.end())
            if end < 0:
                continue
            sig = text[m.start() : end + 1].strip()
            found[m.group(1)] = re.sub(r"^(?:pub )?fn ", "fn ", sig)
        per_arch[arch] = found
    out: dict[str, str] = {}
    for name in names:
        texts = {p.get(name) for p in per_arch.values()}
        if len(texts) == 1 and None not in texts and per_arch:
            out[name] = texts.pop()  # type: ignore[assignment]
    return out
