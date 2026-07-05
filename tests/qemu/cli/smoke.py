"""`vibeos-cli smoke` — the verdict front door (plan §9, §20.1).

Staleness preflight (hard-fail unless --allow-stale), then boot via the existing
QemuVM (which already owns slots/reaper/cap/live-log). Supports --repeat N
(independent cold boots for flake rate), --until <marker>, --fail-on <regex>
(abort at first hard-failure marker), and auto-triage on failure.
"""

from __future__ import annotations

import re
import subprocess
import sys
import time
from pathlib import Path

from ..vm import QemuVM, get_arch_config
from . import archcfg, provenance, triage
from . import config as cli_config
from .artifacts import Run, Stage


def _watch(vm: QemuVM, until: str, fail_on: str | None, timeout: float):
    """Poll serial for the success marker or a fail-marker regex. Returns
    (outcome, text, diag) where outcome in {reached, failmarker, timeout, dead}.

    `diag` carries timeout-characterization (silence duration + last serial line)
    so a HANG_TIMEOUT can be told apart from a slow/contended boot WITHOUT manually
    grepping full.log: if serial was still flowing at the deadline the box was
    making progress (host contention / slow TCG), not wedged."""
    fail_re = re.compile(fail_on) if fail_on else None
    hang_silence = getattr(vm.config, "hang_silence_seconds", 0.0)
    buf = ""
    start = time.monotonic()
    deadline = start + timeout
    last_activity = start
    early_hang = False
    now = start
    while now < deadline:
        chunk = vm.serial.read_available(timeout=0.5)
        now = time.monotonic()
        if chunk:
            buf += chunk
            last_activity = now
            if until in buf:
                return "reached", buf, {}
            if fail_re and fail_re.search(buf):
                return "failmarker", buf, {}
        elif hang_silence and buf and (now - last_activity) > hang_silence:
            # Boot emitted serial, then went silent well past any legitimate
            # inter-line gap on a healthy boot: a wedge. Fail fast instead of
            # burning the rest of boot_timeout (arm64 wedges otherwise sit ~5min).
            early_hang = True
            break
    silent_s = time.monotonic() - last_activity
    last_line = next((ln for ln in reversed(buf.splitlines()) if ln.strip()), "")
    diag = {"silent_s": silent_s, "last_line": last_line[:160], "early_hang": early_hang}
    return "timeout", buf, diag


def _settle_watch(
    vm: QemuVM, seconds: float, milestone: str | None, grace: float = 60.0
) -> tuple[str, bool]:
    """After the boot marker, let user-mode finish drawing the desktop before
    the screenshot. The marker (`_tWinMain entered`) fires the instant explorer
    starts — the taskbar/Start button/tray are built ~15s later. Without this,
    the canonical screenshot is a half-built (taskbar-less) desktop, which both
    hides real shell regressions and looks alarmingly blank.

    The drained serial is KEPT and returned (not discarded): a BugCheck or panic
    during this window used to be a structural false green (the marker had
    already matched). If `milestone` is given (`shell ready` — the documented
    North-Star marker, which fires AFTER `_tWinMain`), waits up to `grace` extra
    seconds for it past the settle window; returns (settle_text, milestone_seen)."""
    buf = ""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        buf += vm.serial.read_available(timeout=0.5) or ""
    if milestone and milestone not in buf:
        extra = time.monotonic() + grace
        while time.monotonic() < extra and milestone not in buf:
            buf += vm.serial.read_available(timeout=0.5) or ""
    return buf, (milestone in buf) if milestone else True


def _screenshot(vm: QemuVM, work: Path, repo: Path, arch: str) -> str | None:
    """Capture a PNG screenshot, pass OR fail, while the VM is still alive.

    Screendumps a PPM then converts via PIL — NOT ImageMagick `convert`, which
    writes a 1-bit colormap PNG for mostly-one-color frames (a bare desktop or a
    solid BSOD), rendering as black and hiding the very state we want. Mirrors
    conftest.py / test_smoke.py::test_screenshot. Also refreshes the canonical
    repo screenshot (qemu/<vm_arch>_smoke.png) so "always check the screenshot"
    points at the latest boot. Returns the PNG path (or the PPM if PIL is
    unavailable, or None if even the screendump failed)."""
    ppm = work / "screen.ppm"
    try:
        vm.screendump(ppm)
    except Exception:
        return None
    png = work / "screen.png"
    try:
        from PIL import Image

        img = Image.open(ppm)
        img.save(png, "PNG")
    except Exception:
        return str(ppm) if ppm.exists() else None
    try:
        canon = repo / "qemu" / f"{archcfg.vm_arch(arch)}_smoke.png"
        img.save(canon, "PNG")
    except Exception:
        pass
    return str(png)


