"""Win32k Pre-Translation Pass 1b — LLM probe-lowering for the ledger.

Pass 1 (`seh_rewrite.py`) mechanically lowered the 97 `(a)` probe sites
it could prove safe and *correctly bailed* on 64 (the
`win32k_pass1_manual_followup.json` ledger): they need a context-
dependent decision the deterministic tool refuses to make —

  - 47  non-swallow handler, **no `NTSTATUS Status` lvalue in scope**,
        handler doesn't transfer control. Lowering needs a `Status`
        declaration injected (where? reuse an existing NTSTATUS local?)
        and the post-block body success-guarded.
  - 15  multi-probe, no `Status` lvalue — the short-circuit chain needs
        a `Status` lvalue.
  -  2  brace-less conditional probe (`if (c) _SEH2_TRY …`) — needs
        re-bracing before lowering.

This stage hands each *enclosing function* (deduped — one prompt covers
all its ledger spans + one `Status` injection) to the LLM with the
**exact canonical shapes `seh_rewrite._build_replacement` emits** as the
contract, so the output is consistent with the deterministic Pass-1
already in the tree. It is still a C→C transform (NOT a translation):
the function comes back C, validated structurally, spliced by offset.

Cost discipline (memory `feedback_translation_philosophy`): every raw
response is cached to `tools/rustconv/out/seh_llm_lower_cache.json`
keyed by (file, function, source-hash). Re-runs reuse the cache and
never re-spend; a changed function invalidates only its own entry.

CLI: `python -m rustconv seh-llm-lower [--apply] [--model M]
      [--only FILE] [--oracle]`.
"""

from __future__ import annotations

import difflib
import hashlib
import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from .batch_runner import submit_single
from .callout_annotate import _mask_literals
from .config import REACTOS_ROOT, TOOLS_DIR
from .seh_rewrite import RE_INCLUDE_W32K, RE_TRY, _find_seh_spans

LEDGER = REACTOS_ROOT / "win32ss/win32k_pass1_manual_followup.json"
CACHE = TOOLS_DIR / "out" / "seh_llm_lower_cache.json"

SYSTEM_PROMPT = """\
You lower ReactOS win32k C structured-exception probe blocks to a
status-returning shim. This is a C→C transform, NOT a translation.
Output ONLY the single rewritten C function inside one ```c fence —
no prose, no extra functions.

The shim (already declared via <safe_user_io.h>, already included):

    NTSTATUS NTAPI SafeProbeForRead (const volatile VOID*, SIZE_T, ULONG);
    NTSTATUS NTAPI SafeProbeForWrite(volatile VOID*,       SIZE_T, ULONG);

It validates [Addr,Addr+Len) is user-range + aligned and RETURNS an
NTSTATUS (STATUS_SUCCESS / STATUS_ACCESS_VIOLATION /
STATUS_DATATYPE_MISALIGNMENT). It NEVER raises and NEVER touches the
page (DUMMY_PSEH: a touch-fault is uncatchable, so validate-only is
correct and matches the Rust target).

RULES — apply mechanically, change NOTHING else:

1. Each `_SEH2_TRY { ProbeForRead/Write(P,L,A); BODY } _SEH2_EXCEPT(...)
   { HANDLER } _SEH2_END;` becomes a probe call whose failure path is
   the EXCEPT handler and whose BODY runs ONLY on probe success.
2. If the function has NO `NTSTATUS` status local in scope, DECLARE one
   (`NTSTATUS Status;`) with the other locals. If an NTSTATUS local
   already exists (any name) reuse it — do not add a second.
3. Canonical shapes (emit EXACTLY these — they match the deterministic
   Pass-1 already in this tree):

   a. swallow handler (`NOTHING`/empty/just sets a status):
        if (NT_SUCCESS(SafeProbeForRead(P, L, A)))
        {
            BODY            // only on success
        }

   b. non-swallow, status local in scope, handler has side effects:
        Status = SafeProbeForRead(P, L, A);
        if (!NT_SUCCESS(Status))
        {
            HANDLER         // exact side effects, _SEH2_GetExceptionCode()
        }                   //   -> Status, _SEH2_YIELD(X) -> X
        else
        {
            BODY
        }

   c. non-swallow, handler is ONLY `Status = _SEH2_GetExceptionCode();`:
        Status = SafeProbeForRead(P, L, A);
        if (NT_SUCCESS(Status))
        {
            BODY
        }

   d. multi-probe in one TRY: short-circuit so a later probe cannot
      overwrite an earlier failure, BODY guarded on overall success:
        Status = SafeProbeForRead(P1, L1, A1);
        if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(P2, L2, A2);
        if (!NT_SUCCESS(Status)) { HANDLER } else { BODY }

4. `_SEH2_GetExceptionCode()` -> the status variable. `_SEH2_YIELD(X)`
   -> `X` (e.g. `_SEH2_YIELD(goto cleanup;)` -> `goto cleanup;`).
   `_SEH2_LEAVE` -> the equivalent post-block control flow.
5. Use ProbeForWrite->SafeProbeForWrite, ProbeForRead->SafeProbeForRead.
   Keep the EXACT (P, L, A) args. Multi-char/typed probes
   (ProbeForReadUlong etc.): expand to SafeProbeForRead(P, sizeof(T),
   __alignof(T)) preserving the original type.
6. Preserve every non-probe statement, comment, return value, NTSTATUS
   constant and control-flow path byte-for-byte. Do NOT add asserts,
   logging, TODOs, or `/* ... */` elisions. Do NOT reorder code.
7. If a `_SEH2_TRY` body contains a non-probe CALL (not memcpy/Rtl*
   copy/logging), it is NOT a pure probe site — leave that block
   unchanged and lower only the pure ones. Better to under-rewrite
   than to change semantics.
8. Brace-less `if (cond) _SEH2_TRY ...`: add the `{ }` braces around
   the lowered statement so control flow is preserved.

Return the WHOLE function, every line, lowered, in one ```c block.
"""


