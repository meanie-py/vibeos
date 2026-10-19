"""`vibeos-cli gate` — the parallel pre-land verdict driver.

The North Star discipline is "gate smoke ×3 (all arches green) before landing".
Historically that meant running `vibeos-cli smoke --arch <a>` three times by hand,
serially — sum of every arch's boot time, plus a manual re-run every time the
amd64 storage KNOWN_FLAKE (or an arm64/i386 documented flake) tripped. On the
default `windows-host` TCG backend every per-run resource is already unique
(staging subdir/ports/-name uid/reaper key are pid+port scoped — see
tests/qemu/vm.py:892-902) and the per-arch cap is bypassed (vm.py:1295-1299), so
concurrent same-arch AND cross-arch boots are collision-safe. The only thing that
was serial was the *orchestrator* — i.e. the human. This driver removes that:

  1+2. builds + lanes, PIPELINED (F5, 2026-07-06) — every arch's build is
     submitted at once (they self-throttle on the broker's build share, CPU-sized
     concurrency fleet-wide; amd64 is submitted first, since it feeds two lanes),
     and each arch's smoke, its strict guards, and (for amd64) the net lane launch
     THE MOMENT its own build lands rather than behind an all-arches barrier.
     Safe by hostlock design:
     windows-host boots run on the Windows scheduling domain (`bootcpu` pool)
     and coexist with the next WSL-side build (build ∥ boot is the documented
     contract). Opt out of builds with --no-build (all lanes launch at once).
     The lint gate runs disjoint source-early/source-deferred partitions once:
     audited source-only checks start immediately, others after all builds.
     It combines both with `check --strict --scope arch --arch <a>` per built arch:
     source lints baseline-diff NEW=0, cargo-link/committed-drift bit-rot guards,
     and per-arch binary lints. Each partition uses the existing strict checkers.
     Both partitions must pass before stamping. A NEW finding
     (classification LINT_FAIL) blocks the push stamp; smokes still run so the
     matrix stays informative. Opt out with --no-lint.
     `check-tooling` (OUR python: ruff/pyright/pytest over rustconv + cli +
     tools) runs ONCE alongside — it is arch-independent and needs no build, so
     it starts before the first build. A failure classifies TOOLING_FAIL and
     blocks the stamp. It shares the --no-lint opt-out.
  3. flake-retries — a *documented* flake (KNOWN_FLAKE / FLAKY), an environment
     failure (LAUNCH_FAIL), or a contention-characterized HANG_TIMEOUT (serial
     still flowing at the deadline AND no kernel census/lockdep wedge
     self-report) triggers up to --retries concurrent fresh attempts; the arch
     passes on the first green. A fresh regression (BUGCHECK / RUST_PANIC /
     FAIL_MARKER / SHELL_NOT_READY / STALE_MEDIA / a wedged or self-confirmed
     HANG_TIMEOUT / UNKNOWN) is NEVER retried — retrying would hide it, which
     is exactly what the North Star forbids.
  4. a results matrix — every arch's final verdict + attempt count + flake note.
     Exit 0 only if every arch is ultimately green.

Concurrency is safe to over-submit: each smoke subprocess self-throttles on the
host boot-CPU pool (hostlock `bootcpu`, capacity-derived — 8 on a 24-vCPU box)
and the global QEMU cap (vm.py QEMU_MAX_CONCURRENT=16); each build self-throttles
on the cpuheavy build share. The executor just needs enough workers to not
re-serialize; the broker does the real throttling.

A green FULL gate mints the push stamp for THE TREE IT GATED (sampled at start —
see _write_gate_stamp). `vibeos-cli land` is the front door: it holds the fleet
land lock across gate+push so master cannot move underneath, which is what makes
a green gate always landable instead of a lottery ticket.
"""

from __future__ import annotations

import concurrent.futures as cf
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

from . import archcfg, canon, hostdisk, hostlock, lints, rustlints, triage
from . import ladder as ladder_mod
from .artifacts import Run, Stage, is_incomplete

# A failure with one of these classifications is a *documented* flake or an
# environment failure — safe to retry. Anything else (a BUGCHECK, a Rust panic,
# a hit fail-marker, stale media, an unknown red) is a real regression and fails
# the arch immediately: retrying a regression away is what the North Star forbids.
#   - LAUNCH_FAIL: a QEMU/interop launch hiccup (env, not the OS).
#   - NET_FLAKE: boots green + net suite failed under concurrent-smoke load (the
#     documented amd64 timing flake, see smoke.py _net_stage) — a real netstack
#     regression fails the retry too, so it still ends red.
#   - HOST_STARVED: the net lane never booted — it spent its whole budget queued
#     on the hostlock lease broker (field manual §9.84: another lane's farm/builds
#     own the host). Retryable because the gate's retry phase runs after its own
#     parallel lanes released their leases, so self-inflicted contention clears;
#     an external holder keeps the retry starved and the final red then SAYS
#     HOST_STARVED instead of masquerading as a netstack flake.
#
# HANG_TIMEOUT is deliberately NOT a blanket member: "HANG_TIMEOUT ≈ host
# contention" dates from the native-KVM era, but on the default windows-host TCG
# backend the dominant real bugs (service wedge, lost wakeups, silent kernel
# panics) present exactly as HANG_TIMEOUT — a ~50% wedge passed a 3-attempt gate
# ~87% of the time. It is retryable ONLY when smoke characterized it as
# slow/contended (serial still flowing at the deadline) AND the kernel didn't
# self-report stuck threads — see _retryable().
# The payload/baseline suite name, from its owner — not re-spelled here, so the two
# places that stamp VIBEOS_PAYLOAD cannot drift apart.
from .conformance import SUITE as SUITE_NAME

RETRYABLE = {
    "KNOWN_FLAKE",
    "FLAKY",
    "LAUNCH_FAIL",
    "NET_FLAKE",
    "HOST_STARVED",
    # A conformance ratchet miss retries for the same reason NET_FLAKE does: the
    # identity floor marks the bistables it KNOWS about, and one that never fired
    # during seeding looks exactly like a regression on a single sweep. A real
    # regression reproduces on the retry; a coin-flip does not. See _conformance_once.
    # RED, not FLAKE: the label is what a lane reads to decide investigate-vs-requeue, and
    # "FLAKE, 3 attempts" invited a blind resubmit of a failure that reproduced 9/9.
    "CONFORMANCE_RED",
}


def _retryable(classification: str, summary: dict) -> bool:
    """Retry policy: documented flakes and environment failures retry; a
    HANG_TIMEOUT retries only when it looks like contention (smoke's
    hang_kind == "slow": serial was still flowing at the deadline) and the
    kernel's census/lockdep instrumentation did NOT confirm a wedge."""
    if classification in RETRYABLE:
        return True
    if classification == "HANG_TIMEOUT":
        return summary.get("hang_kind") == "slow" and not summary.get("wedge_confirmed")
    return False


# Legacy-BIOS boot of the amd64 media. The BIOS loader was pruned 2026-05-29 and
# nothing booted it, so nothing noticed it was gone; restored 2026-10-05 for
# Libreboot/SeaBIOS machines, it is a gate lane so it cannot rot unseen again.
BIOS_LANE = "amd64-bios"


def _smoke_once(repo: Path, arch: str, root: Path, extra: list[str]) -> dict:
    """Run one `vibeos-cli smoke --arch <arch>` in its own artifact root so its
    summary.json is locatable deterministically. Returns a compact result dict."""
    root.mkdir(parents=True, exist_ok=True)
    cmd = [str(repo / "vibeos-cli"), "smoke", "--arch", arch, *extra]
    env = dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root))
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=str(repo), env=env, capture_output=True, text=True)
    dt = time.monotonic() - t0
    # ALWAYS persist the child's console. When smoke dies before it can write a
    # summary.json, this is the ONLY record of why -- and it used to be dropped on
    # the floor, which let the gate reject a lane on `smoke:arm64 FAIL` with an
    # empty salient and an empty run dir. Nothing to read, nobody to blame.
    console = (proc.stdout or "") + (("\n--- stderr ---\n" + proc.stderr) if proc.stderr else "")
    try:
        (root / "smoke-console.log").write_text(console)
    except OSError:
        pass
    summary = _read_summary(root)
    if not summary:
        # No summary.json at all: the boot never got far enough to have a verdict.
        # That is NOT the same as "the OS failed to boot", so don't launder it into
        # a plain FAIL -- name it, and carry the exit code plus the child's last
        # words so the failure is attributable to a lane or to the harness.
        return {
            "arch": arch,
            "verdict": "FAIL",
            "classification": "SMOKE_NO_SUMMARY",
            "ok": False,
            # Retryable: an abort this early is usually the host, not the OS. A
            # DETERMINISTIC cause still fails the retry and ends red, so this can
            # never manufacture a green -- and a recovery is reported as
            # "first pass was SMOKE_NO_SUMMARY", so the harness bug stays visible.
            "retryable": True,
            "exit": proc.returncode,
            "seconds": dt,
            "run_dir": str(root),
            "salient": (
                f"smoke --arch {arch} exited {proc.returncode} after {dt:.1f}s without "
                f"recording a verdict — it died before the boot could return one. "
                f"Last output: {_last_words(console)}  (full: {root / 'smoke-console.log'})"
            ),
        }
    verdict = summary.get("verdict", "FAIL" if proc.returncode else "OK")
    classification = summary.get("classification", "") or ("" if proc.returncode == 0 else "FAIL")
    return {
        "arch": arch,
        "verdict": verdict,
        "classification": classification,
        "ok": verdict in ("OK", "WARN"),
        "retryable": _retryable(classification, summary),
        "hang_kind": summary.get("hang_kind"),
        "wedge_confirmed": bool(summary.get("wedge_confirmed")),
        "census_stuck": summary.get("census_stuck", 0),
        "exit": proc.returncode,
        "seconds": dt,
        "run_dir": summary.get("_dir", str(root)),
        "flake_rate": summary.get("flake_rate"),
        "screenshot": summary.get("screenshot"),
        "salient": _first_salient(summary),
    }


