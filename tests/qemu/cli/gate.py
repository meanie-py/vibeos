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
     submitted at once (they self-throttle on the broker's build share, default 2
     concurrent fleet-wide; amd64 is submitted first, since it feeds two lanes),
     and each arch's smoke, its strict lint, and (for amd64) the net lane launch
     THE MOMENT its own build lands rather than behind an all-arches barrier.
     Safe by hostlock design:
     windows-host boots run on the Windows scheduling domain (`bootcpu` pool)
     and coexist with the next WSL-side build (build ∥ boot is the documented
     contract). Opt out of builds with --no-build (all lanes launch at once).
     The lint gate (BLOCKING) is `check --strict --arch <a>` per built arch —
     the full scanner suite: source lints baseline-diff NEW=0, cargo-link/
     committed-drift bit-rot guards, per-arch binary lints. Reuses the `check`
     command verbatim so there is ONE strict-check code path. A NEW finding
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
import subprocess
import time
from pathlib import Path

from . import archcfg, hostlock
from .artifacts import Run, Stage

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
RETRYABLE = {"KNOWN_FLAKE", "FLAKY", "LAUNCH_FAIL", "NET_FLAKE", "HOST_STARVED"}


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


def _smoke_once(repo: Path, arch: str, root: Path, extra: list[str]) -> dict:
    """Run one `vibeos-cli smoke --arch <arch>` in its own artifact root so its
    summary.json is locatable deterministically. Returns a compact result dict."""
    root.mkdir(parents=True, exist_ok=True)
    cmd = [str(repo / "vibeos-cli"), "smoke", "--arch", arch, *extra]
    env = dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root))
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=str(repo), env=env, capture_output=True, text=True)
    dt = time.monotonic() - t0
    summary = _read_summary(root)
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
        "seconds": dt,
        "run_dir": summary.get("_dir", str(root)),
        "flake_rate": summary.get("flake_rate"),
        "screenshot": summary.get("screenshot"),
        "salient": _first_salient(summary),
    }


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
    data["_dir"] = str(hits[-1].parent)
    return data


def _first_salient(summary: dict) -> str:
    for st in summary.get("stages", []):
        if st.get("status") in ("fail", "warn"):
            for s in st.get("salient", []):
                return str(s)
    return ""


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
        proc = subprocess.run(cmd, cwd=str(repo), capture_output=True, text=True, timeout=1200)
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


def _build_once(repo: Path, run: Run, arch: str, args) -> dict:
    """Build one arch as its own gate lane.

    Submitted CONCURRENTLY for every arch (2026-07-25) instead of the old serial loop. The
    serial loop predated `hostlock` commit 226733668e6: build-vs-build is no longer
    exclusive — a build takes a ceil(K/max_builds) SHARE of the cpuheavy pool, so
    VIBEOS_MAX_CONCURRENT_BUILDS (default 2) fleet-wide is already the sanctioned limit, and
    the per-build-dir lock is what actually protects a build dir. Submitting all arches
    therefore doesn't mean three cold builds fighting over the box: the broker admits two
    (FIFO, arrival-ordered) and queues the third. It just stops the gate from being stricter
    than the broker it delegates to — which matters now that `land` serializes gates, making
    gate wall-clock the landing throughput limit.

    `--no-check` is passed when the strict lint lanes run: that suite is the same scanners,
    blocking, so the build's warn-only pass is pure duplication (~20-40s/arch).
    """
    cmd = [str(repo / "vibeos-cli"), "build", "--arch", arch]
    if not getattr(args, "no_lint", False):
        cmd.append("--no-check")
    if getattr(args, "clean", False):
        cmd.append("--clean")
    t0 = time.monotonic()
    cp = run.run(cmd, cwd=repo, stage=f"build:{arch}")
    return {
        "arch": arch,
        "ok": cp.returncode == 0,
        "exit": cp.returncode,
        "seconds": time.monotonic() - t0,
    }


