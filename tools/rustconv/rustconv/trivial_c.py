"""Deterministic, zero-cost lowering of *trivial* C function bodies to Rust.

WHY this exists: of 952 paid per-function translations in the last program, 394
(41%) sat in the trivial static-difficulty band — no branches, no loops, no
gotos, <= 12 non-blank lines. Paying a model to write ``ERROR_SUCCESS`` buys
nothing, still costs a review pass, and (per the sweep that repaired 61 invented
constants) is measurably *less* reliable than copying the C token. Those bodies
are lowered here instead: deterministically, for $0, with no LLM in the loop.

This is the C-path sibling of ``cpp_com.lower_trivial_worker`` (the shell32
C++/COM equivalent) and keeps the same discipline:

  * recognize ONLY whole-body shapes that can be **proven** equivalent,
  * ``fullmatch`` or nothing — the residual body must be *entirely* accounted
    for, so an unrecognized token can never be silently ignored,
  * return ``None`` ("not a template — pay the model") the instant anything is
    unaccounted for.

The asymmetry that drives every judgement call below: a ``None`` costs one API
call. A wrong lowering costs a silent miscompile that **nobody reviews**, because
unlike a paid translation there is no reviewer on this path at all. So every
shape carries an explicit precondition, and :func:`lower_trivial_c_function`
re-proves its own output through an independent oracle
(:func:`verify_trivial_lowering`) before handing it back. Disagreement between
the matcher and the oracle is a bug in this module and raises
:class:`TrivialLoweringError` — loudly, never as a quiet ``None``.

CONSTANTS ARE NOT OURS TO RESOLVE. Every constant occurrence is copied through
**verbatim** — never evaluated, folded, or normalized (``38 + 1`` never becomes
``39``; ``0x200`` never becomes ``512``). That is the AGENTS.md substrate rule,
and it keeps the output compatible with ``fact_seal``: in a sealed body a
``__RUSTCONV_FACT_nnnn__`` token is simply another opaque constant spelling, so a
sealed body lowers and verifies exactly like an unsealed one.

Accepted shapes, each with its precondition:

``empty``
    Residual body is empty, or exactly ``return;``. Requires a ``void`` C return
    type — an empty body in a value-returning C function falls off the end, and
    we will not invent the value it never returned.
``constant_return``
    ``return <constant-expression>;``. The expression is restricted to constant
    atoms combined with ``| & ^`` and unary ``~`` only (see
    :func:`_lower_constant_expression` for why all arithmetic is excluded).
    Requires a non-``void`` return type, and a non-pointer return type when the
    constant is an integer literal.
``param_return``
    ``return <param>;`` where the parameter's C type spelling equals the return
    type's, so no conversion is implied.
``forwarder``
    ``return Other(a, b, c);`` / ``Other(a, b, c);`` where the argument list is
    *exactly* the parameters, in order, by name, unmodified. Any cast, reorder,
    literal, address-of, or arity change fails the identity check. Every
    parameter must be named.
``forwarder_with_null``
    ``return Other_l(a, b, c, NULL);`` — the ``forwarder`` above, plus literal
    ``NULL`` arguments. This is THE dominant CRT idiom: the locale-aware runtime
    is written as pairs, ``X(args)`` delegating to ``X_l(args, NULL)``, and it is
    145 of msvcrt's 1792 functions (76 in mbcs.c alone — 40% of that file). Those
    are exactly the calls where a model has nothing to add and everything to
    subtly break: a reordered argument, a dropped ``NULL``, the wrong locale
    sentinel.

    Admitting a literal is only sound because the null is PROVEN, not assumed.
    The callee must be defined in the SAME translation unit, so its parameter
    list has already been parsed, and each ``NULL`` is accepted only where that
    callee's parameter at that position is a pointer type — from the signature,
    never inferred from the argument. "Is a pointer" reaches past the spelling:
    the CRT's sentinel is ``_locale_t``, a typedef whose star lives in a header,
    and until ``rustconv.typedefs`` proved those from the include closure every
    one of those 145 refused. Arity must match the callee exactly (so a
    variadic or unknown callee is refused), and the non-``NULL`` arguments must
    still be the parameters in order. ``0``/``FALSE`` are NOT accepted even
    though C would coerce them: ``0`` at an ``int`` position is just zero, and
    distinguishing the two costs more certainty than the shape is worth.
``getter``
    ``return p->field;`` / ``return s.field;`` on a single member of a parameter.
    ``->`` requires a single-star parameter type; ``.`` requires a non-pointer
    one.
``setter``
    ``p->field = <param-or-constant>;`` on a single member of a single-star
    parameter. Requires a ``void`` return type. The ``s.field = ..`` form is
    rejected: assigning through a by-value C parameter mutates a copy, so the
    faithful Rust needs a ``mut`` binding and the statement is dead anyway.

Debug-only bodies fold into the shapes above: recognized debug-channel statements
are removed *before* matching and re-emitted as Rust line comments. They are
NEVER lowered to a call into a no-op function — an out-of-line no-op is an
optimization barrier that uncloaks latent ABI mismatches into ``ud2`` and a
BUGCHECK (AGENTS.md; field manual 9.65). A comment is also why we do not emit
``FIXME!(..)`` even though the crate preludes define exactly such no-op macros:
the C argument text is not guaranteed to *tokenize* as Rust (C string escapes are
laxer than Rust's), and a comment is the shape the reviewed usermode crates
already use (``wkssvc-rust/src/wkssvc/rpcserver.rs``).

Deliberately NOT accepted, so the boundary is explicit rather than implied:
casts, ``NULL``-to-pointer coercion, arithmetic in a returned constant, string
literals, ``ASSERT`` (control flow, not a debug print), C-variadic functions
(stable Rust cannot define one), K&R definitions, array/function-pointer
parameters, and out-parameter writes (``*p = 0; return 0;``) — the last one
because a ``#[no_mangle]`` output-pointer write owes a
``core::ptr::write_volatile`` (``tools/lint_kernel.py``, the captures(none)
class), which makes the Rust no longer a transliteration and so voids the
ordered-identifier oracle this module's safety net is built on.
"""

from __future__ import annotations

import re
from collections import Counter
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass

from tree_sitter import Node, Parser

