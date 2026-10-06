"""Generate a module's `rust_varargs_shim.c` from its C-variadic definitions.

Stable Rust cannot DEFINE a C-variadic function or `va_arg` (both are `c_variadic`); the tree's
answer is a per-module C shim plus `sdk/lib/rust/varargs-rust` (see that crate's docs and
`rust_varargs_shim.c.template`). Ten modules carry one, each written by hand from the
template. This writes it from the C instead: the pullers (PART 1, identical everywhere) and,
per variadic definition, a trampoline that does the one thing Rust cannot -- `va_start` --
and hands `&ap` to `<Name>_body`, which the Rust translation defines:

    #[no_mangle]
    pub unsafe extern "C" fn Name_body(<fixed params>, ap: *mut core::ffi::c_void) -> R

Inside it the C's `va_list` is `*(ap as *mut va_list)` -- a `char*` on all three of our
Windows targets -- and `va_arg(list, T)` is `VaArgs::from_ptr(&raw mut list as *mut c_void)
.arg::<T>()`. Rust callers in the crate keep calling `Name(..)` through an `extern "C"`
declaration of the variadic, which stable Rust allows.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

_COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
_CALLCONV = frozenset(
    {
        "NTAPI",
        "WINAPI",
        "WINAPIV",
        "CDECL",
        "__cdecl",
        "__stdcall",
        "APIENTRY",
        "CALLBACK",
        "STDMETHODCALLTYPE",
        "static",
        "inline",
        "__inline",
        "FORCEINLINE",
        "EXTERN_C",
    }
)
_ANNOTATION = re.compile(r"^(?:IN|OUT|OPTIONAL|_\w+_(?:opt_)?|_In_|_Out_|_Inout_)$")


@dataclass(frozen=True)
class VariadicDef:
    name: str
    header: str  # the definition as written up to (not including) the body's `{`
    ret: str  # the return type, calling convention and storage words removed
    fixed: list[str]  # the named parameters, C text
    names: list[str]  # their identifiers
    line: int


def _matching(text: str, i: int, open_c: str, close_c: str) -> int:
    depth = 0
    while i < len(text):
        if text[i] == open_c:
            depth += 1
        elif text[i] == close_c:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def _split_params(params: str) -> list[str]:
    out, depth, cur = [], 0, []
    for c in params:
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        if c == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


def _param_name(param: str) -> str:
    words = re.findall(r"[A-Za-z_]\w*", re.sub(r"\[[^\]]*\]", "", param))
    words = [w for w in words if not _ANNOTATION.match(w)]
    if len(words) < 2:
        raise ValueError(f"cannot name the parameter `{param}`")
    return words[-1]


def variadic_definitions(c_text: str) -> list[VariadicDef]:
    """Every function DEFINITION (not prototype) whose parameter list ends in `...`."""
    code = _COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), c_text)
    out = []
    for m in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", code):
        if m.group(1) in ("if", "while", "for", "switch", "return", "sizeof"):
            continue
        close = _matching(code, m.end() - 1, "(", ")")
        if close < 0:
            continue
        params = _split_params(code[m.end() : close])
        if not params or params[-1].split()[-1:] != ["..."]:
            continue
        if not code[close + 1 :].lstrip().startswith("{"):
            continue  # a prototype
        line_start = code.rfind("\n", 0, m.start()) + 1
        head = line_start
        while head > 0:  # back over the return-type lines to the previous item's end
            prev_start = code.rfind("\n", 0, head - 1) + 1
            prev = code[prev_start : head - 1].strip()
            if not prev or prev.endswith((";", "}")) or prev.startswith("#"):
                break
            head = prev_start
        ret_words = [w for w in code[head : m.start()].split() if w not in _CALLCONV]
        fixed = params[:-1]
        if not fixed:
            raise ValueError(f"{m.group(1)}: a variadic with no named parameter has no va_start")
        out.append(
            VariadicDef(
                name=m.group(1),
                header=c_text[head : close + 1].strip(),
                ret=" ".join(ret_words),
                fixed=fixed,
                names=[_param_name(p) for p in fixed],
                line=c_text.count("\n", 0, m.start()) + 1,
            )
        )
    return out


_TEMPLATE = (
    Path(__file__).resolve().parents[3] / "sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template"
)


def pullers() -> str:
    """PART 1 of the template, read from it: the one copy varargs-rust's externs mirror."""
    text = _TEMPLATE.read_text()
    found = re.findall(r"(?m)^.*\bvibeos_va_arg_\w+\(void \*app\).*$", text)
    if len(found) != 6:
        raise RuntimeError(f"{_TEMPLATE}: expected the 6 va_arg pullers, found {len(found)}")
    return "\n".join(found) + "\n"


def trampoline(d: VariadicDef, where: str) -> str:
    void = d.ret in ("void", "VOID")
    fixed = ", ".join(d.fixed)
    args = ", ".join(d.names)
    lines = [
        f"/* {d.name} -- {where}:{d.line} */",
        f"{d.ret} __cdecl {d.name}_body({fixed}, void *ap);",
        "",
        d.header,
        "{",
        "    va_list ap;",
    ]
    if not void:
        lines.append(f"    {d.ret} r;")
    lines.append(f"    va_start(ap, {d.names[-1]});")
    call = f"{d.name}_body({args}, (void *)&ap);"
    lines.append(f"    {call}" if void else f"    r = {call}")
    lines.append("    va_end(ap);")
    if not void:
        lines.append("    return r;")
    lines.append("}")
    return "\n".join(lines)


def render(includes: list[str], defs: list[tuple[VariadicDef, str]], module: str) -> str:
    parts = [
        f"/* rust_varargs_shim.c -- GENERATED for {module} by `rustconv port varargs`; do not",
        " * edit, re-run it. Stable Rust cannot define a C-variadic function or va_arg, so this",
        " * file holds only those two primitives: the pullers varargs-rust calls, and one",
        " * trampoline per variadic definition that does va_start and hands `&ap` to the Rust",
        " * `<Name>_body`. All logic stays in Rust (sdk/lib/rust/varargs-rust). */",
        "",
        "#include <stdarg.h>",
        *includes,
        "",
        pullers(),
    ]
    for d, where in defs:
        parts += [trampoline(d, where), ""]
    return "\n".join(parts)


def rust_signature(d: VariadicDef) -> str:
    """What the Rust side defines, for the report (types are the island's, by C name)."""
    return f"{d.name}_body({', '.join(d.names)}, ap: *mut c_void)"


def shim_for(tus: list[Path], repo: Path, module: str) -> tuple[str, list[VariadicDef]] | None:
    from .profile_derive import scan_preamble

    defs: list[tuple[VariadicDef, str]] = []
    includes: list[str] = []
    for tu in tus:
        text = tu.read_text(errors="replace")
        found = variadic_definitions(text)
        if not found:
            continue
        for line in scan_preamble(text)[0]:
            if line not in includes:
                includes.append(line)
        defs += [(d, str(tu.relative_to(repo))) for d in found]
    if not defs:
        return None
    return render(includes, defs, module), [d for d, _ in defs]
