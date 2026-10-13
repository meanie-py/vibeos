"""`vibeos-cli winref`: a real Windows 11 VM as the conformance ORACLE.

ReactOS's suites encode what someone believed Server 2003 did. For the Windows 10/11
surface (docs/engineering/modern-windows-compat-roadmap.md, Rung 0) the expected result
has to come from Windows itself: build a test with our toolchain, run it here, and what
real Windows does is the answer the guest is graded against.

    winref install --iso <Win11.iso>   one unattended install into a base image
    winref run <payload-dir>           boot a throwaway overlay, run <payload-dir>/run.cmd
    winref status                      what is installed, and which build

The VM runs on the WINDOWS host's QEMU under WHPX (the `windows-host` backend's QEMU),
not on KVM inside WSL, which is nested virtualization. It has no network device: no
updates, no account, no drift between runs. Every run boots a qcow2 overlay of the base
image and discards it, so each test starts from the same installed Windows.

Guest protocol, all on COM1 (captured here over a socket, as vm.py does):
    WINREF-SETUP: ...        install-time steps, ending `done build=...`
    WINREF: ready build=...  a run booted
    OUT: <line>              the payload's output, one line per line
    WINREF: exit=<n>         the payload's exit code
The console log is `guest-console.log`, deliberately not `*serial*.log`: a reference-VM
failure is not a vibeos boot failure and must not reach the flake farm.
"""

from __future__ import annotations

import json
import os
import shutil
import socket
import subprocess
import tempfile
import threading
import time
from collections.abc import Callable
from pathlib import Path

from .. import backend as be
from ..qmp_client import QmpClient
from . import artifacts

REPO = Path(__file__).resolve().parents[3]
ASSETS = REPO / "qemu" / "winref"
WINREF_DIR = Path(os.environ.get("VIBEOS_WINREF_DIR", "/mnt/d/vibeos-winref"))
QEMU_DIR = Path(os.environ.get("VIBEOS_WINREF_QEMU_DIR", "/mnt/c/Program Files/qemu"))
OVMF_CODE = Path("/usr/share/OVMF/OVMF_CODE_4M.fd")  # non-secure: test signing needs SB off
OVMF_VARS = Path("/usr/share/OVMF/OVMF_VARS_4M.fd")
# Windows 11 24H2+ requires SSE4.2 and POPCNT; QEMU's default qemu64 model has neither.
# `qemu64,+ssse3,+sse4.1,+sse4.2,+popcnt` got through WinPE but died mid-Setup (the
# specialize/OOBE boot) with `WHPX: Unexpected VP exit code 4` on the AMD Ryzen 9 7950X3D
# host, twice; EPYC reaches the desktop (2026-10-11).
DEFAULT_CPU = "EPYC"
DEFAULT_IRQCHIP = "on"
# TCG by default: under WHPX on the AMD Ryzen 9 7950X3D host, six Windows 11 installs died
# with `WHPX: Unexpected VP exit code 4` (after `Ignoring request for interrupt vector 0`)
# or hung (kernel-irqchip=off), at a different Setup phase each time. QEMU's own interpreter
# never touches the hypervisor, the same reason the smokes use it.
DEFAULT_ACCEL = "tcg"
INSTALL_TIMEOUT = 3 * 3600.0
RUN_TIMEOUT = 1800.0
SHOT_INTERVAL = float(os.environ.get("VIBEOS_WINREF_SHOT", "30"))


def _paths() -> dict[str, Path]:
    return {
        "base": WINREF_DIR / "base.qcow2",
        "vars": WINREF_DIR / "base-vars.fd",
        "code": WINREF_DIR / "OVMF_CODE.fd",
        "state": WINREF_DIR / "state.json",
        "runs": WINREF_DIR / "runs",
        "snapshots": WINREF_DIR / "snapshots",
    }


def _qemu_img(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [str(QEMU_DIR / "qemu-img.exe"), *args], capture_output=True, text=True, timeout=600
    )


def _make_iso(src: Path, out: Path, label: str) -> None:
    proc = subprocess.run(
        ["genisoimage", "-quiet", "-J", "-R", "-V", label, "-o", str(out), str(src)],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"genisoimage failed: {proc.stderr.strip()}")


class _Console:
    """The guest's COM1, over a socket QEMU dials back to (vm.py's windows-host wiring:
    WSL listens on 0.0.0.0, the Windows QEMU connects to 127.0.0.1)."""

    def __init__(self, path: Path) -> None:
        self.path = path
        self._srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._srv.bind(("0.0.0.0", 0))
        self._srv.listen(1)
        self.port = self._srv.getsockname()[1]
        self.lines: list[str] = []
        self._thread = threading.Thread(target=self._pump, daemon=True)
        self._thread.start()

    def _pump(self) -> None:
        self._srv.settimeout(120)
        try:
            conn, _ = self._srv.accept()
        except OSError:
            return
        buf = b""
        with conn, self.path.open("ab") as fh:
            while True:
                try:
                    chunk = conn.recv(4096)
                except OSError:
                    return
                if not chunk:
                    return
                fh.write(chunk)
                fh.flush()
                buf += chunk
                while b"\n" in buf:
                    line, buf = buf.split(b"\n", 1)
                    self.lines.append(line.decode("utf-8", "replace").rstrip("\r"))

    def close(self) -> None:
        self._srv.close()
        self._thread.join(timeout=5)


def _launch(
    uid: str,
    disk: Path,
    vars_fd: Path,
    cds: list[str],
    console: _Console,
    qmp_port: int,
    *,
    cpus: int,
    mem: str,
    cpu: str,
    irqchip: str,
    accel: str,
    show: bool,
) -> subprocess.Popen:
    p = _paths()
    args = [
        str(QEMU_DIR / "qemu-system-x86_64.exe"),
        "-name",
        uid,
        "-machine",
        "q35",
        "-accel",
        f"whpx,kernel-irqchip={irqchip}" if accel == "whpx" else "tcg,thread=multi",
        "-cpu",
        cpu,
        "-smp",
        str(cpus),
        "-m",
        mem,
        "-drive",
        f"if=pflash,format=raw,readonly=on,file={be.win_path(p['code'])}",
        "-drive",
        f"if=pflash,format=raw,file={be.win_path(vars_fd)}",
        "-drive",
        f"file={be.win_path(disk)},if=none,id=hd0,format=qcow2",
        "-device",
        "ide-hd,drive=hd0,bus=ide.0",
        "-nic",
        "none",
        # No USB controller: every crash so far (`WHPX: Unexpected VP exit code 4`) came
        # right after `Ignoring request for interrupt vector 0`, an MSI fired before its
        # vector was programmed. The tablet's controller was the only optional MSI source.
        "-chardev",
        f"socket,id=qmp0,host=127.0.0.1,port={qmp_port},server=off",
        "-mon",
        "chardev=qmp0,mode=control",
        "-chardev",
        f"socket,id=ser0,host=127.0.0.1,port={console.port},server=off",
        "-serial",
        "chardev:ser0",
        "-monitor",
        "none",
        "-display",
        "sdl" if show else "none",
    ]
    # q35's ICH9 AHCI has six ports, ide.0..ide.5; the disk holds ide.0.
    for i, iso in enumerate(cds, start=1):
        args += [
            "-drive",
            f"file={iso},if=none,id=cd{i},media=cdrom,readonly=on",
            "-device",
            f"ide-cd,drive=cd{i},bus=ide.{i}",
        ]
    return subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)


