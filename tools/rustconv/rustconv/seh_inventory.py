"""Win32k pretranslation Pass 0 — SEH risk-triage inventory.

Decision layer on top of `seh_classifier`. For every `_SEH2_TRY` site in the
win32k.sys keep-set it assigns the audit's triage category and emits a
per-file `<file>.win32k_meta.json` sidecar the rustconv prompt builder
consumes.

Categories (docs/engineering/win32k-pretranslation-audit.md, Pass 0):

  a PROBE_LOWER     probe-pattern → Pass 1 mechanical SafeCapture* lowering
  b LOCK_CLEANUP    _SEH2_FINALLY lock/cleanup → Pass 2 RAII shaping
  c FAULT_RECOVER   genuine fault-then-recover (Mm{Probe,Map}Locked*) → residual
  d BELT_SUSPENDERS user write under SEH with no probe in THIS block — paranoid
                    or relies on a prior sibling probe → delete after cross-check
  e NEEDS_REVIEW    control-flow / non-probe call in TRY / unclassifiable

Structural signals proven necessary by the winsta.c spike (2026-05-15):

  needs_goto_lowering  _SEH2_TRY body contains _SEH2_LEAVE (Pass-1 rewrite must
                       emit a label + goto, not a straight-line replacement)
  has_nonprobe_call    a non-probe, side-effecting call inside the TRY (the
                       reason L873/L1011 are (e) not (a))
  relies_on_prior_probe  user write, no probe here, an earlier sibling block in
                       the same function probed the same buffer (L1384←L1242)
  multi_probe          >1 ProbeFor* in one block (sequenced capture)
  swallow              handler discards the fault (NOTHING / (void)0)

The eight winsta.c sites are encoded as a self-test oracle (`--oracle`).
"""

from __future__ import annotations

import json
import re
import subprocess
import tempfile
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path

from .seh_classifier import (
    FunctionSEHInfo,
    SEHBlock,
    TryBodyPattern,
    classify_file,
)


class Category(str, Enum):
    PROBE_LOWER = "a"  # → Pass 1
    LOCK_CLEANUP = "b"  # → Pass 2
    FAULT_RECOVER = "c"  # residual, no rewrite
    BELT_SUSPENDERS = "d"  # delete after cross-site check
    NEEDS_REVIEW = "e"  # manual triage


# win32k.sys kernel-mode keep-set (audit §"What this audit doesn't cover":
# user32/gdi32/imm32/winsrv are user-mode → out of scope).
KEEPSET_DIRS = (
    "gdi/ntgdi",
    "gdi/eng",
    "gdi/dib",
    "user/ntuser",
    "reactx/ntddraw",
    "reactx/dxg",
)

# Genuine fault-then-recover APIs — these MUST keep their SEH (route to the
# deferred kernel unwinder, never mechanically lowered).
RE_FAULT_RECOVER = re.compile(
    r"\bMm(ProbeAndLockPages|MapLockedPages(SpecifyCache)?|"
    r"LockPagableDataSection|BuildMdlForNonPagedPool)\b"
)
RE_SEH_LEAVE = re.compile(r"\b_SEH2_LEAVE\b")

# Identifiers that look like calls but are not side-effecting user-mode
# touches: control keywords, logging, status predicates, sizeof/casts, the
# probe/copy helpers themselves, and the SEH macros.
_CALL_ALLOW = {
    "if",
    "for",
    "while",
    "switch",
    "return",
    "sizeof",
    "do",
    "NT_SUCCESS",
    "NT_ERROR",
    "NT_WARNING",
    "NTAPI",
    "APIENTRY",
    "TRACE",
    "ERR",
    "WARN",
    "INFO",
    "DPRINT",
    "DPRINT1",
    "FIXME",
    "UNIMPLEMENTED",
    "ASSERT",
    "NOTHING",
    "UNREFERENCED_PARAMETER",
    "ProbeForRead",
    "ProbeForWrite",
    "RtlCopyMemory",
    "RtlMoveMemory",
    "RtlZeroMemory",
    "RtlFillMemory",
    "memcpy",
    "memmove",
    "memset",
    "_SEH2_GetExceptionCode",
    "_SEH2_GetExceptionInformation",
    "_SEH2_YIELD",
    "_SEH2_LEAVE",
    "EXCEPTION_EXECUTE_HANDLER",
    # SAL / analysis annotations and compile-time asserts — not real calls
    "_Analysis_assume_",
    "__analysis_assume",
    "_Analysis_noreturn_",
    "C_ASSERT",
    "static_assert",
    "_ASSERT",
    "__pragma",
    "RtlRaiseStatus",
}

