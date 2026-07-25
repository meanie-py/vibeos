"""Guards against a seal token escaping into an assembled module or the cache."""

import pytest
from rustconv.fact_seal import drop_token_use_items, fact_contract, prepare_sealed_source
from rustconv.response_cache import ResponseCache

SEALED_RUST = "pub unsafe fn f() -> u32 { __RUSTCONV_FACT_0000__ }"


# --- `use <token>;` --------------------------------------------------------


def test_use_of_a_seal_token_is_dropped():
    """A token is a value, never a path. `use __RUSTCONV_FACT_0000__;` unseals to
    `use MAX_PATH;` / `use 100;`, which the assembler hoists to file scope."""
    src = "use __RUSTCONV_FACT_0000__;\nuse __RUSTCONV_CALL_0001__;\nlet x = __RUSTCONV_FACT_0000__;\n"
    assert drop_token_use_items(src) == "let x = __RUSTCONV_FACT_0000__;\n"


def test_real_use_items_are_preserved():
    keep = "use crate::types::*;\nuse core::ffi::c_void;\nuse alloc::vec::Vec;\n"
    assert drop_token_use_items(keep) == keep


def test_dropping_the_only_occurrence_still_fails_loudly():
    """If the `use` was a token's ONLY appearance, the closed-set check must still
    catch it — the scrub must not be able to launder a dropped constant."""
    from rustconv.fact_seal import FactSealError, unseal_facts

    sealed = prepare_sealed_source("int f(void) { return MAX_PATH; }", dialect="c")
    token = sealed.facts[0].placeholder
    with pytest.raises(FactSealError):
        unseal_facts(f"use {token};\npub unsafe fn f() -> u32 {{ 0 }}", sealed)


def test_fact_contract_text_is_unchanged():
    """The contract is hashed into the resume-cache key: rewording it invalidates
    every banked sealed entry and re-charges for translations we already own. This
    behaviour is therefore deterministic (drop_token_use_items), NOT a prompt rule."""
    sealed = prepare_sealed_source("int f(void) { return MAX_PATH; }", dialect="c")
    text = fact_contract(sealed)
    for forbidden in ("use <token>", "never a path", "extern` block"):
        assert forbidden not in text, "do not add rules to fact_contract — it is cache-keyed"


# --- cache poisoning -------------------------------------------------------


def test_cache_refuses_to_store_still_sealed_output(tmp_path, monkeypatch):
    """Caching un-unsealed Rust replays it for $0 forever and assembles a module
    full of `__RUSTCONV_FACT_0000__`, which is not valid Rust."""
    monkeypatch.setattr("rustconv.response_cache.RESPONSES_DIR", tmp_path, raising=False)
    cache = ResponseCache("t")
    cache.dir = tmp_path / "t"
    cache.manifest_path = cache.dir / "manifest.json"
    with pytest.raises(ValueError, match="still-sealed"):
        cache.put("f", "int f(void){}", SEALED_RUST)


def test_cache_treats_an_already_poisoned_entry_as_a_miss(tmp_path):
    """An entry written before the guard existed must self-heal, not replay."""
    cache = ResponseCache("t2")
    cache.dir = tmp_path / "t2"
    cache.dir.mkdir(parents=True)
    cache.manifest_path = cache.dir / "manifest.json"
    from rustconv.response_cache import _hash

    (cache.dir / "f.rs").write_text(SEALED_RUST)
    cache._manifest = {"f": {"file": "f.rs", "hash": _hash("body"), "contract": _hash("")}}
    assert cache.get("f", "body", "") is None, "poisoned entry must MISS so it re-translates"


def test_cache_still_returns_clean_output(tmp_path):
    cache = ResponseCache("t3")
    cache.dir = tmp_path / "t3"
    cache.dir.mkdir(parents=True)
    cache.manifest_path = cache.dir / "manifest.json"
    from rustconv.response_cache import _hash

    (cache.dir / "f.rs").write_text("pub unsafe fn f() -> u32 { MAX_PATH }")
    cache._manifest = {"f": {"file": "f.rs", "hash": _hash("body"), "contract": _hash("")}}
    assert cache.get("f", "body", "") == "pub unsafe fn f() -> u32 { MAX_PATH }"
