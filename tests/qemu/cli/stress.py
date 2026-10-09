"""`vibeos-cli stress` — reproduce "host load" as named mechanisms, outside the gate.

"It only fails under host load" bundles three different things, and a loaded host
is a bad instrument for telling them apart: it changes all three at once and never
the same way twice. Each has a deterministic knob, so this runs the SAME media
through a matrix of one-knob-at-a-time profiles and reports the failure rate per
profile. The matrix is the diagnosis:

    profile       knob                                     a failure here is
    baseline      none (control)                           a plain bug, or the media
    slow-cpu      -icount shift=N,sleep=off                a guest-side TIMEOUT: the
                  (guest clock advances 2^N ns/insn,       guest sees minutes pass in
                  whatever the host does)                  a 40s boot
    lockstep      -smp 2, single-threaded TCG              a cross-CPU RACE, now with a
                  (vCPUs round-robin on one host thread)   stable interleaving to debug
    starved-smp   -smp 2 MTTCG, QEMU pinned to ONE host    the "host load" race: the host
                  core (the scheduler picks who runs)      scheduler preempts a vCPU
                                                           mid-critical-section
    tight-clock   harness boot/silence budgets x0.5        mechanism 3: a healthy boot
                                                           that crossed the wall clock —
                                                           fix the budget, not the OS

A profile that cannot be applied (no process to pin, UP media on an SMP profile)
is reported as NOT APPLIED, never as green: smoke's own `_smp_check` already fails
a boot that got the UP kernel, and the pinning watcher records how many processes
it actually pinned.

Not a gate stage. Rows go to the run dir (`rows.jsonl`) and failures keep their
artifacts the way `farm` does, so `flake-report`'s ledger sees them too.

Second payoff of `slow-cpu`: icount makes a single-vCPU boot deterministic, which is
what record/replay needs (`vibeos-cli rr`), so a flake caught once under it can be
replayed under GDB.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import statistics
import subprocess
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path

from . import archcfg, provenance
from .artifacts import Run
from .farm import _extract_evidence, _git_state, _preserve_failure
from .smoke import _one_boot

# Knob values. The icount shift is the only one worth arguing about: shift=3 is a
# ~125 MIPS virtual CPU (about TCG's real pace, so the guest notices nothing);
# shift=7 is ~8 MIPS, a 40s TCG boot during which the guest clock runs ~20 minutes.
DEFAULT_ICOUNT_SHIFT = 7
DEFAULT_TIMEOUT_SCALE = 0.5


@dataclass
class Profile:
    name: str
    mechanism: str
    env: dict[str, str] = field(default_factory=dict)
    smp: int = 0
    pin_cores: int = 0  # >0: pin every QEMU of this boot to that many host cores

    def describe(self) -> str:
        bits = [f"{k}={v}" for k, v in self.env.items()]
        if self.smp:
            bits.append(f"-smp {self.smp}")
        if self.pin_cores:
            bits.append(f"pinned to {self.pin_cores} host core(s)")
        return ", ".join(bits) or "control"


def profiles(args) -> list[Profile]:
    shift = int(getattr(args, "shift", 0) or DEFAULT_ICOUNT_SHIFT)
    scale = float(getattr(args, "timeout_scale", 0) or DEFAULT_TIMEOUT_SCALE)
    smp = int(getattr(args, "smp", 0) or 2)
    cores = int(getattr(args, "cores", 0) or 1)
    table = {
        "baseline": Profile("baseline", "control: the media on an unloaded host"),
        "slow-cpu": Profile(
            "slow-cpu",
            "guest-side timeout (slow emulation)",
            env={"VIBEOS_QEMU_ICOUNT_SHIFT": str(shift)},
        ),
        "lockstep": Profile(
            "lockstep",
            "cross-CPU race (deterministic interleaving)",
            env={"VIBEOS_TCG_THREAD": "single"},
            smp=smp,
        ),
        "starved-smp": Profile(
            "starved-smp",
            "cross-CPU race (host scheduler preemption)",
            smp=smp,
            pin_cores=cores,
        ),
        "tight-clock": Profile(
            "tight-clock",
            "harness wall clock (healthy boot, budget too small)",
            env={"VIBEOS_BOOT_TIMEOUT_SCALE": str(scale)},
        ),
    }
    want = [p.strip() for p in (getattr(args, "profile", "") or "").split(",") if p.strip()]
    if not want or want == ["all"]:
        want = list(table)
    unknown = [p for p in want if p not in table]
    if unknown:
        raise SystemExit(f"stress: unknown profile(s) {unknown}; known: {', '.join(table)}")
    return [table[p] for p in want]


# --- host-side pinning -------------------------------------------------------
#
# The windows-host backend runs qemu.exe on Windows, so affinity is set through
# PowerShell by the unique `-name vibeos-<pid>-<port>` tag the harness gives every
# QEMU it launches (the same handle `vibeos-cli kill` uses). A native QEMU is our
# own child, so it is found by parent pid and pinned with sched_setaffinity.


def _affinity_mask(cores: int) -> tuple[int, set[int]]:
    total = os.cpu_count() or 1
    chosen = set(range(max(0, total - cores), total))  # the LAST cores: stay off CPU0
    return sum(1 << c for c in chosen), chosen


def _pin_windows(uid_prefix: str, mask: int) -> list[int]:
    ps = (
        "Get-CimInstance Win32_Process -Filter \"Name like 'qemu-system-%'\" "
        f"| Where-Object {{ $_.CommandLine -like '*{uid_prefix}*' }} "
        "| ForEach-Object { $p = Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue; "
        f"if ($p -and [int64]$p.ProcessorAffinity -ne {mask}) "
        f"{{ $p.ProcessorAffinity = [IntPtr]{mask}; Write-Output $_.ProcessId }} }}"
    )
    try:
        cp = subprocess.run(
            ["powershell.exe", "-NoProfile", "-Command", ps],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return []
    return [int(x) for x in re.findall(r"^\s*(\d+)\s*$", cp.stdout or "", re.M)]


def _pin_native(parent: int, cpus: set[int]) -> list[int]:
    pinned: list[int] = []
    proc = Path("/proc")
    for d in proc.iterdir():
        if not d.name.isdigit():
            continue
        try:
            stat = (d / "stat").read_text()
            comm = stat[stat.index("(") + 1 : stat.rindex(")")]
            ppid = int(stat[stat.rindex(")") + 2 :].split()[1])
            if ppid != parent or not comm.startswith("qemu-system"):
                continue
            pid = int(d.name)
            if os.sched_getaffinity(pid) != cpus:
                os.sched_setaffinity(pid, cpus)
                pinned.append(pid)
        except (OSError, ValueError):
            continue
    return pinned


class _Pinner:
    """Poll for this process's QEMUs while a boot runs and pin each one once."""

    def __init__(self, cores: int):
        self.mask, self.cpus = _affinity_mask(cores)
        self.pinned: set[int] = set()
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._loop, daemon=True)

    def _loop(self) -> None:
        prefix = f"vibeos-{os.getpid()}-"
        while not self._stop.is_set():
            for pid in _pin_native(os.getpid(), self.cpus):
                self.pinned.add(pid)
            if shutil.which("powershell.exe"):
                for pid in _pin_windows(prefix, self.mask):
                    self.pinned.add(pid)
            self._stop.wait(2.0)

    def __enter__(self) -> _Pinner:
        self._thread.start()
        return self

    def __exit__(self, *exc) -> None:
        self._stop.set()
        self._thread.join(timeout=35)