def _screenshot(qmp: QmpClient, uid: str, dst: Path) -> None:
    """screendump writes on the WINDOWS side, so it lands in WINREF_DIR and moves over."""
    tmp = WINREF_DIR / f"{uid}.ppm"
    try:
        # Nudge the (PS/2) mouse first: Windows blanks an idle display, and a black screenshot
        # of a stuck VM is no evidence at all.
        for dx in (1, -1):
            qmp.execute(
                "input-send-event",
                events=[{"type": "rel", "data": {"axis": "x", "value": dx}}],
            )
        qmp.execute("screendump", filename=be.win_path(tmp))
        time.sleep(1)
        shutil.move(str(tmp), dst)
    except Exception:
        return
    _timeline(dst)


def _timeline(shot: Path) -> None:
    """Keep a small timestamped PNG of every screenshot next to it, in shots/.

    One final frame cannot say where typed keystrokes went or when the desktop came up;
    a timeline can. Downscaled so an hour of them is a few MB. Best-effort: Pillow is in
    the venv the CLI runs from, but its absence must not fail a run.
    """
    try:
        from PIL import Image

        out = shot.parent / "shots"
        out.mkdir(exist_ok=True)
        stamp = time.strftime("%H%M%S")
        with Image.open(shot) as im:
            im.thumbnail((640, 400))
            im.save(out / f"{stamp}-{shot.stem}.png")
    except Exception:
        pass


def _boot(
    run: artifacts.Run,
    disk: Path,
    vars_fd: Path,
    cds: list[str],
    *,
    timeout: float,
    press_keys: float,
    args,
    finish: str | None = None,
    actions: list[tuple[float, Callable[[QmpClient], None]]] | None = None,
) -> tuple[str, list[str]]:
    """Boot, wait for QEMU to exit (the guest powers itself off), return (outcome, lines).

    outcome: "exited" | "timeout" | "launch-failed". `press_keys` seconds of Enter presses
    get past the Windows ISO's "Press any key to boot from CD" prompt on the first boot.
    """
    console = _Console(run.dir / "guest-console.log")
    qmp = QmpClient()
    qmp_port = qmp.listen_tcp("0.0.0.0", 0)
    uid = f"winref-{os.getpid()}-{console.port}"
    proc = _launch(
        uid,
        disk,
        vars_fd,
        cds,
        console,
        qmp_port,
        cpus=args.cpus,
        mem=args.mem,
        cpu=args.cpu,
        irqchip=args.irqchip,
        accel=args.accel,
        show=args.show,
    )
    try:
        try:
            qmp.accept(timeout=120)
        except OSError:
            # Not fatal: QMP only drives the extras (key presses, screenshots, actions, the
            # power-button fallback), and each of those already tolerates a dead monitor.
            # A run that missed the connection still booted, ran its payload and powered off
            # (20261011-210309); calling that "launch-failed" threw away a good result. The
            # verdict comes from the console. A QEMU that never started at all still exits
            # immediately and has no console, which the callers grade as a failure.
            run.log("QMP never connected; continuing without it", header="winref qmp")
        t0 = time.monotonic()
        while time.monotonic() - t0 < press_keys and proc.poll() is None:
            try:
                qmp.execute("send-key", keys=[{"type": "qcode", "data": "ret"}])
            except Exception:
                break
            time.sleep(0.5)
        deadline = time.monotonic() + timeout
        booted = time.monotonic()
        pending = sorted(actions or [], key=lambda a: a[0])
        next_shot = 0.0
        finished_at: float | None = None
        powered_down = False
        while proc.poll() is None and time.monotonic() < deadline:
            # The guest asking to power off is not enough: a shutdown requested from
            # FirstLogonCommands came back as a RESTART, and the VM sat at the desktop
            # until the timeout. Once the guest says it is done, press the power button.
            while pending and time.monotonic() - booted >= pending[0][0]:
                _, act = pending.pop(0)
                try:
                    act(qmp)
                except Exception as e:
                    run.log(f"action failed: {e}", header="winref action")
                time.sleep(5)
                _screenshot(qmp, uid, run.dir / f"after-{act.__name__}.ppm")
            if (
                finish
                and finished_at is None
                and any(ln.startswith(finish) for ln in console.lines)
            ):
                finished_at = time.monotonic()
            if finished_at is not None and not powered_down and time.monotonic() > finished_at + 60:
                try:
                    qmp.execute("system_powerdown")
                except Exception:
                    pass
                powered_down = True
                deadline = min(deadline, time.monotonic() + 600)
            # The last screen is the only evidence of a crash QEMU does not survive (a
            # triple fault ends the process, and a dead VM cannot be screendumped).
            if time.monotonic() >= next_shot:
                _screenshot(qmp, uid, run.dir / "last-screen.ppm")
                next_shot = time.monotonic() + SHOT_INTERVAL
            time.sleep(5)  # the guest powers itself off; QEMU exits with it
        if proc.poll() is None:
            _screenshot(qmp, uid, run.dir / "last-screen.ppm")
            be.kill_by_uid(uid)
            return "timeout", console.lines
        if proc.stdout:
            run.log(proc.stdout.read(), header="qemu output")
        return "exited", console.lines
    finally:
        if proc.poll() is None:
            be.kill_by_uid(uid)
        console.close()


