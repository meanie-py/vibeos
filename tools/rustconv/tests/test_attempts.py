import json
from decimal import Decimal

import pytest

from rustconv import attempts, batch_runner
from rustconv.spend_guard import Budget, BudgetExceeded


def setup_budget(monkeypatch, tmp_path, cap="5"):
    monkeypatch.setattr(attempts, "RUSTCONV_RESPONSES_DIR", tmp_path / "responses")
    monkeypatch.setenv("RUSTCONV_BUDGET_FILE", str(tmp_path / "budget.json"))
    monkeypatch.setenv("RUSTCONV_BUDGET_USD", cap)
    return Budget(tmp_path / "budget.json", cap)


def result(*, known=True, success=True):
    return batch_runner.TranslationResult(
        "source.c",
        "fn f() {}",
        "deepseek-v4-pro",
        100,
        200,
        success,
        usage_known=known,
    )


def test_budget_refuses_before_backend_contact(monkeypatch, tmp_path):
    setup_budget(monkeypatch, tmp_path, "0.01")

    def forbidden(*args, **kwargs):
        pytest.fail("backend contacted without budget")

    monkeypatch.setattr(batch_runner, "_submit_single_backend", forbidden)
    with pytest.raises(BudgetExceeded):
        batch_runner.submit_single("source", model="deepseek-v4-pro")


def test_every_attempt_preserves_raw_code_and_validation_separately(monkeypatch, tmp_path):
    budget = setup_budget(monkeypatch, tmp_path)

    def fake(*args, **kwargs):
        kwargs["sink"].write("```rust\nfn f() {}\n```")
        return result()

    monkeypatch.setattr(batch_runner, "_submit_single_backend", fake)
    first = batch_runner.submit_single("source", model="deepseek-v4-pro")
    second = batch_runner.submit_single("source", model="deepseek-v4-pro")
    assert first.attempt_id != second.attempt_id
    root = tmp_path / "responses" / "attempts" / first.attempt_id
    first.rust_code = "changed by audit"
    for verdict in ("audit_reject", "audit_pass"):
        attempts.record_validation(first.attempt_id, checker="v1", verdict=verdict)
    assert (root / "candidate.rs").read_text() == "fn f() {}"
    assert (root / "response.partial").read_text().startswith("```rust")
    assert len(list(root.glob("validation-*.json"))) == 2
    assert Decimal(budget.snapshot()["spent_usd"]) == Decimal("0.001848")


def test_unknown_usage_keeps_maximum_reserved_even_on_failure(monkeypatch, tmp_path):
    budget = setup_budget(monkeypatch, tmp_path)
    monkeypatch.setattr(
        batch_runner, "_submit_single_backend", lambda *a, **kw: result(known=False, success=False)
    )
    answer = batch_runner.submit_single("source", model="deepseek-v4-pro")
    assert answer.attempt_id in budget.snapshot()["reservations"]
    assert budget.snapshot()["spent_usd"] == "0"


def test_partial_response_and_reservation_survive_exception(monkeypatch, tmp_path):
    budget = setup_budget(monkeypatch, tmp_path)

    def fail(*args, **kwargs):
        kwargs["sink"].write("partial paid work")
        raise RuntimeError("lost connection")

    monkeypatch.setattr(batch_runner, "_submit_single_backend", fail)
    with pytest.raises(RuntimeError, match="lost connection"):
        batch_runner.submit_single("source", model="deepseek-v4-pro")
    assert len(budget.snapshot()["reservations"]) == 1
    root = tmp_path / "responses" / "attempts"
    assert next(root.glob("*/response.partial")).read_text() == "partial paid work"
    assert not list(root.glob("*/result.json"))


def test_archive_same_label_and_unit_do_not_overwrite(monkeypatch, tmp_path):
    from rustconv import sink

    monkeypatch.setattr(sink, "RUSTCONV_RESPONSES_DIR", tmp_path)
    a = sink.RunArchive("same", "deepseek-v4-pro", "target")
    b = sink.RunArchive("same", "deepseek-v4-pro", "target")
    assert a.dir != b.dir
    one = a.commit(0, "preamble", "first", 1, 2)
    two = a.commit(0, "preamble", "second", 2, 3)
    assert one != two
    assert one.read_text() == "first"
    assert len(json.loads((a.dir / "meta.json").read_text())["calls"]) == 2


def test_deepseek_disables_sdk_retries_and_keeps_usage_on_late_failure(monkeypatch):
    from types import SimpleNamespace

    monkeypatch.setenv("DEEPSEEK_API_KEY", "fake-not-a-key")

    def stream(**kwargs):
        yield SimpleNamespace(
            choices=[], usage=SimpleNamespace(prompt_tokens=123, completion_tokens=456)
        )
        raise RuntimeError("stream ended after usage")

    def client(**kwargs):
        assert kwargs["max_retries"] == 0
        return SimpleNamespace(chat=SimpleNamespace(completions=SimpleNamespace(create=stream)))

    monkeypatch.setattr(batch_runner, "OpenAI", client)
    answer = batch_runner._submit_deepseek("fake", "deepseek-v4-pro")
    assert not answer.success
    assert answer.usage_known
    assert (answer.input_tokens, answer.output_tokens) == (123, 456)


@pytest.mark.parametrize("backend", ["_call_deepseek", "_call_claude"])
def test_direct_review_path_cannot_bypass_cap(monkeypatch, backend):
    from rustconv import semdiff

    monkeypatch.setenv("RUSTCONV_BUDGET_USD", "5")
    with pytest.raises(RuntimeError, match="refusing to bypass"):
        getattr(semdiff, backend)("model", "system", "user")
