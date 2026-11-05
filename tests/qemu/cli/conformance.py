"""Compatibility DTOs for the native conformance controller and parsers."""

from __future__ import annotations

import os
from pathlib import Path
from types import SimpleNamespace

from tools import qemu_infra_compat as native
from tools.qemu_infra_runner import run_controller

SUITE = "kmtest"


def _parse_json(value) -> object:
    if isinstance(value, dict):
        return {k: _parse_json(v) for k, v in value.items()}
    if isinstance(value, (list, tuple, set, frozenset)):
        return [_parse_json(v) for v in value]
    return value


def _parse_call(op, **kwargs):
    return native.call(
        "conformance_parse_" + op, {key: _parse_json(value) for key, value in kwargs.items()}
    )


def with_quiet_token(tripwire: str) -> str:
    return _parse_call("quiet_token", tripwire=tripwire)


def read_serial(path: Path) -> str:
    return _parse_call("read_serial", path=str(path))


def baseline_coverage_refusal(results, base_map, *, force: bool) -> str | None:
    return _parse_call("coverage_refusal", results=results, base=base_map, force=force)


def baseline_lowering_note(results, base_map) -> str | None:
    return _parse_call("lowering_note", results=results, base=base_map)


def seed_asserts_into_baseline(baseline_text: str, rounds, parser) -> tuple[str, str]:
    return tuple(
        _parse_call("seed_asserts", text=baseline_text, rounds=rounds, bad=list(parser.BAD))
    )


def _flaky_of(baseline_text: str, name: str) -> set[str]:
    return set(_parse_call("flaky_of", text=baseline_text, name=name))


def merge_baseline_rows(existing_text: str, results, only, fmt):
    formatted = {r["name"]: fmt(r, _flaky_of(existing_text, r["name"])) for r in results}
    dto = _parse_call(
        "merge_rows", text=existing_text, results=results, only=list(only), formatted=formatted
    )
    if dto["added"]:
        print(
            "[conformance] --only added "
            + f"{len(dto['added'])} test(s) the baseline did not carry: {', '.join(dto['added'])}"
        )
    return dto["text"], dto["refusal"]


def round_shape(bugcheck, results: list, done_line, regressions: list, serial_text: str) -> str:
    return _parse_call(
        "round_shape",
        bugcheck=bugcheck,
        results=results,
        done_line=done_line,
        regressions=regressions,
        serial=serial_text,
    )


FARM_DIR = Path.home() / "vibeos-flake-farm"
LEDGER = FARM_DIR / "conformance.jsonl"
_ROUNDS: list[dict] = []
WRITABLE_ROOT_ARCHES = {"amd64", "i386"}
QUIET_TOKEN = "QUIET"


def _options(repo):
    return {
        "suite": SUITE,
        "farm_dir": str(FARM_DIR),
        "ledger": str(LEDGER),
        "baseline_dir": str(repo / "tools/conformance-baseline"),
        "rounds": _ROUNDS,
        "env": dict(os.environ),
    }


def _load_parser(repo):
    from tools.conformance import parse_kmtest_serial

    return parse_kmtest_serial


def media_payload_gap(build_dir: Path) -> str | None:
    return native.call("conformance_media_payload_gap", {"build_dir": str(build_dir)})


def _cmake_cache_bools(cache: Path) -> dict[str, bool]:
    return native.call("conformance_cache_bools", {"path": str(cache)})


def _worktree_provisioning_gaps(repo: Path, base_arch: str) -> list[str]:
    return native.call("conformance_provisioning_gaps", {"arch": base_arch}, root=repo)


def _head_sha(repo: Path) -> str:
    return native.call("farm_git_state", {"env": dict(os.environ)}, root=repo)[0]


def _append_round(rec: dict) -> None:
    native.call("conformance_append_round", {"ledger": str(LEDGER), "record": rec})
    _ROUNDS.append(rec)


def conformance(args, repo: Path) -> int:
    return run_controller(
        "conformance_run",
        args,
        repo,
        command="conformance",
        arch=args.arch.split("-")[0],
        extra=_options(repo),
    )


def _conformance_once(args, repo: Path) -> int:
    dto = SimpleNamespace(**{**vars(args), "repeat": 1, "seed_asserts": False})
    return conformance(dto, repo)


def _seed_asserts_from_rounds(args, repo: Path) -> int:
    return run_controller(
        "conformance_seed_rounds",
        args,
        repo,
        command="conformance",
        arch=args.arch.split("-")[0],
        extra=_options(repo),
    )


def _run_dir(arch: str) -> Path:
    from .artifacts import create_run_dir

    return create_run_dir("conformance", arch)