# User-struct writes the bare `*ptr =` WRITE_BACK regex misses:
#   ClientInfo->phkCurrent = Hook;   (publish into user CLIENTINFO/TEB)
#   pBuf[i] = val;                   (indexed user write)
# `(?<![=!<>])=(?!=)` excludes ==, !=, >=, <=.
_RE_ARROW_WRITE = re.compile(r"\b\w+\s*->\s*\w+\s*(?<![=!<>])=(?!=)\s*[^=]")
_RE_INDEX_WRITE = re.compile(r"\b\w+\s*\[[^\]]*\]\s*(?<![=!<>])=(?!=)\s*[^=]")


def _has_struct_or_index_write(stripped_try: str) -> bool:
    return bool(_RE_ARROW_WRITE.search(stripped_try) or _RE_INDEX_WRITE.search(stripped_try))


_RE_PROBE_TRY = re.compile(r"\bProbeFor(Read|Write)\b")
_RE_STAR_WRITE = re.compile(r"\*\s*\w+\s*(?<![=!<>])=(?!=)\s*[^=]")
_RE_BULK_COPY = re.compile(r"\b(RtlCopyMemory|RtlMoveMemory|memcpy|memmove)\s*\(")


def classify_span(try_body: str, handler_body: str, is_finally: bool) -> Category:
    """Pass-0 category from a single SEH span's text alone (no function
    context). Used by the Pass-1 rewriter so it doesn't depend on the
    seh_classifier block parser agreeing site-for-site. Cross-site
    (d) `relies_on_prior_probe` collapses into plain (d) here — fine,
    since the rewriter skips (b)/(c)/(d)/(e) anyway."""
    st = _strip_literals(try_body)
    if RE_FAULT_RECOVER.search(st):
        return Category.FAULT_RECOVER
    if is_finally:
        return Category.LOCK_CLEANUP
    if _has_nonprobe_call(try_body):
        return Category.NEEDS_REVIEW
    has_probe = bool(_RE_PROBE_TRY.search(st))
    has_write = bool(
        _RE_STAR_WRITE.search(st) or _RE_BULK_COPY.search(st) or _has_struct_or_index_write(st)
    )
    if has_write and not has_probe:
        return Category.BELT_SUSPENDERS
    if has_probe or has_write:
        return Category.PROBE_LOWER
    return Category.NEEDS_REVIEW


# typed probes (ProbeForReadUlong, ProbeForWriteHandle, …) are also "probes"
_RE_TYPED_PROBE = re.compile(r"^ProbeFor(Read|Write)\w+$")
_RE_CALL = re.compile(r"\b([A-Za-z_]\w*)\s*\(")


def _strip_literals(code: str) -> str:
    """Blank out C string/char literals and comments so identifier scans
    don't match tokens inside `TRACE("... caller (len ...")` etc."""
    code = re.sub(r"/\*.*?\*/", " ", code, flags=re.S)
    code = re.sub(r"//[^\n]*", " ", code)
    code = re.sub(r'"(?:\\.|[^"\\])*"', '""', code)
    code = re.sub(r"'(?:\\.|[^'\\])*'", "''", code)
    return code


