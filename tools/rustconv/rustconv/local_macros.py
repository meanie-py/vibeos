"""A module's OWN function-like C macros, ported once into its prelude as `macro_rules!`.

WHY THIS EXISTS
---------------
bindgen emits no function-like macros, and the call seal deliberately leaves them alone
(macro_authority.py: a macro occurrence does not lower to one Rust call). So when a body
calls `DoesFileExist(Root, Name)` -- filesup.h's `#define DoesFileExist(R, F)
DoesPathExist((R), (F), FALSE)` -- the translation keeps the name and the crate has nothing
to resolve it to. SDK header macros are ported once in win32macros-rust; a module's own
macros (setuplib has 27, a Wine DLL routinely more) have had no home but hand-writing.

WHAT IT DOES
------------
Takes the names `cargo check` cannot find, keeps those #defined function-like in the
module's own sources, adds the local macros their bodies use (transitively), and asks the
model for all of them in ONE cheap call -- definitions are a few lines each. The answer is
checked mechanically: exactly the requested names, each once, nothing else, and nothing
but `macro_rules!` items. It lands between generated markers in the prelude and is banked
by a hash of the definitions, so a re-run is $0. integrate's `fix_macro_calls` then turns
every `NAME(` call site into `NAME!(`, and cargo check is the judge.
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path

BEGIN = "// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ===="
END = "// ==== end port macros ===="

_DEFINE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\(([^)]*)\)(.*)$")
# Object-like: `#define SpInfFindFirstLine (SpInfExports.SpInfFindFirstLine)`. Only kept when
# the body is an expression -- a literal is bindgen's or the fact seal's, and an empty body
# is a header switch.
_OBJECT = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)(?![\w(])[ \t]+(\S.*)$")
_LITERAL = re.compile(
    r'^\(?\s*(?:-?(?:0[xX][0-9A-Fa-f]+|\d+)[uUlL]*|L?"(?:[^"\\]|\\.)*"|L?\'.\')\s*\)?$'
)
_IDENT = re.compile(r"\b[A-Za-z_]\w*\b")


@dataclass(frozen=True)
class CMacro:
    name: str
    params: tuple[str, ...] | None  # None: an object-like macro
    body: str
    where: str
    #: "macro", or "fn" for a function DEFINED in a module header (`FORCEINLINE VOID
    #: INF_FreeData(..) {..}` in infsupp.h): no TU's worklist covers a header, so its inline
    #: bodies were never translated. `body` is then the whole C definition.
    kind: str = "macro"

    def c_text(self) -> str:
        if self.kind == "fn":
            return self.body
        if self.params is None:
            return f"#define {self.name} {self.body}"
        return f"#define {self.name}({', '.join(self.params)}) {self.body}"


def _logical_lines(text: str) -> list[tuple[int, str]]:
    out: list[tuple[int, str]] = []
    buf, start = "", 0
    for no, line in enumerate(text.splitlines(), 1):
        if not buf:
            start = no
        if line.endswith("\\"):
            buf += line[:-1] + " "
            continue
        out.append((start, buf + line))
        buf = ""
    return out


def _strip_comments(s: str) -> str:
    s = re.sub(r"/\*.*?\*/", " ", s)
    return re.sub(r"//.*$", "", s)


def module_macros(files: list[Path], root: Path) -> tuple[dict[str, CMacro], set[str]]:
    """Function-like macros these files define: (unambiguous defs, names defined twice apart)."""
    found: dict[str, list[CMacro]] = {}
    for f in files:
        try:
            text = f.read_text(errors="replace")
        except OSError:
            continue
        for no, line in _logical_lines(text):
            rel = f.relative_to(root) if f.is_relative_to(root) else f
            m = _DEFINE.match(line)
            if m:
                params: tuple[str, ...] | None = tuple(
                    p.strip() for p in m.group(2).split(",") if p.strip()
                )
                body = " ".join(_strip_comments(m.group(3)).split())
            elif (o := _OBJECT.match(line)) is not None:
                body = " ".join(_strip_comments(o.group(2)).split())
                if not body or _LITERAL.match(body):
                    continue
                m, params = o, None
            else:
                continue
            found.setdefault(m.group(1), []).append(CMacro(m.group(1), params, body, f"{rel}:{no}"))
    defs: dict[str, CMacro] = {}
    ambiguous: set[str] = set()
    for name, ms in found.items():
        if len({(m.params, m.body) for m in ms}) == 1:
            defs[name] = ms[0]
        else:
            ambiguous.add(name)
    return defs, ambiguous


def header_functions(headers: list[Path], root: Path) -> dict[str, CMacro]:
    """Functions DEFINED in the module's own headers (inline helpers), by the C scanner."""
    from .scanner import scan_file

    out: dict[str, CMacro] = {}
    for h in headers:
        try:
            scan = scan_file(h)
        except Exception:  # a header the scanner cannot read has nothing for us
            continue
        rel = h.relative_to(root) if h.is_relative_to(root) else h
        for fn in scan.functions:
            out[fn.name] = CMacro(fn.name, None, fn.body_text, f"{rel}:{fn.line_start}", "fn")
    return out


