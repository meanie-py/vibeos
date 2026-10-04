"""Language-level C/C++ facts, independent of any object model or dialect.

Declarator parsing, argument splitting, C++ literal spans, trailing default-argument
materialization, same-arity overload binding from declared types, C integer
constant-expression evaluation, and the mechanical C-type -> Rust mapping.

This is the beginning of a GENERIC C++ layer. It was carved out of cpp_com.py, where
roughly 70% of the tool's entire C++ capability had accumulated behind a COM-shaped door
— so translating any C++ that is not COM had almost nothing to stand on. Nothing here
knows what an interface, a vtable slot or an HRESULT contract is; COM keeps those, in
cpp_com.py, which imports from this module rather than the other way round.

Extracted as the transitive closure of a generic seed, so it carries no COM concept in
either direction. Keep it that way: if something here needs to ask about an interface,
it belongs on the other side of the door.
"""

from __future__ import annotations

import ast
import os
import re
from collections import defaultdict
from dataclasses import replace
from pathlib import Path
from typing import TYPE_CHECKING

from .lex_mask import mask_code


def _headers_under(root: Path) -> list[Path]:
    """Every *.h below root. Not Path.rglob: it raises FileNotFoundError when a
    directory vanishes mid-walk, and the gate runs tooling while builds come and go
    in the same tree -- a tmpXXXXXXXX directory created and removed under
    sdk/include/psdk failed test_cpp_com on a green master (2026-10-04). os.walk
    skips a directory it can no longer list."""
    out = []
    for parent, _dirs, files in os.walk(root):
        out.extend(Path(parent) / f for f in files if f.endswith(".h"))
    return out


if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

    from .callgraph import FunctionDep


class CppTranslationError(RuntimeError):
    """A C/C++ fact could not be established deterministically.

    Ambiguity must fail BEFORE payment: never ask a model to guess compiler semantics,
    ABI, constants or layout. cpp_com re-exports this as CppComError, which is the
    historical name and remains the spelling every caller uses.
    """


# Alias so code moved here verbatim keeps raising the name it was written with.
CppComError = CppTranslationError


def _split_args(text: str) -> list[str]:
    out, start, depth = [], 0, 0
    for i, char in enumerate(text):
        if char in "(<[":
            depth += 1
        elif char in ")>]":
            depth -= 1
        elif char == "," and depth == 0:
            out.append(text[start:i].strip())
            start = i + 1
    tail = text[start:].strip()
    if tail:
        out.append(tail)
    return out