def _last_words(console: str, limit: int = 240) -> str:
    """The last non-empty console line, for a one-line 'why did it die' salient."""
    for line in reversed(console.splitlines()):
        if line.strip():
            return line.strip()[:limit]
    return "(no output at all)"


def _read_summary(root: Path) -> dict:
    """The smoke subprocess writes <root>/<ts>-smoke-<arch>/summary.json. The root
    is unique per attempt, so glob finds exactly one."""
    hits = sorted(root.glob("*/summary.json"))
    if not hits:
        return {}
    try:
        data = json.loads(hits[-1].read_text())
    except (OSError, ValueError):
        return {}
    if is_incomplete(data):
        # An interrupted-run MARKER, not a verdict: the child was killed before it
        # could finalize. Report it as "no summary" so the branch below fires -- that
        # path keeps the child's exit code and its last console words, which is
        # strictly more than the marker says. The marker's job is to stop a caller
        # reading a half-written run dir as a pass, not to replace this diagnosis.
        return {}
    data["_dir"] = str(hits[-1].parent)
    return data


def _first_salient(summary: dict) -> str:
    for st in summary.get("stages", []):
        if st.get("status") in ("fail", "warn"):
            for s in st.get("salient", []):
                return str(s)
    return ""


def _ensure_kmtest_payload(repo: Path, arches: list[str]) -> tuple[bool, str]:
    """Make the kmtest sweep runner exist for every arch, or say why it cannot.

    THIS FUNCTION IS A SCAR. Wiring the conformance lanes in made the gate build stamp
    VIBEOS_PAYLOAD=kmtest unconditionally, and `kmtest-sweep-<arch>.exe` is a GITIGNORED
    build product: `git worktree add` brings sweep.c and the test lists but not the exes.
    So the change worked in my own worktree, where months of conformance runs had left
    them lying around, and broke the FIRST fresh tree it met — the merge queue's own
    integration worktree — with

        payloads/compile.py: ERROR: payload 'kmtest': src missing: .../kmtest-sweep-i386.exe

    which surfaces as the livecd media step failing, i.e. a BUILD_FAIL attributed to
    whichever innocent lanes happened to be in the batch. Every lane in the fleet would
    have inherited that. conformance.py already had `_worktree_provisioning_gaps` for
    exactly this class; the gate path simply went around it.

    Building it is cheap and correct: sweep.c is committed, the recipe is one
    cross-compile per arch, and the toolchain is the one thing AGENTS.md explicitly
    allows sharing between worktrees because it is content-addressed. What we must NOT
    do is copy the exe from a sibling worktree — that is the fleet-isolation trap that
    had verify_signatures grading one tree's Rust against another tree's headers.
    """
    script = repo / "tools" / "payloads" / SUITE_NAME / "build.sh"
    if not script.is_file():
        return False, f"no {script}"
    # Missing OR STALE: a worktree that already has an exe keeps it across every later
    # merge, so a change to sweep.c (the 2026-09-25 PnP-quiescence gate) never reached
    # the merge queue's integration worktree, which went on gating with the old sweep and
    # rejected the change for exactly the failures that gate exists to prevent.
    src_dir = repo / "tools" / "payloads" / SUITE_NAME
    newest_src = max(
        (p.stat().st_mtime for p in (src_dir / "sweep.c", script) if p.is_file()), default=0.0
    )

    def _needs_build(a: str) -> bool:
        exe = src_dir / f"{SUITE_NAME}-sweep-{a}.exe"
        return not exe.is_file() or exe.stat().st_mtime < newest_src

    missing = [a for a in arches if _needs_build(a)]
    if not missing:
        return True, ""
    tc = repo / ".build-tools" / "llvm-mingw"
    if not tc.exists() and not list(repo.parent.glob("llvm-mingw-*")):
        # Without the pinned clang we cannot build it, and we must not stamp the
        # payload either — that is what breaks the media step. Report, do not guess.
        return False, f"missing {tc} (pinned clang), needed to build {', '.join(missing)}"
    for a in missing:
        cp = subprocess.run(["bash", str(script), a], cwd=str(repo), capture_output=True, text=True)
        if cp.returncode != 0:
            tail = ((cp.stderr or cp.stdout or "").strip().splitlines() or ["(no output)"])[-1]
            return False, f"build.sh {a} failed: {tail}"
    return True, ""


def _conformance_once(repo: Path, arch: str, root: Path) -> dict:
    """Run the kmtest conformance ratchet for one arch as its own gate lane.

    The ratchet compares which ASSERTIONS fail against the per-arch identity floor in
    `tools/conformance-baseline/<arch>/kmtest.txt`, so it catches what the smoke cannot:
    the smoke asks "does the desktop come up", and every one of these regressions comes
    up to a perfectly good desktop with a kernel API quietly answering wrong.

    It reuses THIS GATE'S MEDIA (`--no-build`), which is why the gate's build carries the
    kmtest payload when conformance is on. The alternative — a second build per arch —
    would roughly double the gate's build cost, and builds are the fleet's actual
    bottleneck: the host broker runs 4/4 build permits with lanes queuing 12+ minutes
    while the boot pool sits at 1/8. Boots are the cheap resource; spend those.

    RETRY SEMANTICS, and why they are not optional. The floor marks known-bistable
    identities `assert?`, but the corpus is not closed — a bistable that simply did not
    fire during seeding is indistinguishable from a regression on any single sweep. So a
    first-attempt failure is CONFORMANCE_RED and retryable, exactly as NET_FLAKE is: a
    real regression reproduces, a coin-flip does not. Without this, one unlucky boot
    would reject a whole merge-queue batch of innocent lanes.
    """
    t0 = time.monotonic()
    root.mkdir(parents=True, exist_ok=True)
    baseline = repo / "tools" / "conformance-baseline" / arch / "kmtest.txt"
    if not baseline.is_file():
        # No floor means nothing to ratchet against. Skip loudly rather than passing
        # silently: a gate stage that cannot fail must not read as one that held.
        return {
            "arch": f"conf:{arch}",
            "verdict": "SKIP",
            "classification": "",
            "ok": True,
            "retryable": False,
            "seconds": 0.0,
            "run_dir": str(root),
            "salient": f"no baseline at {baseline}",
        }
    cmd = [
        str(repo / "vibeos-cli"),
        "conformance",
        "--arch",
        arch,
        "--gate",
        "--no-build",
    ]
    cp = subprocess.run(cmd, cwd=str(repo), capture_output=True, text=True, env=dict(os.environ))
    out = (cp.stdout or "") + (cp.stderr or "")
    (root / "conformance.log").write_text(out)
    ok = cp.returncode == 0
    # The regression lines are the only part worth carrying up; the sweep prints a
    # per-test table that would drown the gate summary.
    salient = [ln.strip() for ln in out.splitlines() if "NEW failing assertion" in ln]
    if not salient and not ok:
        salient = [ln.strip() for ln in out.splitlines() if "->" in ln and ":" in ln][:6]
    return {
        "arch": f"conf:{arch}",
        "verdict": "PASS" if ok else "RATCHET",
        # RED says what was observed -- the ratchet failed -- and nothing about why. It was
        # CONFORMANCE_FLAKE, a CAUSE the gate never established: a deterministic regression
        # that reproduced on every attempt carried the same word as a coin flip, and a lane
        # reading "FLAKE" requeues instead of looking at its own diff.
        "classification": "" if ok else "CONFORMANCE_RED",
        "ok": ok,
        "retryable": not ok,
        "seconds": time.monotonic() - t0,
        "run_dir": str(root),
        "salient": "; ".join(salient[:6]) or ("" if ok else f"conformance exited {cp.returncode}"),
    }


