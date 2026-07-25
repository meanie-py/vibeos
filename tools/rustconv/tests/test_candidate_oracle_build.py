"""The candidate oracle must BUILD, not merely CHECK.

`cargo check` stops before codegen, and `unconditional_panic` is a MIR-level
deny-by-default lint that only fires during codegen. So `check` silently PASSES a
constant out-of-bounds index — the bindgen flex-array shape (`[T; 1]` indexed past
0) that the iphlpapi lane shipped twice before the lesson was written down:
"a translation is not validated until it has been BUILT, not merely checked."

Measured on this tree with a lib whose only defect is `(*a).entries[1]` on a
`[u32; 1]`:
    cargo check -> exit 0, no diagnostics
    cargo build -> error: index out of bounds: the length is 1 but the index is 1
and through the oracle itself:
    --check-only -> "1/1 candidates compile on every arch"
    default      -> "0/1 ... unconditional_panic: this operation will panic at runtime"
"""

import inspect

from rustconv import candidate_oracle


def test_oracle_defaults_to_build():
    sig = inspect.signature(candidate_oracle.run_oracle)
    assert sig.parameters["check_only"].default is False, (
        "cargo build must be the default; check is the weaker opt-in"
    )


def test_check_only_is_reachable_but_explicit():
    """The fast path still exists for iteration — it just cannot be the default."""
    src = inspect.getsource(candidate_oracle.run_oracle)
    assert '"check" if check_only else "build"' in src


def test_the_reason_is_documented_at_the_call_site():
    """Someone will try to 'speed this up' by reverting to check. The docstring has
    to tell them what that costs."""
    doc = candidate_oracle.run_oracle.__doc__ or ""
    assert "unconditional_panic" in doc
    assert "codegen" in doc
