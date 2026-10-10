"""Undo a translation that read a POINTER typedef as the struct it points to.

C names a pointer type with a typedef as often as with a star: `PATAPORT_PORT_DATA PortData`
is one pointer. The model, more than once in a module, read the `P` typedef as the struct
and wrote `PortData: *mut PATAPORT_PORT_DATA` -- a pointer to a pointer -- then dereferenced
it twice so the field accesses would compile: `(*(*PortData)).Worker`. That code BUILDS and
is wrong on every call: it reads the struct's first word as a pointer. atapi had 169 such
double dereferences and pciidex 125, all in the bank, none visible to rustc.

The C declaration is the authority, per name and per function: a parameter, local or return
the C spells `T name` (no star) where the Rust spells `*mut T`, and `T` is a pointer typedef
in the island, loses exactly one level -- the declaration, its `(*(*name))` dereferences,
and the `as *mut T` casts assigned to it. A name the C declares as `T *name` (an
out-parameter: `PDEVICE_OBJECT *Pdo`) is left alone, as is any name the C declares twice
with different depths.

The same misreading has a second, quieter form: C's `sizeof(*DevExt)` came back as
`size_of::<PATAPORT_DEVICE_EXTENSION>()` -- the size of a POINTER. atapi passed that to
IoCreateDevice as the PDO's extension size (8 bytes for a struct of hundreds), and set three
SCSI mode-page lengths from it. Inside a function whose C never writes `sizeof(PFOO)`, a
`size_of::<PFOO>()` is restored to the pointee.
"""

from __future__ import annotations

import re
from collections import Counter
from collections.abc import Iterable, Mapping
from dataclasses import dataclass

from . import goto_arm

#: `pub type PFOO = *mut _FOO;` in a bindgen island.
_PTR_TYPEDEF = re.compile(r"^pub type ([A-Za-z_]\w*) = \*(?:mut|const) ", re.M)


_PTR_TARGET = re.compile(r"^pub type ([A-Za-z_]\w*) = \*(?:mut|const) ([A-Za-z_]\w*);", re.M)


def pointer_targets(islands: Iterable[str]) -> dict[str, str]:
    """`PFOO -> _FOO` where every island agrees on the pointee."""
    maps = [dict(_PTR_TARGET.findall(text)) for text in islands]
    if not maps:
        return {}
    return {k: v for k, v in maps[0].items() if all(m.get(k) == v for m in maps[1:])}


def pointer_typedefs(islands: Iterable[str]) -> set[str]:
    """Names EVERY island defines as a pointer type (one that disagrees is not judged)."""
    sets = [set(_PTR_TYPEDEF.findall(text)) for text in islands]
    return set.intersection(*sets) if sets else set()


@dataclass(frozen=True)
class CFunctionDecls:
    """What one C function declares, for the typedefs that matter: name -> (type, stars)."""

    names: dict[str, tuple[str, int]]
    ret: tuple[str, int] | None
    #: pointer typedefs the C itself takes `sizeof` of (`sizeof(PVOID)` is meant).
    sizeof_types: frozenset[str] = frozenset()
    #: the C takes `sizeof` of a spelled-out pointer type (`sizeof(T *)`) somewhere in it.
    sizeof_raw_pointer: bool = False
    #: the C body uses `~` (bitwise not) anywhere; see logical_not.py.
    bitwise_not: bool = False
    #: the operands of the C's logical `!( .. )`, whitespace removed; see logical_not.py.
    logical_nots: frozenset[str] = frozenset()
    #: every identifier the C body spells (members included, also as a macro argument such as
    #: `CONTAINING_RECORD(.., PortEntry)`); see union_slot.py.
    identifiers: frozenset[str] = frozenset()
    #: how often the C WRITES each member (`.M = `, `->M |= `), as sorted `(name, count)` pairs.
    member_uses: tuple[tuple[str, int], ...] = ()
    #: `(case, label)` per `switch` arm that ends in an unconditional `goto`; see goto_arm.py.
    goto_arms: tuple[tuple[str, str], ...] = ()


def _split_commas(text: str) -> list[str]:
    parts, depth, cur = [], 0, []
    for ch in text:
        depth += ch in "([{"
        depth -= ch in ")]}"
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return parts


