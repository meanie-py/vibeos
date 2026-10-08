"""variadic_shim — a translated C-variadic DEFINITION, moved onto stable Rust.

Stable Rust can declare and call a C-variadic function but cannot DEFINE one (`c_variadic`,
E0658), so c2rust's `fn F(a: A, mut args: ...)` never compiles here. The tree's answer is
`varargs-rust`: a 3-line C trampoline does the `va_start` and calls a fixed-arity Rust twin
`F_v(a, ap_list)`, which reads arguments through `VaArgs` -- the logic stays in Rust. Ten
modules wrote that pair by hand (storport, pcix, shell32, msvcrt, ...); this writes it.

THE ORACLE IS THE C. A twin is written only for a function the module's C defines with `...`
after the same number of fixed parameters. The trampoline's prototype is spelled in primitive
C types read off the twin through the islands' alias chains, and must come out the same on
every arch (`ULONG_PTR` is the one pointer-sized pair allowed) -- or generation refuses.

Only the PULL flavour is rewritten: the body walks the list with `ap.arg::<T>()`. A body that
hands the list on (`args.as_va_list()` into a `v*printf`) is refused by name -- forwarding
needs the callee's `va_list` spelling, which is a decision, not a rewrite.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

_DEF = re.compile(
    r"(?P<attrs>(?:#\[[^\]]*\]\s*)*)(?P<vis>pub(?:\([^)]*\))?\s+)?unsafe\s+extern\s+\"C\"\s+fn\s+"
    r"(?P<name>[A-Za-z_]\w*)\s*\("
)
_VARIADIC_PARAM = re.compile(r",?\s*(?:mut\s+)?(?P<arg>[A-Za-z_]\w*)\s*:\s*\.\.\.\s*,?\s*$")
_VALIST_TYPE = re.compile(r"(?:::)?(?:core::ffi::)?VaList(?:Impl)?\b(?:\s*<[^>]*>)?")


@dataclass
class Rewrite:
    text: str
    names: list[str] = field(default_factory=list)
    refused: dict[str, str] = field(default_factory=dict)


def _close_paren(text: str, open_idx: int) -> int | None:
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i
    return None


def _close_brace(text: str, open_idx: int) -> int | None:
    depth = 0
    i = open_idx
    while i < len(text):
        c = text[i]
        if c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def rewrite(text: str) -> Rewrite:
    """Every variadic DEFINITION in `text` -> its `_v` twin plus an extern declaration of the
    variadic entry (so references to `F` still resolve -- to the C trampoline)."""
    out = Rewrite(text)
    pos = 0
    while True:
        m = _DEF.search(out.text, pos)
        if not m:
            return out
        open_p = m.end() - 1
        close_p = _close_paren(out.text, open_p)
        if close_p is None:
            return out
        params = out.text[open_p + 1 : close_p]
        v = _VARIADIC_PARAM.search(params)
        body_open = out.text.find("{", close_p)
        if not v or body_open < 0 or out.text[close_p:body_open].count(";"):
            pos = close_p
            continue
        body_close = _close_brace(out.text, body_open)
        if body_close is None:
            return out
        name, arg = m.group("name"), v.group("arg")
        fixed = params[: v.start()].rstrip().rstrip(",")
        ret = out.text[close_p + 1 : body_open].strip()
        body = out.text[body_open : body_close + 1]
        body = _VALIST_TYPE.sub("varargs_rust::VaArgs", body)
        body = re.sub(
            rf"\b{arg}\s*\.\s*clone\s*\(\s*\)", "varargs_rust::VaArgs::from_ptr(ap_list)", body
        )
        if re.search(rf"\b{arg}\b", body):
            out.refused[name] = (
                f"`{arg}` is passed on, not walked (forwarding a va_list): write this one by hand"
            )
            pos = body_close
            continue
        sep = ",\n    " if "\n" in params else ", "
        twin = (
            f"{m.group('attrs')}{m.group('vis') or ''}unsafe extern \"C\" fn {name}_v("
            f"{fixed}{sep}ap_list: *mut ::core::ffi::c_void{',' if sep != ', ' else ''}"
            f"{chr(10) if sep != ', ' else ''})"
            f"{' ' + ret if ret else ''} {body}"
        )
        decl_params = " ".join(re.sub(r"\bmut\s+", "", fixed).split())
        # The comment sits INSIDE the block so it rides with the declaration: integrate drops
        # the declaration when the island already has one, and must not orphan the comment.
        decl = (
            f'\n\nextern "C" {{\n'
            f"    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust\n"
            f"    // cannot define one); it `va_start`s and calls `{name}_v` above.\n"
            f'    pub fn {name}({decl_params}, ...){" " + ret if ret else ""};\n}}\n'
        )
        out.text = out.text[: m.start()] + twin + decl + out.text[body_close + 1 :]
        out.names.append(name)
        pos = m.start() + len(twin) + len(decl)


def c_variadic_arity(c_text: str, name: str) -> int | None:
    """How many fixed parameters the C DEFINITION of `name` has before `...`; None if the text
    does not define it variadic. The oracle that the Rust twin is what the C defines."""
    from .crate_fix import _matching_paren  # same scanner the fix stage reads C with

    for m in re.finditer(rf"\b{re.escape(name)}\s*\(", c_text):
        close = _matching_paren(c_text, m.end() - 1)
        if close is None or not c_text[close + 1 :].lstrip().startswith("{"):
            continue
        parts = [p.strip() for p in _split_top(c_text[m.end() : close])]
        if parts and parts[-1] == "...":
            return len(parts) - 1
    return None


_TWIN = re.compile(r'unsafe\s+extern\s+"C"\s+fn\s+(?P<name>[A-Za-z_]\w*)_v\s*\(')
_ALIAS = re.compile(
    r"^pub type (\w+) = ([^;]+);|^pub use self::(\w+) as (\w+);", re.M
)
_PRIM = {
    "c_int": "int", "i32": "int", "c_uint": "unsigned int", "u32": "unsigned int",
    "c_long": "long", "c_ulong": "unsigned long", "c_short": "short", "i16": "short",
    "c_ushort": "unsigned short", "u16": "unsigned short", "c_char": "char",
    "c_schar": "signed char", "i8": "signed char", "c_uchar": "unsigned char",
    "u8": "unsigned char", "c_longlong": "long long", "i64": "long long",
    "c_ulonglong": "unsigned long long", "u64": "unsigned long long",
    "usize": "__UINTPTR_TYPE__", "isize": "__INTPTR_TYPE__", "f64": "double",
    "c_double": "double",
}
# One C name per arch is the point; these pairs are the same pointer-sized integer.
_PTR_SIZED = {
    frozenset({"unsigned long", "unsigned long long"}): "__UINTPTR_TYPE__",
    frozenset({"long", "long long"}): "__INTPTR_TYPE__",
}


def twins(text: str) -> dict[str, tuple[list[tuple[str, str]], str]]:
    """`{name: ([(param, rust type)], rust return type or "")}` for every `F_v` twin."""
    out: dict[str, tuple[list[tuple[str, str]], str]] = {}
    for m in _TWIN.finditer(text):
        close = _close_paren(text, m.end() - 1)
        body = text.find("{", close or 0)
        if close is None or body < 0:
            continue
        parts = [p.strip() for p in _split_top(text[m.end() : close]) if p.strip()]
        if not parts or not parts[-1].startswith("ap_list"):
            continue
        params = []
        for p in parts[:-1]:
            pm = re.fullmatch(r"(?:mut\s+)?(\w+)\s*:\s*(.+)", p, re.S)
            if pm is None:
                break
            params.append((pm.group(1), " ".join(pm.group(2).split())))
        else:
            ret = text[close + 1 : body].strip()
            out[m.group("name")] = (params, ret[2:].strip() if ret.startswith("->") else "")
    return out


def alias_maps(per_arch_sources: list[list[str]]) -> list[dict[str, str]]:
    """One `alias -> target` map per arch, from that arch's island and -sys bindgen text."""
    maps = []
    for texts in per_arch_sources:
        aliases: dict[str, str] = {}
        for text in texts:
            for a, b, src, dst in _ALIAS.findall(text):
                if a:
                    aliases.setdefault(a, b.strip())
                else:
                    aliases.setdefault(dst, src)
        maps.append(aliases)
    return maps


