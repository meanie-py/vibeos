"""extern_promote.py — promote per-module `extern` declarations into shared substrate.

WHY. A translated crate accumulates the same Win32 declaration in module after module,
because each translated TU declares what it happens to call. shell32-rust declares
``HeapFree`` four times and ``ExpandEnvironmentStringsW`` four times. That is not merely
untidy: every copy is an independent chance to get the ABI wrong, and Rust will not tell
you, because each module compiles against its own declaration. The first module whose copy
disagrees with the real export corrupts the stack at the call and nothing points back here.

So the copies want collapsing into one shared declaration — but only after proving they
AGREE, and the proof has to be semantic. Textually these are four different declarations:

    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, pszPath: *mut u16) -> i32;

and they are the same declaration: parameter NAMES are not ABI, ``LPWSTR`` resolves to
``*mut u16``, and ``BOOL`` resolves to ``i32``. Comparing the text would report a conflict
that is not there; comparing after alias resolution reports the ones that are.

WHAT IT REFUSES TO DO. When two modules genuinely disagree — a different arity, a different
resolved parameter type, a different ABI string — this does not pick one. Picking would
silently bless whichever module happened to sort first, and one of the two callers is
already making a wrong call today. Those come back as conflicts for a human to adjudicate
against the C header, which is the only authority that can settle it.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

from .lex_mask import comment_spans, mask_code

_EXTERN_BLOCK = re.compile(r'extern\s+"([A-Za-z-]+)"\s*\{')
_DECL = re.compile(r"(?:pub\s+)?fn\s+(\w+)\s*\(")
_ALIAS = re.compile(r"(?m)^\s*pub\s+type\s+(\w+)\s*=\s*([^;]+);")

#: Spellings that mean the same Rust type. Resolving aliases gets most of the way; these
#: are the remaining synonyms that no `pub type` in the tree relates.
_CANONICAL = {
    "c_void": "core::ffi::c_void",
    "::core::ffi::c_void": "core::ffi::c_void",
    "c_int": "i32",
    "c_uint": "u32",
    "c_char": "i8",
    "c_long": "i32",
    "c_ulong": "u32",
}


@dataclass
class Decl:
    name: str
    abi: str
    params: tuple[str, ...]
    ret: str
    text: str
    origin: Path


@dataclass
class Promotion:
    #: name -> the single agreed declaration, ready to emit
    agreed: dict[str, Decl] = field(default_factory=dict)
    #: name -> the distinct resolved forms that disagree, with where each came from
    conflicts: dict[str, list[tuple[str, list[Path]]]] = field(default_factory=dict)


def _match_brace(text: str, open_index: int) -> int | None:
    depth = 0
    for i in range(open_index, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return None


def _split_top_level(text: str) -> list[str]:
    parts, depth, cur = [], 0, []
    for ch in text:
        if ch in "<([{":
            depth += 1
        elif ch in ">)]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return [p.strip() for p in parts if p.strip()]


def alias_map(roots: list[Path]) -> dict[str, str]:
    """Every `pub type X = Y;` in the tree, transitively resolved to a base spelling."""
    raw: dict[str, str] = {}
    for root in roots:
        for path in sorted(root.rglob("*.rs")):
            for match in _ALIAS.finditer(mask_code(path.read_text(errors="replace"))):
                raw.setdefault(match.group(1), " ".join(match.group(2).split()))
    resolved: dict[str, str] = {}
    for name in raw:
        seen, cur = {name}, raw[name]
        # follow the alias chain; a cycle stops at the first repeat rather than hanging
        for _ in range(16):
            base = cur.lstrip("*").replace("const ", "").replace("mut ", "").strip()
            if base in raw and base not in seen:
                seen.add(base)
                cur = cur.replace(base, raw[base], 1)
                continue
            break
        resolved[name] = cur
    return resolved


def resolve_type(spelling: str, aliases: dict[str, str]) -> str:
    """Reduce a Rust type spelling to a comparable canonical form."""
    text = " ".join(spelling.split())
    for _ in range(16):
        before = text
        for token in sorted(set(re.findall(r"\b\w+\b", text)), key=len, reverse=True):
            if token in aliases:
                text = re.sub(rf"\b{re.escape(token)}\b", aliases[token], text)
        if text == before:
            break
    for token, canon in _CANONICAL.items():
        text = re.sub(rf"(?<![\w:]){re.escape(token)}\b", canon, text)
    return " ".join(text.replace("(", " ( ").replace(")", " ) ").split())


def collect(root: Path, skip: str | None = None) -> dict[str, list[Decl]]:
    """Every `extern "X" { fn ... ; }` declaration under ``root``."""
    found: dict[str, list[Decl]] = {}
    for path in sorted(root.rglob("*.rs")):
        if skip and skip in str(path):
            continue
        src = path.read_text(errors="replace")
        masked = mask_code(src)
        comment = comment_spans(src)
        # The ABI is a string LITERAL, whose content `mask_code` blanks — so the block has
        # to be found in the raw text (and filtered against comments), while brace matching
        # still runs on the masked copy so a brace inside a string cannot mis-nest.
        for block in _EXTERN_BLOCK.finditer(src):
            if comment[block.start()]:
                continue
            end = _match_brace(masked, block.end() - 1)
            if end is None:
                continue
            body, offset = masked[block.end() : end], block.end()
            for decl in _DECL.finditer(body):
                open_paren = offset + decl.end() - 1
                close = _match_brace(masked.replace("(", "{").replace(")", "}"), open_paren)
                if close is None:
                    continue
                params = src[open_paren + 1 : close]
                tail = src[close + 1 :]
                semi = tail.find(";")
                if semi < 0:
                    continue
                ret = tail[:semi].strip()
                ret = ret[2:].strip() if ret.startswith("->") else "()"
                found.setdefault(decl.group(1), []).append(
                    Decl(
                        name=decl.group(1),
                        abi=block.group(1),
                        params=tuple(_split_top_level(params)),
                        ret=ret,
                        text=" ".join(src[offset + decl.start() : close + 1 + semi + 1].split()),
                        origin=path,
                    )
                )
    return found


def _signature(decl: Decl, aliases: dict[str, str]) -> str:
    """ABI + parameter TYPES + return type. Parameter names are deliberately dropped."""
    types = []
    for param in decl.params:
        _, _, spelling = param.partition(":")
        types.append(resolve_type(spelling or param, aliases))
    return f'extern "{decl.abi}" ({", ".join(types)}) -> {resolve_type(decl.ret, aliases)}'


def promote(
    declarations: dict[str, list[Decl]], aliases: dict[str, str], wanted: set[str] | None = None
) -> Promotion:
    out = Promotion()
    for name, decls in sorted(declarations.items()):
        if wanted is not None and name not in wanted:
            continue
        by_sig: dict[str, list[Decl]] = {}
        for decl in decls:
            by_sig.setdefault(_signature(decl, aliases), []).append(decl)
        if len(by_sig) == 1:
            # Prefer the copy with the most descriptive parameter names.
            best = max(decls, key=lambda d: sum(len(p) for p in d.params))
            out.agreed[name] = best
        else:
            out.conflicts[name] = [
                (sig, sorted({d.origin for d in group})) for sig, group in sorted(by_sig.items())
            ]
    return out
