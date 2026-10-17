#!/usr/bin/env python3
"""Resolve every CMake `if()` that tests for the MSVC compiler, which this project never uses.

WHY (owner, 2026-09-26/10-03): MSVC is not a supported toolchain -- every arch builds with
clang (llvm-mingw), where `MSVC` is unset and `CMAKE_C_COMPILER_ID` is "Clang". Upstream
ReactOS still carries an MSVC build, so its CMake is full of branches nothing here can
take. Roadmap section 11 retires them.

Hand-editing ~200 sites in ~140 files is the wrong tool, and it would not stay done: every
upstream merge brings more. So this partially evaluates each `if`/`elseif` condition with
the MSVC facts fixed, and rewrites only what that decides:

    if(MSVC) A else() B endif()            ->  B (dedented)
    if(NOT MSVC) A endif()                 ->  A (dedented)
    if(MSVC AND X) A elseif(Y) B endif()   ->  if(Y) B endif()
    if(MSVC OR X) A endif()                ->  if(X) A endif()

A condition with no MSVC atom in it is never touched, and neither is any atom the facts
do not decide. Run it after an upstream merge:

    python3 tools/strip_msvc_cmake.py --write      # rewrite in place
    ./vibeos-cli check                            # Rust lint reports remaining conditions

Vendored trees (vendor/) are skipped: their CMake belongs to the tree, and editing it makes
the next re-vendor a merge. Generator expressions ($<C_COMPILER_ID:MSVC>) are left alone.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SKIP_PREFIXES = ("vendor/",)

# The facts. A bare variable atom in this set is FALSE; so is a compiler-id test for MSVC.
FALSE_VARS = {"MSVC", "MSVC_IDE"}
_ID_VARS = {"CMAKE_C_COMPILER_ID", "CMAKE_CXX_COMPILER_ID"}
_BINARY = {
    "STREQUAL", "STRLESS", "STRGREATER", "STRLESS_EQUAL", "STRGREATER_EQUAL", "EQUAL",
    "LESS", "GREATER", "LESS_EQUAL", "GREATER_EQUAL", "MATCHES", "VERSION_LESS",
    "VERSION_GREATER", "VERSION_EQUAL", "VERSION_LESS_EQUAL", "VERSION_GREATER_EQUAL",
    "IN_LIST", "IS_NEWER_THAN", "PATH_EQUAL",
}  # fmt: skip
_UNARY = {"EXISTS", "COMMAND", "DEFINED", "TARGET", "POLICY", "TEST", "IS_DIRECTORY",
          "IS_SYMLINK", "IS_ABSOLUTE", "IS_READABLE", "IS_WRITABLE", "IS_EXECUTABLE"}  # fmt: skip


# ── condition expressions ───────────────────────────────────────────────────


@dataclass
class Node:
    kind: str  # "or" | "and" | "not" | "paren" | "atom" | "const"
    kids: list[Node] = field(default_factory=list)
    text: str = ""  # atom source text
    value: bool | None = None  # for "const"


_TOKEN = re.compile(r'\s*("(?:[^"\\]|\\.)*"|\(|\)|[^\s()"]+)', re.S)


def _tokens(s: str) -> list[str]:
    out, pos = [], 0
    s = s.strip()
    while pos < len(s):
        m = _TOKEN.match(s, pos)
        if not m:
            raise ValueError(f"cannot tokenize condition at {s[pos:]!r}")
        out.append(m.group(1))
        pos = m.end()
    return out


class _Parser:
    def __init__(self, toks: list[str]):
        self.t, self.i = toks, 0

    def peek(self) -> str | None:
        return self.t[self.i] if self.i < len(self.t) else None

    def take(self) -> str:
        tok = self.t[self.i]
        self.i += 1
        return tok

    def parse(self) -> Node:
        n = self.or_()
        if self.peek() is not None:
            raise ValueError(f"trailing tokens: {self.t[self.i :]}")
        return n

    def or_(self) -> Node:
        kids = [self.and_()]
        while self.peek() == "OR":
            self.take()
            kids.append(self.and_())
        return kids[0] if len(kids) == 1 else Node("or", kids)

    def and_(self) -> Node:
        kids = [self.not_()]
        while self.peek() == "AND":
            self.take()
            kids.append(self.not_())
        return kids[0] if len(kids) == 1 else Node("and", kids)

    def not_(self) -> Node:
        if self.peek() == "NOT":
            self.take()
            return Node("not", [self.not_()])
        return self.primary()

    def primary(self) -> Node:
        tok = self.peek()
        if tok is None:
            raise ValueError("unexpected end of condition")
        if tok == "(":
            self.take()
            inner = self.or_()
            if self.take() != ")":
                raise ValueError("unbalanced parenthesis")
            return Node("paren", [inner])
        if tok in _UNARY:
            self.take()
            return Node("atom", text=f"{tok} {self.take()}")
        left = self.take()
        if self.peek() in _BINARY:
            op = self.take()
            return Node("atom", text=f"{left} {op} {self.take()}")
        return Node("atom", text=left)


def _atom_value(text: str) -> bool | None:
    parts = text.split()
    if len(parts) == 1 and parts[0] in FALSE_VARS:
        return False
    if (
        len(parts) == 3
        and parts[0] in _ID_VARS
        and parts[1] == "STREQUAL"
        and parts[2].strip('"') == "MSVC"
    ):
        return False
    return None


def _simplify(n: Node) -> tuple[Node, bool]:
    """(simplified node, changed?)"""
    if n.kind == "atom":
        v = _atom_value(n.text)
        return (Node("const", value=v), True) if v is not None else (n, False)
    if n.kind == "paren":
        inner, ch = _simplify(n.kids[0])
        if inner.kind == "const":
            return inner, True
        return Node("paren", [inner]), ch
    if n.kind == "not":
        inner, ch = _simplify(n.kids[0])
        if inner.kind == "const":
            return Node("const", value=not inner.value), True
        return Node("not", [inner]), ch
    absorbing = n.kind == "or"  # OR: a TRUE decides; AND: a FALSE decides
    kids, changed = [], False
    for k in n.kids:
        s, ch = _simplify(k)
        changed |= ch
        if s.kind == "const":
            changed = True
            if s.value == absorbing:
                return Node("const", value=absorbing), True
            continue  # the neutral element drops out
        kids.append(s)
    if not kids:
        return Node("const", value=not absorbing), True
    if len(kids) == 1:
        return kids[0], True
    return Node(n.kind, kids), changed


def _render(n: Node) -> str:
    if n.kind == "atom":
        return n.text
    if n.kind == "paren":
        return f"({_render(n.kids[0])})"
    if n.kind == "not":
        return f"NOT {_render(n.kids[0])}"
    if n.kind == "const":
        return "TRUE" if n.value else "FALSE"
    return f" {n.kind.upper()} ".join(_render(k) for k in n.kids)


def evaluate(cond: str) -> tuple[bool | None, str | None]:
    """(constant value or None, rewritten condition text or None if unchanged)."""
    node, changed = _simplify(_Parser(_tokens(cond)).parse())
    if node.kind == "const":
        return node.value, None
    if node.kind == "paren":  # `if((X))` reads badly once its sibling is gone
        node = node.kids[0]
    return None, (_render(node) if changed else None)


def mentions_facts(cond: str) -> bool:
    return any(_atom_value(a) is not None for a in _atoms(cond))


def _atoms(cond: str) -> list[str]:
    out: list[str] = []

    def walk(n: Node) -> None:
        if n.kind == "atom":
            out.append(n.text)
        for k in n.kids:
            walk(k)

    try:
        walk(_Parser(_tokens(cond)).parse())
    except (ValueError, IndexError):
        pass
    return out


# ── CMake command scanning ──────────────────────────────────────────────────


@dataclass
class Cmd:
    name: str  # lower-case
    args: str
    line_start: int  # offset of the start of the line the command begins on
    end: int  # offset just past the command's line (incl. trailing comment + newline)
    indent: str


_CMD_START = re.compile(r"^([ \t]*)([A-Za-z_][A-Za-z0-9_]*)[ \t]*\(", re.M)
_FLOW = {"if", "elseif", "else", "endif"}


def _scan_args(text: str, i: int) -> int:
    """`i` is just past the opening paren; return the offset just past the matching one."""
    depth = 1
    while i < len(text):
        c = text[i]
        if c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif c == "#":
            nl = text.find("\n", i)
            i = len(text) if nl < 0 else nl
            continue
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError("unterminated command")


def commands(text: str) -> list[Cmd]:
    out, pos = [], 0
    while True:
        m = _CMD_START.search(text, pos)
        if not m:
            return out
        close = _scan_args(text, m.end())
        nl = text.find("\n", close)
        end = len(text) if nl < 0 else nl + 1
        out.append(Cmd(m.group(2).lower(), text[m.end() : close - 1], m.start(), end, m.group(1)))
        pos = end


@dataclass
class Branch:
    cmd: Cmd
    body_start: int
    body_end: int = 0


@dataclass
class Block:
    branches: list[Branch]
    endif: Cmd


def blocks(cmds: list[Cmd]) -> list[Block]:
    """Every if-block, outermost first in source order."""
    stack: list[list[Branch]] = []
    found: list[tuple[int, Block]] = []
    for c in cmds:
        if c.name not in _FLOW:
            continue
        if c.name == "if":
            stack.append([Branch(c, c.end)])
        elif not stack:
            raise ValueError(f"{c.name}() with no open if() at offset {c.line_start}")
        elif c.name in ("elseif", "else"):
            stack[-1][-1].body_end = c.line_start
            stack[-1].append(Branch(c, c.end))
        else:
            br = stack.pop()
            br[-1].body_end = c.line_start
            found.append((len(stack), Block(br, c)))
    if stack:
        raise ValueError("if() without endif()")
    found.sort(key=lambda d: (d[0], d[1].branches[0].cmd.line_start))
    return [b for _, b in found]


# ── rewriting ───────────────────────────────────────────────────────────────


def _dedent(body: str, to: str) -> str:
    lines = body.splitlines(keepends=True)
    widths = [len(ln) - len(ln.lstrip(" \t")) for ln in lines if ln.strip()]
    if not widths:
        return ""
    cut = min(widths) - len(to)
    if cut <= 0:
        return body
    return "".join(ln[cut:] if ln[:cut].strip() == "" else ln for ln in lines)


def _rewrite_block(text: str, b: Block) -> str | None:
    """The replacement for this block's whole span, or None if the facts decide nothing."""
    kept: list[tuple[Branch, str | None]] = []  # (branch, new condition or None=else)
    decided = False
    for br in b.branches:
        if br.cmd.name == "else":
            kept.append((br, None))
            break
        value, rewritten = evaluate(br.cmd.args)
        if value is False:
            decided = True
            continue
        if value is True:
            decided = True
            kept.append((br, None))
            break
        if rewritten is not None:
            decided = True
        kept.append((br, rewritten if rewritten is not None else br.cmd.args.strip()))
    if not decided:
        return None
    indent = b.branches[0].cmd.indent
    if not kept:
        return ""
    first, cond = kept[0]
    if cond is None:  # an unconditional arm wins outright
        return _dedent(text[first.body_start : first.body_end], indent)
    out = []
    for n, (br, cond) in enumerate(kept):
        if cond is None:
            out.append(f"{indent}else()\n")
        else:
            kw = "if" if n == 0 else "elseif"
            same = br.cmd.name == kw and cond == br.cmd.args.strip()
            out.append(text[br.cmd.line_start : br.cmd.end] if same else f"{indent}{kw}({cond})\n")
        out.append(text[br.body_start : br.body_end])
    out.append(text[b.endif.line_start : b.endif.end])
    return "".join(out)