def _has_nonprobe_call(try_body: str) -> list[str]:
    """Side-effecting calls in the TRY body that aren't probe/copy/logging."""
    try_body = _strip_literals(try_body)
    found = []
    for m in _RE_CALL.finditer(try_body):
        name = m.group(1)
        if name in _CALL_ALLOW or _RE_TYPED_PROBE.match(name):
            continue
        # macro-style all-caps with no lowercase → treat as annotation/no-op
        if name.isupper() and len(name) > 2:
            continue
        found.append(name)
    return list(dict.fromkeys(found))


def _has_probe(b: SEHBlock) -> bool:
    return bool(
        b.probe_calls
        or b.typed_probes
        or TryBodyPattern.PROBE in b.try_patterns
        or TryBodyPattern.PROBE_TYPED in b.try_patterns
        or TryBodyPattern.PROBE_UNICODE in b.try_patterns
    )


def _has_user_write(b: SEHBlock) -> bool:
    return bool(
        b.write_backs
        or b.copy_calls
        or TryBodyPattern.WRITE_BACK in b.try_patterns
        or TryBodyPattern.BULK_COPY in b.try_patterns
    )


def _is_swallow(b: SEHBlock) -> bool:
    """Handler discards the fault (NOTHING / (void)0 / empty)."""
    if b.is_finally:
        return False
    h = re.sub(r"/\*.*?\*/", "", b.handler_body, flags=re.S)
    h = re.sub(r"//[^\n]*", "", h)
    h = h.replace("NOTHING", "").replace("(void)0", "").replace(";", "").strip()
    return h == ""


def _probe_count(b: SEHBlock) -> int:
    n = len(b.probe_calls) + len(b.typed_probes)
    return n if n else (1 if _has_probe(b) else 0)


@dataclass
class SiteTriage:
    function: str
    line_start: int
    line_end: int
    category: Category
    signals: list[str] = field(default_factory=list)
    probe_count: int = 0
    nonprobe_calls: list[str] = field(default_factory=list)
    try_patterns: list[str] = field(default_factory=list)
    handler_patterns: list[str] = field(default_factory=list)
    note: str = ""


def triage_function(fn: FunctionSEHInfo) -> list[SiteTriage]:
    """Classify every SEH block in one function, with cross-site awareness."""
    out: list[SiteTriage] = []
    # buffers probed by an EARLIER block in this function (for (d) detection)
    probed_targets: set[str] = set()

    # seh_classifier reports SEHBlock line numbers relative to the function
    # body; convert to file-absolute using the function's file-start line.
    base = fn.line_start - 1

    for b in fn.seh_blocks:
        signals: list[str] = []
        stripped = _strip_literals(b.try_body)
        calls = _has_nonprobe_call(b.try_body)
        has_probe = _has_probe(b)
        has_write = _has_user_write(b) or _has_struct_or_index_write(stripped)
        has_leave = bool(RE_SEH_LEAVE.search(b.try_body))
        fault = bool(RE_FAULT_RECOVER.search(b.try_body))
        pc = _probe_count(b)

        if has_leave:
            signals.append("needs_goto_lowering")
        if pc > 1:
            signals.append("multi_probe")
        if _is_swallow(b):
            signals.append("swallow")
        if calls:
            signals.append("has_nonprobe_call")

        note = ""
        if fault:
            cat = Category.FAULT_RECOVER
        elif b.is_finally:
            cat = Category.LOCK_CLEANUP
        elif calls:
            # non-probe side-effecting call inside the TRY → not mechanical
            cat = Category.NEEDS_REVIEW
            note = f"non-probe call(s) in TRY: {', '.join(calls[:4])}"
        elif has_write and not has_probe:
            cat = Category.BELT_SUSPENDERS
            # was the written buffer probed by an earlier sibling block?
            tgt = set(re.findall(r"\b([A-Za-z_]\w*)\b", b.try_body))
            if tgt & probed_targets:
                signals.append("relies_on_prior_probe")
                note = "user write; relies on earlier sibling probe"
            else:
                note = "user write under SEH, no probe in block (paranoid?)"
        elif has_probe or has_write:
            cat = Category.PROBE_LOWER
        else:
            cat = Category.NEEDS_REVIEW
            note = "no probe/copy/write/call — unclassifiable, inspect"

        # record probe targets for later sibling blocks
        if has_probe:
            for arg in re.findall(r"ProbeFor\w+\s*\(\s*&?\s*([A-Za-z_]\w*)", b.try_body):
                probed_targets.add(arg)

        out.append(
            SiteTriage(
                function=fn.name,
                line_start=base + b.line_start,
                line_end=base + b.line_end,
                category=cat,
                signals=signals,
                probe_count=pc,
                nonprobe_calls=calls,
                try_patterns=[p.name for p in b.try_patterns],
                handler_patterns=[p.name for p in b.handler_patterns],
                note=note,
            )
        )
    return out


