"""C/C++ source parser using tree-sitter for structural analysis.

C++ (.cpp/.cc/.cxx) is parsed with tree-sitter-cpp, a superset grammar. The
L2 "C-as-C++" tier (uniata, udfs — C code in .cpp clothing whose only C++-ism
is `extern "C"` linkage blocks) translates through the same per-function path
as C; the only structural delta is flattening `linkage_specification` nodes so
their contents are treated as top-level (see scan_file).
"""

import re
from dataclasses import dataclass, field
from pathlib import Path

import tree_sitter_c as tsc
import tree_sitter_cpp as tscpp
from tree_sitter import Language, Parser

from .conventions import KW_EXPANSION, KW_EXPANSION_FN, KW_TO_CONV
from .lex_mask import mask_code
from .version_macros import repo_root_of, version_defines

C_LANG = Language(tsc.language())
CPP_LANG = Language(tscpp.language())

# Extensions that select the C++ grammar.
# `.hpp`/`.hxx` are C++ headers: parsed as C, a function returning a reference (`audio_format &
# audio_membuffer__audinfo(..)`) vanished from the plan without a word.
_CPP_SUFFIXES = (".cpp", ".cc", ".cxx", ".c++", ".hpp", ".hxx", ".hh")

# Every macro that ALIASES a calling convention, from the ONE shared table
# (conventions.py — verified against the real SDK headers by tools/verify_conv_macros.py).
# This used to be a hand-written set of 7 that did not know WSAAPI/WSPAPI/SEC_ENTRY/…, so a
# ws2_32 fn declared `SOCKET WSAAPI socket(...)` came back with calling_convention=None.
CALLING_CONVENTIONS = set(KW_TO_CONV) | {"__cdecl", "__stdcall", "__fastcall"}

# Section markers in ReactOS C files
RE_GLOBALS_SECTION = re.compile(r"/\*\s*GLOBALS\s*\*/")
RE_FUNCTIONS_SECTION = re.compile(r"/\*\s*FUNCTIONS\s*\*/")

# Arch preprocessor conditionals
RE_IFDEF_ARCH = re.compile(r"#\s*if(?:def|ndef)?\s+.*?(_M_ARM64|_M_AMD64|_M_IX86|_M_ARM|_WIN64)")

# SEH macros
RE_SEH_TRY = re.compile(r"_SEH2_TRY")
RE_SEH_EXCEPT = re.compile(r"_SEH2_EXCEPT")

# CODE_SEG
RE_CODE_SEG = re.compile(r'CODE_SEG\("(\w+)"\)')
RE_CODE_SEG_BYTES = re.compile(rb'\bCODE_SEG\(\s*"[^"]*"\s*\)')

# CSR_API(n) is a macro (sdk/include/reactos/subsys/csr/csrsrv.h) that expands to
#   NTSTATUS NTAPI n(IN OUT PCSR_API_MESSAGE ApiMessage,
#                    IN OUT PCSR_REPLY_CODE ReplyCode OPTIONAL)
# tree-sitter parses `CSR_API(name){...}` as an expression_statement + block,
# NOT a function_definition, so the scanner skips every CSR server API handler
# (all of basesrv dosdev/vdm's BaseSrv* entry points; same for winsrv consrv/
# usersrv). Splice it to the real signature before parsing so each handler is
# seen as a normal function. The param names (ApiMessage/ReplyCode) match the
# macro, so the captured bodies still resolve. Plain-C splice — safe per the
# differential-oracle rule. NB: \bCSR_API\s*\( won't match CSR_API_NUMBER_*().
RE_CSR_API_DEF = re.compile(r"\bCSR_API\s*\(\s*(\w+)\s*\)")


def _expand_csr_api(text: str) -> str:
    """Rewrite `CSR_API(Name)` → the real NTAPI signature so tree-sitter sees a
    function_definition (otherwise the CSR API handlers are silently skipped)."""
    return RE_CSR_API_DEF.sub(
        r"NTSTATUS NTAPI \1(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)",
        text,
    )


# SAL (Source Annotation Language) tokens. The argless ones (`_In_`, `_Out_`)
# tree-sitter tolerates, but the ones WITH arguments — `_Inout_updates_z_(N)`,
# `_In_reads_or_z_opt_(N)`, `_Success_(return)`, `_Out_writes_bytes_(cb)` — used
# in a parameter list or a local-variable declaration make tree-sitter-c parse
# the whole function_definition as an ERROR node, so the function is silently
# skipped (concfg/font.c: only 2 of 14 fns found). SAL macros are no-ops outside
# static analysis, so stripping them is semantically identical to what the C
# compiler sees and can only REDUCE parse errors (differential-oracle safe).
# Token grammar: `_<Word>[_<word>...]_` (starts `_` + a SAL keyword, ends `_`),
# optionally followed by a nested, balanced annotation argument list.
_SAL_KEYWORDS = (
    "In",
    "Out",
    "Inout",
    "Outptr",
    "Outref",
    "Ret",
    "Pre",
    "Post",
    "Deref",
    "Field",
    "At",
    "When",
    "Success",
    "Return",
    "Reserved",
    "Notnull",
    "Maybenull",
    "Null",
    "Const",
    "Check_return",
    "Must_inspect_result",
    "Analysis_noreturn",
    "Use_decl_annotations",
    "Printf_format_string",
    "Struct_size_bytes_",
    "Frees_ptr",
    "Outptr_result",
    "Result_nullonfailure",
)
RE_SAL = re.compile(
    r"(?<![A-Za-z0-9_])"
    r"_(?:" + "|".join(_SAL_KEYWORDS) + r")"
    r"[A-Za-z0-9_]*_"  # rest of the SAL token, ending in '_'
)
RE_LEGACY_SAL = re.compile(r"(?<![A-Za-z0-9_])(?:IN|OUT|OPTIONAL)(?![A-Za-z0-9_])")


def _strip_sal(text: str) -> str:
    """Remove SAL annotations (no-op macros) so tree-sitter parses the function
    signatures of annotated Windows headers. Safe: only removes annotations."""
    masked = mask_code(text)
    spans = []
    consumed = 0
    for match in RE_SAL.finditer(masked):
        if match.start() < consumed:
            continue
        end = match.end()
        opening = end
        while opening < len(masked) and masked[opening].isspace():
            opening += 1
        if opening < len(masked) and masked[opening] == "(":
            depth = 1
            end = opening + 1
            while end < len(masked) and depth:
                depth += (masked[end] == "(") - (masked[end] == ")")
                end += 1
            if depth:
                raise ValueError(f"unbalanced SAL annotation {match.group()}")
        spans.append((match.start(), end))
        consumed = end
    spans.extend((m.start(), m.end()) for m in RE_LEGACY_SAL.finditer(masked))
    result = list(text)
    for start, end in spans:
        for index in range(start, end):
            if result[index] != "\n":
                result[index] = " "
    return "".join(result)


# Win32 calling-convention macros that tree-sitter-c CANNOT parse between the
# return type and the function name (it recovers catastrophically — e.g. a
# `static BOOL CALLBACK Foo(){...}` makes tree-sitter merge every following
# function into one giant function_definition, so concfg/font.c found 2 of 14
# fns). They are all __stdcall, which tree-sitter DOES parse (ms_call_modifier).
# NTAPI/WINAPI are intentionally NOT normalized here: their error-recovery is
# benign (separate function_definition nodes survive) and they're already in
# CALLING_CONVENTIONS, so the existing scans depend on seeing them verbatim.
# Derived from the ONE shared table, not hand-listed — the hand-listed version knew 5 of
# these and so could not parse `CHAR FAR* WSAAPI inet_ntoa(...)` (ws2_32) at all.
#
# BENIGN_VERBATIM are the aliases tree-sitter recovers from cleanly (separate
# function_definition nodes survive). They are left alone because existing scans depend on
# seeing them verbatim — CALLING_CONVENTIONS matches the token as written.
_BENIGN_VERBATIM = frozenset({"NTAPI", "WINAPI", "FASTCALL", "CDECL", "STDMETHODCALLTYPE"})
_NORMALIZE_TO = {"system": "__stdcall", "cdecl": "__cdecl", "fastcall": "__fastcall"}
_UNPARSEABLE = sorted(
    (k for k, v in KW_TO_CONV.items() if k not in _BENIGN_VERBATIM and not k.startswith("_")),
    key=len,
    reverse=True,
)
_CC_ALT = "|".join(re.escape(k) for k in _UNPARSEABLE)
RE_UNPARSEABLE_CC = re.compile(r"\b(?:" + _CC_ALT + r")\b")

# A convention keyword sitting between a POINTER return type and the name — the one
# position tree-sitter-c cannot parse in any spelling:
#
#     CHAR * __stdcall inet_ntoa(...)     <- FAILS  (no function_definition node at all)
#     CHAR __stdcall * inet_ntoa(...)     <- parses
#     CHAR *           inet_ntoa(...)     <- parses
#
# ws2_32's `CHAR FAR* / WSAAPI / inet_ntoa` lands exactly there, so mapping the alias to
# `__stdcall` (which fixes the non-pointer case) *creates* this one. Drop the keyword here
# instead. Lossless in practice: the scanner's `calling_convention` field is already None
# for every one of these shapes (it is documented lossy on own-line macros), and the
# authoritative convention is re-derived from the RAW C text by abi_enforce._derive_convention,
# which never sees this rewrite.
#
# The alternation here is EVERY convention keyword, including the _BENIGN_VERBATIM ones
# that rule 2 deliberately leaves alone. That exemption exists because those aliases
# recover cleanly ELSEWHERE and existing scans match them as written — but "elsewhere" is
# the whole of its justification, and this position is the exception the block above
# documents: unparseable "in any spelling", no function_definition node at all. Nothing
# can depend on seeing a token in a function the scanner never produced.
#
# Leaving CDECL out of this cost msvcrt 15 functions. `FILE * CDECL fopen(...)` is Wine's
# dominant spelling, so file.c's fopen/freopen/tmpfile/_fdopen/__iob_func and ten more were
# absent from the translation plan — silently, because a dropped function looks exactly
# like a function that was never there. That is worse than a partial file: file.c's
# entry points share one fd table, so a C `fopen` beside a Rust `fclose` is split state.
_CC_ALT_ANY = "|".join(re.escape(k) for k in sorted(KW_TO_CONV, key=len, reverse=True))
RE_CC_AFTER_PTR = re.compile(r"(\*\s*)\b(?:" + _CC_ALT_ANY + r")\b")


def _normalize_callbacks(text: str) -> str:
    """Make the tree-sitter-unparseable convention aliases parse.

    Two rules, in order:
      1. after a pointer return type -> DROP (the position tree-sitter cannot parse at all)
      2. anywhere else               -> map to the raw `__stdcall`/`__cdecl` keyword

    Differential-oracle safe (same ABI, only helps): an alias and its target are the same
    convention by definition, and abi_enforce derives the real convention from raw C.

    Without this, tree-sitter recovers CATASTROPHICALLY — `static BOOL CALLBACK Foo(){...}`
    made it merge every following function into one giant function_definition, so
    concfg/font.c yielded 2 of 14 fns. ws2_32 hit the same wall through `WSAAPI`.
    """
    text = RE_CC_AFTER_PTR.sub(r"\1", text)
    return RE_UNPARSEABLE_CC.sub(lambda m: _NORMALIZE_TO[KW_TO_CONV[m.group(0)]], text)


# The 16-bit memory-model decorations. Both are `#define`d to NOTHING (minwindef.h:129),
# but tree-sitter does not know that, and a parameter spelled `CONST CHAR FAR* cp` is two
# type-identifiers before the `*` — which it cannot disambiguate, so it fails the ENTIRE
# enclosing function_definition. The function is then silently dropped from the scan.
#
# That is not hypothetical: `inet_addr(IN CONST CHAR FAR* cp)` (addrconv.c:70) was reported
# as an unextractable body named `cp`, and inet_addr — an *exported* ws2_32 fn that
# nettest.exe itself calls — was simply absent from the translation plan. `FAR` is all over
# the winsock/mswsock/wsock32 signatures, so this drops functions across the whole family.
#
# Blank them before parsing: they carry no ABI meaning on any target we build (flat 32/64-
# bit), so this is differential-oracle safe in the same way _normalize_callbacks is.
RE_EMPTY_MEMMODEL = re.compile(r"\b(?:FAR|NEAR)\b")

