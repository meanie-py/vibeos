"""`vibeos-cli smoke` — the verdict front door (plan §9, §20.1).

Staleness preflight (hard-fail unless --allow-stale), then boot via the existing
QemuVM (which already owns slots/reaper/cap/live-log). Supports --repeat N
(independent cold boots for flake rate), --until <marker>, --fail-on <regex>
(abort at first hard-failure marker), and auto-triage on failure.
"""

from __future__ import annotations

import contextlib
import os
import re
import subprocess
import sys
import time
from collections.abc import Callable
from pathlib import Path

from ..vm import QemuVM, get_arch_config
from . import archcfg, hostlock, provenance, triage
from . import config as cli_config
from . import ladder as ladder_mod
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


def _cpu_state(
    vm: QemuVM, work: Path, repo: Path, arch: str, serial: str
) -> tuple[str | None, list[str]]:
    """Persist where each vCPU was parked on a hang, symbolicated. (path, lines).

    Only meaningful for a timeout: every other failure already carries its own
    PC on serial. Returns ([], None) rather than raising — see
    QemuVM.capture_cpu_state on why a diagnostic must never fail its own run.
    """
    # Prefer what wait_for_boot grabbed AT the deadline over a fresh sample taken
    # some seconds later: on a hang the two agree, but if the box was merely slow
    # the later one has moved on and would misreport where it was stuck.
    state = vm.hang_cpu_state
    if not state:
        try:
            state = vm.capture_cpu_state()
        except Exception:
            return None, []
    lines = triage.symbolicate_cpu_state(
        repo, arch, state, mp="ntkrnlmp.exe" in serial, serial=serial
    )
    path = work / "hang-cpustate.txt"
    try:
        path.write_text(state + ("\n" + "\n".join(lines) + "\n" if lines else ""))
    except OSError:
        return None, lines
    return str(path), lines


# freeldr announces every image it loads; the kernel is the first one. Used to
# prove WHICH kernel a `--smp` run actually booted (see _one_boot).
_KERNEL_LOAD_RE = re.compile(r"\[FLDR\] load \S*[\\/](ntoskrnl\.exe|ntkrnlmp\.exe)", re.IGNORECASE)
# …and WHICH HAL, from the same stamp. freeldr loads the file named by `/HAL=`
# but registers it in the loader block as "hal.dll", so this serial line is the
# only place the on-disk name is visible — nothing downstream can tell you which
# flavor is running.
_HAL_LOAD_RE = re.compile(r"\[FLDR\] load \S*[\\/](halmp\.dll|hal\.dll)", re.IGNORECASE)


def _booted_kernel(text: str) -> str | None:
    """The kernel image freeldr loaded, from serial. None if not seen yet."""
    m = _KERNEL_LOAD_RE.search(text)
    return m.group(1).lower() if m else None


def _booted_hal(text: str) -> str | None:
    """The HAL image freeldr loaded, from serial. None if not seen yet."""
    m = _HAL_LOAD_RE.search(text)
    return m.group(1).lower() if m else None