def _eval_c_int_constexpr(
    text: str,
    names: Mapping[str, int] | None = None,
    field_size: Callable[[str, str], int | None] | None = None,
) -> int | None:
    """Evaluate an arch-independent integer constant expression (array bounds).

    Accepts integer literals, character literals, parentheses, and the integer
    operators ``+ - * << >> | & ^ ~`` (and unary +/-). Returns None for anything
    outside that whitelist — unknown names, arbitrary calls, floats, division — so
    the caller fails loud instead of guessing a bound. Char literals become their
    code point, so a portable bound like ``('Z' - 'A' + 1)`` resolves to 26 without
    depending on any architecture.

    Three widenings, each admitting only what can be PROVEN from the tree, because a
    wrong array bound is not a compile error — it silently mis-lays-out a struct and
    every body that touches it is wrong with nothing red anywhere:

    * ``names`` resolves bounds written in terms of other macros
      (``ICONLOCATION_CCHMAX (MAX_PATH + 1 + 11)``). Only macros already proven by
      the same agreement rule are in there; an unknown name still returns None.
    * ``max``/``min`` of proven operands. These are the SDK's own integer macros,
      and ``TYPENAME_CCHMAX max(100, ...)`` is unevaluatable without them.
    * ``RTL_FIELD_SIZE(TYPE, field)`` through ``field_size``, which measures the
      real field in the real header. Restricted to fixed-width scalar arrays, so a
      pointer or width-dependent field refuses rather than baking one arch's answer.
    """
    if field_size is not None:

        def _expand(match: re.Match[str]) -> str:
            size = field_size(match.group(1), match.group(2))
            return match.group(0) if size is None else str(size)

        text = re.sub(
            r"\bRTL_FIELD_SIZE\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)",
            _expand,
            text,
        )

    def _char_ord(match: re.Match[str]) -> str:
        inner = match.group(1)
        try:
            decoded = inner.encode("utf-8").decode("unicode_escape")
        except (UnicodeDecodeError, ValueError):
            return match.group(0)
        return str(ord(decoded)) if len(decoded) == 1 else match.group(0)

    expr = re.sub(r"'((?:\\.|[^'\\])+)'", _char_ord, text)
    expr = re.sub(r"\b(0[xX][0-9A-Fa-f]+|\d+)[uUlL]+\b", r"\1", expr)  # drop int suffixes
    try:
        tree = ast.parse(expr.strip(), mode="eval")
    except (SyntaxError, ValueError):
        return None
    allowed = (
        ast.Expression,
        ast.Call,
        ast.Name,
        ast.BinOp,
        ast.UnaryOp,
        ast.Constant,
        ast.Load,
        ast.Add,
        ast.Sub,
        ast.Mult,
        ast.LShift,
        ast.RShift,
        ast.BitOr,
        ast.BitAnd,
        ast.BitXor,
        ast.UAdd,
        ast.USub,
        ast.Invert,
    )
    env: dict[str, object] = {"max": max, "min": min}
    env.update(names or {})
    for node in ast.walk(tree):
        if not isinstance(node, allowed):
            return None
        if isinstance(node, ast.Constant) and not (
            isinstance(node.value, int) and not isinstance(node.value, bool)
        ):
            return None
        # A call is admitted only as `max(...)`/`min(...)`; a Name only when the
        # caller proved it. Everything else -- an unexpanded macro, a sizeof, a
        # helper -- is exactly the case that must stay unevaluatable.
        if isinstance(node, ast.Call) and not (
            isinstance(node.func, ast.Name) and node.func.id in {"max", "min"} and not node.keywords
        ):
            return None
        if isinstance(node, ast.Name) and node.id not in env:
            return None
    try:
        result = eval(compile(tree, "<c-constexpr>", "eval"), {"__builtins__": {}}, env)
    except (ValueError, TypeError, ZeroDivisionError):
        return None
    return result if isinstance(result, int) else None


# Widths that are the same on i386, amd64 and arm64. Deliberately NOT a general sizeof
# table: anything pointer-sized or otherwise arch-dependent is absent, so measuring a
# field that contains one fails instead of baking whichever arch ran the generator.
_FIXED_WIDTH_SCALARS = {
    "char": 1,
    "signed char": 1,
    "unsigned char": 1,
    "CHAR": 1,
    "UCHAR": 1,
    "BYTE": 1,
    "BOOLEAN": 1,
    "short": 2,
    "unsigned short": 2,
    "SHORT": 2,
    "USHORT": 2,
    "WORD": 2,
    "WCHAR": 2,
    "int": 4,
    "unsigned int": 4,
    "long": 4,
    "unsigned long": 4,
    "INT": 4,
    "UINT": 4,
    "LONG": 4,
    "ULONG": 4,
    "DWORD": 4,
    "BOOL": 4,
    "HRESULT": 4,
    "LONGLONG": 8,
    "ULONGLONG": 8,
    "DWORD64": 8,
    "INT64": 8,
    "UINT64": 8,
}


