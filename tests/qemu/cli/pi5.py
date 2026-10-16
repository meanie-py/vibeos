"""`vibeos-cli pi5`: the Raspberry Pi 5 bench, the Pi 4 bench's twin.

The Pi 5's UEFI (worproject rpi5-uefi v0.3, the last release) has no driver for the
board's Ethernet, so there is no netboot. Instead the SD card boots Raspberry Pi OS
by default, and that Linux is the stager: `pi5 boot` SSHes in, copies this tree's
loader and ramdisk onto the card (tools/pi5/stage.sh) and reboots with `tryboot`,
which boots UEFI + VibeOS exactly once. The next reset is Linux again.

Board-specific values (plug, Pico, capture card, bench directory, lease) come from
tools/pi5/env.sh; the mechanisms are tools/pi4's. Docs: docs/engineering/pi5-bench.md.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from pathlib import Path

from . import pi4, pi4lease, provenance, triage
from .artifacts import Run, Stage

BOARD = "pi5"
SSH_WAIT = 150  # seconds from power-on to the stager's sshd
# The VibeOS desktop's background, as the Pi 5's capture card renders it. Without a
# serial console HDMI is the only witness, and this colour is what the default
# desktop shows behind any window (measured 2026-10-04: 41% of a desktop frame with
# Notepad open, 0% of the loader's text screen or a dark frame).
DESKTOP_RGB = (50, 121, 187)
DESKTOP_SHARE = 0.2
# `pi5 boot --stortest`: the volume stortest.exe looks for, served by the Pico as its
# remote disk, and the two payloads its read and write checks compare against
# (stortest.c's MARKER_SUBSTR and WRITE_CONTENT).
STORTEST_LABEL = "VIBEUSB"
STORTEST_MARKER = b"vibeos usb e2e marker\r\n"
STORTEST_WROTE = b"vibeos wrote this through fastfat+usbstor\r\n"
# stortest polls 120 s for the volume, then writes; this is the wait after the desktop.
STORTEST_WAIT = 240


def bench_dir() -> Path:
    return Path(os.environ.get("PI5_BENCH", "/mnt/d/home/vibeos-rpi5-setup"))


def _script(repo: Path, name: str) -> list[str]:
    return ["bash", str(repo / "tools" / "pi5" / name)]


def _env_sh(repo: Path, body: str) -> list[str]:
    """Run shell `body` with tools/pi5/env.sh sourced (pi5_ssh and friends)."""
    return ["bash", "-c", f'. "{repo}/tools/pi5/env.sh"; {body}']


def ssh_up(repo: Path) -> bool:
    rc, _ = pi4._quick(_env_sh(repo, "pi5_ssh true"), timeout=20)
    return rc == 0


def plug_configured(repo: Path) -> bool:
    """Whether env.sh names a plug. It holds the default, so PI5_PLUG need not be set."""
    rc, out = pi4._quick(_env_sh(repo, 'printf %s "$PI4_PLUG"'), timeout=10)
    return rc == 0 and bool(out.strip())


def payload_manifest() -> dict:
    try:
        return json.loads((bench_dir() / "payload.json").read_text())
    except (OSError, ValueError):
        return {}


def desktop_share(path: Path) -> float | None:
    """Fraction of the frame in the desktop's background colour."""
    try:
        from PIL import Image

        with Image.open(path) as im:
            data = im.convert("RGB").resize((192, 108)).tobytes()
    except Exception:
        return None
    r0, g0, b0 = DESKTOP_RGB
    px = list(zip(data[0::3], data[1::3], data[2::3], strict=True))
    near = sum(1 for r, g, b in px if abs(r - r0) < 20 and abs(g - g0) < 20 and abs(b - b0) < 25)
    return near / len(px)


def stortest_image(path: Path) -> None:
    """A 32 MiB FAT volume labelled VIBEUSB holding MARKER.TXT, with no partition
    table: the Pico serves its image as a volume at offset 0."""
    with open(path, "wb") as f:
        f.truncate(32 << 20)
    marker = path.with_name("MARKER.TXT")
    marker.write_bytes(STORTEST_MARKER)
    for cmd in (
        ["mkfs.fat", "-n", STORTEST_LABEL, str(path)],
        ["mcopy", "-i", str(path), str(marker), "::MARKER.TXT"],
    ):
        subprocess.run(cmd, check=True, capture_output=True)


