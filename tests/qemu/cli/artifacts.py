"""DTO compatibility API for native CLI artifacts, execution and reporting.

The shared Rust core owns process supervision, complete logs, publication and
summary rendering. Python callers may retain their existing Run/Stage objects.
"""

from __future__ import annotations

import os
import subprocess
import threading
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path

from tools import build_infra_compat as native

ARTIFACT_ROOT = Path(os.environ.get("VIBEOS_ARTIFACT_ROOT", "/tmp/vibeos-cli"))
FARM_DIR = Path.home() / "vibeos-flake-farm"
BOOT_COMMANDS = frozenset({"smoke", "gate", "boot"})


def _call(operation, **kwargs):
    return native.call("cli_artifacts_" + operation, kwargs)


def create_run_dir(command: str, arch: str, ts: int | None = None) -> Path:
    return Path(
        _call(
            "create",
            artifact_root=ARTIFACT_ROOT,
            command=command,
            arch=arch,
            ts=ts,
            owner=os.getpid(),
        )["dir"]
    )


def write_interrupted_marker(run_dir: Path, command: str, arch: str) -> None:
    _call("marker", dir=run_dir, command=command, arch=arch, owner=os.getpid())


def is_incomplete(summary: dict) -> bool:
    return bool(summary.get("incomplete"))


def _sha12(path: Path) -> str | None:
    return _call("sha12", path=path)


def _snapshot_kernel(repo: Path, arch: str, mp: bool) -> dict:
    return _call("snapshot_kernel", repo=repo, farm=FARM_DIR, arch=arch, mp=mp)


def preserve_failure(
    run_dir: Path, arch: str, classification: str, repo: Path | None
) -> str | None:
    return _call(
        "preserve_failure",
        dir=run_dir,
        farm=FARM_DIR,
        arch=arch,
        classification=classification,
        repo=repo,
    )


def _head_sha(repo: Path) -> str | None:
    return _call("head", repo=repo)


def _is_dirty(repo: Path) -> bool | None:
    return _call("dirty", repo=repo)


def archive_serial(run_dir: Path, live_log: str | Path | None, extra: dict) -> None:
    result = _call("archive_serial", dir=run_dir, live_log=live_log, extra=extra)
    extra.clear()
    extra.update(result)


def _worktree_of(repo: Path) -> Path:
    return repo.resolve()


@dataclass
class Stage:
    name: str
    exit: int = 0
    seconds: float = 0.0
    status: str = "ok"
    detail: str = ""
    salient: list = field(default_factory=list)


def ledger_append(
    repo: Path | None,
    *,
    cmd: str,
    arch: str,
    verdict: str,
    classification: str = "",
    duration_ms: int = 0,
    extra: dict | None = None,
) -> None:
    try:
        _call(
            "ledger",
            repo=repo,
            env=dict(os.environ),
            cmd=cmd,
            arch=arch,
            verdict=verdict,
            classification=classification,
            duration_ms=duration_ms,
            extra=extra or {},
        )
    except native.NativeError:
        pass  # Outcome telemetry remains best effort; summary publication is mandatory.


class CommandResult(subprocess.CompletedProcess[str]):
    timed_out: bool = False


class Run:
    def __init__(self, command: str, arch: str, ts: int | None = None):
        self.command, self.arch = command, arch
        self.dir = create_run_dir(command, arch, ts)
        self.full_log = self.dir / "full.log"
        self.commands_log = self.dir / "commands.jsonl"
        self.stages: list[Stage] = []
        self.command_timings: list[dict] = []
        self.start = time.monotonic()
        self.verdict, self.classification = "OK", ""
        self.extra: dict = {}
        self.finalize_hooks: list = []
        self._state_lock = threading.Lock()

    def _state(self):
        return {
            "command": self.command,
            "arch": self.arch,
            "verdict": self.verdict,
            "classification": self.classification,
            "stages": [asdict(s) for s in self.stages],
            "extra": self.extra,
        }

    def run(
        self,
        cmd: list[str],
        cwd: Path | None = None,
        env: dict | None = None,
        stage: str | None = None,
        timeout: float | None = None,
    ) -> CommandResult:
        # A private owner lets concurrent gate/build commands execute in parallel;
        # metadata requests never queue behind a running command.
        with native.DedicatedSession("--api-jsonl") as session:
            result = session.call(
                "cli_artifacts_command",
                {
                    "dir": self.dir,
                    "repo": native.REPO,
                    "argv": cmd,
                    "cwd": Path.cwd() if cwd is None else cwd,
                    "env": env,
                    "stage": stage,
                    "timeout": timeout,
                    "run_start": self.start,
                },
            )
        self.command_timings.append(result["timing"])
        value = CommandResult(cmd, result["returncode"], result["stdout"], result["stderr"])
        value.timed_out = result["timed_out"]
        return value

    def log(self, text: str, header: str | None = None) -> None:
        _call("log", dir=self.dir, text=text, header=header)

    def add(self, stage: Stage) -> Stage:
        with self._state_lock:
            self.stages.append(stage)
            self.verdict = _call("stage_verdict", verdict=self.verdict, stage=asdict(stage))
        return stage

    def finalize(self, repo: Path | None = None) -> int:
        hooks, self.finalize_hooks = self.finalize_hooks, []
        for hook in hooks:
            hook()
        duration = time.monotonic() - self.start
        result = _call(
            "finalize",
            dir=self.dir,
            repo=repo,
            farm=FARM_DIR,
            state=self._state(),
            command_timings=self.command_timings,
            elapsed=duration,
        )
        self.extra.update(result["extra"])
        self._ledger_append(repo, result["summary"], duration)
        self._print_summary(duration)
        return result["code"]

    def _ledger_append(self, repo: Path | None, summary: dict, duration: float) -> None:
        from tools import qemu_infra_compat

        extra = qemu_infra_compat.call(
            "cli_report_ledger_context", {"repo": repo, "summary": summary}
        )
        ledger_append(
            repo,
            cmd=self.command,
            arch=self.arch,
            verdict=self.verdict,
            classification=self.classification,
            duration_ms=int(duration * 1000),
            extra=extra,
        )

    def _glyph(self, stage: Stage) -> str:
        # Kept for importing report adapters; the full summary is rendered natively.
        return {"ok": "✓", "warn": "⚠", "fail": "✗", "skip": "·"}.get(stage.status, "?")

    def _print_summary(self, duration: float) -> None:
        print(_call("render", state=self._state(), dir=self.dir, elapsed=duration), end="")