def _c_field_size(
    search_roots: Sequence[Path], type_name: str, field: str, names: Mapping[str, int]
) -> int | None:
    """Byte size of one field of a C struct, measured in the real header.

    Backs `RTL_FIELD_SIZE(TYPE, field)` in an array bound. Only a fixed-width scalar
    or an array of one is measurable: those have the same size on all three targets,
    so the answer is a property of the source rather than of whichever arch happened
    to generate. A pointer, a nested struct or an unresolvable bound returns None and
    the bound stays a loud failure.
    """
    body = None
    patterns = (
        re.compile(
            rf"typedef\s+struct\s+(?:[A-Za-z_]\w*)?\s*\{{(?P<body>[^{{}}]*)\}}\s*"
            rf"[^;]*\b{re.escape(type_name)}\b[^;]*;",
            re.S,
        ),
        re.compile(rf"\bstruct\s+{re.escape(type_name)}\s*\{{(?P<body>[^{{}}]*)\}}\s*;", re.S),
    )
    for root in search_roots:
        paths = [root] if root.is_file() else sorted(root.rglob("*.[hi]"))
        for path in paths:
            text = path.read_text(encoding="utf-8", errors="replace")
            if not re.search(rf"\b{re.escape(type_name)}\b", text):
                continue
            for pattern in patterns:
                match = pattern.search(text)
                if match is not None:
                    body = match.group("body")
                    break
            if body is not None:
                break
        if body is not None:
            break
    if body is None:
        return None
    body = re.sub(r"/\*.*?\*/|//[^\n]*", " ", body, flags=re.S)
    for raw in body.split(";"):
        declaration = " ".join(raw.split())
        if not declaration:
            continue
        member = re.fullmatch(
            r"(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*)\s+(?P<name>[A-Za-z_]\w*)"
            r"(?:\s*\[\s*(?P<count>[^\]]+)\s*\])?",
            declaration,
        )
        if member is None or member.group("name") != field:
            continue
        width = _FIXED_WIDTH_SCALARS.get(" ".join(member.group("type").split()))
        if width is None:
            return None
        count_text = member.group("count")
        if count_text is None:
            return width
        count = _eval_c_int_constexpr(count_text, names)
        return None if count is None else width * count
    return None


def _numeric_array_constants(source: Path, declarations: list[str]) -> dict[str, int]:
    """Resolve symbolic fixed-array bounds from compiler-facing headers.

    Every discovered spelling must agree. Bounds may be a decimal literal or an
    arch-independent integer constant expression (see `_eval_c_int_constexpr`);
    anything that fails to evaluate is dropped, so an unproven bound stays a loud
    preflight failure rather than a guess. Searches `sdk/include` plus the class's
    own DLL-module subtree (module-private headers like `shellrecyclebin/` hold
    some shell32 bounds).
    """
    names = {
        match.group(1)
        for declaration in declarations
        if (match := re.search(r"\[\s*([A-Za-z_]\w*)\s*\]", declaration))
    }
    if not names:
        return {}
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk" / "include").is_dir()), None
    )
    if repo_root is None:
        return {}
    search_roots = [repo_root / "sdk" / "include"]
    module_root = next(
        (
            p
            for p in source.parents
            if p.parent is not None and p.parent.name in {"win32", "win32ss"}
        ),
        None,
    )
    if module_root is not None and module_root.is_dir():
        search_roots.append(module_root)
    # The TU ITSELF defines bounds too. `filetypes.cpp` declares `ASSOC_CCHMAX` and
    # `TYPENAME_CCHMAX` beside the struct that uses them, so a header-only search found
    # neither and the struct's layout stayed unproven -- reported as an untranslatable TU
    # when the definition was twelve lines above the field.
    define_roots = [source, *search_roots]

    def measure(type_name: str, field: str, known: Mapping[str, int]) -> int | None:
        return _c_field_size(search_roots, type_name, field, known)

    # A fixpoint, because bounds are written in terms of each other: `ICONLOCATION_CCHMAX`
    # is `(MAX_PATH + 1 + 11)`, and `MAX_PATH` has to be resolved before it can be. Each
    # pass can only ADD names, so this terminates; a name that never resolves is simply
    # never added, which keeps it a loud failure downstream.
    values: dict[str, int] = {}
    pending = set(names)
    while True:
        resolved_this_pass = False
        for name in sorted(pending):
            found: set[int] = set()
            pattern = re.compile(
                rf"^[ \t]*#[ \t]*define[ \t]+{re.escape(name)}[ \t]+(.+?)[ \t]*$", re.M
            )
            for root in define_roots:
                paths = [root] if root.is_file() else root.rglob("*.[hi]")
                for header in paths:
                    text = header.read_text(encoding="utf-8", errors="replace")
                    for replacement in pattern.findall(text):
                        replacement = re.sub(r"//.*$|/\*.*?\*/", "", replacement).strip()
                        value = _eval_c_int_constexpr(
                            replacement,
                            values,
                            lambda t, f: measure(t, f, values),
                        )
                        if value is not None:
                            found.add(value)
            # Every discovered spelling must still AGREE. A name defined differently in two
            # headers is ambiguous, not resolved-twice, and stays unproven.
            if len(found) == 1:
                values[name] = found.pop()
                resolved_this_pass = True
        pending -= values.keys()
        if not resolved_this_pass or not pending:
            break
    return values