def _smp_check(text: str, arch: str) -> dict | None:
    """Prove an `--smp N` boot got the SMP it asked for. Returns a failure dict
    or None.

    An SMP run has three ways to look GREEN while being nothing of the kind, and
    a desktop screenshot distinguishes none of them:
      1. UP kernel — media built without --smp-kernel (freeldr picks the kernel,
         so nothing else notices). x86 would 0x79 on a HAL mis-pair, but a plain
         UP kernel + UP HAL is simply a normal UP boot.
      2. MP kernel on a UP hal.dll — arm64 has no MISMATCHED_HAL handshake, so it
         boots, HalStartNextProcessor is the FALSE stub, and it silently runs on
         1 CPU (smp-bringup-quest.md M5c; cost a debug round already).
      3. APs started but never scheduled — the boot marker only proves the BSP
         reached the desktop. test_smoke.py::test_milestones checks this, but the
         CLI smoke fast path does not run pytest, so `smoke --smp N` had no AP
         liveness check at all.

    (2) is checked twice on purpose. The load-time name check fires first and on
    every arch; the `UP HAL stub called` runtime marker below is the backstop for
    a HAL that somehow IS the UP flavor under an MP name. Before halmp.dll
    existed the name carried no information — the flavor was a cmake cache var
    baked into a file always called hal.dll — so only the runtime marker existed.
    """
    # n == 0 means "no --smp given", i.e. a DEFAULT boot. That used to return
    # early, which was right while the default media was UP -- there was nothing
    # to assert. Since the MP promotion (2026-09-12) the default media boots the
    # MP pair, so the same names must be checked on every boot: otherwise a
    # regression that quietly reverts DefaultOS= to the UP entry passes the whole
    # gate green, which is precisely the false-green (`a boot that reached the
    # desktop running the wrong kernel looks identical to a correct one`) that
    # UP_KERNEL_ON_SMP_RUN exists to catch. Only the AP-liveness check below
    # genuinely needs n > 1.
    smp = os.environ.get("VIBEOS_QEMU_SMP", "").strip()
    n = int(smp) if smp.isdigit() and int(smp) >= 1 else 0
    how = f"--smp {n}" if n else "the default boot"

    booted = _booted_kernel(text)
    if booted and booted != "ntkrnlmp.exe":
        # No arch has a "forgot --smp-kernel" failure mode any more: every media
        # carries both pairs and both freeldr entries, and vm.smp_selected_iso
        # repoints DefaultOS= in a copy of it (raising, never falling back, if it
        # can't). arm64 was the last holdout — it used to get its MP pair from a
        # different BUILD — so a UP kernel here now means the patched ini did not
        # reach freeldr on any arch, not a build mistake.
        mp_entry = "ReactOS_Arm64_MP" if arch.startswith("arm64") else "LiveCD_SMP"
        fix = (
            f"the media's DefaultOS= was patched to {mp_entry} but freeldr booted "
            f"the UP entry anyway — check the [{mp_entry}] Options line"
        )
        return {
            "classification": "UP_KERNEL_ON_SMP_RUN",
            "salient": [
                f"{how} booted {booted}, not ntkrnlmp.exe",
                fix,
            ],
        }
    hal = _booted_hal(text)
    if hal and hal != "halmp.dll":
        return {
            "classification": "SMP_UP_HAL",
            "salient": [
                f"{how} loaded {hal}, not halmp.dll — the MP kernel is running "
                "on the UP HAL, so HalStartNextProcessor is the FALSE stub and "
                "this boot is silently 1 CPU",
                "the boot entry is missing /HAL=halmp.dll (livecd.ini "
                "[LiveCD_SMP], or mkramdisk.sh --smp on arm64)",
            ],
        }
    if "UP HAL stub called" in text:
        return {
            "classification": "SMP_UP_HAL",
            "salient": [
                "MP kernel is running on a UP HAL — HalStartNextProcessor is "
                "the FALSE stub, so this boot is silently 1 CPU",
                f"loaded HAL was {hal or 'not seen on serial'}, so the UP body is "
                "wearing an MP name — check hal-rust's `smp` feature reached "
                "halmp-cargo",
            ],
        }
    # Same requirement test_milestones enforces: one marker per AP, printed by
    # KiSmpNoteApSwitch on the AP's first non-idle context switch.
    # n == 0 (default boot) yields an empty range: one CPU has no APs to check.
    missing = [c for c in range(1, n) if f"CPU{c} scheduling live" not in text]
    if missing:
        return {
            "classification": "SMP_AP_NOT_SCHEDULING",
            "salient": [
                f"-smp {n}: no `[SMP] CPU<n> scheduling live` for CPU "
                + ", ".join(str(c) for c in missing),
                "the BSP reached the desktop but that AP never ran a non-idle thread",
            ],
        }
    return None


def _one_boot(run: Run, repo: Path, arch: str, **kwargs) -> dict:
    started = time.monotonic()
    timings: dict[str, float] = {}
    result = _boot_vm(run, repo, arch, timings=timings, **kwargs)
    timings["total_seconds"] = round(time.monotonic() - started, 3)
    result["timings"] = timings
    serial = run.dir / f"boot{kwargs['idx']}" / "serial.log"
    if serial.is_file():
        result["serial_log"] = str(serial)
    return result