@dataclass
class Unit:
    file: Path
    func: str
    start: int  # char offset of function start in file text
    end: int  # char offset just past function close `}`
    reasons: list[str] = field(default_factory=list)
    span_lines: list[int] = field(default_factory=list)


def _func_offsets(text: str, pos: int) -> tuple[str, int, int]:
    """(name, start_off, end_off) of the function enclosing `pos` — the
    same brace-depth scan callout_annotate uses, but returning char
    offsets so we can splice the rewritten function back exactly."""
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
    if func_open == -1:
        return "?", 0, len(text)
    # start: walk back over signature to the line after the previous
    # top-level `}` / `;` / file start.
    s = func_open
    bdepth = 0
    while s > 0:
        ch = masked[s - 1]
        if ch == "}" and bdepth == 0:
            break
        if ch == ";" and bdepth == 0:
            break
        if ch == ")":
            bdepth += 1
        elif ch == "(":
            bdepth -= 1
        s -= 1
    while s < func_open and masked[s] in " \t\r\n":
        s += 1
    # name: identifier before the signature `(`
    j = func_open - 1
    while j >= 0 and masked[j] in " \t\r\n":
        j -= 1
    pd = 0
    while j >= 0:
        if masked[j] == ")":
            pd += 1
        elif masked[j] == "(":
            pd -= 1
            if pd == 0:
                break
        j -= 1
    nm = re.search(r"([A-Za-z_]\w*)\s*$", masked[:j])
    name = nm.group(1) if nm else "?"
    # end: matching `}` of the body
    d = 0
    k = func_open
    while k < len(masked):
        if masked[k] == "{":
            d += 1
        elif masked[k] == "}":
            d -= 1
            if d == 0:
                return name, s, k + 1
        k += 1
    return name, s, len(text)


def load_units(only: str | None = None) -> list[Unit]:
    ledger = json.loads(LEDGER.read_text())
    by_file: dict[str, list[dict]] = {}
    for s in ledger["sites"]:
        by_file.setdefault(s["file"], []).append(s)

    units: dict[tuple[str, int], Unit] = {}
    for rel, sites in by_file.items():
        if only and only not in rel:
            continue
        path = REACTOS_ROOT / "win32ss" / rel
        if not path.exists():
            continue
        text = path.read_text(errors="replace")
        # ledger lines were recorded against the post-Pass-1 tree; map
        # each to the SEH span that still starts on/near that line.
        spans = _find_seh_spans(text)
        for site in sites:
            ln = site["line"]
            sp = min(spans, key=lambda s: abs(s.line - ln), default=None)
            if sp is None:
                continue
            name, fs, fe = _func_offsets(text, sp.start)
            key = (rel, fs)
            u = units.get(key)
            if u is None:
                u = Unit(file=path, func=name, start=fs, end=fe)
                units[key] = u
            u.reasons.append(site["reason"])
            u.span_lines.append(sp.line)
    return list(units.values())


