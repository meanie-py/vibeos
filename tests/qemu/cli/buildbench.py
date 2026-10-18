"""Reproducible build + lint resource benchmark for vibeos-cli."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import threading
import time
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from . import buildpolicy, hostjobs, hostlock

ARCHES = ("i386", "amd64", "arm64")
SAMPLE_PERIOD = 0.2
ENV_KEYS = (
    "VIBEOS_BUILD_CACHE",
    "VIBEOS_HOST_JOBS",
    "VIBEOS_BUILD_CORE_RESERVE",
    "VIBEOS_BUILD_TIMEOUT",
    "CARGO_BUILD_JOBS",
    "MAKEFLAGS",
    "NINJAFLAGS",
    "CCACHE_DIR",
    "CCACHE_MAXSIZE",
    "SCCACHE_DIR",
    "SCCACHE_CACHE_SIZE",
    "RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "VIBEOS_HOST_MEMORY_MIB",
    "VIBEOS_HOST_MEMORY_HEADROOM_MIB",
    "VIBEOS_HOST_JOB_MEMORY_MIB",
    "VIBEOS_HOST_JOB_SLOTS",
    "VIBEOS_HOST_LINK_SLOTS",
    "VIBEOS_LINT_JOBS",
)


def _read_meminfo(path: Path = Path("/proc/meminfo")) -> dict[str, int]:
    result: dict[str, int] = {}
    for line in path.read_text().splitlines():
        key, _, rest = line.partition(":")
        fields = rest.split()
        if fields and fields[0].isdigit():
            result[key] = int(fields[0]) * (1024 if len(fields) > 1 and fields[1] == "kB" else 1)
    if not result:
        raise RuntimeError(f"no memory readings in {path}")
    return result


def _read_vmstat(path: Path = Path("/proc/vmstat")) -> dict[str, int]:
    result: dict[str, int] = {}
    for line in path.read_text().splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[1].isdigit() and fields[0].startswith(("oom", "pgmajfault")):
            result[fields[0]] = int(fields[1])
    if not result:
        raise RuntimeError(f"no OOM or major-fault counters in {path}")
    return result


def _read_pressure(path: Path = Path("/proc/pressure")) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for resource in ("memory", "cpu", "io"):
        try:
            body = (path / resource).read_text()
        except FileNotFoundError:
            result[resource] = {"available": False, "reason": "kernel PSI file absent"}
            continue
        rows: dict[str, dict[str, float]] = {}
        for line in body.splitlines():
            fields = line.split()
            if not fields:
                continue
            values: dict[str, float] = {}
            for pair in fields[1:]:
                key, _, value = pair.partition("=")
                values[key] = float(value)
            rows[fields[0]] = values
        result[resource] = {"available": True, "values": rows}
    return result


def _git_state(repo: Path) -> dict[str, Any]:
    def git(*args: str) -> str:
        cp = subprocess.run(["git", *args], cwd=repo, text=True, capture_output=True, check=False)
        if cp.returncode:
            raise RuntimeError(f"git {' '.join(args)} failed: {cp.stderr.strip()}")
        return cp.stdout.strip()

    status = git("status", "--porcelain=v1", "--untracked-files=all")
    # Porcelain alone cannot detect edits to an already-dirty path.
    diff = git("diff", "HEAD", "--binary")
    return {
        "head": git("rev-parse", "HEAD"),
        "dirty": bool(status),
        "dirty_status": status,
        "diff_sha256": hashlib.sha256(diff.encode()).hexdigest(),
    }


def _environment() -> dict[str, str]:
    return {key: os.environ[key] for key in ENV_KEYS if key in os.environ}


def _atomic_write(path: Path, data: dict[str, Any]) -> None:
    temp = path.with_name(f".{path.name}.{os.getpid()}.{uuid.uuid4().hex}.tmp")
    try:
        temp.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        temp.replace(path)
    finally:
        temp.unlink(missing_ok=True)


class Sampler:
    """Stream durable samples; required meminfo/vmstat failures reach the caller."""

    def __init__(self, path: Path, period: float = SAMPLE_PERIOD):
        self.path = path
        self.period = period
        self.stop_event = threading.Event()
        self.thread: threading.Thread | None = None
        self.error: BaseException | None = None
        self.count = 0
        self.min_available: int | None = None

    def _sample(self) -> None:
        try:
            with self.path.open("x", encoding="utf-8") as stream:
                while not self.stop_event.is_set():
                    row = {
                        "time": time.time(),
                        "meminfo_bytes": _read_meminfo(),
                        "vmstat": _read_vmstat(),
                        "pressure": _read_pressure(),
                    }
                    available = row["meminfo_bytes"]["MemAvailable"]
                    self.min_available = (
                        available
                        if self.min_available is None
                        else min(self.min_available, available)
                    )
                    stream.write(json.dumps(row) + "\n")
                    stream.flush()
                    self.count += 1
                    self.stop_event.wait(self.period)
        except BaseException as exc:
            self.error = exc
            self.stop_event.set()

    def start(self) -> None:
        self.thread = threading.Thread(target=self._sample, name="buildbench-sampler", daemon=True)
        self.thread.start()

    def stop(self) -> int:
        self.stop_event.set()
        if self.thread:
            self.thread.join()
        if self.error:
            raise RuntimeError(f"resource sampler failed: {self.error}") from self.error
        return self.count


def _run_child(name: str, command: list[str], repo: Path, log: Path) -> dict[str, Any]:
    started = time.monotonic()
    record: dict[str, Any] = {"name": name, "command": command, "log": str(log), "exit_code": None}
    try:
        artifact_root = log.with_suffix(".artifacts")
        with log.open("x", encoding="utf-8", errors="replace") as stream:
            cp = subprocess.run(
                command,
                cwd=repo,
                stdout=stream,
                stderr=subprocess.STDOUT,
                env=dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(artifact_root)),
                check=False,
            )
        record["exit_code"] = cp.returncode
        summaries = sorted(artifact_root.glob("*/summary.json"))
        record["summaries"] = [str(p) for p in summaries]
        if cp.returncode == 0 and (
            not summaries
            or any(
                json.loads(p.read_text()).get("verdict") not in ("OK", "WARN") for p in summaries
            )
        ):
            record["exit_code"] = 1
            record["artifact_error"] = "missing or unsuccessful child summary"
        record["status"] = "OK" if record["exit_code"] == 0 else "FAILED"
    except Exception as exc:  # includes launch and log creation failures
        record["launch_error"] = f"{type(exc).__name__}: {exc}"
        record["exit_code"] = 127
        record["status"] = "FAILED"
        try:
            with log.open("a", encoding="utf-8") as stream:
                stream.write(record["launch_error"] + "\n")
        except OSError:
            pass
    record["wall_seconds"] = time.monotonic() - started
    return record


def _run_children(
    commands: list[tuple[str, list[str]]], repo: Path, log_dir: Path
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    with ThreadPoolExecutor(max_workers=len(commands) or 1) as pool:
        futures = {
            pool.submit(_run_child, name, command, repo, log_dir / f"{name}.log"): name
            for name, command in commands
        }
        for future in as_completed(futures):
            try:
                records.append(future.result())
            except BaseException as exc:  # executor-level failures still become failed child rows
                name = futures[future]
                records.append(
                    {
                        "name": name,
                        "command": [],
                        "log": None,
                        "exit_code": 127,
                        "status": "FAILED",
                        "launch_error": f"{type(exc).__name__}: {exc}",
                    }
                )
    return sorted(records, key=lambda row: row["name"])


def _media_path(repo: Path, arch: str) -> Path:
    if arch == "arm64":
        return repo / "qemu/ramdisk.img"
    return repo / f"build-{arch}/livecd.iso"


def _check_build_artifacts(
    repo: Path, arches: list[str], children: list[dict[str, Any]]
) -> dict[str, dict[str, Any]]:
    by_name = {child["name"]: child for child in children}
    artifacts = {}
    for arch in arches:
        artifact = _media_path(repo, arch)
        exists = artifact.is_file()
        artifacts[arch] = {
            "path": str(artifact),
            "exists": exists,
            "size_bytes": artifact.stat().st_size if exists else None,
        }
        if not exists:
            by_name[f"build-{arch}"]["artifact_error"] = "expected boot media is absent"
        if arch == "arm64":
            boot = repo / "qemu/boot.img"
            artifacts[arch]["boot_image"] = str(boot)
            artifacts[arch]["boot_image_bytes"] = boot.stat().st_size if boot.is_file() else 0
            if not artifacts[arch]["boot_image_bytes"]:
                by_name[f"build-{arch}"]["artifact_error"] = "arm64 boot image is absent or empty"
    return artifacts


def add_parser(sub) -> None:
    parser = sub.add_parser("build-bench", help="measure concurrent build and lint resource use")
    parser.add_argument("--arches", nargs="+", choices=ARCHES, required=True)
    parser.add_argument(
        "--clean",
        action="store_true",
        help="clean build outputs; shared compiler caches remain warm",
    )
    parser.add_argument("--repeat", type=int, default=1)
    parser.add_argument(
        "--output", type=Path, help="summary JSON path (default: unique .cache/build-bench file)"
    )
    parser.set_defaults(fn=run)


def run(args, repo: Path) -> int:
    if args.repeat < 1:
        raise SystemExit("--repeat must be at least 1")
    initial_git = _git_state(repo)
    if initial_git["dirty"]:
        raise SystemExit("build-bench requires a clean committed tree for repeatable inputs")
    arches = list(dict.fromkeys(args.arches))
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S") + f"-{uuid.uuid4().hex[:8]}"
    out = (
        args.output.expanduser() if args.output else repo / ".cache/build-bench" / f"{stamp}.json"
    ).resolve()
    out.parent.mkdir(parents=True, exist_ok=True)
    try:
        with out.open("x", encoding="utf-8"):
            pass
    except FileExistsError as exc:
        raise SystemExit(f"build-bench output already exists: {out}") from exc
    report: dict[str, Any] = {
        "schema": 1,
        "status": "INCOMPLETE",
        "started_utc": datetime.now(timezone.utc).isoformat(),
        "repo": str(repo.resolve()),
        "git_before": initial_git,
        "arches": arches,
        "clean_build_outputs": bool(args.clean),
        "cache_note": "--clean removes build outputs only; compiler caches are not cleared",
        "repeat_requested": args.repeat,
        "environment": _environment(),
        "repeats": [],
        "build_policy": buildpolicy.read(hostlock.POLICY_DIR),
        "broker_before": hostjobs.status(repo),
    }
    _atomic_write(out, report)
    overall_failed = False
    for index in range(1, args.repeat + 1):
        run_dir = out.parent / f"{out.stem}-run{index:02d}"
        try:
            run_dir.mkdir(parents=False, exist_ok=False)
        except FileExistsError:
            report["status"] = "FAILED"
            report["error"] = f"run directory already exists: {run_dir}"
            _atomic_write(out, report)
            return 1
        repeat: dict[str, Any] = {
            "index": index,
            "status": "INCOMPLETE",
            "children": [],
            "samples_jsonl": str(run_dir / "samples.jsonl"),
        }
        report["repeats"].append(repeat)
        _atomic_write(out, report)
        sampler = Sampler(run_dir / "samples.jsonl")
        children: list[dict[str, Any]] = []
        before_vmstat: dict[str, int] = {}
        after_vmstat: dict[str, int] = {}
        started = time.monotonic()
        try:
            before_vmstat = _read_vmstat()
            sampler.start()
            cli = str(repo / "vibeos-cli")
            first = [
                (
                    f"build-{arch}",
                    [cli, "build", "--arch", arch, "--no-check"]
                    + (["--clean"] if args.clean else []),
                )
                for arch in arches
            ]
            first.append(
                ("check-source-early", [cli, "check", "--strict", "--scope", "source-early"])
            )
            defer_wow64 = "amd64" in arches and "i386" in arches
            if defer_wow64:
                for name, command in first:
                    if name == "build-amd64":
                        command.append("--no-wow64-stage")
            first_children = _run_children(first, repo, run_dir)
            children.extend(first_children)
            build_children = [row for row in first_children if row["name"].startswith("build-")]
            artifacts = _check_build_artifacts(repo, arches, build_children)
            builds_ok = all(
                row["exit_code"] == 0 and not row.get("artifact_error") for row in build_children
            )
            if builds_ok and defer_wow64:
                restage = _run_children(
                    [("build-amd64-restage", [cli, "build", "--arch", "amd64", "--no-check"])],
                    repo,
                    run_dir,
                )
                children.extend(restage)
                artifacts = _check_build_artifacts(repo, arches, build_children)
                builds_ok = all(row["exit_code"] == 0 for row in restage) and all(
                    not row.get("artifact_error") for row in build_children
                )
            if builds_ok:
                followups = [
                    (
                        "check-source-deferred",
                        [cli, "check", "--strict", "--scope", "source-deferred"],
                    )
                ]
                followups.extend(
                    (
                        f"check-arch-{arch}",
                        [cli, "check", "--strict", "--scope", "arch", "--arch", arch],
                    )
                    for arch in arches
                )
                children.extend(_run_children(followups, repo, run_dir))
            else:
                skipped = [
                    (
                        "check-source-deferred",
                        [cli, "check", "--strict", "--scope", "source-deferred"],
                    )
                ]
                skipped.extend(
                    (
                        f"check-arch-{arch}",
                        [cli, "check", "--strict", "--scope", "arch", "--arch", arch],
                    )
                    for arch in arches
                )
                children.extend(
                    {
                        "name": name,
                        "command": command,
                        "log": None,
                        "exit_code": None,
                        "status": "SKIPPED",
                        "reason": "one or more builds failed",
                    }
                    for name, command in skipped
                )
            sample_count = sampler.stop()
            after_vmstat = _read_vmstat()
            end_git = _git_state(repo)
            report["git_after"] = end_git
            if end_git != initial_git:
                raise RuntimeError(
                    f"git state changed during benchmark: before={initial_git}, after={end_git}"
                )
            failures = [
                row["name"]
                for row in children
                if row["exit_code"] not in (0, None) or row.get("artifact_error")
            ]
            if not builds_ok:
                failures.extend(
                    name
                    for name in ["check-source-deferred", *(f"check-arch-{a}" for a in arches)]
                    if name not in failures
                )
            repeat.update(
                {
                    "wall_seconds": time.monotonic() - started,
                    "children": sorted(children, key=lambda row: row["name"]),
                    "artifacts": artifacts,
                    "sample_count": sample_count,
                    "min_mem_available_bytes": sampler.min_available,
                    "vmstat_before": before_vmstat,
                    "vmstat_after": after_vmstat,
                    "oom_delta": {
                        key: after_vmstat.get(key, 0) - before_vmstat.get(key, 0)
                        for key in sorted(set(before_vmstat) | set(after_vmstat))
                        if key.startswith("oom")
                    },
                    "failed_children": sorted(set(failures)),
                    "status": "FAILED" if failures else "OK",
                }
            )
        except Exception as exc:
            overall_failed = True
            try:
                sample_count = sampler.stop()
            except Exception as sample_exc:
                repeat["sampler_error"] = str(sample_exc)
                sample_count = sampler.count
            repeat.update(
                {
                    "status": "FAILED",
                    "error": f"{type(exc).__name__}: {exc}",
                    "children": sorted(children, key=lambda row: row["name"]),
                    "sample_count": sample_count,
                    "vmstat_before": before_vmstat,
                    "vmstat_after": after_vmstat,
                }
            )
        if repeat["status"] != "OK":
            overall_failed = True
        report["status"] = "INCOMPLETE"
        report["updated_utc"] = datetime.now(timezone.utc).isoformat()
        _atomic_write(out, report)
        print(f"build-bench repeat {index}/{args.repeat}: {repeat['status']}; summary {out}")
    report["status"] = "FAILED" if overall_failed else "OK"
    report["exit_code"] = int(overall_failed)
    try:
        final_git = _git_state(repo)
        report["git_after"] = final_git
        if final_git != initial_git:
            report["status"] = "FAILED"
            report["exit_code"] = 1
    except Exception as exc:
        report["status"] = "FAILED"
        report["exit_code"] = 1
        report["git_after_error"] = f"{type(exc).__name__}: {exc}"
    report["updated_utc"] = datetime.now(timezone.utc).isoformat()
    _atomic_write(out, report)
    return report["exit_code"]