_RUST_SCALARS = {
    "void": "core::ffi::c_void",
    "VOID": "()",
    "char": "i8",
    # The one entry here that needs no platform commitment: `signed char` is exactly 8 bits and
    # signed in every C implementation, where plain `char`'s signedness is implementation-defined
    # and the `i8` above is really a bet on MSVC. Its absence had nothing to do with doubt — the
    # table simply covered two of the three char spellings, and the third rejected CDefView's
    # `signed char m_SpecialFolder` as "not proven C scalar/pointer state".
    "signed char": "i8",
    "unsigned char": "u8",
    "short": "i16",
    "unsigned short": "u16",
    "int": "i32",
    "unsigned int": "u32",
    "long": "i32",
    "unsigned long": "u32",
}


def c_type_to_rust(c_type: str) -> str:
    """Mechanical C/C++ declarator -> Rust type mapping.

    Handles both `const T *` and `T const *` spellings, references, pointer depth,
    explicit `struct`/`enum`/`union` tag keywords, and qualified nested type names.
    """
    text = " ".join(c_type.replace("CONST", "const").split())
    # Headers use both ``const T *`` and ``T const *``.  A qualifier left in
    # the base spelling becomes invalid Rust (``*mut T const``), so normalize
    # either C spelling before building the pointer declarator.
    is_const = re.search(r"\bconst\b", text) is not None
    text = re.sub(r"\bconst\b", "", text)
    text = " ".join(text.split())
    is_reference = text.endswith("&")
    if is_reference:
        text = text[:-1].rstrip()
    stars = text.count("*")
    base = " ".join(text.replace("*", " ").split())
    # C and IDL-generated headers keep the explicit tag keyword in some signatures
    # (``struct tagNMHDR *``). Rust names the tag type directly; retaining the
    # C keyword produces invalid ``*mut struct tagNMHDR`` syntax.
    base = re.sub(r"^(?:struct|enum|union)\s+", "", base)
    # C++ nested PODs are emitted as module-level Rust structs by the
    # deterministic scaffold (``CShellLink::volume_info`` -> ``volume_info``).
    base = re.sub(r"\b[A-Za-z_]\w*::", "", base)
    rust = _RUST_SCALARS.get(base, base)
    for depth in range(stars):
        qualifier = "const" if is_const and depth == 0 else "mut"
        rust = f"*{qualifier} {rust}"
    if is_reference:
        rust = f"&{'' if is_const else 'mut '}{rust}"
    return rust


def _cpp_literal_spans(text: str) -> list[tuple[int, int]]:
    """(start, end) of every real string/char literal, comments excluded.

    A regex cannot find these. `case 'L':` closes a NARROW literal with the very
    two characters that open a wide one, so `L'(?:\\.|[^'\\])+'` latches onto the
    `L':` and runs to the next apostrophe — in shlexec.cpp that is 1,800 lines
    away, inside a comment reading "PIDL's". A literal boundary only exists
    relative to a scan that has already consumed everything before it, so scan.

    An `L` prefix is folded into the span it belongs to, which is what makes the
    narrow/wide distinction fall out: `'L'` starts at the quote, `L':'` starts one
    character earlier.
    """
    spans: list[tuple[int, int]] = []
    i, n = 0, len(text)
    while i < n:
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            i = n if end < 0 else end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            i = n if end < 0 else end + 2
        elif text[i] in "\"'":
            quote, start = text[i], i
            if start >= 1 and text[start - 1] == "L":
                prev = text[start - 2] if start >= 2 else ""
                if not (prev.isalnum() or prev == "_"):
                    start -= 1
            i += 1
            while i < n:
                if text[i] == "\\":
                    i += 2
                    continue
                if text[i] == quote:
                    i += 1
                    break
                i += 1
            spans.append((start, i))
        else:
            i += 1
    return spans


