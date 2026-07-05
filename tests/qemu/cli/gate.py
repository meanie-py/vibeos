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

  1. builds — serial (still gated by the exclusive host CPU lease; parallel builds
     are a separate follow-up, see docs/engineering). Opt out with --no-build.
  1.5. lint gate (BLOCKING) — `check --strict --arch <a>` per built arch, in
     parallel (the full scanner suite: source lints baseline-diff NEW=0,
     cargo-link/committed-drift bit-rot guards, per-arch binary lints). Reuses
     the `check` command verbatim so there is ONE strict-check code path, not a
     reimplementation that could drift. A NEW finding (classification LINT_FAIL)
     blocks the push stamp; smokes still run so the matrix stays informative.
     Opt out with --no-lint. This closes the "detection without enforcement"
     gap where the arsenal ran warn-only in `build` and nothing forced
     `check --strict` before a land.
  2. smokes — ALL arches launched concurrently. Wall-clock goes sum -> max.
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
host CPU-lease pool (hostlock, DEFAULT_CPU_LEASES=4) and the global QEMU cap
(vm.py QEMU_MAX_CONCURRENT=10). The executor just needs enough workers to not
re-serialize; the harness does the real throttling.
"""

from __future__ import annotations

import concurrent.futures as cf
import json
import os
import subprocess
import time
from pathlib import Path

from . import archcfg
from .artifacts import Run, Stage

# A failure with one of these classifications is a *documented* flake or an
# environment failure — safe to retry. Anything else (a BUGCHECK, a Rust panic,
# a hit fail-marker, stale media, an unknown red) is a real regression and fails
# the arch immediately: retrying a regression away is what the North Star forbids.
#   - LAUNCH_FAIL: a QEMU/interop launch hiccup (env, not the OS).
#   - NET_FLAKE: boots green + net suite failed under concurrent-smoke load (the
#     documented amd64 timing flake, see smoke.py _net_stage) — a real netstack
#     regression fails the retry too, so it still ends red.
#
# HANG_TIMEOUT is deliberately NOT a blanket member: "HANG_TIMEOUT ≈ host
# contention" dates from the native-KVM era, but on the default windows-host TCG
# backend the dominant real bugs (service wedge, lost wakeups, silent kernel
# panics) present exactly as HANG_TIMEOUT — a ~50% wedge passed a 3-attempt gate
# ~87% of the time. It is retryable ONLY when smoke characterized it as
# slow/contended (serial still flowing at the deadline) AND the kernel didn't
# self-report stuck threads — see _retryable().
RETRYABLE = {"KNOWN_FLAKE", "FLAKY", "LAUNCH_FAIL", "NET_FLAKE"}


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


def _write_gate_stamp(run: Run, repo: Path, arches: list[str]) -> None:
    """T1.3 (mechanized North Star): a green FULL gate mints the push
    credential — the pre-push hook (tools/git-hooks/pre-push) refuses updates
    to master whose tree doesn't match a fresh green stamp. Only a CLEAN tree
    is stamped: a dirty tree gates content no commit will ever hash to.
    Partial-arch gates never stamp (the North Star is all three)."""
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
        (common / "vibeos-gate-stamp.json").write_text(json.dumps(stamp, indent=1))
        run.add(Stage("stamp", status="ok", detail=f"push stamp minted for tree {tree[:12]}"))
    except (OSError, subprocess.SubprocessError) as e:
        run.add(Stage("stamp", status="warn", detail=f"stamp write failed: {e}"))


def gate(args, repo: Path) -> int:
    arches = _resolve_arches(getattr(args, "arches", None))
    retries = max(0, int(getattr(args, "retries", 2) or 0))
    do_build = not getattr(args, "no_build", False)
    smoke_extra: list[str] = []
    if getattr(args, "no_net", False):
        smoke_extra.append("--no-net")

    run = Run("gate", "+".join(arches))
    gate_root = run.dir

    # --- phase 1: builds (serial — exclusive host CPU lease) ---
    if do_build:
        for arch in arches:
            bt0 = time.monotonic()
            cmd = [str(repo / "vibeos-cli"), "build", "--arch", arch]
            if getattr(args, "clean", False):
                cmd.append("--clean")
            cp = run.run(cmd, cwd=repo, stage=f"build:{arch}")
            bdt = time.monotonic() - bt0
            if cp.returncode != 0:
                run.add(
                    Stage(
                        f"build:{arch}",
                        exit=cp.returncode,
                        seconds=bdt,
                        status="fail",
                        detail="build failed",
                        salient=[f"vibeos-cli build --arch {arch} exited {cp.returncode}"],
                    )
                )
                # a build failure is fatal for that arch; keep building the rest so
                # the matrix is complete, but this arch won't smoke (filtered below).
            else:
                run.add(Stage(f"build:{arch}", seconds=bdt, status="ok", detail="built"))
        built = {
            s.name.split(":", 1)[1]
            for s in run.stages
            if s.name.startswith("build:") and s.status == "ok"
        }
        arches = [a for a in arches if a in built]
        if not arches:
            run.classification = "BUILD_FAIL"
            return run.finalize(repo)

    # --- phase 1.5: strict lint gate per built arch (T3 — blocks the stamp) ---
    # Runs the scanner suite blocking (baseline-diff NEW=0) so the arsenal
    # enforces on the land path instead of warn-only. Independent per arch →
    # parallel; a NEW finding fails the gate but smokes still run (informative).
    lint_ok: dict[str, bool] = {}
    if not getattr(args, "no_lint", False):
        with cf.ThreadPoolExecutor(max_workers=min(len(arches), 4) or 1) as lex:
            lfut = {lex.submit(_lint_once, repo, a, gate_root / f"lint-{a}"): a for a in arches}
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

    # --- phase 2: first-pass smokes, all arches in parallel ---
    max_workers = min(len(arches) * (1 + retries), 12) or 1
    results: dict[str, dict] = {}
    with cf.ThreadPoolExecutor(max_workers=max_workers) as ex:
        fut = {
            ex.submit(_smoke_once, repo, a, gate_root / f"smoke-{a}-a1", smoke_extra): a
            for a in arches
        }
        for f in cf.as_completed(fut):
            r = f.result()
            results[r["arch"]] = r

        # --- phase 3: parallel flake-retries for retryable failures ---
        pending = [a for a in arches if not results[a]["ok"] and results[a]["retryable"]]
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
                    retry_fut[ex.submit(_smoke_once, repo, a, root, smoke_extra)] = a
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
    for arch in _resolve_arches(getattr(args, "arches", None)):
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
    all_ok = (
        bool(results)
        and all(results.get(a, {}).get("ok") for a in requested)
        and len(results) == len(requested)
        # T3: strict lints must be clean for every requested arch. lint_ok has no
        # entry when --no-lint or the arch didn't build (the latter already fails
        # the results check above), so default True there and let those paths gate
        # on their own condition.
        and all(lint_ok.get(a, True) for a in requested)
    )
    # Defense-in-depth: never mint the credential when ANY stage failed. all_ok
    # is computed over smoke + lint_ok only; this also catches any future stage
    # that fails the run (sets a fail Stage → verdict FAIL) without being wired
    # into all_ok — the stamp is the push credential, so err on the safe side.
    if all_ok and run.verdict != "FAIL":
        _write_gate_stamp(run, repo, requested)

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