# `EXTERN_C` is winnt.h's `#define EXTERN_C extern "C"` (C++) / `extern` (C). tree-sitter
# does not know it, so it is parsed as a type name and the real return type becomes a second
# type-identifier. The enclosing function then depends on tree-sitter's ERROR RECOVERY, whose
# repair cost is length-sensitive: with a pointer return type the function is recovered while
# the type name is <=9 characters and LOST at >=10 — `EXTERN_C FOOBAR*` extracts,
# `EXTERN_C FILEDESCRIPTOR*` does not. That cliff is why shell32's stubs.cpp reported exactly
# 2 of 66 bodies unextractable (GetFileDescriptor, SHGetShellFolderViewCB) — both pointer
# returns, both names >=10 chars, while every VOID/BOOL stub beside them parsed.
#
# Substitute the macro's real expansion rather than blanking it. Blanking looks equivalent
# and is NOT: `FILEDESCRIPTOR* WINAPI Bar(...)` with no linkage specifier parses as an
# ambiguous declaration and yields ZERO function_definition nodes AND an EMPTY `unextracted`
# list — a silent drop, the failure mode this scanner exists to prevent.
RE_EXTERN_C = re.compile(r"\bEXTERN_C\b")

# The convention aliases that are really `[linkage] <TYPE> <conv>` — see KW_EXPANSION for
# why normalizing one to a bare `__stdcall` deletes the function.
RE_COMPOSITE_CC = re.compile(r"\b(?:" + "|".join(map(re.escape, KW_EXPANSION)) + r")\b")

# The function-like forms take their return type as an argument, so they need a capture
# rather than a word match. Longest-first, or `STDAPI_` would shadow `STDAPIV_`. The
# argument is a type, so it cannot itself contain a paren -- a flat `[^()]+` is exact here.
RE_COMPOSITE_CC_FN = re.compile(
    r"\b("
    + "|".join(sorted(map(re.escape, KW_EXPANSION_FN), key=len, reverse=True))
    + r")\s*\(\s*([^()]+?)\s*\)"
)

# `CONST` is `#define CONST const` (winnt.h) — same macro, uppercase. tree-sitter-c does not
# know it, so `CONST struct sockaddr *name` reads as TWO type-identifiers and the enclosing
# function produces ZERO function_definition nodes: it is silently dropped from the plan.
#
# That is not hypothetical either. It ate `bind` (socklife.c), `connect` (sockctrl.c) and
# `sendto` (send.c) — the three core socket calls, all three of which take
# `IN CONST struct sockaddr *name` — so ws2_32 was translated, compiled clean on all three
# arches, and passed every check while missing the functions a socket client cannot work
# without. Only the .spec export audit caught it. Normalise to the lowercase keyword.
RE_UPPER_CONST = re.compile(r"\bCONST\b")

# `#define DECLSPEC_HIDDEN` (winnt_old.h:78) expands to NOTHING, but tree-sitter reads it
# as a second type-identifier and fails the declaration it trails — and a failed top-level
# declaration makes tree-sitter hoist the NEXT function's body to the root, where
# preamble_audit reports its locals as dropped file-scope state. wldap32.h:28
# (`ULONG map_error( int ) DECLSPEC_HIDDEN;`) is the live case; 168 files in the tree carry
# the macro. Same no-ABI-meaning, blank-it-before-parsing rule as FAR/NEAR above.
# Section placement for DATA has no ABI meaning either: `DATA_SEG("PAGECONS")` and the
# per-driver aliases spelled `<DRIVER>_PAGED_DATA` (pciidex/atapi `#define X_PAGED_DATA
# DATA_SEG("PAGECONS")`). tree-sitter cannot parse a call-shaped attribute in front of
# `static const struct {...} T[] = {...}`, so the table read as an unextractable FUNCTION
# and a function holding such a static local fell out of the plan entirely.
RE_EMPTY_ATTR = re.compile(
    r"\bDECLSPEC_HIDDEN\b|\bDATA_SEG\(\s*\"[^\"]*\"\s*\)|\b[A-Z][A-Z0-9]*_PAGED_DATA\b"
)

# ATL spells in-class COM method definitions as macro declarators:
#
#     STDMETHOD(QueryService)(REFGUID svc, REFIID riid, void **ppv) override
#     {
#         ...
#     }
#
# The compiler expands this to an ordinary HRESULT method, but tree-sitter-cpp
# sees a function_definition whose declarator is just ``STDMETHOD`` and records
# the real method as an unextractable body.  Normalize only the definition shape
# (a balanced-looking parameter list followed by an optional ``override`` and
# ``{``); declarations ending in ``;`` are deliberately untouched.
RE_INLINE_STDMETHOD = re.compile(
    r"\bSTDMETHOD\s*\(\s*(?P<name>[A-Za-z_]\w*)\s*\)\s*"
    r"\((?P<params>[^;{}]*)\)\s*(?:override\s*)?(?=\{)",
    re.S,
)
RE_INLINE_STDMETHOD_TYPED = re.compile(
    r"\bSTDMETHOD_\s*\(\s*(?P<ret>[^,()]+?)\s*,\s*"
    r"(?P<name>[A-Za-z_]\w*)\s*\)\s*"
    r"\((?P<params>[^;{}]*)\)\s*(?:override\s*)?(?=\{)",
    re.S,
)


def _expand_inline_stdmethod(text: str) -> str:
    """Expose ATL inline COM definitions to the C++ structural scanner."""
    text = RE_INLINE_STDMETHOD_TYPED.sub(
        lambda match: (
            f"{match.group('ret').strip()} {match.group('name')}({match.group('params')}) "
        ),
        text,
    )
    return RE_INLINE_STDMETHOD.sub(
        lambda match: f"HRESULT {match.group('name')}({match.group('params')}) ", text
    )


def _blank_memmodel(text: str) -> str:
    """Blank the empty `FAR`/`NEAR`/`DECLSPEC_HIDDEN` decorations and normalise `CONST`
    so tree-sitter can parse the declaration. All are no-op/aliased Win32 macros with no
    ABI meaning, and each one tree-sitter does not know costs a whole declaration."""
    return RE_UPPER_CONST.sub("const", RE_EMPTY_ATTR.sub("", RE_EMPTY_MEMMODEL.sub("", text)))


def _expand_composite_conventions(text: str) -> str:
    """Expand the aliases that carry a RETURN TYPE, not just a convention.

    Must run BEFORE `_expand_extern_c` and `_normalize_callbacks`, because the expansion is
    the header's verbatim text and hands each of them their own work: `STDAPI` becomes
    `EXTERN_C HRESULT STDAPICALLTYPE`, whose `EXTERN_C` the former resolves and whose
    `STDAPICALLTYPE` the latter maps to `__stdcall`. Encoding the finished form here
    instead would duplicate both, and get the C/C++ linkage split wrong.

    Fixpoint, because the headers chain them (`WINOLEAPI` -> `STDAPI` -> …). Bounded: the
    chains are two deep and a cycle would be a header bug, so stop rather than spin.
    """
    for _ in range(4):
        expanded = RE_COMPOSITE_CC_FN.sub(
            lambda m: re.sub(r"\\1", m.group(2), KW_EXPANSION_FN[m.group(1)]), text
        )
        expanded = RE_COMPOSITE_CC.sub(lambda m: KW_EXPANSION[m.group(0)], expanded)
        if expanded == text:
            return text
        text = expanded
    return text


def _expand_extern_c(text: str, is_cpp: bool) -> str:
    """Expand `EXTERN_C` to the linkage specifier it is #defined to.

    See RE_EXTERN_C: leaving it as an unknown identifier makes extraction depend on
    tree-sitter's length-sensitive error recovery, and deleting it silently drops the
    function. The replacement is the macro's own definition, so it is exact — and it
    preserves line numbers (same line, no newline added).
    """
    return RE_EXTERN_C.sub('extern "C"' if is_cpp else "extern", text)


@dataclass
class CFunction:
    name: str
    return_type: str
    calling_convention: str | None  # "NTAPI", "FASTCALL", None
    params: list[tuple[str, str]]  # [(type, name), ...]
    is_static: bool
    code_seg: str | None  # "INIT", "PAGE", etc.
    seh_count: int  # number of _SEH2_TRY blocks
    line_start: int
    line_end: int
    body_text: str  # raw C source of the function body
    # C++ method support (explorer/shell translation): the owning class for
    # out-of-line (`Cls::m`) and in-class method definitions. None for free
    # functions. When set, `name` is the plain method name and the translation
    # convention is `extern "C" fn <Cls>_<name>(this: *mut <Cls>, …)` — see
    # docs/engineering/explorer-rust-translation-plan.md.
    cpp_class: str | None = None
    is_ctor: bool = False
    is_dtor: bool = False

    @property
    def display_name(self) -> str:
        return f"{self.cpp_class}::{self.name}" if self.cpp_class else self.name


@dataclass
class CGlobal:
    name: str
    type_str: str
    initializer: str | None
    line: int


@dataclass
class ScanResult:
    path: Path
    globals: list[CGlobal] = field(default_factory=list)
    functions: list[CFunction] = field(default_factory=list)
    identifiers: set[str] = field(default_factory=set)
    local_names: set[str] = field(default_factory=set)  # body locals + params
    arch_ifdefs: list[str] = field(default_factory=list)  # arch macros used
    includes: list[str] = field(default_factory=list)
    defines: list[str] = field(default_factory=list)
    raw_source: str = ""
    globals_text: str = ""  # text between GLOBALS and FUNCTIONS markers
    # File-scope preamble (typedefs, structs/enums/unions, #define, static
    # tables, file-scope globals) derived from the AST as the complement of
    # function bodies + #includes. Marker-INDEPENDENT (unlike globals_text,
    # which only populates when the C has /* GLOBALS */ // /* FUNCTIONS */
    # comments). This is what the `translate` pipeline feeds through the LLM
    # as a synthetic preamble unit so the stitched .rs actually compiles.
    preamble_text: str = ""
    # Function definitions inside a MALFORMED preamble node (an ERROR, or an unbalanced `#if`
    # spanning bodies): a parse defect, never a real file-scope construct.
    preamble_swallowed_functions: int = 0
    # Function bodies the scanner SAW but could not extract — C++ methods
    # (out-of-line `Class::m` qualified declarators, in-class definitions,
    # ctors/dtors/operators). Entries are "name @ Lstart". A non-empty list
    # means per-function translation would SILENTLY cover only part of the
    # file (explorer tier-C files: 1-6 of dozens) — `translate`/`submit`
    # hard-fail on it unless --allow-partial (fail-loud rule, AGENTS.md).
    unextracted: list[str] = field(default_factory=list)


def _extract_text(node, source: bytes) -> str:
    return source[node.start_byte : node.end_byte].decode("utf-8", errors="replace")


def _collect_identifiers(node, source: bytes, identifiers: set[str]) -> None:
    """Recursively collect all identifier tokens from a tree-sitter node."""
    if node.type == "identifier" or node.type == "type_identifier":
        name = _extract_text(node, source)
        if len(name) > 1:  # skip single-char vars
            identifiers.add(name)
    for child in node.children:
        _collect_identifiers(child, source, identifiers)


_DECLARATOR_TYPES = (
    "init_declarator",
    "pointer_declarator",
    "array_declarator",
    "parenthesized_declarator",
    "identifier",
)


def _declarator_name(node, source: bytes) -> str | None:
    """Descend a declarator subtree to the declared identifier name."""
    if node.type == "identifier":
        return _extract_text(node, source)
    for child in node.children:
        if child.type in _DECLARATOR_TYPES:
            n = _declarator_name(child, source)
            if n:
                return n
    return None


def _collect_local_decls(node, source: bytes, names: set[str]) -> None:
    """Collect identifiers that are *defining declarators* of local
    variable / parameter declarations (not types, not calls).

    These are never FFI imports — the per-file identifier scanner
    otherwise reports Hungarian-cased locals (AnsiString, AdminSid, …)
    as phantom "unresolved". Generic: helps every target's resolve
    accuracy, not just smss-csrss.
    """
    if node.type in ("declaration", "parameter_declaration"):
        for child in node.children:
            if child.type in _DECLARATOR_TYPES:
                nm = _declarator_name(child, source)
                if nm:
                    names.add(nm)
    for child in node.children:
        _collect_local_decls(child, source, names)