_COMMENT_LINE = re.compile(r"^[ \t]*#.*\n", re.M)
_TALKS_MSVC = re.compile(r"MSVC|msvc|clang-cl|\bVS\b|Visual Studio|\bcl\b|msbuild|PDB", re.I)


def _swallow_attached_comment(text: str, start: int, *, deleted: bool) -> int:
    """The comment lines directly above a block (no blank line between) describe it. When
    the whole block is deleted they describe nothing, so they go too. When the block's body
    survives, they go only if they are about the MSVC side ("clang-cl defines this one for
    itself" above a now-unconditional define)."""
    lines = text[:start].splitlines(keepends=True)
    k = len(lines)
    while k and _COMMENT_LINE.fullmatch(lines[k - 1]):
        k -= 1
    if k == len(lines) or not (deleted or _TALKS_MSVC.search("".join(lines[k:]))):
        return start
    return sum(len(ln) for ln in lines[:k])


def rewrite(text: str) -> str:
    """Resolve the facts everywhere, one block at a time until nothing changes."""
    for _ in range(10_000):
        for b in blocks(commands(text)):
            new = _rewrite_block(text, b)
            if new is None:
                continue
            start, end = b.branches[0].cmd.line_start, b.endif.end
            start = _swallow_attached_comment(text, start, deleted=new == "")
            if new == "":
                before, after = text[:start], text[end:]
                # Don't leave a double blank line where the block was.
                if before.endswith("\n\n") and after.startswith("\n"):
                    after = after[1:]
                if not after.strip():  # nor a trailing one at the end of the file
                    before, after = before.rstrip("\n") + "\n" if before.strip() else "", ""
                text = before + after
            else:
                text = text[:start] + new + text[end:]
            break
        else:
            return text
    raise RuntimeError("no fixpoint")