def _launch_with_retry(
    make_vm: Callable[[], QemuVM],
    timings: dict[str, float],
    log: Callable[[str], None],
    label: str,
) -> tuple[QemuVM | None, int, Exception | None]:
    """Start a VM, retrying a host-side launch failure.

    Returns (vm, attempts, last_exc); `last_exc` is None on success. A fresh VM
    is built per attempt because the port and serial socket path are chosen in
    QemuVM.__init__ -- retrying start() on the same instance would retry onto
    the same bad port and could never recover.
    """
    last_exc: Exception | None = None
    vm: QemuVM | None = None
    attempts = 0
    while attempts < _LAUNCH_ATTEMPTS:
        attempts += 1
        vm = make_vm()
        started = time.monotonic()
        try:
            vm.start()
            timings["start_seconds"] = round(time.monotonic() - started, 3)
            timings.update(vm.launch_timings)
            if attempts > 1:
                # Loud on the way through: a retried launch is still an incident.
                timings["launch_attempts"] = attempts
                log(f"{label}: launched on attempt {attempts} (host-side flake)")
            return vm, attempts, None
        except Exception as e:
            last_exc = e
            first = str(e).splitlines()[0][:140]
            log(f"{label}: start failed (attempt {attempts}/{_LAUNCH_ATTEMPTS}): {first}")
            with contextlib.suppress(Exception):
                vm.stop()
            if attempts < _LAUNCH_ATTEMPTS:
                time.sleep(_LAUNCH_RETRY_SECONDS)
    return vm, attempts, last_exc


# Host-side launch retries. Small on purpose: the failure mode this exists for
# resolves on the next attempt (a lost socket / port), so a third attempt buys
# almost nothing and a large number would turn a genuinely broken launch into a
# slow one. Overridable for tests.
_LAUNCH_ATTEMPTS = 3
_LAUNCH_RETRY_SECONDS = 2.0


