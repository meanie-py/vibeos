"""Move a `match` arm that carries C's `default:` to the end of its match.

C tries every `case` before `default`, wherever `default:` sits in the switch. A Rust match
tries arms in order, so `case 0: default:` translated as `0 | _ => ..` ahead of `1 => ..`
sends every value down the default branch: net's ParseDate and cmd's `date` / `dir` read
every locale's LOCALE_IDATE as month-first. A match arm never falls through, so moving the
wildcard arm last is exactly the C dispatch. rustc only warns (`unreachable pattern`).

An arm is the wildcard arm when one of its top-level `|` alternatives is a bare `_` and it has
no guard. Arms are found on a copy with comments and literals blanked, so a `{` in a string or
a `=>` in a comment cannot move a boundary.
"""

from __future__ import annotations

import re
from pathlib import Path

_LEX = re.compile(
    r"/\*.*?\*/|//[^\n]*|b?r#*\"(?:.|\n)*?\"#*|b?\"(?:\\.|[^\"\\])*\"|b?'(?:\\.|[^\\'\n])'",
    re.S,
)
_MATCH = re.compile(r"\bmatch\b")
_OPEN = "([{"
_CLOSE = ")]}"


def _mask(text: str) -> str:
    return _LEX.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def _close_of(masked: str, i: int) -> int:
    """Index of the bracket closing the one at `i`, or -1."""
    depth = 0
    for j in range(i, len(masked)):
        c = masked[j]
        if c in _OPEN:
            depth += 1
        elif c in _CLOSE:
            depth -= 1
            if depth == 0:
                return j
    return -1


def _top_level(masked: str, start: int, end: int, needle: str) -> int:
    depth = 0
    for j in range(start, end):
        if depth == 0 and masked.startswith(needle, j):
            return j
        c = masked[j]
        if c in _OPEN:
            depth += 1
        elif c in _CLOSE:
            depth -= 1
            if depth < 0:
                return -1
    return -1


def _arms(masked: str, body_start: int, body_end: int) -> list[tuple[int, int, int]] | None:
    """(segment start, `=>` index, segment end incl. trailing comma) for each arm."""
    out = []
    pos = body_start
    while True:
        while pos < body_end and masked[pos].isspace():
            pos += 1
        if pos >= body_end:
            return out
        arrow = _top_level(masked, pos, body_end, "=>")
        if arrow < 0:
            return None
        k = arrow + 2
        while k < body_end and masked[k].isspace():
            k += 1
        if k < body_end and masked[k] == "{":
            close = _close_of(masked, k)
            if close < 0 or close >= body_end:
                return None
            end = close + 1
            n = end
            while n < body_end and masked[n] in " \t":
                n += 1
            if n < body_end and masked[n] == ",":
                end = n + 1
        else:
            comma = _top_level(masked, k, body_end, ",")
            end = comma + 1 if comma >= 0 else len(masked[:body_end].rstrip())
        out.append((pos, arrow, end))
        pos = end


def _is_wildcard(pattern: str) -> bool:
    if re.search(r"\bif\b", pattern):
        return False
    depth, alts, cur = 0, [], []
    for c in pattern:
        if c in _OPEN:
            depth += 1
        elif c in _CLOSE:
            depth -= 1
        if c == "|" and depth == 0:
            alts.append("".join(cur))
            cur = []
        else:
            cur.append(c)
    alts.append("".join(cur))
    return any(a.strip() == "_" for a in alts)


def _leading_trivia(text: str, start: int) -> int:
    """Start of the comment/blank lines directly above an arm, so they move with it."""
    line_start = text.rfind("\n", 0, start) + 1
    s = line_start
    while s > 0:
        prev = text.rfind("\n", 0, s - 1) + 1
        line = text[prev : s - 1].strip()
        if line.startswith("//"):
            s = prev
        else:
            break
    return s


def rewrite(text: str) -> tuple[str, int]:
    """(new text, number of arms moved). Innermost matches first, one move per pass."""
    moved = 0
    for _ in range(1000):
        masked = _mask(text)
        hit = None
        for m in _MATCH.finditer(masked):
            brace = _top_level(masked, m.end(), len(masked), "{")
            if brace < 0:
                continue
            close = _close_of(masked, brace)
            arms = _arms(masked, brace + 1, close) if close > 0 else None
            if not arms or len(arms) < 2:
                continue
            for idx, (s, arrow, _e) in enumerate(arms[:-1]):
                if _is_wildcard(masked[s:arrow]):
                    hit = (arms, idx, close)
            if hit:
                break
        if not hit:
            return text, moved
        arms, idx, close = hit
        s, _arrow, e = arms[idx]
        last_s, _last_arrow, last_e = arms[-1]
        comma = "" if masked[last_s:last_e].rstrip().endswith((",", "}")) else ","
        line_end = text.find("\n", last_e)
        if line_end >= 0 and line_end < close and not text[last_e:line_end].strip():
            # Multi-line match: whole lines move, comments above the arm with it.
            seg_start = _leading_trivia(text, s)
            seg_end = text.find("\n", e) + 1
            segment = text[seg_start:seg_end]
            # Arms separated by blank lines: the blank line travels with the arm, ahead of it.
            nxt = arms[idx + 1][0]
            gap = text[seg_end : text.rfind("\n", 0, nxt) + 1]
            if gap and not gap.strip():
                seg_end += len(gap)
                segment = gap + segment
            insert_at = line_end + 1
            text = (
                text[:seg_start]
                + text[seg_end:last_e]
                + comma
                + text[last_e:insert_at]
                + segment
                + text[insert_at:]
            )
        else:
            segment = text[s:e].strip()
            gap_end = arms[idx + 1][0]
            text = text[:s] + text[gap_end:last_e] + comma + " " + segment + text[last_e:]
        moved += 1
    return text, moved


def rewrite_file(path: Path) -> int:
    text = path.read_text()
    new, moved = rewrite(text)
    if moved:
        path.write_text(new)
    return moved
