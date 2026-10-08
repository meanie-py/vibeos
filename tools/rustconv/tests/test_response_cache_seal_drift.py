"""Changing a seal RULE must not re-bill work already paid for and accepted.

The contract text ends in a bare enumeration of the tokens it minted, which is
fully derived from (body, seal rules). Hashing it meant that excluding one macro
from the call seal voided every banked entry in the file — 6,800 lines of
accepted, already-unsealed Rust. What the hash must still catch is a CAPABILITY
change (the provided-RAII API a pre-capability output could not have used) and a
seal that got *looser*.
"""

import pytest

from rustconv import response_cache
from rustconv.response_cache import ResponseCache

FACT_ENVELOPE = (
    "\n## Deterministic fact envelope\nPreserve EACH token at least once.\n"
    "Tokens (2): __RUSTCONV_FACT_0000__, __RUSTCONV_FACT_0001__\n"
)
CALL_ENVELOPE = (
    "\n## Deterministic call envelope\nPreserve EACH token exactly once.\n"
    "Call tokens (2): __RUSTCONV_CALL_0000__, __RUSTCONV_CALL_0001__\n"
)
# Same rules, one fewer token — what excluding a macro from the seal produces.
CALL_ENVELOPE_SMALLER = CALL_ENVELOPE.replace(
    "Call tokens (2): __RUSTCONV_CALL_0000__, __RUSTCONV_CALL_0001__",
    "Call tokens (1): __RUSTCONV_CALL_0000__",
)
RAII = "\n## Provided local RAII type: CFoo (deterministic — already lowered)\n```rust\nx\n```\n"

BODY = "int f(void) { return Helper(IID_PPV_ARG(I, &p)); }"
RUST = "unsafe fn f() -> i32 { Helper(&IID_I, &mut p) }"


def _cache(tmp_path, monkeypatch, name="sub__stem"):
    monkeypatch.setattr(response_cache, "RESPONSES_DIR", tmp_path)
    return ResponseCache(name)


def test_shrinking_the_seal_set_keeps_the_banked_entry(tmp_path, monkeypatch):
    cache = _cache(tmp_path, monkeypatch)
    sealed = frozenset({"call:Helper", "call:IID_PPV_ARG"})
    cache.put(BODY, BODY, RUST, FACT_ENVELOPE + CALL_ENVELOPE, sealed)

    # The macro is no longer sealed: fewer tokens, fewer sealed names. Still a HIT.
    assert (
        cache.get(BODY, BODY, FACT_ENVELOPE + CALL_ENVELOPE_SMALLER, frozenset({"call:Helper"}))
        == RUST
    )


def test_growing_the_seal_set_misses(tmp_path, monkeypatch):
    """An entry sealed LESS tightly let a name through the model unguarded."""
    cache = _cache(tmp_path, monkeypatch)
    cache.put(BODY, BODY, RUST, FACT_ENVELOPE + CALL_ENVELOPE, frozenset({"call:Helper"}))
    assert (
        cache.get(
            BODY,
            BODY,
            FACT_ENVELOPE + CALL_ENVELOPE,
            frozenset({"call:Helper", "call:Extra"}),
        )
        is None
    )


def test_a_new_capability_block_still_misses(tmp_path, monkeypatch):
    """The CDrvDefExt stale-replay bug: normalization must not resurrect it."""
    cache = _cache(tmp_path, monkeypatch)
    sealed = frozenset({"call:Helper"})
    cache.put(BODY, BODY, RUST, FACT_ENVELOPE + CALL_ENVELOPE, sealed)
    assert cache.get(BODY, BODY, FACT_ENVELOPE + CALL_ENVELOPE + RAII, sealed) is None


def test_legacy_entry_survives_a_seal_change_but_not_a_capability_one(tmp_path, monkeypatch):
    """Entries banked before normalization stamped the FULL text; recover them."""
    cache = _cache(tmp_path, monkeypatch)
    cache.put(BODY, BODY, RUST, FACT_ENVELOPE + CALL_ENVELOPE)
    # Simulate the pre-normalization stamp: full-text hash, no "sealed", no "caps".
    entry = cache._manifest[BODY]
    entry["contract"] = response_cache._hash(FACT_ENVELOPE + CALL_ENVELOPE)
    entry.pop("sealed", None)
    entry.pop("caps", None)

    assert cache.get(BODY, BODY, FACT_ENVELOPE + CALL_ENVELOPE_SMALLER) == RUST
    assert cache.get(BODY, BODY, FACT_ENVELOPE + CALL_ENVELOPE + RAII) is None


def test_legacy_entry_from_an_UNSEALED_run_never_replays_into_a_sealed_one(tmp_path, monkeypatch):
    cache = _cache(tmp_path, monkeypatch)
    cache.put(BODY, BODY, RUST, "")
    cache._manifest[BODY]["contract"] = response_cache._hash("")
    cache._manifest[BODY].pop("sealed", None)
    cache._manifest[BODY].pop("caps", None)  # exercise the LEGACY path, not the caps one
    assert cache.get(BODY, BODY, FACT_ENVELOPE + CALL_ENVELOPE) is None
    # ...but an unsealed entry replayed into an unsealed run is unchanged, so it hits.
    assert cache.get(BODY, BODY, "") == RUST


def test_rejected_output_is_kept_for_forensics_and_never_replayed(tmp_path, monkeypatch):
    """A reject is paid artifact too, but it must never become a $0 cache hit."""
    cache = _cache(tmp_path, monkeypatch)
    cache.put_rejected("f", "unsafe fn f() { /* dropped a call */ }", "envelope broken")

    kept = (cache.dir / "__rejected__" / "f.rs").read_text()
    assert "envelope broken" in kept
    assert "dropped a call" in kept
    assert cache.get("f", BODY, FACT_ENVELOPE) is None


def test_a_multi_line_reject_reason_stays_out_of_the_body(tmp_path, monkeypatch):
    """hotplug: the audit message spanned two lines and its second leaked into the body."""
    cache = _cache(tmp_path, monkeypatch)
    cache.put_rejected("f", "unsafe fn f() {}", "line one\n        line two -- reason")
    assert cache.get_rejected("f") == "unsafe fn f() {}"
    # A file written by the old, broken header still reads back clean.
    (cache.dir / "__rejected__" / "f.rs").write_text(
        "// REJECTED by the deterministic audit\n// line one\n   line two\n\nunsafe fn f() {}"
    )
    assert cache.get_rejected("f") == "unsafe fn f() {}"


def test_a_changed_c_body_may_be_bought_again(tmp_path, monkeypatch):
    # setuplib SetupMoveFile: banked from a body the scanner had cut at `_SEH2_TRY`
    cache = _cache(tmp_path, monkeypatch)
    cache.put("F", "old truncated body", "fn F() {}")
    with pytest.raises(ValueError, match="did not replay"):
        cache.refuse_repurchase("F", body_text="old truncated body")
    cache.refuse_repurchase("F", body_text="the whole body, FINALLY and all")