# `_canonical_block` is the ONE renderer for "C convention + linkage -> Rust fn
# header" (abi_enforce.py's docstring is the reference for why each shape looks
# the way it does, including the #[cfg] duals). Re-deriving it here would be a
# second copy of the project's #1 i386 crash-cause truth, so reach for the
# private helper instead — the same way cpp_com reaches into call_seal.
from .abi_enforce import _canonical_block
from .callgraph import FunctionDep
from .conventions import CONV_KEYWORDS, KW_TO_CONV
from .cpp_core import c_type_to_rust
from .scanner import CPP_LANG
from .semantic_skeleton import RUST_LANG, extract_skeleton
from .typedefs import NULL_CONST, NULL_MUT


class TrivialLoweringError(RuntimeError):
    """A matched shape failed independent re-verification — a bug in this module.

    Deliberately not a ``None``: ``None`` means "not a template, pay the model",
    which is a normal outcome. A matcher/oracle disagreement means the
    deterministic path produced something it cannot prove, and per AGENTS.md that
    must fail loudly rather than degrade into a paid call that hides it.
    """


SHAPES = (
    "empty",
    "constant_return",
    "param_return",
    "forwarder",
    "forwarder_with_null",
    "getter",
    "setter",
)

# The oracle compares C and Rust token-for-token, and a null pointer is the one
# atom whose two spellings cannot match: C's `NULL` is a single `null` node (and
# an uppercase-constant occurrence), while Rust needs `core::ptr::null_mut()` —
# three identifiers and an extra call node. Rather than weaken checks 2/3/4 for
# everyone, both sides are rewritten to this single sentinel before comparison.
# It is deliberately lowercase so `_constants` stops counting it on the C side too,
# which keeps the missing/invented constant invariant symmetric.
_NULL_SENTINEL = "__rustconv_null"
_RE_C_NULL = re.compile(r"(?<![A-Za-z0-9_])NULL(?![A-Za-z0-9_])")
_RE_RUST_NULL = re.compile(r"core::ptr::null(?:_mut)?\(\)")

# Debug channels that are MACROS in the C and no-op `macro_rules!` in the Rust
# substrate (see e.g. `wkssvc-rust/src/prelude.rs`), so dropping the statement
# matches what the translated crates already do.
#
# `ASSERT` is deliberately absent: ReactOS ASSERT is control flow (it can
# bugcheck), not a debug print, and its expression may be load-bearing.
# `DbgPrint`/`DbgPrintEx` ARE included even though they are real ntoskrnl
# exports: a function whose entire body is a debug print is a stub with no
# functional behaviour, and the substrate already no-ops the DPRINT macros that
# wrap them. Trim this set if a target disagrees — it is the one policy knob.
DEBUG_STATEMENT_MACROS = frozenset(
    {
        "TRACE",
        "WARN",
        "ERR",
        "FIXME",
        "MESSAGE",
        "WINE_TRACE",
        "WINE_WARN",
        "WINE_ERR",
        "WINE_FIXME",
        "DPRINT",
        "DPRINT1",
        "DbgPrint",
        "DbgPrintEx",
        "UNIMPLEMENTED",
        "UNIMPLEMENTED_ONCE",
    }
)

# The subset that PRINTS in our build (DBG=1, and the kernel's ERR/WARN are DPRINT1).
# Under ``keep_diagnostics`` these are not dropped but re-emitted as the crate's macro
# with the C arguments verbatim -- ``ERR("x %lu\n", v);`` -> ``ERR!("x %lu\n", v);`` --
# which is only proven safe when the arguments call nothing: a ``debugstr_a`` has no
# Rust counterpart to name, so such a body goes to the model instead. atapi and
# pciidex lost ~90% of their error output to stripping these.
KEPT_DEBUG_MACROS = frozenset({"ERR", "WARN", "WINE_ERR", "WINE_WARN", "DPRINT1"})

_RE_KEPT_STATEMENT = re.compile(rf"^({'|'.join(sorted(KEPT_DEBUG_MACROS))})\s*\((.*)\)\s*;$", re.S)


def _kept_rendering(statement: str) -> str | None:
    """The Rust macro statement a kept C diagnostic becomes, or None if it is dropped."""
    match = _RE_KEPT_STATEMENT.match(statement)
    if match is None:
        return None
    return f"{match.group(1)}!({match.group(2)});"


# The only calls allowed *inside* a dropped debug statement's arguments. Wine's
# debugstr_* family formats into a per-thread ring buffer and is used for no
# other purpose, so removing the call along with its statement removes nothing
# observable. Any other call makes the statement potentially side-effecting and
# sends the whole function back to the paid path.
PURE_DEBUG_ARGUMENT_HELPERS = frozenset(
    {
        "debugstr_a",
        "debugstr_an",
        "debugstr_w",
        "debugstr_wn",
        "debugstr_guid",
        "debugstr_variant",
        "wine_dbgstr_a",
        "wine_dbgstr_an",
        "wine_dbgstr_w",
        "wine_dbgstr_wn",
        "wine_dbgstr_guid",
        "wine_dbgstr_rect",
        "wine_dbgstr_longlong",
    }
)

_VOID_SPELLINGS = frozenset({"void", "VOID"})
_TYPE_NOISE_WORDS = frozenset({"static", "extern", "inline", "__inline", "FORCEINLINE"})
_PROBE = "__rustconv_trivial_probe"

_IDENT = r"[A-Za-z_]\w*"
_FACT = r"__RUSTCONV_FACT_[0-9]{4}__"
_INT = r"(?:0[xX][0-9A-Fa-f]+|0[bB][01]+|[0-9]+)[uUlL]*"
# One constant OCCURRENCE, in either an unsealed or a fact-sealed body. The
# placeholder alternative leads so a sealed token is never split into `0000`.
_CONSTANT_OCCURRENCE = re.compile(
    rf"{_FACT}"
    rf"|(?<![A-Za-z0-9_]){_INT}(?![A-Za-z0-9_])"
    r"|(?<![A-Za-z0-9_])[A-Z][A-Z0-9_]*(?![A-Za-z0-9_])"
)
# An ATOM carries NO C suffix: `0L`/`1UL`/`0xFFul` are not valid Rust literals, and
# dropping the suffix would be normalizing a constant we are contractually required
# to copy verbatim. So a suffixed literal is not an atom, the expression fails to
# tokenize, and the function goes back on the paid path. (The oracle caught this on
# iphlpapi's `return 0L;` stubs — `0L` parses in Rust as `0` plus a stray `L`.)
_CONSTANT_ATOM = re.compile(rf"{_FACT}|(?:0[xX][0-9A-Fa-f]+|0[bB][01]+|[0-9]+)|[A-Z][A-Z0-9_]*")
_INT_ONLY = re.compile(rf"[!~\s]*{_INT}")
_RUST_TYPE_OK = re.compile(r"(?:\*(?:const|mut) )*[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*")

