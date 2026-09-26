"""Durable artifact directory + token-frugal summary (plan §6).

Every command run gets ${VIBEOS_ARTIFACT_ROOT:-/tmp/vibeos-cli}/<ts>-<unique>-<cmd>-<arch>/
with full.log (raw stage output under headers), summary.json (the machine-true
verdict — re-readable, never scraped from stdout), commands.jsonl, and any
serial/debugcon/screenshot captured. stdout gets only a bounded summary.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path

ARTIFACT_ROOT = Path(os.environ.get("VIBEOS_ARTIFACT_ROOT", "/tmp/vibeos-cli"))


def create_run_dir(command: str, arch: str, ts: int | None = None) -> Path:
    """Atomically allocate a private directory, including for same-second runs.

    Also seeds the INTERRUPTED MARKER (see `write_interrupted_marker`). Done here
    because this is the one choke point every run dir passes through -- `Run`
    takes it, and so do the commands that build their own summary by hand
    (conformance).
    """
    ts = ts if ts is not None else int(time.time())
    stamp = time.strftime("%Y%m%d-%H%M%S", time.localtime(ts))
    ARTIFACT_ROOT.mkdir(parents=True, exist_ok=True)
    # Keep the date prefix and command/arch suffix for existing artifact scans.
    # A random component also separates us from older lanes using bare seconds.
    run_dir = Path(
        tempfile.mkdtemp(prefix=f"{stamp}-", suffix=f"-{command}-{arch}", dir=ARTIFACT_ROOT)
    )
    write_interrupted_marker(run_dir, command, arch)
    return run_dir


def write_interrupted_marker(run_dir: Path, command: str, arch: str) -> None:
    """Write a summary.json that says INCOMPLETE, before the work starts.

    A run that is SIGKILLed -- by the host's memory reaper, by a broad-kill, by
    the machine going away -- never reaches finalize(), so it never writes a
    verdict. Until now that left an artifact dir holding a full.log full of green
    stage lines and NO summary.json, and a caller whose exit status is not the
    work's (a pipeline's last stage, a backgrounded shell's `echo`) reads that as
    a pass. A reaped process must not look successful to its caller.

    Absence cannot carry that meaning, because absence is overloaded three ways:
    the run is still going, the run was killed, or the directory was never a run
    at all. So the claim is made POSITIVELY at creation and overwritten by
    finalize(). Whatever is on disk is then always a true statement about the run:
    INCOMPLETE until a verdict exists, and a verdict once one does.

    This is the same shape as `.vibeos-build-stamp.json` (written at the END of a
    successful build, so its presence means "finished") turned inside out: there
    the positive claim is completion, here it is incompletion. Both exist so that
    no consumer has to infer a run's state from an exit code it may not have.
    """
    try:
        (run_dir / "summary.json").write_text(
            json.dumps(
                {
                    "run_id": run_dir.name,
                    "command": command,
                    "arch": arch,
                    "verdict": "INCOMPLETE",
                    "classification": "RUN_INTERRUPTED",
                    "incomplete": True,
                    "pid": os.getpid(),
                    "started_epoch": time.time(),
                    "note": (
                        "This run never reached finalize(): it was killed, or it is still "
                        "running. Either way it has no verdict and must not be read as one."
                    ),
                },
                indent=2,
            )
        )
    except OSError:
        # Never fail a run over its own bookkeeping. The pre-existing behaviour
        # (no summary at all) is what callers already handle.
        pass


def is_incomplete(summary: dict) -> bool:
    """True for an interrupted-run marker: a summary that carries no verdict.

    Callers that already have a good "no verdict" path (gate's SMOKE_NO_SUMMARY,
    which keeps the child's console and its exit code) should route markers into
    it rather than treating INCOMPLETE as an ordinary red -- the marker says less
    than that path already knows.
    """
    return bool(summary.get("incomplete"))


FARM_DIR = Path.home() / "vibeos-flake-farm"

# Commands whose ledger rows carry a flake signature (see `Run._ledger_append`).
BOOT_COMMANDS = frozenset({"smoke", "gate", "boot"})


def _sha12(path: Path) -> str | None:
    import hashlib

    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()[:12]
    except OSError:
        return None


def _snapshot_kernel(repo: Path, arch: str, mp: bool) -> dict:
    """Park the booted kernel + its symbol index under farm/kernels/<sha12>/.

    A preserved serial that cannot be symbolicated is not evidence. The index is
    rebuilt on every `vibeos-cli build`, so by the time anyone reads a fleet
    flake the tsv that matched it is long gone -- this is exactly how a capture
    came back as `XHF_NOEXECUTE+310101`, a real-looking symbol from a stale
    relocation. Content-addressed by the kernel's own hash, so N flakes on one
    build cost ONE copy and re-preserving is free.
    """
    import shutil

    base = repo / f"build-{arch}" / "ntoskrnl"
    kernel = base / "ntkrnlmp" / "ntkrnlmp.exe" if mp else base / "ntoskrnl.exe"
    sha = _sha12(kernel) if kernel.is_file() else None
    if not sha:
        return {"kernel_sha12": None, "kernels_dir": None}
    dst = FARM_DIR / "kernels" / sha
    info = {"kernel_sha12": sha, "kernels_dir": str(dst)}
    if dst.is_dir():
        return info  # already parked by an earlier flake on this build
    try:
        tmp = dst.with_name(dst.name + ".partial")
        tmp.mkdir(parents=True, exist_ok=True)
        shutil.copy2(kernel, tmp / kernel.name)
        idx = repo / "qemu" / f"symbols-{arch}{'-mp' if mp else ''}.tsv"
        if idx.is_file():
            shutil.copy2(idx, tmp / idx.name)
        hal = base.parent / "hal" / ("halmp" if mp else "hal") / ("halmp.dll" if mp else "hal.dll")
        if hal.is_file():
            shutil.copy2(hal, tmp / hal.name)
        tmp.rename(dst)  # publish atomically: a half-copied dir must never look done
    except OSError:
        return {"kernel_sha12": sha, "kernels_dir": None}
    return info


def preserve_failure(
    run_dir: Path, arch: str, classification: str, repo: Path | None
) -> str | None:
    """Copy a failed boot's evidence somewhere that survives, and pin the
    binaries needed to read it later.

    The artifact root defaults to /tmp/vibeos-cli and nothing caps its retention,
    so a flake's serial is gone within days. That is not theoretical: three
    BUGCHECK 0x1E captures from a 60-round arm64 soak were lost exactly this way,
    and they were the whole point of the run. `farm` preserved failures already
    but copied neither the serial nor the binaries -- the two things a later
    reader actually needs. Now every command that boots gets this, because the
    fleet runs `smoke`/`gate`, not `farm`.

    Never raises: evidence collection must not turn a reported failure into a
    crash in the reporter.
    """
    import shutil

    try:
        serials = sorted(run_dir.glob("*serial*.log"))
        if not serials:
            return None  # not a boot-bearing run; nothing to preserve
        mp = any("ntkrnlmp.exe" in s.read_text(errors="replace") for s in serials)
        tag = re.sub(r"[^A-Za-z0-9_.-]+", "_", classification or "FAIL").strip("_")
        dst = FARM_DIR / "fails" / "auto" / f"{run_dir.name}-{tag}"
        dst.mkdir(parents=True, exist_ok=True)
        for src in serials:
            shutil.copy2(src, dst / src.name)
        for name in ("summary.json", "full.log", "hang-cpustate.txt"):
            src = run_dir / name
            if src.is_file():
                shutil.copy2(src, dst / name)
        # A conformance round's parsed results: without them a later parser fix
        # cannot re-grade history (NpfsFileInfo.c:57's "passes" were parser misses,
        # and the files that would prove it were in /tmp).
        for src in run_dir.glob("*-results.txt"):
            shutil.copy2(src, dst / src.name)
        for shot in run_dir.glob("*.png"):
            shutil.copy2(shot, dst / shot.name)
        manifest = {
            "run_id": run_dir.name,
            "arch": arch,
            "classification": classification,
            "mp_kernel": mp,
            "head": _head_sha(repo) if repo else None,
            "dirty": _is_dirty(repo) if repo else None,
        }
        if repo:
            manifest.update(_snapshot_kernel(repo, arch, mp))
        (dst / "manifest.json").write_text(json.dumps(manifest, indent=2))
        return str(dst)
    except (OSError, ValueError):
        return None


def _head_sha(repo: Path) -> str | None:
    try:
        cp = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        return cp.stdout.strip() if cp.returncode == 0 else None
    except (OSError, subprocess.SubprocessError):
        return None


def _is_dirty(repo: Path) -> bool | None:
    try:
        cp = subprocess.run(
            ["git", "-C", str(repo), "status", "--porcelain"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        return bool(cp.stdout.strip()) if cp.returncode == 0 else None
    except (OSError, subprocess.SubprocessError):
        return None


def archive_serial(run_dir: Path, live_log: str | Path | None, extra: dict) -> None:
    """Copy the boot's serial log into the run dir, and repoint `serial_log` at
    the copy.

    `qemu/<arch>_smoke_live.log` is a FIXED path that every boot of that arch
    truncates, so a summary.json pointing there describes whatever booted last,
    not this run. That is silent evidence loss and it is not hypothetical: an
    IoFilesystem BUGCHECK 0x19 printed a full [POOL] page dump + free ring to
    serial, and by the time anyone read the run dir a later boot had overwritten
    every byte of it — the crash had to be re-collected with a fresh build+boot.

    Cheap enough to do unconditionally (serial logs run ~100KB), and never fatal:
    a run that produced no serial at all is normal (build-only failures).
    """
    if not live_log:
        return
    src = Path(live_log)
    extra["live_log"] = str(src)
    if not src.is_file():
        return
    dst = run_dir / "serial.log"
    try:
        dst.write_bytes(src.read_bytes())
    except OSError:
        return
    extra["serial_log"] = str(dst)


def _worktree_of(repo: Path) -> Path:
    """Absolute path of the worktree a run belongs to, for summary.json's
    `worktree` field. Plain resolve() — no git call, because finalize() runs on
    every command including failures and must never itself fail or block."""
    try:
        return Path(repo).resolve()
    except OSError:
        return Path(repo)


@dataclass
class Stage:
    name: str
    exit: int = 0
    seconds: float = 0.0
    status: str = "ok"  # ok | warn | fail | skip
    detail: str = ""  # one-line, shown in summary
    salient: list = field(default_factory=list)  # key lines for the summary


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
    """T3 (fleet-isolation program): durable outcome ledger. /tmp run dirs die on
    reboot, and gate verdicts are point samples — the ledger under VIBEOS_ROOT
    survives both, so `vibeos-cli health` can show trailing per-arch PASS RATES
    (a 100%→20% arch regression becomes visible data within a day even when
    individual gates roll green). One JSON line per run; O_APPEND keeps
    concurrent lanes' writes whole. Never fails the run.

    Module-level, not a `Run` method, because a command does not have to use
    `Run` to have an outcome worth keeping. `conformance` builds its summary by
    hand, so for the project's whole history it wrote ZERO rows here — 19,711
    ledger rows across build/smoke/check/gate/farm/land and not one conformance
    verdict. Its round records went to `~/vibeos-flake-farm/` instead, which is
    outside the tree the backups cover: when the box was rebuilt on 2026-09-14
    the shared ledger came back and the flake farm did not, taking a 201-run
    arm64 census with it. The command whose verdicts we most wanted counted over
    time was the only one absent from the record that survives.
    """
    try:
        if repo is None:
            return
        from . import cachecfg  # local import: avoid a cycle at module load

        root = cachecfg.vibeos_root(repo)
        if not root.is_dir():
            return
        head = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=repo,
            capture_output=True,
            text=True,
            timeout=15,
        ).stdout.strip()
        row = {
            "ts": int(time.time()),
            "cmd": cmd,
            "arch": arch,
            "verdict": verdict,
            "classification": classification,
            "duration_ms": duration_ms,
            "head": head,
            "worktree": Path(repo).name,
            **(extra or {}),
        }
        with open(root / "vibeos-ledger.jsonl", "a") as f:
            f.write(json.dumps(row) + "\n")
    except Exception:
        pass  # the ledger is telemetry; it must never break a build/smoke


class Run:
    """One command execution: owns the artifact dir, the full log, and the
    structured summary."""

    def __init__(self, command: str, arch: str, ts: int | None = None):
        self.command = command
        self.arch = arch
        self.dir = create_run_dir(command, arch, ts)
        self.full_log = self.dir / "full.log"
        self.commands_log = self.dir / "commands.jsonl"
        self.stages: list[Stage] = []
        self.command_timings: list[dict] = []
        # full.log / commands.jsonl are appended from WORKER THREADS now (gate submits its
        # per-arch builds concurrently), and a multi-KB build log is several buffer flushes
        # — without this the two builds' output interleaves mid-line and the log stops being
        # readable. `add()` needs no lock (list.append is atomic under the GIL).
        self._log_lock = threading.Lock()
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
        with self._log_lock:
            timing = {
                "stage": stage or cmd[0],
                "start_seconds": round(t0 - self.start, 6),
                "seconds": round(dt, 6),
                "exit": rc,
            }
            self.command_timings.append(timing)
            with open(self.full_log, "a") as f:
                f.write(f"\n=== {stage or cmd[0]}: {' '.join(cmd)} — exit {rc}, {dt:.1f}s ===\n")
                if out:
                    f.write(out)
                if err:
                    f.write("\n--- stderr ---\n" + err)
            with open(self.commands_log, "a") as f:
                f.write(json.dumps({"cmd": cmd, "cwd": str(cwd or ""), **timing}) + "\n")
        return cp

    def log(self, text: str, header: str | None = None) -> None:
        with self._log_lock, open(self.full_log, "a") as f:
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
        archive_serial(self.dir, self.extra.get("live_log"), self.extra)
        summary = {
            "run_id": self.dir.name,
            "command": self.command,
            "arch": self.arch,
            # WHOSE run this is. VIBEOS_ARTIFACT_ROOT defaults to /tmp/vibeos-cli
            # for EVERY worktree, so run dirs from all lanes interleave in one
            # directory and `ls -dt /tmp/vibeos-cli/*-build-arm64 | head -1` hands
            # you whichever lane happened to start last. That is the read half of
            # the fleet-isolation law (AGENTS.md §8), and it has already produced a
            # wrong status report: another lane's green arm64 build was read as
            # this lane's. Attribution belonged in the artifact, not in a grep of
            # full.log that every caller has to reinvent.
            "worktree": str(_worktree_of(repo)) if repo else None,
            "verdict": self.verdict,
            "classification": self.classification,
            "duration_ms": int(dur * 1000),
            "stages": [s.__dict__ for s in self.stages],
            # Separate from verdict stages: one verdict can cover several commands,
            # and parallel command durations must not be summed as elapsed time.
            "command_timings": self.command_timings,
            "artifacts": {"full_log": str(self.full_log)},
            **self.extra,
        }
        (self.dir / "summary.json").write_text(json.dumps(summary, indent=2))
        # A red boot's evidence outlives the volatile artifact root. Done here
        # rather than per-command because finalize() is the one choke point every
        # command passes through, and the fleet runs smoke/gate, not farm.
        if self.verdict not in ("OK", "WARN"):
            kept = preserve_failure(self.dir, self.arch, self.classification, repo)
            if kept:
                summary["preserved"] = kept
                (self.dir / "summary.json").write_text(json.dumps(summary, indent=2))
        self._ledger_append(repo, summary, dur)
        self._print_summary(dur)
        return 0 if self.verdict in ("OK", "WARN") else 1

    def _ledger_append(self, repo: Path | None, summary: dict, dur: float) -> None:
        """Delegates to the module-level `ledger_append`. See its docstring for why
        that is not a method: a command does not have to use `Run` to have an
        outcome worth keeping, and `conformance` does not.

        A boot's row carries enough to NAME its failure after /tmp is gone: the
        signature, the preserved-evidence dir, and whether the tree was dirty (a
        lane iterating on uncommitted edits otherwise reads as a flaky commit).
        `vibeos-cli flake-report` is the reader."""
        extra: dict = {"run_id": summary.get("run_id")}
        if self.command in BOOT_COMMANDS and repo is not None:
            from . import flakes

            extra["dirty"] = _is_dirty(repo)
            if "runs" in summary:
                extra["boots"] = summary.get("runs")
                extra["passes"] = summary.get("passes")
            sig = flakes.run_signature(summary)
            if sig:
                extra["sig"] = sig
            if summary.get("preserved"):
                extra["preserved"] = summary["preserved"]
        ledger_append(
            repo,
            cmd=self.command,
            arch=self.arch,
            verdict=self.verdict,
            classification=self.classification,
            duration_ms=int(dur * 1000),
            extra=extra,
        )

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
