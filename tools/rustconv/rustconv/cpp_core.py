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
import re
import subprocess
from collections import defaultdict
from dataclasses import replace
from pathlib import Path
from typing import TYPE_CHECKING

from .lex_mask import mask_code

if TYPE_CHECKING:
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


def _eval_c_int_constexpr(text: str) -> int | None:
    """Evaluate an arch-independent integer constant expression (array bounds).

    Accepts integer literals, character literals, parentheses, and the integer
    operators ``+ - * << >> | & ^ ~`` (and unary +/-). Returns None for anything
    outside that whitelist — names, calls, floats, division — so the caller fails
    loud instead of guessing a bound. Char literals become their code point, so a
    portable bound like ``('Z' - 'A' + 1)`` resolves to 26 without depending on
    any architecture.
    """

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
    for node in ast.walk(tree):
        if not isinstance(node, allowed):
            return None
        if isinstance(node, ast.Constant) and not (
            isinstance(node.value, int) and not isinstance(node.value, bool)
        ):
            return None
    try:
        result = eval(compile(tree, "<c-constexpr>", "eval"), {"__builtins__": {}}, {})
    except (ValueError, TypeError, ZeroDivisionError):
        return None
    return result if isinstance(result, int) else None


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
    values: dict[str, int] = {}
    for name in names:
        found: set[int] = set()
        pattern = re.compile(
            rf"^[ \t]*#[ \t]*define[ \t]+{re.escape(name)}[ \t]+(.+?)[ \t]*$", re.M
        )
        for root in search_roots:
            for header in root.rglob("*.[hi]"):
                text = header.read_text(encoding="utf-8", errors="replace")
                for replacement in pattern.findall(text):
                    replacement = re.sub(r"//.*$|/\*.*?\*/", "", replacement).strip()
                    value = _eval_c_int_constexpr(replacement)
                    if value is not None:
                        found.add(value)
        if len(found) == 1:
            values[name] = found.pop()
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
    process = subprocess.run(
        [
            "rg",
            "-n",
            "--no-heading",
            "--glob",
            "*.h",
            pattern,
            *(str(root) for root in search_roots if root.is_dir()),
        ],
        capture_output=True,
        text=True,
    )
    if process.returncode not in (0, 1):
        return {}
    values: dict[str, set[str]] = defaultdict(set)
    definition = re.compile(
        r"^\s*(?:#\s*define\s+)?(?P<name>[A-Za-z_]\w*)\s*(?:=\s*|\s+)"
        r"(?P<value>0[xX][0-9A-Fa-f]+|[0-9]+)[uUlL]*\b"
    )
    for line in process.stdout.splitlines():
        parts = line.split(":", 2)
        if len(parts) != 3:
            continue
        match = definition.match(parts[2])
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
    argument: str, symbols: dict[str, str], call_returns: dict[str, str] | None = None
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
    if re.fullmatch(r'L"(?:\\.|[^"\\])*"', text):
        return "LPCWSTR"
    if re.fullmatch(r'"(?:\\.|[^"\\])*"', text):
        return "LPCSTR"
    identifier = re.fullmatch(r"(?:this\s*->\s*)?([A-Za-z_]\w*)", text)
    if identifier is None:
        return None
    c_type = symbols.get(identifier.group(1))
    if c_type == "CStringW":
        return "LPCWSTR"
    if c_type == "CStringA":
        return "LPCSTR"
    return c_type


def _abi_types_compatible(actual: str, expected: str) -> bool:
    left, right = _abi_c_type_key(actual), _abi_c_type_key(expected)
    if left == right:
        return True
    if all("PIDL" in item or "ITEMID" in item for item in (left, right)):
        return True
    return {left, right} in ({"LPWSTR", "LPCWSTR"}, {"LPSTR", "LPCSTR"})


def _abi_c_type_key(c_type: str) -> str:
    """Collapse spelling-only Win32 aliases for overload binding, not codegen."""
    text = " ".join(c_type.replace("CONST", "const").split())
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
