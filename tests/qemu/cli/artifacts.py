"""Durable artifact directory + token-frugal summary (plan §6).

Every command run gets ${VIBEOS_ARTIFACT_ROOT:-/tmp/vibeos-cli}/<ts>-<cmd>-<arch>/
with full.log (raw stage output under headers), summary.json (the machine-true
verdict — re-readable, never scraped from stdout), commands.jsonl, and any
serial/debugcon/screenshot captured. stdout gets only a bounded summary.
"""

from __future__ import annotations

import json
import os
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path

ARTIFACT_ROOT = Path(os.environ.get("VIBEOS_ARTIFACT_ROOT", "/tmp/vibeos-cli"))


@dataclass
class Stage:
    name: str
    exit: int = 0
    seconds: float = 0.0
    status: str = "ok"  # ok | warn | fail | skip
    detail: str = ""  # one-line, shown in summary
    salient: list = field(default_factory=list)  # key lines for the summary


class Run:
    """One command execution: owns the artifact dir, the full log, and the
    structured summary."""

    def __init__(self, command: str, arch: str, ts: int | None = None):
        self.command = command
        self.arch = arch
        ts = ts if ts is not None else int(time.time())
        # ts passed in so callers (and tests) control it; default real time.
        stamp = time.strftime("%Y%m%d-%H%M%S", time.localtime(ts))
        self.dir = ARTIFACT_ROOT / f"{stamp}-{command}-{arch}"
        self.dir.mkdir(parents=True, exist_ok=True)
        self.full_log = self.dir / "full.log"
        self.commands_log = self.dir / "commands.jsonl"
        self.stages: list[Stage] = []
        self.start = time.monotonic()
        self.verdict = "OK"
        self.classification = ""
        self.extra: dict = {}  # arbitrary verdict fields (flake rate, frame, …)

    # --- subprocess with full-log capture + command journal ---------------
    def run(
        self,
        cmd: list[str],
        cwd: Path | None = None,
        env: dict | None = None,
        stage: str | None = None,
        timeout: float | None = None,
    ) -> subprocess.CompletedProcess:
        t0 = time.monotonic()
        try:
            cp = subprocess.run(
                cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout
            )
            out, err, rc = cp.stdout, cp.stderr, cp.returncode
        except subprocess.TimeoutExpired as e:
            # text=True yields str, but TimeoutExpired types stdout/stderr as
            # str|bytes|None — coerce defensively so the full-log writes are str.
            def _as_str(s: str | bytes | None) -> str:
                return s.decode(errors="replace") if isinstance(s, bytes) else (s or "")

            out, err, rc = _as_str(e.stdout), _as_str(e.stderr), 124
            cp = subprocess.CompletedProcess(cmd, 124, out, err)
        except OSError as e:
            out, err, rc = "", str(e), 127
            cp = subprocess.CompletedProcess(cmd, 127, "", str(e))
        dt = time.monotonic() - t0
        with open(self.full_log, "a") as f:
            f.write(f"\n=== {stage or cmd[0]}: {' '.join(cmd)} — exit {rc}, {dt:.1f}s ===\n")
            if out:
                f.write(out)
            if err:
                f.write("\n--- stderr ---\n" + err)
        with open(self.commands_log, "a") as f:
            f.write(
                json.dumps({"cmd": cmd, "cwd": str(cwd or ""), "exit": rc, "seconds": round(dt, 2)})
                + "\n"
            )
        return cp

    def log(self, text: str, header: str | None = None) -> None:
        with open(self.full_log, "a") as f:
            if header:
                f.write(f"\n=== {header} ===\n")
            f.write(text + ("\n" if not text.endswith("\n") else ""))

    def add(self, stage: Stage) -> Stage:
        self.stages.append(stage)
        if stage.status == "fail":
            self.verdict = "FAIL"
        elif stage.status == "warn" and self.verdict == "OK":
            self.verdict = "WARN"
        return stage

    # --- finalize: write summary.json, print bounded summary --------------
    def finalize(self, repo: Path | None = None) -> int:
        dur = time.monotonic() - self.start
        summary = {
            "run_id": self.dir.name,
            "command": self.command,
            "arch": self.arch,
            "verdict": self.verdict,
            "classification": self.classification,
            "duration_ms": int(dur * 1000),
            "stages": [s.__dict__ for s in self.stages],
            "artifacts": {"full_log": str(self.full_log)},
            **self.extra,
        }
        (self.dir / "summary.json").write_text(json.dumps(summary, indent=2))
        self._print_summary(dur)
        return 0 if self.verdict in ("OK", "WARN") else 1

    def _glyph(self, st: Stage) -> str:
        return {"ok": "✓", "warn": "⚠", "fail": "✗", "skip": "·"}.get(st.status, "?")

    def _print_summary(self, dur: float) -> None:
        head = f"{self.command} {self.arch}  —  {self.verdict}"
        if self.classification:
            head += f": {self.classification}"
        mins = f"{int(dur) // 60}m{int(dur) % 60:02d}s" if dur >= 60 else f"{dur:.0f}s"
        print(f"{head}   ({mins})")
        line = "  " + "   ".join(
            f"{self._glyph(s)} {s.name}"
            + (f"({s.detail})" if s.detail and s.status in ("warn", "fail") else "")
            for s in self.stages
        )
        if self.stages:
            print(line)
        for s in self.stages:
            for sal in s.salient[:4]:
                print(f"  > {sal}")
        print(f"full log:  {self.full_log}")
        for k in ("serial_log", "live_log", "screenshot"):
            if k in self.extra:
                print(f"{k.replace('_', ' '):<10} {self.extra[k]}")
