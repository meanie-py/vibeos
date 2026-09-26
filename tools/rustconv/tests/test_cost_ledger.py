import json
import shutil

import pytest

from rustconv.cost_ledger import build_report


def write_bank(bank, events=(), calls=()):
    bank.mkdir(parents=True)
    (bank / "outcomes.jsonl").write_text("".join(json.dumps(e) + "\n" for e in events))
    run = bank / "run-1"
    run.mkdir()
    (run / "meta.json").write_text(
        json.dumps(
            {
                "label": "module",
                "model": "model",
                "target": "module",
                "calls": list(calls),
            }
        )
    )
    for call in calls:
        (run / call["file"]).write_text("fn f() {}")


def event(**fields):
    return {
        "schema": 1,
        "run_id": "run-1",
        "target": "module",
        "model": "model",
        "unit": "f",
        "source_file": "module/f.c",
        "source_sha256": "source",
        "arch": "i386",
        "timestamp": "2026-09-22T00:00:00+00:00",
        "outcome": "generated",
        "input_tokens": 100,
        "output_tokens": 200,
        "cost_usd": 0.12345678,
        "artifact": "/original/run-1/01_f.rs",
        **fields,
    }


def call(**fields):
    return {
        "idx": 1,
        "name": "f",
        "file": "01_f.rs",
        "input_tokens": 100,
        "output_tokens": 200,
        "cost_usd": 0.1235,
        **fields,
    }


def test_copied_bank_and_meta_event_are_not_double_counted(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    write_bank(a, [event()], [call()])
    shutil.copytree(a, b)
    report = build_report([a, b, a])
    assert report["totals"]["records"] == 1
    assert report["totals"]["recorded_estimated_usd"] == 0.12345678
    assert report["normalization"]["copied_outcome_records_removed"] == 1
    assert report["normalization"]["copied_archive_calls_removed"] == 1
    assert len(report["calls"][0]["evidence"]) == 4


def test_reused_preamble_path_joins_by_usage_without_claiming_body_identity(tmp_path):
    bank = tmp_path / "bank"
    events = [
        event(unit="__preamble__", artifact="/run-1/00_preamble.rs"),
        event(
            unit="__preamble__",
            artifact="/run-1/00_preamble.rs",
            source_file="module/g.c",
            input_tokens=101,
        ),
    ]
    calls = [
        call(name="preamble", file="00_preamble.rs"),
        call(name="preamble", file="00_preamble.rs", input_tokens=101),
    ]
    write_bank(bank, events, calls)
    report = build_report([bank])
    assert report["totals"]["records"] == 2
    assert all(r["body_sha256"] is None for r in report["calls"])
    assert all(r["archive_path_reused"] for r in report["calls"])


def test_ambiguous_join_is_loud(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank, [event(), event(source_file="module/g.c")], [call()])
    with pytest.raises(ValueError, match="Ambiguous"):
        build_report([bank])


def test_absent_seal_evidence_is_not_false(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank, [event()], [call()])
    ns = json.loads(build_report([bank])["calls"][0]["namespace"])
    assert ns["seal_wide"] is None


def test_source_changes_and_arch_changes_are_not_retries(tmp_path):
    bank = tmp_path / "bank"
    es = [
        event(outcome="audit_reject", error="wrong", artifact=None),
        event(timestamp="2026-09-22T01:00:00+00:00"),
        event(source_sha256="changed"),
        event(arch="arm64"),
    ]
    write_bank(bank, es)
    report = build_report([bank])
    assert report["repeated_observed_attempts"]["repeats"] == 1
    assert report["repeated_observed_attempts"]["subsequent_nonreject_outcome"] == 1


def test_deterministic_fraction_counts_source_versions_not_replays(tmp_path):
    bank = tmp_path / "bank"
    es = [
        event(outcome="deterministic", cost_usd=None),
        event(outcome="cache_hit"),
        event(outcome="cache_hit", run_id="run-2"),
        event(unit="g", source_sha256="other"),
    ]
    write_bank(bank, es)
    report = build_report([bank])
    assert report["deterministic_by_target"]["module"]["fraction"] == 0.5
    assert report["totals"]["records"] == 1


def test_partial_ledger_and_missing_bank_fail(tmp_path):
    bank = tmp_path / "bank"
    with pytest.raises(ValueError, match="does not exist"):
        build_report([bank])
    write_bank(bank)
    (bank / "outcomes.jsonl").write_text("{")
    with pytest.raises(ValueError, match="Invalid ledger record"):
        build_report([bank])


def test_distinct_archive_calls_cannot_share_one_outcome(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank, [event()], [call(), call()])
    with pytest.raises(ValueError, match="Multiple archive calls"):
        build_report([bank])


def test_usage_conflict_fails_instead_of_double_counting(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank, [event()], [call(input_tokens=999)])
    with pytest.raises(ValueError, match="Conflicting"):
        build_report([bank])


def test_identical_archive_records_at_distinct_indices_are_distinct_calls(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank, [], [call(), call()])
    assert build_report([bank])["totals"]["records"] == 2


def write_attempt(bank, attempt_id="purchase", **fields):
    import hashlib

    path = bank / "attempts" / attempt_id
    path.mkdir(parents=True)
    (path / "request.json").write_text(json.dumps({"schema": 1, "attempt_id": attempt_id}))
    code = b"fn f() {}"
    (path / "candidate.rs").write_bytes(code)
    (path / "result.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "attempt_id": attempt_id,
                "model": "model",
                "success": True,
                "candidate_sha256": hashlib.sha256(code).hexdigest(),
                "input_tokens": 100,
                "output_tokens": 200,
                "cost_usd": 0.12345678,
                "usage_known": True,
                **fields,
            }
        )
    )


def test_immutable_attempt_joins_terminal_outcome_and_counts_intermediate_retry(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank, [event(attempt_id="terminal")], [call(attempt_id="terminal")])
    write_attempt(bank, "terminal")
    write_attempt(bank, "retry", success=False)
    report = build_report([bank])
    assert report["totals"]["records"] == 2
    assert report["totals"]["recorded_estimated_usd"] == 0.24691356
    assert report["normalization"]["immutable_attempts_joined_to_legacy"] == 1


def test_copied_attempt_not_double_counted_and_changed_candidate_is_loud(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank)
    write_attempt(bank)
    shutil.copytree(bank, tmp_path / "copy")
    assert build_report([bank, tmp_path / "copy"])["totals"]["records"] == 1
    (bank / "attempts/purchase/candidate.rs").write_text("changed")
    with pytest.raises(ValueError, match="Changed immutable candidate"):
        build_report([bank])


def test_interrupted_request_is_unknown_not_a_zero_cost_purchase(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank)
    write_attempt(bank, usage_known=False, cost_usd=None)
    request = bank / "attempts/interrupted/request.json"
    request.parent.mkdir()
    request.write_text('{"schema": 1}')
    report = build_report([bank])
    assert report["totals"]["records_with_cost"] == 0
    assert report["normalization"]["incomplete_attempt_requests"] == [str(request)]


def test_accounting_receipt_recovers_recorded_charge_without_repricing(tmp_path):
    bank = tmp_path / "bank"
    write_bank(bank)
    write_attempt(bank, cost_usd=None)
    (bank / "attempts/purchase/accounting.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "attempt_id": "purchase",
                "estimated_usd": "0.125",
            }
        )
    )
    report = build_report([bank])
    assert report["totals"]["recorded_estimated_usd"] == 0.125
    assert any(p.endswith("accounting.json") for p in report["calls"][0]["evidence"])