def _sdk_numeric_definitions(source_path: Path, used_words: set[str]) -> dict[str, str]:
    """Resolve simple referenced SDK numeric facts in one indexed header pass.

    Only architecture-invariant literal definitions are admitted. Expressions,
    aliases, casts, and conflicting spellings remain unresolved and visible to
    the substrate planner.
    """
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return {}
    pattern = (
        r"^\s*#\s*define\s+[A-Za-z_]\w*\s+(?:0[xX][0-9A-Fa-f]+|[0-9]+)[uUlL]*\b"
        r"|^\s*[A-Za-z_]\w*\s*=\s*(?:0[xX][0-9A-Fa-f]+|[0-9]+)[uUlL]*\s*,"
    )
    search_roots = [repo_root / "sdk/include"] + [
        repo_root / f"build-{arch}" / "sdk" / "include" / "psdk"
        for arch in ("i386", "amd64", "arm64")
    ]
    # In Python, not an `rg` shell-out: rg exits 2 when ANY file under a root is unreadable,
    # and that path returned {} -- every constant silently unresolved. latin-1 decodes every
    # byte, so a stray cp1252 byte cannot make a header "binary" and drop it either.
    candidate = re.compile(pattern)
    values: dict[str, set[str]] = defaultdict(set)
    definition = re.compile(
        r"^\s*(?:#\s*define\s+)?(?P<name>[A-Za-z_]\w*)\s*(?:=\s*|\s+)"
        r"(?P<value>0[xX][0-9A-Fa-f]+|[0-9]+)[uUlL]*\b"
    )
    for root in search_roots:
        if not root.is_dir():
            continue
        for header in sorted(_headers_under(root)):
            if not header.is_file():
                continue
            # Line by line, as rg matched: `^\s*` must not run across blank lines.
            for line in header.read_bytes().decode("latin-1").splitlines():
                if not candidate.search(line):
                    continue
                match = definition.match(line)
                if (
                    match is not None
                    and match.group("name") in used_words
                    and match.group("name") not in {"false", "true"}
                ):
                    values[match.group("name")].add(match.group("value"))
    return {name: next(iter(found)) for name, found in values.items() if len(found) == 1}


_masked_cpp = mask_code


def _matching_paren(text: str, opening: int) -> int:
    depth = 0
    for pos in range(opening, len(text)):
        if text[pos] == "(":
            depth += 1
        elif text[pos] == ")":
            depth -= 1
            if depth == 0:
                return pos
    raise CppComError("unbalanced helper call argument list")


def _argument_count(text: str, opening: int, closing: int) -> int:
    inner = text[opening + 1 : closing]
    if not inner.strip() or inner.strip() == "void":
        return 0
    depth = 0
    count = 1
    for char in inner:
        # Parentheses/brackets/braces can contain comma expressions. Do not
        # count angle brackets here: in an expression ``x < y`` is much more
        # common than an explicit-template argument containing a comma. The
        # latter stays outside this narrow arity-only overload subset and will
        # fail preflight instead of silently binding the wrong overload.
        if char in "([{":
            depth += 1
        elif char in ")]}":
            if depth:
                depth -= 1
        elif char == "," and depth == 0:
            count += 1
    return count


def _named_call_spans(text: str, names: set[str]) -> list[tuple[int, int, str, int]]:
    """Return identifier spans and arities for selected free-function calls.

    The same syntax covers a function definition's declarator. That is useful:
    helper definitions and every call site are renamed in one deterministic pass.
    """
    masked = _masked_cpp(text)
    spans = []
    for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked):
        name = match.group(1)
        if name not in names:
            continue
        opening = masked.find("(", match.start(1) + len(name))
        closing = _matching_paren(masked, opening)
        spans.append(
            (match.start(1), match.end(1), name, _argument_count(masked, opening, closing))
        )
    return spans