def _one_boot(
    run: Run, repo: Path, arch: str, *, until: str, fail_on: str | None, idx: int, total: int
) -> dict:
    """A single independent boot. Returns {ok, outcome, classification, ...}.
    `arch` is the base arch (i386/amd64/arm64); the boot config is the vm_arch
    (i386 → i386-uefi) since x86 boots UEFI now."""
    config = get_arch_config(archcfg.vm_arch(arch))
    marker = until or config.boot_marker
    work = run.dir / f"boot{idx}"
    work.mkdir(exist_ok=True)
    # Launch backend from layered config (DEFAULTS < .vibeos-cli.local.json < env). Default
    # "native"; a WSL user can set qemu.backend="windows-host" to escape nested-virt contention.
    cli_cfg = cli_config.load(repo)
    backend = cli_cfg.get("qemu.backend", "native")
    vm = QemuVM(config, work, backend=backend, windows=cli_cfg.get("qemu.windows", {}))
    label = f"boot {idx + 1}/{total}" if total > 1 else "boot"
    if backend != "native":
        run.log(f"backend={backend} (qemu.windows={cli_cfg.get('qemu.windows', {})})", header=label)
    try:
        vm.start()
    except Exception as e:  # surface any launch failure
        run.log(f"{label}: start failed: {e}", header=label)
        return {
            "ok": False,
            "outcome": "launch_fail",
            "classification": "LAUNCH_FAIL",
            "salient": [str(e).splitlines()[0][:140]],
        }
    try:
        outcome, text, diag = _watch(vm, marker, fail_on, config.boot_timeout)
        run.log(text[-8000:], header=f"{label} serial tail")
        # On success, let the desktop settle (taskbar/tray render ~15s after the
        # marker) so the canonical screenshot shows the real shell, KEEPING the
        # settle-window serial for the post-marker health check. On failure,
        # screenshot immediately to capture the broken state.
        shell_seen = True
        if outcome == "reached" and config.desktop_settle_seconds > 0:
            run.log(f"settling {config.desktop_settle_seconds:.0f}s for desktop", header=label)
            # `shell ready` is the documented North-Star marker and fires after
            # `_tWinMain` — require it unless the caller asked for a custom
            # --until (early-marker debug boots) or the config doesn't emit it.
            need_shell = (
                not until
                and any("shell ready" in m for m in config.required_milestones)
                and "shell ready" not in text
            )
            settle_txt, shell_seen = _settle_watch(
                vm, config.desktop_settle_seconds, "shell ready" if need_shell else None
            )
            if settle_txt:
                text += settle_txt
                run.log(settle_txt[-4000:], header=f"{label} settle serial")
        # Always capture a PNG screenshot (pass or fail), while the VM is alive.
        shot = _screenshot(vm, work, repo, arch)
        dbg_raw = ""
        try:
            dbg_raw = vm.read_debugcon().decode("ascii", errors="replace")
        except Exception:
            pass
        if outcome == "reached":
            # Post-marker health: a BugCheck/panic during settle, a kernel panic
            # on debugcon, or a missing `shell ready` is a failure — the early
            # marker alone used to green all of these.
            post = triage.post_boot_check(text, dbg_raw)
            if post is None and not shell_seen:
                last = [ln for ln in text.splitlines() if ln.strip()][-2:]
                post = {
                    "classification": "SHELL_NOT_READY",
                    "salient": [
                        "boot marker reached but `shell ready` never followed "
                        f"(settle+{60:.0f}s grace)",
                        *[f"serial tail: {ln[:120]}" for ln in last],
                    ],
                }
            if post is not None:
                if shot:
                    post["screenshot"] = shot
                return {"ok": False, "outcome": "post-marker", **post}
            res = {"ok": True, "outcome": "reached", "classification": ""}
            ev = triage.wedge_evidence(text)
            if ev["census_stuck"] or ev["lockdep"]:
                res["census_stuck"] = ev["census_stuck"]
                res["lockdep_hits"] = ev["lockdep"]
                res["census_lines"] = ev["lines"]
            if shot:
                res["screenshot"] = shot
            return res
        # failure — gather evidence
        dbg = None
        try:
            dbg = vm.debugcon_report()
        except Exception:
            pass
        tri = triage.classify(
            text,
            reached=False,
            fail_marker=(marker if outcome == "failmarker" and fail_on else None),
            debugcon=dbg,
            debugcon_raw=dbg_raw,
        )
        sym = triage.symbolicate_pc(repo, arch, text)
        if sym:
            tri["salient"].append(sym)
        # Distinguish a real wedge from a slow/contended boot on timeout: if serial
        # was still flowing near the deadline the box was making progress (host
        # contention / slow TCG), not stuck — so a retry, not a regression, is the move.
        if outcome == "timeout":
            silent_s = diag.get("silent_s", 0.0)
            if silent_s < 5.0:
                hint = (
                    f"serial STILL FLOWING at deadline (silent {silent_s:.0f}s) — "
                    "likely SLOW/CONTENDED host; retry before assuming a wedge"
                )
                tri["hang_kind"] = "slow"
            else:
                hint = f"serial SILENT {silent_s:.0f}s before deadline — likely genuinely WEDGED"
                if diag.get("early_hang"):
                    hint += " (early-hang abort — failed fast, skipped the rest of boot_timeout)"
                tri["hang_kind"] = "wedged"
            last_line = diag.get("last_line", "")
            tail = f"; last serial: {last_line}" if last_line else ""
            tri["salient"].insert(0, f"timeout: {hint}{tail}")
        if shot:
            tri["screenshot"] = shot
        return {"ok": False, "outcome": outcome, **tri}
    finally:
        vm.stop()