def _matching_brace_byte(source: bytes, open_byte: int) -> int | None:
    """Byte offset of the ``}`` that closes the ``{`` at ``open_byte`` (or None).

    A small brace matcher that skips ``//`` and ``/* */`` comments and string /
    char literals so their braces don't perturb the count.  Used to bound an ATL
    class body that tree-sitter only recovered as a flat ERROR node — that node
    keeps the ``{`` child token but not the matching ``}`` (see
    ``_error_node_class_owner``).
    """
    depth = 0
    i = open_byte
    n = len(source)
    while i < n:
        ch = source[i]
        if ch == 0x2F and i + 1 < n and source[i + 1] == 0x2F:  # //
            nl = source.find(b"\n", i)
            i = n if nl < 0 else nl + 1
            continue
        if ch == 0x2F and i + 1 < n and source[i + 1] == 0x2A:  # /* ... */
            end = source.find(b"*/", i + 2)
            i = n if end < 0 else end + 2
            continue
        if ch in (0x22, 0x27):  # " or ' literal
            i += 1
            while i < n:
                if source[i] == 0x5C:  # escape
                    i += 2
                    continue
                if source[i] == ch:
                    i += 1
                    break
                i += 1
            continue
        if ch == 0x7B:  # {
            depth += 1
        elif ch == 0x7D:  # }
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def _error_node_class_owner(err_node, method_node, source: bytes) -> str | None:
    """Owning class name for a method definition flattened into an ERROR node.

    Macro-heavy multi-class ATL headers can defeat tree-sitter's class parse,
    leaving every ``class NAME : ... {`` header and in-class method as flat
    children of one ERROR node.  The class keyword, its name, and its opening
    brace survive as tokens (the matching ``}`` does not), so recover the owner
    by brace-balanced containment: the class whose ``{`` .. ``}`` body encloses
    ``method_node``.  Taking the innermost such class means a helper defined
    *after* a class's closing brace is left unowned rather than mis-attributed
    (the reason the earlier, cruder recovery deliberately handled only `struct`).
    """
    target = method_node.start_byte
    kids = list(err_node.children)
    owner: str | None = None
    for i, child in enumerate(kids):
        if child.type not in ("class", "struct"):
            continue
        name_tok = next((k for k in kids[i + 1 :] if k.type == "type_identifier"), None)
        if name_tok is None:
            continue
        brace = next(
            (k for k in kids[i + 1 :] if k.type == "{" and k.start_byte > name_tok.start_byte),
            None,
        )
        if brace is None or brace.start_byte >= target:
            continue
        close = _matching_brace_byte(source, brace.start_byte)
        if close is not None and brace.start_byte < target < close:
            owner = _extract_text(name_tok, source)
    return owner


def _parse_function(node, source: bytes, source_text: str) -> CFunction | None:
    """Parse a function_definition node into CFunction."""
    name = None
    return_type = ""
    calling_convention = None
    params = []
    is_static = False
    code_seg = None

    # Check for storage class and CODE_SEG in the preceding lines
    start_line = node.start_point[0]
    # Look at up to 5 preceding lines for CODE_SEG / static
    line_start = max(0, start_line - 5)
    preceding = "\n".join(source_text.splitlines()[line_start : start_line + 1])
    # ...or inside the declaration itself, between the return type and the name.
    header = _extract_text(node, source).split("{", 1)[0]
    m = RE_CODE_SEG.search(preceding) or RE_CODE_SEG.search(header)
    if m:
        code_seg = m.group(1)

    # Walk the declaration
    declarator = None
    for child in node.children:
        text = _extract_text(child, source)

        if child.type == "storage_class_specifier" and text == "static":
            is_static = True
        elif (
            child.type == "type_identifier"
            or child.type == "primitive_type"
            or child.type == "sized_type_specifier"
        ):
            if text in CALLING_CONVENTIONS:
                calling_convention = text
            else:
                return_type = text
        elif child.type == "identifier":
            if text in CALLING_CONVENTIONS:
                calling_convention = text
            elif return_type == "":
                return_type = text
        elif child.type == "qualified_identifier" and return_type == "":
            # A nested C++ return type is a top-level type node, distinct from
            # the qualified method name inside the function declarator:
            # `CNewMenu::ITEM *CNewMenu::LoadItem(...)`.
            return_type = text
        elif (
            child.type in ("struct_specifier", "union_specifier", "enum_specifier")
            and return_type == ""
        ):
            # An ELABORATED type: `struct berval *bvdup(...)`. Neither a
            # type_identifier nor an identifier, so the base type was dropped and
            # bvdup's return type came out as bare `*` — a pointer to nothing.
            # Take the tag with its keyword; the pointer stars are appended below.
            name_node = child.child_by_field_name("name")
            if name_node is not None:
                kind = child.type.split("_", 1)[0]
                return_type = f"{kind} {_extract_text(name_node, source)}"
        elif child.type == "function_declarator":
            declarator = child
        elif child.type in ("pointer_declarator", "reference_declarator"):
            # Return type is a pointer and/or a reference. tree-sitter NESTS one declarator
            # per token, so `char **f(...)` is pointer_declarator(pointer_declarator(
            # function_declarator)) and the function_declarator is a GRANDchild.
            # Looking only at direct children silently dropped every T** function:
            # 10 of wldap32.h's 62 marshalling helpers (strarrayWtoU, controlarrayAtoW,
            # modarrayAtoW, …) were reported as unextractable "C++ methods" in a C
            # header. They are `static inline`, so the .spec export cross-check could
            # not catch it either — the run just translated 52 of 62 and said so in a
            # warning. Descend instead, recording each token as we go.
            #
            # A REFERENCE return nests identically and had no case here at all, so EVERY
            # `T& f() {...}` was dropped — not a rare shape, it is how C++ spells a
            # singleton accessor. CDefView's `static ATL::CWndClassInfo& GetWndClassInfo()`
            # is the instance that held a 4,869-line TU out of paid translation, and the
            # return type is irrelevant to the failure: a plain `int&` dropped too.
            # Keeping the `&` in the spelling is what lets c_type_to_rust lower it, since
            # that function already reads a trailing `&` and emits `&mut T` / `&T`.
            sub = child
            suffix = ""
            while sub is not None and sub.type in ("pointer_declarator", "reference_declarator"):
                suffix += "*" if sub.type == "pointer_declarator" else "&"
                sub = next(
                    (
                        c
                        for c in sub.children
                        if c.type
                        in ("pointer_declarator", "reference_declarator", "function_declarator")
                    ),
                    None,
                )
            if sub is not None and sub.type == "function_declarator":
                declarator = sub
                return_type = (return_type + " " + suffix).strip()

    cpp_class = None
    is_ctor = False
    is_dtor = False
    recovered_declarator = False

    if declarator is None and node.type == "function_definition":
        # tree-sitter-cpp parses some perfectly ordinary out-of-line methods
        # without a calling-convention token as ``field_identifier`` + ERROR:
        #
        #     BOOL CThing::Method(DWORD flags) { ... }
        #
        # Recover only that narrow, compiler-unambiguous surface from the text
        # before the compound body.  This keeps the class-atomic fail-loud gate
        # useful without accepting arbitrary malformed C++.
        body_node = next((c for c in node.children if c.type == "compound_statement"), None)
        if body_node is not None:
            signature = source[node.start_byte : body_node.start_byte].decode(
                "utf-8", errors="replace"
            )
            recovered = re.fullmatch(
                r"\s*(?P<prefix>.*?)\b(?P<class>[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*)"
                r"::(?P<name>~?[A-Za-z_]\w*)\s*\((?P<params>.*)\)\s*",
                signature,
                flags=re.S,
            )
            if recovered:
                prefix_tokens = recovered.group("prefix").split()
                is_static = "static" in prefix_tokens
                prefix_tokens = [token for token in prefix_tokens if token != "static"]
                for token in tuple(prefix_tokens):
                    if token in CALLING_CONVENTIONS:
                        calling_convention = token
                        prefix_tokens.remove(token)
                return_type = " ".join(prefix_tokens)
                cpp_class = recovered.group("class")
                name = recovered.group("name")
                is_dtor = name.startswith("~")
                name = name.lstrip("~")
                is_ctor = not is_dtor and name == cpp_class.rpartition("::")[2]

                raw_params = recovered.group("params").strip()
                if raw_params and raw_params != "VOID":
                    depth = 0
                    pieces: list[str] = []
                    start = 0
                    for index, char in enumerate(raw_params):
                        if char in "(<[":
                            depth += 1
                        elif char in ")>]":
                            depth -= 1
                        elif char == "," and depth == 0:
                            pieces.append(raw_params[start:index].strip())
                            start = index + 1
                    pieces.append(raw_params[start:].strip())
                    for raw_param in pieces:
                        parts = raw_param.rsplit(None, 1)
                        if len(parts) == 2:
                            ptype, pname = parts
                            while pname.startswith("*"):
                                ptype += " *"
                                pname = pname[1:]
                            params.append((ptype.strip(), pname.strip()))
                        elif "*" in raw_param:
                            adjacent = re.match(r"(?P<type>.+\*)(?P<name>[A-Za-z_]\w*)$", raw_param)
                            if adjacent:
                                params.append(
                                    (adjacent.group("type").strip(), adjacent.group("name"))
                                )
                            else:
                                params.append((raw_param, ""))
                        else:
                            params.append((raw_param, ""))

                declarator = node  # mark the deliberately recovered shape as parsed
                recovered_declarator = True

    if declarator is None:
        return None

    # Extract function name from declarator
    for child in () if recovered_declarator else declarator.children:
        if child.type == "identifier":
            name = _extract_text(child, source)
        elif child.type == "field_identifier":
            # In-class C++ method definition (`void m() {…}` inside a class
            # body). Class name comes from the enclosing specifier below.
            name = _extract_text(child, source)
        elif child.type == "qualified_identifier":
            # Out-of-line C++ method (`RET [CC] Cls::m(…)`). tree-sitter glues
            # a calling-convention macro into the node text ("STDMETHODCALLTYPE
            # CFoo::m") — take the last whitespace token before splitting.
            qtext = _extract_text(child, source).split()[-1]
            if "::" in qtext:
                cpp_class, _, name = qtext.rpartition("::")
                if name.startswith("~"):
                    is_dtor = True
                    name = name.lstrip("~")
                elif name == cpp_class.rpartition("::")[2]:
                    is_ctor = True
                if name.startswith("operator"):
                    return None  # operators stay unextracted (no C ABI shape)
            else:
                name = qtext
        elif (
            child.type == "ERROR"
            and name in CALLING_CONVENTIONS
            and len(child.children) == 1
            and child.children[0].type == "identifier"
        ):
            # tree-sitter's recovery from an unknown `WINAPI` lands either way round:
            # `LPVOID WINAPI ReAlloc (...)` puts WINAPI in an ERROR before the declarator,
            # but `LPVOID WINAPI Alloc (...)` makes WINAPI the declarator's name and puts
            # `Alloc` in the ERROR after it -- comctl32's Alloc and Free were planned as two
            # functions named `WINAPI`. The convention is never the name; the identifier is.
            calling_convention = name
            name = _extract_text(child.children[0], source)
        elif child.type == "destructor_name":
            is_dtor = True
            name = _extract_text(child, source).lstrip("~")
        elif child.type == "operator_name":
            return None  # operators stay unextracted (no C ABI shape)
        elif child.type == "parameter_list":
            for param_node in child.children:
                if param_node.type in (
                    "parameter_declaration",
                    "optional_parameter_declaration",
                ):
                    param_text = _extract_text(param_node, source)
                    if param_node.type == "optional_parameter_declaration":
                        param_text = param_text.split("=", 1)[0].strip()
                    if cpp_class is not None and param_text == "THIS":
                        # Legacy DECLARE_INTERFACE methods sometimes retain the
                        # C spelling in an out-of-line C++ definition. In C++
                        # this macro expands to void; the receiver is implicit.
                        continue
                    if param_text in ("void", "VOID"):
                        # C's EMPTY parameter list. `f(void)` declares zero parameters, but
                        # tree-sitter still reports one parameter_declaration, and recording
                        # it gave the function arity 1. The call binder then saw
                        # `SelectExtOnRename()` as arity 0 against "available arities: [1]"
                        # and refused to bind — which stopped the whole 4,869-line CDefView
                        # TU, on a function whose C says plainly that it takes nothing.
                        # Only the bare spelling counts: `void *p` keeps its declarator and
                        # is a real parameter, so matching the exact text is what separates
                        # them. The uppercase `VOID` was already special-cased on the other
                        # two parameter paths in this file and simply never on this one.
                        continue
                    spaced_array = re.fullmatch(
                        r"(?P<type>.+?)\s+(?P<name>[A-Za-z_]\w*)\s*\[[^]]*\]",
                        param_text,
                    )
                    if spaced_array:
                        # A space between the identifier and brackets makes
                        # rsplit() see "[]" as the name. Function-parameter
                        # arrays still decay to pointers regardless of that
                        # harmless C/C++ spelling difference.
                        params.append(
                            (
                                f"{spaced_array.group('type').strip()} *",
                                spaced_array.group("name"),
                            )
                        )
                        continue
                    # Crude but effective: split into type and name
                    parts = param_text.rsplit(None, 1)
                    if len(parts) == 2:
                        ptype, pname = parts
                        array = re.fullmatch(r"([A-Za-z_]\w*)\s*\[[^]]*\]", pname)
                        if array:
                            # C/C++ adjusts function-parameter arrays to
                            # pointers; retain that ABI fact, not declarator
                            # punctuation that is invalid in Rust.
                            ptype += " *"
                            pname = array.group(1)
                        # Remove pointer markers from name
                        while pname.startswith("*"):
                            ptype += " *"
                            pname = pname[1:]
                        params.append((ptype.strip(), pname.strip()))
                    elif parts and "*" in parts[0]:
                        adjacent = re.match(r"(?P<type>.+\*)(?P<name>[A-Za-z_]\w*)$", parts[0])
                        if adjacent:
                            params.append((adjacent.group("type").strip(), adjacent.group("name")))
                        else:
                            params.append((parts[0], ""))
                    elif parts and parts[0] != "VOID":
                        params.append((parts[0], ""))

    if name is None:
        return None

    if cpp_class is None:
        # In-class definition (field_identifier/identifier/destructor_name
        # declarator with no `Cls::` prefix)? Ascend to the enclosing
        # class/struct specifier for the owning class name.
        anc = node.parent
        while anc is not None:
            if anc.type in ("class_specifier", "struct_specifier"):
                for c in anc.children:
                    if c.type == "type_identifier":
                        cpp_class = _extract_text(c, source)
                        break
                break
            # Macro-heavy ATL classes — especially a header holding several such
            # classes — can make tree-sitter wrap the whole region in one flat
            # ERROR node, dropping the class_specifier structure.  The
            # `class NAME : ... {` headers are still tokenized inside it, so
            # recover the owning class by brace-balanced containment: attribute
            # this definition to the class whose body actually encloses it.  A
            # free helper past a class's closing brace is therefore never
            # mis-attributed — the concern that previously limited this recovery
            # to the `struct` shape.
            if anc.type == "ERROR":
                owner = _error_node_class_owner(anc, node, source)
                if owner is not None:
                    cpp_class = owner
                    break
            anc = anc.parent
        # A destructor_name declarator already stripped its `~` — the name now
        # equals the class name, but it is NOT the constructor (CMenuSite FP).
        if cpp_class and name == cpp_class and not is_dtor:
            is_ctor = True

    if return_type == "" and cpp_class is not None and not (is_ctor or is_dtor):
        # tree-sitter-cpp can absorb `RET WINAPI` into the qualified
        # declarator node, leaving no top-level return-type child. Recover the
        # prefix from the intact source signature; the qualified owner/name
        # makes this shape unambiguous.
        body_node = next(
            (child for child in node.children if child.type == "compound_statement"), None
        )
        body_start = (
            body_node.start_byte if body_node is not None else source.find(b"{", node.start_byte)
        )
        if body_start >= node.start_byte:
            signature = source[node.start_byte : body_start].decode("utf-8", errors="replace")
            qualified = rf"{re.escape(cpp_class)}::{re.escape(name)}\s*\("
            prefix = re.match(rf"\s*(?P<prefix>.*?)\b{qualified}", signature, flags=re.S)
            if prefix is None or not prefix.group("prefix").strip():
                line_start_byte = source.rfind(b"\n", 0, node.start_byte) + 1
                leading = source[line_start_byte : node.start_byte].decode(
                    "utf-8", errors="replace"
                )
                prefix = re.match(r"\s*(?P<prefix>.*?)\s*$", leading, flags=re.S)
            if prefix is not None:
                tokens = [
                    token
                    for token in prefix.group("prefix").split()
                    if token not in CALLING_CONVENTIONS and token != "static"
                ]
                return_type = " ".join(tokens)

    # Count SEH blocks in body
    body_text = _extract_text(node, source)
    if calling_convention is None:
        # C++ recovery can put NTAPI inside the declarator's ERROR node instead
        # of the top-level children. Read only the prefix before the function
        # name; callback conventions inside parameters are unrelated.
        signature_name = re.search(rf"\b{re.escape(name)}\s*\(", body_text)
        if signature_name is not None:
            conventions = [
                word
                for word in re.findall(r"\b\w+\b", body_text[: signature_name.start()])
                if word in CALLING_CONVENTIONS
            ]
            if conventions:
                calling_convention = conventions[-1]
    seh_count = body_text.count("_SEH2_TRY")

    return CFunction(
        name=name,
        return_type=return_type,
        calling_convention=calling_convention,
        params=params,
        is_static=is_static,
        code_seg=code_seg,
        seh_count=seh_count,
        line_start=node.start_point[0] + 1,
        line_end=node.end_point[0] + 1,
        body_text=body_text,
        cpp_class=cpp_class,
        is_ctor=is_ctor,
        is_dtor=is_dtor,
    )


