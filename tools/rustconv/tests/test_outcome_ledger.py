import json
from concurrent.futures import ThreadPoolExecutor

from rustconv.outcome_ledger import OutcomeLedger, lexical_fingerprint, source_features


def test_lexical_fingerprint_ignores_comments_and_whitespace_but_not_literals():
    a = 'int f(void) { /* why */ return "x y"[0]; }'
    b = 'int f(void){return "x y"[0];}// trailing\n'
    c = 'int f(void){return "xy"[0];}'
    assert lexical_fingerprint(a) == lexical_fingerprint(b)
    assert lexical_fingerprint(a) != lexical_fingerprint(c)


def test_source_features_are_stable_and_cheap():
    features = source_features("int f(int *p) { if (p) return call(p->x); goto out; }")
    assert features["branches"] == 1
    assert features["calls"] == 2  # definition declarator + call
    assert features["gotos"] == 1
    assert features["pointer_arrows"] == 1


def test_concurrent_records_remain_independent_json_lines(tmp_path):
    path = tmp_path / "outcomes.jsonl"
    ledger = OutcomeLedger("run-1", {"model": "cheap", "target": "shell32"}, path)

    def write(index):
        ledger.record(
            unit=f"f{index}",
            source=f"int f{index}(void) {{ return {index}; }}",
            outcome="api_error" if index % 2 else "generated",
            input_tokens=index,
        )

    with ThreadPoolExecutor(max_workers=8) as executor:
        list(executor.map(write, range(40)))

    events = [json.loads(line) for line in path.read_text().splitlines()]
    assert len(events) == 40
    assert {event["unit"] for event in events} == {f"f{i}" for i in range(40)}
    assert all(event["schema"] == 1 for event in events)
    assert all("source_sha256" in event and "features" in event for event in events)
