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
        # ntoskrnl compiles no C through CMake (cargo-linked, every .c flipped), so it has no
        # db entry anywhere. macro_authority synthesizes its real compile environment from the
        # derived kernel profile; the call seal already depends on that same command.
        from .macro_authority import _derived_kernel_entry

        for bd in order:
            argv = _derived_kernel_entry(REACTOS_ROOT, bd.removeprefix("build-"), c_file)
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
    # MSVC INTERLOCKED INTRINSICS -- the third blocker. Under -fms-extensions clang treats
    # `_InterlockedOr` & co. as language builtins, c2rust 0.22 has no lowering for them, and it
    # DROPS every function that calls one ("Unimplemented builtin _InterlockedOr"). That made
    # five of ntoskrnl/ps/job.c's functions unanchorable, so its required-anchor check refused
    # the whole TU. Renaming the intrinsic to its un-underscored name turns the declaration
    # into an ordinary function. The `InterlockedOr` -> `_InterlockedOr` header macro then stops
    # at `InterlockedOr`, since a macro is not re-expanded inside itself, so the anchor shows
    # the plain call the Rust uses (ntkrnl_sys::interlocked).
    *(
        f"-D_Interlocked{op}{width}=Interlocked{op}{width}"
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
        result = rs.read_text(errors="replace")
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
    if strict and not fns:
        raise C2RustError(f"{c_file}: c2rust output contains no extracted function definitions")
    missing = (required_names or set()) - set(fns)
    if strict and missing:
        raise C2RustError(
            f"{c_file}: missing required function anchors: " + ", ".join(sorted(missing))
        )
    if verbose:
        print(f"  [c2rust] anchors: {len(fns)} fn(s) from {c_file.name}", file=sys.stderr)
    return fns


# File-scope DATA tables c2rust transcribes compiler-exact: the lookup tables / S-boxes /
# dispatch tables / quirk lists the LLM preamble pass FABRICATES or truncates. We take ONLY
# `static [mut]` items and `const` items with an AGGREGATE initializer (`[`/`{`) — i.e. real
# data. We deliberately EXCLUDE scalar `const NAME: _ENUM = 5;` items: c2rust floods those from
# every header enum it touched (PCIBus/PowerAction…, hundreds per file), they are NOT the file's
# own data, and they COLLIDE with the enums bindgen/ntkrnl_sys already define. Struct/type/union
# defs are likewise excluded — drivers get those exact from bindgen `types_*.rs`; only DATA is
# fabrication-prone. Scalar #define consts stay the harvest_consts/LLM-preamble job (low risk).
_DATA_SIG = re.compile(r"^(?:pub\s+)?(static(?:\s+mut)?|const)\s+[A-Za-z_]\w*\s*:", re.M)


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
            if m.group(1).startswith("static") or ("[" in block or "{" in block):
                out.append(block)
            i = j + 1
            continue
        i += 1
    if verbose:
        print(f"  [c2rust] preamble: {len(out)} data item(s) from {c_file.name}", file=sys.stderr)
    return "\n\n".join(out)