# A stortest boot is ~80-100 s; the outer timeout only catches a wedged pytest, which the
# test's own 240 s verdict wait and the vm fixture's boot budget should never reach.
STORTEST_TIMEOUT_S = 900


def _stortest_once(repo: Path, arch: str, root: Path) -> dict:
    """tests/qemu/test_stortest.py for one arch, on THIS GATE'S MEDIA: stortest.exe reads,
    writes and reads back a VIBEUSB stick (usbstor + fastfat + mountmgr), and the write is
    then read off the stick image on the host. Nothing else in the gate writes a file
    through a USB disk; smoke boots from read-only media.

    The test refuses stale media itself (its preflight is check_staleness), and the media
    patch it boots is an equal-length copy, so no rebuild is ever involved. Its verdict
    comes back as JSON through VIBEOS_STORTEST_RESULT; a boot that never reached the
    desktop writes none and is classified from the console with triage, so a red lane
    always names a cause (BUGCHECK / HANG_TIMEOUT / STORTEST_FAIL / STORTEST_MISMATCH).
    Only a failure that never reached a verdict retries: a stortest FAIL or a wrong
    readback is the storage stack answering wrong, which reproduces."""
    t0 = time.monotonic()
    root.mkdir(parents=True, exist_ok=True)
    result = root / "stortest.json"
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        str(repo / "tests" / "qemu" / "test_stortest.py"),
        "--arch",
        archcfg.vm_arch(arch),
        "--skip-clean-build",
        "-q",
        "-p",
        "no:cacheprovider",
    ]
    env = dict(
        os.environ,
        VIBEOS_QEMU_STORTEST="1",
        VIBEOS_STORTEST_RESULT=str(result),
        VIBEOS_ARTIFACT_DIR=str(root),
    )
    try:
        cp = subprocess.run(
            cmd,
            cwd=str(repo),
            env=env,
            capture_output=True,
            text=True,
            timeout=STORTEST_TIMEOUT_S,
        )
        out, rc = (cp.stdout or "") + (cp.stderr or ""), cp.returncode
    except subprocess.TimeoutExpired as e:
        out = (e.stdout or b"").decode(errors="replace") if isinstance(e.stdout, bytes) else ""
        rc = -1
    (root / "stortest.log").write_text(out)
    try:
        verdict = json.loads(result.read_text())
    except (OSError, ValueError):
        verdict = None
    if verdict is not None:
        classification = verdict.get("classification", "")
        salient = "; ".join(verdict.get("salient", [])[:4])
    elif rc == 0:
        # pytest passed without the test writing its verdict: it was skipped.
        classification, salient = "STORTEST_NO_VERDICT", "pytest passed but recorded no verdict"
    elif "Boot failed" in out:
        # The vm fixture's boot never reached the desktop; its message carries the
        # serial's head and tail, which is what triage reads.
        v = triage.classify(out, reached=False, fail_marker=None, debugcon=None)
        classification = v["classification"] or "STORTEST_BOOT_FAIL"
        salient = "; ".join(v.get("salient", [])[:3]) or _last_words(out)
    else:
        # pytest itself failed (collection, a fixture, a timeout): say so, with its error.
        errors = [ln[1:].strip() for ln in out.splitlines() if ln.startswith("E ")]
        errors = [e for e in errors if e]
        classification = "STORTEST_ERROR"
        salient = (errors[0] if errors else _last_words(out))[:300]
    ok = rc == 0 and verdict is not None and not classification
    return {
        "arch": f"stor:{arch}",
        "verdict": "PASS" if ok else "FAIL",
        "classification": "" if ok else classification,
        "ok": ok,
        # Retry only what never reached stortest's own verdict (the boot, or the host).
        "retryable": not ok and verdict is None,
        "exit": rc,
        "seconds": time.monotonic() - t0,
        "run_dir": str(root),
        "salient": "" if ok else f"{salient}  (log: {root / 'stortest.log'})",
    }


def _net_once(repo: Path, root: Path) -> dict:
    """Run the amd64 netstack suite as its OWN gate lane (a full amd64-uefi-net
    boot + 15 tests, ~110-150s). Historically this ran INSIDE `smoke --arch
    amd64` serially after the plain boot (smoke.py _net_stage), which made the
    amd64 lane two-boots-long (~200s) and THE gate critical path. As a parallel
    lane the gate smoke wall drops to max(plain boots, net suite). Mirrors
    _net_stage's semantics: a first-attempt failure is classified NET_FLAKE
    (documented 2026-07-04 timing flake under concurrent-smoke load) and is
    gate-retryable; a real regression fails the retry too and ends red."""
    import sys as _sys

    root.mkdir(parents=True, exist_ok=True)
    test_file = repo / "tests" / "qemu" / "test_net.py"
    if not test_file.exists():
        return {
            "arch": "net",
            "verdict": "SKIP",
            "classification": "",
            "ok": True,
            "retryable": False,
            "seconds": 0.0,
            "run_dir": str(root),
            "salient": "",
        }
    # --skip-clean-build: the gate's own build stage produced the media this
    # lane boots (builds strictly precede smokes), so the conftest fixture's
    # nuke-cargo-target + `ninja livecd` re-run is redundant here — measured
    # 17.6s of the lane's wall. Standalone `pytest test_net.py` keeps the
    # clean-build guard by default (raw pytest has no staleness gate).
    cmd = [
        _sys.executable,
        "-m",
        "pytest",
        str(test_file),
        "--arch",
        "amd64-uefi-net",
        "--timeout",
        "600",
        "-q",
        "-p",
        "no:cacheprovider",
        "--skip-clean-build",
    ]
    t0 = time.monotonic()
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(repo),
            capture_output=True,
            text=True,
            timeout=1200,
            env={**os.environ, "VIBEOS_ARTIFACT_DIR": str(root)},
        )
    except subprocess.TimeoutExpired:
        return {
            "arch": "net",
            "verdict": "FAIL",
            "classification": "NET_HANG",
            "ok": False,
            "retryable": False,
            "seconds": time.monotonic() - t0,
            "run_dir": str(root),
            "salient": "tests/qemu/test_net.py hung > 1200s",
        }
    dt = time.monotonic() - t0
    out_all = (proc.stdout or "") + "\n" + (proc.stderr or "")
    try:
        (root / "pytest.out").write_text(out_all)
    except OSError:
        pass
    tail = (proc.stdout or "").strip().splitlines()
    summary = next(
        (ln for ln in reversed(tail) if " passed" in ln or " failed" in ln or " error" in ln),
        tail[-1] if tail else "",
    )
    ok = proc.returncode == 0
    # §9.84: hostlock announced the queue but never the acquisition → the suite's flat
    # timeouts mean it NEVER BOOTED. Name the real cause instead of "NET_FLAKE" so a red
    # gate doesn't send anyone chasing a netstack bug that is actually a busy host.
    starved = (not ok) and hostlock.looks_starved(out_all)
    return {
        "arch": "net",
        "verdict": "OK" if ok else "FAIL",
        "classification": "" if ok else ("HOST_STARVED" if starved else "NET_FLAKE"),
        "ok": ok,
        "retryable": not ok,
        "seconds": dt,
        "run_dir": str(root),
        "salient": ""
        if ok
        else (
            "net lane never booted — queued on host CPU leases the whole run (§9.84); "
            "`vibeos-cli ps` names the holder"
            if starved
            else summary.strip("= ")
        ),
    }