def _net_stage(run: Run, repo: Path, arch: str) -> None:
    """Folded netstack coverage (M6): run the net suite (tests/qemu/test_net.py
    against the amd64-uefi-net config — UEFI smoke boot + a virtio-net NIC on
    slirp) and fold its pass/fail into the smoke verdict. The smoltcp netstack
    (netstack.sys) replaced the old C tcpip/lwip stack, so a netstack regression
    must turn the North Star red like any other boot regression. amd64-only (the
    virtio-net config is amd64); boot smoke itself stays no-NIC by design."""
    test_file = repo / "tests" / "qemu" / "test_net.py"
    if not test_file.exists():
        run.add(Stage("net", status="skip", detail="no test_net.py"))
        return
    cmd = [
        sys.executable,
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
    ]
    try:
        proc = subprocess.run(cmd, cwd=str(repo), capture_output=True, text=True, timeout=1200)
    except subprocess.TimeoutExpired:
        run.add(
            Stage(
                "net",
                status="fail",
                detail="net suite timed out",
                salient=["tests/qemu/test_net.py hung > 1200s"],
            )
        )
        return
    tail = (proc.stdout or "").strip().splitlines()
    summary = next(
        (ln for ln in reversed(tail) if " passed" in ln or " failed" in ln or " error" in ln),
        tail[-1] if tail else "",
    )
    if proc.returncode == 0:
        run.add(Stage("net", status="ok", detail=summary.strip("= ")))
    else:
        # DOCUMENTED FLAKE (2026-07-04, T1.4): when the boot runs themselves are
        # green, a net-suite failure under gate load is a known timing flake —
        # observed twice as "2 failed, 13 passed" while 3 arch smokes ran
        # concurrently on TCG, and 15/15 standalone immediately after, on both
        # instrumented and plain kernels. Classify NET_FLAKE (gate-retryable) so
        # the gate retries instead of flapping red. A REAL netstack regression
        # also fails the retry, so it still ends red — retries only absorb
        # nondeterminism, never determinism.
        # boot stage is "boot" for a single boot, "runN" under --repeat
        runs = [s for s in run.stages if s.name == "boot" or s.name.startswith("run")]
        boots_green = bool(runs) and all(s.status == "ok" for s in runs)
        if boots_green and not run.classification:
            run.classification = "NET_FLAKE"
        run.add(
            Stage(
                "net",
                status="fail",
                detail="net suite FAILED",
                salient=[
                    summary.strip("= "),
                    "repro: pytest tests/qemu/test_net.py --arch amd64-uefi-net",
                ],
            )
        )