def _boot_vm(
    run: Run,
    repo: Path,
    arch: str,
    *,
    until: str,
    fail_on: str | None,
    idx: int,
    total: int,
    want_ladder: bool = False,
    timings: dict[str, float] | None = None,
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
    label = f"boot {idx + 1}/{total}" if total > 1 else "boot"
    if backend != "native":
        run.log(f"backend={backend} (qemu.windows={cli_cfg.get('qemu.windows', {})})", header=label)
    timings = timings if timings is not None else {}

    # A launch failure happens BEFORE a single guest instruction runs, so it
    # says nothing about the OS -- but it still consumed a boot and, in a
    # --repeat run, it lands as a red flake next to real verdicts. Two host-side
    # failures cost runs on 2026-09-20: "QEMU did not connect to serial socket
    # within 15.0s" killed one boot of five, and the net suite's chardev
    # "Failed to connect to 127.0.0.1:<port>" errored all 15 of its tests
    # identically. Both are the host losing a socket, both passed on a rerun of
    # the same media.
    #
    # So retry the LAUNCH, with a fresh QemuVM each time because the port and
    # socket path are chosen in the constructor -- reusing the instance would
    # retry onto the same bad port. Retries are LOGGED and counted into the
    # timings so a worsening host cannot hide behind them: this is meant to stop
    # a flake costing a 50-minute run, not to make it invisible. A genuinely
    # broken launch (missing media, no QEMU) fails all attempts in seconds and
    # reports exactly as before.
    vm, attempts, last_exc = _launch_with_retry(
        lambda: QemuVM(
            config,
            work,
            backend=backend,
            windows=cli_cfg.get("qemu.windows", {}),
            serial_log=work / "serial.log",
        ),
        timings,
        lambda msg: run.log(msg, header=label),
        label,
    )

    if last_exc is not None:
        run.log(f"{label}: start failed: {last_exc}", header=label)
        return {
            "ok": False,
            "outcome": "launch_fail",
            "classification": "LAUNCH_FAIL",
            "launch_attempts": attempts,
            "salient": [
                f"{attempts} launch attempts, all failed",
                str(last_exc).splitlines()[0][:140],
            ],
        }
    assert vm is not None  # _launch_with_retry returns a VM whenever last_exc is None
    try:
        watching = time.monotonic()
        outcome, text, diag = _watch(vm, marker, fail_on, config.boot_timeout)
        timings["watch_seconds"] = round(time.monotonic() - watching, 3)
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
            settling = time.monotonic()
            settle_txt, shell_seen = _settle_watch(
                vm, config.desktop_settle_seconds, "shell ready" if need_shell else None
            )
            timings["settle_seconds"] = round(time.monotonic() - settling, 3)
            if settle_txt:
                text += settle_txt
                run.log(settle_txt[-4000:], header=f"{label} settle serial")
        # --ladder: the WoW64 probe guests run AFTER explorer, i.e. after the
        # marker this boot already matched, so without an explicit extra watch
        # the gate cannot see them at all (ladder.py documents the two times
        # that hid a real red rung). Bounded, and strictly additive: with the
        # flag off not a byte of the boot path changes.
        if want_ladder and outcome == "reached":
            run.log(
                f"draining up to {ladder_mod.LADDER_TIMEOUT_SECONDS:.0f}s for the WoW64 ladder",
                header=label,
            )
            _lad_t0 = time.monotonic()
            # The drain waits on the same media answer check() grades with --
            # otherwise it can stop between the last parity verdict and an
            # optional guest's launch line, and check() fails a guest mid-run.
            # program_done_expected: the media is this tree's (staleness was
            # checked before boot), so its winlogon prints PROGRAM_DONE.
            ladder_txt = ladder_mod.watch(
                vm,
                seen=text,
                optional_staged=ladder_mod.media_optional_staged(repo, arch),
                program_done_expected=True,
            )
            # Record what the drain actually COST, not just its ceiling. This is
            # additive wall-clock on every gate that stages guests, and the ceiling
            # (420s) is ~2x the whole gate's modelled warm runtime -- so the real
            # number has to be visible in the artifact, or nobody can tell a healthy
            # ladder from one that sat waiting for a guest that never reported.
            run.log(
                f"ladder drain: {time.monotonic() - _lad_t0:.1f}s "
                f"(ceiling {ladder_mod.LADDER_TIMEOUT_SECONDS:.0f}s)",
                header=label,
            )
            if ladder_txt:
                text += ladder_txt
                run.log(ladder_txt[-8000:], header=f"{label} ladder serial")
        # Always capture a PNG screenshot (pass or fail), while the VM is alive.
        shot = _screenshot(vm, work, repo, arch)
        dbg_raw = ""
        try:
            dbg_raw = vm.read_debugcon().decode("ascii", errors="replace")
        except Exception:
            pass
        if outcome == "reached":
            # A green desktop does not prove an `--smp N` run actually got SMP.
            smp_fail = _smp_check(text, arch)
            if smp_fail is not None:
                return {
                    "ok": False,
                    "outcome": "smp-check",
                    **smp_fail,
                    **({"screenshot": shot} if shot else {}),
                }
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
            # A green desktop says nothing about the 32-bit guests, which is the
            # whole point of --ladder: fail the boot on a red rung instead of
            # letting it pass because explorer came up.
            if want_ladder:
                # Ask the MEDIA which optional guests are aboard, not the
                # serial: an optional guest runs after the parity set, so the
                # drain can end before its launch line is even emitted, and a
                # hung one then reads as absent (observed: a wedged sqlite32
                # passed as "9/9 guests PASS").
                lad = ladder_mod.check(
                    text, optional_staged=ladder_mod.media_optional_staged(repo, arch)
                )
                if lad is not None:
                    if shot:
                        lad["screenshot"] = shot
                    return {"ok": False, "outcome": "ladder", **lad}
                run.log(f"ladder: {ladder_mod.summary(text)}", header=label)
            res = {"ok": True, "outcome": "reached", "classification": ""}
            ev = triage.wedge_evidence(text)
            if ev["census_stuck"] or ev["lockdep"] or ev["cswitch"]:
                res["census_stuck"] = ev["census_stuck"]
                res["lockdep_hits"] = ev["lockdep"]
                res["cswitch_hits"] = ev["cswitch"]
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
            # A hang is the ONE failure with no PC of its own: no bugcheck banner,
            # no panic, nothing on serial. Ask QEMU directly, while the VM is still
            # alive (vm.stop() is in the `finally` below).
            cpu_path, cpu_lines = _cpu_state(vm, work, repo, arch, text)
            if cpu_path:
                tri["cpu_state"] = cpu_path
            tri["salient"].extend(f"hang: {ln}" for ln in cpu_lines)
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
        stopping = time.monotonic()
        vm.stop()
        timings["stop_seconds"] = round(time.monotonic() - stopping, 3)


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
        # smoke already validated this media. A second clean-build fixture
        # races other networking runs on the same build directory.
        "--skip-clean-build",
    ]
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(repo),
            capture_output=True,
            text=True,
            timeout=1200,
            env={**os.environ, "VIBEOS_ARTIFACT_DIR": str(run.dir / "net")},
        )
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
    run.log((proc.stdout or "") + (proc.stderr or ""), header="network pytest")
    tail = (proc.stdout or "").strip().splitlines()
    summary = next(
        (ln for ln in reversed(tail) if " passed" in ln or " failed" in ln or " error" in ln),
        tail[-1] if tail else "",
    )
    if proc.returncode == 0:
        run.add(Stage("net", status="ok", detail=summary.strip("= ")))
    else:
        # §9.84: the suite queued on the hostlock broker its whole budget and never
        # booted — a busy host, not a netstack failure. Classified apart from
        # NET_FLAKE so the red names the real cause.
        starved = hostlock.looks_starved((proc.stdout or "") + "\n" + (proc.stderr or ""))
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
        if starved and not run.classification:
            run.classification = "HOST_STARVED"
        elif boots_green and not run.classification:
            run.classification = "NET_FLAKE"
        run.add(
            Stage(
                "net",
                status="fail",
                detail="net suite FAILED",
                salient=[
                    "net lane never booted — queued on host CPU leases (§9.84); "
                    "`vibeos-cli ps` names the holder"
                    if starved
                    else summary.strip("= "),
                    "repro: pytest tests/qemu/test_net.py --arch amd64-uefi-net",
                ],
            )
        )


