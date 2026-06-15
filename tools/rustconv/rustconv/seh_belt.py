"""Pass 1c: Belt-and-suspenders SEH wrapper deletion (Pass-0 category (d)).

The (d) BELT_SUSPENDERS sites are `_SEH2_TRY { user write; } _SEH2_EXCEPT
{ handler; } _SEH2_END;` blocks with **no `ProbeFor*` in the TRY body**.
Under DUMMY_PSEH the EXCEPT body is dead code on all 3 arches — the
kernel cannot recover from a fault inside the TRY. So at runtime:

- write succeeds → Status retains its pre-TRY value (the handler that
  would have overwritten it is dead anyway)
- write faults    → kernel bugchecks (same as if the wrapper weren't there)

Removing the wrapper preserves runtime behavior exactly. The win: the
rustconv translator no longer has to reason about a dead SEH frame per
file, and the Rust translation is plain Rust (no `unsafe { let _ = ... }`
SEH carcass).

Engineering guards (mirror Pass 1 / Pass 1b):
- **Live re-classification**, not stale sidecar line numbers. Pass 1
  lowering drifted many (d) site lines (~2-8 lines on the files we
  spot-checked) so we trust the AST, not the sidecar.
- **Faithful**: replacement = `_dedent_to(span.try_body, indent)` — the
  body text is byte-preserved minus one level of nesting. Handler body
  is dropped (dead code under DUMMY_PSEH); no logic synthesis.
- **Brace balance preserved** — verified per-file before write.
- **Idempotent**: a re-run finds zero (d) spans and is a no-op.
- **`--apply` opt-in**, default is `--dry-run` with unified diff.

Reuses primitives from seh_rewrite.py (_find_seh_spans, _dedent_to) and
seh_inventory.py (classify_span, _in_keepset).
"""

from __future__ import annotations

import difflib
import re
from dataclasses import dataclass, field
from pathlib import Path

from .seh_inventory import Category, _in_keepset, classify_span
from .seh_rewrite import SehSpan, _dedent_to, _find_seh_spans


@dataclass
class BeltResult:
    text: str
    rewritten: int = 0
    skipped: list[tuple[int, str]] = field(default_factory=list)
    sites: list[dict] = field(default_factory=list)


def _brace_counts(text: str) -> tuple[int, int, int, int]:
    """Return ({, }, (, )) counts ignoring string/char/comment content."""
    t = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    t = re.sub(r"//[^\n]*", " ", t)
    t = re.sub(r'"(?:\\.|[^"\\])*"', '""', t)
    t = re.sub(r"'(?:\\.|[^'\\])*'", "''", t)
    return t.count("{"), t.count("}"), t.count("("), t.count(")")


def _balance_delta_preserved(orig: str, new: str) -> bool:
    """The pre-existing `{`/`}` and `(`/`)` deltas must survive the edit.
    Absolute balance can be nonzero (e.g. desktop.c has 1 extra `{` inside
    `#if 0` / macros that the literal stripper doesn't see) — what we
    require is that we didn't *change* that delta."""
    ao, ac, bo, bc = _brace_counts(orig)
    ao2, ac2, bo2, bc2 = _brace_counts(new)
    return (ao - ac) == (ao2 - ac2) and (bo - bc) == (bo2 - bc2)


def rewrite_text(text: str, src_name: str = "<text>") -> BeltResult:
    """Delete every (d)-class SEH wrapper in `text`, keeping the TRY body."""
    spans = _find_seh_spans(text)
    # filter to BELT_SUSPENDERS via standalone classifier (no cross-site;
    # the rewriter doesn't need it — (d) plain and (d)+relies_on_prior_probe
    # take the same action under DUMMY_PSEH)
    targets: list[SehSpan] = []
    for sp in spans:
        if sp.is_finally:
            continue
        cat = classify_span(sp.try_body, sp.handler_body, sp.is_finally)
        if cat == Category.BELT_SUSPENDERS:
            targets.append(sp)

    if not targets:
        return BeltResult(text=text)

    # apply in reverse so earlier offsets stay valid.
    # Substitute from beginning-of-line so the body's re-indent is the
    # only leading whitespace (avoids doubling: indent_before_sp.start
    # + indent_added_by_dedent_to).
    new_text = text
    for sp in reversed(targets):
        bol = new_text.rfind("\n", 0, sp.start) + 1
        indent = new_text[bol : sp.start]
        body = _dedent_to(sp.try_body, indent)
        new_text = new_text[:bol] + body + new_text[sp.end :]

    if not _balance_delta_preserved(text, new_text):
        # rewrite changed the brace/paren delta = corruption; refuse
        return BeltResult(
            text=text,
            rewritten=0,
            skipped=[(0, f"{src_name}: brace/paren delta changed by rewrite — refusing")],
        )

    res = BeltResult(text=new_text, rewritten=len(targets))
    for sp in targets:
        res.sites.append(
            {"line": sp.line, "try_len": len(sp.try_body), "handler_len": len(sp.handler_body)}
        )
    return res


def rewrite_file(path: Path) -> BeltResult:
    return rewrite_text(path.read_text(errors="replace"), src_name=path.name)


# -------- oracle (synthetic, deterministic) --------

_ORACLE_INPUT = """\
NTSTATUS Foo(PVOID UserPtr)
{
    NTSTATUS Status = STATUS_SUCCESS;

    _SEH2_TRY
    {
        RtlMoveMemory(Local, UserPtr, sizeof(Local));
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = _SEH2_GetExceptionCode();
    }
    _SEH2_END;

    return Status;
}

NTSTATUS Bar(PVOID UserPtr)
{
    NTSTATUS Status;
    _SEH2_TRY
    {
        ProbeForRead(UserPtr, 4, 1);
        Local = *(PULONG)UserPtr;
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = _SEH2_GetExceptionCode();
    }
    _SEH2_END;
    return Status;
}
"""

_ORACLE_EXPECTED = """\
NTSTATUS Foo(PVOID UserPtr)
{
    NTSTATUS Status = STATUS_SUCCESS;

    RtlMoveMemory(Local, UserPtr, sizeof(Local));

    return Status;
}

NTSTATUS Bar(PVOID UserPtr)
{
    NTSTATUS Status;
    _SEH2_TRY
    {
        ProbeForRead(UserPtr, 4, 1);
        Local = *(PULONG)UserPtr;
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = _SEH2_GetExceptionCode();
    }
    _SEH2_END;
    return Status;
}
"""


def run_oracle() -> int:
    """Returns 0 on pass, 1 on fail. The 'Bar' span has a probe → not (d) →
    untouched; only 'Foo' (paranoid bulk-copy, no probe) is deleted."""
    res = rewrite_text(_ORACLE_INPUT, src_name="<oracle>")
    if res.rewritten != 1:
        print(f"oracle FAIL: expected 1 rewrite, got {res.rewritten}")
        return 1
    if res.text != _ORACLE_EXPECTED:
        print("oracle FAIL: text mismatch")
        print(
            "".join(
                difflib.unified_diff(
                    _ORACLE_EXPECTED.splitlines(True), res.text.splitlines(True), "expected", "got"
                )
            )
        )
        return 1
    # idempotency
    res2 = rewrite_text(res.text, src_name="<oracle-rerun>")
    if res2.rewritten != 0:
        print(f"oracle FAIL: re-run not idempotent ({res2.rewritten} hits)")
        return 1
    print("oracle PASS (1 rewrite, idempotent, probe-bearing untouched)")
    return 0


def iter_keepset(win32ss_root: Path):
    for c in sorted(win32ss_root.rglob("*.c")):
        if _in_keepset(c):
            yield c
