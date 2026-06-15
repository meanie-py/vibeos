"""Win32k Pre-Translation Pass 2 — GDI manual-lock-bracket detection.

The audit (docs/engineering/win32k-pretranslation-audit.md) re-scoped
Pass 2: the kernel keep-set has **zero `_SEH2_FINALLY`**, so the original
"SEH-wrapped lock cleanup → RAII" rewrite has nothing to match. The real
Pass-2 work is detecting the *manual* GDI lock brackets that the SEH
classifier dumped into the `(e)` NEEDS_REVIEW bucket:

    if (!(dc = DC_LockDc(hDC)))         <- acquire + NULL guard
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID;             <- early-exit (no unlock — correct)
    }
    ...body...
    DC_UnlockDc(dc);                    <- release (the Drop point)
    return oldColor;

Like Pass 3 this is **extract-only** — no source rewrite. It emits a
per-file `<file>.win32k_gdilocks.json` sidecar (schema
`win32k_pass2_v1`) so the rustconv prompt builder can tell the LLM
"`dc` is a `DC_LockDc` RAII handle, its Drop is `DC_UnlockDc`, the
bracket shape is CLEAN_BRACKET" instead of re-deriving the lifecycle in
every translation unit. The `DC_UnlockDc` (218) vs `DC_LockDc` (143)
asymmetry — objects acquired via handle-validation paths, not 1:1
textual Lock — is exactly the per-site shape this records.

CLI: `python -m rustconv gdi-locks [path] [--json] [--write-sidecars]
      [--oracle]`  (mirrors `callout-annotate`).
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from .callout_annotate import (
    _enclosing_func_span,
    _line_of,
    _mask_literals,
)
from .seh_inventory import _in_keepset

# <Prefix>_Lock<Suffix>(  — DC_LockDc, REGION_LockRgn, PATH_LockPath,
# TEXTOBJ_LockText, GDIOBJ_LockObject, COLORSPACEOBJ_LockCS, …
RE_GDI_LOCK = re.compile(r"\b([A-Z][A-Za-z0-9]+)_Lock([A-Za-z0-9]*)\s*\(")


def _unlock_of(prefix: str, suffix: str) -> str:
    """The matching release symbol: first `_Lock` → `_Unlock`, suffix
    preserved (DC_LockDc→DC_UnlockDc, REGION_LockRgn→REGION_UnlockRgn,
    GDIOBJ_LockObject→GDIOBJ_UnlockObject, COLORSPACEOBJ_LockCS→…CS)."""
    return f"{prefix}_Unlock{suffix}"


@dataclass
class GdiLockSite:
    function: str
    func_line: int
    func_end_line: int
    lock_fn: str  # e.g. DC_LockDc
    unlock_fn: str  # inferred release symbol
    lock_line: int
    var: str  # variable the locked object is bound to
    null_guarded: bool  # acquire is NULL-checked before use
    unlock_lines: list[int]  # release sites for `var` within the fn
    shape: str  # CLEAN_BRACKET | MULTI_UNLOCK |
    # BORROWED | NESTED | REVIEW
    signals: list[str] = field(default_factory=list)


def _stmt_lo(masked: str, at: int) -> int:
    """Offset of the start of the statement containing `at` (just past the
    previous `;`/`{`/`}`)."""
    return max(masked.rfind(";", 0, at), masked.rfind("{", 0, at), masked.rfind("}", 0, at)) + 1


def _stmt_before(masked: str, at: int) -> str:
    """The statement the lock call sits in (for `var = Lock(...)` /
    `if (!(var = Lock(...)))`)."""
    return masked[_stmt_lo(masked, at) : at]


def extract_file(path: Path) -> list[GdiLockSite]:
    text = path.read_text(errors="replace")
    masked = _mask_literals(text)
    sites: list[GdiLockSite] = []

    for m in RE_GDI_LOCK.finditer(masked):
        prefix, suffix = m.group(1), m.group(2)
        lock_fn = f"{prefix}_Lock{suffix}"
        unlock_fn = _unlock_of(prefix, suffix)
        open_paren = masked.index("(", m.start())
        # Skip a *definition* of the lock fn itself (its body `{` follows
        # the param list, no assignment precedes it).
        stmt = _stmt_before(masked, m.start())
        vm = re.search(r"([A-Za-z_]\w*)\s*=\s*$", stmt)
        if vm is None:
            # Not `var = Lock(...)` — e.g. the Lock fn definition, or the
            # result is used inline. Record with var "?" for review.
            var = "?"
        else:
            var = vm.group(1)

        fn_name, fn_lo, fn_hi = _enclosing_func_span(text, m.start())
        if var == "?" and fn_name == lock_fn:
            continue  # this *is* the Lock fn definition — not a use site

        lock_line = _line_of(text, m.start())

        # NULL guard, two canonical shapes:
        #  (1) acquire is the if-condition itself —
        #      `if (!(var = Lock(...)))` / `if ((var = Lock(...)) == NULL)`
        #      → the statement start..lock contains `if (`.
        #  (2) `var = Lock(...);` then a following stmt tests `var` for
        #      NULL before first use.
        guard = False
        if var != "?":
            pre = masked[_stmt_lo(masked, m.start()) : m.start()]
            if re.search(r"\bif\s*\(", pre):
                guard = True
            else:
                after = masked[open_paren : open_paren + 240]
                if re.search(rf"\bif\s*\([^)]*\b{re.escape(var)}\b[^)]*\)", after) and re.search(
                    rf"(!\s*{re.escape(var)}\b|"
                    rf"\b{re.escape(var)}\s*==\s*NULL|"
                    rf"\bNULL\s*==\s*{re.escape(var)}\b)",
                    after,
                ):
                    guard = True

        # Release sites for this var inside the enclosing function body.
        fn_text = masked
        # bound the search to [fn_lo line .. fn_hi line] via char offsets
        lo_off = sum(len(ln) + 1 for ln in text.splitlines()[: fn_lo - 1])
        hi_off = sum(len(ln) + 1 for ln in text.splitlines()[:fn_hi])
        body = fn_text[lo_off:hi_off]
        unlock_lines: list[int] = []
        if var != "?":
            for um in re.finditer(rf"\b{re.escape(unlock_fn)}\s*\(\s*{re.escape(var)}\b", body):
                unlock_lines.append(_line_of(text, lo_off + um.start()))

        signals: list[str] = []
        if var == "?":
            shape = "REVIEW"
            signals.append("no_assignment_target")
        elif not unlock_lines:
            shape = "BORROWED"
            signals.append("no_unlock_in_fn")  # transferred/handle-acquired
        elif len(unlock_lines) == 1 and guard:
            shape = "CLEAN_BRACKET"
        elif len(unlock_lines) >= 2:
            shape = "MULTI_UNLOCK"
            signals.append("multi_exit_release")
        else:
            shape = "REVIEW"
        if not guard and var != "?":
            signals.append("no_null_guard")

        sites.append(
            GdiLockSite(
                function=fn_name,
                func_line=fn_lo,
                func_end_line=fn_hi,
                lock_fn=lock_fn,
                unlock_fn=unlock_fn,
                lock_line=lock_line,
                var=var,
                null_guarded=guard,
                unlock_lines=unlock_lines,
                shape=shape,
                signals=signals,
            )
        )
    return sites


def extract_keepset(win32ss_root: Path) -> dict[Path, list[GdiLockSite]]:
    result: dict[Path, list[GdiLockSite]] = {}
    for c in sorted(win32ss_root.rglob("*.c")):
        if not _in_keepset(c):
            continue
        sites = extract_file(c)
        if sites:
            result[c] = sites
    return result


def write_sidecar(path: Path, sites: list[GdiLockSite]) -> Path:
    out = path.with_suffix(path.suffix + ".win32k_gdilocks.json")
    payload = {
        "source": str(path),
        "schema": "win32k_pass2_v1",
        "count": len(sites),
        "sites": [s.__dict__ for s in sites],
    }
    out.write_text(json.dumps(payload, indent=2))
    return out


def _shape_counts(inv: dict[Path, list[GdiLockSite]]) -> dict[str, int]:
    c: dict[str, int] = {}
    for ss in inv.values():
        for s in ss:
            c[s.shape] = c.get(s.shape, 0) + 1
    return c


def print_summary(inv: dict[Path, list[GdiLockSite]]) -> None:
    n = sum(len(v) for v in inv.values())
    print(f"\n{'=' * 64}")
    print(f"Win32k Pass 2 — {n} GDI lock sites across {len(inv)} keep-set files")
    print(f"{'=' * 64}")
    for shape in ("CLEAN_BRACKET", "MULTI_UNLOCK", "BORROWED", "NESTED", "REVIEW"):
        k = _shape_counts(inv).get(shape, 0)
        pct = (k * 100 / n) if n else 0
        print(f"  {shape:<14} {k:>4}  {pct:5.1f}%")
    pairs: dict[str, int] = {}
    for ss in inv.values():
        for s in ss:
            pairs[s.lock_fn] = pairs.get(s.lock_fn, 0) + 1
    print("\n  Lock symbols (sites):")
    for lk, c in sorted(pairs.items(), key=lambda x: -x[1]):
        print(f"    {c:>4}  {lk} → {lk.replace('_Lock', '_Unlock', 1)}")
    top = sorted(inv.items(), key=lambda x: -len(x[1]))[:10]
    print("\n  Hotspot files:")
    for p, ss in top:
        rel = p.as_posix().split("/win32ss/")[-1]
        print(f"    {len(ss):>3}  {rel}")


# --- dcutil.c ground-truth oracle -------------------------------------------
# dcutil.c is a dense, regular DC_LockDc/DC_UnlockDc bracket file. Observed
# directly 2026-05-16: the IntGdiSetBkColor bracket (var `dc`, NULL-guarded,
# single DC_UnlockDc) is the canonical CLEAN_BRACKET.
def run_oracle(dcutil_path: Path) -> int:
    sites = extract_file(dcutil_path)
    print(f"dcutil.c: {len(sites)} GDI lock sites")
    ok = True

    dc_sites = [s for s in sites if s.lock_fn == "DC_LockDc"]
    if len(dc_sites) >= 6:
        print(f"  DC_LockDc sites: {len(dc_sites)} (>=6)  [OK]")
    else:
        print(f"  DC_LockDc sites: expect >=6, got {len(dc_sites)}  [MISMATCH]")
        ok = False

    if all(s.unlock_fn == "DC_UnlockDc" for s in dc_sites):
        print("  unlock inference DC_LockDc→DC_UnlockDc  [OK]")
    else:
        print("  unlock inference  [MISMATCH]")
        ok = False

    bk = [s for s in sites if s.function == "IntGdiSetBkColor"]
    if bk and bk[0].var == "dc" and bk[0].null_guarded and bk[0].shape == "CLEAN_BRACKET":
        print("  IntGdiSetBkColor = CLEAN_BRACKET(dc, guarded)  [OK]")
    else:
        got = f"{bk[0].var}/{bk[0].null_guarded}/{bk[0].shape}" if bk else "absent"
        print(f"  IntGdiSetBkColor: expect dc/True/CLEAN_BRACKET, got {got}  [MISMATCH]")
        ok = False

    # No site should resolve its enclosing function to the lock symbol
    # itself (that would mean we caught a Lock-fn definition as a use).
    selfdef = [s for s in sites if s.function == s.lock_fn]
    if selfdef:
        print(f"  {len(selfdef)} lock-fn-definition false positives  [MISMATCH]")
        ok = False
    else:
        print("  no lock-fn-definition false positives  [OK]")

    print("ORACLE PASS" if ok else "ORACLE FAIL")
    return 0 if ok else 1