def cmd_install(args, repo: Path) -> int:
    run = artifacts.Run("winref-install", "x64")
    p = _paths()
    if args.adopt:
        return _adopt(run, Path(args.adopt), repo)
    iso = Path(args.iso or "")
    if not args.iso or not iso.is_file():
        run.add(artifacts.Stage("iso", status="fail", detail=f"no ISO at {iso}"))
        return run.finalize(repo)
    disk = WINREF_DIR / "install.qcow2"
    work_vars = WINREF_DIR / "install-vars.fd"
    resume = args.resume and disk.is_file() and work_vars.is_file()
    if args.resume and not resume:
        run.add(
            artifacts.Stage("resume", status="fail", detail=f"no install in progress at {disk}")
        )
        return run.finalize(repo)
    if p["base"].exists() and not args.force:
        run.add(
            artifacts.Stage(
                "base", status="fail", detail=f"{p['base']} exists; --force reinstalls over it"
            )
        )
        return run.finalize(repo)
    WINREF_DIR.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(OVMF_CODE, p["code"])
    if not resume:
        shutil.copyfile(OVMF_VARS, work_vars)
        disk.unlink(missing_ok=True)
        cp = _qemu_img("create", "-f", "qcow2", be.win_path(disk), args.disk_size)
        if cp.returncode != 0:
            run.add(artifacts.Stage("disk", status="fail", detail=cp.stderr.strip()[:300]))
            return run.finalize(repo)
    with tempfile.TemporaryDirectory() as td:
        stage = Path(td) / "cd"
        (stage / "winref").mkdir(parents=True)
        shutil.copy(ASSETS / "autounattend.xml", stage / "autounattend.xml")
        for f in ("setup.ps1", "boot.ps1"):
            shutil.copy(ASSETS / f, stage / "winref" / f)
        _make_iso(stage, WINREF_DIR / "unattend.iso", "WINREF")
    run.add(
        artifacts.Stage(
            "prepare",
            detail=f"{'resuming ' if resume else ''}{disk} ({args.disk_size}), answer-file CD",
        )
    )
    outcome, lines = _boot(
        run,
        disk,
        work_vars,
        [be.win_path(iso), be.win_path(WINREF_DIR / "unattend.iso")],
        timeout=args.timeout,
        # Only the FIRST boot must take the CD; a resumed install boots its disk, which
        # is what letting the "press any key" prompt time out does.
        # TCG reaches the prompt far later than WHPX: give it a wider window.
        press_keys=0 if resume else (90 if args.accel == "tcg" else 40),
        args=args,
        finish="WINREF-SETUP: done",
    )
    setup = [ln for ln in lines if ln.startswith("WINREF-SETUP:")]
    done = next((ln for ln in setup if ln.startswith("WINREF-SETUP: done")), None)
    run.extra["setup_lines"] = setup
    if outcome != "exited" or not done:
        run.classification = "WINREF_INSTALL_FAIL"
        run.add(
            artifacts.Stage(
                "install",
                status="fail",
                detail=f"{outcome}; last setup line: {setup[-1] if setup else '(none)'}",
                salient=setup[-8:],
            )
        )
        return run.finalize(repo)
    _promote(run, disk, work_vars, done, str(iso), args.cpu)
    return run.finalize(repo)


def _promote(run: artifacts.Run, disk: Path, vars_fd: Path, done: str, iso: str, cpu: str) -> None:
    """Make a finished install the base image, record it, and keep a pristine copy."""
    p = _paths()
    os.replace(disk, p["base"])
    os.replace(vars_fd, p["vars"])
    state = {
        "installed": time.strftime("%Y-%m-%d %H:%M:%S"),
        "iso": iso,
        "setup": done.removeprefix("WINREF-SETUP: done ").strip(),
        "cpu": cpu,
    }
    p["state"].write_text(json.dumps(state, indent=2))
    run.extra["winref"] = state
    run.add(artifacts.Stage("install", detail=state["setup"]))
    # An install costs ~2 h under TCG; keep the pristine result apart from the live base,
    # which a later `install --force` or a deliberate base change would overwrite.
    snap = _snapshot(POST_INSTALL)
    run.add(artifacts.Stage("snapshot", detail=str(snap)))


def _adopt(run: artifacts.Run, install_run: Path, repo: Path) -> int:
    """Promote install.qcow2 from an install run whose setup finished but whose VM never
    exited on its own (the guest's shutdown came back as a restart). The evidence that
    setup completed is that run's console, not anyone's say-so."""
    console = install_run / "guest-console.log"
    lines = console.read_text(errors="replace").splitlines() if console.is_file() else []
    done = next((ln for ln in lines if ln.startswith("WINREF-SETUP: done")), None)
    disk = WINREF_DIR / "install.qcow2"
    vars_fd = WINREF_DIR / "install-vars.fd"
    if not done or not disk.is_file() or not vars_fd.is_file():
        run.add(
            artifacts.Stage(
                "adopt",
                status="fail",
                detail=f"need `WINREF-SETUP: done` in {console} and an install disk",
            )
        )
        return run.finalize(repo)
    _promote(run, disk, vars_fd, done, f"adopted from {install_run.name}", DEFAULT_CPU)
    return run.finalize(repo)


POST_INSTALL = "post-install"
SNAPSHOT_FILES = ("base", "vars", "state")


def _copy(src: Path, dst: Path) -> None:
    """Copy a (large) file, on the Windows side when both ends are Windows paths.

    shutil over DrvFS failed the first post-install snapshot with ENOMEM from sendfile on a
    20 GB qcow2: the same failure backend.stage_file works around with small writes. A
    native same-drive Copy-Item never crosses the WSL boundary at all.
    """
    try:
        ws, wd = be.win_path(src), be.win_path(dst)
    except ValueError:
        ws = wd = ""
    if not ws or src.stat().st_size < 64 << 20:
        shutil.copyfile(src, dst)  # small, or not a Windows path: DrvFS copes fine
        return
    err = ""
    # Twice: launching powershell.exe through WSL interop itself fails now and then
    # (`UtilAcceptVsock: accept4 failed 110`), which is not the copy failing.
    for _ in range(2):
        cp = subprocess.run(
            [
                "powershell.exe",
                "-NoProfile",
                "-Command",
                f"Copy-Item -LiteralPath '{ws}' -Destination '{wd}' -Force -ErrorAction Stop",
            ],
            capture_output=True,
            text=True,
        )
        if cp.returncode == 0 and dst.is_file() and dst.stat().st_size == src.stat().st_size:
            return
        err = cp.stderr.strip()[-300:]
    raise OSError(f"copy {src} -> {dst} failed: {err}")