_SPLIT_INCLUDE_RE = re.compile(
    r'^[ \t]*#[ \t]*include[ \t]+"([^"]+\.split\d+\.c)"[ \t]*$',
    re.MULTILINE,
)


def _expand_split_includes(path: Path, text: str, _depth: int = 0) -> str:
    """Inline pretranslation `#include "X.splitN.c"` fragments.

    The giant-file pretranslation split carves trailing function groups
    into `X.splitN.c` siblings that the original `X.c` `#include`s back,
    so the C preprocessor reassembles ONE translation unit verbatim (no
    CMake/linkage change). rustconv parses C with tree-sitter, NOT cpp,
    so without this it would silently miss every split-off function (and
    their shared external-linkage TU globals would become two clashing
    Rust modules). Mirror cpp: textually inline the fragment in place.

    The include sits at the END of the parent, so inlining APPENDS after
    the parent's last line — every original line number is preserved and
    existing Pass-0/2/3 sidecars (keyed by parent line ranges) stay
    valid. Iterative/recursive: `X.c` may include `X.split2.c` and
    `X.split3.c`; a fragment may include the next.
    """
    if _depth > 16:
        return text

    def _sub(m: "re.Match") -> str:
        frag = path.parent / m.group(1)
        if not frag.exists():
            return m.group(0)  # not a local split fragment — leave it
        frag_text = frag.read_text(errors="replace")
        return _expand_split_includes(frag, frag_text, _depth + 1)

    return _SPLIT_INCLUDE_RE.sub(_sub, text)


# win32k debug switches HARD-defined to 0 in win32ss/win32kp.h (never
# build-overridden — the build has -DDBG=1 but these finer feature
# switches stay 0). rustconv parses with tree-sitter not cpp, so without
# this it translates these build-excluded blocks and emits references to
# fields/macros that legitimately don't exist (e.g. BASEOBJECT.slhLog,
# .apvBackTrace, DBG_DUMP_EVENT_LIST). Conservative: only these proven-0
# symbols, only the simple `#if SYM ... #endif` shape — bail to verbatim
# on ANY #else/#elif/nested #if (see feedback_preprocessor_elif_chain).
_KNOWN_OFF_IFDEF_SYMS = (
    "DBG_ENABLE_EVENT_LOGGING",
    "DBG_ENABLE_GDIOBJ_BACKTRACES",
)
_IF_LINE = re.compile(r"^[ \t]*#[ \t]*(if|ifdef|ifndef|elif|else|endif)\b")


def _strip_known_off_ifdefs(text: str) -> str:
    lines = text.split("\n")
    syms = "|".join(re.escape(s) for s in _KNOWN_OFF_IFDEF_SYMS)
    open_re = re.compile(rf"^[ \t]*#[ \t]*if[ \t]+(?:!\s*)?(?:{syms})\b")
    out, i, n = [], 0, len(lines)
    while i < n:
        if open_re.match(lines[i]):
            # Scan to the matching #endif at the same nesting depth.
            depth, j, simple = 1, i + 1, True
            while j < n and depth > 0:
                m = _IF_LINE.match(lines[j])
                if m:
                    kw = m.group(1)
                    if kw in ("if", "ifdef", "ifndef"):
                        depth += 1
                        simple = False  # nested conditional inside
                    elif kw == "endif":
                        depth -= 1
                    elif depth == 1 and kw in ("else", "elif"):
                        simple = False  # has an alternative branch
                j += 1
            if depth == 0 and simple:
                i = j  # drop [i, j): the whole #if…#endif block
                continue
            # Not a simple known-0 block → leave verbatim.
        out.append(lines[i])
        i += 1
    return "\n".join(out)


# Feature macros that are NEVER defined in the build — placeholders for
# not-yet-implemented paths (concfg/font.c's FONT_CACHE_PRESENT). A
# `#ifndef FONT_CACHE_PRESENT` block sits INSIDE a function body; tree-sitter-c
# cannot parse a `#if/#endif` inside a compound statement and recovers
# catastrophically (the enclosing function_definition swallows every following
# function — font.c found 2 of 14). Resolving them (drop the directive lines,
# keep the live branch) lets tree-sitter parse the body.
_KNOWN_UNDEFINED_SYMS = ("FONT_CACHE_PRESENT",)
# Symbols the build defines to 1 (`-DDBG=1` for the checked/debug kernel). A bare
# `#if DBG` body sits INSIDE function bodies (the FAT/CDFS drivers wrap their SEH
# frames in `#if DBG`); tree-sitter-c recovers catastrophically from a `#if/#endif`
# inside a compound statement (swallows every following function — fsctrl.c lost 50
# of 76, FatMountVolume among them). Resolving `#if DBG` → keep the live branch,
# drop the directive lines, lets the SEH parse with balanced braces.
_KNOWN_DEFINED_TRUE_SYMS = ("DBG",)
_PP_IF = re.compile(r"^[ \t]*#[ \t]*(if|ifdef|ifndef)\b[ \t]*(.*)$")
# Bounded on purpose: an unanchored `[A-Za-z_]\w*` also matches the `x600` inside the
# hex literal `0x600`, so `#if WINVER >= 0x600` had its literal rewritten to `00`.
_PP_IDENT = re.compile(r"\b[A-Za-z_]\w*")

# `target_compile_definitions(msvcrt_static PRIVATE _MSVCR_VER=0)` and
# `add_definitions(-D_MSVCR_VER=0)`. Only object-like NAME[=VALUE] forms.
_CMAKE_TGT_DEFS = re.compile(
    r"target_compile_definitions\s*\(([^)]*)\)|add_definitions\s*\(([^)]*)\)", re.S
)
_CMAKE_DEF_ITEM = re.compile(
    r"(?:^|\s)-?D?([A-Za-z_]\w*)(?:=(-?(?:0[xX][0-9a-fA-F]+|\d+)))?(?=\s|$)"
)
_CMAKE_SCOPE_WORDS = frozenset({"PRIVATE", "PUBLIC", "INTERFACE", "BEFORE"})


def cmake_target_defines(source: Path) -> dict[str, str]:
    """Integer object-like macros the module's own ``CMakeLists.txt`` puts on the compile line.

    This is what makes ``#if _MSVCR_VER >= 80`` decidable. msvcrt builds every target with
    ``_MSVCR_VER=0``, so those 284 conditionals across the module are not "conditional" at
    all: the ``>=80/100/110/120/140`` arms are DEAD (translating them pays for code the C
    never compiles and emits `#[no_mangle]` symbols the DLL does not have) and the ``==0``
    arms are LIVE (dropping them silently deletes behaviour — the model turned one into
    `#[cfg(0)]`, which neither compiles nor keeps the code).

    Only integer values are taken, because only those can settle an ``#if``. A macro two
    targets in the same file define DIFFERENTLY is dropped rather than picked between —
    the same disagreement rule the typedef authority uses, and for the same reason: we are
    not the preprocessor and must not pretend to resolve what the build genuinely splits.
    """
    cmake = source.parent / "CMakeLists.txt"
    try:
        text = cmake.read_text(errors="replace")
    except OSError:
        return {}
    text = re.sub(r"#[^\n]*", "", text)  # CMake comments
    seen: dict[str, set[str]] = {}
    for match in _CMAKE_TGT_DEFS.finditer(text):
        args = match.group(1) or match.group(2) or ""
        for item in _CMAKE_DEF_ITEM.finditer(args):
            name, value = item.group(1), item.group(2)
            if name in _CMAKE_SCOPE_WORDS:
                continue
            if value is None:
                # A bare `-DFOO` is 1 to the preprocessor, but a bare word in
                # target_compile_definitions' arg list is just as likely the TARGET name.
                # Only `NAME=VALUE` is unambiguous, so that is all we take.
                continue
            seen.setdefault(name, set()).add(value)
    return {name: next(iter(vals)) for name, vals in seen.items() if len(vals) == 1}