def stortest_verdict(text: str) -> tuple[str | None, list[str]]:
    """("ok" | "fail" | None while still running, this boot's [STORTEST] lines)."""
    lines = [ln.strip() for ln in text.splitlines() if "[STORTEST]" in ln]
    if any("[STORTEST] ALL OK" in ln for ln in lines):
        return "ok", lines
    if any("[STORTEST] FAILURES" in ln or "no VIBEUSB volume" in ln for ln in lines):
        return "fail", lines
    return None, lines


def stortest_readback(image: Path) -> bytes | None:
    """WRITETST.TXT as it landed on the served image: the medium's own account of the
    write, read on the host rather than by the guest that did it."""
    cp = subprocess.run(
        ["mtype", "-i", str(image), "::WRITETST.TXT"], capture_output=True, timeout=30
    )
    return cp.stdout if cp.returncode == 0 else None


def doctor_stages(repo: Path) -> list[Stage]:
    stages: list[Stage] = []
    if not plug_configured(repo):
        stages.append(
            Stage(
                "plug",
                status="warn",
                detail="no plug named",
                salient=[
                    "name the Pi 5's Shelly plug in tools/pi5/env.sh (or PI5_PLUG);"
                    " until then a hung board needs hands"
                ],
            )
        )
    else:
        rc, out = pi4._quick([*_script(repo, "power.sh"), "status"])
        power = pi4.parse_power(out)
        if rc or not power:
            stages.append(Stage("plug", status="fail", detail="unreachable", salient=[out[-160:]]))
        else:
            stages.append(
                Stage("plug", detail=f"{'on' if power['on'] else 'off'} {power['watts']:.1f} W")
            )

    rc, out = pi4._quick([*_script(repo, "pico.sh"), "usb"])
    usb = pi4.parse_usb(out)
    host = os.environ.get("PI5_PICO_HOST", "picousb")
    if "error" in usb:
        stages.append(
            Stage(
                "pico",
                status="warn",
                detail=str(usb["error"]),
                salient=[f"{host} is not answering"],
            )
        )
    else:
        stages.append(Stage("pico", detail=f"{host} mounted={usb.get('mounted')}"))

    state = pi4.listener_state(bench_dir())
    if pi4.serial_file(state) is None or not pi4._pid_runs(state.get("pid"), "pico_serial.py"):
        stages.append(
            Stage(
                "serial",
                status="warn",
                detail="no listener",
                salient=["tools/pi5/serial.sh up (verdicts fall back to HDMI without it)"],
            )
        )
    else:
        stages.append(Stage("serial", detail=str(state.get("directory", ""))))

    if ssh_up(repo):
        stages.append(Stage("stager", detail="Linux up, SSH ok"))
    else:
        stages.append(
            Stage(
                "stager",
                status="warn",
                detail="no SSH",
                salient=["the board is off, booting, or in VibeOS; `pi5 boot` power-cycles it"],
            )
        )

    rc, _ = pi4._quick(
        [*_script(repo, "hdmi.sh"), str(bench_dir() / "hdmi-doctor.png")], timeout=60
    )
    stages.append(
        Stage("hdmi", detail="frame ok")
        if rc == 0
        else Stage(
            "hdmi",
            status="fail",
            detail="no frame",
            salient=["is the Camera app holding the card?"],
        )
    )
    return stages


# --- commands -----------------------------------------------------------------


def cmd_status(args, repo: Path) -> int:
    print(f"lease  : {pi4lease.describe(pi4lease.current())}")
    if plug_configured(repo):
        rc, out = pi4._quick([*_script(repo, "power.sh"), "status"])
        print(f"power  : {out if rc == 0 else 'unreachable'}")
    else:
        print("power  : no plug named (tools/pi5/env.sh)")
    rc, out = pi4._quick([*_script(repo, "pico.sh"), "usb"])
    print(f"pico   : {out.splitlines()[-1] if out else 'no answer'}")
    state = pi4.listener_state(bench_dir())
    print(f"serial : {state.get('status', 'none')} {state.get('directory', '')}")
    print(f"stager : {'up' if ssh_up(repo) else 'no SSH'}")
    m = payload_manifest()
    print(f"payload: {m.get('commit', 'none')} staged {m.get('staged', '-')}")
    return 0


