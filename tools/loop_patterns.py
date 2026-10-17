"""Pure loop checks used to audit candidate Rust translations before integration."""

from __future__ import annotations

import functools
import re

COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
STRING_RE = re.compile(r'b?"(?:\\.|[^"\\])*"')
CHAR_RE = re.compile(r"b?'(?:\\.|[^'\\])'")
_LEXEME_RE = re.compile("|".join(p.pattern for p in (COMMENT_RE, CHAR_RE, STRING_RE)), re.S)
LOOP_RE = re.compile(r"(?:'(\w+)\s*:\s*)?\bloop\s*\{")
NESTED_LOOP_RE = re.compile(r"\b(?:loop|while|for)\b")
TRAILING_BREAK_IF_RE = re.compile(r"\bif\b[^{};]*\{\s*break\s*;\s*\}\s*$")
FN_RE = re.compile(r"\bfn\s+(\w+)")
NOLINT = "nolint: dowhile-continue"


def _blank(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


@functools.lru_cache(maxsize=2)
def _code_only(raw: str) -> str:
    """`raw` with comments, strings and char literals blanked (offsets preserved). Each
    scan below wants it, so a file is blanked once."""
    # ONE left-to-right pass: blanking strings before chars read the `"` in `'"'` as a string
    # opening, which swallowed msiexec's `}` and handed a truncated loop body to the scans.
    return _LEXEME_RE.sub(_blank, raw)


def _match_brace(text: str, open_idx: int) -> int:
    depth = 0
    for i in range(open_idx, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def scan_text(raw: str) -> list[tuple[int, str, int]]:
    """(line, enclosing fn, number of skipping continues) per suspicious loop."""
    text = _code_only(raw)
    raw_lines = raw.splitlines()
    out = []
    for m in LOOP_RE.finditer(text):
        label = m.group(1)
        open_idx = text.index("{", m.start())
        close_idx = _match_brace(text, open_idx)
        if close_idx < 0:
            continue
        body = text[open_idx + 1 : close_idx]
        if not TRAILING_BREAK_IF_RE.search(body.rstrip()):
            continue
        # Mask nested loop bodies: an unlabelled `continue` there belongs to them.
        masked = list(body)
        for n in NESTED_LOOP_RE.finditer(body):
            k = body.find("{", n.end())
            if k < 0:
                continue
            e = _match_brace(body, k)
            if e < 0:
                continue
            for q in range(k, e + 1):
                masked[q] = " "
        own = re.findall(r"\bcontinue\b(?:\s*'(\w+))?\s*;", "".join(masked))
        count = sum(1 for lab in own if lab in ("", label))
        if label:
            # A labelled continue from inside a nested loop still targets this one.
            count += len(re.findall(r"\bcontinue\s*'" + re.escape(label) + r"\s*;", body)) - sum(
                1 for lab in own if lab == label
            )
        if not count:
            continue
        line = text.count("\n", 0, m.start()) + 1
        if NOLINT in raw_lines[line - 1]:
            continue
        fns = FN_RE.findall(text, 0, m.start())
        out.append((line, fns[-1] if fns else "?", count))
    return out


WHILE_RE = re.compile(r"(?:'(\w+)\s*:\s*)?\bwhile\b[^{;]*\{")
TRAILING_STEP_RE = re.compile(r"(?:^|[;{}])\s*([\w.*()\[\]]+?)\s*(?:=|\+=|-=)\s*[^;=][^;]*;\s*$")


def _own_continues(body: str, label: str | None) -> list[int]:
    """Offsets of the `continue`s in `body` aimed at the loop that owns it."""
    masked = list(body)
    for n in NESTED_LOOP_RE.finditer(body):
        k = body.find("{", n.end())
        if k < 0:
            continue
        e = _match_brace(body, k)
        if e < 0:
            continue
        for q in range(k, e + 1):
            masked[q] = " "
    flat = "".join(masked)
    out = [
        cm.start()
        for cm in re.finditer(r"\bcontinue\b(?:\s*'(\w+))?\s*;", flat)
        if (cm.group(1) or "") in ("", label or "")
    ]
    if label:
        out += [
            cm.start()
            for cm in re.finditer(r"\bcontinue\s*'" + re.escape(label) + r"\s*;", body)
            if flat[cm.start() : cm.start() + 8] != "continue"
        ]
    return sorted(out)


def _block_ends(body: str, positions: list[int]) -> dict[int, int]:
    """{pos: end offset of the innermost block of `body` holding pos} in one pass
    (len(body) for the top level)."""
    want = sorted(set(positions))
    opened: dict[int, list[int]] = {}
    stack: list[int] = []
    ends: dict[int, int] = {}
    w = 0
    for i, ch in enumerate(body):
        while w < len(want) and want[w] == i:
            opened.setdefault(stack[-1] if stack else -1, []).append(i)
            w += 1
        if ch == "{":
            stack.append(i)
        elif ch == "}" and stack:
            for q in opened.pop(stack.pop(), []):
                ends[q] = i
    for lst in opened.values():
        for q in lst:
            ends.setdefault(q, len(body))
    return ends


def scan_while_steps(raw: str) -> list[tuple[int, str, str, int]]:
    """(line, enclosing fn, step variable, skipping continues) per `while` whose
    trailing update a `continue` jumps past."""
    text = _code_only(raw)
    raw_lines = raw.splitlines()
    out = []
    for m in WHILE_RE.finditer(text):
        label = m.group(1)
        open_idx = m.end() - 1
        close_idx = _match_brace(text, open_idx)
        if close_idx < 0:
            continue
        body = text[open_idx + 1 : close_idx]
        sm = TRAILING_STEP_RE.search(body.rstrip())
        if not sm or sm.group(1) == "_":
            continue
        step_at = sm.start(1)
        assign = re.compile(r"(?<![\w.])" + re.escape(sm.group(1)) + r"\s*(?:=(?!=)|\+=|-=)")
        updates = [a.start() for a in assign.finditer(body) if a.start() < step_at]
        conts = _own_continues(body, label)
        if not conts:
            continue
        ends = _block_ends(body, updates)
        skipping = sum(1 for c in conts if not any(u < c < ends[u] for u in updates))
        if not skipping:
            continue
        line = text.count("\n", 0, m.start()) + 1
        if NOLINT in raw_lines[line - 1]:
            continue
        fns = FN_RE.findall(text, 0, m.start())
        out.append((line, fns[-1] if fns else "?", sm.group(1), skipping))
    return out


MACRO_RE = re.compile(r"\bmacro_rules!\s*(\w+)\s*\{")
MACRO_LOOP_RE = re.compile(r"\b(?:while\b[^{;]*|loop\s*)\{")
SPLICE_RE = re.compile(r"(?m)^\s*\$(\w+)\s*;?\s*$")


def scan_macros(raw: str) -> list[tuple[int, str]]:
    """(line, macro name) for each loop macro that advances AFTER splicing `$body`."""
    text = _code_only(raw)
    out = []
    for m in MACRO_RE.finditer(text):
        end = _match_brace(text, m.end() - 1)
        if end < 0:
            continue
        macro = text[m.end() : end]
        for lm in MACRO_LOOP_RE.finditer(macro):
            o = lm.end() - 1
            c = _match_brace(macro, o)
            if c < 0:
                continue
            body = macro[o + 1 : c]
            # Only the loop's OWN statements: blank nested blocks (an `if` holding the
            # advance at the top is the fix, and must not read as "after the body").
            flat = list(body)
            i = 0
            while i < len(body):
                if body[i] == "{":
                    e = _match_brace(body, i)
                    if e < 0:
                        break
                    for q in range(i + 1, e):
                        flat[q] = " "
                    i = e + 1
                else:
                    i += 1
            flat_s = "".join(flat)
            splice = None
            for sm in SPLICE_RE.finditer(flat_s):
                splice = sm
            if splice and flat_s[splice.end() :].strip():
                out.append((text.count("\n", 0, m.start()) + 1, m.group(1)))
                break
    return out

