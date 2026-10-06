"""`vibeos-cli pi4` - the Raspberry Pi 4B bench as a tool any lane can use.

    vibeos-cli pi4 status                 # lease, power, Pico, serial listener
    vibeos-cli pi4 lease take|drop|show   # hold the board across several commands
    vibeos-cli pi4 doctor                 # preflight every bench service, with the fix
    vibeos-cli pi4 boot [--kernel up|mp]  # THIS tree's arm64 build -> desktop verdict

`boot` is smoke for real hardware. It takes the bench lease, runs doctor,
refuses a stale build the way smoke does, stages this tree's arm64 media,
netboots it (tools/pi4/netboot.sh: power cycle, UEFI Shell over serial, iPXE
sanboot), then waits for the same `shell ready` marker smoke requires. It
classifies with smoke's own triage and saves this boot's slice of the serial log
plus an HDMI screenshot. The bench scripts themselves are documented in
docs/engineering/pi4-bench.md; this module only orchestrates them.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import time
from pathlib import Path

from . import ladder, pi4lease, provenance, triage
from .artifacts import Run, Stage

MARKER = "shell ready"
DARK = 12  # mean 0-255 luma below which an HDMI frame counts as black
DEFAULT_DESKTOP_TIMEOUT = 300


def bench_dir() -> Path:
    return Path(os.environ.get("PI4_BENCH", "/mnt/d/home/vibeos-rpi4-setup"))


def _script(repo: Path, name: str) -> list[str]:
    return ["bash", str(repo / "tools" / "pi4" / name)]


def _quick(cmd: list[str], timeout: float = 60) -> tuple[int, str]:
    try:
        cp = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return 124, "timed out"
    except OSError as error:
        return 127, str(error)
    return cp.returncode, (cp.stdout + cp.stderr).strip()


def parse_usb(text: str) -> dict:
    """`pico.sh usb` prints `uptime=623s mounted=True suspended=False ...` or a JSON error."""
    line = text.strip().splitlines()[-1] if text.strip() else ""
    if line.startswith("{"):
        try:
            return {"error": json.loads(line).get("error", "unknown")}
        except ValueError:
            return {"error": line}
    out: dict = {}
    for key, value in re.findall(r"(\w+)=(\[[^\]]*\]|\S+)", line):
        out[key] = {"True": True, "False": False}.get(value, value)
    return out if "mounted" in out else {"error": line or "no output"}


def parse_power(text: str) -> dict:
    m = re.search(r"\b(on|off)\s+([\d.]+)\s*W", text)
    return {"on": m.group(1) == "on", "watts": float(m.group(2))} if m else {}


def listener_state(bench: Path) -> dict:
    try:
        return json.loads((bench / "serial-current.json").read_text(encoding="utf-8-sig"))
    except (OSError, ValueError):
        return {}


def _pid_runs(pid: object, needle: str) -> bool:
    try:
        return needle in Path(f"/proc/{int(str(pid))}/cmdline").read_bytes().decode(
            errors="replace"
        )
    except (OSError, TypeError, ValueError):
        return False


def serial_file(state: dict) -> Path | None:
    d = state.get("directory")
    if not d:
        return None
    if not d.startswith("/"):
        d = d.replace("D:\\home", "/mnt/d/home").replace("\\", "/")
    return Path(d) / "serial.bin"


# --- doctor -------------------------------------------------------------------


def doctor_stages(repo: Path, run_dir: Path | None = None) -> list[Stage]:
    """One stage per bench service. `fail` blocks a boot; `warn` does not."""
    stages: list[Stage] = []
    bench = bench_dir()

    rc, out = _quick([*_script(repo, "power.sh"), "status"])
    power = parse_power(out)
    if rc or not power:
        stages.append(
            Stage(
                "plug",
                status="fail",
                detail="unreachable",
                salient=[out[-160:], "check PI4_PLUG / the Shelly plug on the LAN"],
            )
        )
    else:
        state = "on" if power["on"] else "off"
        stages.append(Stage("plug", detail=f"{state} {power['watts']:.1f} W"))

    rc, out = _quick([*_script(repo, "pico.sh"), "usb"])
    usb = parse_usb(out)
    if "error" in usb:
        stages.append(
            Stage(
                "pico",
                status="fail",
                detail=str(usb["error"]),
                salient=[
                    "the Pico 2 W is not answering on "
                    f"{os.environ.get('PI4_PICO_HOST', 'picousb5')}:8080",
                    "green LED off = no Wi-Fi; its own log is on GP4 (picousb docs/protocol.md)",
                ],
            )
        )
    elif "target_powered" in usb and power and usb["target_powered"] != power["on"]:
        # The Pico sees VBUS from whatever its USB is plugged into. If that does
        # not follow the Pi's plug, it is plugged into another host, and HID
        # input would land THERE.
        stages.append(
            Stage(
                "pico",
                status="fail",
                detail="attached elsewhere",
                salient=[
                    f"Pico VBUS={usb['target_powered']} but the Pi's plug is "
                    f"{'on' if power['on'] else 'off'}: its USB is on another host",
                    "plug the Pico's USB into the Pi before sending input",
                ],
            )
        )
    elif power and not power["on"] and usb.get("mounted") and not usb.get("suspended"):
        # The Pi is off, yet something is driving the Pico's USB: it is plugged
        # into another host, and HID input would land THERE.
        stages.append(
            Stage(
                "pico",
                status="fail",
                detail="attached elsewhere",
                salient=[
                    "USB is configured while the Pi is powered off: the Pico is on another host",
                    "plug the Pico's USB into the Pi before sending input",
                ],
            )
        )
    elif power.get("on") and not usb.get("mounted"):
        stages.append(
            Stage(
                "pico",
                status="warn",
                detail="not enumerated",
                salient=["the Pi is on but has not configured the Pico (early boot?)"],
            )
        )
    else:
        stages.append(Stage("pico", detail=f"mounted={usb.get('mounted')} up {usb.get('uptime')}"))

    state = listener_state(bench)
    serial_mode = os.environ.get("PI4_SERIAL", "pico")
    if state.get("status") != "LISTENING":
        stages.append(
            Stage(
                "serial",
                status="fail",
                detail=str(state.get("status", "no listener")),
                salient=["start it: tools/pi4/bench.sh up"],
            )
        )
    elif serial_mode == "pico" and not _pid_runs(state.get("pid"), "pico_serial.py"):
        stages.append(
            Stage(
                "serial",
                status="fail",
                detail="listener exited",
                salient=[
                    f"serial-current.json names pid {state.get('pid')}, which is gone",
                    "restart it: tools/pi4/bench.sh up",
                ],
            )
        )
    elif serial_mode == "pico" and not state.get("connected"):
        stages.append(
            Stage(
                "serial",
                status="fail",
                detail="not connected to the Pico",
                salient=[f"see {bench / 'pico-serial.log'}"],
            )
        )
    else:
        stages.append(Stage("serial", detail=f"{serial_mode} listening"))

    rc, out = _quick([*_script(repo, "bench.sh"), "status"], timeout=90)
    for service, line_key, fix in (
        ("tftp", "tftp server", "tools/pi4/bench.sh up"),
        ("http", "http server", "tools/pi4/bench.sh http"),
    ):
        m = re.search(rf"^{line_key}\s*:\s*(.*)$", out, re.M)
        value = m.group(1).strip() if m else ""
        if not value or value == "down":
            stages.append(
                Stage(service, status="fail", detail="down", salient=[f"start it: {fix}"])
            )
        else:
            stages.append(Stage(service, detail=f"pid {value}"))

    if run_dir is not None:
        shot = run_dir / "hdmi-doctor.png"
        rc, out = 1, ""
        for _ in range(3):
            rc, out = _quick([*_script(repo, "hdmi.sh"), str(shot)], timeout=60)
            if rc == 0 and shot.is_file():
                break
            time.sleep(2)
        if rc == 0 and shot.is_file():
            stages.append(Stage("hdmi", detail="frame captured"))
        else:
            stages.append(
                Stage(
                    "hdmi",
                    status="warn",
                    detail="no frame",
                    salient=["capture card busy? close the Windows Camera app", out[-160:]],
                )
            )

    manifest = payload_manifest(bench)
    head = _head(repo)
    if not manifest:
        stages.append(Stage("payload", status="warn", detail="none staged"))
    elif manifest.get("source_head") != head or manifest.get("source_dirty"):
        stages.append(
            Stage(
                "payload",
                status="warn",
                detail=f"staged from {str(manifest.get('source_head'))[:12]}",
                salient=["not this tree: `vibeos-cli pi4 boot` restages it"],
            )
        )
    else:
        stages.append(Stage("payload", detail="this tree"))
    return stages


def payload_manifest(bench: Path) -> dict:
    try:
        return json.loads((bench / "payload" / "manifest.json").read_text())
    except (OSError, ValueError):
        return {}


def _head(repo: Path) -> str:
    return subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=repo, capture_output=True, text=True
    ).stdout.strip()


# --- commands -----------------------------------------------------------------


def cmd_status(args, repo: Path) -> int:
    print(f"lease  : {pi4lease.describe(pi4lease.current())}")
    rc, out = _quick([*_script(repo, "power.sh"), "status"])
    print(f"power  : {out if rc == 0 else 'unreachable'}")
    rc, out = _quick([*_script(repo, "pico.sh"), "usb"])
    print(f"pico   : {out.splitlines()[-1] if out else 'no answer'}")
    state = listener_state(bench_dir())
    print(
        f"serial : {state.get('status', 'none')} connected={state.get('connected')} "
        f"{state.get('directory', '')}"
    )
    manifest = payload_manifest(bench_dir())
    print(
        f"payload: {str(manifest.get('source_head', 'none'))[:12]} "
        f"staged {manifest.get('staged', '-')}"
    )
    return 0


def cmd_lease(args, repo: Path) -> int:
    owner = pi4lease.owner_of(repo)
    if args.action == "show":
        print(pi4lease.describe(pi4lease.current()))
        return 0
    if args.action == "take":
        ok, lease = pi4lease.take(owner, args.minutes, args.why or "")
    else:
        ok, lease = pi4lease.drop(owner, force=args.force)
    if not ok:
        print(f"refused: the bench is {pi4lease.describe(lease)}")
        if args.action == "drop":
            print("  --force releases another lane's lease; only do that for a dead lane")
        return pi4lease.REFUSED
    print("released" if args.action == "drop" else pi4lease.describe(lease))
    return 0


def cmd_doctor(args, repo: Path) -> int:
    run = Run("pi4-doctor", "arm64")
    for stage in doctor_stages(repo, run.dir):
        run.add(stage)
    if run.verdict == "FAIL":
        run.classification = "BENCH_UNHEALTHY"
    return run.finalize(repo)


def _env(repo: Path, kernel: str) -> dict:
    env = dict(os.environ, PI4_KERNEL=kernel)
    env.setdefault("PI4_BENCH", str(bench_dir()))
    return env


def wait_for_desktop(serial: Path, start: int, timeout: float) -> tuple[str, bool]:
    """This boot's serial text and whether the marker arrived. Stops early on a
    bugcheck or a Rust panic: those never recover into a desktop."""
    deadline = time.monotonic() + timeout
    text = ""
    while True:
        try:
            with open(serial, "rb") as f:
                f.seek(start)
                text = f.read().decode("utf-8", errors="replace")
        except OSError:
            text = ""
        if MARKER in text:
            return text, True
        if "*** BUGCHECK" in text or "panicked at" in text:
            time.sleep(3)  # let the rest of the crash record arrive
            with open(serial, "rb") as f:
                f.seek(start)
                return f.read().decode("utf-8", errors="replace"), False
        if time.monotonic() > deadline:
            return text, False
        time.sleep(2)


class SerialTail:
    """The bench capture file, read the way `ladder.watch` reads a VM's serial
    (`vm.serial.read_available`), so the board's ladder is drained and graded
    by the SAME code as smoke's instead of a second copy of its rules.

    It also timestamps each guest's launch and verdict line as it arrives. The
    capture is raw bytes with no clock, so arrival time is the only timing the
    bench can give: good to the poll interval, which is plenty against guests
    that run for seconds."""

    def __init__(self, path: Path, offset: int, t0: float) -> None:
        self.path, self.pos, self.t0 = path, offset, t0
        self.times: dict[str, float] = {}
        self._tail = ""

    @property
    def serial(self) -> SerialTail:
        return self

    def read_available(self, timeout: float = 0.5) -> str:
        time.sleep(timeout)
        try:
            with open(self.path, "rb") as f:
                f.seek(self.pos)
                raw = f.read()
        except OSError:
            return ""
        self.pos += len(raw)
        text = raw.decode("utf-8", errors="replace")
        self._stamp(text)
        return text

    def _stamp(self, text: str) -> None:
        # Search the previous chunk's tail too: a line can straddle two reads.
        window = self._tail + text
        now = round(time.monotonic() - self.t0, 1)
        for g in (*ladder.LADDER, *ladder.OPTIONAL):
            for event, needle in (("launch", f"{g}.exe ("), ("exit", f"{g}.exe exit = ")):
                key = f"{g} {event}"
                if key not in self.times and needle in window:
                    self.times[key] = now
        if "program_done" not in self.times and ladder.PROGRAM_DONE in window:
            self.times["program_done"] = now
        self._tail = text[-200:]

    def durations(self) -> dict[str, float]:
        """Seconds from each guest's launch line to its verdict line."""
        out = {}
        for g in (*ladder.LADDER, *ladder.OPTIONAL):
            a, b = self.times.get(f"{g} launch"), self.times.get(f"{g} exit")
            if a is not None and b is not None:
                out[g] = round(b - a, 1)
        return out


