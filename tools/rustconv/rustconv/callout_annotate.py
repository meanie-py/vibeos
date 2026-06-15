"""Win32k Pre-Translation Pass 3 — USER callout annotation (extract-only).

The audit (docs/engineering/win32k-pretranslation-audit.md §"Pass 3")
specifies: *no automated source rewrite*. The win32k user-mode callout
pattern is structurally diverse; mechanical rewriting is risky. Pass 3 is
therefore an **extractor**: it produces a deterministic per-file sidecar
JSON describing every real `KeUserModeCallback` site so the rustconv
prompt builder can hand the LLM a structured "this is a callout — here is
the lock-release/reacquire pair, the marshal-in buffer, and the
marshal-back probe" hint instead of making it re-derive the pattern in
every translation-unit prompt.

The callout shape in win32k is highly regular:

    NTSTATUS co_IntFoo(...)            <- enclosing wrapper (co_ prefix)
    {
        ...marshal args into pArguments...
        UserLeaveCo();                 <- release the global USER lock
        Status = KeUserModeCallback(USER32_CALLBACK_FOO,    <- callback idx
                                    pArguments, ArgLen,     <- marshal-in
                                    &ResultPointer,         <- marshal-back
                                    &ResultLength);
        UserEnterCo();                 <- reacquire the lock
        ...
        Status = SafeProbeForRead(ResultPointer, ...);      <- marshal-back probe
        ...
    }

This module locates each `KeUserModeCallback(` call, resolves its
enclosing function + line span, splits the five call arguments, and finds
the bracketing `UserLeaveCo`/`UserEnterCo` and the first post-reacquire
probe. Pure regex + brace-matching, consistent with the rest of the
win32k pretranslation tooling (tree-sitter cannot parse the SEH/callout
macros). Annotation-only; the source is never mutated.

CLI: `python -m rustconv callout-annotate [path] [--json]
      [--write-sidecars] [--oracle]`  (mirrors `seh-inventory`).
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from .seh_inventory import _in_keepset

RE_KUMC = re.compile(r"\bKeUserModeCallback\s*\(")
RE_CB_INDEX = re.compile(r"\bUSER32_CALLBACK_[A-Z0-9_]+\b")
RE_LEAVE_CO = re.compile(r"\bUserLeaveCo\s*\(\s*\)")
RE_ENTER_CO = re.compile(r"\bUserEnterCo\s*\(\s*\)")
# The marshal-back probe Pass 1 lowered to (or a still-bare probe).
RE_RESULT_PROBE = re.compile(
    r"\b(SafeProbeForRead|SafeProbeForWrite|ProbeForRead|ProbeForWrite)\s*\("
)


def _line_of(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def _balanced_args(text: str, open_paren: int) -> tuple[str, int]:
    """Given the index of a `(`, return (inner_text, index_of_matching_`)`).
    Honors nested parens, string and char literals."""
    depth = 0
    i = open_paren
    n = len(text)
    in_str = in_chr = False
    while i < n:
        c = text[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif in_chr:
            if c == "\\":
                i += 2
                continue
            if c == "'":
                in_chr = False
        elif c == '"':
            in_str = True
        elif c == "'":
            in_chr = True
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return text[open_paren + 1 : i], i
        i += 1
    return text[open_paren + 1 :], n - 1


def _split_top_commas(s: str) -> list[str]:
    """Split on top-level commas only (skip nested () [] and literals)."""
    out: list[str] = []
    depth = 0
    buf: list[str] = []
    in_str = in_chr = False
    i = 0
    while i < len(s):
        c = s[i]
        if in_str:
            buf.append(c)
            if c == "\\" and i + 1 < len(s):
                buf.append(s[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
        elif in_chr:
            buf.append(c)
            if c == "\\" and i + 1 < len(s):
                buf.append(s[i + 1])
                i += 2
                continue
            if c == "'":
                in_chr = False
        elif c == '"':
            in_str = True
            buf.append(c)
        elif c == "'":
            in_chr = True
            buf.append(c)
        elif c in "([":
            depth += 1
            buf.append(c)
        elif c in ")]":
            depth -= 1
            buf.append(c)
        elif c == "," and depth == 0:
            out.append("".join(buf).strip())
            buf = []
        else:
            buf.append(c)
        i += 1
    if buf:
        out.append("".join(buf).strip())
    return out


def _mask_literals(text: str) -> str:
    """Length-preserving copy with string/char-literal and comment bodies
    blanked (kept same length, newlines preserved) so brace/paren scans
    and line counts stay index-accurate."""
    out = list(text)
    n = len(text)
    i = 0
    while i < n:
        c = text[i]
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            while i < n and text[i] != "\n":
                out[i] = " "
                i += 1
        elif c == "/" and i + 1 < n and text[i + 1] == "*":
            out[i] = out[i + 1] = " "
            i += 2
            while i < n and not (text[i] == "*" and i + 1 < n and text[i + 1] == "/"):
                if text[i] != "\n":
                    out[i] = " "
                i += 1
            if i < n:
                out[i] = " "
                if i + 1 < n:
                    out[i + 1] = " "
                i += 2
        elif c in "\"'":
            q = c
            out[i] = " "
            i += 1
            while i < n and text[i] != q:
                if text[i] == "\\" and i + 1 < n:
                    out[i] = out[i + 1] = " "
                    i += 2
                    continue
                if text[i] != "\n":
                    out[i] = " "
                i += 1
            if i < n:
                out[i] = " "
                i += 1
        else:
            i += 1
    return "".join(out)


def _enclosing_func_span(text: str, pos: int) -> tuple[str, int, int]:
    """(name, def_line, end_line) of the function definition enclosing
    `pos`, found by brace-depth scan (robust to file-scope `typedef
    struct {...} Name;` and `= { ... }` initializers, which the simpler
    "previous `}` on its own line" heuristic mis-bounds).

    The enclosing function body `{` is the *last* brace-depth-0 `{`
    before `pos` whose preceding non-whitespace char is `)` (a function
    signature's param-list close). Struct/initializer braces are excluded
    (preceded by an identifier or `=`); control braces live at depth>0."""
    masked = _mask_literals(text)
    depth = 0
    func_open = -1
    i = 0
    while i < pos:
        c = masked[i]
        if c == "{":
            if depth == 0:
                j = i - 1
                while j >= 0 and masked[j] in " \t\r\n":
                    j -= 1
                if j >= 0 and masked[j] == ")":
                    func_open = i
            depth += 1
        elif c == "}":
            depth -= 1
        i += 1

    name = "?"
    if func_open == -1:
        return name, _line_of(text, 0), _line_of(text, len(text))

    # Walk back from the body `{` over the param-list to its `(`, then the
    # identifier just before that `(` is the function name.
    j = func_open - 1
    while j >= 0 and masked[j] in " \t\r\n":
        j -= 1
    # j is at the signature's `)`; balance back to its matching `(`.
    pdepth = 0
    while j >= 0:
        if masked[j] == ")":
            pdepth += 1
        elif masked[j] == "(":
            pdepth -= 1
            if pdepth == 0:
                break
        j -= 1
    paren_open = j
    nm = re.search(r"([A-Za-z_]\w*)\s*$", masked[:paren_open])
    def_line = _line_of(text, func_open)
    if nm:
        name = nm.group(1)
        def_line = _line_of(text, nm.start(1))

    # End bound: the matching `}` of the function body.
    depth = 0
    end_pos = len(text)
    k = func_open
    while k < len(masked):
        if masked[k] == "{":
            depth += 1
        elif masked[k] == "}":
            depth -= 1
            if depth == 0:
                end_pos = k
                break
        k += 1
    return name, def_line, _line_of(text, end_pos)


@dataclass
class CalloutSite:
    function: str
    func_line: int
    func_end_line: int
    callback_index: str  # USER32_CALLBACK_*  (or "?" if not literal)
    kumc_line: int
    input_arg: str  # marshal-in buffer expr
    input_len_arg: str
    result_ptr_arg: str  # marshal-back pointer expr
    result_len_arg: str
    leave_co_line: int | None  # global USER lock release before callout
    enter_co_line: int | None  # reacquire after callout
    result_probe_line: int | None  # first marshal-back probe after reacquire
    signals: list[str] = field(default_factory=list)


def extract_file(path: Path) -> list[CalloutSite]:
    text = path.read_text(errors="replace")
    sites: list[CalloutSite] = []
    for km in RE_KUMC.finditer(text):
        open_paren = text.index("(", km.start())
        inner, close = _balanced_args(text, open_paren)
        args = _split_top_commas(inner)
        # KeUserModeCallback(Index, InputBuffer, InputLength,
        #                    *OutputBuffer, *OutputLength)
        idx = args[0].strip() if args else "?"
        cb = RE_CB_INDEX.search(idx)
        callback_index = cb.group(0) if cb else (idx or "?")
        fn_name, fn_line, fn_end = _enclosing_func_span(text, km.start())
        kumc_line = _line_of(text, km.start())

        # Bracketing lock release/reacquire, scoped to the enclosing fn.
        fn_lo = text.rfind("\n}", 0, km.start())
        fn_lo = fn_lo + 1 if fn_lo != -1 else 0
        before = text[fn_lo : km.start()]
        after_start = close + 1
        em = re.search(r"\n\}\s*\n", text[after_start:])
        after = text[after_start : after_start + (em.start() if em else len(text))]

        lv = list(RE_LEAVE_CO.finditer(before))
        leave_line = _line_of(text, fn_lo + lv[-1].start()) if lv else None
        en = RE_ENTER_CO.search(after)
        enter_line = _line_of(text, after_start + en.start()) if en else None
        pr = RE_RESULT_PROBE.search(after, en.end() if en else 0)
        probe_line = _line_of(text, after_start + pr.start()) if pr else None

        signals: list[str] = []
        if not cb:
            signals.append("nonliteral_callback_index")
        if leave_line is None or enter_line is None:
            signals.append("missing_leaveco_enterco_pair")
        if probe_line is None:
            signals.append("no_marshal_back_probe")
        if fn_name.startswith("co_"):
            signals.append("co_prefixed")

        sites.append(
            CalloutSite(
                function=fn_name,
                func_line=fn_line,
                func_end_line=fn_end,
                callback_index=callback_index,
                kumc_line=kumc_line,
                input_arg=(args[1].strip() if len(args) > 1 else "?"),
                input_len_arg=(args[2].strip() if len(args) > 2 else "?"),
                result_ptr_arg=(args[3].strip() if len(args) > 3 else "?"),
                result_len_arg=(args[4].strip() if len(args) > 4 else "?"),
                leave_co_line=leave_line,
                enter_co_line=enter_line,
                result_probe_line=probe_line,
                signals=signals,
            )
        )
    return sites


def extract_keepset(win32ss_root: Path) -> dict[Path, list[CalloutSite]]:
    result: dict[Path, list[CalloutSite]] = {}
    for c in sorted(win32ss_root.rglob("*.c")):
        if not _in_keepset(c):
            continue
        sites = extract_file(c)
        if sites:
            result[c] = sites
    return result


def write_sidecar(path: Path, sites: list[CalloutSite]) -> Path:
    out = path.with_suffix(path.suffix + ".win32k_callouts.json")
    payload = {
        "source": str(path),
        "schema": "win32k_pass3_v1",
        "count": len(sites),
        "sites": [s.__dict__ for s in sites],
    }
    out.write_text(json.dumps(payload, indent=2))
    return out


def print_summary(inv: dict[Path, list[CalloutSite]]) -> None:
    n = sum(len(v) for v in inv.values())
    print(f"\n{'=' * 64}")
    print(f"Win32k Pass 3 — {n} KeUserModeCallback sites across {len(inv)} keep-set files")
    print(f"{'=' * 64}")
    for p, ss in sorted(inv.items(), key=lambda x: -len(x[1])):
        rel = p.as_posix().split("/win32ss/")[-1]
        idxs = ", ".join(sorted({s.callback_index.replace("USER32_CALLBACK_", "") for s in ss}))
        print(f"  {len(ss):>2}  {rel}")
        print(f"      {idxs}")
    paired = sum(
        1 for ss in inv.values() for s in ss if "missing_leaveco_enterco_pair" not in s.signals
    )
    print(f"\n  {paired}/{n} sites have a clean UserLeaveCo/UserEnterCo pair")


# --- callback.c ground-truth oracle -----------------------------------------
# Observed directly from the tree 2026-05-16. The first callout in
# callback.c is co_IntClientLoadLibrary / USER32_CALLBACK_CLIENTLOADLIBRARY,
# and the lock release/reacquire must straddle the KeUserModeCallback call.
def run_oracle(callback_path: Path) -> int:
    sites = extract_file(callback_path)
    print(f"callback.c: {len(sites)} KeUserModeCallback sites")
    ok = True

    # callback.c has exactly 17 real KeUserModeCallback calls (the often-
    # miscounted 18th grep hit is the `ERR(...)` log line after SENDASYNC).
    if len(sites) != 17:
        print(f"  count: expect 17, got {len(sites)}  [MISMATCH]")
        ok = False
    else:
        print("  count: 17  [OK]")

    first = sites[0] if sites else None
    if (
        first
        and first.function == "co_IntClientLoadLibrary"
        and first.callback_index == "USER32_CALLBACK_CLIENTLOADLIBRARY"
    ):
        print("  site[0] = co_IntClientLoadLibrary/CLIENTLOADLIBRARY  [OK]")
    else:
        got = f"{first.function}/{first.callback_index}" if first else "none"
        print(f"  site[0]: expect co_IntClientLoadLibrary/CLIENTLOADLIBRARY, got {got}  [MISMATCH]")
        ok = False

    if first and (
        first.leave_co_line
        and first.enter_co_line
        and first.leave_co_line < first.kumc_line < first.enter_co_line
    ):
        print("  site[0] lock straddle: Leave < KUMC < Enter  [OK]")
    else:
        print("  site[0] lock straddle  [MISMATCH]")
        ok = False

    nonliteral = [s for s in sites if "nonliteral_callback_index" in s.signals]
    if nonliteral:
        print(f"  {len(nonliteral)} non-literal callback index  [MISMATCH]")
        ok = False
    else:
        print("  all callback indices are USER32_CALLBACK_* literals  [OK]")

    print("ORACLE PASS" if ok else "ORACLE FAIL")
    return 0 if ok else 1