def _expand_default_helper_arguments(
    deps: list[FunctionDep], by_source: dict[str, list[FunctionDep]]
) -> list[FunctionDep]:
    """Materialize trailing C++ helper defaults before overload binding/prompting."""
    defaults: dict[str, tuple[int, tuple[str, ...]]] = {}
    for source_name, candidates in by_source.items():
        if len(candidates) != 1:
            continue
        dep = candidates[0]
        masked = _masked_cpp(dep.body_text)
        name_match = re.search(rf"\b{re.escape(source_name)}\s*\(", masked)
        if name_match is None:
            continue
        opening = masked.find("(", name_match.start())
        closing = _matching_paren(masked, opening)
        pieces = _split_args(dep.body_text[opening + 1 : closing])
        trailing: list[str] = []
        for piece in reversed(pieces):
            if "=" not in piece:
                break
            trailing.append(piece.split("=", 1)[1].strip())
        if trailing:
            trailing.reverse()
            defaults[source_name] = (len(pieces) - len(trailing), tuple(trailing))

    rewritten = []
    for dep in deps:
        body = dep.body_text
        definition_brace = body.find("{")
        for start, end, source_name, arity in reversed(_named_call_spans(body, set(defaults))):
            candidates = by_source[source_name]
            if dep in candidates and start < definition_brace:
                continue
            required, default_values = defaults[source_name]
            full_arity = required + len(default_values)
            if not required <= arity < full_arity:
                continue
            opening = body.find("(", end)
            closing = _matching_paren(_masked_cpp(body), opening)
            missing = default_values[arity - required :]
            insertion = (", " if arity else "") + ", ".join(missing)
            body = body[:closing] + insertion + body[closing:]
        rewritten.append(replace(dep, body_text=body))
    return rewritten


def _argument_c_type(
    argument: str,
    symbols: dict[str, str],
    call_returns: dict[str, str] | None = None,
    field_types: dict[str, dict[str, str]] | None = None,
    type_aliases: dict[str, str] | None = None,
) -> str | None:
    text = argument.strip()
    explicit_cast = re.search(r"\(\s*(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*\s*\*+)\s*\)", text)
    if explicit_cast is not None:
        return " ".join(explicit_cast.group("type").split())
    call = re.match(r"(?P<name>[A-Za-z_]\w*)\s*\(", text)
    if call is not None and call_returns is not None:
        return_type = call_returns.get(call.group("name"))
        if return_type is not None:
            return return_type
    text = re.sub(r"^\([^()]+\)\s*", "", text)
    conditional = re.fullmatch(r"[A-Za-z_]\w*\s*\?\s*([A-Za-z_]\w*)\s*:\s*(.+)", text, flags=re.S)
    if conditional is not None:
        true_type = symbols.get(conditional.group(1))
        false_text = conditional.group(2).strip()
        false_type = (
            "LPCWSTR"
            if re.fullmatch(r"CPP_COM_WSTR_[A-F0-9_]+\.as_ptr\(\)", false_text)
            else _argument_c_type(false_text, symbols, call_returns)
        )
        if true_type is not None and (
            false_type is None or _abi_types_compatible(true_type, false_type)
        ):
            return true_type
    # A NONZERO integer literal is an int and, unlike `0`, is not a null pointer
    # constant -- so it can never bind a pointer parameter. `0` is deliberately left
    # untyped: typing it `int` would discard a pointer candidate the source may well
    # have meant. (`SelectItem(-2, ...)` is CDefView's case.)
    integer_literal = re.fullmatch(r"[+-]?(?:0[xX][0-9a-fA-F]+|\d+)[uUlL]*", text)
    if integer_literal is not None:
        digits = text.rstrip("uUlL").lstrip("+-")
        value = int(digits, 16) if digits[:2].lower() == "0x" else int(digits)
        return "int" if value != 0 else None
    if re.fullmatch(r'L"(?:\\.|[^"\\])*"', text):
        return "LPCWSTR"
    if re.fullmatch(r'"(?:\\.|[^"\\])*"', text):
        return "LPCSTR"
    member = re.fullmatch(
        r"(?:this\s*->\s*)?(?P<base>[A-Za-z_]\w*)\s*(?P<op>\.|->)\s*(?P<field>[A-Za-z_]\w*)",
        text,
    )
    if member is not None:
        # C++ types `hittest.iItem` from the STRUCT, and without this a member access was
        # no evidence at all -- which is how two same-arity overloads stayed tied and hard
        # -failed before payment. The struct shape comes from the caller (the per-arch
        # bindgen islands), so this function stays free of any table of its own.
        if not field_types:
            return None
        base_type = symbols.get(member.group("base"))
        if base_type is None:
            return None
        # Win32 hides most stars behind a typedef (`LPNMLISTVIEW`), so resolve the
        # spelling before asking whether the access went through a pointer.
        if type_aliases:
            base_type = type_aliases.get(base_type, base_type)
        # `->` must go through a pointer and `.` must not. A mismatch means the symbol
        # table and the source disagree, so decline instead of typing the field anyway.
        indirect = "*" in base_type
        if indirect != (member.group("op") == "->"):
            return None
        return field_types.get(base_type.replace("*", "").strip(), {}).get(member.group("field"))
    identifier = re.fullmatch(r"(?:this\s*->\s*)?([A-Za-z_]\w*)", text)
    if identifier is None:
        return None
    c_type = symbols.get(identifier.group(1))
    if c_type == "CStringW":
        return "LPCWSTR"
    if c_type == "CStringA":
        return "LPCSTR"
    return c_type