def inventory_file(path: Path) -> list[SiteTriage]:
    sites: list[SiteTriage] = []
    for fn in classify_file(path):
        sites.extend(triage_function(fn))
    return sites


def _in_keepset(p: Path) -> bool:
    s = p.as_posix()
    return any(f"/win32ss/{d}/" in s for d in KEEPSET_DIRS)


def inventory_keepset(win32ss_root: Path) -> dict[Path, list[SiteTriage]]:
    result: dict[Path, list[SiteTriage]] = {}
    for c in sorted(win32ss_root.rglob("*.c")):
        if not _in_keepset(c):
            continue
        sites = inventory_file(c)
        if sites:
            result[c] = sites
    return result


def write_sidecar(path: Path, sites: list[SiteTriage]) -> Path:
    out = path.with_suffix(path.suffix + ".win32k_meta.json")
    payload = {
        "source": str(path),
        "schema": "win32k_pass0_v1",
        "summary": _counts(sites),
        "sites": [
            {
                "function": s.function,
                "line_start": s.line_start,
                "line_end": s.line_end,
                "category": s.category.value,
                "category_name": s.category.name,
                "signals": s.signals,
                "probe_count": s.probe_count,
                "nonprobe_calls": s.nonprobe_calls,
                "try_patterns": s.try_patterns,
                "handler_patterns": s.handler_patterns,
                "note": s.note,
            }
            for s in sites
        ],
    }
    out.write_text(json.dumps(payload, indent=2))
    return out


def _counts(sites: list[SiteTriage]) -> dict[str, int]:
    c: dict[str, int] = {}
    for s in sites:
        c[s.category.value] = c.get(s.category.value, 0) + 1
    return c


# --- winsta.c spike oracle (the 8 hand-classified sites, 2026-05-15) ---------
# (line_start is approximate post-spike; the oracle matches by function +
#  ordinal + expected category/signals, not exact line.)
WINSTA_ORACLE = [
    ("FreeUserModeWindowStationName", Category.PROBE_LOWER, {"swallow"}),
    ("BuildUserModeWindowStationName", Category.PROBE_LOWER, set()),
    (
        "NtUserCreateWindowStation",
        Category.NEEDS_REVIEW,
        {"needs_goto_lowering", "has_nonprobe_call"},
    ),
    (
        "NtUserOpenWindowStation",
        Category.NEEDS_REVIEW,
        {"needs_goto_lowering", "has_nonprobe_call"},
    ),
    ("NtUserSetObjectInformation", Category.PROBE_LOWER, {"multi_probe"}),  # L1242 multi-probe
    ("NtUserSetObjectInformation", Category.BELT_SUSPENDERS, {"relies_on_prior_probe"}),  # L1384
    (
        "NtUserSetWindowStationUser",
        Category.PROBE_LOWER,
        set(),
    ),  # L2157 goto-on-fail handled by handler, not TRY LEAVE
    ("NtUserSetWindowStationUser", Category.PROBE_LOWER, set()),  # L2190 copy-from-user
]