def c_primitive(rust_type: str, maps: list[dict[str, str]]) -> str:
    """The C type a trampoline spells for `rust_type`: identical ABI on every arch, or raise."""
    seen: set[str] = set()
    for aliases in maps:
        t = rust_type.strip()
        for _ in range(12):
            if t.startswith(("*mut", "*const", "&", "Option<")):
                seen.add("void *")
                break
            last = t.split("::")[-1].strip()
            if last in _PRIM:
                seen.add(_PRIM[last])
                break
            if last not in aliases:
                raise ValueError(f"no primitive C spelling for `{rust_type}` (stuck at `{last}`)")
            t = aliases[last]
        else:
            raise ValueError(f"alias chain for `{rust_type}` does not end")
    if len(seen) == 1:
        return seen.pop()
    if frozenset(seen) in _PTR_SIZED:
        return _PTR_SIZED[frozenset(seen)]
    raise ValueError(f"`{rust_type}` is {sorted(seen)} across the arches")


def c_trampoline(name: str, params: list[tuple[str, str]], ret: str, maps: list[dict[str, str]]) -> str:
    """The shim entry: `name(..., ...)` -> `name_v(..., &ap)`, in primitive C types only.

    No module header is included: atapi.h pulls in <initguid.h>, so a shim that included it
    DEFINED every WMI GUID a second time beside the Rust crate's (link-guard caught
    MSIde_PortDeviceInfo_GUID). A trampoline needs the ABI, not the names.
    """
    if not params:
        raise ValueError(f"{name}: a variadic needs a named parameter to va_start from")
    typed = [f"{c_primitive(t, maps)} {p}" for p, t in params]
    c_ret = c_primitive(ret, maps) if ret and ret != "()" else "void"
    call = f"{name}_v({', '.join(p for p, _ in params)}, (void *)&ap)"
    void = c_ret == "void"
    return (
        f"extern {c_ret} {name}_v({', '.join(typed)}, void *ap);\n\n"
        f"{c_ret} __cdecl {name}({', '.join(typed)}, ...)\n{{\n    va_list ap;\n"
        + ("" if void else f"    {c_ret} r;\n")
        + f"    va_start(ap, {params[-1][0]});\n"
        + (f"    {call};\n" if void else f"    r = {call};\n")
        + "    va_end(ap);\n"
        + ("" if void else "    return r;\n")
        + "}\n"
    )


