"""`vibeos-cli smoke` — the verdict front door (plan §9, §20.1).

Staleness preflight (hard-fail unless --allow-stale), then boot via the existing
QemuVM (which already owns slots/reaper/cap/live-log). Supports --repeat N
(independent cold boots for flake rate), --until <marker>, --fail-on <regex>
(abort at first hard-failure marker), and auto-triage on failure.
"""
from __future__ import annotations

import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

from . import archcfg, provenance, triage
from .artifacts import Run, Stage
from ..vm import QemuVM, get_arch_config


def _watch(vm: QemuVM, until: str, fail_on: str | None, timeout: float):
    """Poll serial for the success marker or a fail-marker regex. Returns
    (outcome, text) where outcome in {reached, failmarker, timeout, dead}."""
    fail_re = re.compile(fail_on) if fail_on else None
    buf = ""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        chunk = vm.serial.read_available(timeout=0.5)
        if chunk:
            buf += chunk
            if until in buf:
                return "reached", buf
            if fail_re and fail_re.search(buf):
                return "failmarker", buf
    return "timeout", buf


def _settle(vm: QemuVM, seconds: float) -> None:
    """After the boot marker, let user-mode finish drawing the desktop before
    the screenshot. The marker (`_tWinMain entered`) fires the instant explorer
    starts — the taskbar/Start button/tray are built ~15s later. Without this,
    the canonical screenshot is a half-built (taskbar-less) desktop, which both
    hides real shell regressions and looks alarmingly blank. Drains serial while
    waiting so the chardev buffer can't stall the VM."""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        vm.serial.read_available(timeout=0.5)


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


def _one_boot(run: Run, repo: Path, arch: str, *, until: str, fail_on: str | None,
              idx: int, total: int) -> dict:
    """A single independent boot. Returns {ok, outcome, classification, ...}.
    `arch` is the base arch (i386/amd64/arm64); the boot config is the vm_arch
    (i386 → i386-uefi) since x86 boots UEFI now."""
    config = get_arch_config(archcfg.vm_arch(arch))
    marker = until or config.boot_marker
    work = run.dir / f"boot{idx}"
    work.mkdir(exist_ok=True)
    vm = QemuVM(config, work)
    label = f"boot {idx+1}/{total}" if total > 1 else "boot"
    try:
        vm.start()
    except Exception as e:  # noqa: BLE001 — surface any launch failure
        run.log(f"{label}: start failed: {e}", header=label)
        return {"ok": False, "outcome": "launch_fail", "classification": "LAUNCH_FAIL",
                "salient": [str(e).splitlines()[0][:140]]}
    try:
        outcome, text = _watch(vm, marker, fail_on, config.boot_timeout)
        run.log(text[-8000:], header=f"{label} serial tail")
        # On success, let the desktop settle (taskbar/tray render ~15s after the
        # marker) so the canonical screenshot shows the real shell. On failure,
        # screenshot immediately to capture the broken state.
        if outcome == "reached" and config.desktop_settle_seconds > 0:
            run.log(f"settling {config.desktop_settle_seconds:.0f}s for desktop",
                    header=label)
            _settle(vm, config.desktop_settle_seconds)
        # Always capture a PNG screenshot (pass or fail), while the VM is alive.
        shot = _screenshot(vm, work, repo, arch)
        if outcome == "reached":
            res = {"ok": True, "outcome": "reached", "classification": ""}
            if shot:
                res["screenshot"] = shot
            return res
        # failure — gather evidence
        dbg = None
        try:
            dbg = vm.debugcon_report()
        except Exception:
            pass
        tri = triage.classify(text, reached=False,
                              fail_marker=(marker if outcome == "failmarker" and fail_on else None),
                              debugcon=dbg)
        sym = triage.symbolicate_pc(repo, arch, text)
        if sym:
            tri["salient"].append(sym)
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
    cmd = [sys.executable, "-m", "pytest", str(test_file),
           "--arch", "amd64-uefi-net", "--timeout", "600", "-q",
           "-p", "no:cacheprovider"]
    try:
        proc = subprocess.run(cmd, cwd=str(repo), capture_output=True,
                              text=True, timeout=1200)
    except subprocess.TimeoutExpired:
        run.add(Stage("net", status="fail", detail="net suite timed out",
                      salient=["tests/qemu/test_net.py hung > 1200s"]))
        return
    tail = (proc.stdout or "").strip().splitlines()
    summary = next((l for l in reversed(tail) if " passed" in l or " failed" in l
                    or " error" in l), tail[-1] if tail else "")
    if proc.returncode == 0:
        run.add(Stage("net", status="ok", detail=summary.strip("= ")))
    else:
        run.add(Stage("net", status="fail", detail="net suite FAILED",
                      salient=[summary.strip("= "),
                               "repro: pytest tests/qemu/test_net.py --arch amd64-uefi-net"]))


def smoke(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)   # base for staleness/build dir; boot resolves vm_arch
    run = Run("smoke", arch)

    # --- staleness preflight (hard boundary) ---
    st = provenance.check_staleness(repo, arch)
    if st.stale and not args.allow_stale:
        run.classification = "STALE_MEDIA"
        run.add(Stage("staleness", status="fail", detail="stale",
                      salient=st.reasons + ["run `vibeos-cli build` (or --allow-stale)"]))
        return run.finalize(repo)
    if st.warm:
        run.add(Stage("staleness", status="warn", detail="WARM build"))
    elif st.stale and args.allow_stale:
        run.add(Stage("staleness", status="warn", detail="stale (--allow-stale)",
                      salient=st.reasons[:2]))
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
        run.add(Stage(f"run{i+1}" if total > 1 else "boot", status=status,
                      detail=detail, salient=r.get("salient", [])))
        # carry first failure's evidence to the top-level verdict
        if not r["ok"] and not run.classification:
            run.classification = r.get("classification", "FAIL")
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
    run.extra["live_log"] = str((repo / "qemu" / f"{archcfg.vm_arch(arch)}_smoke_live.log"))

    # M6: fold the netstack net-suite into the verdict. Only when boot is green
    # (no point net-testing a broken boot) and only on amd64 (the virtio-net
    # config). --no-net opts out for fast boot-only iteration.
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
    r = _one_boot(run, repo, arch, until=(args.until or ""),
                  fail_on=getattr(args, "fail_on", None), idx=0, total=1)
    run.add(Stage("boot", status="ok" if r["ok"] else "fail",
                  detail="reached" if r["ok"] else r.get("classification", r["outcome"]),
                  salient=r.get("salient", [])))
    if not r["ok"]:
        run.classification = r.get("classification", "FAIL")
    run.extra["live_log"] = str((repo / "qemu" / f"{archcfg.vm_arch(arch)}_smoke_live.log"))
    return run.finalize(repo)