def cmd_boot(args, repo: Path) -> int:
    run = Run("pi4-boot", "arm64")
    owner = pi4lease.owner_of(repo)
    held_before = pi4lease.current().get("owner") == owner
    # --ladder adds the drain: its 420s ceiling plus an extension per optional guest.
    minutes = max(30, args.timeout / 60 + 20) + (30 if getattr(args, "ladder", False) else 0)
    ok, lease = pi4lease.take(owner, minutes, "vibeos-cli pi4 boot")
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


def _boot(run: Run, args, repo: Path) -> int:
    if not args.skip_doctor:
        for stage in doctor_stages(repo):
            if stage.name == "payload":
                continue  # restaged below
            stage.name = f"doctor:{stage.name}"
            run.add(stage)
        if run.verdict == "FAIL":
            run.classification = "BENCH_UNHEALTHY"
            return run.finalize(repo)

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

    env = _env(repo, args.kernel)
    cp = run.run(
        [*_script(repo, "bench.sh"), "stage"], cwd=repo, env=env, stage="stage", timeout=300
    )
    manifest = payload_manifest(bench_dir())
    if cp.returncode or manifest.get("source_head") != _head(repo):
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
    run.add(Stage("stage", detail=f"{manifest['source_head'][:12]} kernel={args.kernel}"))

    serial = serial_file(listener_state(bench_dir()))
    if serial is None:
        run.classification = "BENCH_UNHEALTHY"
        run.add(Stage("serial", status="fail", detail="no capture file"))
        return run.finalize(repo)
    start = serial.stat().st_size if serial.exists() else 0
    run.extra["live_serial"] = str(serial)
    run.extra["serial_offset"] = start

    t_netboot = time.monotonic()
    cp = run.run(_script(repo, "netboot.sh"), cwd=repo, env=env, stage="netboot", timeout=600)
    lines = (cp.stdout + cp.stderr).strip().splitlines()
    if cp.returncode:
        text = _slice(serial, start)
        _save_serial(run, text)
        run.classification = "NETBOOT_FAIL"
        run.add(Stage("netboot", status="fail", detail=f"exit {cp.returncode}", salient=lines[-3:]))
        _screenshot(run, repo)
        return run.finalize(repo)
    run.add(Stage("netboot", detail=lines[-1][:80] if lines else ""))

    text, reached = wait_for_desktop(serial, start, args.timeout)
    _save_serial(run, text)
    verdict = triage.classify(text, reached=reached, fail_marker=None, debugcon=None)
    if reached:
        run.extra["desktop_s"] = round(time.monotonic() - t_netboot, 1)
        run.add(Stage("desktop", detail=MARKER))
        if getattr(args, "ladder", False):
            text = _ladder(run, repo, serial, start, text)
            _save_serial(run, text)
    else:
        run.classification = verdict["classification"]
        salient = list(verdict.get("salient", []))
        if verdict["classification"] in ("BUGCHECK", "KNOWN_FLAKE"):
            sym = triage.symbolicate_pc(repo, "arm64", text)
            if sym:
                salient.append(sym)
        run.add(Stage("desktop", status="fail", detail=verdict["classification"], salient=salient))
    _screenshot(run, repo)
    return run.finalize(repo)