# A C identifier that is a Rust keyword becomes a raw identifier. `type`, `ref`,
# `fn`, `match`, `box` and friends are ordinary C names; emitted bare they are
# either a syntax error or — worse — reinterpreted (`fn: T` opens a nested item and
# silently changes the ARITY of the signature, which is how the oracle found this
# on rpcrt4's `RpcMgmtSetAuthorizationFn`).
_RUST_KEYWORDS = frozenset(
    {
        # strict (2015/2018) keywords …
        "as", "async", "await", "break", "const", "continue", "dyn", "else",
        "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let",
        "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "static",
        "struct", "trait", "true", "type", "union", "unsafe", "use", "where",
        "while",
        # … plus reserved-for-future-use, which are equally illegal bare.
        "abstract", "become", "box", "do", "final", "macro", "override", "priv",
        "try", "typeof", "unsized", "virtual", "yield",
    }
)  # fmt: skip
# These four cannot be spelled `r#...` at all, so there is no faithful rendering.
_UNRAWABLE_KEYWORDS = frozenset({"crate", "self", "Self", "super"})

_RE_EMPTY = re.compile(r"(?:return\s*;)?")
_RE_CONST_RETURN = re.compile(r"return\s+(?P<expr>\S.*?)\s*;")
_RE_FORWARD = re.compile(rf"(?P<ret>return\s+)?(?P<callee>{_IDENT})\s*\((?P<args>.*)\)\s*;")
_RE_GETTER = re.compile(rf"return\s+(?P<base>{_IDENT})\s*(?P<op>->|\.)\s*(?P<field>{_IDENT})\s*;")
_RE_SETTER = re.compile(
    rf"(?P<base>{_IDENT})\s*->\s*(?P<field>{_IDENT})\s*=\s*(?P<value>\S.*?)\s*;"
)

# The C grammar does NOT report `TRUE`/`FALSE`/`NULL`/`nullptr`/`this` as
# identifiers — it folds the Windows/C++ spellings into dedicated `true`/`false`/
# `null`/`this` nodes, while tree-sitter-rust sees the same token as a plain
# `identifier`. The oracle found this the first time it met `return TRUE;`
# (wsock32 `DllMain`) and again on msvcrt's `return this->hr;`, and correctly
# refused both lowerings. Enumerating the node types is the fix; weakening the
# comparison would not be. `boolean_literal` covers a lowercase C `true`/`false`.
_C_IDENTIFIER_NODES = frozenset({"identifier", "field_identifier", "true", "false", "null", "this"})
_RUST_IDENTIFIER_NODES = frozenset({"identifier", "field_identifier", "boolean_literal"})
# `dereferences` is excluded on purpose: C spells a member load `p->f` and Rust
# spells it `(*p).f`, so the deref counts legitimately differ by exactly one.
_SKELETON_FIELDS = ("assignments", "members", "calls")


@dataclass(frozen=True)
class TrivialLowering:
    """One proven lowering, ready to drop into the translated-function map."""

    name: str
    shape: str
    signature: str
    rust: str
    dropped_debug: tuple[str, ...]


# --------------------------------------------------------------------------- #
# source masking
# --------------------------------------------------------------------------- #


def _mask_source(text: str, *, mask_strings: bool) -> str:
    """Blank comments (and optionally string/char literals), preserving offsets.

    ``mask_strings=False`` is precisely why this is local rather than reused from
    ``fact_seal._code_mask`` / ``cpp_com._masked_cpp``: the residual matcher must
    keep string literals VISIBLE so a body containing one fails the ``fullmatch``
    instead of quietly reducing to an accepted shape. Blanking would turn
    ``return "x";`` into ``return    ;``, which reads as a bare return — exactly
    the class of silent mis-lowering this module exists to prevent.
    """
    out = list(text)
    index = 0
    while index < len(text):
        if text.startswith("//", index):
            end = text.find("\n", index + 2)
            end = len(text) if end < 0 else end
            for pos in range(index, end):
                out[pos] = " "
            index = end
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            end = len(text) if end < 0 else end + 2
            for pos in range(index, end):
                if out[pos] != "\n":
                    out[pos] = " "
            index = end
        elif text[index] in ('"', "'"):
            quote, start = text[index], index
            index += 1
            while index < len(text):
                if text[index] == "\\":
                    index += 2
                    continue
                if text[index] == quote:
                    index += 1
                    break
                index += 1
            if mask_strings:
                for pos in range(start, min(index, len(text))):
                    if out[pos] != "\n":
                        out[pos] = " "
        else:
            index += 1
    return "".join(out)


def _matching_paren(text: str, opening: int) -> int | None:
    depth = 0
    for pos in range(opening, len(text)):
        if text[pos] == "(":
            depth += 1
        elif text[pos] == ")":
            depth -= 1
            if depth == 0:
                return pos
    return None


def _matching_open_paren(text: str, closing: int) -> int | None:
    depth = 0
    for pos in range(closing, -1, -1):
        if text[pos] == ")":
            depth += 1
        elif text[pos] == "(":
            depth -= 1
            if depth == 0:
                return pos
    return None


def _split_top_level(text: str) -> list[str]:
    """Split an argument list on depth-0 commas. ``""`` yields ``[]``."""
    out: list[str] = []
    start = depth = 0
    for index, char in enumerate(text):
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        elif char == "," and depth == 0:
            out.append(text[start:index].strip())
            start = index + 1
    tail = text[start:].strip()
    if tail or out:
        out.append(tail)
    return out


# --------------------------------------------------------------------------- #
# debug-statement removal
# --------------------------------------------------------------------------- #


