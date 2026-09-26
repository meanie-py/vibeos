"""Measure wall-clock steps independently of time-service status reports."""

from __future__ import annotations

import itertools
import json
import subprocess
import time
from pathlib import Path

from .artifacts import Run, Stage

IMPLICIT = Path("/sys/module/hv_utils/parameters/timesync_implicit")


def command_output(argv: list[str]) -> dict:
    result = subprocess.run(argv, capture_output=True, text=True, timeout=20)
    return {
        "argv": argv,
        "exit": result.returncode,
        "stdout": result.stdout.strip(),
        "stderr": result.stderr.strip(),
    }


def environment() -> dict:
    return {
        "release": Path("/proc/sys/kernel/osrelease").read_text().strip(),
        "hyperv_implicit": IMPLICIT.read_text().strip() if IMPLICIT.exists() else None,
        "timesyncd": command_output(
            ["systemctl", "show", "systemd-timesyncd", "-p", "ActiveState", "-p", "UnitFileState"]
        ),
        "ntp": command_output(["timedatectl", "timesync-status"]),
    }


def steps(samples: list[dict]) -> list[dict]:
    # Reject pairs descheduled between the two clock reads. Preserve them in
    # raw evidence; do not mistake a scheduling pause for a clock step.
    valid = [s for s in samples if s["read_latency_ns"] <= 2_000_000]
    return [
        {
            "at_seconds": right["elapsed_seconds"],
            "step_ms": (right["offset_ns"] - left["offset_ns"]) / 1_000_000,
        }
        for left, right in itertools.pairwise(valid)
    ]


def probe(seconds: int, destination: Path) -> dict:
    start = time.monotonic()
    samples = []
    while True:
        before = time.monotonic_ns()
        wall = time.time_ns()
        after = time.monotonic_ns()
        elapsed = time.monotonic() - start
        samples.append(
            {
                "elapsed_seconds": elapsed,
                "wall_ns": wall,
                "monotonic_ns": (before + after) // 2,
                "offset_ns": wall - (before + after) // 2,
                "read_latency_ns": after - before,
            }
        )
        if elapsed >= seconds:
            break
        time.sleep(min(0.1, seconds - elapsed))
    changes = steps(samples)
    result = {
        "duration_seconds": time.monotonic() - start,
        "max_step_ms": max((abs(s["step_ms"]) for s in changes), default=0),
        "steps_over_50ms": [s for s in changes if abs(s["step_ms"]) > 50],
        "valid_pairs": len(changes),
        "samples": samples,
    }
    destination.write_text(json.dumps(result, indent=2) + "\n")
    return result


def command(args, repo: Path) -> int:
    run = Run("host-clock", "host")
    try:
        if not 1 <= args.seconds <= 3600:
            raise ValueError("--seconds must be between 1 and 3600")
        before = environment()
        run.extra["before"] = before
        if args.fix_wsl:
            if "microsoft" not in before["release"].lower() or before["hyperv_implicit"] != "Y":
                raise RuntimeError("Refusing NTP change: active Hyper-V implicit sync is required")
            # Scope the repair to the documented conflict. Never disable time
            # synchronization on a native host, nor claim Windows NTP was fixed.
            result = command_output(
                ["sudo", "-n", "systemctl", "disable", "--now", "systemd-timesyncd.service"]
            )
            run.extra["repair"] = result
            if result["exit"]:
                raise RuntimeError(f"Could not disable competing WSL NTP: {result['stderr']}")
            print(
                "[host-clock] WSL NTP disabled; Hyper-V remains the time source. "
                "Recording 10s of settling, then the requested observation.",
                flush=True,
            )
            run.extra["settling"] = {
                k: v for k, v in probe(10, run.dir / "settling.json").items() if k != "samples"
            }
        print(
            f"[host-clock] measuring wall vs monotonic for {args.seconds}s; evidence: {run.dir}",
            flush=True,
        )
        measured = probe(args.seconds, run.dir / "samples.json")
        run.extra["measurement"] = {k: v for k, v in measured.items() if k != "samples"}
        run.extra["after"] = environment()
        failed = measured["steps_over_50ms"] or measured["valid_pairs"] < 2
        run.add(
            Stage(
                "clock",
                status="fail" if failed else "ok",
                detail=f"largest wall-clock step {measured['max_step_ms']:.3f}ms "
                f"over {measured['duration_seconds']:.1f}s",
            )
        )
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        run.add(Stage("clock", status="fail", detail=str(exc)))
    return run.finalize(repo)


def add_parser(sub) -> None:
    parser = sub.add_parser(
        "host-clock", help="measure host clock steps; diagnose WSL time conflicts"
    )
    parser.add_argument(
        "--seconds",
        type=int,
        default=65,
        help="observation window (default 65s covers two 32s NTP polls)",
    )
    parser.add_argument(
        "--fix-wsl",
        action="store_true",
        help="disable competing systemd-timesyncd only when Hyper-V sync is active",
    )
    parser.set_defaults(fn=command)