def c_declarations(
    name: str, params: Iterable[tuple[str, str]], body_text: str, types: set[str]
) -> CFunctionDecls:
    """Each parameter and local of `types` the C function declares, with its star count."""
    seen: dict[str, set[tuple[str, int]]] = {}

    def note(var: str, typ: str, stars: int) -> None:
        seen.setdefault(var, set()).add((typ, stars))

    for ptype, pname in params:
        words = re.findall(r"[A-Za-z_]\w*", ptype)
        hit = [w for w in words if w in types]
        if hit:
            # A C++ reference parameter (`PVOID& AddressInfo`) is one level of indirection in
            # Rust (`*mut PVOID`), exactly as a `*` is: counting only stars "repaired"
            # tracert's correct `*mut PVOID` back to `PVOID`.
            note(pname, hit[-1], ptype.count("*") + ptype.count("&"))
    open_brace = body_text.find("{")
    body = body_text[open_brace:] if open_brace != -1 else ""
    # Blank comments and string literals so neither can pose as a declaration.
    body = re.sub(r"/\*.*?\*/|//[^\n]*|\"(?:\\.|[^\"\\])*\"", " ", body, flags=re.S)
    alt = "|".join(sorted((re.escape(t) for t in types), key=len, reverse=True))
    if alt:
        # The anchor is a LOOKBEHIND: consuming the `;` hid every second of consecutive
        # declarations (pciidex's `PATA_CONTROLLER Controller` after `ChanData`).
        decl = re.compile(rf"(?:^|(?<=[;{{}}]))\s*(?:const\s+|volatile\s+)*({alt})\b([^;(){{}}]*);")
        for m in decl.finditer(body):
            for declarator in _split_commas(m.group(2)):
                d = declarator.split("=", 1)[0]
                v = re.fullmatch(r"\s*(\**)\s*(?:const\s+)?([A-Za-z_]\w*)\s*(?:\[[^\]]*\])?\s*", d)
                if v:
                    note(v.group(2), m.group(1), len(v.group(1)))
    names = {var: next(iter(s)) for var, s in seen.items() if len(s) == 1}
    ret = None
    head = body_text[: body_text.find(f"{name}(")] if f"{name}(" in body_text else ""
    words = re.findall(r"[A-Za-z_]\w*|\*", head)
    for i, w in enumerate(words):
        if w in types:
            ret = (w, words[i + 1 :].count("*"))
    sized = frozenset(
        t for t in re.findall(r"\bsizeof\s*\(\s*([A-Za-z_]\w*)\s*\)", body) if t in types
    )
    raw = re.search(r"\bsizeof\s*\(\s*(?:const\s+)?[A-Za-z_]\w*\s*\*", body) is not None
    # `sizeof(p)` / `sizeof p` of a pointer VARIABLE means a pointer size too.
    for v in re.findall(r"\bsizeof\s*(?:\(\s*([A-Za-z_]\w*)\s*\)|\s([A-Za-z_]\w*)\b)", body):
        typ, depth = names.get(v[0] or v[1], ("", 0))
        raw = raw or depth > 0 or typ in types
    code = re.sub(r'"(?:[^"\\]|\\.)*"', "", body)
    nots: set[str] = set()
    for m in re.finditer(r"!\s*\(", code):
        depth, i = 0, m.end() - 1
        for i in range(m.end() - 1, len(code)):
            depth += {"(": 1, ")": -1}.get(code[i], 0)
            if depth == 0:
                break
        nots.add(re.sub(r"\s+", "", code[m.end() : i]))
    idents = frozenset(re.findall(r"[A-Za-z_]\w*", code))
    uses = Counter(re.findall(r"(?:\.|->)\s*([A-Za-z_]\w*)\s*(?:[-+*/%&|^]|<<|>>)?=(?!=)", code))
    return CFunctionDecls(
        names,
        ret,
        sized,
        raw,
        "~" in code,
        frozenset(nots),
        idents,
        tuple(sorted(uses.items())),
        goto_arm.c_goto_arms(body),
    )


def _fn_spans(text: str) -> Iterable[tuple[str, int, int, int]]:
    """`(name, header start, body open brace, body end)` for each top-level `fn`."""
    for m in re.finditer(
        r"^(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?fn\s+([A-Za-z_]\w*)\s*\(",
        text,
        re.M,
    ):
        depth, i = 1, m.end()
        while i < len(text) and depth:
            depth += {"(": 1, ")": -1}.get(text[i], 0)
            i += 1
        brace = text.find("{", i)
        semi = text.find(";", i)
        if brace == -1 or (semi != -1 and semi < brace):
            continue
        depth, j = 1, brace + 1
        while j < len(text) and depth:
            depth += {"{": 1, "}": -1}.get(text[j], 0)
            j += 1
        yield m.group(1), m.start(), brace, j