def _snapshot(name: str, *, overwrite: bool = False) -> Path:
    """Full copies of the base disk, its UEFI variables and state.json into snapshots/<name>.

    Copies, not qcow2 internal snapshots: an internal snapshot lives INSIDE base.qcow2, so
    the reinstall it is meant to survive deletes it along with the file.
    """
    p = _paths()
    dst = p["snapshots"] / name
    if dst.exists() and not overwrite:
        raise FileExistsError(f"snapshot {name!r} exists at {dst}")
    tmp = p["snapshots"] / f".{name}.partial"
    shutil.rmtree(tmp, ignore_errors=True)
    tmp.mkdir(parents=True)
    for key in SNAPSHOT_FILES:
        if p[key].is_file():
            _copy(p[key], tmp / p[key].name)
    if dst.exists():
        shutil.rmtree(dst)
    os.replace(tmp, dst)
    return dst


def cmd_snapshot(args, repo: Path) -> int:
    p = _paths()
    if not p["base"].is_file():
        print("winref snapshot: no base image to snapshot")
        return 1
    try:
        dst = _snapshot(args.name, overwrite=args.force)
    except FileExistsError as e:
        print(f"winref snapshot: {e} (--force replaces it)")
        return 1
    print(f"winref snapshot: {dst}")
    return 0


def cmd_restore(args, repo: Path) -> int:
    p = _paths()
    src = p["snapshots"] / args.name
    if not (src / p["base"].name).is_file():
        have = sorted(d.name for d in p["snapshots"].iterdir()) if p["snapshots"].is_dir() else []
        print(f"winref restore: no snapshot {args.name!r} (have: {', '.join(have) or 'none'})")
        return 1
    for key in SNAPSHOT_FILES:
        f = src / p[key].name
        if f.is_file():
            tmp = p[key].with_name(p[key].name + ".restoring")
            _copy(f, tmp)
            os.replace(tmp, p[key])
    print(f"winref restore: base <- {src}")
    return 0


def cmd_run(args, repo: Path) -> int:
    run = artifacts.Run("winref-run", "x64")
    _run_payload(run, Path(args.payload), args)
    return run.finalize(repo)


def _run_payload(run: artifacts.Run, payload: Path, args, *, relaunch: int = 0) -> list[str] | None:
    """Boot a throwaway overlay with `payload` as a CD, run its run.cmd, and grade the run.

    Writes output.txt (the payload's stdout, `OUT: ` stripped) into the run dir, records
    the stages, and returns the output lines; None when the payload never finished.

    `relaunch`: how many times to boot the SAME overlay again when QEMU dies under the
    guest. Under WHPX on this host QEMU dies (`Unexpected VP exit code 4`) while Windows
    reboots after a bugcheck: the kmtest column died that way at test 54 of 99. The
    overlay keeps whatever the payload wrote to C:\\winref, so a payload that records its
    progress there (kmtest-sweep.ps1) carries on where it stopped.
    """
    p = _paths()
    if not p["base"].is_file():
        run.add(
            artifacts.Stage(
                "base", status="fail", detail="no base image: vibeos-cli winref install --iso ..."
            )
        )
        return None
    if not (payload / "run.cmd").is_file():
        run.add(artifacts.Stage("payload", status="fail", detail=f"no run.cmd in {payload}"))
        return None
    rd = p["runs"] / run.dir.name
    rd.mkdir(parents=True, exist_ok=True)
    try:
        overlay = rd / "overlay.qcow2"
        cp = _qemu_img(
            "create",
            "-f",
            "qcow2",
            "-b",
            be.win_path(p["base"]),
            "-F",
            "qcow2",
            be.win_path(overlay),
        )
        if cp.returncode != 0:
            run.add(artifacts.Stage("overlay", status="fail", detail=cp.stderr.strip()[:300]))
            return None
        vars_fd = rd / "vars.fd"
        shutil.copyfile(p["vars"], vars_fd)
        _make_iso(payload, rd / "payload.iso", "PAYLOAD")
        lines: list[str] = []
        deadline = time.monotonic() + args.timeout
        lost = 0
        while True:
            outcome, got = _boot(
                run,
                overlay,
                vars_fd,
                [be.win_path(rd / "payload.iso")],
                timeout=max(60.0, deadline - time.monotonic()),
                press_keys=0,
                args=args,
                finish="WINREF: poweroff",
            )
            lines += got
            powered_off = any(ln.startswith("WINREF: poweroff") for ln in got)
            if outcome != "exited" or powered_off or lost >= relaunch:
                break
            lost += 1
            run.log(
                f"QEMU exited with the payload unfinished; booting the same overlay again "
                f"({lost}/{relaunch})",
                header="winref relaunch",
            )
        run.extra["vm_lost"] = lost
    finally:
        if not args.keep:
            shutil.rmtree(rd, ignore_errors=True)
    out = [ln[5:] for ln in lines if ln.startswith("OUT: ")]
    (run.dir / "output.txt").write_text("\n".join(out) + ("\n" if out else ""))
    ready = next((ln for ln in lines if ln.startswith("WINREF: ready")), None)
    exit_line = next((ln for ln in lines if ln.startswith("WINREF: exit=")), None)
    run.extra.update(
        {
            "windows": ready.removeprefix("WINREF: ready ").strip() if ready else None,
            "payload": str(payload),
            "payload_exit": int(exit_line.split("=", 1)[1]) if exit_line else None,
            "output": str(run.dir / "output.txt"),
            "output_lines": len(out),
        }
    )
    if outcome != "exited" or exit_line is None:
        run.classification = "WINREF_RUN_FAIL"
        run.add(
            artifacts.Stage(
                "run",
                status="fail",
                detail=f"{outcome}; ready={'yes' if ready else 'no'}; no exit line"
                if exit_line is None
                else outcome,
                salient=lines[-8:],
            )
        )
    else:
        run.add(
            artifacts.Stage(
                "run",
                detail=f"{exit_line.removeprefix('WINREF: ')}, {len(out)} output lines",
                salient=out[-8:],
            )
        )
        return out
    return None


# The reference column: the conformance lane's suites, graded by Windows 11 itself. Each
# suite runs the guest's own binaries from this tree's build-amd64, with the same markers,
# and is parsed by the same parser the guest's serial log is.
WIN_BASELINE = "win11-amd64"
CANON = "/tmp/vibeos/"  # build paths in the CD manifests are canonical (tools/canon-run.sh)


def staged_bin(manifest: str, repo: Path) -> dict[str, Path]:
    """Every file a livecd manifest puts in reactos/bin: what add_rostests_file staged.

    The guest's sweeps find their binaries there, and kmtest_.exe loads its drivers from
    its own directory, so the Windows run gets exactly that set, laid out the same way.
    """
    out: dict[str, Path] = {}
    for line in manifest.splitlines():
        dst, sep, src = line.partition("=")
        if sep and dst.startswith("reactos/bin/"):
            src = src.strip()
            if src.startswith(CANON):
                src = str(repo / src[len(CANON) :])
            out[dst[len("reactos/bin/") :]] = Path(src)
    return out