def smoke(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)  # base for staleness/build dir; boot resolves vm_arch
    run = Run("smoke", arch)

    # --- staleness preflight (hard boundary) ---
    st = provenance.check_staleness(repo, arch)
    if st.stale and not args.allow_stale:
        run.classification = "STALE_MEDIA"
        run.add(
            Stage(
                "staleness",
                status="fail",
                detail="stale",
                salient=[*st.reasons, "run `vibeos-cli build` (or --allow-stale)"],
            )
        )
        return run.finalize(repo)
    if st.warm:
        run.add(Stage("staleness", status="warn", detail="WARM build"))
    elif st.stale and args.allow_stale:
        run.add(
            Stage(
                "staleness", status="warn", detail="stale (--allow-stale)", salient=st.reasons[:2]
            )
        )
    else:
        run.add(Stage("staleness", status="ok", detail="current"))

    total = max(1, int(getattr(args, "repeat", 1) or 1))
    until = getattr(args, "until", None) or ""
    fail_on = getattr(args, "fail_on", None)

    results = []
    for i in range(total):
        r = _one_boot(run, repo, arch, until=until, fail_on=fail_on, idx=i, total=total)
        results.append(r)
        status = "ok" if r["ok"] else "fail"
        detail = "reached" if r["ok"] else r.get("classification", r["outcome"])
        run.add(
            Stage(
                f"run{i + 1}" if total > 1 else "boot",
                status=status,
                detail=detail,
                salient=r.get("salient", []),
            )
        )
        # carry first failure's evidence to the top-level verdict, including the
        # retry-policy fields the gate reads from summary.json (hang_kind: a
        # slow/contended timeout is retryable; a wedge or self-confirmed wedge
        # is a regression and blocks).
        if not r["ok"] and not run.classification:
            run.classification = r.get("classification", "FAIL")
            for k in ("hang_kind", "wedge_confirmed", "census_stuck", "lockdep_hits"):
                if k in r:
                    run.extra[k] = r[k]
            if r.get("screenshot"):
                run.extra["screenshot"] = r["screenshot"]
        # fail-fast within a single run already handled by _watch; across runs
        # we keep going to characterize a flake.

    # Surface a screenshot even on a clean pass: the failure branch above already
    # carried the first failure's; otherwise take the first boot's (every boot
    # now captures one regardless of outcome).
    if "screenshot" not in run.extra:
        for r in results:
            if r.get("screenshot"):
                run.extra["screenshot"] = r["screenshot"]
                break

    npass = sum(1 for r in results if r["ok"])
    run.extra["passes"] = npass
    run.extra["runs"] = total
    run.extra["live_log"] = str(repo / "qemu" / f"{archcfg.vm_arch(arch)}_smoke_live.log")

    # Kernel sync-instrumentation health on GREEN boots: [CENSUS] STUCK /
    # [LOCKDEP] self-reports mean threads are permanently parked even though the
    # desktop came up (the "11/11 green yet eventlog silently dead" class). WARN
    # — visible and trendable in summary.json — not FAIL, because the known Mm
    # WS self-deadlock currently strands threads on every boot until its fix
    # lands; gating red on it would block all landings on a pre-existing bug.
    stuck = max((r.get("census_stuck", 0) for r in results), default=0)
    lockdep = max((r.get("lockdep_hits", 0) for r in results), default=0)
    if stuck or lockdep:
        run.extra["census_stuck"] = stuck
        run.extra["lockdep_hits"] = lockdep
        lines = next((r["census_lines"] for r in results if r.get("census_lines")), [])
        run.add(
            Stage(
                "health",
                status="warn",
                detail=f"{stuck} [CENSUS] STUCK, {lockdep} [LOCKDEP] on a green boot",
                salient=lines[:2],
            )
        )

    # Fold the netstack net-suite into the verdict (amd64 only — the virtio-net
    # config). ALWAYS run by default so a netstack regression turns the North Star
    # red like any other regression — a net failure is flagged, never hidden. It
    # is a SECOND full UEFI boot (NIC config), kept separate from the plain boot so
    # the desktop-reach North Star verdict stays free of slirp/hostfwd flake; that
    # isolation is the reason for the extra boot, not an accident. Only runs when
    # the boot itself is green. `--no-net` skips it for fast local boot-only
    # iteration (a dev escape hatch, not the gate default).
    if arch == "amd64" and npass == total and not getattr(args, "no_net", False):
        _net_stage(run, repo, arch)

    if total > 1:
        run.extra["flake_rate"] = f"{npass}/{total}"
        if 0 < npass < total:
            run.classification = run.classification or "FLAKY"
            # a flake is a real signal — don't call it green
            run.verdict = "FAIL"
            run.add(Stage("flake", status="fail", detail=f"{npass}/{total} pass"))
    return run.finalize(repo)


def boot(args, repo: Path) -> int:
    """Raw fast boot — NO staleness gate, reuses smoke's EXACT arch-config
    (machine/KVM/vga/media/debugcon) so the fast path can't diverge from smoke
    (smss-csrss #3: q35-vs-pc misdiagnosis). Writes artifacts, decodes debugcon."""
    arch = archcfg.base_arch(args.arch)
    run = Run("boot", arch)
    r = _one_boot(
        run,
        repo,
        arch,
        until=(args.until or ""),
        fail_on=getattr(args, "fail_on", None),
        idx=0,
        total=1,
    )
    run.add(
        Stage(
            "boot",
            status="ok" if r["ok"] else "fail",
            detail="reached" if r["ok"] else r.get("classification", r["outcome"]),
            salient=r.get("salient", []),
        )
    )
    if not r["ok"]:
        run.classification = r.get("classification", "FAIL")
    run.extra["live_log"] = str(repo / "qemu" / f"{archcfg.vm_arch(arch)}_smoke_live.log")
    return run.finalize(repo)