# Unambiguous C/Win32 integer spellings. Used only to REJECT a candidate whose
# parameter is a pointer: an integer lvalue has no conversion to a pointer type in C++
# (a literal 0 does, but a literal is never typed here), so this can only ever discard
# a candidate the compiler discards too. Kept deliberately small -- a name that is not
# certainly an integer is simply absent, which decides nothing.
_C_INTEGER_KEYS = frozenset(
    {
        "INT",
        "UINT",
        "long",
        "unsigned long",
        "short",
        "unsigned short",
        "char",
        "signed char",
        "unsigned char",
        "long long",
        "unsigned long long",
        "DWORD",
        "LONG",
        "ULONG",
        "WORD",
        "BYTE",
        "SHORT",
        "USHORT",
        "UCHAR",
        "BOOL",
        "BOOLEAN",
        "SIZE_T",
        "INT_PTR",
        "UINT_PTR",
        "LONG_PTR",
        "ULONG_PTR",
        # Fixed by the Win32 ABI to integer types: LRESULT/LPARAM are LONG_PTR,
        # WPARAM is UINT_PTR, HRESULT/LSTATUS/NTSTATUS are LONG.
        "LRESULT",
        "LPARAM",
        "WPARAM",
        "HRESULT",
        "LSTATUS",
        "NTSTATUS",
    }
)


def is_c_integer(c_type: str) -> bool:
    """Whether ``c_type`` is certainly an integer. An absent name answers False, not True."""
    return _abi_c_type_key(c_type) in _C_INTEGER_KEYS


def integer_cannot_bind_pointer(actual: str, expected: str) -> bool:
    """True when ``actual`` is certainly an integer and ``expected`` is certainly a pointer."""
    return _abi_c_type_key(actual) in _C_INTEGER_KEYS and "*" in _abi_c_type_key(expected)


def _abi_types_compatible(actual: str, expected: str) -> bool:
    left, right = _abi_c_type_key(actual), _abi_c_type_key(expected)
    if left == right:
        return True
    if all("PIDL" in item or "ITEMID" in item for item in (left, right)):
        return True
    return {left, right} in ({"LPWSTR", "LPCWSTR"}, {"LPSTR", "LPCSTR"})


