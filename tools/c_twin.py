"""
c_twin.py — find the C function a Rust function was translated from, by NAME.

Shared by the scanners whose oracle is the ORIGINAL C rather than a header catalog
(scan_ptr_byte_diff, scan_dropped_null_guard). Both need to ask "what did the C do at
this spot?", and a cohab flip leaves the C on disk with nothing linking the two files
(the problem `scan_dropped_seh_pairs` documents) — so pairing is by function name.

Lookup is lazy and batched: callers hand over every Rust function name they need,
and one `git grep -w` finds the candidate `.c` files, so a scan that has no
candidates never reads the 190 MB of C. Only TRACKED `.c`/`.cpp` files are searched: build
output and paid-artifact banks are never an oracle.

A name can have several C definitions (Wine and ReactOS copies, per-arch variants).
All are returned; `prefer()` orders them by how many path words they share with the
Rust crate, and a caller decides whether ANY or the BEST twin counts as evidence.
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

_C_COMMENT_OR_STRING = re.compile(
    r"//[^\n]*|/\*.*?\*/|\"(?:\\.|[^\"\\\n])*\"|'(?:\\.|[^'\\\n])+'", re.S
)
_RS_COMMENT_OR_STRING = re.compile(
    r"//[^\n]*|/\*.*?\*/|b?\"(?:\\.|[^\"\\])*\"|b?'(?:\\.|[^'\\\n])'", re.S
)


def _blank(m: re.Match[str]) -> str:
    """Same length, newlines kept, so offsets and line numbers survive."""
    t = m.group(0)
    if "\n" not in t:
        return " " * len(t)
    return "\n".join(" " * len(part) for part in t.split("\n"))


def blank_c(text: str) -> str:
    return _C_COMMENT_OR_STRING.sub(_blank, text)


def blank_rust(text: str) -> str:
    return _RS_COMMENT_OR_STRING.sub(_blank, text)


def match_brace(text: str, open_at: int) -> int:
    """Index just past the brace matching text[open_at] (a `{`), or -1."""
    depth = 0
    for i in range(open_at, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i + 1
    return -1


@dataclass(frozen=True)
class Func:
    name: str
    path: Path
    line: int  # 1-based line of the name
    body: str  # blanked text from `{` to the matching `}`, inclusive
    body_start: int  # offset of `{` in the (blanked) file text
    params: str  # blanked parameter list, without the parens


_RS_FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{]*>)?\s*\(")


def rust_functions(text: str, path: Path) -> list[Func]:
    """Every `fn` with a body in a Rust file. `text` must already be blanked."""
    out = []
    for m in _RS_FN.finditer(text):
        close = _match_paren(text, m.end() - 1)
        if close < 0:
            continue
        brace = text.find("{", close)
        semi = text.find(";", close)
        if brace < 0 or (0 <= semi < brace):
            continue
        end = match_brace(text, brace)
        if end < 0:
            continue
        out.append(
            Func(
                name=m.group(1),
                path=path,
                line=text.count("\n", 0, m.start()) + 1,
                body=text[brace:end],
                body_start=brace,
                params=text[m.end() : close],
            )
        )
    return out


def enclosing(funcs: list[Func], at: int) -> Func | None:
    """The innermost function whose body contains offset `at`."""
    owner = None
    for f in funcs:
        inside = f.body_start <= at < f.body_start + len(f.body)
        if inside and (owner is None or f.body_start > owner.body_start):
            owner = f
    return owner


def _match_paren(text: str, open_at: int) -> int:
    depth = 0
    for i in range(open_at, len(text)):
        c = text[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i
    return -1


# A C definition: the name, its parameter list, then (after K&R/annotation noise
# that is not a `;`) the opening brace. Calls and prototypes fail the brace test.
_C_KEYWORDS = {"if", "while", "for", "switch", "return", "sizeof", "do", "else", "case"}


def c_functions(text: str, path: Path, names: set[str]) -> list[Func]:
    """Definitions of any of `names` in one C file. `text` must already be blanked."""
    out = []
    pat = re.compile(r"\b(" + "|".join(re.escape(n) for n in sorted(names)) + r")\s*\(")
    for m in pat.finditer(text):
        name = m.group(1)
        if name in _C_KEYWORDS:
            continue
        # A definition's name is not preceded by an operator or `.`/`->` — that is a call.
        before = text[max(0, m.start() - 3) : m.start()].rstrip()
        if before.endswith((".", "->", "=", ",", "(", "!", "&", "|", "+", "-", "?", ":")):
            continue
        close = _match_paren(text, m.end() - 1)
        if close < 0:
            continue
        j = close + 1
        while j < len(text) and text[j] not in "{;=,)":
            j += 1
        if j >= len(text) or text[j] != "{":
            continue
        end = match_brace(text, j)
        if end < 0:
            continue
        out.append(
            Func(
                name=name,
                path=path,
                line=text.count("\n", 0, m.start()) + 1,
                body=text[j:end],
                body_start=j,
                params=text[m.end() : close],
            )
        )
    return out


INDEX_PATH = REPO / "tools" / ".cache" / "c_twin_defs.json"
_NAME_PAREN = re.compile(r"\b([A-Za-z_]\w*)\s*\(")


def defined_names(text: str) -> list[str]:
    """Names of the function DEFINITIONS at file scope. `text` must already be blanked.

    Only brace depth 0 is searched, and each body is skipped whole, so a call inside a
    function is never mistaken for a definition."""
    out = []
    pos = 0
    depth = 0
    n = len(text)
    while pos < n:
        m = _NAME_PAREN.search(text, pos)
        if m is None:
            break
        gap = text[pos : m.start()]
        depth += gap.count("{") - gap.count("}")
        if depth < 0:
            depth = 0
        pos = m.end()
        if depth != 0 or m.group(1) in _C_KEYWORDS:
            continue
        close = _match_paren(text, m.end() - 1)
        if close < 0:
            continue
        j = close + 1
        while j < n and text[j] not in "{;=,)":
            j += 1
        if j < n and text[j] == "{":
            end = match_brace(text, j)
            if end < 0:
                break
            out.append(m.group(1))
            pos = end
        else:
            pos = close + 1
    return out


def _index_stamp() -> str:
    return hashlib.sha256(Path(__file__).read_bytes()).hexdigest()[:16]


class CIndex:
    """Name → C definitions, over tracked `.c`/`.cpp` files outside the Rust tree.

    Which file defines which name is kept in tools/.cache/c_twin_defs.json, per file and
    keyed by (size, mtime_ns), so only changed files are re-read: a fresh `git grep` for
    6k names cost 81 s on every run, and the C side almost never changes. The cache is an
    accelerator, never an oracle: a stale or unreadable one is rebuilt, and the bodies a
    caller gets are always parsed from the file as it is now."""

    def __init__(self, repo: Path = REPO, cache: Path | None = INDEX_PATH):
        self.repo = repo
        self.cache = cache
        self._defs: dict[str, list[Func]] = {}
        self._blanked: dict[Path, str] = {}
        self._local: dict[tuple[Path, str], list[Func]] = {}
        self._by_name: dict[str, list[str]] | None = None

    def _tracked(self) -> list[str]:
        proc = subprocess.run(
            ["git", "-C", str(self.repo), "ls-files", "-z", "--", "*.c", "*.cpp", ":!sdk/lib/rust"],
            capture_output=True,
            text=True,
            check=False,
        )
        if proc.returncode != 0:
            raise RuntimeError(f"git ls-files failed: {proc.stderr.strip()}")
        return [f for f in proc.stdout.split("\0") if f]

    def _build(self) -> dict[str, list[str]]:
        stamp = _index_stamp()
        old: dict = {}
        if self.cache is not None:
            try:
                data = json.loads(self.cache.read_text())
                if data.get("gen") == stamp:
                    old = data.get("files", {})
            except (OSError, ValueError):
                old = {}
        files: dict[str, list] = {}
        for rel in self._tracked():
            path = self.repo / rel
            try:
                st = path.stat()
            except FileNotFoundError:
                continue  # deleted in the worktree, not yet in the index
            key = [st.st_size, st.st_mtime_ns]
            ent = old.get(rel)
            if ent and ent[0] == key:
                files[rel] = ent
                continue
            try:
                raw = path.read_text(errors="replace")
            except OSError as e:
                raise RuntimeError(f"cannot read tracked C source {rel}: {e}") from e
            files[rel] = [key, sorted(set(defined_names(blank_c(raw))))]
        if self.cache is not None and files != old:
            try:
                self.cache.parent.mkdir(parents=True, exist_ok=True)
                tmp = self.cache.with_suffix(".tmp")
                tmp.write_text(json.dumps({"gen": stamp, "files": files}))
                tmp.replace(self.cache)
            except OSError:
                pass
        by_name: dict[str, list[str]] = {}
        for rel, (_, names) in files.items():
            for name in names:
                by_name.setdefault(name, []).append(rel)
        return by_name

    def load(self, names: set[str]) -> None:
        want = {n for n in names if n not in self._defs and n not in _C_KEYWORDS}
        if not want:
            return
        if self._by_name is None:
            self._by_name = self._build()
        present: dict[str, set[str]] = {}
        for n in want:
            self._defs[n] = []
            for rel in self._by_name.get(n, []):
                present.setdefault(rel, set()).add(n)
        for rel in sorted(present):
            path = self.repo / rel
            text = self._blanked.get(path)
            if text is None:
                text = blank_c(path.read_text(errors="replace"))
                self._blanked[path] = text
            for f in c_functions(text, path, present[rel]):
                self._defs[f.name].append(f)

    def get(self, name: str) -> list[Func]:
        return self._defs.get(name, [])

    def in_file(self, path: Path, name: str) -> list[Func]:
        """Definitions of `name` in one already-known C file (a callee next to its caller)."""
        key = (path, name)
        if key not in self._local:
            text = self._blanked.get(path)
            if text is None:
                text = blank_c(path.read_text(errors="replace"))
                self._blanked[path] = text
            self._local[key] = c_functions(text, path, {name}) if name in text else []
        return self._local[key]


def crate_of(path: Path) -> str:
    parts = path.parts
    if "rust" in parts:
        i = parts.index("rust")
        if i + 1 < len(parts):
            return parts[i + 1]
    return ""


def affinity(rust_path: Path, twin: Func) -> int:
    """Path words a C twin shares with the Rust file (crate name, file stem, subsystem)."""
    words = set(re.split(r"[^a-z0-9]+", str(rust_path).lower())) - {
        "",
        "src",
        "rust",
        "rs",
        "sdk",
        "lib",
    }
    words |= set(re.split(r"[^a-z0-9]+", crate_of(rust_path).lower())) - {"", "rust"}
    try:
        rel = twin.path.relative_to(REPO)
    except ValueError:
        rel = twin.path
    return len(words & set(re.split(r"[^a-z0-9]+", str(rel).lower())))


def prefer(rust_path: Path, twins: list[Func]) -> list[Func]:
    """Order C twins by how many path words they share with the Rust file's path."""
    return sorted(twins, key=lambda f: (-affinity(rust_path, f), str(f.path)))
