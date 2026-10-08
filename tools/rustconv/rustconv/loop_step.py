"""A C `for` lowered to a Rust `while` whose `continue` skips the step.

In C, `continue` inside `for (i = 0; i < n; ++i)` still runs `++i`. Lowered to

    while i < n { ...; if x { continue; } ...; i = i.wrapping_add(1); }

every `continue` must repeat the step first, and a translator that forgets one turns the
skipped iteration into an infinite loop (atapi's AtaFdoEnumeratePort retried a failed PDO
allocation forever). The loop's step is its last top-level statement; each `continue` that
belongs to the loop (not to a nested one) must be directly preceded by that same statement.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

_WHILE = re.compile(r"\bwhile\b")
_STEP = re.compile(r"^(\w+)\s*(?:=\s*\1\s*\.\s*wrapping_(?:add|sub)\s*\(.*\)|[+-]=\s*.+)$", re.S)
_LOOP_KW = re.compile(r"\b(?:while|loop|for)\b")


@dataclass(frozen=True)
class Skip:
    line: int
    var: str
    step: str


def _strip(rust: str) -> str:
    """Blank comments and string/char literals, keeping offsets."""
    out = list(rust)
    i, n = 0, len(rust)
    while i < n:
        c = rust[i]
        if rust.startswith("//", i):
            j = rust.find("\n", i)
            j = n if j < 0 else j
        elif rust.startswith("/*", i):
            j = rust.find("*/", i + 2)
            j = n if j < 0 else j + 2
        elif c == '"' or (c == "b" and i + 1 < n and rust[i + 1] == '"'):
            j = i + (2 if c == "b" else 1)
            while j < n and rust[j] != '"':
                j += 2 if rust[j] == "\\" else 1
            j += 1
        else:
            i += 1
            continue
        for k in range(i, min(j, n)):
            if out[k] != "\n":
                out[k] = " "
        i = j
    return "".join(out)


def _match_brace(s: str, open_at: int) -> int:
    depth = 0
    for k in range(open_at, len(s)):
        if s[k] == "{":
            depth += 1
        elif s[k] == "}":
            depth -= 1
            if depth == 0:
                return k
    return -1


def _top_statements(body: str) -> list[str]:
    """Top-level `;`-terminated statements of a block body (braced items count as one)."""
    stmts, depth, start = [], 0, 0
    for k, c in enumerate(body):
        if c in "{([":
            depth += 1
        elif c in "})]":
            depth -= 1
            if depth == 0 and c == "}":
                stmts.append(body[start : k + 1].strip())
                start = k + 1
        elif c == ";" and depth == 0:
            stmts.append(body[start:k].strip())
            start = k + 1
    tail = body[start:].strip()
    if tail:
        stmts.append(tail)
    return [s for s in stmts if s]


def _norm(s: str) -> str:
    return re.sub(r"\s+", "", s)


def scan(rust: str) -> list[Skip]:
    s = _strip(rust)
    found: list[Skip] = []
    for m in _WHILE.finditer(s):
        open_at = s.find("{", m.end())
        if open_at < 0:
            continue
        close = _match_brace(s, open_at)
        if close < 0:
            continue
        body = s[open_at + 1 : close]
        stmts = _top_statements(body)
        if not stmts:
            continue
        step = _STEP.match(stmts[-1])
        # A trailing `found += 1` is a counter, not the step; the step drives the condition.
        if not step or not re.search(rf"\b{re.escape(step.group(1))}\b", s[m.end() : open_at]):
            continue
        want = _norm(stmts[-1])
        # Walk the body; a `continue` at loop-nesting 0 (relative to this loop) is ours.
        k, nest = 0, []
        depth = 0
        while k < len(body):
            c = body[k]
            if c == "{":
                head = body[
                    max(
                        0,
                        body.rfind(";", 0, k) + 1,
                        body.rfind("}", 0, k) + 1,
                        body.rfind("{", 0, k) + 1,
                    ) : k
                ]
                nest.append(bool(_LOOP_KW.search(head)) and not head.strip().startswith("if"))
                depth += 1
            elif c == "}":
                if nest:
                    nest.pop()
                depth -= 1
            elif (
                body.startswith("continue", k)
                and not any(nest)
                and (k == 0 or not (body[k - 1].isalnum() or body[k - 1] == "_"))
            ):
                after = body[k + 8 : k + 9]
                labeled = (
                    body[k + 8 :].lstrip().startswith("'")
                )  # `continue 'outer` is another loop's
                if not (after.isalnum() or after == "_" or labeled):
                    before = body[:k].rstrip()
                    prev = before
                    # the statement right before: text between the two previous `;`/`{`/`}`
                    cut = max(prev.rfind(";"), prev.rfind("{"), prev.rfind("}"))
                    if cut >= 0 and prev[cut] == ";":
                        start = max(
                            prev.rfind(";", 0, cut),
                            prev.rfind("{", 0, cut),
                            prev.rfind("}", 0, cut),
                        )
                        last = prev[start + 1 : cut]
                    else:
                        last = ""
                    # Any write to the loop variable advances it (arm64 page.rs skips a 2MB block
                    # with `Va = (Va | MASK) + 1; continue;`) -- only a path with none is stuck.
                    # Narrowed to the shape that hangs: NO write to the variable anywhere between
                    # the loop's `{` and this `continue`. A hand-written loop that advances earlier
                    # on the path (`input += 2; ... continue;`) is the norm, not a defect.
                    write = rf"\b{re.escape(step.group(1))}\s*(?:[+\-*|&^]|<<|>>)?=[^=]"
                    advances = re.search(write, before)
                    if _norm(last) != want and not advances:
                        line = rust.count("\n", 0, open_at + 1 + k) + 1
                        found.append(Skip(line, step.group(1), stmts[-1].strip()))
            k += 1
    return found