def build_int_defines(source: Path) -> dict[str, str]:
    """`cmake_target_defines` plus the Windows version macros the module is compiled at.

    The version macros come from version_macros (sdkddkver.h + the module's override), so
    `#if (NTDDI_VERSION >= NTDDI_VISTA)` is as decidable as `#if _MSVCR_VER >= 80`: the
    dead arm is dropped before the model ever sees it. A module's own `-D` wins, then the
    version table fills the names it does not set.
    """
    defines = cmake_target_defines(source)
    repo = repo_root_of(source)
    if repo is not None:
        for name, value in version_defines(source.parent, repo).items():
            defines.setdefault(name, hex(value))
    return defines


_TU_DEFINE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)(?![\w(])[ \t]*(.*?)[ \t]*$")
_TU_OFF_SWITCH = re.compile(r"^[ \t]*//[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\b", re.M)
_DEFINED_TERM = re.compile(r"defined\s*\(\s*([A-Za-z_]\w*)\s*\)|defined\s+([A-Za-z_]\w*)")


def tu_switches(path: Path, raw: str) -> tuple[dict[str, str], frozenset[str]]:
    """Feature switches the TU ITSELF settles: (defined -> value, deliberately off).

    cmd.exe's parser.c/for.c wrap statements in `#ifdef MSCMD_*` arms that split braces
    (`if (..) {` opens in one arm and closes in another). With both arms kept, tree-sitter
    cannot find the function, and the planner drops it. The switches are decidable from the
    file: `#define MSCMD_X` at file scope (outside any conditional) is ON; a switch is OFF
    only with positive evidence -- the TU carries a commented-out `// #define NAME` toggle,
    and NAME is defined by no active line in the TU and mentioned by no header in its
    directory nor its CMakeLists.txt. Anything else stays undecided and keeps both arms,
    exactly as before.
    """
    defined: dict[str, str] = {}
    depth = 0
    for line in raw.splitlines():
        m = re.match(r"^[ \t]*#[ \t]*(if|ifdef|ifndef|endif)\b", line)
        if m:
            depth += -1 if m.group(1) == "endif" else 1
            continue
        if depth == 0:
            d = _TU_DEFINE.match(line)
            if d and not d.group(2).endswith("\\"):  # a continued body is a macro, not a switch
                val = re.sub(r"/\*.*?\*/", "", d.group(2)).split("//")[0].strip()
                defined[d.group(1)] = val or "1"
    off = set(_TU_OFF_SWITCH.findall(raw)) - defined.keys()
    if off:
        others = []
        for h in (*sorted(path.parent.glob("*.h")), path.parent / "CMakeLists.txt"):
            try:
                others.append(h.read_text(errors="replace"))
            except OSError:
                pass
        for name in sorted(off):
            active = re.compile(rf"^[ \t]*#[ \t]*define[ \t]+{re.escape(name)}\b", re.M)
            mention = re.compile(rf"(?:(?<!\w)|(?<=-D)){re.escape(name)}\b")  # and `-DNAME`
            if active.search(raw) or any(mention.search(t) for t in others):
                off.discard(name)
    return defined, frozenset(off | (_header_off_switches(path, raw) - defined.keys()))


def _local_header_closure(path: Path, raw: str) -> list[Path]:
    """The module-directory headers a TU `#include "..."`s, transitively."""
    seen: list[Path] = []
    todo = [raw]
    while todo:
        for inc in re.findall(r'^[ \t]*#[ \t]*include[ \t]+"([^"]+)"', todo.pop(), re.M):
            h = (path.parent / inc).resolve()
            if h.parent == path.parent.resolve() and h.is_file() and h not in seen:
                seen.append(h)
                todo.append(h.read_text(errors="replace"))
    return seen


def _header_off_switches(path: Path, raw: str) -> frozenset[str]:
    """Switches a module's own configuration HEADER turns off (cmd.exe's config.h).

    `//#define FEATURE_UNIX_FILENAME_COMPLETION` in config.h is as much a decision as the
    same toggle in the TU, but `tu_switches` only read the TU, so filecomp.c's dead UNIX
    arm was translated alongside the live 4NT one: two `CompleteFilename`s, the dead one
    exported with the wrong signature. Positive evidence only, as for a TU toggle: the
    header carries the commented-out toggle, and no `.c`/`.h` of the module defines the
    name on an active line and its CMakeLists.txt never mentions it (`-DNAME`).

    ON switches from headers are deliberately not read: a kept `#ifdef ON` arm translates
    correctly, and deciding them would re-key every banked function under one.
    """
    toggles: set[str] = set()
    for h in _local_header_closure(path, raw):
        toggles |= set(_TU_OFF_SWITCH.findall(h.read_text(errors="replace")))
    if not toggles:
        return frozenset()
    sources = [raw]
    for f in (*sorted(path.parent.glob("*.[ch]")), path.parent / "CMakeLists.txt"):
        try:
            sources.append(f.read_text(errors="replace"))
        except OSError:
            pass
    cmake = sources[-1] if (path.parent / "CMakeLists.txt").is_file() else ""
    off = set()
    for name in toggles:
        active = re.compile(rf"^[ \t]*#[ \t]*(define|undef)[ \t]+{re.escape(name)}\b", re.M)
        # `\b` alone misses `-DNAME`: D and N are both word characters.
        if not any(active.search(t) for t in sources) and not re.search(
            rf"(?:(?<!\w)|(?<=-D)){re.escape(name)}\b", cmake
        ):
            off.add(name)
    return frozenset(off)


def _eval_pp_cond(
    kw: str,
    expr: str,
    defines: dict[str, str] | None = None,
    off: frozenset[str] = frozenset(),
) -> bool | None:
    """Resolve a preprocessor condition to True/False, or None if unknown."""
    expr = expr.split("//")[0].split("/*")[0].strip()
    defines = defines or {}
    if kw != "if":
        sym0 = expr.split()[0] if expr.split() else ""
        if sym0 in off:
            return kw == "ifndef"
    if kw == "if":
        if off:
            expr = _DEFINED_TERM.sub(
                lambda m: "0" if (m.group(1) or m.group(2)) in off else m.group(0), expr
            )
        v = {"0": False, "1": True}.get(expr)
        if v is not None:
            return v
        if expr in _KNOWN_DEFINED_TRUE_SYMS:  # bare `#if DBG` → DBG=1 in the build
            return True
        # Build-fixed integer macros. EVERY identifier must be known -- INCLUDING the ones
        # inside `defined(...)`. An expression mentioning one macro we know and one we do
        # not is undecidable, and guessing the unknown one as 0 (what the .cpp-path
        # evaluator does) drops live code.
        #
        # mathf.c is the case that proves it: `#if defined(__i386__) || (_MSVCR_VER > 0 &&
        # _MSVCR_VER < 80)`. Exempting `defined()` from this check made `__i386__` fall to
        # 0, the condition false, and TEN live float functions (sinf/cosf/tanf/...) vanish
        # from the i386 build. Arch macros must stay verbatim regardless: one Rust file
        # serves all three arches, so an arch conditional is `#[cfg(target_arch)]`'s job,
        # never ours to fold away.
        idents = set(_PP_IDENT.findall(expr)) - {"defined"}
        if idents and idents <= defines.keys():
            return _eval_c_const_expr(expr, defines)
        return None
    sym = expr.split()[0] if expr.split() else ""
    if sym in _KNOWN_UNDEFINED_SYMS:
        return kw == "ifndef"  # #ifndef UNDEF → True, #ifdef UNDEF → False
    if sym in _KNOWN_DEFINED_TRUE_SYMS:
        return kw == "ifdef"  # #ifdef DBG → True, #ifndef DBG → False
    if sym in (defines or {}):
        return kw == "ifdef"  # the build puts -DSYM on the command line
    return None


def _resolve_known_preproc(
    text: str, defines: dict[str, str] | None = None, off: frozenset[str] = frozenset()
) -> str:
    """Resolve body-internal `#if 0`/`#if 1` and `#if(n)def` of known-undefined
    feature macros — keep the live branch, drop the dead branch + the directive
    lines — so tree-sitter parses function bodies that contain them. Leaves any
    condition it cannot resolve verbatim; bails on `#elif`/nesting (handles only
    a simple block with at most one `#else`).

    *defines* adds the module's build-fixed integer macros (see
    :func:`cmake_target_defines`), which is what settles msvcrt's `#if _MSVCR_VER`
    family. Dropping a dead arm here removes its functions from the scan entirely, so
    they are never extracted, never paid for, and never emitted."""
    lines = text.split("\n")
    out, i, n = [], 0, len(lines)
    while i < n:
        m = _PP_IF.match(lines[i])
        if not m:
            out.append(lines[i])
            i += 1
            continue
        cond = _eval_pp_cond(m.group(1), m.group(2), defines, off)
        if cond is None:
            out.append(lines[i])
            i += 1
            continue
        depth, j, else_idx, ok = 1, i + 1, None, True
        while j < n and depth > 0:
            if _PP_IF.match(lines[j]):
                depth += 1
            elif _IF_LINE.match(lines[j]) and re.match(r"^[ \t]*#[ \t]*endif\b", lines[j]):
                depth -= 1
            elif depth == 1 and re.match(r"^[ \t]*#[ \t]*elif\b", lines[j]):
                ok = False
            elif depth == 1 and re.match(r"^[ \t]*#[ \t]*else\b", lines[j]):
                if else_idx is None:
                    else_idx = j
                else:
                    ok = False
            j += 1
        if depth != 0 or not ok:
            out.append(lines[i])
            i += 1
            continue  # bail → verbatim
        endif_idx = j - 1
        then_body = lines[i + 1 : (else_idx if else_idx is not None else endif_idx)]
        else_body = lines[else_idx + 1 : endif_idx] if else_idx is not None else []
        out.extend(then_body if cond else else_body)
        i = j
    return "\n".join(out)


# `#define NAME(a, b) ...` — name, parameter list, first body line. Function-like only:
# the `(` must touch the name, which is what the C preprocessor itself requires.
FN_MACRO_RE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\(([^)]*)\)(.*)$", re.M)


def macro_body(src_text: str, start: int) -> str:
    """The full replacement list at *start*, following backslash line-continuations."""
    body: list[str] = []
    for line in src_text[start:].splitlines():
        if line.endswith("\\"):
            body.append(line[:-1])
            continue
        body.append(line)
        break
    return "\n".join(body)


def split_macro_args(src_text: str, open_paren_end: int) -> list[str] | None:
    """Split a macro invocation's arguments at top-level commas.

    *open_paren_end* is the index just past the ``(``. Returns None if the parentheses
    never balance, so a false ``NAME(`` match inside a string or comment drops out rather
    than being treated as an invocation.
    """
    depth = 1
    arg = ""
    args: list[str] = []
    for ch in src_text[open_paren_end:]:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                args.append(arg)
                return args
        if depth == 1 and ch == ",":
            args.append(arg)
            arg = ""
            continue
        arg += ch
    return None