def _debug_statement_spans(masked: str) -> list[tuple[int, int, str | None]]:
    """Locate complete debug-channel statements as ``(start, end, masked_args)``.

    Mirrors ``cpp_com.strip_debug_statements`` but also hands back each
    statement's argument text, because this module must PROVE the arguments are
    side-effect-free before it is allowed to delete them. Accepts the bare
    ``UNIMPLEMENTED;`` form and wine's channel form ``TRACE_(chan)("..")``.
    """
    names = "|".join(
        re.escape(name) for name in sorted(DEBUG_STATEMENT_MACROS, key=len, reverse=True)
    )
    spans: list[tuple[int, int, str | None]] = []
    for match in re.finditer(rf"\b(?:{names})(?![A-Za-z0-9])", masked):
        before = masked[: match.start()].rstrip()
        if before and before[-1] not in "{;}":
            continue
        pos = match.end()
        channel = re.match(rf"\s*_\s*\(\s*{_IDENT}\s*\)", masked[pos:])
        if channel is not None:
            pos += channel.end()
        args: str | None = None
        end = pos
        if re.match(r"\s*\(", masked[pos:]) is not None:
            opening = masked.index("(", pos)
            closing = _matching_paren(masked, opening)
            if closing is None:
                continue
            args = masked[opening + 1 : closing]
            end = closing + 1
        terminator = re.match(r"\s*;", masked[end:])
        if terminator is None:
            continue
        spans.append((match.start(), end + terminator.end(), args))
    return spans


def masked_name_is_kept(statement: str) -> bool:
    """True when this debug statement's macro (any form, channel included) still prints."""
    # `ERR_(chan)` is ERR too: same exclusion as the span finder, letters and digits only.
    names = "|".join(sorted(KEPT_DEBUG_MACROS, key=len, reverse=True))
    return re.match(rf"\s*(?:{names})(?![A-Za-z0-9])", statement) is not None


def _debug_arguments_are_pure(args: str | None) -> bool:
    """True when deleting this debug statement deletes nothing observable."""
    if args is None:
        return True
    if re.search(r"(?<![=!<>+\-*/%&|^])=(?!=)", args) is not None:
        return False
    if "++" in args or "--" in args:
        return False
    return all(
        call.group(1) in PURE_DEBUG_ARGUMENT_HELPERS
        for call in re.finditer(rf"({_IDENT})\s*\(", args)
    )


def _residual_body(
    c_body: str, keep_diagnostics: bool = False
) -> tuple[str, tuple[str, ...]] | None:
    """Reduce a whole C function definition to its residual statement text.

    Returns ``(residual, dropped_debug_statements)``, or ``None`` when the
    function is outside the accepted contract for a reason about the *function*
    rather than its body shape: no body, a C-variadic or K&R declarator, or an
    array/function-pointer parameter. Comments are removed (they carry no
    semantics); string literals deliberately are not (see :func:`_mask_source`).
    """
    masked = _mask_source(c_body, mask_strings=True)
    open_brace, close_brace = masked.find("{"), masked.rfind("}")
    if open_brace < 0 or close_brace <= open_brace:
        return None

    head = masked[:open_brace]
    if "..." in head:
        return None  # C-variadic: stable Rust cannot DEFINE one.
    params_close = head.rfind(")")
    if params_close < 0 or head[params_close + 1 :].strip():
        return None  # K&R definition, or a trailing attribute we cannot read.
    params_open = _matching_open_paren(head, params_close)
    if params_open is None:
        return None
    if any(token in head[params_open + 1 : params_close] for token in "([{"):
        return None  # array / function-pointer parameter: unmappable declarator.

    body = c_body[open_brace + 1 : close_brace]
    body_masked = masked[open_brace + 1 : close_brace]
    dropped: list[str] = []
    for start, end, args in reversed(_debug_statement_spans(body_masked)):
        if not _debug_arguments_are_pure(args):
            return None
        text = " ".join(body[start:end].split())
        if keep_diagnostics and masked_name_is_kept(body_masked[start:end]):
            # Re-emitted verbatim -- raw, since collapsing whitespace would edit the format
            # string -- so it must be a plain `NAME(args);` calling nothing.
            text = body[start:end]
            if _kept_rendering(text) is None or (args and re.search(rf"{_IDENT}\s*\(", args)):
                return None
        dropped.append(text)
        blank = " " * (end - start)
        body = body[:start] + blank + body[end:]
        body_masked = body_masked[:start] + blank + body_masked[end:]
    dropped.reverse()

    # Comment-only blanking, offset-preserving, so a `//` inside a string literal
    # cannot swallow real code.
    residual = " ".join(_mask_source(body, mask_strings=False).split())
    return residual, tuple(dropped)


# --------------------------------------------------------------------------- #
# C signature -> Rust signature
# --------------------------------------------------------------------------- #


def _rust_ident(name: str) -> str | None:
    """Spell one C identifier as Rust, escaping keywords; ``None`` if impossible.

    A raw identifier IS the same identifier — ``r#type`` and ``type`` name the same
    thing — so this is an escaping, not a rename, and the oracle strips the ``r#``
    back off before comparing. ``crate``/``self``/``Self``/``super`` cannot be
    raw-escaped, so those go back on the paid path.
    """
    if name in _UNRAWABLE_KEYWORDS:
        return None
    return f"r#{name}" if name in _RUST_KEYWORDS else name


def _normalize_c_type(c_type: str) -> str:
    text = " ".join(c_type.replace("CONST", "const").split())
    return re.sub(r"\s*\*\s*", " *", text)


def _return_type(dep: FunctionDep) -> str | None:
    """The C return type with declaration noise stripped, or ``None`` if unknown."""
    tokens = [
        token
        for token in _normalize_c_type(dep.return_type).split()
        if token not in _TYPE_NOISE_WORDS and token not in CONV_KEYWORDS
    ]
    return " ".join(tokens) or None


def _is_void(return_type: str) -> bool:
    return return_type in _VOID_SPELLINGS


def _normalized_params(params: Sequence[tuple[str, str]]) -> list[tuple[str, str]]:
    """Drop the ``(void)`` pseudo-parameter the scanner reports as ``('void', '')``."""
    if len(params) == 1 and params[0][0].strip() in _VOID_SPELLINGS and not params[0][1]:
        return []
    return list(params)