# --- the matrix --------------------------------------------------------------

_KNOB_ENV = (
    "VIBEOS_QEMU_ICOUNT_SHIFT",
    "VIBEOS_TCG_THREAD",
    "VIBEOS_BOOT_TIMEOUT_SCALE",
    "VIBEOS_QEMU_SMP",
)


def _with_env(values: dict[str, str]):
    """Context manager: set exactly these knob vars, restore ALL knob vars after."""
    import contextlib

    @contextlib.contextmanager
    def cm():
        saved = {k: os.environ.get(k) for k in _KNOB_ENV}
        for k in _KNOB_ENV:
            os.environ.pop(k, None)
        os.environ.update(values)
        try:
            yield
        finally:
            for k, v in saved.items():
                if v is None:
                    os.environ.pop(k, None)
                else:
                    os.environ[k] = v

    return cm()


def _boot(repo: Path, arch: str, profile: Profile, idx: int, sha: str, dirty: bool) -> dict:
    run = Run("stress", arch)
    env = dict(profile.env)
    if profile.smp:
        env["VIBEOS_QEMU_SMP"] = str(profile.smp)
    t0 = time.monotonic()
    pinned: set[int] = set()
    with _with_env(env):
        try:
            if profile.pin_cores:
                with _Pinner(profile.pin_cores) as pinner:
                    r = _one_boot(run, repo, arch, until="", fail_on=None, idx=0, total=1)
                pinned = set(pinner.pinned)
            else:
                r = _one_boot(run, repo, arch, until="", fail_on=None, idx=0, total=1)
        except Exception as e:  # a matrix cell must never take the matrix down
            r = {
                "ok": False,
                "outcome": "stress_error",
                "classification": "STRESS_ERROR",
                "salient": [repr(e)[:200]],
            }
    dur = time.monotonic() - t0
    run.classification = r.get("classification", "") or ""
    try:
        run.finalize(repo)
    except Exception:
        pass
    applied = not profile.pin_cores or bool(pinned)
    rec = {
        "ts": time.strftime("%Y%m%d-%H%M%S"),
        "arch": arch,
        "sha": sha,
        "dirty": dirty,
        "profile": profile.name,
        "mechanism": profile.mechanism,
        "knobs": env,
        "pinned": sorted(pinned),
        "applied": applied,
        "boot": idx,
        "ok": bool(r.get("ok", False)) and applied,
        "outcome": r.get("outcome", ""),
        "classification": r.get("classification", "")
        if applied
        else "NOT_APPLIED (no QEMU process was pinned)",
        "hang_kind": r.get("hang_kind"),
        "salient": (r.get("salient") or [])[:8],
        "screenshot": r.get("screenshot"),
        "run_dir": str(run.dir),
        "duration_s": round(dur, 1),
        "timings": r.get("timings", {}),
    }
    rec.update(_extract_evidence(run.dir, rec["salient"]))
    if not rec["ok"]:
        rec["preserved"] = _preserve_failure(run.dir, rec)
    return rec