def pending(text: str) -> int:
    """How many if/elseif conditions the facts would still decide."""
    return sum(1 for c in commands(text) if c.name in ("if", "elseif") and mentions_facts(c.args))


def cmake_files(paths: list[str]) -> list[Path]:
    if paths:
        return [Path(p).resolve() for p in paths]
    out = subprocess.run(
        ["git", "ls-files", "*.cmake", "*CMakeLists.txt"],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    return [REPO / p for p in out if not p.startswith(SKIP_PREFIXES)]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--write", action="store_true", required=True, help="rewrite files in place")
    ap.add_argument("paths", nargs="*", help="files (default: every tracked CMake file)")
    a = ap.parse_args(argv)
    left = changed = 0
    for f in cmake_files(a.paths):
        try:
            text = f.read_text()
        except (OSError, UnicodeDecodeError):
            continue
        if "MSVC" not in text:
            continue
        rel = f.relative_to(REPO) if f.is_relative_to(REPO) else f
        try:
            n = pending(text)
            if not n:
                continue
            if a.write:
                new = rewrite(text)
                if new != text:
                    f.write_text(new)
                    changed += 1
                rest = pending(new)
                if rest:
                    left += rest
                    print(f"{rel}: {rest} site(s) could not be resolved", file=sys.stderr)
            else:
                left += n
                print(f"{rel}: {n} MSVC condition(s)")
        except ValueError as e:
            left += 1
            print(f"{rel}: cannot parse ({e})", file=sys.stderr)
    if a.write:
        print(f"[strip_msvc_cmake] rewrote {changed} file(s); {left} site(s) left")
    elif left:
        print(
            f"[strip_msvc_cmake] {left} MSVC condition(s) left -- MSVC is not a supported "
            "toolchain; resolve them with: python3 tools/strip_msvc_cmake.py --write"
        )
    return 1 if left else 0


if __name__ == "__main__":
    sys.exit(main())