def _expand_case_macros(text: str) -> str:
    """Expand TU-local function-like macros that GENERATE a ``case`` label.

    tree-sitter cannot parse ``ERR_CASE(ERROR_ACCESS_DENIED)`` sitting where a case label
    belongs — a bare ``IDENT(ARG)`` with no semicolon is not a statement — and the damage is
    not local: on msvcrt's errno.c it produced ONE ``ERROR`` node spanning lines 132-451,
    swallowing ``msvcrt_set_errno`` and ``_errno`` whole while the walk recovered the
    survivors. Nothing landed in ``unextracted``, so the catalog came back looking healthy:
    16 paid functions, the two most important ones missing, and ``ERR_CASE`` — a macro —
    promoted to a billable function.

    Expanding is also the better prompt. The model sees real ``case`` labels, which is what
    ``--seal-facts`` needs to protect the constants, and there is no Rust rendering of the
    C macro worth preserving anyway.

    Deliberately narrow: only macros defined in THIS text whose replacement list contains a
    ``case`` token. Runs after :func:`_resolve_known_preproc`, so macros in a dead ``#if``
    arm are already gone and are never expanded.
    """
    defs: list[tuple[int, int, str, list[str], str]] = []
    for m in FN_MACRO_RE.finditer(text):
        body = macro_body(text, m.start(3))
        if not re.search(r"\bcase\b", body):
            continue
        params = [p.strip() for p in m.group(2).split(",") if p.strip()]
        if not params:
            continue
        end = m.start(3) + len(body)
        # One logical line: a replacement list's backslash-continuations are formatting, and
        # collapsing them is what lets an expansion be substituted without moving any line.
        defs.append((m.start(), end, m.group(1), params, " ".join(body.split())))
    if not defs:
        return text  # byte-identical for every TU without one, which is most of them

    names = sorted({d[2] for d in defs}, key=len, reverse=True)
    call_re = re.compile(rf"\b(?:{'|'.join(re.escape(n) for n in names)})[ \t]*\(")
    def_at = {d[0]: d for d in defs}
    # Left to right, tracking which definition is in force. A name can be REDEFINED with a
    # different body — dinput's effect_linuxinput.c defines `FE(x)` once as `type = #x` and
    # again as `wave = #x` — so a name->body map would expand the first group with the
    # second body. Position is what disambiguates, so this walks rather than substitutes.
    live: dict[str, tuple[list[str], str]] = {}
    out: list[str] = []
    i = 0
    while i < len(text):
        if i in def_at:
            _, end, name, params, body = def_at[i]
            live[name] = (params, body)
            # Drop the directive (meaningless once expanded) but keep its NEWLINES. Every
            # normalizer in this pipeline preserves the line count, and preamble_audit
            # reports the LINE of an unparsed region — a pass that moves lines makes that
            # number point at innocent code.
            out.append("\n" * text.count("\n", i, end))
            i = end
            continue
        call = call_re.search(text, i)
        if call is None:
            out.append(text[i:])
            break
        nxt_def = min((s for s in def_at if s > i), default=len(text))
        if call.start() >= nxt_def:
            out.append(text[i:nxt_def])
            i = nxt_def
            continue
        name = text[call.start() : call.end() - 1].strip().rstrip("(").strip()
        args = split_macro_args(text, call.end())
        if args is None or name not in live:
            out.append(text[i : call.end()])
            i = call.end()
            continue
        params, body = live[name]
        rep = body
        for k, p in enumerate(params):
            rep = re.sub(rf"\b{re.escape(p)}\b", (args[k].strip() if k < len(args) else ""), rep)
        out.append(text[i : call.start()])
        out.append(rep)
        i = call.end() + sum(len(a) for a in args) + len(args)  # args + commas + `)`
    return "".join(out)


def extract_preamble(scan: "ScanResult") -> str:
    """File-scope C that `translate` (function-by-function) otherwise drops.

    cmd_translate only ever sends *functions* to the LLM, so every
    `#define`, global variable DEFINITION (`PENTRY gpentHmgr;`), static
    const table (`apfnCleanup[]`), and file-scope typedef is silently
    lost — the #1 root of "cannot find value" + extern-declared-but-
    undefined-global errors on any multi-function file. Carve raw_source
    minus the function line spans; drop #include lines (resolved
    separately) so the preamble translates as a clean declarations-only
    unit.
    """
    lines = scan.raw_source.splitlines()
    covered: set[int] = set()
    for f in scan.functions:
        for ln in range(f.line_start, f.line_end + 1):
            covered.add(ln)
    out: list[str] = []
    for idx, line in enumerate(lines, 1):
        if idx in covered:
            continue
        s = line.lstrip()
        if s.startswith("#include"):
            continue
        out.append(line)
    # Collapse the runs of blank lines the carve leaves behind.
    text = "\n".join(out)
    return re.sub(r"\n{3,}", "\n\n", text).strip()


# ---------------------------------------------------------------------------
# Partial preprocessor — #if/#ifdef conditional resolution (macros preserved).
#
# tree-sitter does NOT evaluate preprocessor conditionals, so #if/#endif blocks
# that span constructs (pervasive in #ifdef-heavy drivers like uniata: 190 such
# lines in id_ata.cpp vs 7 in atapi.c) wreck the parse — functions get swept
# into the file-scope preamble instead of extracted per-function. We resolve the
# conditionals ourselves against a symbol table (a ReactOS-i386 build -D default
# set + object-like #defines harvested from the target's local headers + in-file
# #defines tracked top-down), dropping dead branches and the directive lines but
# leaving macro INVOCATIONS unexpanded (faithful translation). Applied to C++
# (.cpp) only for now — the proven C path is unchanged. Unknown conditions keep
# their branch (never silently drop live code).
# ---------------------------------------------------------------------------

# Stable ReactOS i386 driver-build symbols (from the cmake compile command's -D
# flags). Per-target -D overrides could be threaded later; these cover the gate
# symbols that decide live branches (__REACTOS__ defined; feature flags off).
_DEFAULT_PP_DEFINES: dict[str, str] = {
    "__REACTOS__": "1",
    "_M_IX86": "1",
    "_X86_": "1",
    "__i386__": "1",
    "i386": "1",
    "_USE_DUMMY_PSEH": "1",
    "DBG": "1",
    "WINVER": "0x502",
    "_WIN32_WINNT": "0x502",
    "_WIN32_WINDOWS": "0x502",
    "_WIN32_IE": "0x603",
    "_SETUPAPI_VER": "0x502",
    "USE_COMPILER_EXCEPTIONS": "1",
    "_NEW_DELETE_OPERATORS_": "1",
    "DLL_EXPORT_VERSION": "0x502",
    "__USE_CRTIMP": "1",
    "_DLL": "1",
    "MINGW_HAS_SECURE_API": "1",
}

_PP_SAFE_EXPR = re.compile(
    r"^[0-9xXa-fA-F\s()+\-*/%<>=!&|~^]*"
    r"(?:\b(?:and|or|not)\b[0-9xXa-fA-F\s()+\-*/%<>=!&|~^]*)*$"
)
_DEFINE_RE = re.compile(r"^\s*#\s*define\s+([A-Za-z_]\w*)(?!\()(?:\s+(.*))?$")
_UNDEF_RE = re.compile(r"^\s*#\s*undef\s+([A-Za-z_]\w*)")
_COND_RE = re.compile(r"^\s*#\s*(ifdef|ifndef|if|elif|else|endif)\b(.*)$")


def _harvest_local_defines(path: Path) -> dict[str, str]:
    """Object-like #defines from the target's own directory + inc/ headers."""
    defs: dict[str, str] = {}
    for d in (path.parent, path.parent / "inc"):
        if not d.is_dir():
            continue
        for h in sorted(d.glob("*.h")):
            try:
                ht = h.read_text(errors="replace")
            except Exception:
                continue
            for m in _DEFINE_RE.finditer(ht):
                # strip trailing line-comments from the value
                val = m.group(2) or "1"
                val = re.sub(r"/\*.*?\*/", "", val)
                val = val.split("//")[0].strip() or "1"
                defs.setdefault(m.group(1), val)
    return defs


def _eval_c_const_expr(expr: str, symbols: dict[str, str]):
    """Evaluate a #if/#elif integer-constant expression. Returns bool or None."""
    expr = re.sub(r"/\*.*?\*/", "", expr).split("//")[0]
    # defined(X) / defined X
    expr = re.sub(
        r"defined\s*\(\s*([A-Za-z_]\w*)\s*\)", lambda m: "1" if m.group(1) in symbols else "0", expr
    )
    expr = re.sub(
        r"defined\s+([A-Za-z_]\w*)", lambda m: "1" if m.group(1) in symbols else "0", expr
    )
    # substitute object-like macro identifiers with their values (bounded passes)
    for _ in range(8):

        def _sub(m):
            name = m.group(0)
            v = symbols.get(name)
            return f"({v})" if (v not in (None, "")) else "0"

        new = _PP_IDENT.sub(_sub, expr)
        if new == expr:
            break
        expr = new
    # any leftover bare identifiers → undefined → 0
    expr = _PP_IDENT.sub("0", expr)
    # strip integer suffixes (0x1FUL, 5L, ...)
    expr = re.sub(r"(\b0[xX][0-9a-fA-F]+|\b\d+)[uUlL]+", r"\1", expr)
    # C → Python operators
    expr = expr.replace("&&", " and ").replace("||", " or ")
    expr = re.sub(r"!(?!=)", " not ", expr)
    expr = re.sub(r"(?<![/])/(?![/])", "//", expr)  # int division
    if not _PP_SAFE_EXPR.match(expr):
        return None
    try:
        return bool(eval(expr, {"__builtins__": {}}, {}))  # sanitized: digits/ops only
    except Exception:
        return None


def _resolve_pp_conditionals(path: Path, text: str) -> str:
    """Drop dead #if/#ifdef branches; keep live ones + all macro invocations."""
    symbols = dict(_DEFAULT_PP_DEFINES)
    symbols.update(_harvest_local_defines(path))
    out: list[str] = []
    stack: list[dict] = []  # each: {active, taken, parent, unknown}

    def emitting() -> bool:
        return all(f["active"] for f in stack)

    for line in text.splitlines():
        cm = _COND_RE.match(line)
        if not cm:
            if emitting():
                dm = _DEFINE_RE.match(line)
                if dm:
                    val = dm.group(2) or "1"
                    val = re.sub(r"/\*.*?\*/", "", val).split("//")[0].strip() or "1"
                    symbols[dm.group(1)] = val
                um = _UNDEF_RE.match(line)
                if um:
                    symbols.pop(um.group(1), None)
                out.append(line)
            continue
        kw, rest = cm.group(1), cm.group(2).strip()
        if kw in ("ifdef", "ifndef", "if"):
            parent = emitting()
            if not parent:
                stack.append({"active": False, "taken": True, "parent": False})
            elif kw == "ifdef":
                v = re.match(r"([A-Za-z_]\w*)", rest)
                val = bool(v and v.group(1) in symbols)
                stack.append({"active": val, "taken": val, "parent": True})
            elif kw == "ifndef":
                v = re.match(r"([A-Za-z_]\w*)", rest)
                val = not (v and v.group(1) in symbols)
                stack.append({"active": val, "taken": val, "parent": True})
            else:
                val = _eval_c_const_expr(rest, symbols)
                if val is None:  # unknown → keep branch (never drop live code)
                    stack.append({"active": True, "taken": True, "parent": True, "unknown": True})
                else:
                    stack.append({"active": val, "taken": val, "parent": True})
        elif kw == "elif":
            if not stack:
                continue
            top = stack[-1]
            if top.get("unknown") or not top["parent"]:
                top["active"] = top.get("unknown", False)
            elif top["taken"]:
                top["active"] = False
            else:
                val = _eval_c_const_expr(rest, symbols)
                if val is None:
                    top["active"] = True
                    top["unknown"] = True
                else:
                    top["active"] = val
                top["taken"] = top["taken"] or top["active"]
        elif kw == "else":
            if not stack:
                continue
            top = stack[-1]
            if top.get("unknown"):
                top["active"] = True
            else:
                top["active"] = top["parent"] and not top["taken"]
            top["taken"] = True
        elif kw == "endif":
            if stack:
                stack.pop()
    return "\n".join(out)


_SEH_BLANK = re.compile(rb"\b(?:_SEH2_END|_SEH2_LEAVE|_SEH2_VOLATILE|_SEH2_Volatile)\b")
_SEH_YIELD = re.compile(rb"\b_SEH2_YIELD\s*\(")


def mask_seh_for_parse(source: bytes) -> bytes:
    """PSEH macros rewritten to SAME-LENGTH valid C, for tree-sitter only.

    `_SEH2_TRY { A } _SEH2_FINALLY { B } _SEH2_END;` is not C until the preprocessor runs,
    and tree-sitter's error recovery decides by LAYOUT alone where the function ends: the
    same function parses whole on one indentation and is cut at the TRY block's `}` on
    another. 174 of the tree's 827 PSEH functions were cut that way (2026-10-05), and the
    tail -- the FINALLY cleanup and the final `return` -- was sent to the model as stray
    file-scope "preamble" (setuplib SetupMoveFile lost its `NtClose` and `return Status`).
    Masked: TRY -> `if(1)`, FINALLY -> `else`, EXCEPT(f) -> `else if(f)`, YIELD(s) ->
    `s` (`#define _SEH2_YIELD(STMT_) STMT_`; the `;` follows the `)`), END/LEAVE -> blanks.
    Same length, so every byte and line offset is unchanged.
    """

    def pad(word: bytes, n: int) -> bytes:
        return word + b" " * (n - len(word))

    out = re.sub(rb"\b_SEH2_TRY\b", lambda m: pad(b"if(1)", len(m.group())), source)
    out = re.sub(rb"\b_SEH2_FINALLY\b", lambda m: pad(b"else", len(m.group())), out)
    out = re.sub(rb"\b_SEH2_EXCEPT\b", lambda m: pad(b"else if", len(m.group())), out)
    out = _SEH_BLANK.sub(lambda m: b" " * len(m.group()), out)
    buf = bytearray(out)
    for m in _SEH_YIELD.finditer(out):
        depth, i = 0, m.end() - 1
        while i < len(buf):
            c = buf[i]
            if c == ord("("):
                depth += 1
            elif c == ord(")"):
                depth -= 1
                if depth == 0:
                    break
            i += 1
        if i >= len(buf):
            continue
        buf[m.start() : m.end()] = b" " * (m.end() - m.start())
        buf[i] = ord(" ")
    assert len(buf) == len(source)
    return bytes(buf)