def kmtest_order(listed: list[str], built: set[str]) -> list[str]:
    """The tests the Windows column runs, in order: the guest sweep's own list first (so
    tests meet the same predecessors as in the guest), then every other visible test.

    The rest are mostly the guest's QUARANTINES, and they are the point: a test the guest
    cannot survive says nothing on the guest, but what Windows does with it is exactly
    the expected result. Hidden (`-Name`) tests stay out, as they do in the guest.
    """
    seen = set(listed)
    return [*listed, *sorted(t for t in built if t not in seen and not t.startswith("-"))]


def _load(path: Path, name: str):
    import importlib.util
    import sys

    sys.path.insert(0, str(path.parent))  # the parsers import their sibling ratchet.py
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# Exit codes of a process the loader killed before main(): the suite never ran.
LOAD_FAILURES = {
    0xC0000135: "STATUS_DLL_NOT_FOUND",
    0xC0000139: "STATUS_ENTRYPOINT_NOT_FOUND",
    0xC0000142: "STATUS_DLL_INIT_FAILED",
}


def not_run(r: dict) -> bool:
    """The harness could not run this row (a driver that will not load, a binary that
    will not start): its result is about the rig, not about Windows."""
    return r["status"] == "FAILSTART" or (r["status"] == "NO-END" and r.get("errors", 0) > 0)


def column_report(win: list[dict], guest: dict[str, dict]) -> list[str]:
    """One line per row where Windows and the guest floor disagree, sorted by kind.

    It says what was OBSERVED, not why: "Windows clean, guest fails" is where to look
    for a guest bug first, and "fails on Windows 11 too" is a test whose expectation
    Windows does not meet, or one that is timing-sensitive in a VM (KeMutex fails 517
    there), or one our toolchain builds differently from the test's author.
    """
    rows: list[tuple[int, str]] = []
    for r in win:
        g = guest.get(r["name"])
        w_ok = r["status"] == "OK" and r["fail"] == 0
        if not_run(r):
            # Says nothing about the test: Windows never got to run it.
            rows.append(
                (
                    -1,
                    f"{r['name']}: NOT RUN on Windows ({r['status']}, "
                    f"{r.get('errors', 0)} harness error(s))",
                )
            )
            continue
        if g is None:
            if guest:
                rows.append(
                    (
                        0 if w_ok else 2,
                        f"{r['name']}: not in the guest floor; "
                        f"Windows {r['status']} pass={r['pass']} fail={r['fail']}",
                    )
                )
            continue
        g_ok = g["status"] == "OK" and g.get("fail", 0) == 0
        if w_ok and not g_ok:
            rows.append(
                (1, f"{r['name']}: Windows clean, guest {g['status']} fail={g.get('fail', 0)}")
            )
        elif not w_ok:
            shared = len(set(r.get("asserts") or ()) & set(g.get("asserts") or ()))
            rows.append(
                (
                    3,
                    f"{r['name']}: fails on Windows 11 too: Windows {r['status']} "
                    f"fail={r['fail']}, guest fail={g.get('fail', 0)}, "
                    f"{shared} failing assertion(s) shared",
                )
            )
    return [line for _, line in sorted(rows)]


SUITES = {
    # suite: (parser, sweep marker prefix)
    "kmtest": ("parse_kmtest_serial.py", "[KMTEST-SWEEP]"),
    "apitest": ("parse_apitest_serial.py", "[APITEST-SWEEP]"),
}


def _stage_kmtest(stage: Path, repo: Path, only: list[str] | None = None) -> int:
    lists = _load(repo / "tools/payloads/kmtest/test_kmtest_lists.py", "kmtest_lists")
    tests = kmtest_order(lists.listed("amd64"), lists.built_tests("amd64"))
    if only:
        unknown = sorted(set(only) - set(tests))
        if unknown:
            raise FileNotFoundError(f"--only names tests kmtest does not build: {unknown}")
        tests = [t for t in tests if t in only]
    shutil.copy2(ASSETS / "kmtest-sweep.ps1", stage / "kmtest-sweep.ps1")
    (stage / "bin" / "kmtest-tests.txt").write_text("\n".join(tests) + "\n")
    (stage / "run.cmd").write_text(
        '@powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0kmtest-sweep.ps1"'
        ' -Bin "%~dp0bin"\r\n'
    )
    return len(tests)


def _stage_apitest(stage: Path, repo: Path) -> int:
    # The guest's own sweep, unmodified in behaviour: it reads %SystemRoot%\bin, so the
    # payload's bin goes there. Every suite the build produced, with NO quarantine: those
    # protect the GUEST from tests that take it down, and Windows needs no protecting.
    sweep = repo / "tools/payloads/apitest/apitest-sweep-amd64.exe"
    if not sweep.is_file():
        raise FileNotFoundError(f"{sweep}; fix: bash tools/payloads/apitest/build.sh amd64")
    gen = _load(repo / "tools/payloads/apitest/gen_suites.py", "gen_suites")
    suites = gen.discover(repo / "build-amd64")
    shutil.copy2(sweep, stage / "bin" / "apitest-sweep.exe")
    (stage / "bin" / "apitest-suites.txt").write_text("\n".join(suites) + "\n")
    (stage / "run.cmd").write_text(
        "@echo off\r\n"
        'xcopy /e /i /y /q "%~dp0bin" "%SystemRoot%\\bin" >nul\r\n'
        '"%SystemRoot%\\bin\\apitest-sweep.exe"\r\n'
    )
    return len(suites)