def _ladder(run: Run, repo: Path, serial: Path, start: int, text: str) -> str:
    """Drain and grade the WoW64 ladder after the desktop, as `smoke --ladder`
    does. Returns the boot's whole serial text, ladder included."""
    t0 = time.monotonic()
    tail = SerialTail(serial, start, t0)
    seen = tail.read_available(0)  # everything since this boot's offset, so far
    staged = ladder.media_optional_staged(repo, "arm64")
    # program_done_expected: staleness was checked before staging, so the
    # board runs this tree's winlogon, which prints PROGRAM_DONE.
    text = seen + ladder.watch(tail, seen=seen, optional_staged=staged, program_done_expected=True)
    run.extra["ladder_drain_s"] = round(time.monotonic() - t0, 1)
    run.extra["ladder_times"] = tail.times
    run.extra["ladder_durations"] = tail.durations()
    run.extra["ladder_verdicts"] = ladder.verdicts(text)
    lad = ladder.check(text, optional_staged=staged)
    if lad:
        run.classification = lad["classification"]
        run.add(
            Stage("ladder", status="fail", detail=lad["classification"], salient=lad["salient"])
        )
    else:
        run.add(Stage("ladder", detail=ladder.summary(text)))
    return text


def _slice(serial: Path, start: int) -> str:
    try:
        with open(serial, "rb") as f:
            f.seek(start)
            return f.read().decode("utf-8", errors="replace")
    except OSError:
        return ""