def _build_once(
    repo: Path, run: Run, arch: str, args, *, defer_wow64: bool = False, restage: bool = False
) -> dict:
    """Build one arch as its own gate lane.

    All architectures submit together; hostlock admits them against the shared
    CPU pool and the per-build-directory lock protects each output tree.
    Source lint runs in the gate partitions, independently of compilation.

    `defer_wow64` / `restage`: amd64's media stages the i386 SysWOW64 set out of
    build-i386 at CONFIGURE time, so building it BESIDE i386 snapshots a tree that
    is mid-rebuild (a cold one has no kernel32.dll at all). The concurrent build
    defers the staging; the restage is a warm amd64 build, never --clean, run
    once i386 has finished.
    """
    cmd = [str(repo / "vibeos-cli"), "build", "--arch", arch]
    if not getattr(args, "no_lint", False):
        cmd.append("--no-check")
    if getattr(args, "clean", False) and not restage:
        cmd.append("--clean")
    if getattr(args, "artifact_store", None):
        cmd.extend(["--artifact-store", str(args.artifact_store)])
        if getattr(args, "artifact_store_verify", False) and not restage:
            cmd.append("--artifact-store-verify")
    if defer_wow64:
        cmd.append("--no-wow64-stage")
    if getattr(args, "keep_going", False):
        cmd.append("--keep-going")
    env = None
    if not getattr(args, "no_conformance", False):
        # Stage the kmtest corpus into THIS media so the conformance lanes can boot it
        # with --no-build instead of each doing its own build. The payload selection is
        # stamped into the build dir (payload-harness-design.md), so it must be set here,
        # on the build the gate actually runs — a gate build with no selection stamps
        # `<none>`, and the conformance lane would then boot fine and report
        # `[KMTEST-SWEEP] launch FAILED` minutes later, a layer away from the cause.
        #
        # This only ADDS files to the ramdisk; the smoke boots the same media and still
        # asserts the same desktop marker, and nothing launches the corpus unless asked.
        cmd.append("--kmtests")
        env = dict(os.environ, VIBEOS_PAYLOAD="kmtest")
    t0 = time.monotonic()
    cp = run.run(cmd, cwd=repo, stage=f"build:{arch}{':restage' if restage else ''}", env=env)
    return {
        "arch": arch,
        "ok": cp.returncode == 0,
        "exit": cp.returncode,
        "seconds": time.monotonic() - t0,
        "classification": _child_classification(cp.stdout or ""),
    }


def _child_classification(stdout: str) -> str:
    """The classification the child `vibeos-cli build` gave ITSELF, recovered from the
    artifact dir it prints on its last line.

    Without this the gate reports `classification: ""` for a build failure it fully
    understands, and "" is in queue.INCONCLUSIVE *by design* — it means "no information".
    So a build that failed for a named, reproducible reason was indistinguishable from a
    host hiccup: the merge-queue train halted, rejected nobody, and re-gated the entire
    batch. That happened 5 times on 2026-07-25 (all arm64 BUILD_INCOMPLETE), each costing a
    full 3-build/4-boot gate cycle, and it is most of why the land queue reached 8 deep with
    the oldest waiting 54 minutes. The halt itself is still correct for a genuine unknown;
    what was wrong is calling a known failure unknown.
    """
    if re.search(r"^DISK_ADMISSION_FAIL:", stdout, re.M):
        return "DISK_ADMISSION_FAIL"
    m = re.search(r"^full log:\s*(\S+)/full\.log\s*$", stdout, re.M)
    if not m:
        return ""
    try:
        summary = json.loads((Path(m.group(1)) / "summary.json").read_text())
    except (OSError, ValueError):
        return ""
    return str(summary.get("classification") or "")


def _lint_once(repo: Path, arch: str, root: Path) -> dict:
    """Run one strict partition: shared source, or one arch's complete guards.

    Each child is explicitly partial. Only the parent gate combines ALL of them
    into a full verdict; a failing or absent partition must block its stamp.
    """
    root.mkdir(parents=True, exist_ok=True)
    cmd = [str(repo / "vibeos-cli"), "check", "--strict", "--scope"]
    cmd += [arch] if arch.startswith("source") else ["arch", "--arch", arch]
    env = dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root))
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=str(repo), env=env, capture_output=True, text=True)
    dt = time.monotonic() - t0
    salient = ""
    for ln in (proc.stdout or "").splitlines():
        if "✗" in ln or "FAIL" in ln:
            salient = ln.strip()
            break
    inventory_ok = True
    summary = _read_summary(root)
    classification = summary.get("classification") or ""
    if lints._scanner_error(proc, "stdout"):
        classification = "SCANNER_ERROR"
    if arch in ("source-early", "source-deferred"):
        expected = [name for name, _, _ in lints.source_partition_specs(repo, arch)]
        inventory_ok = summary.get("checked_source_checks") == expected
        if not inventory_ok:
            salient = "missing or incorrect source-partition coverage inventory"
    return {
        "arch": arch,
        "ok": proc.returncode == 0 and inventory_ok and classification != "SCANNER_ERROR",
        "seconds": dt,
        "salient": salient,
        "exit": proc.returncode,
        "classification": "SCANNER_ERROR" if not inventory_ok else classification,
    }


def _cohab_once(repo: Path, arch: str, root: Path) -> dict:
    """Run the native cohab coverage plugin for one built arch.

    This lives here rather than in `check --strict` for one reason: it reads OBJECT FILES.
    ordinary `check` can run without a build, and the whole point of
    this verifier is that the source cannot answer the question — a cohab flip is correct only
    if the symbols the Rust actually EMITS match the ones the C object actually EXPORTS, after
    every `-D`, `#ifdef` and ABI decoration. So it launches with the arch's smoke, the moment
    that arch's build lands.

    It exists because `cargo check` never codegens: `global_asm!` is never assembled and
    `#[export_name]` never becomes a symbol, so a whole class of flip bug is invisible to every
    gate stage that ran before this one. Eight msvcrt stdcall stubs shipped as
    `___adj_fdiv_m16i@4@4` — `#[export_name]` STACKS with i386 PE decoration rather than
    replacing it — and three more symbols were defined under names nothing imports. All of it
    built, linked and booted green: the Rust exported symbols nobody asked for, the C bodies
    stayed live, and the "flip" was a no-op nothing could see.
    """
    root.mkdir(parents=True, exist_ok=True)
    t0 = time.monotonic()
    proc = rustlints.run_binary(
        repo,
        arch,
        ["verify_cohab_coverage"],
        lambda cmd: subprocess.run(cmd, cwd=repo, capture_output=True, text=True),
    )["verify_cohab_coverage"]
    dt = time.monotonic() - t0
    console = (proc.stdout or "") + (("\n--- stderr ---\n" + proc.stderr) if proc.stderr else "")
    try:
        (root / "cohab-console.log").write_text(console)
    except OSError:
        pass
    # Findings go to stdout one per line; the coverage tally and every hard-fail
    # (stale archive, missing build dir) go to stderr. Prefer a finding, fall back
    # to the last words so a refusal-to-judge never reports a bare "exit 1".
    salient = next((ln.strip() for ln in (proc.stdout or "").splitlines() if ".rs:" in ln), "")
    if not salient and proc.returncode:
        salient = (
            f"exit {proc.returncode}: {_last_words(console)}  (full: {root / 'cohab-console.log'})"
        )
    return {
        "arch": arch,
        "ok": proc.returncode == 0,
        "seconds": dt,
        "salient": salient,
        "exit": proc.returncode,
    }


def _tooling_once(repo: Path, root: Path) -> dict:
    """Run `vibeos-cli check-tooling` once for the whole gate.

    This is OUR Python (rustconv / vibeos-cli / tools/*.py): ruff, ruff-format,
    pyright, the generated-file staleness checks, and the pytest suites. It is
    arch-independent and needs no build artifacts, so it runs once and can start
    before the first build rather than per-arch like `check --strict`.

    It gates the push stamp for the same reason the strict lint does: without it
    the gate is structurally blind to Python regressions, and master shipped
    `check-tooling` red on two consecutive landings (a mutable-default-argument
    sweep, then four pyright errors in the msvcrt `--target` work) because
    nothing between the author and the stamp ever ran it.
    """
    root.mkdir(parents=True, exist_ok=True)
    cmd = [str(repo / "vibeos-cli"), "check-tooling"]
    env = dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root))
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=str(repo), env=env, capture_output=True, text=True)
    dt = time.monotonic() - t0
    # Persist the console. Without this a TOOLING_FAIL leaves NOTHING to read: the
    # stage dir holds only whatever the pytest suites happened to drop there, the
    # gate's own full.log never sees this subprocess, and the salient below is a
    # single line scraped by a marker search that can miss. A red gate whose cause
    # you cannot look up stalls the merge train for everyone.
    console = (proc.stdout or "") + (("\n--- stderr ---\n" + proc.stderr) if proc.stderr else "")
    try:
        (root / "check-tooling-console.log").write_text(console)
    except OSError:
        pass
    salient = ""
    for ln in (proc.stdout or "").splitlines():
        if "\u2717" in ln or "FAIL" in ln:
            salient = ln.strip()
            break
    if not salient and proc.returncode:
        # The marker scan came up empty (ruff/pyright do not print \u2717, and a
        # traceback goes to stderr) \u2014 fall back to the child's last words rather
        # than reporting a bare "exit 1".
        salient = (
            f"exit {proc.returncode}: {_last_words(console)}  "
            f"(full: {root / 'check-tooling-console.log'})"
        )
    return {
        "ok": proc.returncode == 0,
        "seconds": dt,
        "salient": salient,
        "exit": proc.returncode,
        "classification": _child_classification(proc.stdout or ""),
    }