def _convention(c_body: str, declared: str | None) -> str:
    """Read the calling convention off the definition's own qualifier run.

    ``FunctionDep.calling_convention`` is lossy — it is ``None`` even for the
    ``DWORD WINAPI Foo(..)`` shape that dominates the Wine-derived DLLs — and a
    stdcall export derived as cdecl is the project's historical #1 i386 crash.
    ``dep.body_text`` starts at the definition node, so its head IS the bounded
    qualifier window ``abi_enforce._derive_convention`` looks for. ``postprocess``
    re-derives this from the C file afterwards and remains the final authority;
    this only has to be right enough to read.
    """
    head = _mask_source(c_body, mask_strings=True)
    head = head[: head.find("{")] if "{" in head else head
    # Bound the window to the return-type/qualifier run, exactly as
    # `_derive_convention` does, so a parameter's TYPE name can never be read as
    # the function's convention.
    params_close = head.rfind(")")
    params_open = _matching_open_paren(head, params_close) if params_close >= 0 else None
    if params_open is not None:
        head = head[:params_open]
    for keyword in CONV_KEYWORDS:
        if re.search(rf"\b{re.escape(keyword)}\b", head) is not None:
            return KW_TO_CONV[keyword]
    return KW_TO_CONV.get(declared or "", "cdecl")


def _render_params(params: Sequence[tuple[str, str]]) -> list[str] | None:
    """Render each Rust parameter binding, or ``None`` if a declarator is unmappable."""
    rendered: list[str] = []
    for c_type, name in params:
        rust_type = c_type_to_rust(c_type)
        if _RUST_TYPE_OK.fullmatch(rust_type) is None:
            return None
        # An unnamed C parameter has no Rust binding to make; `_` is the shape the
        # reviewed crates use (wkssvc-rust `_: WKSSVC_IDENTIFY_HANDLE`).
        binding = _rust_ident(name) if name else "_"
        if binding is None:
            return None
        rendered.append(f"{binding}: {rust_type}")
    return rendered


# --------------------------------------------------------------------------- #
# constant expressions
# --------------------------------------------------------------------------- #


def _lower_constant_expression(text: str) -> str | None:
    """Render a C constant expression to Rust, or ``None`` if not provable.

    Accepted: constant atoms (integer literals in any C base with any C suffix,
    ALL-CAPS symbolic constants, ``fact_seal`` placeholders) combined with ``|``,
    ``&``, ``^``, balanced parentheses, and unary ``~``.

    Rejected: everything else, and in particular ALL ARITHMETIC. C evaluates
    ``A - B`` after integer promotion and then truncates to the return type; Rust
    evaluates it *in* the return type, where an underflow panics in debug and
    wraps in release. ``0 - 1`` therefore has two different meanings and we
    cannot tell them apart without type information we do not have. The same
    argument covers ``+``, ``*``, ``/``, ``%`` and ``<<``. Bitwise ``| & ^ ~``
    commute with truncation for every integer width and signedness, so those are
    provable; anything arithmetic goes back on the paid path.

    The rendering is VERBATIM apart from ``~`` -> ``!`` (Rust spells bitwise-not
    that way). No atom is ever evaluated, folded, re-based, or re-suffixed.
    """
    tokens: list[str] = []
    pos = 0
    while pos < len(text):
        if text[pos].isspace():
            pos += 1
            continue
        atom = _CONSTANT_ATOM.match(text, pos)
        if atom is not None:
            tokens.append("atom")
            pos = atom.end()
            continue
        if text[pos] in "|&^~()":
            tokens.append(text[pos])
            pos += 1
            continue
        return None

    expect_operand, depth = True, 0
    for token in tokens:
        if expect_operand:
            if token == "~":
                continue
            if token == "(":
                depth += 1
                continue
            if token == "atom":
                expect_operand = False
                continue
            return None
        if token in "|&^":
            expect_operand = True
            continue
        if token == ")":
            depth -= 1
            if depth < 0:
                return None
            continue
        return None
    if expect_operand or depth:
        return None
    return text.replace("~", "!")


# --------------------------------------------------------------------------- #
# the matcher
# --------------------------------------------------------------------------- #


def _member_access_ok(c_type: str, op: str) -> bool:
    stars = _normalize_c_type(c_type).count("*")
    return stars == 1 if op == "->" else stars == 0


def _member(base: str, op: str, field: str) -> str | None:
    rust_base, rust_field = _rust_ident(base), _rust_ident(field)
    if rust_base is None or rust_field is None:
        return None
    return f"(*{rust_base}).{rust_field}" if op == "->" else f"{rust_base}.{rust_field}"


def _is_const_pointer_c_type(c_type: str) -> bool:
    """`const T *p` -> null(); `T *p` / `T *const p` -> null_mut().

    Only a const applying to the POINTEE picks the const spelling, so the const
    must appear before the star. (rustc is the backstop either way: a wrong
    choice is a type error at candidate-check, not a silent miscompile.)
    """
    normalized = _normalize_c_type(c_type)
    return "const" in normalized.split("*")[0].split()


def _null_spelling(c_type: str, pointer_typedefs: Mapping[str, str]) -> str | None:
    """Rust `NULL` for a parameter of type *c_type*, or None if it is not provably a pointer.

    Two sources, in order of directness. A star in the spelling proves itself. Otherwise
    the name is looked up in *pointer_typedefs*, which `rustconv.typedefs` derives from the
    TU's real header closure — that is what makes `_locale_t` (the CRT's `X`/`X_l` locale
    sentinel, and 145 refused lowerings across msvcrt) spellable at all.

    A qualifier on a typedef'd pointer binds to the POINTER, never the pointee -- `const
    _locale_t` is a const pointer to mutable state -- so the typedef's own recorded spelling
    decides and the qualifier is dropped. That is the opposite of the `const T *` case above,
    which is why the two are not one code path.
    """
    normalized = _normalize_c_type(c_type)
    if "*" in normalized:
        return NULL_CONST if _is_const_pointer_c_type(normalized) else NULL_MUT
    bare = " ".join(word for word in normalized.split() if word not in ("const", "volatile"))
    return pointer_typedefs.get(bare)