def _split_top(params: str) -> list[str]:
    out, depth, cur = [], 0, []
    for ch in params:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    out.append("".join(cur))
    return out


SHIM_PULLERS = """\
#include <stdarg.h>

/* va_arg pullers -- mirrored by the extern block in sdk/lib/rust/varargs-rust/src/lib.rs. */
int                vibeos_va_arg_i32(void *app) { return va_arg(*(va_list *)app, int); }
unsigned int       vibeos_va_arg_u32(void *app) { return va_arg(*(va_list *)app, unsigned int); }
long long          vibeos_va_arg_i64(void *app) { return va_arg(*(va_list *)app, long long); }
unsigned long long vibeos_va_arg_u64(void *app) { return va_arg(*(va_list *)app, unsigned long long); }
double             vibeos_va_arg_f64(void *app) { return va_arg(*(va_list *)app, double); }
void *             vibeos_va_arg_ptr(void *app) { return va_arg(*(va_list *)app, void *); }
"""


def write_shim(module_dir: Path, entries: dict[str, tuple[str, str]]) -> Path:
    """(Re)write `<module_dir>/rust_varargs_shim.c` for `entries` = {name: (tu path, trampoline)}.

    Generated whole each time from the crate's twins, so it never drifts from them.
    """
    body = "\n".join(
        f"/* {name} -- C definition in {Path(tu).name}; logic in Rust `{name}_v`. */\n{tramp}"
        for name, (tu, tramp) in sorted(entries.items())
    )
    text = (
        "/* rust_varargs_shim.c -- GENERATED by rustconv (variadic_shim.py); do not edit.\n"
        " *\n"
        " * Stable Rust cannot define a C-variadic function (`c_variadic`), so each entry below\n"
        " * is a trampoline: it `va_start`s and calls the Rust twin `<name>_v` with `&ap`.\n"
        " * Prototypes are primitive C (ABI-identical on every arch) so no module header -- and\n"
        " * none of its definitions -- is pulled in. Not a translation reverted to C.\n"
        " */\n\n" + SHIM_PULLERS + "\n" + body
    )
    path = module_dir / "rust_varargs_shim.c"
    path.write_text(text)
    return path


def ensure_dependency(cargo_toml: Path) -> bool:
    text = cargo_toml.read_text()
    if re.search(r"^varargs-rust\s*=", text, re.M):
        return False
    text = re.sub(
        r"^\[dependencies\]\s*\n",
        '[dependencies]\n# C-variadic entry points -> rust_varargs_shim.c (stable Rust cannot define one).\n'
        'varargs-rust = { path = "../varargs-rust" }\n',
        text,
        count=1,
        flags=re.M,
    )
    cargo_toml.write_text(text)
    return True
