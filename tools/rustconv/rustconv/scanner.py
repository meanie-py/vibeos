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

C_LANG = Language(tsc.language())
CPP_LANG = Language(tscpp.language())

# Extensions that select the C++ grammar.
_CPP_SUFFIXES = (".cpp", ".cc", ".cxx", ".c++", ".cpp")

# Calling convention macros
CALLING_CONVENTIONS = {"NTAPI", "FASTCALL", "WINAPI", "CDECL", "__cdecl", "__stdcall", "__fastcall"}

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
# optionally followed by a (balanced-once) argument list.
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
    r"(?:\s*\([^()]*\))?"  # optional ( args ) — SAL args have no nesting
)


def _strip_sal(text: str) -> str:
    """Remove SAL annotations (no-op macros) so tree-sitter parses the function
    signatures of annotated Windows headers. Safe: only removes annotations."""
    return RE_SAL.sub("", text)


# Win32 calling-convention macros that tree-sitter-c CANNOT parse between the
# return type and the function name (it recovers catastrophically — e.g. a
# `static BOOL CALLBACK Foo(){...}` makes tree-sitter merge every following
# function into one giant function_definition, so concfg/font.c found 2 of 14
# fns). They are all __stdcall, which tree-sitter DOES parse (ms_call_modifier).
# NTAPI/WINAPI are intentionally NOT normalized here: their error-recovery is
# benign (separate function_definition nodes survive) and they're already in
# CALLING_CONVENTIONS, so the existing scans depend on seeing them verbatim.
RE_UNPARSEABLE_CC = re.compile(r"\b(?:CALLBACK|APIENTRY|APIPRIVATE|WINAPIV|PASCAL)\b")


def _normalize_callbacks(text: str) -> str:
    """Map the tree-sitter-unparseable __stdcall aliases to `__stdcall` so the
    function signatures parse. Differential-oracle safe (same ABI, only helps)."""
    return RE_UNPARSEABLE_CC.sub("__stdcall", text)


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
    m = RE_CODE_SEG.search(preceding)
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
        elif child.type == "function_declarator":
            declarator = child
        elif child.type == "pointer_declarator":
            # Return type is a pointer
            for sub in child.children:
                if sub.type == "function_declarator":
                    declarator = sub

    if declarator is None:
        return None

    # Extract function name from declarator
    for child in declarator.children:
        if child.type == "identifier":
            name = _extract_text(child, source)
        elif child.type == "parameter_list":
            for param_node in child.children:
                if param_node.type == "parameter_declaration":
                    param_text = _extract_text(param_node, source)
                    # Crude but effective: split into type and name
                    parts = param_text.rsplit(None, 1)
                    if len(parts) == 2:
                        ptype, pname = parts
                        # Remove pointer markers from name
                        while pname.startswith("*"):
                            ptype += " *"
                            pname = pname[1:]
                        params.append((ptype.strip(), pname.strip()))
                    elif parts and parts[0] != "VOID":
                        params.append((parts[0], ""))

    if name is None:
        return None

    # Count SEH blocks in body
    body_text = _extract_text(node, source)
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


def _eval_pp_cond(kw: str, expr: str) -> bool | None:
    """Resolve a preprocessor condition to True/False, or None if unknown."""
    expr = expr.split("//")[0].split("/*")[0].strip()
    if kw == "if":
        v = {"0": False, "1": True}.get(expr)
        if v is not None:
            return v
        if expr in _KNOWN_DEFINED_TRUE_SYMS:  # bare `#if DBG` → DBG=1 in the build
            return True
        return None
    sym = expr.split()[0] if expr.split() else ""
    if sym in _KNOWN_UNDEFINED_SYMS:
        return kw == "ifndef"  # #ifndef UNDEF → True, #ifdef UNDEF → False
    if sym in _KNOWN_DEFINED_TRUE_SYMS:
        return kw == "ifdef"  # #ifdef DBG → True, #ifndef DBG → False
    return None


def _resolve_known_preproc(text: str) -> str:
    """Resolve body-internal `#if 0`/`#if 1` and `#if(n)def` of known-undefined
    feature macros — keep the live branch, drop the dead branch + the directive
    lines — so tree-sitter parses function bodies that contain them. Leaves any
    condition it cannot resolve verbatim; bails on `#elif`/nesting (handles only
    a simple block with at most one `#else`)."""
    lines = text.split("\n")
    out, i, n = [], 0, len(lines)
    while i < n:
        m = _PP_IF.match(lines[i])
        if not m:
            out.append(lines[i])
            i += 1
            continue
        cond = _eval_pp_cond(m.group(1), m.group(2))
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

        new = re.sub(r"[A-Za-z_]\w*", _sub, expr)
        if new == expr:
            break
        expr = new
    # any leftover bare identifiers → undefined → 0
    expr = re.sub(r"[A-Za-z_]\w*", "0", expr)
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


def scan_file(path: Path) -> ScanResult:
    """Parse a C/C++ source file and extract structural information."""
    raw = _expand_split_includes(path, path.read_text(errors="replace"))
    is_cpp = path.suffix.lower() in _CPP_SUFFIXES
    if is_cpp:
        # Resolve preprocessor conditionals so #ifdef-spanning constructs don't
        # break tree-sitter function extraction (uniata etc.). Macros preserved.
        raw = _resolve_pp_conditionals(path, raw)
    source_text = _normalize_callbacks(
        _strip_sal(_expand_csr_api(_resolve_known_preproc(_strip_known_off_ifdefs(raw))))
    )
    source = source_text.encode("utf-8")

    parser = Parser(CPP_LANG if is_cpp else C_LANG)
    tree = parser.parse(source)

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
    cursor = tree.walk()

    def walk(node):
        if node.type == "function_definition":
            func = _parse_function(node, source, source_text)
            if func:
                result.functions.append(func)
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

    walk(cursor.node)

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
            preamble_parts.append(_extract_text(child, source))

    if cursor.node is not None:
        _collect_preamble(cursor.node.children)
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
        params_str = ", ".join(f"{t} {n}" for t, n in f.params)
        print(
            f"  {seg}{static}{f.return_type} {cc} {f.name}({params_str}){seh}"
            f"  L{f.line_start}-{f.line_end}"
        )

    print(f"\nTotal identifiers: {len(result.identifiers)}")