def closure(names: set[str], defs: dict[str, CMacro]) -> dict[str, CMacro]:
    """`names` plus every local macro their bodies use, transitively."""
    want: dict[str, CMacro] = {}
    todo = [n for n in sorted(names) if n in defs]
    while todo:
        n = todo.pop()
        if n in want:
            continue
        want[n] = defs[n]
        for ident in _IDENT.findall(defs[n].body):
            # Object-like macros join only by request: inside a body they are VALUES.
            if (
                ident in defs
                and ident not in want
                and ident != n
                and defs[ident].params is not None
            ):
                todo.append(ident)
    return want


SYSTEM = (
    "You port C preprocessor macros to Rust `macro_rules!` for a no_std crate. You output "
    "exactly one ```rust fenced block and nothing else."
)


#: Bumped whenever the prompt's rules change, so a reply bought under old rules never replays.
PROMPT_VERSION = 3


def prompt(macros: dict[str, CMacro], in_scope: frozenset[str] | set[str] = frozenset()) -> str:
    defs = "\n".join(
        m.c_text() + f"    /* {m.where} */"
        for m in sorted(macros.values(), key=lambda m: m.name)
        if m.kind == "macro"
    )
    fns = "\n\n".join(
        f"/* {m.where} */\n{m.c_text()}"
        for m in sorted(macros.values(), key=lambda m: m.name)
        if m.kind == "fn"
    )
    fn_part = (
        "\n\nAlso port these C functions, which the module's HEADERS define inline, as "
        "`#[inline] pub unsafe fn NAME(..)` items of the SAME name (not `extern`, not "
        "`#[no_mangle]`), parameters and return typed by the C's own type NAMES:\n\n" + fns
        if fns
        else ""
    )
    scope = (
        "\n- These are ALREADY Rust macros where yours expand, so call them with `!` too: "
        + ", ".join(sorted(in_scope))
        if in_scope
        else ""
    )
    return f"""Port each of these C function-like macros to a Rust `macro_rules!` of the SAME name.

{defs}{fn_part}

Rules:
- One `macro_rules! NAME {{ ... }}` per macro above (and one `pub unsafe fn` per function, if
  any), and nothing else: no consts, no `use`, no helpers, no comments outside the items.
- An OBJECT-like macro (no parameter list) is CALLED like a function in the crate, so it
  takes `($($arg:expr),* $(,)?)` and calls its expansion with them. A bindgen function-pointer
  field is an `Option<fn>`: `(SpInfExports.SpInfFindFirstLine)` is
  `(SpInfExports.SpInfFindFirstLine.unwrap())($($arg),*)`.
- Each C parameter becomes `$param:expr` (or `$param:ty` where C uses it as a type, `$param:ident`
  where C uses it as a field name), in the same order. Accept an optional trailing comma.
- Wrap an expression body in parentheses; a statement body (`{{...}}`, `do {{...}} while (0)`)
  becomes a `{{ ... }}` block.
- Keep C's meaning exactly: `p->f` is `(*$p).f`; `(T)x` is `($x) as T`; `sizeof(T)` is
  `core::mem::size_of::<T>()`; `L'c'` is `(b'c' as u16)`. Pointer arithmetic on bytes uses
  `.cast::<u8>().add(n)` / `wrapping_add` as the C does.
- A comparison or `&&`/`||`/`!` result is a Rust `bool`. Any other macro yields the value C
  computes, with C's type: do NOT convert an integer result to bool.
- Another macro from this list is called with `!`. Every other name (functions, constants,
  types, struct fields) is spelled exactly as in the C; it resolves where the macro is used.{scope}
- Follow the C's TYPES through every operator. `(T*)(int_expr + n)` adds INTEGERS and casts
  last -- never cast to the pointer first and `.add()`, which scales by the pointee size. An
  array field used as a pointer (`p->szKey + n`) is `(*$p).szKey.as_ptr().add(n)`; `*array` is
  `array[0]`. Narrow operands (WORD/USHORT) are widened with `as` before mixing with ULONG_PTR.
- A macro returning a POINTER from CONTAINING_RECORD is dereferenced `(*CONTAINING_RECORD!(..)).f`.
- A macro that is used as a type, or as a value with a cast to a pointer type, keeps that cast.
"""