def cmd_suite(args, repo: Path) -> int:
    suite = args.winref_cmd
    run = artifacts.Run(f"winref-{suite}", "x64")
    if args.regrade:
        # Grade a finished run's output again (a parser or grading change) without
        # booting Windows: the output is what Windows did, and it does not go stale.
        prev = Path(args.regrade)
        summary = json.loads((prev / "summary.json").read_text())
        extra = summary.get("extra", summary)
        for k in ("windows", "corpus_built", "corpus_files", "listed"):
            run.extra[k] = extra.get(k)
        run.extra["regraded"] = str(prev)
        out = (prev / "output.txt").read_text().splitlines()
        run.add(artifacts.Stage("run", detail=f"regrade of {prev.name}: {len(out)} lines"))
        return _grade(run, suite, out, args, repo)
    manifest = repo / "build-amd64" / "boot" / "livecd.Debug.lst"
    files = staged_bin(manifest.read_text(), repo) if manifest.is_file() else {}
    missing = sorted(n for n, src in files.items() if not src.is_file())
    if not files or missing:
        run.add(
            artifacts.Stage(
                "corpus",
                status="fail",
                detail=(
                    f"{len(missing)} staged file(s) missing, e.g. {missing[:3]}; "
                    if missing
                    else f"no reactos/bin in {manifest}; "
                )
                + "fix: ./vibeos-cli build --arch amd64 --kmtests --apitests --no-wow64-stage",
            )
        )
        return run.finalize(repo)
    stage = run.dir / "payload"
    (stage / "bin").mkdir(parents=True)
    for name, src in files.items():
        (stage / "bin" / name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, stage / "bin" / name)
    try:
        if suite == "kmtest":
            count = _stage_kmtest(stage, repo, args.only)
        else:
            count = _stage_apitest(stage, repo)
    except FileNotFoundError as e:
        run.add(artifacts.Stage("corpus", status="fail", detail=str(e)))
        return run.finalize(repo)
    newest = max(src.stat().st_mtime for src in files.values())
    run.extra.update(
        {
            "corpus_files": len(files),
            "corpus_built": time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(newest)),
            "listed": count,
        }
    )
    # Only the kmtest sweep resumes (its progress lives on C:); the apitest sweep would
    # start over and run every suite twice. Each kmtest that bugchecks Windows costs one
    # relaunch (QEMU dies in the reboot), and two of them do (ObType, RtlRangeList).
    out = _run_payload(run, stage, args, relaunch=10 if suite == "kmtest" else 0)
    if not args.keep:
        shutil.rmtree(stage, ignore_errors=True)
    if out is None:
        return run.finalize(repo)
    return _grade(run, suite, out, args, repo)


def _grade(run: artifacts.Run, suite: str, out: list[str], args, repo: Path) -> int:
    parser_file, marker = SUITES[suite]
    parser = _load(repo / "tools/conformance" / parser_file, parser_file[:-3])
    parsed = parser.parse("\n".join(out))
    results, done = (parsed[0], parsed[1]) if suite == "kmtest" else (parsed[0], parsed[1]["done"])
    table = "\n".join(parser.fmt(r) for r in results) + "\n"
    (run.dir / f"{suite}.txt").write_text(table)
    guest_path = repo / "tools/conformance-baseline/amd64" / f"{suite}.txt"
    guest = parser.load_baseline(guest_path) if guest_path.is_file() else {}
    report = column_report(results, guest)
    clean = sum(1 for r in results if r["status"] == "OK" and r["fail"] == 0)
    # A test the VM died under twice (kmtest-sweep.ps1) says nothing about Windows.
    lost = {ln.split()[2] for ln in out if ln.startswith("[KMTEST-SWEEP] VM-LOST ")}
    unrun = [
        f"{r['name']} (VM lost under it twice)"
        if r["name"] in lost
        else f"{r['name']} ({r['status']}, {r.get('errors', 0)} harness error(s))"
        for r in results
        if not_run(r) or r["name"] in lost
    ]
    if suite == "apitest":
        # A suite whose binary Windows will not even load enumerates no subtests, so it
        # has no rows at all; without this it would vanish from the column silently.
        for name, (code, _first) in sorted(parsed[1]["nosubtests"].items()):
            why = LOAD_FAILURES.get(int(code) & 0xFFFFFFFF)
            if why:
                unrun.append(f"{name} (whole suite: {why})")
    report = [*(f"NOT RUN on Windows: {u}" for u in unrun if "whole suite" in u), *report]
    (run.dir / "column.txt").write_text("\n".join(report) + "\n")
    ran = [r for r in results if r["status"] in ("OK", "FAIL-EXIT")]
    nothing_ran = not ran
    notes = [ln.split("] ", 1)[1] for ln in out if ln.startswith(marker) and " signed " in ln]
    run.extra.update(
        {
            "signed": notes[0] if notes else None,
            "sweep": done,
            "rows": len(results),
            "not_run": unrun,
            "clean": clean,
            "guest_floor": str(guest_path) if guest else None,
            "disagreements": len(report),
            "table": str(run.dir / f"{suite}.txt"),
            "column": str(run.dir / "column.txt"),
        }
    )
    run.add(
        artifacts.Stage(
            "column",
            status="fail" if not done or nothing_ran else "warn" if unrun else "ok",
            detail=f"{len(results)} rows, {clean} clean on Windows"
            + (f", {len(unrun)} NOT RUN (harness)" if unrun else "")
            + (f", {len(report)} disagree with the guest floor" if guest else ", no guest floor")
            + ("" if done else "; SWEEP DID NOT FINISH"),
            salient=report[:12],
        )
    )
    if not done:
        run.classification = f"WINREF_{suite.upper()}_INCOMPLETE"
    elif nothing_ran:
        # Every row is a harness failure (the first kmtest column: a driver that would not
        # load). That is a broken rig, not a reference: refuse to bank it.
        run.classification = f"WINREF_{suite.upper()}_NOT_RUN"
    elif args.write_baseline and getattr(args, "only", None):
        # A subset is not the column: banking it would drop every row it did not run.
        run.add(
            artifacts.Stage(
                "baseline", status="fail", detail="--write-baseline refused with --only"
            )
        )
    elif args.write_baseline:
        dst = repo / "tools/conformance-baseline" / WIN_BASELINE / f"{suite}.txt"
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_text(
            f"# {suite} REFERENCE column: what Windows 11 itself does with each test.\n"
            f"# `vibeos-cli winref {suite} --write-baseline`, {run.extra.get('windows')}, "
            f"corpus built {run.extra['corpus_built']}"
            + (f", from {Path(run.extra['regraded']).name}" if run.extra.get("regraded") else "")
            + ".\n"
            "# Not a floor the guest must match row for row. A row failing here fails on\n"
            "# Windows 11 itself: an expectation Windows does not meet (often 2003-era\n"
            "# behaviour), a test that is timing-sensitive in a VM, or one our toolchain\n"
            "# builds differently from its author's.\n"
            + "".join(f"# NOT RUN on Windows: {u}\n" for u in unrun)
            + table
        )
        run.extra["baseline"] = str(dst)
    return run.finalize(repo)


SEVEN_ZIP = Path(os.environ.get("VIBEOS_WINREF_7Z", "/mnt/c/Program Files/7-Zip/7z.exe"))
# What `logs` pulls off the Windows partition: Setup's own record of every unattend pass
# (which is where "FirstLogonCommands never ran" turned out to be an account that was never
# made an administrator), and the runner's working tree.
LOG_PATHS = [
    "Windows/Panther/UnattendGC/setupact.log",
    "Windows/Panther/setupact.log",
    "Windows/System32/Tasks/winref-boot",  # the runner task as Task Scheduler stored it
    "winref",
]