def _payload_stage(run: Run, repo: Path, arch: str, payloads: str) -> None:
    """Opt-in payload gate (--payload <names> / legacy --doom): run
    tests/qemu/test_payload.py (launch marker + frame-delta render gate +
    input-drive playable gate, per payload YAML) and fold the verdict in.
    Requires media built with VIBEOS_PAYLOAD=<names> — the suite itself fails
    loud with the build recipe if the app never launches. Design doc:
    docs/engineering/payload-harness-design.md."""
    test_file = repo / "tests" / "qemu" / "test_payload.py"
    pytest_arch = {"i386": "i386-uefi", "amd64": "amd64-uefi"}.get(arch, arch)
    # Payloads that declare `net: true` (e.g. httpd) need the -net arch variant
    # (NIC + payload hostfwd). Sniff the specs; one net payload upgrades the boot.
    # The pytest timeout covers the boot AND each payload's launch drain. A fixed 600s let a
    # payload's declared launch_drain_secs be cut short silently (procstorm declares 1800), and
    # failed regdiff on amd64 while its VERDICT line was still on its way. Budget the boot
    # separately and add the longest drain.
    boot_budget = 600
    drain = 0.0
    try:
        import yaml as _yaml

        for _name in payloads.split(","):
            _spec_path = repo / "tools" / "payloads" / _name.strip() / "payload.yaml"
            if _spec_path.is_file():
                _spec = _yaml.safe_load(_spec_path.read_text()) or {}
                _test = _spec.get("test", {}) or {}
                drain = max(drain, float(_test.get("launch_drain_secs", 30)))
                if _test.get("net") and not pytest_arch.endswith("-net"):
                    pytest_arch += "-net"
    except ImportError:
        pass  # no pyyaml -> test_payload itself will fail loud on the guard
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        str(test_file),
        "--arch",
        pytest_arch,
        "--payload",
        payloads,
        "--skip-clean-build",  # smoke already gated staleness; don't rebuild
        "--timeout",
        str(int(boot_budget + drain)),
        "-q",
        "-p",
        "no:cacheprovider",
    ]
    outer_timeout = int(boot_budget + drain) * 2
    # The payload suite boots its OWN VM. Without an artifact dir its serial landed in a pytest
    # tmp dir that pytest prunes, so a payload that reports its result over serial (regdiff's
    # VERDICT line, a sweep's per-test lines) left no record of it anywhere once the run
    # ended — the stage summary only says whether the launch marker appeared.
    payload_dir = run.dir / "payload"
    payload_dir.mkdir(parents=True, exist_ok=True)
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(repo),
            capture_output=True,
            text=True,
            timeout=outer_timeout,
            env={**os.environ, "VIBEOS_ARTIFACT_DIR": str(payload_dir)},
        )
    except subprocess.TimeoutExpired:
        run.add(
            Stage(
                "payload",
                status="fail",
                detail=f"payload suite ({payloads}) timed out",
                salient=[f"tests/qemu/test_payload.py hung > {outer_timeout}s"],
            )
        )
        return
    (payload_dir / "pytest.log").write_text((proc.stdout or "") + (proc.stderr or ""))
    tail = (proc.stdout or "").strip().splitlines()
    summary = next(
        (ln for ln in reversed(tail) if " passed" in ln or " failed" in ln or " error" in ln),
        tail[-1] if tail else "",
    )
    if proc.returncode == 0:
        run.add(Stage("payload", status="ok", detail=summary.strip("= ")))
    else:
        run.add(
            Stage(
                "payload",
                status="fail",
                detail=f"payload suite ({payloads}) FAILED",
                salient=[
                    summary.strip("= "),
                    f"repro: pytest tests/qemu/test_payload.py --arch {pytest_arch} "
                    f"--payload {payloads}",
                    f"serial + pytest.log: {payload_dir}",
                ],
            )
        )