class _Root:
    """A translation unit's top-level nodes, possibly drawn from two parses of one text."""

    type = "translation_unit"

    def __init__(self, children: list):
        self.children = children


def _count_function_definitions(node) -> int:
    """function_definition nodes at or below *node*."""
    count, stack = 0, [node]
    while stack:
        cur = stack.pop()
        if cur.type == "function_definition":
            count += 1
        stack.extend(cur.children)
    return count


# `#define A B` where both are identifiers, scoped by a later `#undef A` (or the end of
# the file). imagelist.c is the case: ReactOS's "big bad image list hack" renames 19
# ImageList_* to Internal_*, defines the internal bodies under the PUBLIC spelling, then
# `#undef`s and defines the real exported wrappers. The preprocessor sees Internal_Add and
# ImageList_Add; a raw-text scan saw ImageList_Add twice, so the Rust would have exported
# the unchecked internal body under the public name, and c2rust's anchors could not match.
_LOCAL_DEFINE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+([A-Za-z_]\w*)[ \t]*$")
_LOCAL_UNDEF = re.compile(r"^[ \t]*#[ \t]*undef[ \t]+([A-Za-z_]\w*)")
_STRING_OR_WORD = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|[A-Za-z_]\w*')


def _rename_windows(raw: str) -> list[tuple[str, str, int, int]]:
    """``(name, target, first_line, last_line)``, 1-based, for each in-file identifier rename."""
    lines = raw.split("\n")
    open_: dict[str, tuple[str, int]] = {}
    windows: list[tuple[str, str, int, int]] = []
    # Only a rename every build sees counts: one under `#ifdef USE_RTL_BITMAP64` (bitmap.c,
    # which bitmap64.c includes with it set) is not this file's spelling at all. So each
    # enclosing conditional must be one this tree always takes -- `#ifdef __REACTOS__`.
    stack: list[bool] = []
    for no, line in enumerate(lines, 1):
        if m := _PP_IF.match(line):
            sym = m.group(2).strip()
            stack.append(
                (m.group(1) == "ifdef" and sym == "__REACTOS__")
                or (m.group(1) == "if" and sym == "defined(__REACTOS__)")
            )
            continue
        if re.match(r"^[ \t]*#[ \t]*(else|elif)\b", line) and stack:
            stack[-1] = False
            continue
        if re.match(r"^[ \t]*#[ \t]*endif\b", line) and stack:
            stack.pop()
            continue
        if not all(stack):
            continue
        if m := _LOCAL_DEFINE.match(line):
            if m.group(1) != m.group(2):
                open_[m.group(1)] = (m.group(2), no + 1)
        elif (m := _LOCAL_UNDEF.match(line)) and m.group(1) in open_:
            target, first = open_.pop(m.group(1))
            windows.append((m.group(1), target, first, no - 1))
    windows.extend((name, t, first, len(lines)) for name, (t, first) in open_.items())
    return windows


def _defines_inside_window(raw: str, result: "ScanResult") -> bool:
    """Whether some scanned definition is spelled with a name renamed where it stands."""
    return any(
        f.name == name and first <= f.line_start <= last
        for name, _t, first, last in _rename_windows(raw)
        for f in result.functions
    )


def _apply_local_renames(raw: str) -> str:
    """Respell each renamed identifier inside its window, as the preprocessor would.

    String and character literals are left alone (a TRACE format naming the function is
    text, not a token), and so are directive lines. Line count is preserved.
    """
    windows = _rename_windows(raw)
    lines = raw.split("\n")
    for no in range(1, len(lines) + 1):
        line = lines[no - 1]
        if line.lstrip().startswith("#"):
            continue
        active = {n: t for n, t, first, last in windows if first <= no <= last}
        if not active:
            continue
        lines[no - 1] = _STRING_OR_WORD.sub(
            lambda m, active=active: active.get(m.group(0), m.group(0)), line
        )
    return "\n".join(lines)


def _merged_root(parser, source: bytes) -> _Root:
    """Top-level nodes of the SEH-masked parse, falling back to the raw parse where it fuses.

    Masking fixes the PSEH class outright (see :func:`mask_seh_for_parse`). But a file that is
    unparseable for ANOTHER reason -- virtual.c's `MiDeletePte` opens a `{` inside `#if
    (_MI_PAGING_LEVELS == 2)` and closes it after the `#endif` -- recovers wherever
    tree-sitter's error recovery happens to land, and the mask can move that landing: the
    masked parse of virtual.c is ONE 5,400-line "function" where the raw parse finds 39.
    So wherever one masked function spans the starts of two or more raw functions, the raw
    nodes for that span are used instead. Both parses are of byte-identical-length text, so
    every node slices `source` the same way.
    """
    # CODE_SEG("PAGE") is blanked for the PARSE only (same length, so every slice still
    # reads `source`, and _parse_function still finds it in the text). Spelled before the
    # return type it parses by luck; spelled between the return type and the name, as
    # pciidex's amd.c and atapi's pdo.c do, the function vanished from the plan.
    parse_src = RE_CODE_SEG_BYTES.sub(lambda m: b" " * len(m.group()), source)
    masked = parser.parse(mask_seh_for_parse(parse_src)).root_node.children
    if b"_SEH2_" not in source:
        return _Root(list(masked))
    raw = parser.parse(parse_src).root_node.children
    # Only column-0 starts count: raw recovery also mints phantom "functions" from inside a
    # PSEH body (`_SEH2_TRY`, `if`), always indented, which would fool this test.
    raw_fn_starts = [
        c.start_byte for c in raw if c.type == "function_definition" and c.start_point[1] == 0
    ]
    out: list = []
    for node in masked:
        if node.type == "function_definition":
            inside = [b for b in raw_fn_starts if node.start_byte <= b < node.end_byte]
            if len(inside) >= 2:
                out.extend(
                    r
                    for r in raw
                    if node.start_byte <= r.start_byte and r.end_byte <= node.end_byte
                )
                continue
        out.append(node)
    return _Root(out)


def scan_file(path: Path, *, force_cpp: bool = False, _resolve_reactos: bool = False) -> ScanResult:
    """Parse a C/C++ source file and extract structural information."""
    result = _scan_file(path, force_cpp=force_cpp, _resolve_reactos=_resolve_reactos)
    if _resolve_reactos or force_cpp or path.suffix.lower() != ".c":
        return result
    raw = path.read_text(errors="replace")
    # A definition spelled with a name the file itself renames where it stands (see
    # _rename_windows) is scanned again with the renames applied. Gated like the fallback
    # below: a file with no such definition keeps its exact text.
    renames = _defines_inside_window(raw, result)
    if renames:
        result = _scan_file(path, force_cpp=False, _resolve_reactos=False, _local_renames=True)
    # Wine code splits a body's braces across `#ifdef __REACTOS__` arms (combo.c's
    # WM_SYSKEYDOWN case opens a `{` in one arm and closes it in the other), which
    # tree-sitter cannot parse: COMBO_WindowProc, TOOLBAR_DrawButton and six more comctl32
    # functions vanished from the plan, and translate refused the TU. Every compile line in
    # this tree carries -D__REACTOS__, so the arm is decided. It is resolved ONLY when the
    # plain scan has lost a live definition -- the case translate already refuses -- so a
    # file that scans completely, and every body banked from it, keeps its exact text.
    # (Resolving it everywhere changed bodies in ~130 files, fastfat and user32 among them.)
    if "__REACTOS__" not in raw:
        # No arm to resolve, but a definition can still be lost (calc's theme.c: bare
        # `GET_CB(x)` statement macros with no `;` swallow the next function).
        from .c_defs import c_functions as _cf
        from .dead_regions import never_compiled_lines as _ncl

        text0 = _apply_local_renames(raw) if renames else raw
        dead0 = _ncl(text0, *_scan_switches(path, raw))
        live0 = "\n".join("" if n in dead0 else ln for n, ln in enumerate(text0.splitlines(), 1))
        lost0 = _cf(live0) - {f.name for f in result.functions}
        return _scan_live(path, live0, result, lost0)
    # The same split can leave every definition extracted and still be broken: listview.c
    # splits a SIGNATURE across the arms (LISTVIEW_NextIconPosTop), tree-sitter wraps the rest
    # of the file in one ERROR node, the walk still finds all 240 functions inside it -- and
    # the preamble, which keeps every top-level node that is not a function, carried 272K
    # characters of function bodies into every call's shared context. 160K-token prompts,
    # over Haiku 5.5's 100K tier on every call: listview alone cost more than the other 16
    # files of the run together. treeview.c's 108K preamble is the same shape.
    from .c_defs import c_functions
    from .dead_regions import never_compiled_lines

    live_raw = _apply_local_renames(raw) if renames else raw
    dead = never_compiled_lines(live_raw, *_scan_switches(path, raw))
    live = "\n".join("" if n in dead else ln for n, ln in enumerate(live_raw.splitlines(), 1))
    have = {f.name for f in result.functions}
    lost = c_functions(live) - have
    if result.preamble_swallowed_functions:
        retry = _scan_file(path, force_cpp=False, _resolve_reactos=True, _local_renames=renames)
        again = {f.name for f in retry.functions}
        # Better on EITHER count, losing nothing: calc's winmain.c splits `_tWinMain`'s
        # HEADER across `#if !defined(__REACTOS__)` arms, the resolved scan recovers the
        # definition, but only a smaller swallowed count was accepted here, so the plan
        # lacked _tWinMain and translate refused the TU.
        fewer = retry.preamble_swallowed_functions < result.preamble_swallowed_functions
        if (fewer or len(lost - again) < len(lost)) and have <= again:
            return retry
        return _scan_live(path, live, result, lost)
    if not lost:
        return result
    retry = _scan_file(path, force_cpp=False, _resolve_reactos=True, _local_renames=renames)
    again = {f.name for f in retry.functions}
    if len(lost - again) < len(lost) and have <= again:
        return retry
    return _scan_live(path, live, result, lost)


def _pinned_ints(path: Path) -> dict[str, int]:
    """The module's integer `-D`s (cmake_target_defines), so a dead-region decision here is
    the one the scan itself already makes: mbcs.c's `#if _MSVCR_VER>=80` bodies are dead
    at the pinned 0 and must not come back as "lost" definitions."""
    out: dict[str, int] = {}
    for k, v in build_int_defines(path).items():
        try:
            out[k] = int(v, 0)
        except (TypeError, ValueError):
            continue
    return out


def _scan_switches(path: Path, raw: str) -> tuple[dict[str, int], frozenset[str]]:
    """The dead-line decision the scan itself makes: the module's pinned `-D` ints plus the
    TU's own switches (tu_switches: file-scope `#define`s ON, `//#define` toggles OFF,
    cmd.exe's config.h included). A "lost" definition is only lost if it is live by THESE --
    filecomp.c's CompleteUnix sits under a config.h toggle that is off."""
    pinned = _pinned_ints(path)
    defined, off = tu_switches(path, raw)
    for k, v in defined.items():
        try:
            pinned.setdefault(k, int(str(v).rstrip("uUlL") or "1", 0))
        except ValueError:
            pinned.setdefault(k, 1)
    return pinned, off


def _scan_live(path: Path, live: str, result: ScanResult, lost: set[str]) -> ScanResult:
    """Last resort for a definition still lost: scan the LIVE text, every never-compiled line
    blank (line numbers kept). The `__REACTOS__` resolution only decides plain
    `#ifdef __REACTOS__` arms; calc's winmain.c splits `_tWinMain`'s header across
    `#if defined(__GNUC__) && !defined(__REACTOS__)`, which dead_regions evaluates and the
    resolver does not. Taken only when it recovers a lost definition and drops none."""
    if not lost:
        return result
    # With the dead arm blank, its `#if`/`#else`/`#endif` lines only split the surviving
    # header from its body for tree-sitter; blank them too (line numbers kept).
    live = re.sub(r"(?m)^[ \t]*#[ \t]*(?:if|ifdef|ifndef|elif|else|endif)\b.*$", "", live)
    # An indented statement macro written without its `;` (`GET_CB(OpenThemeData)` --
    # the macro brings its own) makes tree-sitter fold the next definition into an ERROR.
    live = re.sub(
        r"(?m)^([ \t]+(?!(?:if|while|for|switch|return|sizeof|else)\b)"
        r"[A-Za-z_]\w*\s*\([^;{}\n]*\))[ \t]*$",
        r"\1;",
        live,
    )
    retry = _scan_file(path, force_cpp=False, _resolve_reactos=False, _text=live)
    have = {f.name for f in result.functions}
    again = {f.name for f in retry.functions}
    return retry if len(lost - again) < len(lost) and have <= again else result