def largest_ntfs(listing: str) -> tuple[int, int]:
    """(offset, size) in bytes of the largest NTFS partition in a `7z l -slt` listing."""
    best: tuple[int, int] = (0, 0)
    entry: dict[str, str] = {}
    for line in [*listing.replace("\r", "").splitlines(), ""]:
        if " = " in line:
            k, v = line.split(" = ", 1)
            entry[k] = v
            continue
        if entry.get("Path", "").endswith(".ntfs") and "Offset" in entry:
            off, size = int(entry["Offset"]), int(entry["Size"])
            if size > best[1]:
                best = (off, size)
        entry = {}
    return best


def dd_window(offset: int, size: int) -> tuple[int, int]:
    """(skip, count) in MiB for `qemu-img dd bs=1M` to copy exactly one partition.

    qemu-img's dd counts the SKIPPED blocks in `count` (unlike coreutils dd): a count of the
    partition's length alone copied 317 MiB short, dropped NTFS's backup boot sector off the
    end, and 7-Zip refused the result as "not an archive".
    """
    mib = 1 << 20
    if offset % mib:
        raise ValueError(f"partition at {offset} is not MiB-aligned")
    skip = offset // mib
    return skip, skip + -(-size // mib)


def _ntfs_partition(image: Path) -> tuple[int, int]:
    cp = subprocess.run(
        [str(SEVEN_ZIP), "l", "-slt", be.win_path(image)], capture_output=True, text=True
    )
    best = largest_ntfs(cp.stdout)
    if not best[1]:
        raise RuntimeError(f"no NTFS partition in {image}: {cp.stdout[-300:]} {cp.stderr[-300:]}")
    return best


def cmd_logs(args, repo: Path) -> int:
    """Copy Setup's logs and C:\\winref out of a STOPPED VM's disk, without booting it.

    No nbd in WSL and no admin to mount a VHD, so: 7-Zip reads the qcow2's GPT, qemu-img
    copies the Windows partition out raw, and 7-Zip reads NTFS from that. 7-Zip's CLI cannot
    open an image nested in another, hence the copy (~80 GB on the image's drive, deleted
    after).
    """
    p = _paths()
    image = Path(args.image) if args.image else WINREF_DIR / "install.qcow2"
    if not image.is_file():
        image = p["base"]
    out = Path(args.out) if args.out else WINREF_DIR / "logs" / time.strftime("%Y%m%d-%H%M%S")
    skip, count = dd_window(*_ntfs_partition(image))
    raw = WINREF_DIR / "logs-partition.ntfs"
    try:
        cp = _qemu_img(
            "dd",
            "-f",
            "qcow2",
            "-O",
            "raw",
            "bs=1M",
            f"skip={skip}",
            f"count={count}",
            f"if={be.win_path(image)}",
            f"of={be.win_path(raw)}",
        )
        if cp.returncode != 0:
            raise RuntimeError(f"qemu-img dd: {cp.stderr.strip()}")
        out.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                str(SEVEN_ZIP),
                "x",
                "-tntfs",
                be.win_path(raw),
                f"-o{be.win_path(out)}",
                *LOG_PATHS,
                "-y",
            ],
            capture_output=True,
            text=True,
        )
    finally:
        raw.unlink(missing_ok=True)
    got = sorted(str(f.relative_to(out)) for f in out.rglob("*") if f.is_file())
    print(f"winref logs: {image.name} -> {out}")
    for f in got:
        print(f"  {f}")
    if not got:
        print("  (nothing extracted)")
    return 0 if got else 1


_QCODE_PLAIN = {
    " ": "spc",
    "-": "minus",
    "\\": "backslash",
    ".": "dot",
    "/": "slash",
    "=": "equal",
    ";": "semicolon",
    "'": "apostrophe",
    ",": "comma",
}
_QCODE_SHIFT = {":": "semicolon", "_": "minus", '"': "apostrophe", "+": "equal", "?": "slash"}


def keys_for(text: str) -> list[list[str]]:
    """QMP send-key chords that type `text` on a US layout: one chord per character."""
    out: list[list[str]] = []
    for ch in text:
        if ch.isascii() and (ch.islower() or ch.isdigit()):
            out.append([ch])
        elif ch.isascii() and ch.isupper():
            out.append(["shift", ch.lower()])
        elif ch in _QCODE_PLAIN:
            out.append([_QCODE_PLAIN[ch]])
        elif ch in _QCODE_SHIFT:
            out.append(["shift", _QCODE_SHIFT[ch]])
        else:
            raise ValueError(f"cannot type {ch!r}")
    return out


def _press(qmp: QmpClient, *chords: list[str], gap: float = 0.15) -> None:
    for chord in chords:
        qmp.execute("send-key", keys=[{"type": "qcode", "data": k} for k in chord])
        time.sleep(gap)


MAINT_COMMAND = "powershell -ExecutionPolicy Bypass -File D:\\maint.ps1"