def _difftest_once(repo: Path, root: Path) -> dict:
    """Run every host differential harness once for the whole gate.

    These grade the shipping Rust against the genuine C it replaced, in one
    process, on the host — no QEMU, no build-<arch>, ~20s for the set. So they
    run alongside check-tooling, before the first build finishes.

    They gate the push stamp because the alternative was demonstrated: reachable
    only by hand, `rtl-difftest` sat 2/87 red on master for long enough that the
    breaking commit is no longer identifiable, and one of the two was a real
    divergence in eleven `Rtl*String` converters. Smoke proves the OS boots;
    these prove a translation still means what the C meant, which is a different
    question and the one this project is actually answering.
    """
    root.mkdir(parents=True, exist_ok=True)
    cmd = [str(repo / "vibeos-cli"), "difftest"]
    env = dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root))
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=str(repo), env=env, capture_output=True, text=True)
    dt = time.monotonic() - t0
    # Same evidence rule as _tooling_once: a marker-scraped salient is a summary,
    # not a record. Keep the console so a red difftest can be diagnosed at all.
    console = (proc.stdout or "") + (("\n--- stderr ---\n" + proc.stderr) if proc.stderr else "")
    try:
        (root / "difftest-console.log").write_text(console)
    except OSError:
        pass
    salient = ""
    for ln in (proc.stdout or "").splitlines():
        if ln.strip().startswith(">") or "✗" in ln:
            salient = ln.strip().lstrip("> ")
            break
    if not salient and proc.returncode:
        salient = (
            f"exit {proc.returncode}: {_last_words(console)}  "
            f"(full: {root / 'difftest-console.log'})"
        )
    return {"ok": proc.returncode == 0, "seconds": dt, "salient": salient, "exit": proc.returncode}


def _resolve_arches(raw: str | None) -> list[str]:
    if not raw:
        return ["i386", "amd64", "arm64"]
    out: list[str] = []
    for tok in raw.split(","):
        tok = tok.strip()
        if not tok:
            continue
        out.append(archcfg.base_arch(tok))
    return out


# vCPUs for every smoke leg the gate runs (see smoke_extra in gate()).
GATE_SMP = 2
FULL_ARCH_SET = {"i386", "amd64", "arm64"}


def _prune_stale_stamps(stamps_dir: Path, max_age_s: int = 24 * 3600) -> None:
    """Drop per-tree gate stamps older than max_age_s so the store self-cleans.
    (The pre-push hook already ignores stale stamps; this just keeps the dir tidy.)"""
    now = time.time()
    for f in stamps_dir.glob("*.json"):
        try:
            if now - f.stat().st_mtime > max_age_s:
                f.unlink()
        except OSError:
            pass


def _tree_state(repo: Path) -> tuple[str, bool]:
    """(HEAD's tree hash, is-dirty) — sampled at gate START so the stamp can certify the
    content the gate actually validated. See _write_gate_stamp."""

    def _git(*a: str) -> str:
        return subprocess.run(
            ["git", *a], cwd=repo, capture_output=True, text=True, timeout=60
        ).stdout.strip()

    return (
        _git("rev-parse", "HEAD^{tree}"),
        bool(_git("status", "--porcelain", "--untracked-files=no")),
    )


def _write_gate_stamp(
    run: Run, repo: Path, arches: list[str], gated: tuple[str, bool] | None = None
) -> None:
    """T1.3 (mechanized North Star): a green FULL gate mints the push
    credential — the pre-push hook (tools/git-hooks/pre-push) refuses updates
    to master whose tree doesn't match a fresh green stamp. Only a CLEAN tree
    is stamped: a dirty tree gates content no commit will ever hash to.
    Partial-arch gates never stamp (the North Star is all three).

    `gated` is (tree, dirty) sampled at gate START. It must match the tree still checked
    out at the end, because this function used to read HEAD's tree at stamp time and
    certify THAT — so a commit made while the gate ran (an agent tidying up during a
    20-minute gate) minted a credential for content nothing had built or booted
    (2026-07-25). The stamp is a claim about what was validated; it must be exactly that
    or nothing."""
    if not FULL_ARCH_SET.issubset(set(arches)):
        run.log("gate green but partial arch set — no push stamp minted", header="stamp")
        return

    def _git(*a: str) -> str:
        return subprocess.run(
            ["git", *a], cwd=repo, capture_output=True, text=True, timeout=60
        ).stdout.strip()

    try:
        dirty = _git("status", "--porcelain", "--untracked-files=no")
        if dirty:
            run.log(
                "gate green but the tree is DIRTY — no push stamp (commit first, "
                "then re-gate; the stamp must hash the exact content that was gated)",
                header="stamp",
            )
            run.add(Stage("stamp", status="warn", detail="dirty tree — no push stamp"))
            return
        tree = _git("rev-parse", "HEAD^{tree}")
        if gated is not None:
            start_tree, start_dirty = gated
            if start_dirty:
                run.log(
                    "gate green but the tree was DIRTY when the gate STARTED — what ran is "
                    "not what any commit hashes to, so nothing here can be certified. "
                    "Commit first, then gate.",
                    header="stamp",
                )
                run.add(Stage("stamp", status="warn", detail="dirty at start — no push stamp"))
                return
            if start_tree and tree != start_tree:
                run.log(
                    f"gate green but the checkout MOVED during the gate "
                    f"({start_tree[:12]} → {tree[:12]}) — the boots validated the old tree, "
                    "so stamping the new one would certify content nothing built. "
                    "Re-gate the tree you mean to push (`vibeos-cli land` holds the fleet "
                    "land lock across gate+push precisely so this cannot happen).",
                    header="stamp",
                )
                run.add(
                    Stage("stamp", status="warn", detail="tree changed mid-gate — no push stamp")
                )
                return
        head = _git("rev-parse", "HEAD")
        common = Path(_git("rev-parse", "--git-common-dir"))
        if not common.is_absolute():
            common = (repo / common).resolve()
        stamp = {
            "tree": tree,
            "head": head,
            "time": int(time.time()),
            "arches": sorted(arches),
            "worktree": str(repo),
        }
        # Content-addressed stamp store: one file per gated TREE hash, so
        # concurrent lanes gating DIFFERENT trees never clobber each other. The
        # old single shared vibeos-gate-stamp.json was last-writer-wins across
        # all worktrees → a guaranteed race whenever two lanes gated at once
        # (2026-07-07). Keyed by tree (the content identity the hook validates),
        # atomic write (temp + rename), self-pruning past 24h.
        stamps_dir = common / "vibeos-gate-stamps"
        stamps_dir.mkdir(parents=True, exist_ok=True)
        _prune_stale_stamps(stamps_dir)
        tmp = stamps_dir / f".{tree}.{os.getpid()}.tmp"
        tmp.write_text(json.dumps(stamp, indent=1))
        tmp.replace(stamps_dir / f"{tree}.json")  # atomic on the same fs
        run.add(Stage("stamp", status="ok", detail=f"push stamp minted for tree {tree[:12]}"))
    except (OSError, subprocess.SubprocessError) as e:
        run.add(Stage("stamp", status="warn", detail=f"stamp write failed: {e}"))