def _save_serial(run: Run, text: str) -> None:
    path = run.dir / "serial.log"
    path.write_text(text)
    run.extra["serial_log"] = str(path)


def _brightness(path: Path) -> float | None:
    try:
        from PIL import Image, ImageStat  # the harness venv has Pillow; smoke uses it too

        with Image.open(path) as im:
            return float(ImageStat.Stat(im.convert("L")).mean[0])
    except Exception:
        return None


def _screenshot(run: Run, repo: Path, settle: float = 30, hdmi: list[str] | None = None) -> None:
    """The Pi's HDMI lags `shell ready` by a few seconds (measured 2026-10-03: the
    frame AT the marker was black, the desktop was up seconds later), so keep
    grabbing until the frame is not black, and say so if it never lights up."""
    shot = run.dir / "hdmi.png"
    deadline = time.monotonic() + settle
    level = None
    while True:
        rc, _out = _quick([*(hdmi or _script(repo, "hdmi.sh")), str(shot)], timeout=60)
        if rc == 0 and shot.is_file():
            run.extra["screenshot"] = str(shot)
            level = _brightness(shot)
            if level is None or level > DARK:
                return
        if time.monotonic() > deadline:
            break
        time.sleep(3)
    if "screenshot" not in run.extra:
        run.add(Stage("screenshot", status="warn", detail="no frame (capture card busy?)"))
    else:
        run.add(
            Stage(
                "screenshot",
                status="warn",
                detail=f"screen still dark after {settle:.0f}s",
                salient=["HDMI output never lit up after the marker: check the display path"],
            )
        )