def _forward_args(
    args: Sequence[str],
    names: Sequence[str],
    callee_params: Sequence[tuple[str, str]] | None,
    pointer_typedefs: Mapping[str, str],
) -> list[str] | None:
    """Rust spellings for a forwarder's arguments, or None if not provable.

    Accepts the parameters in order, with literal `NULL` allowed ONLY at a position
    the callee's own parsed signature declares a pointer.
    """
    if callee_params is None or len(args) != len(callee_params):
        return None  # unknown or variadic callee: nothing to prove the NULL against
    spelled: list[str] = []
    remaining = list(names)
    # strict=True is provable here: the length mismatch returned None above.
    for arg, (param_type, _) in zip(args, callee_params, strict=True):
        if arg == "NULL":
            null = _null_spelling(param_type, pointer_typedefs)
            if null is None:
                return None  # NULL at a position not provably a pointer: not our shape
            spelled.append(null)
            continue
        if not remaining or arg != remaining[0]:
            return None  # reordered, substituted, or a non-parameter expression
        remaining.pop(0)
        rust_name = _rust_ident(arg)
        if rust_name is None:
            return None
        spelled.append(rust_name)
    return spelled if not remaining else None


def _match_shape(
    residual: str,
    params: Sequence[tuple[str, str]],
    return_type: str,
    own_name: str,
    # Mapping (covariant values), not dict — see lower_trivial_c_function.
    siblings: Mapping[str, Sequence[tuple[str, str]]] | None = None,
    pointer_typedefs: Mapping[str, str] | None = None,
) -> tuple[str, str] | None:
    """Return ``(shape, rust_statements)`` for a recognized residual body."""
    names = [name for _, name in params]
    by_name = {name: c_type for c_type, name in params if name}
    void = _is_void(return_type)

    if _RE_EMPTY.fullmatch(residual) is not None:
        return ("empty", "") if void else None

    match = _RE_GETTER.fullmatch(residual)
    if match is not None:
        base, op, field = match.group("base", "op", "field")
        if void or base not in by_name or not _member_access_ok(by_name[base], op):
            return None
        member = _member(base, op, field)
        return None if member is None else ("getter", member)

    match = _RE_SETTER.fullmatch(residual)
    if match is not None:
        base, field, value = match.group("base", "field", "value")
        if not void or base not in by_name or not _member_access_ok(by_name[base], "->"):
            return None
        member = _member(base, "->", field)
        rust_value = _rust_ident(value) if value in by_name else _lower_constant_expression(value)
        if member is None or rust_value is None:
            return None
        return "setter", f"{member} = {rust_value};"

    match = _RE_FORWARD.fullmatch(residual)
    if match is not None:
        callee, args = match.group("callee"), match.group("args")
        if callee == own_name or callee in DEBUG_STATEMENT_MACROS:
            return None
        if match.group("ret") is None and not void:
            return None  # value-returning C function that never returns: UB, not a template.
        if not all(names):
            return None
        rust_callee = _rust_ident(callee)
        if rust_callee is None:
            return None
        actual = _split_top_level(args)
        if actual == names:
            spelled = [_rust_ident(name) for name in names]
            if any(item is None for item in spelled):
                return None
            call = f"{rust_callee}({', '.join(item for item in spelled if item)})"
            return "forwarder", (f"{call};" if void else call)
        # …the same forwarder, plus literal NULLs proven against the callee's own
        # parsed signature (the CRT's X -> X_l(…, NULL) idiom). See the module docstring.
        if "NULL" not in actual:
            return None
        spelled_args = _forward_args(
            actual, names, (siblings or {}).get(callee), pointer_typedefs or {}
        )
        if spelled_args is None:
            return None
        call = f"{rust_callee}({', '.join(spelled_args)})"
        return "forwarder_with_null", (f"{call};" if void else call)

    match = _RE_CONST_RETURN.fullmatch(residual)
    if match is not None:
        expr = match.group("expr")
        if void:
            return None
        if expr in by_name:
            # `return x;` for a parameter x: provable only with no implied
            # conversion, i.e. the two C type spellings agree.
            if _normalize_c_type(by_name[expr]) != _normalize_c_type(return_type):
                return None
            spelled_param = _rust_ident(expr)
            return None if spelled_param is None else ("param_return", spelled_param)
        rust = _lower_constant_expression(expr)
        if rust is None:
            return None
        # An integer literal is not a pointer. `return 0;` from a pointer-typed C
        # function needs `core::ptr::null_mut()` — an interpretation of the
        # constant, and interpretation is not ours to do.
        if "*" in _normalize_c_type(return_type) and _INT_ONLY.fullmatch(rust) is not None:
            return None
        return "constant_return", rust

    return None


_MAX_HEADER = 96


def _render_item(
    name: str,
    convention: str,
    bindings: Sequence[str],
    return_type: str,
    statements: str,
    dropped_debug: Sequence[str],
    is_static: bool,
    keep_diagnostics: bool = False,
) -> str:
    lines = [
        (_kept_rendering(text) if keep_diagnostics else None) or f"// {text}"
        for text in dropped_debug
    ]
    if statements:
        lines.append(statements)
    body = "{\n" + "".join(f"    {line}\n" for line in lines) + "}" if lines else "{}"
    ret_suffix = "" if _is_void(return_type) else f" -> {c_type_to_rust(return_type)}"

    def block(params_text: str) -> str:
        return _canonical_block(
            name, convention, params_text, ret_suffix, body, "", is_static, com_method_macro=None
        )

    single = block("(" + ", ".join(bindings) + ")")
    if not bindings or max(len(line) for line in single.splitlines()) <= _MAX_HEADER:
        return single
    return block("(\n    " + ",\n    ".join(bindings) + ",\n)")