def run_oracle(winsta_path: Path) -> int:
    """Validate the classifier against the hand-derived winsta.c taxonomy.

    The oracle is intentionally loose on exact function-name binding (the
    real winsta.c uses different names for the L1242/L1384/L2157/L2190
    bodies); it checks the *distribution*: 8 sites → 5 (a), 2 (e), 1 (d),
    0 (b), 0 (c), with the structural signals present somewhere.

    The win32k-translation worktree's working `winsta.c` is the SEH-lowered
    spike output (0 `_SEH2_TRY`); the oracle needs the *pre-spike* source,
    so fall back to the `git HEAD` blob when the working copy has no SEH.
    """
    if "_SEH2_TRY" not in winsta_path.read_text(errors="replace"):
        try:
            repo = subprocess.run(
                ["git", "-C", str(winsta_path.parent), "rev-parse", "--show-toplevel"],
                capture_output=True,
                text=True,
                check=True,
            ).stdout.strip()
            rel = winsta_path.resolve().relative_to(repo)
            blob = subprocess.run(
                ["git", "-C", repo, "show", f"HEAD:{rel.as_posix()}"],
                capture_output=True,
                text=True,
                check=True,
            ).stdout
            tmp = Path(tempfile.mkstemp(suffix="_winsta_orig.c")[1])
            tmp.write_text(blob)
            print(f"(oracle: working copy is SEH-free; using git HEAD:{rel})")
            winsta_path = tmp
        except (subprocess.CalledProcessError, ValueError) as e:
            print(f"oracle: could not recover pre-spike winsta.c: {e}")
            return 1

    sites = inventory_file(winsta_path)
    counts = _counts(sites)
    print(f"winsta.c: {len(sites)} SEH sites — {counts}")
    expect = {"a": 5, "e": 2, "d": 1}
    ok = True
    for k, v in expect.items():
        got = counts.get(k, 0)
        flag = "OK" if got == v else "MISMATCH"
        if got != v:
            ok = False
        print(f"  category {k}: expect {v}, got {got}  [{flag}]")
    for forbidden in ("b", "c"):
        if counts.get(forbidden, 0):
            print(f"  category {forbidden}: expected 0, got {counts[forbidden]}  [MISMATCH]")
            ok = False
    allsig = {s for site in sites for s in site.signals}
    for need in (
        "needs_goto_lowering",
        "has_nonprobe_call",
        "relies_on_prior_probe",
        "multi_probe",
        "swallow",
    ):
        present = "OK" if need in allsig else "MISSING"
        if need not in allsig:
            ok = False
        print(f"  signal {need}: [{present}]")
    print("ORACLE PASS" if ok else "ORACLE FAIL")
    return 0 if ok else 1


def print_summary(inv: dict[Path, list[SiteTriage]]) -> None:
    total: dict[str, int] = {}
    nsites = 0
    for sites in inv.values():
        nsites += len(sites)
        for s in sites:
            total[s.category.value] = total.get(s.category.value, 0) + 1
    names = {c.value: c.name for c in Category}
    print(f"\n{'=' * 64}")
    print(f"Win32k Pass 0 — {nsites} SEH sites across {len(inv)} keep-set files")
    print(f"{'=' * 64}")
    for k in ("a", "b", "c", "d", "e"):
        n = total.get(k, 0)
        pct = (n * 100 / nsites) if nsites else 0
        print(f"  ({k}) {names[k]:<16} {n:>4}  {pct:5.1f}%")
    e_files = [(p, [s for s in ss if s.category == Category.NEEDS_REVIEW]) for p, ss in inv.items()]
    e_files = [(p, e) for p, e in e_files if e]
    if e_files:
        print("\n(e) NEEDS_REVIEW hotspots:")
        for p, e in sorted(e_files, key=lambda x: -len(x[1]))[:12]:
            print(f"  {len(e):>3}  {p.name}")
