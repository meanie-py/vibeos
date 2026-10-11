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
            out = proc.stdout.read() if proc.poll() is not None and proc.stdout else ""
            run.log(out, header="qemu output (QMP never connected)")
            return "launch-failed", console.lines
        t0 = time.monotonic()
        while time.monotonic() - t0 < press_keys and proc.poll() is None:
            try:
                qmp.execute("send-key", keys=[{"type": "qcode", "data": "ret"}])
            except Exception:
                break
            time.sleep(0.5)
        deadline = time.monotonic() + timeout
        next_shot = 0.0
        while proc.poll() is None and time.monotonic() < deadline:
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
    iso = Path(args.iso)
    if not iso.is_file():
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
    os.replace(disk, p["base"])
    os.replace(work_vars, p["vars"])
    state = {
        "installed": time.strftime("%Y-%m-%d %H:%M:%S"),
        "iso": str(iso),
        "setup": done.removeprefix("WINREF-SETUP: done ").strip(),
        "cpu": args.cpu,
    }
    p["state"].write_text(json.dumps(state, indent=2))
    run.extra["winref"] = state
    run.add(artifacts.Stage("install", detail=state["setup"], salient=setup))
    # An install costs ~2 h under TCG; keep the pristine result apart from the live base,
    # which a later `install --force` or a deliberate base change would overwrite.
    snap = _snapshot(POST_INSTALL)
    run.add(artifacts.Stage("snapshot", detail=str(snap)))
    return run.finalize(repo)


POST_INSTALL = "post-install"
SNAPSHOT_FILES = ("base", "vars", "state")


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
            shutil.copyfile(p[key], tmp / p[key].name)
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
            shutil.copyfile(f, tmp)
            os.replace(tmp, p[key])
    print(f"winref restore: base <- {src}")
    return 0


def cmd_run(args, repo: Path) -> int:
    run = artifacts.Run("winref-run", "x64")
    p = _paths()
    payload = Path(args.payload)
    if not p["base"].is_file():
        run.add(
            artifacts.Stage(
                "base", status="fail", detail="no base image: vibeos-cli winref install --iso ..."
            )
        )
        return run.finalize(repo)
    if not (payload / "run.cmd").is_file():
        run.add(artifacts.Stage("payload", status="fail", detail=f"no run.cmd in {payload}"))
        return run.finalize(repo)
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
            return run.finalize(repo)
        vars_fd = rd / "vars.fd"
        shutil.copyfile(p["vars"], vars_fd)
        _make_iso(payload, rd / "payload.iso", "PAYLOAD")
        outcome, lines = _boot(
            run,
            overlay,
            vars_fd,
            [be.win_path(rd / "payload.iso")],
            timeout=args.timeout,
            press_keys=0,
            args=args,
        )
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
    return run.finalize(repo)


SEVEN_ZIP = Path(os.environ.get("VIBEOS_WINREF_7Z", "/mnt/c/Program Files/7-Zip/7z.exe"))
# What `logs` pulls off the Windows partition: Setup's own record of every unattend pass
# (which is where "FirstLogonCommands never ran" turned out to be an account that was never
# made an administrator), and the runner's working tree.
LOG_PATHS = ["Windows/Panther/UnattendGC/setupact.log", "Windows/Panther/setupact.log", "winref"]


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
    i.add_argument("--iso", required=True, help="Windows 11 x64 ISO (a /mnt/<drive> path)")
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
    verbs.add_parser("status", help="installed base image and its Windows build")
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
        "status": cmd_status,
        "logs": cmd_logs,
        "snapshot": cmd_snapshot,
        "restore": cmd_restore,
    }[args.winref_cmd](args, repo)