def lower_trivial_c_function(
    dep: FunctionDep,
    # Mapping, not dict: dict's value type is invariant, so a caller's
    # dict[str, list[...]] (which is what _normalized_params builds) would not be
    # assignable to dict[str, Sequence[...]].
    siblings: Mapping[str, Sequence[tuple[str, str]]] | None = None,
    pointer_typedefs: Mapping[str, str] | None = None,
    keep_diagnostics: bool = False,
) -> TrivialLowering | None:
    """Lower one trivial C function to Rust for $0, or ``None`` to pay the model.

    ``None`` is the correct, expected outcome for anything outside the proven
    shapes — see the module docstring for the accepted set and the deliberate
    exclusions. The returned lowering has already been re-proved against the C by
    :func:`verify_trivial_lowering`; a disagreement raises
    :class:`TrivialLoweringError` rather than silently falling back.

    ``siblings`` maps every same-TU function name to its parsed parameter list. It
    is what lets ``forwarder_with_null`` PROVE a literal ``NULL`` sits at a pointer
    position instead of assuming it; without it that shape is simply not offered,
    so a caller with no sibling map degrades to the pre-existing behaviour.
    """
    if dep.cpp_class is not None:
        return None  # C++ methods belong to cpp_com.lower_trivial_worker.
    if dep.name in _RUST_KEYWORDS or dep.name in _UNRAWABLE_KEYWORDS:
        # A raw-escaped DEFINITION name would no longer be found by
        # `abi_enforce._find_defs` (it greps `fn <name>`), so the downstream ABI
        # enforcement would silently skip the function. Not worth the shape.
        return None
    residual_pair = _residual_body(dep.body_text, keep_diagnostics)
    if residual_pair is None:
        return None
    residual, dropped = residual_pair

    return_type = _return_type(dep)
    if return_type is None:
        return None
    if not _is_void(return_type) and _RUST_TYPE_OK.fullmatch(c_type_to_rust(return_type)) is None:
        # The scanner does not always recover a compound return type: it reports
        # `static struct module* f(..)` as the bare spelling `*`, which renders as
        # the un-parseable `-> *mut `. Same guard as for parameters — an
        # unmappable declarator is a reason to pay the model, not to emit garbage.
        # (The oracle found this on dbghelp's `native_load_module`.)
        return None
    params = _normalized_params(dep.params)
    matched = _match_shape(residual, params, return_type, dep.name, siblings, pointer_typedefs)
    if matched is None:
        return None
    bindings = _render_params(params)
    if bindings is None:
        return None

    shape, statements = matched
    rust = _render_item(
        dep.name,
        _convention(dep.body_text, dep.calling_convention),
        bindings,
        return_type,
        statements,
        dropped,
        dep.is_static,
        keep_diagnostics,
    )
    head = rust.index("{", rust.index(f"fn {dep.name}"))
    lowering = TrivialLowering(dep.name, shape, rust[:head].rstrip(), rust, dropped)

    findings = verify_trivial_lowering(dep, lowering, keep_diagnostics)
    if findings:
        raise TrivialLoweringError(
            f"{dep.name}: deterministic lowering failed its own oracle "
            f"({'; '.join(findings)}) — that is a bug in trivial_c.py, not a "
            "reason to fall back silently"
        )
    return lowering


def lower_trivial_c_functions(
    deps: Iterable[FunctionDep],
    pointer_typedefs: Mapping[str, str] | None = None,
    keep_diagnostics: bool = False,
) -> dict[str, TrivialLowering]:
    """Lower every function in ``deps`` that is provably trivial.

    Builds the same-TU signature map once and hands it to every function, so
    ``forwarder_with_null`` can check a literal ``NULL`` against the callee's real
    parameter type. Only C++-free, non-class deps contribute (the C++ path belongs
    to ``cpp_com``).

    ``pointer_typedefs`` is :func:`rustconv.typedefs.pointer_typedefs_for_source`'s
    verdict for this TU — the names a ``NULL`` may be spelled at even though their C
    spelling carries no star. Omitting it is safe and simply lowers fewer functions;
    on msvcrt it is the difference between 123 and 268.
    """
    deps = list(deps)
    siblings = {dep.name: _normalized_params(dep.params) for dep in deps if dep.cpp_class is None}
    lowered: dict[str, TrivialLowering] = {}
    for dep in deps:
        lowering = lower_trivial_c_function(dep, siblings, pointer_typedefs, keep_diagnostics)
        if lowering is not None:
            lowered[dep.name] = lowering
    return lowered


# --------------------------------------------------------------------------- #
# the independent oracle
# --------------------------------------------------------------------------- #


def _first_node(root: Node, node_type: str) -> Node | None:
    """Breadth-first search, so the OUTERMOST node of that type wins."""
    queue = [root]
    while queue:
        node = queue.pop(0)
        if node.type == node_type:
            return node
        queue.extend(node.children)
    return None


def _ordered_identifiers(node: Node, source: bytes, kinds: frozenset[str]) -> tuple[str, ...]:
    """Every identifier token under ``node``, in source order.

    A leading ``r#`` is stripped: a raw identifier names the same thing as the bare
    one, so ``r#type`` must compare equal to the C's ``type``. That cannot mask a
    substitution — ``r#foo`` still differs from ``bar``.
    """
    out: list[str] = []

    def visit(current: Node) -> None:
        if current.type in kinds:
            text = source[current.start_byte : current.end_byte].decode("utf-8", "replace")
            out.append(text[2:] if text.startswith("r#") else text)
            return
        for child in current.children:
            visit(child)

    visit(node)
    return tuple(out)


def _constants(text: str) -> Counter[str]:
    return Counter(
        match.group(0)
        for match in _CONSTANT_OCCURRENCE.finditer(_mask_source(text, mask_strings=True))
    )


def _block_body(rust: str) -> tuple[Node, bytes, str] | None:
    """The first ``fn`` block of a Rust item, as ``(node, bytes, inner_text)``."""
    encoded = rust.encode("utf-8", "replace")
    block = _first_node(Parser(RUST_LANG).parse(encoded).root_node, "block")
    if block is None:
        return None
    inner = encoded[block.start_byte + 1 : block.end_byte - 1].decode("utf-8", "replace")
    return block, encoded, inner


def _c_probe(residual: str) -> tuple[Node, bytes, str] | None:
    """Parse residual C statements inside a throwaway wrapper.

    The wrapper's own identifiers are never visited: we descend to the
    ``compound_statement`` and walk only that, so the comparison sees exactly the
    statements the C body contained.
    """
    text = f"void {_PROBE}(void) {{{residual}}}"
    encoded = text.encode("utf-8", "replace")
    block = _first_node(Parser(CPP_LANG).parse(encoded).root_node, "compound_statement")
    return (block, encoded, text) if block is not None else None