def _load_cache() -> dict:
    if CACHE.exists():
        return json.loads(CACHE.read_text())
    return {}


def _save_cache(c: dict) -> None:
    CACHE.parent.mkdir(parents=True, exist_ok=True)
    CACHE.write_text(json.dumps(c, indent=2))


def _ckey(rel: str, func: str, src: str) -> str:
    h = hashlib.sha1(src.encode()).hexdigest()[:12]
    return f"{rel}::{func}::{h}"


def _extract_c(resp: str) -> str | None:
    lines = resp.splitlines()
    out: list[str] = []
    infence = False
    seen = False
    for ln in lines:
        st = ln.strip()
        if not infence and (st.startswith("```c") or st == "```C" or st.startswith("```cpp")):
            infence = True
            seen = True
            continue
        if infence and st == "```":
            break
        if infence:
            out.append(ln)
    if not seen:
        return None
    return "\n".join(out).strip("\n") + "\n"


def _balanced(src: str) -> bool:
    m = _mask_literals(src)
    return m.count("{") == m.count("}") and m.count("(") == m.count(")")


_STUB_MARKERS = (
    "/* ... */",
    "/*...*/",
    "// ...",
    "TODO",
    "FIXME",
    "unimplemented",
    "UNIMPLEMENTED",
    "<rest of",
    "remaining code",
    "rest of the function",
    "elided",
)


def validate(orig: str, new: str, unit: Unit) -> str | None:
    """Return a rejection reason, or None if the rewrite is safe to
    apply. Guards the known LLM C→C failure modes (memories
    feedback_rustconv_stubbed_bodies / _dropped_fn_calls)."""
    if not new or len(new) < 0.5 * len(orig):
        return "output too short (possible truncation/stub)"
    if not _balanced(new):
        return "unbalanced braces/parens"
    low = new
    for mk in _STUB_MARKERS:
        if mk in low and mk not in orig:
            return f"stub/elision marker introduced: {mk!r}"
    if "SafeProbeFor" not in new:
        return "no SafeProbeFor* call emitted"
    # token-superset: every identifier used in the ORIGINAL (minus the
    # SEH macros) must still appear in the rewrite (catches dropped
    # calls / silently stubbed bodies).
    SEH = {
        "_SEH2_TRY",
        "_SEH2_EXCEPT",
        "_SEH2_FINALLY",
        "_SEH2_END",
        "_SEH2_LEAVE",
        "_SEH2_YIELD",
        "_SEH2_GetExceptionCode",
        "_SEH2_GetExceptionInformation",
        "ProbeForRead",
        "ProbeForWrite",
        "EXCEPTION_EXECUTE_HANDLER",
    }

    def ids(s: str) -> set[str]:
        return set(re.findall(r"\b[A-Za-z_]\w+\b", _mask_literals(s)))

    missing = (ids(orig) - SEH) - ids(new)
    # allow typed-probe expansion (ProbeForReadUlong family) to vanish
    missing = {m for m in missing if not m.startswith("ProbeFor")}
    if missing:
        return f"dropped identifiers: {sorted(missing)[:8]}"
    # the targeted SEH must be gone (we only flag if NOTHING was lowered)
    if RE_TRY.search(_mask_literals(new)) and not RE_TRY.search(_mask_literals(orig)):
        return "introduced _SEH2_TRY"
    return None


@dataclass
class Outcome:
    unit: Unit
    applied: bool
    reason: str
    diff: str = ""