def gate(args, repo: Path) -> int:
    arches = _resolve_arches(getattr(args, "arches", None))
    retries = max(0, int(getattr(args, "retries", 2) or 0))
    do_build = not getattr(args, "no_build", False)
    # The net suite always runs OUTSIDE the amd64 smoke now: either as its own
    # parallel gate lane (default) or not at all (--no-net). The smoke itself
    # therefore always gets --no-net — its net stage is the serial-two-boots
    # shape this lane split removes.
    # Every arch's smoke leg boots TWO CPUs. Until 2026-10-04 the gate booted one, so
    # nothing on the landing path ever ran the MP kernel's cross-CPU paths -- and
    # amd64 shipped a lazy-TLB hole that corrupted memory on roughly half of -smp 2
    # boots (field manual: "Successful reads that leave their pages ZERO"), visible
    # only to whoever passed --smp by hand. The MP kernel has been the default media
    # selection since 2026-09-12; this makes the gate run it the way it ships.
    smoke_extra: list[str] = ["--no-net", "--smp", str(GATE_SMP)]

    run = Run("gate", "+".join(arches))
    store_requested = getattr(args, "artifact_store", None)
    verify_requested = getattr(args, "artifact_store_verify", False)
    if (verify_requested and not store_requested) or (store_requested and not do_build):
        run.classification = "ARTIFACT_STORE_FAIL"
        run.add(
            Stage(
                "artifact-store",
                status="fail",
                detail=(
                    "store verification requires --artifact-store; store gating requires a build"
                ),
            )
        )
        return run.finalize(repo)
    # Host free space BEFORE anything builds. A full disk does not present as a
    # disk problem — it surfaces as a RED with no failing test, an ENOSPC while
    # WRITING the verdict, and a harness "low on memory" message that points away
    # from the cause. One line here beats four lanes re-reading artifacts.
    if not hostdisk.preflight(run, repo, arches):
        return 1
    # Sample the checkout BEFORE anything builds: the stamp must certify this tree, not
    # whatever HEAD happens to be when the gate finishes (see _write_gate_stamp).
    gated_state = _tree_state(repo)
    gate_root = run.dir

    # --- phases 1+1.5+2, PIPELINED (F5, 2026-07-06) ---
    # Each arch's lint + smoke (+ the amd64 net lane) launch THE MOMENT its own build lands,
    # instead of waiting for an all-arches build barrier. Safe by hostlock design:
    # windows-host QEMU boots run on the WINDOWS scheduling domain and throttle on the
    # separate `bootcpu` pool, so a boot coexists with a WSL-side build — build ∥ boot is
    # the documented contract. amd64 goes first: it feeds two lanes (own smoke + net, the
    # longest). Modeled wall-clock (warm): ~390s → ~230s.
    #
    # The BUILDS are submitted concurrently too (2026-07-25) and self-throttle on the
    # broker's build share — see _build_once. They used to be a serial loop here, which was
    # stricter than the broker (2 concurrent builds are sanctioned fleet-wide) and made the
    # gate's build phase sum-of-arches for no reason.
    #
    # Strict arch guards follow their own build; only deferred source checks need
    # the complete build matrix. Both source partitions gate the push stamp.
    net_requested = not getattr(args, "no_net", False)
    # The kmtest conformance ratchet, one lane per arch. On by default: a ratchet that
    # only reports is a ratchet nothing has to obey, and the whole point of seeding the
    # identity floors was to make a kernel-API regression stop a landing the way a dead
    # desktop does. `--no-conformance` opts out (and is what a docs-only or
    # tooling-only change should use if the media cannot be staged).
    conf_requested = not getattr(args, "no_conformance", False)
    if conf_requested:
        # Provision BEFORE any build is submitted. The build stamps the payload into the
        # build dir, so discovering the exe is missing at media-roll time is discovering
        # it far too late — it fails the livecd step and reads as a BUILD_FAIL against
        # whichever lanes share the batch. See _ensure_kmtest_payload for the incident.
        conf_ok, conf_why = _ensure_kmtest_payload(repo, arches)
        if not conf_ok:
            # Degrade the ratchet, never the build: without the payload the honest state
            # is "conformance did not run", and stamping VIBEOS_PAYLOAD anyway would
            # break every arch's media. Loud, not silent — the stage is recorded so a
            # green gate cannot be read as "the ratchet held".
            conf_requested = False
            # _build_once reads this same flag; without clearing it the builds would
            # still stamp VIBEOS_PAYLOAD=kmtest and fail the media step — the exact
            # breakage this block exists to prevent.
            args.no_conformance = True
            run.add(
                Stage(
                    "conformance",
                    status="warn",
                    detail="DISABLED — ratchet did not run",
                    salient=[
                        f"{conf_why}",
                        "fix: bash tools/payloads/kmtest/build.sh <arch>; "
                        "this gate did NOT verify kmtest conformance",
                    ],
                )
            )
    # tests/qemu/test_stortest.py, one lane per arch, on this gate's media. On by default:
    # it is the only thing in the gate that writes through a USB disk. `--no-stortest`
    # opts out.
    stor_requested = not getattr(args, "no_stortest", False)
    do_lint = not getattr(args, "no_lint", False)
    build_order = sorted(arches, key=lambda a: a != "amd64")  # amd64 first, rest in order
    results: dict[str, dict] = {}
    lint_ok: dict[str, bool] = {}
    cohab_ok: dict[str, bool] = {}
    fut: dict = {}
    lfut: dict = {}
    cfut: dict = {}
    tooling_ok = True
    tfut: dict = {}
    difftest_ok = True
    dfut: dict = {}

    # --ladder for the arches whose media actually carries the WoW64 guests.
    #
    # WHY THE GATE NEEDS THIS AT ALL. The nine probe guests are spawned by
    # winlogon AFTER explorer, and the smoke's success condition IS the desktop
    # marker -- so an ordinary gate stops watching before the ladder starts, and a
    # guest can be RED indefinitely with every gate green. That is not
    # hypothetical: selfmod32 exited 0xBAD9 on amd64 from the day E17 landed it,
    # and thread32 sat dead on rung 1 until a154628337b. ladder.py turned "remember
    # to run --until on both arches" into a flag; this turns the flag into policy.
    #
    # WHY IT IS CONDITIONAL. The guests stage from sibling trees (build-i386,
    # build-fex-hello) and arm64 stages them only under VIBEOS_FEX=1, so a lane
    # that never staged one would otherwise burn the full ladder drain per boot and
    # then report LADDER_NOT_STAGED as though something regressed. The probe reads
    # the MEDIA, not those trees -- see ladder.media_has_guests.
    #
    # WHY THE SKIP IS LOUD. A gate that quietly stops checking the thing it was
    # asked to check is the exact failure this lane exists to remove, so the
    # decision is recorded per arch either way. Nothing here is retryable on
    # purpose: with staging VERIFIED on the media, a LADDER_NOT_STAGED verdict
    # means the image and this probe disagree, which is a real finding, and a red
    # rung is a regression. Both fall through RETRYABLE and fail the arch at once.
    # Memoized per lane: decided once on the media this gate built, then reused by
    # every retry attempt. A retry must not re-decide -- rescanning the image would
    # both repeat the scan and let an arch silently change policy mid-gate.
    ladder_decided: dict[str, list[str]] = {}

    def _ladder_extra(lane: str) -> list[str]:
        if lane not in ladder_decided:
            extra, staged, why = ladder_mod.gate_decision(repo, lane)
            ladder_decided[lane] = extra
            run.add(Stage(f"ladder:{lane}", status="ok" if staged else "skip", detail=why))
        return ladder_decided[lane]

    def _lane_once(lane: str, root: Path) -> dict:
        if lane == "net":
            return _net_once(repo, root)
        if lane == BIOS_LANE:
            # Same media as the amd64 lane, booted by SeaBIOS. No ladder: the WoW64
            # guests are a property of the build, which the amd64 lane already grades.
            return _smoke_once(repo, lane, root, smoke_extra)
        if lane.startswith("conf:"):
            return _conformance_once(repo, lane.split(":", 1)[1], root)
        if lane.startswith("stor:"):
            return _stortest_once(repo, lane.split(":", 1)[1], root)
        return _smoke_once(repo, lane, root, smoke_extra + _ladder_extra(lane))

    # Workers: every lane's first pass can be in flight while retries queue, plus
    # the (cheap, thread-blocking) lint runs and the concurrent builds share the same
    # executor. All of these BLOCK on subprocesses or on the host broker, so the worker
    # count only has to be large enough not to re-serialize what the broker would allow.
    # Retries need one worker PER LANE, not per attempt: a lane's attempts run serially
    # inside a single task (see phase 3 — concurrent attempts recreate the very load
    # NET_FLAKE and HOST_STARVED are defined by). Keep the +n_lanes headroom anyway so
    # a retrying lane never has to wait on a first-pass worker to drain.
    n_lanes = (
        len(arches)
        + (1 if net_requested and "amd64" in arches else 0)
        + (1 if "amd64" in arches else 0)
        + (len(arches) if conf_requested else 0)
        + (len(arches) if stor_requested else 0)
    )
    max_workers = (
        min(
            n_lanes * 2
            + (len(arches) + 2 if do_lint else 0)
            + (len(arches) if do_build else 0)
            # check-tooling + difftest: two arch-independent workers
            + (2 if do_lint else 0),
            20,
        )
        or 1
    )

    # check-tooling READS this worktree's build dirs: rustconv's tests resolve macros
    # through build-<arch>/compile_commands.json and the generated headers. A build
    # that must (re)configure from scratch deletes its build dir first (build.py
    # _maybe_canonize), so tooling started beside it reads a tree with neither --
    # 2026-10-09: test_deterministic_only failed MacroAuthorityError on a green
    # master, the line auto-held red, and five lanes stalled. Only that case waits;
    # an already-configured tree keeps the overlap.
    tooling_waits = (
        do_lint
        and do_build
        and any(not canon.is_canonical(repo / f"build-{a}") for a in build_order)
    )
    with cf.ThreadPoolExecutor(max_workers=max_workers) as ex:
        # check-tooling overlaps the builds -- unless one of them is about to recreate
        # its build dir, in which case it is submitted once the builds settle (below).
        if do_lint:
            lfut[ex.submit(_lint_once, repo, "source-early", gate_root / "lint-source-early")] = (
                "source-early"
            )
            if tooling_waits:
                run.log(
                    "check-tooling deferred until the builds finish: a fresh configure "
                    "recreates build-<arch>/ (compile_commands.json, generated headers), "
                    "which rustconv's tests read",
                    header="tooling",
                )
            else:
                tfut[ex.submit(_tooling_once, repo, gate_root / "tooling")] = "tooling"

        difftest_ready: set[str] = set()
        difftest_dependencies = {"i386", "amd64"} & set(build_order)

        def _launch_lanes(arch: str) -> None:
            """Submit an arch's post-build lanes (smoke [+ net] + lint) immediately."""
            difftest_ready.add(arch)
            # These are HOST tests of genuine C, not artifact-independent tests:
            # msvcrt requires amd64 objects/headers; SEH layout requires i386
            # headers. Start when those producers finish, without waiting for ARM64.
            if do_lint and not dfut and difftest_dependencies <= difftest_ready:
                dfut[ex.submit(_difftest_once, repo, gate_root / "difftest")] = "difftest"
            fut[ex.submit(_lane_once, arch, gate_root / f"smoke-{arch}-a1")] = arch
            if arch == "amd64" and net_requested:
                fut[ex.submit(_lane_once, "net", gate_root / "smoke-net-a1")] = "net"
            if arch == "amd64":
                fut[ex.submit(_lane_once, BIOS_LANE, gate_root / f"smoke-{BIOS_LANE}-a1")] = (
                    BIOS_LANE
                )
            if conf_requested:
                fut[ex.submit(_lane_once, f"conf:{arch}", gate_root / f"conf-{arch}-a1")] = (
                    f"conf:{arch}"
                )
            if stor_requested:
                fut[ex.submit(_lane_once, f"stor:{arch}", gate_root / f"stor-{arch}-a1")] = (
                    f"stor:{arch}"
                )
            if do_lint:
                lfut[ex.submit(_lint_once, repo, arch, gate_root / f"lint-{arch}")] = arch
            # Post-build by necessity: this one reads the arch's OBJECTS and
            # requires artifacts produced by this gate, not an arbitrary check. It
            # needs BOTH opt-ins: it is static verification like the lint (--no-lint means
            # "skip that"), and it can only judge artifacts this run produced — the verifier
            # refuses an archive older than the tree, so under --no-build a stale carry-over
            # would read as a red instead of the "nothing was rebuilt" it actually is.
            if do_lint and do_build:
                cfut[ex.submit(_cohab_once, repo, arch, gate_root / f"cohab-{arch}")] = arch

        if do_build:
            # Submit every arch's build at once and let the host broker decide how many
            # run (see _build_once). amd64 is submitted FIRST so FIFO admission gives it
            # the earliest permit — it feeds two lanes (own smoke + net, the longest).
            # amd64 stages out of build-i386, so beside an i386 build it defers
            # that and restages once i386 is done (see _build_once). Its lanes wait
            # for the restage; the other arches' lanes do not wait for anything.
            defer = {"amd64", "i386"} <= set(build_order)
            bfut = {
                ex.submit(_build_once, repo, run, a, args, defer_wow64=defer and a == "amd64"): a
                for a in build_order
            }
            amd64_built: dict | None = None
            i386_result: dict | None = None
            restage_fut = None

            def _maybe_restage() -> None:
                nonlocal restage_fut
                if restage_fut is not None or amd64_built is None or i386_result is None:
                    return
                if i386_result["ok"]:
                    restage_fut = ex.submit(_build_once, repo, run, "amd64", args, restage=True)
                else:
                    # No i386 set to stage. amd64 is still built and still smokes;
                    # its ladder then declines loudly as "not staged".
                    _launch_lanes("amd64")

            for f in cf.as_completed(bfut):
                r = f.result()
                if defer and r["arch"] == "i386":
                    i386_result = r
                if not r["ok"]:
                    kind = r.get("classification") or ""
                    run.add(
                        Stage(
                            f"build:{r['arch']}",
                            exit=r["exit"],
                            seconds=r["seconds"],
                            status="fail",
                            detail=f"build failed ({kind})" if kind else "build failed",
                            salient=[
                                f"vibeos-cli build --arch {r['arch']} exited {r['exit']}"
                                + (f" — {kind}" if kind else "")
                            ],
                        )
                    )
                    # Carry the child's own verdict up. Only the FIRST failing arch sets it,
                    # so a later arch cannot overwrite a more specific earlier reason; and it
                    # is set even when other arches build fine, because the old code assigned
                    # a classification only when EVERY arch failed (`if not arches` below) and
                    # left it empty for a partial failure — which the queue reads as "no
                    # information" and halts the whole train on. See _child_classification.
                    if kind and not run.classification:
                        run.classification = kind
                    # a build failure is fatal for that arch; the others still build
                    # and smoke so the matrix stays complete.
                elif defer and r["arch"] == "amd64":
                    amd64_built = r  # recorded once the restage settles
                else:
                    run.add(
                        Stage(
                            f"build:{r['arch']}",
                            seconds=r["seconds"],
                            status="ok",
                            detail="built",
                        )
                    )
                    _launch_lanes(r["arch"])  # pipeline: smoke this arch while others build
                if defer:
                    _maybe_restage()
            if amd64_built is not None:
                rs = restage_fut.result() if restage_fut is not None else None
                if rs is None or rs["ok"]:
                    run.add(
                        Stage(
                            "build:amd64",
                            seconds=amd64_built["seconds"] + (rs["seconds"] if rs else 0),
                            status="ok",
                            detail=(
                                "built + WoW64 restaged"
                                if rs
                                else "built (WoW64 NOT staged: i386 failed)"
                            ),
                        )
                    )
                    if rs is not None:
                        _launch_lanes("amd64")
                else:
                    kind = rs.get("classification") or ""
                    run.add(
                        Stage(
                            "build:amd64",
                            exit=rs["exit"],
                            seconds=amd64_built["seconds"] + rs["seconds"],
                            status="fail",
                            detail=(
                                f"WoW64 restage failed ({kind})" if kind else "WoW64 restage failed"
                            ),
                            salient=[
                                f"vibeos-cli build --arch amd64 (restage) exited {rs['exit']}"
                            ],
                        )
                    )
                    if kind and not run.classification:
                        run.classification = kind
            if tooling_waits:
                tfut[ex.submit(_tooling_once, repo, gate_root / "tooling")] = "tooling"
            built = {
                s.name.split(":", 1)[1]
                for s in run.stages
                if s.name.startswith("build:") and s.status == "ok"
            }
            arches = [a for a in arches if a in built]
            if not arches:
                run.classification = run.classification or "BUILD_FAIL"
                return run.finalize(repo)
        else:
            for arch in build_order:
                _launch_lanes(arch)

        if do_lint:
            # Header-dependent and unaudited scanners wait for this worktree's
            # build matrix. Early + deferred retain the original full inventory.
            lfut[
                ex.submit(_lint_once, repo, "source-deferred", gate_root / "lint-source-deferred")
            ] = "source-deferred"

        net_lane = ("amd64" in arches) and net_requested
        lanes = (
            list(arches)
            + (["net"] if net_lane else [])
            + ([BIOS_LANE] if "amd64" in arches else [])
            + ([f"conf:{a}" for a in arches] if conf_requested else [])
            + ([f"stor:{a}" for a in arches] if stor_requested else [])
        )

        def _collect_lints() -> None:
            for f in cf.as_completed(lfut):
                r = f.result()
                lint_ok[r["arch"]] = r["ok"]
                run.add(
                    Stage(
                        f"lint:{r['arch']}",
                        seconds=r["seconds"],
                        status="ok" if r["ok"] else "fail",
                        detail="check --strict clean"
                        if r["ok"]
                        else (
                            "SCANNER_ERROR: scanner did not complete; coverage unavailable"
                            if r.get("classification") == "SCANNER_ERROR"
                            else f"NEW lint findings (exit {r['exit']})"
                        ),
                        salient=[]
                        if r["ok"]
                        else [r["salient"] or f"strict check partition {r['arch']} failed"],
                    )
                )
                if not r["ok"] and (
                    r.get("classification") == "SCANNER_ERROR" or not run.classification
                ):
                    run.classification = r.get("classification") or "LINT_FAIL"

        def _collect_cohab() -> None:
            for f in cf.as_completed(cfut):
                r = f.result()
                cohab_ok[r["arch"]] = r["ok"]
                run.add(
                    Stage(
                        f"cohab:{r['arch']}",
                        seconds=r["seconds"],
                        status="ok" if r["ok"] else "fail",
                        detail="flipped modules fully covered"
                        if r["ok"]
                        else f"cohab coverage gap (exit {r['exit']})",
                        salient=[]
                        if r["ok"]
                        else [r["salient"] or f"verify_cohab_coverage --arch {r['arch']} failed"],
                    )
                )
                if not r["ok"] and not run.classification:
                    run.classification = "COHAB_FAIL"

        def _collect_tooling() -> None:
            nonlocal tooling_ok
            for f in cf.as_completed(tfut):
                r = f.result()
                tooling_ok = r["ok"]
                run.add(
                    Stage(
                        "tooling",
                        seconds=r["seconds"],
                        status="ok" if r["ok"] else "fail",
                        detail="check-tooling clean"
                        if r["ok"]
                        else f"python tooling findings (exit {r['exit']})",
                        salient=[] if r["ok"] else [r["salient"] or "check-tooling failed"],
                    )
                )
                if not r["ok"] and not run.classification:
                    run.classification = r.get("classification") or "TOOLING_FAIL"

        def _collect_difftest() -> None:
            nonlocal difftest_ok
            for f in cf.as_completed(dfut):
                r = f.result()
                difftest_ok = r["ok"]
                run.add(
                    Stage(
                        "difftest",
                        seconds=r["seconds"],
                        status="ok" if r["ok"] else "fail",
                        detail="difftest clean"
                        if r["ok"]
                        else f"rust-vs-C divergence (exit {r['exit']})",
                        salient=[] if r["ok"] else [r["salient"] or "difftest failed"],
                    )
                )
                if not r["ok"] and not run.classification:
                    run.classification = "DIFFTEST_FAIL"

        _collect_tooling()  # concurrent with the in-flight smokes
        _collect_difftest()  # concurrent with the in-flight smokes
        _collect_lints()  # concurrent with the in-flight smokes
        _collect_cohab()  # concurrent with the in-flight smokes
        for f in cf.as_completed(fut):
            r = f.result()
            results[r["arch"]] = r

        # --- phase 3: flake-retries for retryable failures ---
        # Lanes retry in PARALLEL with each other, but each lane's own attempts run
        # SEQUENTIALLY and stop at the first green. Firing all `retries` at once —
        # which this did — makes the retry a worse experiment than the original:
        #   * NET_FLAKE is defined as "the net suite failed under concurrent-smoke
        #     load". Launching its attempts concurrently RECREATES the load, so the
        #     retry re-runs the failing condition instead of removing it. With only
        #     the net lane pending, `--retries 2` put two amd64-uefi-net VMs in
        #     contention with each other and nothing else — self-inflicted, and
        #     harsher than the first pass. Observed 2026-08-08: a gate and then a
        #     land both burned 3/3 attempts on test_tdi_selftest_roundtrip, while
        #     the same test on byte-identical media passed 4/4 standalone seconds
        #     later. The second of those rejected a lane whose entire diff was
        #     three one-line baseline edits and a doc.
        #   * HOST_STARVED is "the lane never booted because it spent its budget
        #     queued". Competing retries make starvation MORE likely, not less.
        # A repeated experiment that holds the suspected cause fixed falsifies
        # nothing — the same rule the field manual states for harnesses. Serial
        # attempts also cost less: a green first retry means the rest never run.
        pending = [a for a in lanes if not results[a]["ok"] and results[a]["retryable"]]
        if pending and retries > 0:
            run.log(
                f"flake-retry: {', '.join(pending)} "
                f"({', '.join(results[a]['classification'] for a in pending)}) "
                f"— up to {retries} serial attempts each, lanes in parallel",
                header="flake-retry",
            )

            def _retry_lane(a: str) -> tuple[dict, int]:
                r, n = None, 0
                for k in range(retries):
                    r = _lane_once(a, gate_root / f"smoke-{a.replace(':', '-')}-r{k + 1}")
                    n += 1
                    if r["ok"]:
                        break
                return r, n  # type: ignore[return-value]

            retry_fut = {ex.submit(_retry_lane, a): a for a in pending}
            # an arch passes on its first green attempt.
            for f in cf.as_completed(retry_fut):
                r, n = f.result()
                a = r["arch"]
                prev = results[a]
                r["attempts"] = prev.get("attempts", 1) + n
                # keep the best: a green retry supersedes the failed first pass.
                if r["ok"] and not prev["ok"]:
                    r["recovered"] = True
                    r["first_classification"] = prev.get("classification", "")
                    results[a] = r
                else:
                    prev["attempts"] = r["attempts"]

    # --- verdict matrix ---
    _conf_lanes = [f"conf:{a}" for a in arches] if conf_requested else []
    _stor_lanes = [f"stor:{a}" for a in arches] if stor_requested else []
    # Every lane _launch_lanes submits must be listed here AND in requested_lanes below,
    # or it runs, prints, and is ignored: the BIOS lane did exactly that on its first day.
    _bios_lanes = [BIOS_LANE] if "amd64" in arches else []
    for arch in (
        _resolve_arches(getattr(args, "arches", None))
        + (["net"] if net_lane else [])
        + _bios_lanes
        + _conf_lanes
        + _stor_lanes
    ):
        r = results.get(arch)
        if r is None:
            continue  # build-failed arch already has its Stage
        status = "ok" if r["ok"] else "fail"
        bits = [r["verdict"] if r["ok"] else r.get("classification", "FAIL")]
        att = r.get("attempts", 1)
        if att > 1:
            bits.append(f"{att} attempts")
        if r.get("recovered"):
            bits.append("recovered-after-flake")
        if r.get("flake_rate"):
            bits.append(f"flake {r['flake_rate']}")
        detail = ", ".join(bits)
        salient = []
        if not r["ok"] and r.get("salient"):
            salient.append(r["salient"])
        if r["ok"] and r.get("recovered"):
            first = r.get("first_classification") or "a flake"
            salient.append(f"first pass was {first}; passed on retry")
        if arch.startswith("conf:"):
            stage_name = f"conformance:{arch.split(':', 1)[1]}"
        elif arch.startswith("stor:"):
            stage_name = f"stortest:{arch.split(':', 1)[1]}"
        else:
            stage_name = f"smoke:{arch}"
        run.add(
            Stage(
                stage_name,
                exit=r.get("exit", 0),
                seconds=r["seconds"],
                status=status,
                detail=detail,
                salient=salient,
            )
        )
        if not r["ok"] and not run.classification:
            run.classification = r.get("classification", "FAIL")

    # --- push stamp (T1.3): green FULL gate on a clean tree mints the credential ---
    requested = _resolve_arches(getattr(args, "arches", None))
    requested_lanes = (
        requested
        + (["net"] if net_lane else [])
        + ([BIOS_LANE] if "amd64" in requested else [])
        # The whole point of wiring the ratchet in: a conformance regression must
        # withhold the push credential, exactly as a dead desktop does. Without this
        # line the lane would run, print red, and still mint the stamp.
        + ([f"conf:{a}" for a in requested] if conf_requested else [])
        # Same for stortest: a storage-stack regression must withhold the stamp.
        + ([f"stor:{a}" for a in requested] if stor_requested else [])
    )
    all_ok = (
        bool(results)
        and all(results.get(a, {}).get("ok") for a in requested_lanes)
        and len(results) == len(requested_lanes)
        # Fail closed: BOTH the shared suite and every requested arch's guards
        # must return success. --no-lint remains explicitly unstamped below.
        and (
            not do_lint
            or all(lint_ok.get(a, False) for a in ["source-early", "source-deferred", *requested])
        )
        # Same per-arch contract as lint_ok, and defaulting True for the same reasons
        # (--no-lint/--no-build, or an arch that never built). A cohab gap means a module's
        # Rust is not actually the thing that links — the build is green and the port is
        # a no-op — so it must not mint a push credential.
        and all(cohab_ok.get(a, True) for a in requested)
        # check-tooling is arch-independent and runs once; it defaults True under
        # --no-lint, matching how lint_ok defaults above.
        and tooling_ok
        # Same contract for the host difftests: arch-independent, once per gate,
        # True under --no-lint. A rust-vs-C divergence must not mint a stamp.
        and difftest_ok
    )
    # Defense-in-depth: never mint the credential when ANY stage failed. all_ok
    # is computed over smoke + lint_ok only; this also catches any future stage
    # that fails the run (sets a fail Stage → verdict FAIL) without being wired
    # into all_ok — the stamp is the push credential, so err on the safe side.
    if all_ok and run.verdict != "FAIL":
        _write_gate_stamp(run, repo, requested, gated_state)

    run.extra["arches"] = arches
    run.extra["matrix"] = {
        a: {
            "verdict": r["verdict"],
            "classification": r["classification"],
            "attempts": r.get("attempts", 1),
            "recovered": bool(r.get("recovered")),
            "hang_kind": r.get("hang_kind"),
            "wedge_confirmed": bool(r.get("wedge_confirmed")),
            "census_stuck": r.get("census_stuck", 0),
            "run_dir": r["run_dir"],
        }
        for a, r in results.items()
    }
    return run.finalize(repo)