def verify_trivial_lowering(
    dep: FunctionDep, lowering: TrivialLowering, keep_diagnostics: bool = False
) -> list[str]:
    """Re-prove a lowering against the C body, independently of the matcher.

    Nothing here reuses the shape regexes. The only machinery shared with
    :func:`lower_trivial_c_function` is the debug-statement stripper — there is
    one definition of "which statements were dropped" and both sides must agree
    on it by construction. Everything after that is a tree-sitter comparison:

    1. **Both sides parse cleanly and contain NO control construct at all** —
       stricter than ``semantic_skeleton``'s ``control_exact``, which would
       happily accept an ``if`` on both sides.
    2. **Shape counters agree** for assignments, member accesses and calls
       (``semantic_skeleton.extract_skeleton``), measured on equivalent
       single-function probes so neither the ``#[cfg]`` dual nor the attribute
       scaffolding can skew a count.
    3. **The ordered identifier sequence is identical.** This is the check that
       actually catches a bad forwarder: reordered arguments, a substituted
       literal, a dropped argument, or a renamed callee all perturb the sequence
       even though they preserve every counter above.
    4. **Constants survive exactly.** Every constant occurrence in the residual C
       must appear in the Rust body with the same multiplicity, and the Rust body
       must contain no constant the C did not — the ``fact_seal``
       missing/invented invariant applied to a deterministic lowering. This is
       what makes folding (``38 + 1`` -> ``39``) or re-basing (``0x200`` ->
       ``512``) impossible to ship unnoticed.
    5. **The emitted item really defines this C function**, with the same arity.

    A null pointer is the one atom whose two spellings cannot compare equal —
    C's `NULL` is one `null` node and an uppercase constant, Rust's
    `core::ptr::null_mut()` is three identifiers plus a call. Both sides are
    rewritten to `_NULL_SENTINEL` before checks 2/3/4 so the comparison stays
    token-for-token everywhere else: a `NULL` that moved position, was dropped,
    or replaced a real argument still perturbs the identifier sequence. (The one
    thing the sentinel cannot distinguish is `null()` from `null_mut()`, which is
    a *type* error rustc catches at candidate-check, not a silent miscompile.)

    Returns a list of findings; empty means proven.
    """
    findings: list[str] = []
    residual_pair = _residual_body(dep.body_text, keep_diagnostics)
    if residual_pair is None:
        return ["residual C body is no longer recoverable"]
    residual, dropped = residual_pair
    if dropped != lowering.dropped_debug:
        findings.append("dropped debug statements do not match the C body")

    rust_parsed = _block_body(lowering.rust)
    if rust_parsed is None:
        return [*findings, "generated Rust has no function block"]
    _, _, rust_inner_raw = rust_parsed
    # A kept diagnostic is the C statement re-spelled as a macro, not part of the residual
    # the checks below compare: it must be present exactly as rendered, and is then removed.
    for text in dropped:
        kept = _kept_rendering(text) if keep_diagnostics else None
        if kept is None:
            continue
        if kept not in rust_inner_raw:
            findings.append(f"kept diagnostic missing from the Rust: {kept}")
        rust_inner_raw = rust_inner_raw.replace(kept, " " * len(kept), 1)
    # Canonicalize the one atom whose spellings cannot match (see the docstring).
    # Re-parse from the rewritten text so the node offsets belong to what we compare.
    # Both Rust spellings fold: `core::ptr::null_mut()` (forwarder_with_null) and a
    # bare `NULL` token (constant_return, where the crate supplies the constant and
    # the C spelling is copied through verbatim).
    residual = _RE_C_NULL.sub(_NULL_SENTINEL, residual)
    rust_inner = _RE_C_NULL.sub(_NULL_SENTINEL, _RE_RUST_NULL.sub(_NULL_SENTINEL, rust_inner_raw))
    rust_canon = _block_body(f"fn {_PROBE}() {{{rust_inner}}}")
    if rust_canon is None:
        return [*findings, "generated Rust has no function block"]
    rust_block, rust_bytes, rust_inner = rust_canon
    c_parsed = _c_probe(residual)
    if c_parsed is None:
        return [*findings, "residual C statements do not parse"]
    c_block, c_bytes, c_probe_text = c_parsed

    item_skeleton = extract_skeleton(lowering.rust, RUST_LANG)
    if item_skeleton.parse_errors:
        findings.append(f"generated Rust does not parse ({item_skeleton.parse_errors} errors)")
    c_skeleton = extract_skeleton(c_probe_text, CPP_LANG)
    rust_skeleton = extract_skeleton(f"fn {_PROBE}() {{{rust_inner}}}", RUST_LANG)
    if c_skeleton.parse_errors:
        findings.append(f"residual C does not parse ({c_skeleton.parse_errors} errors)")
    if c_skeleton.controls or rust_skeleton.controls:
        findings.append(
            f"body is not control-free: c={c_skeleton.controls} rust={rust_skeleton.controls}"
        )
    for field in _SKELETON_FIELDS:
        left, right = getattr(c_skeleton, field), getattr(rust_skeleton, field)
        if left != right:
            findings.append(f"{field} count differs: c={left} rust={right}")

    c_identifiers = _ordered_identifiers(c_block, c_bytes, _C_IDENTIFIER_NODES)
    rust_identifiers = _ordered_identifiers(rust_block, rust_bytes, _RUST_IDENTIFIER_NODES)
    if c_identifiers != rust_identifiers:
        findings.append(f"identifier sequence differs: c={c_identifiers} rust={rust_identifiers}")

    c_constants, rust_constants = _constants(residual), _constants(rust_inner)
    missing, invented = c_constants - rust_constants, rust_constants - c_constants
    if missing:
        findings.append("constants lost: " + ", ".join(sorted(missing.elements())))
    if invented:
        findings.append("constants invented: " + ", ".join(sorted(invented.elements())))

    if re.search(rf"\bfn\s+{re.escape(dep.name)}\s*\(", lowering.rust) is None:
        findings.append(f"generated Rust does not define {dep.name}")
    parameters = _first_node(
        Parser(RUST_LANG).parse(lowering.rust.encode("utf-8", "replace")).root_node, "parameters"
    )
    arity = 0 if parameters is None else len(parameters.named_children)
    expected = len(_normalized_params(dep.params))
    if arity != expected:
        findings.append(f"parameter arity differs: c={expected} rust={arity}")
    return findings