_MACRO_RULES = re.compile(r"macro_rules!\s+([A-Za-z_]\w*)\s*\{")
_ITEM = re.compile(
    r"macro_rules!\s+([A-Za-z_]\w*)\s*\{|(?:pub\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_]\w*)\b"
)


def parse_reply(text: str, want: set[str]) -> dict[str, str]:
    """{name: macro_rules text}. Raises ValueError unless it is exactly the requested set."""
    # batch_runner already strips the fence; a raw reply still carries it.
    m = re.search(r"```rust\s*\n(.*?)```", text, re.S)
    code = m.group(1) if m else text
    out: dict[str, str] = {}
    pos = 0
    for mm in _ITEM.finditer(code):
        if mm.start() < pos:
            continue
        gap = code[pos : mm.start()].strip()
        if gap and not re.fullmatch(r"(?:#\[[^\]]*\]\s*)+(?:pub\s+)?(?:unsafe\s+)?", gap):
            raise ValueError(f"non-macro text in the reply: {gap[:80]!r}")
        depth, i = 0, code.index("{", mm.end() - 1)
        while i < len(code):
            if code[i] == "{":
                depth += 1
            elif code[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        else:
            raise ValueError(f"unbalanced braces in item {mm.group(1) or mm.group(2)}")
        name = mm.group(1) or mm.group(2)
        if name in out:
            raise ValueError(f"item {name} defined twice")
        out[name] = code[pos : i + 1].strip()  # with its attributes, which the gap held
        pos = i + 1
    tail = code[pos:].strip()
    if tail:
        raise ValueError(f"non-macro text in the reply: {tail[:80]!r}")
    if set(out) != want:
        missing, extra = sorted(want - set(out)), sorted(set(out) - want)
        raise ValueError(f"reply does not match the request: missing {missing}, extra {extra}")
    return out


def key(macros: dict[str, CMacro], thinking: str = "on") -> str:
    blob = json.dumps(
        [
            PROMPT_VERSION,
            thinking,
            sorted((m.name, m.params, m.body, m.kind) for m in macros.values()),
        ]
    )
    return hashlib.sha256(blob.encode()).hexdigest()[:16]


def legacy_key(macros: dict[str, CMacro], thinking: str = "on") -> str:
    """`key` as spelled before header functions joined (no `kind`), for macro-only sets."""
    blob = json.dumps(
        [PROMPT_VERSION, thinking, sorted((m.name, m.params, m.body) for m in macros.values())]
    )
    return hashlib.sha256(blob.encode()).hexdigest()[:16]


def render(ported: dict[str, str], macros: dict[str, CMacro]) -> str:
    out = [BEGIN]
    for name in sorted(ported):
        m = macros[name]
        if m.kind == "fn":
            out.append(f"// {m.where}: header-inline `{name}`")
            out.append("#[allow(dead_code)]")
            out.append(ported[name])
            continue
        out.append(f"// {m.where}: {m.c_text()}")
        out.append("#[allow(unused_macros)]")
        out.append(ported[name])
        out.append(f"#[allow(unused_imports)]\npub(crate) use {name};")
    out.append(END)
    return "\n".join(out) + "\n"


def in_scope_macros(prelude: Path) -> set[str]:
    """Rust macros the prelude already provides OUTSIDE the generated block."""
    text = prelude.read_text()
    text = re.sub(re.escape(BEGIN) + r".*?" + re.escape(END), "", text, flags=re.S)
    names = set(_MACRO_RULES.findall(text))
    for m in re.finditer(r"pub use win32macros_rust::\{([^}]*)\}", text):
        names |= {n.strip() for n in m.group(1).split(",") if n.strip()}
    return names


def existing(prelude: Path) -> set[str]:
    m = re.search(re.escape(BEGIN) + r"(.*?)" + re.escape(END), prelude.read_text(), re.S)
    if not m:
        return set()
    return {a or b for a, b in _ITEM.findall(m.group(1))}


def write_block(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    m = re.search(re.escape(BEGIN) + r".*?" + re.escape(END) + r"\n?", text, re.S)
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True
