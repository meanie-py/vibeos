import json

import pytest

from rustconv import pilot


def test_type_names_are_not_expression_facts():
    source = 'struct HEADER { int value; }; static_assert(sizeof(HEADER) == 4, "size");'
    sealed = pilot.prepared_source(source)
    assert "struct HEADER" in sealed.text
    assert all(f.original != "HEADER" for f in sealed.facts)
    assert any(f.original == "4" for f in sealed.facts)


def test_existing_purchase_refuses_before_payment(monkeypatch, tmp_path):
    bank = tmp_path / "bank"
    path = bank / "host-pilots/txt2nls/purchase.json"
    path.parent.mkdir(parents=True)
    path.write_text("{}")
    monkeypatch.setattr(pilot, "discover_banks", lambda _: ([bank], []))
    with pytest.raises(ValueError, match="Prior txt2nls work"):
        pilot.prior_output_audit(tmp_path)


def test_replay_does_not_rescue_missing_tokens_by_spelling(monkeypatch, tmp_path):
    from dataclasses import asdict

    sealed = pilot.prepared_source("x = 10;")
    (tmp_path / "preparation.json").write_text(json.dumps({"sealed": asdict(sealed)}))
    (tmp_path / "purchase-result.json").write_text(
        json.dumps({"rust_code": "x = 10;", "attempt_id": None})
    )
    with pytest.raises(ValueError, match="Fact tokens missing"):
        pilot.replay(tmp_path)
    assert not list(tmp_path.glob("candidate-*.rs"))


@pytest.mark.parametrize("code,stream", [("fn f() {}", ""), ("", "partial")])
def test_empty_retry_refuses_any_existing_code(monkeypatch, tmp_path, code, stream):
    monkeypatch.setattr(pilot, "RUSTCONV_RESPONSES_DIR", tmp_path)
    directory = tmp_path / "pilot"
    directory.mkdir()
    (directory / "purchase.json").write_text("{}")
    (directory / "purchase-result.json").write_text(
        json.dumps(
            {
                "attempt_id": "id",
                "success": False,
                "rust_code": code,
            }
        )
    )
    attempt = tmp_path / "attempts/id"
    attempt.mkdir(parents=True)
    (attempt / "response.partial").write_text(stream)
    with pytest.raises(ValueError, match="never repurchased"):
        pilot.retry_empty(directory)
    assert not (directory / "nonthinking").exists()
