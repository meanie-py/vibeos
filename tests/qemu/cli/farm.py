"""Compatibility DTOs for the native flake-farm controller."""

from __future__ import annotations

import os
from pathlib import Path

from tools import test_infra_compat as native
from tools.test_infra_runner import run_controller

FARM_DIR = Path.home() / "vibeos-flake-farm"
CORPUS = FARM_DIR / "corpus.jsonl"


def _options():
    return {"farm_dir": str(FARM_DIR), "corpus": str(CORPUS), "env": dict(os.environ)}


def _git_state(repo: Path) -> tuple[str, bool]:
    return tuple(native.call("farm_git_state", _options(), root=repo))


def _extract_evidence(run_dir: Path, salient: list[str]) -> dict:
    return native.call("farm_extract_evidence", {"run_dir": str(run_dir), "salient": salient})


def _preserve_failure(run_dir: Path, rec: dict) -> str | None:
    return native.call(
        "farm_preserve_failure", {**_options(), "run_dir": str(run_dir), "record": rec}
    )


def _append(rec: dict) -> None:
    native.call("farm_append", {**_options(), "record": rec})


def farm(args, repo: Path) -> int:
    return run_controller("farm_run", args, repo, command="farm", arch="all", extra=_options())


def farm_report(args, repo: Path) -> int:
    return run_controller(
        "farm_report", args, repo, command="farm-report", arch="all", extra=_options()
    )