def repair(
    rust: str,
    decls: Mapping[str, CFunctionDecls],
    types: set[str],
    targets: Mapping[str, str] | None = None,
) -> tuple[str, list[str]]:
    """Collapse the extra level for every name the C declares one level shallower."""
    fixes: list[str] = []
    out, last = [], 0
    for name, start, brace, end in _fn_spans(rust):
        c = decls.get(name)
        if c is None:
            continue
        fn = rust[start:end]
        header, body = fn[: brace - start], fn[brace - start :]
        wrong: dict[str, str] = {}  # var -> typedef
        for m in re.finditer(
            r"\b(?:mut\s+)?([A-Za-z_]\w*)\s*:\s*\*(?:mut|const)\s+([A-Za-z_]\w*)\b", fn
        ):
            var, typ = m.group(1), m.group(2)
            if typ in types and c.names.get(var) == (typ, 0):
                wrong[var] = typ
        ret_typ = None
        r = re.search(r"\)\s*->\s*\*(?:mut|const)\s+([A-Za-z_]\w*)\s*$", header.rstrip())
        if r and r.group(1) in types and c.ret == (r.group(1), 0):
            ret_typ = r.group(1)
        sized = sorted(
            {
                t
                for t in re.findall(r"\bsize_of::<\s*([A-Za-z_]\w*)\s*>\s*\(\s*\)", body)
                if targets
                and t in targets
                and t in types
                and t not in c.sizeof_types
                # An array OF the pointer type in the same body makes it an element type, so its
                # size is meant: atapi's RTL_NUMBER_OF over a `PCSTR[]` is
                # `size_of::<[PCSTR; 4]>() / size_of::<PCSTR>()`, and the macro hides the C's
                # sizeof from the guard above.
                and not re.search(rf"\[\s*{re.escape(t)}\s*;", body)
            }
        )
        # The raw-pointer spelling of the same misreading: acpi.c's `sizeof(*TimingMode)` came
        # back as `size_of::<*mut _IDE_ACPI_TIMING_MODE_BLOCK>()` and sized an ACPI output
        # buffer 8 bytes short of the struct it receives. Only where the C never takes the
        # size of ANY pointer type, so a deliberate pointer size is never touched.
        raw_sized = (
            sorted(
                set(re.findall(r"\bsize_of::<\s*\*(?:mut|const)\s+([A-Za-z_]\w*)\s*>", body))
                - {"c_void", "core", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "usize"}
                - set(re.findall(r"\[\s*\*(?:mut|const)\s+([A-Za-z_]\w*)\s*;", body))
            )
            if not c.sizeof_types and not c.sizeof_raw_pointer
            else []
        )
        if not wrong and ret_typ is None and not sized and not raw_sized:
            continue
        for typ in raw_sized:
            body = re.sub(
                rf"\bsize_of::<\s*\*(?:mut|const)\s+{re.escape(typ)}\s*>",
                f"size_of::<{typ}>",
                body,
            )
        for typ in sized:
            assert targets is not None  # `sized` is empty without them
            body = re.sub(
                rf"\bsize_of::<\s*{re.escape(typ)}\s*>", f"size_of::<{targets[typ]}>", body
            )
        for var, typ in wrong.items():
            decl = rf"(\b(?:mut\s+)?{re.escape(var)}\s*:\s*)\*(?:mut|const)\s+{re.escape(typ)}\b"
            header = re.sub(decl, rf"\g<1>{typ}", header)
            body = re.sub(decl, rf"\g<1>{typ}", body)
            body = re.sub(rf"\(\*\(\*{re.escape(var)}\)\)", f"(*{var})", body)
            # `v = expr as *mut T;` / `let v: T = expr as *mut T;`
            body = re.sub(
                rf"(\b{re.escape(var)}(?:\s*:\s*{re.escape(typ)})?\s*=\s*[^;]*?\bas\s+)"
                rf"\*(?:mut|const)\s+{re.escape(typ)}\b(\s*;)",
                rf"\g<1>{typ}\g<2>",
                body,
            )
        if ret_typ is not None:
            header = re.sub(
                rf"(\)\s*->\s*)\*(?:mut|const)\s+{re.escape(ret_typ)}\b", rf"\g<1>{ret_typ}", header
            )
        new = header + body
        if new != fn:
            out.append(rust[last:start] + new)
            last = end
            what = sorted(wrong) + (["-> return"] if ret_typ else [])
            what += [f"size_of::<{t}>" for t in sized]
            what += [f"size_of::<*mut {t}>" for t in raw_sized]
            fixes.append(f"{name}: {', '.join(what)}")
    out.append(rust[last:])
    return "".join(out), fixes