def cmd_lease(args, repo: Path) -> int:
    return pi4.cmd_lease(args, repo)


def cmd_doctor(args, repo: Path) -> int:
    run = Run("pi5-doctor", "arm64")
    for stage in doctor_stages(repo):
        run.add(stage)
    if run.verdict == "FAIL":
        run.classification = "BENCH_UNHEALTHY"
    return run.finalize(repo)


def cmd_boot(args, repo: Path) -> int:
    run = Run("pi5-boot", "arm64")
    owner = pi4lease.owner_of(repo)
    held_before = pi4lease.current().get("owner") == owner
    ok, lease = pi4lease.take(owner, max(30, args.timeout / 60 + 20), "vibeos-cli pi5 boot")
    if not ok:
        run.classification = "BENCH_BUSY"
        run.add(Stage("lease", status="fail", detail="busy", salient=[pi4lease.describe(lease)]))
        return run.finalize(repo)
    run.add(Stage("lease", detail="held" if held_before else "taken"))
    try:
        return _boot(run, args, repo)
    finally:
        if not held_before:
            pi4lease.drop(owner)


def _wait_ssh(repo: Path, timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if ssh_up(repo):
            return True
        time.sleep(5)
    return False


def _boot(run: Run, args, repo: Path) -> int:
    st = provenance.check_staleness(repo, "arm64")
    if st.stale and not args.allow_stale:
        run.classification = "STALE_MEDIA"
        run.add(
            Stage(
                "staleness",
                status="fail",
                detail="stale",
                salient=[*st.reasons, "run `vibeos-cli build --arch arm64` (or --allow-stale)"],
            )
        )
        return run.finalize(repo)
    run.add(
        Stage(
            "staleness",
            status="warn" if st.stale or st.warm else "ok",
            detail="WARM build" if st.warm else ("stale" if st.stale else "current"),
        )
    )

    # The stager is whatever a plain reset boots, so a board left in VibeOS (or
    # hung) is one power cycle from it.
    if not ssh_up(repo):
        if not plug_configured(repo):
            run.classification = "BENCH_UNHEALTHY"
            run.add(
                Stage(
                    "stager",
                    status="fail",
                    detail="no SSH, no plug named",
                    salient=["power-cycle the Pi 5 by hand, or set PI5_PLUG"],
                )
            )
            return run.finalize(repo)
        run.run(
            [*_script(repo, "power.sh"), "cycle", "5"], cwd=repo, stage="power-cycle", timeout=60
        )
        if not _wait_ssh(repo, SSH_WAIT):
            run.classification = "BENCH_UNHEALTHY"
            run.add(Stage("stager", status="fail", detail=f"no SSH {SSH_WAIT}s after power-on"))
            pi4._screenshot(run, repo, hdmi=_script(repo, "hdmi.sh"))
            return run.finalize(repo)
    run.add(Stage("stager", detail="Linux up"))

    state = pi4.listener_state(bench_dir())
    serial = pi4.serial_file(state) if pi4._pid_runs(state.get("pid"), "pico_serial.py") else None
    start = serial.stat().st_size if serial and serial.exists() else 0

    options = " ".join([*args.options, *(["/STORTEST"] if args.stortest else [])])
    image = None
    if args.stortest:
        # Its grade is read off serial; an HDMI-only boot cannot give one.
        if serial is None:
            run.classification = "BENCH_UNHEALTHY"
            run.add(
                Stage(
                    "stortest",
                    status="fail",
                    detail="no serial console",
                    salient=["tools/pi5/serial.sh up"],
                )
            )
            return run.finalize(repo)
        image = run.dir / "vibeusb.img"
        stortest_image(image)
    env = {**os.environ, "VIBEOS_LOADER_OPTIONS": options} if options else None
    cp = run.run(_script(repo, "stage.sh"), cwd=repo, env=env, stage="stage", timeout=600)
    manifest = payload_manifest()
    head = subprocess.run(
        ["git", "rev-parse", "--short=12", "HEAD"], cwd=repo, capture_output=True, text=True
    ).stdout.strip()
    if cp.returncode or manifest.get("commit") != head:
        run.classification = "STAGE_FAIL"
        run.add(
            Stage(
                "stage",
                status="fail",
                detail=f"exit {cp.returncode}",
                salient=(cp.stderr or cp.stdout).strip().splitlines()[-3:],
            )
        )
        return run.finalize(repo)
    run.extra["payload"] = manifest
    run.add(Stage("stage", detail=f"{head} (tryboot){' ' + options if options else ''}"))

    if image is None:
        return _await(run, args, repo, serial, start)
    # The Pico must hold the medium from before the reboot until stortest is done;
    # it drops it the moment the server exits.
    log = (run.dir / "serve-image.log").open("w")
    server = subprocess.Popen(
        [*_script(repo, "pico.sh"), "serve-image", str(image), "--writable"],
        cwd=repo,
        stdout=log,
        stderr=subprocess.STDOUT,
    )
    try:
        time.sleep(5)
        if server.poll() is not None:
            run.classification = "BENCH_UNHEALTHY"
            run.add(
                Stage(
                    "serve-image",
                    status="fail",
                    detail=f"exit {server.returncode}",
                    salient=[f"see {run.dir / 'serve-image.log'}"],
                )
            )
            return run.finalize(repo)
        run.add(Stage("serve-image", detail=f"{STORTEST_LABEL} writable"))
        rc = _await(run, args, repo, serial, start, stortest=True)
    finally:
        server.terminate()
        try:
            server.wait(timeout=30)
        except subprocess.TimeoutExpired:
            server.kill()
        log.close()
    if not run.classification:
        wrote = stortest_readback(image)
        if wrote != STORTEST_WROTE:
            run.classification = "STORTEST_FAIL"
            run.add(
                Stage(
                    "readback",
                    status="fail",
                    detail="WRITETST.TXT missing" if wrote is None else "WRITETST.TXT differs",
                    salient=[repr(wrote)[:120]],
                )
            )
            return run.finalize(repo)
        run.add(Stage("readback", detail="WRITETST.TXT on the image matches"))
        return run.finalize(repo)
    return rc


def _await(
    run: Run, args, repo: Path, serial: Path | None, start: int, stortest: bool = False
) -> int:

    if serial is not None:
        run.extra["live_serial"] = str(serial)
        text, reached = pi4.wait_for_desktop(serial, start, args.timeout)
        pi4._save_serial(run, text)
        if reached:
            run.add(Stage("desktop", detail=pi4.MARKER))
            if stortest:
                _await_stortest(run, serial, start)
        else:
            verdict = triage.classify(text, reached=False, fail_marker=None, debugcon=None)
            run.classification = verdict["classification"]
            salient = list(verdict.get("salient", []))
            if verdict["classification"] in ("BUGCHECK", "KNOWN_FLAKE"):
                sym = triage.symbolicate_pc(repo, "arm64", text)
                if sym:
                    salient.append(sym)
            run.add(
                Stage("desktop", status="fail", detail=verdict["classification"], salient=salient)
            )
        pi4._screenshot(run, repo, hdmi=_script(repo, "hdmi.sh"))
        return 0 if stortest and not run.classification else run.finalize(repo)

    # No serial console: HDMI is the only witness. Weaker than the marker (it sees
    # a desktop, not that the shell finished), so it can pass only as a warning.
    # Every frame is kept (hdmi-000.png, ...): without serial, the sequence - did
    # the firmware hand over, did UEFI draw its logo, did the loader list files -
    # is the only record of how far a failed boot got.
    shot = run.dir / "hdmi.png"
    frames = run.dir / "hdmi-frames"
    frames.mkdir(exist_ok=True)
    deadline = time.monotonic() + args.timeout
    share = None
    n = 0
    while time.monotonic() < deadline:
        time.sleep(10)
        frame = frames / f"hdmi-{n:03d}.png"
        rc, _ = pi4._quick([*_script(repo, "hdmi.sh"), str(frame)], timeout=60)
        if rc == 0:
            n += 1
            shutil.copyfile(frame, shot)
            run.extra["screenshot"] = str(shot)
            share = desktop_share(shot)
            if share is not None and share >= DESKTOP_SHARE:
                run.add(
                    Stage(
                        "desktop",
                        status="warn",
                        detail=f"seen on HDMI ({share:.0%} desktop)",
                        salient=["no serial console: HDMI verdict only (tools/pi5/serial.sh up)"],
                    )
                )
                return run.finalize(repo)
    run.classification = "HANG_TIMEOUT"
    run.add(
        Stage(
            "desktop",
            status="fail",
            detail="no desktop on HDMI",
            salient=[f"last frame {share:.0%} desktop" if share is not None else "no frame"],
        )
    )
    return run.finalize(repo)


def _await_stortest(run: Run, serial: Path, start: int) -> None:
    deadline = time.monotonic() + STORTEST_WAIT
    while True:
        try:
            with open(serial, "rb") as f:
                f.seek(start)
                text = f.read().decode("utf-8", errors="replace")
        except OSError:
            text = ""
        verdict, lines = stortest_verdict(text)
        if verdict is not None or time.monotonic() > deadline:
            break
        time.sleep(5)
    pi4._save_serial(run, text)
    run.extra["stortest"] = lines
    if verdict == "ok":
        run.add(Stage("stortest", detail=lines[-1]))
        return
    run.classification = "STORTEST_FAIL" if verdict else "HANG_TIMEOUT"
    run.add(
        Stage(
            "stortest",
            status="fail",
            detail="failed" if verdict else f"no verdict {STORTEST_WAIT}s after the desktop",
            salient=lines[-6:] or ["no [STORTEST] line: did winlogon start it?"],
        )
    )


def add_parser(sub) -> None:
    p = sub.add_parser("pi5", help="Raspberry Pi 5 bench: lease, doctor, boot this tree")
    p.set_defaults(fn=command)
    verbs = p.add_subparsers(dest="pi5_cmd", required=True)
    verbs.add_parser("status", help="lease, power, Pico, serial, Linux stager, staged payload")
    lease = verbs.add_parser("lease", help="hold the board across several commands")
    lease.add_argument("action", choices=["take", "drop", "show"])
    lease.add_argument("--minutes", type=float, default=pi4lease.DEFAULT_MINUTES)
    lease.add_argument("--why", help="shown to other lanes while you hold it")
    lease.add_argument("--force", action="store_true", help="drop: release another lane's lease")
    verbs.add_parser("doctor", help="preflight the bench, with the fix for each problem")
    boot = verbs.add_parser(
        "boot", help="stage this tree's arm64 build over SSH, tryboot it; verdict"
    )
    boot.add_argument(
        "--timeout",
        type=float,
        default=pi4.DEFAULT_DESKTOP_TIMEOUT,
        help="seconds from the tryboot reboot to the desktop",
    )
    boot.add_argument("--allow-stale", action="store_true")
    boot.add_argument(
        "--options",
        nargs="+",
        default=[],
        metavar="OPT",
        help="loader options for this boot only, e.g. /PROCTRACE (pi5-bench.md)",
    )
    boot.add_argument(
        "--stortest",
        action="store_true",
        help="serve a VIBEUSB image from the Pico, boot with /STORTEST, grade stortest.exe "
        "on serial and the written file on the image",
    )


def command(args, repo: Path) -> int:
    # Everything below, and every script it runs, is the Pi 5's: lease included.
    os.environ["VIBEOS_BENCH_BOARD"] = BOARD
    os.environ.setdefault("PI5_BENCH", str(bench_dir()))
    return {
        "status": cmd_status,
        "lease": cmd_lease,
        "doctor": cmd_doctor,
        "boot": cmd_boot,
    }[args.pi5_cmd](args, repo)