def _abi_c_type_key(c_type: str) -> str:
    """Collapse spelling-only Win32 aliases for overload binding, not codegen."""
    # A C++ `T&` parameter binds an LVALUE of type T, so for BINDING the reference is
    # not part of the type. (It very much is for codegen, which is why this collapse is
    # scoped to overload keys — the caller still emits the reference.)
    text = " ".join(c_type.replace("CONST", "const").split()).rstrip("&").strip()
    aliases = {
        "int": "INT",
        "unsigned int": "UINT",
        "PSTR": "LPSTR",
        "PWSTR": "LPWSTR",
        "PCSTR": "LPCSTR",
        "PCWSTR": "LPCWSTR",
        "LPCOLESTR": "LPCWSTR",
        "LPOLESTR": "LPWSTR",
        "LPVOID": "void *",
        "PVOID": "void *",
    }
    for spelling, canonical in aliases.items():
        text = re.sub(rf"\b{re.escape(spelling)}\b", canonical, text)
    return re.sub(r"\s*\*\s*", "*", text)


_ATL_NOT_A_METHOD = frozenset({"if", "for", "while", "return", "switch", "sizeof", "ATLASSERT"})

# A method declared at one level of nesting inside a class body: an optional storage
# specifier, a return type, the name, and a parameter list that ends the declaration
# (`;`) or opens an inline body (`{`).
_ATL_METHOD_RE = re.compile(
    r"(?m)^\s{4}(?:static\s+|virtual\s+)?[A-Za-z_][\w:*&<>\s]*?\b(\w+)\s*\(([^;{]*?)\)\s*"
    r"(?:const\s*)?[{;]"
)


def class_body_text(source: str, class_name: str) -> str | None:
    """The brace-balanced body of `class <class_name>`, or None when it is not here."""
    match = re.search(rf"(?m)^class\s+{re.escape(class_name)}\b[^{{;]*\{{", source)
    if match is None:
        return None
    depth, i = 1, match.end()
    while i < len(source) and depth:
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
        i += 1
    return source[match.end() : i]


def cwindow_method_arity(atlwin_source: str) -> dict[str, int]:
    """Declared parameter count of every ATL `CWindow` method, by name.

    A CWindow method call carries its window handle IMPLICITLY -- `DestroyWindow()` inside a
    `CWindowImpl`-derived class is `this->m_hWnd`'s window, and `m_ListView.ScreenToClient(&pt)`
    is that control's. The only Rust lowering is the Win32 API of the same name with the handle
    as the FIRST argument, so a faithful Rust call site has one MORE argument than the C++ one.

    Counting the C++ site alone therefore rejects correct output, and did: CDefView's
    `DestroyWindow wants 0, got 1` and `ScreenToClient wants 1, got 2` were both the model
    obeying the extern declaration the prompt handed it. `receiver_callees` already closes
    exactly this hole for SIBLING methods; a base-class method needs the same treatment and an
    authority for the count, which is this header rather than a remembered list.

    Overloads collapse to the LARGEST count, which only widens the accepted range -- the safe
    direction, since the C call site independently supplies the lower bound.
    """
    body = class_body_text(atlwin_source, "CWindow")
    if body is None:
        return {}
    arity: dict[str, int] = {}
    for match in _ATL_METHOD_RE.finditer(body):
        name, params = match.group(1), match.group(2)
        if name in _ATL_NOT_A_METHOD or name == "CWindow":
            continue
        arity[name] = max(arity.get(name, 0), _cwindow_param_count(params))
    return arity


def derives_from_cwindow(source: str, class_name: str) -> bool:
    """Whether `class_name`'s base list names CWindow or CWindowImpl.

    Only then does an unqualified `DestroyWindow()` in one of its bodies mean the member
    rather than the Win32 API -- the member hides the global, so the two can never be
    confused inside such a class.
    """
    match = re.search(rf"(?m)^class\s+{re.escape(class_name)}\b\s*:([^{{]*)\{{", source)
    if match is None:
        return False
    return re.search(r"\bCWindow(?:Impl)?\b", match.group(1)) is not None


def _cwindow_param_count(params: str) -> int:
    text = params.strip()
    return 0 if not text or text == "void" else len(_split_args(text))