def run(
    apply: bool = False, model: str = "", only: str | None = None, units: list[Unit] | None = None
) -> list[Outcome]:
    units = units if units is not None else load_units(only)
    cache = _load_cache()
    outcomes: list[Outcome] = []
    # group by file so we apply all splices to one buffer, high→low.
    by_file: dict[Path, list[Unit]] = {}
    for u in units:
        by_file.setdefault(u.file, []).append(u)

    for path, us in by_file.items():
        text = path.read_text(errors="replace")
        edits: list[tuple[int, int, str]] = []
        for u in us:
            orig = text[u.start : u.end]
            rel = str(path.relative_to(REACTOS_ROOT / "win32ss"))
            ck = _ckey(rel, u.func, orig)
            resp = cache.get(ck)
            if resp is None:
                reasons = "; ".join(sorted(set(u.reasons)))
                prompt = (
                    f"File: {rel}  Function: {u.func}\n"
                    f"Ledger reasons for this function's probe block(s): "
                    f"{reasons}\n\n"
                    f"Lower the probe SEH block(s) per the rules. Return "
                    f"the WHOLE function in one ```c block.\n\n"
                    f"```c\n{orig}\n```\n"
                )
                r = submit_single(prompt, model=model, system_prompt=SYSTEM_PROMPT)
                if not r.success:
                    outcomes.append(Outcome(u, False, f"API error: {r.error}"))
                    continue
                resp = r.rust_code
                cache[ck] = resp
                _save_cache(cache)
            new = _extract_c(resp)
            if new is None:
                outcomes.append(Outcome(u, False, "no ```c fence in reply"))
                continue
            why = validate(orig, new, u)
            if why:
                outcomes.append(Outcome(u, False, f"rejected: {why}"))
                continue
            diff = "".join(
                difflib.unified_diff(
                    orig.splitlines(True),
                    new.splitlines(True),
                    f"a/{rel}:{u.func}",
                    f"b/{rel}:{u.func}",
                )
            )
            edits.append((u.start, u.end, new))
            outcomes.append(Outcome(u, True, "ok", diff))

        if apply and edits:
            for s, e, rep in sorted(edits, key=lambda x: -x[0]):
                text = text[:s] + rep + text[e:]
            # Match the deterministic Pass-1: ensure the shim is declared.
            if "safe_user_io.h" not in text and "SafeProbeFor" in text:
                m = RE_INCLUDE_W32K.search(text)
                if m:
                    text = text[: m.end()] + "\n#include <safe_user_io.h>" + text[m.end() :]
            path.write_text(text)

    return outcomes


def print_report(outcomes: list[Outcome], apply: bool) -> int:
    ok = [o for o in outcomes if o.applied]
    bad = [o for o in outcomes if not o.applied]
    print(f"\n{'=' * 64}")
    print(
        f"Pass 1b LLM probe-lowering — {len(ok)} ready, "
        f"{len(bad)} held back  ({'APPLIED' if apply else 'DRY-RUN'})"
    )
    print(f"{'=' * 64}")
    for o in ok:
        print(f"  ✓ {o.unit.file.name}:{o.unit.func} (lines {sorted(set(o.unit.span_lines))})")
    if bad:
        print("\n  held back (left in ledger, never applied):")
        for o in bad:
            print(f"  ✗ {o.unit.file.name}:{o.unit.func} — {o.reason}")
    return 0 if not bad else (0 if ok else 1)


# --- oracle: dry-run a single known ledger function, structural-only ---------
def run_oracle() -> int:
    units = load_units()
    if not units:
        print("oracle: ledger produced no units  [MISMATCH]")
        return 1
    print(
        f"oracle: ledger → {len(units)} dedup'd functions across "
        f"{len({u.file for u in units})} files"
    )
    ok = True
    # every unit must resolve a real function span containing its spans
    for u in units:
        txt = u.file.read_text(errors="replace")
        seg = txt[u.start : u.end]
        if not (
            u.func != "?"
            and seg.startswith(tuple("ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz"))
            and seg.rstrip().endswith("}")
            and u.func in seg
        ):
            print(f"  bad span: {u.file.name}:{u.func}  [MISMATCH]")
            ok = False
            break
    else:
        print(
            f"  all {len(units)} units have a well-formed function span containing their name  [OK]"
        )
    total = sum(len(u.reasons) for u in units)
    if total == 64:
        print("  ledger site coverage: 64/64  [OK]")
    else:
        print(f"  ledger site coverage: {total}/64  [MISMATCH]")
        ok = False
    print("ORACLE PASS" if ok else "ORACLE FAIL")
    return 0 if ok else 1
