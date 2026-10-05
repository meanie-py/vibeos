"""Fill a new crate's prelude from what the rest of the tree already agrees on.

WHY THIS EXISTS
---------------
Every user-mode port meets the same unresolved names -- `_wcsicmp`, `UNICODE_NULL`,
`FILE_GENERIC_READ`, `FSCTL_LOCK_VOLUME`, `RtlStringCbCopyW` -- and every lane has
hand-declared them again in its own prelude: on 2026-10-05 `_wcsicmp` alone was declared
in advapi32, desk, dnsrslvr and more, `UNICODE_NULL` in a dozen crates. That hand work
is the "write the prelude" step a desk-sized port spent its afternoon on.

WHAT IT DOES
------------
Takes the names `cargo check` cannot find, finds every definition of each in the TRACKED
workspace (`git grep`, so the crate being built is never its own evidence), and copies
one into the prelude ONLY when every definition in the tree normalises to the same text:
a `pub const`, a `pub type`, or an extern `fn` declaration WITH its block's ABI (an
`extern "C"` copy of an `extern "system"` decl is an i386 link bug). Anything else is
reported, never guessed: definitions that disagree, names nothing defines, and names this
module's own C defines (a TU not translated yet -- an extern for it would shadow the real
body later). Re-run to a fixpoint: a copied constant can name another missing one.

The copies land between generated markers in src/prelude.rs, so a re-run replaces them.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

from .crate_integrate import extern_block_spans, split_decls

BEGIN = "// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ===="
END = "// ==== end port resolve ===="

_MISSING = re.compile(
    r"^cannot find (?:function|value|type|constant|function, tuple struct or tuple variant)"
    r"(?: or [a-z ]+)? `([A-Za-z_][A-Za-z0-9_]*)` in this scope"
)


@dataclass(frozen=True)
class Item:
    kind: str  # const | type | fn
    text: str  # the item as written, normalised
    abi: str | None
    path: str


@dataclass
class Resolution:
    copied: dict[str, Item] = field(default_factory=dict)
    conflicting: dict[str, str] = field(default_factory=dict)  # name -> why not
    undefined: list[str] = field(default_factory=list)
    pending: list[str] = field(default_factory=list)


def unresolved_names(crate: str, triple: str, workspace: Path, target_dir: Path) -> set[str]:
    proc = subprocess.run(
        ["cargo", "check", "-p", crate, "--target", triple, "--message-format=json"],
        cwd=workspace,
        env=dict(os.environ, CARGO_TARGET_DIR=str(target_dir)),
        capture_output=True,
        text=True,
    )
    names: set[str] = set()
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        m = _MISSING.match(str((msg.get("message") or {}).get("message") or ""))
        if m:
            names.add(m.group(1))
    return names


def _norm(text: str) -> str:
    text = re.sub(r"//[^\n]*", "", text)
    text = re.sub(r"#\[[^\]]*\]", "", text)
    text = re.sub(r"\bpub(\([^)]*\))?\s+", "", text)
    return " ".join(text.split())


_CONST = r"^[ \t]*pub[ \t]+const[ \t]+{n}[ \t]*:[^=;]+=[^;]+;"
_TYPE = r"^[ \t]*pub[ \t]+type[ \t]+{n}[ \t]*=[^;]+;"


def workspace_items(names: set[str], workspace: Path, exclude: Path) -> dict[str, list[Item]]:
    """Every tracked definition of each name: consts, type aliases, extern fn decls."""
    if not names:
        return {}
    pattern = "|".join(sorted(names))
    proc = subprocess.run(
        ["git", "grep", "-l", "-w", "-E", pattern, "--", "*.rs"],
        cwd=workspace,
        capture_output=True,
        text=True,
    )
    found: dict[str, list[Item]] = {n: [] for n in names}
    for rel in proc.stdout.split():
        path = workspace / rel
        if exclude in path.parents or "/target/" in rel:
            continue
        try:
            src = path.read_text(errors="replace")
        except OSError:
            continue
        present = {n for n in names if re.search(rf"\b{re.escape(n)}\b", src)}
        for n in present:
            for kind, pat in (("const", _CONST), ("type", _TYPE)):
                for m in re.finditer(pat.format(n=re.escape(n)), src, re.M):
                    found[n].append(Item(kind, _norm(m.group(0)), None, rel))
        for _start, _end, body, abi in extern_block_spans(src):
            for decl in split_decls(body):
                m = re.search(r"\bfn\s+([A-Za-z_]\w*)\s*\(", decl)
                if m and m.group(1) in present:
                    found[m.group(1)].append(Item("fn", _norm(decl), abi, rel))
    return found


_INT = re.compile(
    r"^\(?\s*(0[xX][0-9A-Fa-f_]+|[0-9][0-9_]*)(?:_?[iu](?:8|16|32|64|size))?\s*\)?"
    r"(?:\s+as\s+[A-Za-z_][\w:]*)?$"
)


def const_value(item: Item) -> int | None:
    """The integer a `const NAME: T = <literal>[ as T];` declares, else None."""
    m = re.search(r"=\s*(.+?)\s*;\s*$", item.text)
    if not m:
        return None
    lit = _INT.match(m.group(1))
    if not lit:
        return None
    digits = lit.group(1).replace("_", "")
    return int(digits, 16) if digits[:2] in ("0x", "0X") else int(digits)


def _abi_class(abi: str | None) -> str:
    a = (abi or '"C"').strip('"')
    return "system" if a in ("system", "stdcall") else a


def _placeholder_decl(item: Item) -> bool:
    """`fn NtClose(_: usize);` -- a link-only stub some crates keep. Not evidence of a signature."""
    params = re.search(r"\((.*)\)", item.text)
    return bool(params) and all(
        re.fullmatch(r"\s*_\s*:\s*usize\s*", p) for p in params.group(1).split(",") if p.strip()
    )


def consensus(name: str, items: list[Item]) -> tuple[Item | None, str]:
    """The definition the tree agrees on, or (None, why not).

    A constant needs every literal definition to agree on the VALUE (`0x80`, `0x0000_0080`
    and `128u32 as T` are one value); the most common declaration of it is copied. An extern
    fn needs every real declaration to agree on the ABI class -- `system`/`stdcall` vs `C` is
    an i386 link bug, not a spelling -- and the most common declaration is copied. Types are
    never copied: a missing type belongs in the bindgen island's allowlist.
    """
    consts = [i for i in items if i.kind == "const"]
    fns = [i for i in items if i.kind == "fn" and not _placeholder_decl(i)]
    if consts and not fns:
        literal = [i for i in consts if const_value(i) is not None]
        values = {const_value(i) for i in literal}
        if len(values) != 1:
            return None, (f"{len(values)} different values" if values else "no literal definition")
        # An expression definition (`CTL_CODE(...)`) is not evaluated here; it neither
        # vetoes nor is copied -- the literals that agree are the evidence.
        return _most_common(literal), ""
    if fns and not consts:
        classes = {_abi_class(i.abi) for i in fns}
        if len(classes) != 1:
            return None, f"ABI disagreement ({', '.join(sorted(classes))})"
        return _most_common(fns), ""
    if any(i.kind == "type" for i in items):
        return None, "a type: add its header to the island allowlist"
    return None, "mixed kinds"


def _most_common(items: list[Item]) -> Item:
    counts: dict[tuple[str, str | None], int] = {}
    for i in items:
        counts[(i.text, i.abi)] = counts.get((i.text, i.abi), 0) + 1
    best = max(counts.items(), key=lambda kv: (kv[1], -len(kv[0][0])))[0]
    return next(i for i in items if (i.text, i.abi) == best)


def render(copied: dict[str, Item]) -> str:
    consts = [i for i in copied.values() if i.kind in ("const", "type")]
    fns: dict[str, list[Item]] = {}
    for i in copied.values():
        if i.kind == "fn":
            fns.setdefault(i.abi or '"C"', []).append(i)
    out = [BEGIN]
    for i in sorted(consts, key=lambda i: i.text):
        out.append(f"pub {i.text} // {i.path}")
    for abi, items in sorted(fns.items()):
        out.append(f"extern {abi} {{")
        for i in sorted(items, key=lambda i: i.text):
            text = i.text if i.text.startswith("fn") else re.sub(r"^.*?\bfn\b", "fn", i.text)
            out.append(f"    pub {text} // {i.path}")
        out.append("}")
    out.append(END)
    return "\n".join(out) + "\n"


def write_block(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    m = re.search(re.escape(BEGIN) + r".*?" + re.escape(END) + r"\n?", text, re.S)
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True


def existing_block(prelude: Path) -> dict[str, Item]:
    """Items a previous run copied, so a re-run keeps them (cargo no longer reports them)."""
    text = prelude.read_text()
    m = re.search(re.escape(BEGIN) + r"(.*?)" + re.escape(END), text, re.S)
    if not m:
        return {}
    out: dict[str, Item] = {}
    abi = None
    for line in m.group(1).splitlines():
        s = line.strip()
        ab = re.match(r'^extern ("[^"]+") \{$', s)
        if ab:
            abi = ab.group(1)
            continue
        if s == "}":
            abi = None
            continue
        body, _, src = s.partition(" // ")
        body = re.sub(r"^pub\s+", "", body)
        n = re.search(r"\b(?:const|type|fn)\s+([A-Za-z_]\w*)", body)
        if n:
            kind = "fn" if body.startswith("fn") else body.split()[0]
            out[n.group(1)] = Item(kind, body, abi, src)
    return out
