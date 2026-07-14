"""Performance regression tests for the stitch → postprocess assembly path.

Background (2026-07-13): `rustconv translate` hung forever on large user-mode
files — confirmed on `dll/win32/ole32/storage32.c` (10.3K LOC, 252 functions)
and `dll/win32/ole32/moniker.c`. Every per-function API call completed and the
Rust landed in `.responses/`, then the process printed `Stitching N functions...`
and spun at 100% CPU on one core until killed.

That hang is *expensive*: the operator kills the run and hand-assembles the
`.responses/*.rs` pieces, which bypasses `postprocess()` entirely — and with it
the ABI enforcement and the (already paid-for) translated preamble. Every
downstream defect on those two files traces back to this hang.

Root cause is NOT the stitcher (it assembles storage32 in ~10 ms). It is
catastrophic regex backtracking in `postprocess.drop_pseh_macro_externs`
(and its identical twin `drop_gdiobj_macro_externs`), triggered *by* the
stitcher's output: `_merge_extern_blocks` correctly emits one big
`extern "system" { … }` block per linkage, and as soon as that block holds a
non-`fn` declaration (e.g. `static IID_IPersistStream: IID;`) alongside 40+ fn
decls, the pattern

    extern\\s+"([^"]+)"\\s*\\{\\s*\\n
    ((?:[ \\t]*(?:#\\[[^\\]]+\\]\\s*\\n[ \\t]*)?(?:pub\\s+)?fn\\s+[A-Za-z_]\\w*\\b
      (?:\\s*\\([^)]*\\))?\\s*(?:->\\s*[^;{]+)?\\s*;\\s*\\n)+)
    [ \\t]*\\}\\s*\\n?

cannot reach the closing `}`. The repeated group is ambiguous (`[^;{]+` in the
return-type part matches newlines, so one repetition can span several decl
lines), so the engine explores exponentially many ways to partition the
preceding decls before giving up. Measured on the real storage32 extern block:
18 decls = 13 ms, 22 = 214 ms, 26 = 5.1 s, 30 = 37 s — a clean 2^n. The real
block has 46. It never returns.

These tests pin both halves:
  * the stitcher stays linear, and
  * its output does not detonate the postprocess extern-block passes.

See `tools/rustconv/STITCHER_PATCH_NOTES.md` for the proposed fix to
postprocess.py (owned by another lane — a linear line-scan replacing the
ambiguous regex).
"""

from __future__ import annotations

import subprocess
import sys
import textwrap
import time
from pathlib import Path

import pytest

RUSTCONV_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(RUSTCONV_ROOT))

from rustconv.callgraph import FunctionDep
from rustconv.catalog import Symbol
from rustconv.config import TARGETS
from rustconv.postprocess import drop_pseh_macro_externs
from rustconv.resolver import ImportGroup, ResolveResult
from rustconv.scanner import ScanResult
from rustconv.stitcher import stitch_file

# Generous so it can't flake on a loaded host. The pre-fix code blows past this
# by many orders of magnitude (it never terminates at all).
BUDGET_S = 30.0

N_FUNCS = 260


def _synthetic_function(i: int) -> str:
    """A realistic per-function LLM output: uses + an extern block + a body.

    Mirrors what the ole32 translations actually look like — each function
    forward-declares its file-local callees and its imports in its own
    `extern "system"` block, and the stitcher merges all 260 of them into one.

    The LAST function also declares a `static` (an interface GUID) — exactly
    like storage32.c, whose merged `extern "system"` block holds 46 fn decls
    and then `static IID_IPersistStream: IID;`. That trailing non-fn decl is
    what the old block regex could never get past: the repeated fn-decl group
    can't reach the closing `}`, so it backtracks over every way to partition
    the *preceding* fn decls. The `static` must come after a long run of fn
    decls for the blowup to be exponential rather than trivially-failing.
    """
    body = "\n".join(
        f"        let tmp_{j} = (param_a as usize).wrapping_add({j});\n"
        f"        acc = acc.wrapping_add(tmp_{j} as u32);"
        for j in range(40)
    )
    guid = f"    static IID_IPersistStream: IID;\n" if i == N_FUNCS - 1 else ""
    return f"""use ntkrnl_sys::ddk::{{HRESULT, ULONG}};
use core::ffi::c_void;

extern "system" {{
    fn HelperCallee{i}(this: *mut c_void, index: ULONG, entry: *mut c_void) -> HRESULT;
    fn OtherCallee{i}(a: *mut c_void, b: ULONG) -> HRESULT;
{guid}}}

#[no_mangle]
pub unsafe extern "system" fn SynthFunc{i}(param_a: *mut c_void, param_b: ULONG) -> HRESULT {{
    let mut acc: u32 = 0;
{body}
    if acc == 0 {{
        return HelperCallee{i}(param_a, param_b, core::ptr::null_mut());
    }}
    OtherCallee{i}(param_a, param_b)
}}
"""