def cmd_maint(args, repo: Path) -> int:
    """Run a PowerShell script ELEVATED inside the BASE image, then power off.

    The base cannot be edited offline (7-Zip reads NTFS, nothing here writes it) and a
    reinstall costs ~2 h, so this is the way to change it. It boots the base ITSELF (no
    overlay), so snapshot first; that is why `post-install` exists.

    Normally the script rides the runner: the CD carries a run.cmd that launches it, and
    the runner (elevated, relaying stdout as OUT: lines) does the rest. `--keys` is the
    bootstrap for a base whose runner does not work: wait for the autologon desktop and
    type the script into Win+R as an elevated command (Ctrl+Shift+Enter, Alt+Y at the
    consent prompt) over QMP. The first base was repaired that way.

    Success needs the script's own `WINREF-MAINT: done`. Any WINREF: line is not enough:
    the runner prints its own, and a maintenance run whose script never ran graded itself
    OK that way.
    """
    run = artifacts.Run("winref-maint", "x64")
    p = _paths()
    script = Path(args.script)
    if not p["base"].is_file() or not script.is_file():
        run.add(artifacts.Stage("inputs", status="fail", detail="need a base image and a script"))
        return run.finalize(repo)
    with tempfile.TemporaryDirectory() as td:
        cd = Path(td) / "cd"
        cd.mkdir()
        shutil.copy(script, cd / "maint.ps1")
        shutil.copy(ASSETS / "boot.ps1", cd / "boot.ps1")
        if not args.keys:
            (cd / "run.cmd").write_text(
                "@set WINREF_RUNNER=1\r\n"
                '@powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0maint.ps1"\r\n'
            )
        _make_iso(cd, WINREF_DIR / "maint.iso", "MAINT")
    settle = args.settle if args.settle is not None else (1800 if args.accel == "tcg" else 300)

    def type_command(qmp: QmpClient) -> None:
        _press(qmp, ["meta_l", "r"])
        time.sleep(3)
        _press(qmp, *keys_for(MAINT_COMMAND))
        _press(qmp, ["ctrl", "shift", "ret"])

    def consent(qmp: QmpClient) -> None:
        _press(qmp, ["alt", "y"])

    outcome, lines = _boot(
        run,
        p["base"],
        p["vars"],
        [be.win_path(WINREF_DIR / "maint.iso")],
        timeout=args.timeout,
        press_keys=0,
        args=args,
        finish="WINREF: poweroff",
        actions=[(settle, type_command), (settle + 30, consent)] if args.keys else None,
    )
    maint = [
        ln.removeprefix("OUT: ")
        for ln in lines
        if ln.startswith(("WINREF-MAINT:", "WINREF:", "OUT: WINREF-MAINT:"))
    ]
    run.extra["maint_lines"] = maint
    ok = (
        outcome == "exited"
        and not any("FAILED" in ln for ln in maint)
        and "WINREF-MAINT: done" in maint
    )
    run.add(
        artifacts.Stage(
            "maint",
            status="ok" if ok else "fail",
            detail=f"{outcome}; {maint[-1] if maint else 'no maintenance output'}",
            salient=maint[-10:],
        )
    )
    return run.finalize(repo)


def cmd_status(args, repo: Path) -> int:
    p = _paths()
    if not p["base"].is_file():
        print(f"winref: not installed ({p['base']} missing)")
        return 1
    state = json.loads(p["state"].read_text()) if p["state"].is_file() else {}
    size = p["base"].stat().st_size / 1e9
    print(f"winref: {p['base']} ({size:.1f} GB on disk)")
    for k, v in state.items():
        print(f"  {k}: {v}")
    return 0


def add_parser(sub) -> None:
    w = sub.add_parser("winref", help="real Windows 11 VM: the conformance oracle")
    w.set_defaults(fn=command)
    verbs = w.add_subparsers(dest="winref_cmd", required=True)

    def vm_opts(v) -> None:
        v.add_argument("--cpus", type=int, default=4)
        v.add_argument("--mem", default="8G")
        v.add_argument("--cpu", default=DEFAULT_CPU, help="QEMU -cpu model")
        v.add_argument("--show", action="store_true", help="open a window on the host")
        v.add_argument(
            "--accel",
            default=DEFAULT_ACCEL,
            choices=["tcg", "whpx"],
            help="tcg (default: slow, but WHPX dies mid-Setup on this host) or whpx",
        )
        v.add_argument(
            "--irqchip",
            default=DEFAULT_IRQCHIP,
            choices=["on", "off"],
            help="WHPX kernel-irqchip (Hyper-V's in-kernel APIC)",
        )

    i = verbs.add_parser("install", help="unattended install into the base image")
    i.add_argument("--iso", help="Windows 11 x64 ISO (a /mnt/<drive> path)")
    i.add_argument(
        "--adopt",
        metavar="RUN_DIR",
        help="promote an install whose setup finished (per RUN_DIR's console) but never exited",
    )
    i.add_argument("--disk-size", default="80G")
    i.add_argument("--timeout", type=float, default=INSTALL_TIMEOUT)
    i.add_argument("--force", action="store_true", help="reinstall over an existing base")
    i.add_argument(
        "--resume", action="store_true", help="continue an interrupted install from its disk"
    )
    vm_opts(i)
    r = verbs.add_parser("run", help="boot a throwaway overlay and run <payload>/run.cmd")
    r.add_argument("payload", help="directory with run.cmd at its root (becomes a CD)")
    r.add_argument("--timeout", type=float, default=RUN_TIMEOUT)
    r.add_argument("--keep", action="store_true", help="keep the overlay for inspection")
    vm_opts(r)
    for suite, what in (("kmtest", "kernel-mode"), ("apitest", "user-mode")):
        k = verbs.add_parser(
            suite, help=f"run the {what} {suite} corpus on Windows 11: the reference column"
        )
        k.add_argument("--timeout", type=float, default=RUN_TIMEOUT * 4)
        k.add_argument("--keep", action="store_true", help="keep the overlay and payload")
        k.add_argument(
            "--regrade", metavar="RUN_DIR", help="grade a finished run's output.txt again, no VM"
        )
        if suite == "kmtest":
            k.add_argument(
                "--only",
                nargs="+",
                metavar="TEST",
                help="run just these tests (a new test, graded by Windows first)",
            )
        k.add_argument(
            "--write-baseline",
            action="store_true",
            help=f"store the result as tools/conformance-baseline/{WIN_BASELINE}/{suite}.txt",
        )
        vm_opts(k)
    verbs.add_parser("status", help="installed base image and its Windows build")
    mt = verbs.add_parser("maint", help="run a script elevated inside the BASE image (writes it)")
    mt.add_argument("script", help="PowerShell script, typed into Win+R as D:\\maint.ps1")
    mt.add_argument(
        "--keys", action="store_true", help="type the script into Win+R (runner broken)"
    )
    mt.add_argument("--settle", type=float, help="--keys: seconds to the autologon desktop")
    mt.add_argument("--timeout", type=float, default=RUN_TIMEOUT * 2)
    vm_opts(mt)
    sn = verbs.add_parser("snapshot", help="full copy of the base image into snapshots/<name>")
    sn.add_argument("name", nargs="?", default=POST_INSTALL)
    sn.add_argument("--force", action="store_true", help="replace an existing snapshot")
    rs = verbs.add_parser("restore", help="put snapshots/<name> back as the base image")
    rs.add_argument("name", nargs="?", default=POST_INSTALL)
    lg = verbs.add_parser("logs", help="copy Setup logs + C:\\winref off a stopped VM's disk")
    lg.add_argument("--image", help="qcow2 to read (default: install.qcow2, else base)")
    lg.add_argument("--out", help="destination directory (a /mnt/<drive> path)")


def command(args, repo: Path) -> int:
    return {
        "install": cmd_install,
        "run": cmd_run,
        "kmtest": cmd_suite,
        "apitest": cmd_suite,
        "status": cmd_status,
        "logs": cmd_logs,
        "snapshot": cmd_snapshot,
        "maint": cmd_maint,
        "restore": cmd_restore,
    }[args.winref_cmd](args, repo)