def smoke(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)  # base for staleness/build dir; boot resolves vm_arch
    run = Run("smoke", arch)

    # --timing: stamp the live serial log with elapsed seconds so the boot can
    # be decomposed into phases afterwards. The env var is read by the
    # SerialClient tee (serial_client.py) in THIS process's children.
    if getattr(args, "timing", False):
        os.environ["VIBEOS_SERIAL_TIMESTAMPS"] = "1"

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

    # --- probe preflight (hard boundary) ---
    # An instrumented run whose probe is not in the media is worse than no run:
    # it spends the boots AND reads as evidence of absence. Fail before booting.
    require = [s for s in (getattr(args, "require_probe", None) or []) if s.strip()]
    if require:
        missing = provenance.probes_missing_from_media(repo, arch, require)
        if missing:
            run.classification = "PROBE_NOT_IN_MEDIA"
            run.add(
                Stage(
                    "probe-check",
                    status="fail",
                    detail=f"{len(missing)} of {len(require)} probe(s) absent",
                    salient=[*missing, "rebuild so the probe reaches the media, then re-run"],
                )
            )
            return run.finalize(repo)
        run.add(
            Stage(
                "probe-check",
                status="ok",
                detail=f"{len(require)} probe(s) present in every boot image",
            )
        )

    total = max(1, int(getattr(args, "repeat", 1) or 1))
    until = getattr(args, "until", None) or ""
    fail_on = getattr(args, "fail_on", None)

    # Tripwire channel: hand the config to QemuVM via env (read by
    # vm.py _tripwire_args -> QEMU -fw_cfg -> kernel tripwire.rs).
    tripwire = (getattr(args, "tripwire", "") or "").strip()
    if tripwire:
        os.environ["VIBEOS_TRIPWIRE"] = tripwire
        run.add(Stage("tripwire", status="warn", detail=tripwire))

    # SMP channel: hand the vCPU count to QemuVM via env (read by
    # vm.py _smp_args -> QEMU -smp N). Opt-in; default boots stay UP.
    smp = int(getattr(args, "smp", 0) or 0)
    if smp:
        os.environ["VIBEOS_QEMU_SMP"] = str(smp)
        run.add(Stage("smp", status="warn", detail=f"-smp {smp}"))

    results = []
    for i in range(total):
        r = _one_boot(
            run,
            repo,
            arch,
            until=until,
            fail_on=fail_on,
            idx=i,
            total=total,
            want_ladder=bool(getattr(args, "ladder", False)),
        )
        results.append(r)
        status = "ok" if r["ok"] else "fail"
        detail = "reached" if r["ok"] else r.get("classification", r["outcome"])
        run.add(
            Stage(
                f"run{i + 1}" if total > 1 else "boot",
                status=status,
                seconds=r["timings"]["total_seconds"],
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
            carry = ("hang_kind", "wedge_confirmed", "census_stuck", "lockdep_hits", "cswitch_hits")
            for k in carry:
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
    _boot_evidence(run, results)

    # Kernel sync-instrumentation health on GREEN boots: [CENSUS] STUCK /
    # [LOCKDEP] self-reports mean threads are permanently parked even though the
    # desktop came up (the "11/11 green yet eventlog silently dead" class). WARN
    # — visible and trendable in summary.json — not FAIL, because the known Mm
    # WS self-deadlock currently strands threads on every boot until its fix
    # lands; gating red on it would block all landings on a pre-existing bug.
    stuck = max((r.get("census_stuck", 0) for r in results), default=0)
    lockdep = max((r.get("lockdep_hits", 0) for r in results), default=0)
    cswitch = max((r.get("cswitch_hits", 0) for r in results), default=0)
    if stuck or lockdep or cswitch:
        run.extra["census_stuck"] = stuck
        run.extra["lockdep_hits"] = lockdep
        run.extra["cswitch_hits"] = cswitch
        lines = next((r["census_lines"] for r in results if r.get("census_lines")), [])
        run.add(
            Stage(
                "health",
                status="warn",
                detail=f"{stuck} [CENSUS] STUCK, {lockdep} [LOCKDEP], "
                f"{cswitch} [CSWITCH] on a green boot",
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

    # Opt-in payload gate (--payload / legacy --doom alias): its own boot via
    # pytest (tests/qemu/test_payload.py) against VIBEOS_PAYLOAD-staged media.
    # Only when the plain boot is green — a red North Star makes the payload
    # verdict meaningless.
    payloads = getattr(args, "payload", "") or ""
    if getattr(args, "doom", False) and "doom" not in payloads.split(","):
        payloads = f"{payloads},doom".strip(",")
    if payloads and npass == total:
        _payload_stage(run, repo, arch, payloads)

    if total > 1:
        run.extra["flake_rate"] = f"{npass}/{total}"
        if 0 < npass < total:
            run.classification = run.classification or "FLAKY"
            # a flake is a real signal — don't call it green
            run.verdict = "FAIL"
            run.add(Stage("flake", status="fail", detail=f"{npass}/{total} pass"))

    # --timing: decompose the stamped live log into boot phases + biggest gaps.
    if getattr(args, "timing", False):
        live = Path(run.extra.get("live_log", run.dir / "serial.log"))
        tool = repo / "tools" / "boot_phase_report.py"
        if live.is_file() and tool.is_file():
            cp = subprocess.run(
                [sys.executable, str(tool), str(live), "--sizes", str(repo / f"build-{arch}")],
                capture_output=True,
                text=True,
                timeout=120,
            )
            report = (cp.stdout or "").strip()
            if report:
                run.log(report[:12000], header="boot-phase timing")

    return run.finalize(repo)


def _boot_evidence(run: Run, results: list[dict]) -> None:
    """Keep every boot; top-level serial must describe the selected verdict."""
    run.extra["boots"] = results
    selected = next((r for r in results if not r["ok"]), results[0])
    if selected.get("serial_log"):
        run.extra["live_log"] = selected["serial_log"]


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
    _boot_evidence(run, [r])
    return run.finalize(repo)
