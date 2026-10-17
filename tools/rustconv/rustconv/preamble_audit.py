"""Audit a translated file-scope preamble for SILENTLY DROPPED declarations.

`rustconv translate` sends the file-scope preamble (`extract_preamble()` in
scanner.py — everything that is not a function body or an `#include`) to the LLM
as ONE unit and pastes back whatever comes out. Nothing checked that the output
*covered* the input, and it does not: on `dll/win32/ole32/storage32.c` the model
was handed a 49,870-char preamble containing 10 COM vtable tables and returned
142 lines containing **none** of them.

That failure is invisible. A missing `#define` is a compile error you fix in a
minute; a missing vtbl table means the translated COM object has **no methods at
all**, and the only way you find out is at runtime. 23 such tables were missing
across ole32 and had to be hand-authored.

So: enumerate the file-scope declaration NAMES the C actually defines, check each
one appears as a definition in the returned Rust, and report what does not.
Loud beats silent (AGENTS.md: "Fail loud, never silently degrade").

Calibration — this exists to catch the catastrophic classes, so it is tuned to
NEVER miss a file-scope `static` table or a struct/union/enum definition, and to
stay quiet about the merely cosmetic:

* `extern` declarations are not definitions (they live in another TU) — not reported.
* `typedef struct Foo Foo;` folded into one `pub struct Foo` — not reported (a tag
  and its typedef alias cover each other).
* An object-like `#define` the *functions* inlined instead of emitting as a const
  is a normal, correct translation choice — reported only in `strict` mode, and
  never for include guards.

Public API:
    audit_preamble(c_path, rust_preamble, ...) -> list[str]   (human-readable findings)
    preamble_decls(c_path) -> list[CDecl]                     (what the C defines)
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

import tree_sitter_c as tsc
from tree_sitter import Language, Parser

# Same normalizer the function scanner uses, for the same reason: `FAR`/`NEAR` are
# #defined to nothing, tree-sitter does not know that, and a parameter spelled
# `CONST CHAR FAR *p` fails the whole enclosing function. Reused rather than
# reimplemented so both parsers agree on what the C says.
from .scanner import (
    CPP_LANG,
    RE_CODE_SEG_BYTES,
    _blank_memmodel,
    _expand_case_macros,
    _normalize_callbacks,
    _strip_sal,
    mask_seh_for_parse,
)

C_LANG = Language(tsc.language())

#: Suffixes the function catalog parses with the C++ grammar (scanner.py picks the same way).
_CPP_SUFFIXES = (".cpp", ".cxx", ".cc", ".hpp", ".hxx")

#: Kinds whose absence BREAKS things (silently, in the vtbl case). Always reported.
#: Calling-convention macros, which tree-sitter reads as a declarator name when an `#if`
#: separates them from the function name (the arbiter library's NTDDI-split signatures).
_CALLING_CONVENTIONS = frozenset(
    {"NTAPI", "WINAPI", "APIENTRY", "CALLBACK", "STDCALL", "FASTCALL", "CDECL", "STDMETHODCALLTYPE"}
)

CRITICAL_KINDS = ("table", "variable", "struct", "union", "enum")

_GUARD_RE = re.compile(r"(^_|_H_?$|_INCLUDED_?$|^__)", re.I)


@dataclass(frozen=True)
class CDecl:
    """A file-scope thing the C source DEFINES (not merely declares)."""

    name: str
    kind: str  #: "table" | "variable" | "struct" | "union" | "enum" | "typedef" | "define"
    line: int
    detail: str = ""  #: e.g. the vtbl's type, for the report
    #: For an `enum`, its variant names. A C enum's payload is its variants (plain
    #: constants), so translating `enum foo { A=1, B=4 };` to `pub const A/B` loses
    #: nothing — unlike a struct, whose identity carries layout.
    members: tuple[str, ...] = ()

    @property
    def is_critical(self) -> bool:
        return self.kind in CRITICAL_KINDS


def _txt(node, src: bytes) -> str:
    return src[node.start_byte : node.end_byte].decode("utf-8", "replace")


def _declarator_name(node, src: bytes) -> tuple[str | None, bool]:
    """(innermost declarator name, is_function_declarator)."""
    is_fn = False
    cur = node
    while cur is not None:
        if cur.type == "function_declarator":
            is_fn = True
        if cur.type in ("identifier", "type_identifier", "field_identifier"):
            return _txt(cur, src), is_fn
        nxt = cur.child_by_field_name("declarator")
        if nxt is None:
            for ch in cur.named_children:
                if ch.type.endswith("declarator") or ch.type == "identifier":
                    nxt = ch
                    break
        cur = nxt
    return None, is_fn


def _scan(c_path: Path) -> tuple[list[CDecl], dict[str, str], list[int]]:
    """(file-scope definitions, tag<->typedef alias map, unparsed top-level lines).

    Parsed from the real AST (not the carved preamble text) so line numbers are
    true and the parse is clean; a top-level declaration is by construction never
    inside a function body, so this is exactly the preamble's semantic content.

    That "by construction" holds only while the parse SUCCEEDS. When tree-sitter
    fails on a function's parameter list it recovers by hoisting the body's
    statements to the root, and every local then looks like a file-scope
    definition — `WSAttemptAutodialAddr(IN CONST SOCKADDR FAR *Name)` reported its
    locals `Ip`/`NetBios`/`AutodialAddress` as dropped file-scope state. Garbage
    findings are worse than none: they teach you to skim past the real ones.

    So: blank the memory-model macros first (scanner._blank_memmodel — the same
    no-op `FAR`/`NEAR` that once dropped `inet_addr` from a translation plan
    entirely, and which preserves line numbers), then REFUSE to mine any root that
    still holds a top-level ERROR. Note `root.has_error` is useless as that gate —
    it is True for nearly every real Windows TU because of macros inside bodies,
    which do not hoist anything. A top-level ERROR node is the precise signal.
    """
    raw = Path(c_path).read_text(encoding="utf-8", errors="replace")
    # Both normalizers, for the same reason and with the same guarantee: they make
    # shapes tree-sitter cannot parse parse, and neither changes the line count.
    # Blanking only the memory-model macros left `FILE * CDECL fopen(...)` — a
    # convention keyword after a pointer return type — as a top-level ERROR, so this
    # audit returned INCOMPLETE for every msvcrt file and the file-scope checks it
    # exists for never ran. The function catalog normalizes both; so must this, or
    # the two disagree about what the file even contains.
    # Three normalizers now, same reason and same guarantee (none changes the line count).
    # `_expand_case_macros` is the one this comment's warning was written for: it went into
    # the function catalog first, and until it landed here too the two disagreed about
    # errno.c — the catalog saw 18 functions while this audit still called line 132 a
    # top-level ERROR and skipped every file-scope check it exists to perform.
    # And a fourth: the catalog blanks `CODE_SEG("PAGE")` for its parse (scanner's
    # RE_CODE_SEG_BYTES); left in here, every PAGED function of the arbiter library read as a
    # top-level ERROR and all six of its files skipped their file-scope checks.
    raw = RE_CODE_SEG_BYTES.sub(lambda m: b" " * len(m.group()), raw.encode()).decode()
    text = _normalize_callbacks(_blank_memmodel(_expand_case_macros(_strip_sal(raw))))
    # A fifth and sixth: the catalog also strips SAL (`_Out_writes_bytes_(DataLen)` in a
    # parameter list is a top-level ERROR) and parses with SEH/RpcTryExcept/__TRY masked
    # to same-length C (a nested RpcTryExcept ends the function early, its tail reads as a
    # top-level ERROR). setupapi's cfgmgr.c skipped every file-scope check on both.
    src = mask_seh_for_parse(text.encode())
    # Parse a .cpp with the C++ grammar, as the function catalog already does. Using the C
    # grammar on C++ makes `class X : public Y {` a top-level ERROR by construction, which
    # this function reads as "cannot trust the parse" and returns INCOMPLETE -- so every
    # file-scope check was skipped for exactly the files most likely to hold a hand-rolled
    # vtable. recyclebin_v5_enumerator.cpp (two COM classes) never got audited at all.
    language = CPP_LANG if Path(c_path).suffix.lower() in _CPP_SUFFIXES else C_LANG
    tree = Parser(language).parse(src)
    root = tree.root_node

    unparsed = [n.start_point[0] + 1 for n in root.named_children if n.type == "ERROR"]
    if unparsed:
        return [], {}, unparsed

    decls: dict[tuple[str, str], CDecl] = {}
    tags: set[str] = set()  # struct/union/enum tags that got a definition
    typedef_of: dict[str, str] = {}  # typedef name -> underlying tag

    def add(d: CDecl) -> None:
        key = (d.kind, d.name)
        # A definition (with initializer) supersedes an earlier forward decl.
        if key in decls and decls[key].detail and not d.detail:
            return
        decls[key] = d

    def record_tag(node) -> str | None:
        """Register a struct/union/enum DEFINITION (must have a body)."""
        name_node = node.child_by_field_name("name")
        body = node.child_by_field_name("body")
        if name_node is None or body is None:
            return None
        name = _txt(name_node, src)
        kind = {"struct_specifier": "struct", "union_specifier": "union"}.get(node.type, "enum")
        members: tuple[str, ...] = ()
        if kind == "enum":
            members = tuple(
                _txt(e.child_by_field_name("name") or e, src)
                for e in body.named_children
                if e.type == "enumerator"
            )
        tags.add(name)
        add(
            CDecl(
                name=name,
                kind=kind,
                line=node.start_point[0] + 1,
                detail="def",
                members=members,
            )
        )
        return name

    for node in root.named_children:
        # A bare `struct Bare { int b; };` at file scope is a top-level
        # struct_specifier — NOT wrapped in a `declaration`. Missing this case
        # would drop struct definitions silently, the exact class this exists to
        # catch.
        if node.type in ("struct_specifier", "union_specifier", "enum_specifier"):
            record_tag(node)

        elif node.type == "preproc_def":
            name_node = node.child_by_field_name("name")
            val = node.child_by_field_name("value")
            if name_node is not None and val is not None:
                add(
                    CDecl(
                        name=_txt(name_node, src),
                        kind="define",
                        line=node.start_point[0] + 1,
                        detail=_txt(val, src).strip(),
                    )
                )

        elif node.type == "type_definition":
            for ch in node.named_children:
                if ch.type in ("struct_specifier", "union_specifier", "enum_specifier"):
                    tag = record_tag(ch)
                    for d in node.children_by_field_name("declarator"):
                        nm, _ = _declarator_name(d, src)
                        if nm and tag:
                            typedef_of[nm] = tag
            for d in node.children_by_field_name("declarator"):
                nm, is_fn = _declarator_name(d, src)
                if nm and not is_fn:
                    add(CDecl(name=nm, kind="typedef", line=node.start_point[0] + 1))

        elif node.type == "declaration":
            raw = _txt(node, src)
            # `extern` is a DECLARATION, not a definition — the symbol lives in
            # another TU, so the Rust preamble is right not to define it.
            if re.match(r"^\s*extern\b", raw):
                continue
            for ch in node.named_children:
                if ch.type in ("struct_specifier", "union_specifier", "enum_specifier"):
                    record_tag(ch)
            type_node = node.child_by_field_name("type")
            type_name = _txt(type_node, src).strip() if type_node is not None else ""
            for d in node.children_by_field_name("declarator"):
                has_init = d.type == "init_declarator"
                target = d.child_by_field_name("declarator") if has_init else d
                nm, is_fn = _declarator_name(target, src)
                if not nm or is_fn:
                    continue  # function prototype — the function pipeline owns it
                # `NTSTATUS NTAPI` + `#if (NTDDI_VERSION >= ...)` + the NAME: a function whose
                # two signatures are chosen between the convention and the name parses as a
                # declaration of a variable called NTAPI. A convention is never a variable.
                if nm in _CALLING_CONVENTIONS and not has_init:
                    continue
                # A `static const <X>Vtbl name = {...}` is the class that silently
                # produces method-less COM objects. Call it out by name.
                kind = "table" if has_init and type_name.endswith("Vtbl") else "variable"
                add(
                    CDecl(
                        name=nm,
                        kind=kind,
                        line=node.start_point[0] + 1,
                        detail=type_name if has_init else "",
                    )
                )

    # A forward declaration (`static const IStorageVtbl Foo;`, no initializer) is
    # the SAME symbol as its later definition (`... Foo = {..}`), which we classify
    # as a "table". Collapse it so a table is not also reported as a bare variable.
    table_names = {d.name for d in decls.values() if d.kind == "table"}
    for key in [k for k in decls if k[0] == "variable" and k[1] in table_names]:
        del decls[key]

    # A tag and its typedef alias cover each other: C `typedef struct TagFoo {..}
    # Foo;` legitimately becomes ONE `pub struct Foo` in Rust, so seeing either
    # name defined means neither was dropped. Bidirectional.
    aliases: dict[str, str] = {}
    for td, tag in typedef_of.items():
        aliases[td] = tag
        aliases[tag] = td
    # Also the body-less `typedef struct Foo Foo;` form (no struct body to record).
    for m in re.finditer(r"typedef\s+(?:struct|union|enum)\s+(\w+)\s+(\w+)\s*;", text):
        aliases.setdefault(m.group(1), m.group(2))
        aliases.setdefault(m.group(2), m.group(1))

    out = sorted(decls.values(), key=lambda d: (d.line, d.kind, d.name))
    return out, aliases, []


def preamble_decls(c_path: Path) -> list[CDecl]:
    """Every file-scope declaration the C source DEFINES."""
    return _scan(Path(c_path))[0]


# ---------------------------------------------------------------------------
# Presence in the Rust
# ---------------------------------------------------------------------------


def _defines_value(rust: str, name: str) -> bool:
    """`pub static NAME: T = ..` / `pub const NAME: T = ..` / `static mut NAME`."""
    return bool(re.search(rf"\b(?:static|const)\s+(?:mut\s+)?{re.escape(name)}\s*:", rust))


def _defines_type(rust: str, name: str) -> bool:
    """`pub struct NAME` / `enum` / `union` / `type NAME =`."""
    return bool(re.search(rf"\b(?:struct|enum|union|type)\s+{re.escape(name)}\b", rust))


def _mentioned(rust: str, name: str) -> bool:
    return bool(re.search(rf"\b{re.escape(name)}\b", rust))


def _covered(decl: CDecl, rust: str, aliases: dict[str, str]) -> bool:
    if decl.kind in ("table", "variable"):
        return _defines_value(rust, decl.name)
    if decl.kind in ("struct", "union", "enum", "typedef"):
        if _defines_type(rust, decl.name):
            return True
        # A tag may have been emitted under its typedef alias (or vice-versa).
        alias = aliases.get(decl.name)
        if alias and _defines_type(rust, alias):
            return True
        # A C enum's payload is its VARIANTS, which are just constants: emitting
        # `pub const OleStream_Convert: u32 = 4;` instead of a Rust `enum` loses
        # nothing, and is what the model idiomatically does for a flags enum. If
        # every variant is defined, the enum is covered whatever the tag is called.
        if decl.kind == "enum" and decl.members:
            return all(_defines_value(rust, m) or _defines_type(rust, m) for m in decl.members)
        # A typedef may be folded into a value-position alias (`pub type X = u32;`)
        # or, for a pointer typedef, into a plain use. Accept a `type` alias only.
        return False
    if decl.kind == "define":
        return _defines_value(rust, decl.name) or _defines_type(rust, decl.name)
    return True


def audit_preamble(
    c_path: Path,
    rust_preamble: str,
    strict: bool = False,
    rust_full: str | None = None,
) -> list[str]:
    """File-scope C declarations MISSING from the translated Rust preamble.

    Args:
        c_path: the C source that was translated.
        rust_preamble: the LLM's preamble output (the `00_preamble.rs` artifact).
        strict: also report object-like `#define`s and typedefs that have no Rust
            definition. Off by default: inlining a `#define` into the function
            bodies is a legitimate translation, so these are noise, not bugs.
        rust_full: if given (the whole stitched module, functions included), a name
            defined *anywhere* in it counts as covered — kills the false positive
            where the preamble omits a const because a function body already has it.

    Returns:
        Human-readable findings, most-severe first. Empty list == preamble is complete.
        The critical kinds (`static` tables, file-scope variables, struct/union/enum
        definitions) are ALWAYS reported; nothing suppresses them.
    """
    c_path = Path(c_path)
    haystack = rust_preamble + ("\n" + rust_full if rust_full else "")
    decls, aliases, unparsed = _scan(c_path)

    findings: list[str] = []
    if unparsed:
        # Fail loud rather than mine parse debris: say the audit did NOT run.
        lines = ", ".join(str(n) for n in unparsed[:5])
        return [
            f"PREAMBLE AUDIT INCOMPLETE — {c_path.name} does not parse at top level "
            f"(line(s) {lines}); file-scope checks were SKIPPED, not passed. A dropped "
            "static or vtable in this file would go unreported. Usually an unknown "
            "annotation macro in a parameter list; add it to scanner's normalizers."
        ]
    for d in decls:
        if not d.is_critical and not strict:
            continue
        if d.kind == "define" and _GUARD_RE.search(d.name):
            continue  # include guard / reserved name
        if _covered(d, haystack, aliases):
            continue
        if d.kind == "define" and _mentioned(haystack, d.name):
            continue  # emitted under another shape, or already used — not a drop
        if d.kind == "table":
            findings.append(
                f"MISSING COM VTABLE '{d.name}' ({d.detail}) — {c_path.name}:{d.line}. "
                "The translated COM object has NO METHODS. Generate it with "
                "rustconv.vtbl_gen (deterministic), do not ask the model."
            )
        elif d.kind == "variable":
            findings.append(f"MISSING file-scope definition '{d.name}' — {c_path.name}:{d.line}.")
        elif d.kind in ("struct", "union", "enum"):
            findings.append(f"MISSING {d.kind} definition '{d.name}' — {c_path.name}:{d.line}.")
        elif d.kind == "typedef":
            findings.append(f"missing typedef '{d.name}' — {c_path.name}:{d.line}.")
        else:
            findings.append(f"missing #define '{d.name}' (= {d.detail}) — {c_path.name}:{d.line}.")

    rank = {"MISSING COM VTABLE": 0, "MISSING file-scope": 1, "MISSING": 2}
    findings.sort(key=lambda s: next((v for k, v in rank.items() if s.startswith(k)), 3))
    return findings