def _lint_once(repo: Path, arch: str, root: Path) -> dict:
    """Run `vibeos-cli check --strict --arch <arch>` for one built arch. This is
    the T3 land-gate: the scanner suite (verify_*, lint_kernel, scan_ud2,
    committed_drift, cargo_link, …) enforces baseline-diff NEW=0 instead of the
    old warn-only path. Returns a compact result; exit != 0 blocks the stamp."""
    root.mkdir(parents=True, exist_ok=True)
    cmd = [str(repo / "vibeos-cli"), "check", "--strict", "--arch", arch]
    env = dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root))
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=str(repo), env=env, capture_output=True, text=True)
    dt = time.monotonic() - t0
    salient = ""
    for ln in (proc.stdout or "").splitlines():
        if "✗" in ln or "FAIL" in ln:
            salient = ln.strip()
            break
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
    salient = ""
    for ln in (proc.stdout or "").splitlines():
        if "\u2717" in ln or "FAIL" in ln:
            salient = ln.strip()
            break
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
    smoke_extra: list[str] = ["--no-net"]

    run = Run("gate", "+".join(arches))
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
    # Lint (`check --strict`, T3) is source-only per arch → independent of the
    # build artifacts; it launches alongside the arch's smoke and gates the push
    # stamp, not the boots.
    net_requested = not getattr(args, "no_net", False)
    do_lint = not getattr(args, "no_lint", False)
    build_order = sorted(arches, key=lambda a: a != "amd64")  # amd64 first, rest in order
    results: dict[str, dict] = {}
    lint_ok: dict[str, bool] = {}
    fut: dict = {}
    lfut: dict = {}
    tooling_ok = True
    tfut: dict = {}

    def _lane_once(lane: str, root: Path) -> dict:
        if lane == "net":
            return _net_once(repo, root)
        return _smoke_once(repo, lane, root, smoke_extra)

    # Workers: every lane's first pass can be in flight while retries queue, plus
    # the (cheap, thread-blocking) lint runs and the concurrent builds share the same
    # executor. All of these BLOCK on subprocesses or on the host broker, so the worker
    # count only has to be large enough not to re-serialize what the broker would allow.
    n_lanes = len(arches) + (1 if net_requested and "amd64" in arches else 0)
    max_workers = (
        min(
            n_lanes * (1 + retries)
            + (len(arches) if do_lint else 0)
            + (len(arches) if do_build else 0)
            + (1 if do_lint else 0),  # check-tooling: one arch-independent worker
            20,
        )
        or 1
    )

    with cf.ThreadPoolExecutor(max_workers=max_workers) as ex:
        # check-tooling needs no build artifacts, so start it immediately — it
        # overlaps the first build instead of adding wall-clock.
        if do_lint:
            tfut[ex.submit(_tooling_once, repo, gate_root / "tooling")] = "tooling"

        def _launch_lanes(arch: str) -> None:
            """Submit an arch's post-build lanes (smoke [+ net] + lint) immediately."""
            fut[ex.submit(_lane_once, arch, gate_root / f"smoke-{arch}-a1")] = arch
            if arch == "amd64" and net_requested:
                fut[ex.submit(_lane_once, "net", gate_root / "smoke-net-a1")] = "net"
            if do_lint:
                lfut[ex.submit(_lint_once, repo, arch, gate_root / f"lint-{arch}")] = arch

        if do_build:
            # Submit every arch's build at once and let the host broker decide how many
            # run (see _build_once). amd64 is submitted FIRST so FIFO admission gives it
            # the earliest permit — it feeds two lanes (own smoke + net, the longest).
            bfut = {ex.submit(_build_once, repo, run, a, args): a for a in build_order}
            for f in cf.as_completed(bfut):
                r = f.result()
                if not r["ok"]:
                    run.add(
                        Stage(
                            f"build:{r['arch']}",
                            exit=r["exit"],
                            seconds=r["seconds"],
                            status="fail",
                            detail="build failed",
                            salient=[f"vibeos-cli build --arch {r['arch']} exited {r['exit']}"],
                        )
                    )
                    # a build failure is fatal for that arch; the others still build
                    # and smoke so the matrix stays complete.
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
            built = {
                s.name.split(":", 1)[1]
                for s in run.stages
                if s.name.startswith("build:") and s.status == "ok"
            }
            arches = [a for a in arches if a in built]
            if not arches:
                run.classification = "BUILD_FAIL"
                return run.finalize(repo)
        else:
            for arch in build_order:
                _launch_lanes(arch)

        net_lane = ("amd64" in arches) and net_requested
        lanes = list(arches) + (["net"] if net_lane else [])

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
                        else f"NEW lint findings (exit {r['exit']})",
                        salient=[]
                        if r["ok"]
                        else [r["salient"] or f"check --strict --arch {r['arch']} failed"],
                    )
                )
                if not r["ok"] and not run.classification:
                    run.classification = "LINT_FAIL"

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
                    run.classification = "TOOLING_FAIL"

        _collect_tooling()  # concurrent with the in-flight smokes
        _collect_lints()  # concurrent with the in-flight smokes
        for f in cf.as_completed(fut):
            r = f.result()
            results[r["arch"]] = r

        # --- phase 3: parallel flake-retries for retryable failures ---
        pending = [a for a in lanes if not results[a]["ok"] and results[a]["retryable"]]
        if pending and retries > 0:
            run.log(
                f"flake-retry: {', '.join(pending)} "
                f"({', '.join(results[a]['classification'] for a in pending)}) "
                f"— up to {retries} parallel attempts each",
                header="flake-retry",
            )
            retry_fut = {}
            for a in pending:
                for k in range(retries):
                    root = gate_root / f"smoke-{a}-r{k + 1}"
                    retry_fut[ex.submit(_lane_once, a, root)] = a
            # collect all retries; an arch passes on its first green attempt.
            for f in cf.as_completed(retry_fut):
                r = f.result()
                a = r["arch"]
                prev = results[a]
                r["attempts"] = prev.get("attempts", 1) + 1
                # keep the best: a green retry supersedes the failed first pass.
                if r["ok"] and not prev["ok"]:
                    r["recovered"] = True
                    r["first_classification"] = prev.get("classification", "")
                    results[a] = r
                else:
                    prev["attempts"] = r["attempts"]

    # --- verdict matrix ---
    for arch in _resolve_arches(getattr(args, "arches", None)) + (["net"] if net_lane else []):
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
        run.add(
            Stage(
                f"smoke:{arch}",
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
    requested_lanes = requested + (["net"] if net_lane else [])
    all_ok = (
        bool(results)
        and all(results.get(a, {}).get("ok") for a in requested_lanes)
        and len(results) == len(requested_lanes)
        # T3: strict lints must be clean for every requested arch. lint_ok has no
        # entry when --no-lint or the arch didn't build (the latter already fails
        # the results check above), so default True there and let those paths gate
        # on their own condition.
        and all(lint_ok.get(a, True) for a in requested)
        # check-tooling is arch-independent and runs once; it defaults True under
        # --no-lint, matching how lint_ok defaults above.
        and tooling_ok
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
