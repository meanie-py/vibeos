"""Read-only historical accounting; never imports a model client or changes a bank.

Run with ``python -m rustconv.cost_ledger --worktrees REPO --output report.json``.
Costs are recorded estimates, not invoices. Unknown historical evidence stays unknown.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from collections import Counter, defaultdict
from datetime import datetime, timezone
from decimal import Decimal
from itertools import pairwise
from pathlib import Path
from typing import Any

PAID_OUTCOMES = {"generated", "audit_reject", "api_error"}


def _digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _canonical(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def discover_banks(repo: Path) -> tuple[list[Path], list[str]]:
    """Registered local worktrees, including named backups; no claim about remote hosts."""
    raw = subprocess.check_output(["git", "-C", str(repo), "worktree", "list", "--porcelain", "-z"])
    roots = [Path(p[9:].decode()) for p in raw.split(b"\0") if p.startswith(b"worktree ")]
    banks, absent = [], []
    for root in roots:
        found = sorted(p for p in (root / "tools/rustconv").glob(".responses*") if p.is_dir())
        banks.extend(found)
        if not found:
            absent.append(str(root))
    return banks, absent


def _unit(name: str) -> str:
    return "__preamble__" if name == "preamble" else name


def _namespace(event: dict) -> str:
    return _canonical(
        {k: event.get(k) for k in ("mode", "seal_facts", "seal_calls", "seal_wide", "cache_tag")}
    )


def _identity(event: dict) -> tuple | None:
    fields = ("target", "source_file", "unit", "source_sha256", "arch", "cpp_class")
    if not all(event.get(k) for k in fields[:4]):
        return None
    return tuple(event.get(k) for k in fields)


def _summary(rows: list[dict]) -> dict:
    costs = [r["cost_usd"] for r in rows if r.get("cost_usd") is not None]
    elapsed = [r["elapsed_ms"] for r in rows if r.get("elapsed_ms") is not None]
    return {
        "records": len(rows),
        "records_with_cost": len(costs),
        "recorded_estimated_usd": round(sum(costs), 8),
        "mean_recorded_estimated_usd": round(sum(costs) / len(costs), 8) if costs else None,
        "input_tokens": sum(r.get("input_tokens", 0) for r in rows),
        "output_tokens": sum(r.get("output_tokens", 0) for r in rows),
        "records_with_api_elapsed": len(elapsed),
        "api_elapsed_sum_seconds": round(sum(elapsed) / 1000, 3),
    }


def _groups(rows: list[dict], key: str) -> dict:
    groups: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        groups[str(row.get(key, "unknown"))].append(row)
    return {k: _summary(v) for k, v in sorted(groups.items())}


def build_report(banks: list[Path]) -> dict:
    inputs: list[dict] = []
    events: dict[str, dict] = {}
    metas: dict[str, dict] = {}
    inventory: list[dict] = []
    attempts: dict[str, dict] = {}
    incomplete_attempts: list[str] = []

    def read(path: Path) -> bytes:
        data = path.read_bytes()
        inputs.append({"path": str(path), "sha256": _digest(data), "bytes": len(data)})
        return data

    for bank in sorted({p.resolve() for p in banks}):
        if not bank.is_dir():
            raise ValueError(f"Response bank does not exist: {bank}")
        paths = sorted(bank.glob("*/meta.json"))
        ledger = bank / "outcomes.jsonl"
        inventory.append(
            {"path": str(bank), "meta_files": len(paths), "has_ledger": ledger.exists()}
        )
        if ledger.exists():
            for number, line in enumerate(read(ledger).splitlines(), 1):
                try:
                    event = json.loads(line)
                except ValueError as error:
                    raise ValueError(f"Invalid ledger record {ledger}:{number}: {error}") from error
                if event.get("schema") != 1 or not event.get("outcome"):
                    raise ValueError(f"Unsupported outcome record {ledger}:{number}")
                key = _digest(_canonical(event).encode())
                if key not in events:
                    events[key] = {"event": event, "evidence": []}
                events[key]["evidence"].append(f"{ledger}:{number}")
        for request_path in sorted(bank.glob("attempts/*/request.json")):
            request = json.loads(read(request_path))
            result_path = request_path.with_name("result.json")
            if not result_path.exists():
                incomplete_attempts.append(str(request_path))
                continue
            result = json.loads(read(result_path))
            attempt_id = result["attempt_id"]
            accounting_path = result_path.with_name("accounting.json")
            accounting = json.loads(read(accounting_path)) if accounting_path.exists() else None
            if accounting is not None:
                if accounting.get("schema") != 1 or accounting.get("attempt_id") != attempt_id:
                    raise ValueError(f"Invalid accounting identity: {accounting_path}")
                charge = accounting["estimated_usd"]
                if charge is not None and (not Decimal(charge).is_finite() or Decimal(charge) < 0):
                    raise ValueError(f"Invalid recorded charge: {accounting_path}")
                result = {**result, "cost_usd": float(charge) if charge is not None else None}
            if (
                request.get("schema") != 1
                or result.get("schema") != 1
                or request.get("attempt_id") != attempt_id
            ):
                raise ValueError(f"Invalid attempt identity: {result_path}")
            evidence = [str(request_path), str(result_path)]
            if accounting is not None:
                evidence.append(str(accounting_path))
            candidate = result_path.with_name("candidate.rs")
            if _digest(read(candidate)) != result["candidate_sha256"]:
                raise ValueError(f"Changed immutable candidate: {candidate}")
            if attempt_id in attempts:
                if (
                    attempts[attempt_id]["result"] != result
                    or attempts[attempt_id]["request"] != request
                ):
                    raise ValueError(f"Conflicting immutable attempt: {attempt_id}")
                attempts[attempt_id]["evidence"].extend(evidence)
            else:
                attempts[attempt_id] = {
                    "request": request,
                    "result": result,
                    "evidence": evidence,
                }
        for path in paths:
            meta = json.loads(read(path))
            for index, call in enumerate(meta["calls"]):
                artifact = path.parent / call["file"]
                body_hash = _digest(artifact.read_bytes()) if artifact.is_file() else None
                # Copy identity includes bytes, not just a basename or a function label.
                identity = [path.parent.name, meta["model"], meta["target"], index, call, body_hash]
                key = _digest(_canonical(identity).encode())
                if key not in metas:
                    metas[key] = {
                        "run_id": path.parent.name,
                        "target": meta["target"],
                        "model": meta["model"],
                        "label": meta["label"],
                        **call,
                        "body_sha256": body_hash,
                        "evidence": [],
                    }
                metas[key]["evidence"].append(f"{path}#calls/{index}")

    rows: list[dict] = []
    # Join by run + target + model + artifact basename. Never by function name alone.
    by_artifact: dict[tuple, list[dict]] = defaultdict(list)
    for item in events.values():
        event = item["event"]
        if event["outcome"] not in PAID_OUTCOMES:
            continue
        row = {**event, "namespace": _namespace(event), "evidence": item["evidence"][:]}
        row["unit"] = _unit(row["unit"])
        rows.append(row)
        if event.get("artifact"):
            key = (
                *(event.get(k) for k in ("run_id", "target", "model")),
                Path(event["artifact"]).name,
            )
            by_artifact[key].append(row)
    joined = 0
    matched_rows: set[int] = set()
    for meta in metas.values():
        key = tuple(meta[k] for k in ("run_id", "target", "model", "file"))
        candidates = by_artifact.get(key, [])
        matches = [
            row
            for row in candidates
            if all(row.get(k) == meta.get(k) for k in ("input_tokens", "output_tokens"))
        ]
        if candidates and not matches:
            raise ValueError(f"Conflicting outcome/archive usage: {key}")
        if len(matches) > 1:
            raise ValueError(f"Ambiguous outcome/archive join: {key}")
        if matches:
            row = matches[0]
            if id(row) in matched_rows:
                raise ValueError(f"Multiple archive calls match one outcome: {key}")
            matched_rows.add(id(row))
            row["evidence"].extend(meta["evidence"])
            row["archive_path_reused"] = len(candidates) > 1
            row["body_sha256"] = meta["body_sha256"] if len(candidates) == 1 else None
            joined += 1
        else:
            rows.append({**meta, "unit": _unit(meta["name"]), "namespace": "unknown"})

    # Immutable purchases include intermediate retries absent from terminal outcome rows.
    # An exact attempt id is the join key; equal prompts or bodies are NOT one purchase.
    attempt_joins = 0
    for attempt_id, item in attempts.items():
        result = item["result"]
        matches = [row for row in rows if row.get("attempt_id") == attempt_id]
        if len(matches) > 1:
            raise ValueError(f"Multiple legacy rows for immutable attempt: {attempt_id}")
        if matches:
            row = matches[0]
            if any(row.get(k) != result.get(k) for k in ("input_tokens", "output_tokens")):
                raise ValueError(f"Conflicting immutable attempt usage: {attempt_id}")
            row["evidence"].extend(item["evidence"])
            attempt_joins += 1
        else:
            row = {
                "attempt_id": attempt_id,
                "target": "unknown",
                "unit": "unknown",
                "model": result["model"],
                "namespace": "unknown",
                "source_file": result.get("source_file"),
                "evidence": item["evidence"],
            }
            rows.append(row)
        row.update(
            {
                k: result.get(k)
                for k in ("input_tokens", "output_tokens", "cost_usd", "elapsed_ms", "usage_known")
            }
        )
        row["raw_candidate_sha256"] = result["candidate_sha256"]
        row["api_success"] = result["success"]

    outcomes = [v["event"] for v in events.values()]
    sequences: dict[tuple, list[dict]] = defaultdict(list)
    for event in outcomes:
        identity = _identity(event)
        if identity and event["outcome"] in PAID_OUTCOMES:
            sequences[(*identity, _namespace(event))].append(event)
    repeats = changed = same_reject = after_reject = generated_after_reject = 0
    for sequence in sequences.values():
        sequence.sort(key=lambda e: e["timestamp"])
        repeats += max(0, len(sequence) - 1)
        for previous, current in pairwise(sequence):
            if previous["outcome"] == "audit_reject":
                after_reject += 1
                changed += current["outcome"] != "audit_reject"
                generated_after_reject += current["outcome"] == "generated"
                same_reject += current["outcome"] == "audit_reject" and current.get(
                    "error"
                ) == previous.get("error")

    unit_routes: dict[str, dict[tuple, set[str]]] = defaultdict(lambda: defaultdict(set))
    for event in outcomes:
        identity = _identity(event)
        if identity and event["unit"] != "__preamble__":
            unit_routes[event["target"]][identity].add(event["outcome"])
    deterministic = {}
    for target, units in sorted(unit_routes.items()):
        hits = sum("deterministic" in routes for routes in units.values())
        deterministic[target] = {
            "observed_source_versions": len(units),
            "ever_deterministic": hits,
            "fraction": hits / len(units),
        }

    return {
        "schema": 1,
        "implementation_sha256": _digest(Path(__file__).read_bytes()),
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "scope": "Accessible explicitly selected banks; not a remote-fleet census or invoice",
        "banks": inventory,
        "inputs": inputs,
        "totals": _summary(rows),
        "by_target": _groups(rows, "target"),
        "by_model": _groups(rows, "model"),
        "by_namespace": _groups(rows, "namespace"),
        "outcomes": dict(sorted(Counter(e["outcome"] for e in outcomes).items())),
        "normalization": {
            "unique_immutable_attempts": len(attempts),
            "immutable_attempts_joined_to_legacy": attempt_joins,
            "incomplete_attempt_requests": incomplete_attempts,
            "unique_outcome_records": len(events),
            "unique_archive_calls": len(metas),
            "archive_calls_joined_to_outcomes": joined,
            "archive_calls_without_body": sum(m["body_sha256"] is None for m in metas.values()),
            "copied_outcome_records_removed": sum(len(v["evidence"]) - 1 for v in events.values()),
            "copied_archive_calls_removed": sum(len(v["evidence"]) - 1 for v in metas.values()),
        },
        "repeated_observed_attempts": {
            "identity": (
                "target/source_file/unit/exact source hash/arch/class/namespace; model may differ"
            ),
            "repeats": repeats,
            "subsequent_attempts_after_reject": after_reject,
            "subsequent_nonreject_outcome": changed,
            "identical_reject_message": same_reject,
            "generated_after_reject": generated_after_reject,
            "warning": (
                "Not retry causality: prompt, checker and substrate versions were not recorded"
            ),
        },
        "deterministic_by_target": deterministic,
        "unavailable": [
            "Actual invoice cost (historical costs use token-based configured price estimates)",
            "Historical retry rate/cost: legacy sequential loop recorded only terminal attempts",
            "Complete API-error billing: interrupted streams can lack usage",
            "Stage wall clocks for profile_derive, crate-repair, cargo check and module makespan",
            "Repository-wide deterministic coverage: ledger covers only observed source versions",
            "Semantic correctness, integration status and human repair time per candidate",
        ],
        "calls": rows,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("banks", type=Path, nargs="*")
    parser.add_argument(
        "--worktrees", type=Path, help="discover banks in registered local worktrees"
    )
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    banks, absent = discover_banks(args.worktrees) if args.worktrees else ([], [])
    banks.extend(args.banks)
    if not banks:
        parser.error("No banks found; provide bank paths or --worktrees REPO")
    try:
        report = build_report(banks)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f"cost-ledger: {error}; no report written\n")
    report["worktrees_without_banks"] = absent
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"report": str(args.output), **report["totals"]}, indent=2))


if __name__ == "__main__":
    main()