def _synthetic_inputs():
    """(functions, deps, scan, file_resolve, preamble) — the exact 5 args
    `__main__`'s translate path hands to `stitch_file`."""
    functions = {f"SynthFunc{i}": _synthetic_function(i) for i in range(N_FUNCS)}
    deps = [FunctionDep(name=f"SynthFunc{i}", line_start=i * 50) for i in range(N_FUNCS)]

    resolve = ResolveResult()
    resolve.imports["ddk"] = ImportGroup(
        module="ddk",
        symbols=[Symbol(name=n, kind="type", module="ddk") for n in ("HRESULT", "ULONG", "IID")],
    )
    scan = ScanResult(path=Path("synthetic.c"))
    preamble = "pub type IID = [u8; 16];\npub const SYNTH_FLAG: u32 = 1;\n"
    return functions, deps, scan, resolve, preamble


def test_stitch_file_is_linear_on_a_large_file():
    """260 functions must stitch in well under a second, let alone 30s.

    Guards the stitcher itself (the merge/dedup passes are all set-based and
    linear today — this keeps them that way).
    """
    functions, deps, scan, resolve, preamble = _synthetic_inputs()

    t0 = time.monotonic()
    out = stitch_file(functions, deps, scan, resolve, target=TARGETS["ntoskrnl"], preamble=preamble)
    elapsed = time.monotonic() - t0

    assert elapsed < BUDGET_S, f"stitch_file took {elapsed:.1f}s for {N_FUNCS} functions"
    # Sanity: every paid function body survived the stitch (nothing silently dropped).
    for i in range(N_FUNCS):
        assert f"fn SynthFunc{i}(" in out, f"SynthFunc{i} missing from stitched output"
    # And the merged extern block is the shape that used to detonate postprocess:
    # one block per linkage, a long run of fn decls followed by a non-fn decl.
    assert 'extern "system" {' in out
    assert "static IID_IPersistStream: IID;" in out


def test_stitched_output_does_not_detonate_pseh_extern_pass(tmp_path):
    """THE regression: the stitcher's own output through the extern-block pass.

    Pre-fix this never returns — `drop_pseh_macro_externs`' block regex cannot
    reach the closing `}` of a mixed fn/static extern block, and backtracks
    exponentially over the 260 merged fn decls that precede the `static`.

    It runs in a **subprocess** on purpose: a catastrophically backtracking
    regex is *uninterruptible* in CPython (`re.sub` never returns to the eval
    loop, so no signal handler — not even SIGINT — can fire; that's precisely
    why the operator has to `kill -9` a wedged `rustconv translate`). An
    in-process timing assertion could therefore never be reached: the whole
    test session would hang forever instead of failing. A killable child is
    the only way to turn this into a *test failure*.
    """
    functions, deps, scan, resolve, preamble = _synthetic_inputs()
    stitched = stitch_file(
        functions, deps, scan, resolve, target=TARGETS["ntoskrnl"], preamble=preamble
    )
    src = tmp_path / "stitched.rs"
    src.write_text(stitched)

    child = textwrap.dedent(
        f"""
        import sys
        sys.path.insert(0, {str(RUSTCONV_ROOT)!r})
        from rustconv.postprocess import drop_pseh_macro_externs
        code = open({str(src)!r}).read()
        out = drop_pseh_macro_externs(code)
        print("IDENTICAL" if out == code else "CHANGED")
        """
    )
    t0 = time.monotonic()
    try:
        proc = subprocess.run(
            [sys.executable, "-c", child],
            timeout=BUDGET_S,
            capture_output=True,
            text=True,
        )
    except subprocess.TimeoutExpired:
        pytest.fail(
            f"drop_pseh_macro_externs did not finish within {BUDGET_S:.0f}s on a "
            f"{len(stitched):,}-char stitched file ({N_FUNCS} functions) — "
            f"catastrophic regex backtracking. This is the bug that wedges "
            f"`rustconv translate` on storage32.c/moniker.c. "
            f"Fix: tools/rustconv/STITCHER_PATCH_NOTES.md"
        )
    elapsed = time.monotonic() - t0
    assert proc.returncode == 0, f"child failed: {proc.stderr}"
    assert elapsed < BUDGET_S
    # Nothing to drop here (no _SEH2_* names) — output must be untouched.
    assert "IDENTICAL" in proc.stdout, f"pass mutated a clean file: {proc.stdout}"


def test_pseh_extern_pass_still_drops_seh2_from_a_mixed_block():
    """The mixed fn/static block must still get its _SEH2_* decls stripped.

    This is the case the old regex could never even reach (it hung first), so
    it is untested today: a block that holds a `static` alongside the fns.
    """
    code = (
        'extern "system" {\n'
        "    fn _SEH2_TRY();\n"
        "    fn StgStreamImpl_Construct(parent: *mut c_void) -> HRESULT;\n"
        "    static IID_IPersistStream: IID;\n"
        "    fn _SEH2_GetExceptionCode() -> u32;\n"
        "}\n"
    )
    t0 = time.monotonic()
    out = drop_pseh_macro_externs(code)
    assert time.monotonic() - t0 < BUDGET_S

    head = out.split("// drop")[0]
    assert "fn _SEH2_TRY" not in head
    assert "_SEH2_GetExceptionCode" not in head
    assert "StgStreamImpl_Construct" in out  # real import kept
    assert "static IID_IPersistStream: IID;" in out  # non-fn decl kept
    assert 'extern "system" {' in out  # block survives, original ABI
