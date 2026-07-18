import pytest

from rustconv.fact_seal import (
    FactSealError,
    canonicalize_hresult_predicates,
    prepare_sealed_source,
    seal_facts,
    unseal_facts,
)


def test_hresult_predicates_lower_without_touching_literals_or_longer_macros():
    source = (
        'if (SUCCEEDED(hr)) return S_OK; /* FAILED(x) */ TRACE("SUCCEEDED(x)");\n'
        "if (FAILED(call(1))) return E_FAIL; "
        "if (FAILED_UNEXPECTEDLY(hr)) return hr;"
    )
    out = canonicalize_hresult_predicates(source)
    assert "if (((hr) >= 0))" in out
    assert "if (((call(1)) < 0))" in out
    assert "FAILED_UNEXPECTEDLY(hr)" in out
    assert 'TRACE("SUCCEEDED(x)")' in out


def test_seals_each_fact_occurrence_but_not_types_calls_or_literals():
    source = (
        "HRESULT hr = S_OK; if (FAILED(hr)) return E_FAIL; x = 0x20u + 7; "
        "TRACE(\"S_OK 99\"); WCHAR c = L'8';"
    )
    sealed = seal_facts(source)
    originals = [fact.original for fact in sealed.facts]
    assert originals == ["S_OK", "E_FAIL", "0x20u", "7"]
    assert "HRESULT" in sealed.text
    assert "FAILED(hr)" in sealed.text
    assert 'TRACE("S_OK 99")' in sealed.text
    assert "L'8'" in sealed.text


def test_prepare_canonicalizes_before_sealing_generated_zero():
    sealed = prepare_sealed_source("if (SUCCEEDED(hr)) return S_OK;")
    assert "SUCCEEDED" not in sealed.text
    assert [fact.original for fact in sealed.facts] == ["0", "S_OK"]


def test_boolean_macros_are_values_owned_by_fact_sealing():
    sealed = seal_facts("Call(TRUE); Call(FALSE);")
    assert [fact.original for fact in sealed.facts] == ["TRUE", "FALSE"]


def test_unseal_requires_closed_token_set_and_allows_required_repetition():
    sealed = seal_facts("return S_OK + 7;")
    rust = f"unsafe fn body() -> HRESULT {{ {sealed.text} }}"
    restored = unseal_facts(rust, sealed)
    assert "return S_OK + 7;" in restored

    with pytest.raises(FactSealError, match="missing"):
        unseal_facts("unsafe fn body() {}", sealed)
    duplicated = unseal_facts(rust + "\n// " + sealed.facts[0].placeholder, sealed)
    assert duplicated.count(sealed.facts[0].original) >= 2
    with pytest.raises(FactSealError, match="invented"):
        unseal_facts(rust + " __RUSTCONV_FACT_9999__", sealed)


def test_unseal_recovers_exact_original_values_only_from_executable_code():
    sealed = seal_facts("return 0 + 0 + S_OK;")
    restored = unseal_facts("unsafe fn body() -> HRESULT { return 0 + 0 + S_OK; }", sealed)
    assert "return 0 + 0 + S_OK" in restored

    with pytest.raises(FactSealError, match="missing"):
        unseal_facts('unsafe fn body() { /* 0 + S_OK */ TRACE("0"); }', sealed)


def test_model_hresult_predicate_can_be_canonicalized_before_fact_recovery():
    sealed = prepare_sealed_source("HRESULT f() { return SUCCEEDED(hr); }")
    model = "unsafe fn f() -> bool { SUCCEEDED(hr) }"
    restored = unseal_facts(canonicalize_hresult_predicates(model), sealed)
    assert "((hr) >= 0)" in restored