def add_parser(sub) -> None:
    p = sub.add_parser("pi4", help="Raspberry Pi 4B bench: lease, doctor, boot this tree")
    p.set_defaults(fn=command)
    verbs = p.add_subparsers(dest="pi4_cmd", required=True)
    verbs.add_parser("status", help="lease, power, Pico, serial listener, staged payload")
    lease = verbs.add_parser("lease", help="hold the board across several commands")
    lease.add_argument("action", choices=["take", "drop", "show"])
    lease.add_argument("--minutes", type=float, default=pi4lease.DEFAULT_MINUTES)
    lease.add_argument("--why", help="shown to other lanes while you hold it")
    lease.add_argument("--force", action="store_true", help="drop: release another lane's lease")
    verbs.add_parser("doctor", help="preflight every bench service, with the fix for each")
    boot = verbs.add_parser("boot", help="stage + netboot this tree's arm64 build; desktop verdict")
    boot.add_argument("--kernel", choices=["mp", "up"], default="mp")
    boot.add_argument(
        "--timeout",
        type=float,
        default=DEFAULT_DESKTOP_TIMEOUT,
        help="seconds from loader start to `shell ready`",
    )
    boot.add_argument(
        "--ladder",
        action="store_true",
        help="after the desktop, drain and grade the WoW64 ladder as `smoke --ladder` does "
        "(needs VIBEOS_FEX=1 media); records per-guest timings in summary.json",
    )
    boot.add_argument("--allow-stale", action="store_true")
    boot.add_argument("--skip-doctor", action="store_true")


def command(args, repo: Path) -> int:
    return {
        "status": cmd_status,
        "lease": cmd_lease,
        "doctor": cmd_doctor,
        "boot": cmd_boot,
    }[args.pi4_cmd](args, repo)