def summarize(rows: list[dict]) -> list[str]:
    """One line per profile: boots, failures, failure families, median duration."""
    by: dict[str, list[dict]] = {}
    for r in rows:
        by.setdefault(r["profile"], []).append(r)
    out = [f"{'profile':12} {'boots':>5} {'fail':>4} {'median':>7}  failures"]
    for name, rs in by.items():
        fails = [r for r in rs if not r["ok"]]
        fam: dict[str, int] = {}
        for r in fails:
            key = r.get("classification") or r.get("outcome") or "?"
            if r.get("stop_code"):
                key += f" stop={r['stop_code']}"
            if r.get("top_symbol"):
                key += f" @ {r['top_symbol']}"
            fam[key] = fam.get(key, 0) + 1
        med = statistics.median(r["duration_s"] for r in rs) if rs else 0.0
        fams = "; ".join(f"{k} x{n}" for k, n in sorted(fam.items(), key=lambda kv: -kv[1]))
        out.append(f"{name:12} {len(rs):5d} {len(fails):4d} {med:6.0f}s  {fams or '-'}")
    return out


def diagnosis(rows: list[dict]) -> str:
    """Read the matrix the way the module docstring says to."""
    fails = {
        p
        for p in {r["profile"] for r in rows}
        if any(not r["ok"] for r in rows if r["profile"] == p)
    }
    ran = {r["profile"] for r in rows}
    if not fails:
        return (
            "no profile failed: whatever this flake is, these three mechanisms are not it "
            "at this sample size — go look for a real bug, not for host weather"
        )
    if "baseline" in fails:
        return "baseline fails: this is not a load flake, it is a bug (or stale media)"
    parts = []
    if "slow-cpu" in fails:
        parts.append(
            "slow-cpu fails -> a guest-side timeout fires when the guest clock outruns the CPU"
        )
    if fails & {"lockstep", "starved-smp"}:
        which = ", ".join(sorted(fails & {"lockstep", "starved-smp"}))
        parts.append(
            f"{which} fails -> a cross-CPU race; lockstep gives a stable interleaving to debug"
        )
    if "tight-clock" in fails and not (fails - {"tight-clock"}):
        parts.append(
            "ONLY tight-clock fails -> a healthy boot crossing the wall clock; "
            "the budget is the bug"
        )
    elif "tight-clock" in fails:
        parts.append("tight-clock also fails (expected alongside a real slowdown)")
    missing = {"baseline", "slow-cpu", "lockstep", "starved-smp"} - ran
    if missing:
        parts.append(f"(profiles not run: {', '.join(sorted(missing))})")
    return "; ".join(parts)


def stress(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)
    repeat = int(args.repeat or 3)
    st = provenance.check_staleness(repo, arch)
    if st.stale and not getattr(args, "allow_stale", False):
        print(f"[stress] {arch}: STALE media — build first (or --allow-stale)")
        return 1
    plan = profiles(args)
    sha, dirty = _git_state(repo)
    root = Run("stress", arch)
    rows_path = root.dir / "rows.jsonl"
    print(
        f"[stress] arch={arch} sha={sha}{' DIRTY' if dirty else ''} repeat={repeat} "
        f"profiles={','.join(p.name for p in plan)} -> {rows_path}"
    )
    rows: list[dict] = []
    for profile in plan:
        print(f"[stress {profile.name}] {profile.mechanism}: {profile.describe()}", flush=True)
        for i in range(1, repeat + 1):
            rec = _boot(repo, arch, profile, i, sha, dirty)
            rows.append(rec)
            with rows_path.open("a") as f:
                f.write(json.dumps(rec) + "\n")
            state = "ok" if rec["ok"] else (rec["classification"] or rec["outcome"])
            print(
                f"[stress {profile.name}] boot {i}/{repeat}: {state} ({rec['duration_s']:.0f}s)"
                + (f" stop={rec['stop_code']}" if rec.get("stop_code") else "")
                + (f" @ {rec['top_symbol']}" if rec.get("top_symbol") else "")
                + (f" pinned={rec['pinned']}" if profile.pin_cores else ""),
                flush=True,
            )
    print()
    for line in summarize(rows):
        print(line)
    print(f"\n[stress] reading: {diagnosis(rows)}")
    print(f"[stress] rows: {rows_path}")
    return 1 if any(not r["ok"] for r in rows) else 0