def _scan_file(
    path: Path,
    *,
    force_cpp: bool,
    _resolve_reactos: bool,
    _local_renames: bool = False,
    _text: str | None = None,
) -> ScanResult:
    raw = _expand_split_includes(path, path.read_text(errors="replace") if _text is None else _text)
    if _local_renames:
        raw = _apply_local_renames(raw)
    # portcls.h defines PORTCLASSAPI solely as DECLSPEC_IMPORT/DECLSPEC_EXPORT.
    # It is a linkage annotation, not the following function's return type.
    # Keep NTAPI and the real return type intact; preserve line coordinates.
    raw = re.sub(r"(?m)^[ \t]*PORTCLASSAPI[ \t]*$", lambda m: " " * len(m.group()), raw)
    # wine/asm.h: this emits an import-symbol assembler directive (or nothing),
    # never a C body. Without a semicolon tree-sitter absorbs the next function
    # as its declarator: mathf.c's powf was extracted under the name `float`.
    raw = re.sub(
        r"(?m)^[ \t]*__ASM_GLOBAL_IMPORT\s*\(\s*\w+\s*\)[ \t]*$",
        lambda m: " " * len(m.group()),
        raw,
    )
    is_cpp = force_cpp or path.suffix.lower() in _CPP_SUFFIXES
    is_header_c = not is_cpp and path.suffix.lower() == ".h"
    if is_cpp:
        # Resolve preprocessor conditionals so #ifdef-spanning constructs don't
        # break tree-sitter function extraction (uniata etc.). Macros preserved.
        raw = _resolve_pp_conditionals(path, raw)
        raw = _expand_inline_stdmethod(raw)
    # The module's own -D flags plus the version macros it is compiled at. Anything else
    # identifier-bearing in an `#if` stays undecidable and keeps both arms.
    build_defines = build_int_defines(path)
    tu_defined, tu_off = tu_switches(path, raw)
    for _k, _v in tu_defined.items():
        build_defines.setdefault(_k, _v)

    def _prepare(defines: dict[str, str]) -> str:
        return _normalize_callbacks(
            _blank_memmodel(
                _expand_extern_c(
                    _expand_composite_conventions(
                        _strip_sal(
                            _expand_csr_api(
                                _expand_case_macros(
                                    _resolve_known_preproc(
                                        _strip_known_off_ifdefs(raw), defines, tu_off
                                    )
                                )
                            )
                        )
                    ),
                    is_cpp,
                )
            )
        )

    parser = Parser(CPP_LANG if is_cpp else C_LANG)
    if _resolve_reactos:
        build_defines = {**build_defines, "__REACTOS__": "1"}
    source_text = _prepare(build_defines)
    source = source_text.encode("utf-8")

    # Parse a SEH-masked copy; every slice below still reads `source`, byte for byte.
    root = _merged_root(parser, source)

    result = ScanResult(path=path, raw_source=source_text)

    # Extract globals text (between GLOBALS and FUNCTIONS markers)
    lines = source_text.splitlines()
    globals_start = None
    functions_start = None
    for i, line in enumerate(lines):
        if RE_GLOBALS_SECTION.search(line):
            globals_start = i + 1
        if RE_FUNCTIONS_SECTION.search(line):
            functions_start = i
            break

    if globals_start is not None and functions_start is not None:
        result.globals_text = "\n".join(lines[globals_start:functions_start])

    # Extract arch ifdefs
    for line in lines:
        m = RE_IFDEF_ARCH.search(line)
        if m:
            macro = m.group(1)
            if macro not in result.arch_ifdefs:
                result.arch_ifdefs.append(macro)

    # Walk the AST

    def walk(node):
        if node.type == "function_definition":
            # `= delete` / `= default` — tree-sitter files both under function_definition, but
            # neither HAS a body, so neither is a translatable unit and neither may count as
            # under-translation. Recording them made _gate_unextracted HARD FAIL on a false
            # positive, and the only escape was --allow-partial, which suppresses the gate for
            # the WHOLE file — trading a false positive for blindness to the true ones. Found on
            # recyclebin_v5.cpp's `CZZWStr& operator=(const CZZWStr&) = delete;`.
            # NB `= default` does carry implicit semantics (a defaulted dtor destroys members);
            # those belong to the ctor/dtor lowering, which synthesises the bodies, not to the
            # per-function extractor asking "is there source code here we would miss?"
            if any(
                c.type in ("delete_method_clause", "default_method_clause") for c in node.children
            ):
                return
            func = _parse_function(node, source, source_text)
            if func:
                # A function DEFINED in a C header has no symbol of its own: it is
                # FORCEINLINE (`__forceinline` = gnu_inline `extern __inline__`, which emits
                # no out-of-line copy) or `static inline`, compiled privately into every
                # includer. Translated as exported, identify_funcs.h's 44 helpers came out
                # `#[no_mangle]` -- one duplicate symbol away from a link failure the moment a
                # second crate ports the same header. (C++ headers are not touched: their
                # inline methods belong to the class path.)
                if is_header_c:
                    func.is_static = True
                result.functions.append(func)
            else:
                # C++ method (qualified/field/destructor/operator declarator)
                # or otherwise unparseable body — record it so callers can
                # fail loud instead of silently under-translating the file.
                decl_text = ""
                for child in node.children:
                    if child.type in ("function_declarator", "pointer_declarator"):
                        decl_text = _extract_text(child, source).split("(")[0].strip()
                        break
                result.unextracted.append(
                    f"{decl_text or '<unparsed>'} @ L{node.start_point[0] + 1}"
                )
            # Collect identifiers from the function
            _collect_identifiers(node, source, result.identifiers)
            # Collect local-variable / parameter declarator names so the
            # resolver can exclude them (they're never FFI imports).
            _collect_local_decls(node, source, result.local_names)
            return  # don't recurse into function body again

        if node.type == "declaration":
            # Top-level declarations could be globals
            _collect_identifiers(node, source, result.identifiers)

        if node.type == "preproc_include":
            for child in node.children:
                if child.type in ("string_literal", "system_lib_string"):
                    result.includes.append(_extract_text(child, source))

        if node.type == "preproc_def":
            for child in node.children:
                if child.type == "identifier":
                    result.defines.append(_extract_text(child, source))
                    break

        for child in node.children:
            walk(child)

    walk(root)

    # Derive the file-scope preamble: every top-level construct that is NOT
    # a function body and NOT an #include (imports are the resolver's job).
    # This captures typedefs, struct/union/enum specifiers, #define /
    # #define-function macros, file-scope declarations (static lookup
    # tables, globals) and arch #ifdef type blocks — in source order, with
    # original spacing. Marker-independent: works on raw drivers like
    # arm64input.c that have no /* GLOBALS */ markers, where globals_text
    # is empty. `translate` feeds this through the LLM as one preamble unit.
    _SKIP_TOPLEVEL = {"function_definition", "preproc_include", "comment"}
    preamble_parts: list[str] = []

    def _collect_preamble(children):
        # Append every top-level construct that is NOT a function body / include
        # / comment. C++ `extern "C" { ... }` is a `linkage_specification`
        # wrapping a `declaration_list`; flatten it so its inner functions
        # (already captured as CFunctions via walk) aren't re-dumped as text and
        # its inner typedefs/decls still land in the preamble.
        for child in children:
            if child.type in _SKIP_TOPLEVEL:
                continue
            if child.type == ";":  # stray empty top-level statement
                continue
            if child.type == "linkage_specification":
                body = child.child_by_field_name("body")
                if body is not None and body.type == "declaration_list":
                    _collect_preamble(body.children)
                    continue
                # `extern "C" <single-decl>;` (no braces) — keep verbatim.
            if child.type in ("{", "}"):  # declaration_list braces when flattened
                continue
            if child.has_error:  # a balanced `#ifdef` around a whole function is not a defect
                result.preamble_swallowed_functions += _count_function_definitions(child)
            preamble_parts.append(_extract_text(child, source))

    _collect_preamble(root.children)
    result.preamble_text = "\n".join(preamble_parts).strip()

    # Parse globals from the globals section text using simple regex
    if result.globals_text:
        for line in result.globals_text.splitlines():
            line = line.strip()
            if not line or line.startswith("//") or line.startswith("/*"):
                continue
            # Match patterns like: TYPE NAME = INIT;  or  TYPE NAME;
            # This is simplified — globals can be complex
            m = re.match(r"^(\w[\w\s\*]+?)\s+(\w+)\s*(?:=\s*(.+?))?\s*;", line)
            if m:
                result.globals.append(
                    CGlobal(
                        name=m.group(2),
                        type_str=m.group(1).strip(),
                        initializer=m.group(3).strip() if m.group(3) else None,
                        line=0,  # approximate
                    )
                )

    return result


def extract_class_context(scan: ScanResult, class_name: str, max_chars: int = 8000) -> str | None:
    """Return the class/struct definition for `class_name` with in-class method
    BODIES stripped (declarations kept) — the field-layout/type context a
    per-method translation prompt needs without re-sending every body.

    Looks in the scanned .cpp first, then in sibling headers (explorer keeps
    CAppBarManager in appbar.h, CTrayShowDesktopButton in traydeskbtn.h, …).
    """

    def _find_and_strip(source: bytes) -> str | None:
        tree = Parser(CPP_LANG).parse(source)
        hit = None

        def find(node):
            nonlocal hit
            if hit is not None:
                return
            if node.type in ("class_specifier", "struct_specifier"):
                for c in node.children:
                    if c.type == "type_identifier" and _extract_text(c, source) == class_name:
                        hit = node
                        return
            for c in node.children:
                find(c)

        find(tree.root_node)
        if hit is None:
            return None
        cuts: list[tuple[int, int]] = []

        def collect_bodies(node):
            if node.type == "function_definition":
                for c in node.children:
                    if c.type == "compound_statement":
                        cuts.append((c.start_byte, c.end_byte))
                return
            for c in node.children:
                collect_bodies(c)

        collect_bodies(hit)
        text = source[hit.start_byte : hit.end_byte]
        for s, e in sorted(cuts, reverse=True):
            text = text[: s - hit.start_byte] + b";" + text[e - hit.start_byte :]
        return text.decode("utf-8", errors="replace")

    out = _find_and_strip(scan.raw_source.encode("utf-8"))
    if out is None:
        for header in sorted(scan.path.parent.glob("*.h")):
            try:
                out = _find_and_strip(header.read_text(errors="replace").encode("utf-8"))
            except OSError:
                continue
            if out is not None:
                break
    if out is not None and len(out) > max_chars:
        out = out[:max_chars] + "\n/* … class context truncated … */"
    return out


def print_scan_result(result: ScanResult) -> None:
    """Print scan results in human-readable form."""
    print(f"File: {result.path}")
    print(f"Includes: {len(result.includes)}")
    print(f"Defines: {len(result.defines)}")
    print(f"Arch ifdefs: {result.arch_ifdefs or 'none'}")
    print()

    if result.globals:
        print(f"Globals ({len(result.globals)}):")
        for g in result.globals:
            init = f" = {g.initializer}" if g.initializer else ""
            print(f"  {g.type_str} {g.name}{init}")
        print()

    print(f"Functions ({len(result.functions)}):")
    for f in result.functions:
        cc = f.calling_convention or ""
        static = "static " if f.is_static else ""
        seg = f"[{f.code_seg}] " if f.code_seg else ""
        seh = f" (SEH×{f.seh_count})" if f.seh_count else ""
        kind = " [ctor]" if f.is_ctor else (" [dtor]" if f.is_dtor else "")
        params_str = ", ".join(f"{t} {n}" for t, n in f.params)
        print(
            f"  {seg}{static}{f.return_type} {cc} {f.display_name}({params_str}){seh}{kind}"
            f"  L{f.line_start}-{f.line_end}"
        )

    if result.unextracted:
        n = len(result.unextracted)
        print(f"\n⚠ NOT extracted ({n}) — C++ method bodies the per-function")
        print("  pipeline cannot reach yet (translate/submit hard-fail on these):")
        for entry in result.unextracted[:20]:
            print(f"    {entry}")
        if n > 20:
            print(f"    … and {n - 20} more")

    print(f"\nTotal identifiers: {len(result.identifiers)}")
