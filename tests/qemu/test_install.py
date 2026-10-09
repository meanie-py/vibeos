"""Unattended-install smoke (amd64-bios): `bootcdregtest.iso` installs onto a blank
disk, and the installed disk then boots to the desktop with no CD attached.

Nothing else runs the installer. Every other boot starts from the livecd, an image
that was laid down at build time, so setuplib's disk-writing half (partitioning,
formatting, the file copy, the hives, the boot loader) and usetup itself -- the
bootcd's smss.exe -- were all Rust that no gate had ever executed.

Two VMs over one disk image:

1. The CD and the blank disk. usetup reads the CD's `unattend.inf`, partitions,
   formats, copies, installs FreeLdr into the MBR and VBR, and reboots. `-no-reboot`
   turns that reboot into QEMU exiting, which is the end of the stage.
2. The disk alone. It boots FreeLdr from the MBR, runs the second (GUI) stage, reboots
   on its own, and comes up to Explorer. This VM allows reboots, so the stage is one
   serial stream across both boots, and the reboot count is checked.

Media: a fresh `vibeos-cli build --arch amd64`. The test then builds `bootcdregtest`
itself (an mkisofs over the same build tree, a minute or two), so the CD can never be
older than the build it is checked against.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path

import pytest

from tests.qemu import vm as vm_mod
from tests.qemu.cli import config as cli_config
from tests.qemu.cli.provenance import check_staleness
from tests.qemu.test_smoke import _FATAL_MARKERS
from tests.qemu.vm import PROJECT_DIR, Amd64BiosConfig, QemuVM, launch_with_retry

_BUILD = PROJECT_DIR / "build-amd64"
_ISO = _BUILD / "bootcdregtest.iso"
_DISK_NAME = "install.qcow2"
_DISK_SIZE = "4G"

# usetup prints these (usetup-rust SuccessPage / QuitPage). The success line is the
# end of stage 1; the quit line means setup gave up and is about to reboot anyway.
_SETUP_DONE = "[USETUP] installation succeeded"
_SETUP_QUIT = "[USETUP] setup quit"
_SMSS = "[SMSS] _main entered"


@dataclass
class _SetupStage(Amd64BiosConfig):
    """Stage 1: boot the regtest CD with the blank install disk attached."""

    disk: Path = Path()
    boot_marker: str = _SETUP_DONE
    # Partition, format, ~1000 files out of a cabinet and the hive import, under TCG: the
    # registry page alone runs 20+ minutes. A crash still ends the stage at once (fatal
    # markers are polled), so the budget only bounds a genuine stall.
    boot_timeout: float = 3600.0
    hang_silence_seconds: float = 600.0
    desktop_settle_seconds: float = 0.0
    memory: str = "1G"

    def qemu_args(self, work_dir: Path) -> list[str]:
        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-drive",
            f"file={self.disk},format=qcow2,if=none,id=inst",
            "-device",
            "ide-hd,drive=inst,bus=ide.0,unit=0",
            "-drive",
            f"id=cd0,if=none,format=raw,file={_ISO},readonly=on",
            "-device",
            "ide-cd,drive=cd0,bus=ide.1,unit=0,bootindex=0",
            "-vga",
            "std",
            "-display",
            "none",
            "-no-reboot",
        ]


@dataclass
class _InstalledStage(_SetupStage):
    """Stage 2: the installed disk alone, reboots allowed (GUI setup reboots once)."""

    boot_marker: str = "[EXPLORER] _tWinMain entered"
    desktop_settle_seconds: float = 15.0
    # GUI setup runs on the first boot -- with Explorer already up beside it (winlogon
    # spawns the shell without waiting for setup) -- and reboots when it finishes; the
    # marker that counts is the desktop of the SECOND boot.
    boots_required: int = 2

    def qemu_args(self, work_dir: Path) -> list[str]:
        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-drive",
            f"file={self.disk},format=qcow2,if=none,id=inst",
            "-device",
            "ide-hd,drive=inst,bus=ide.0,unit=0,bootindex=0",
            "-vga",
            "std",
            "-display",
            "none",
        ]


def _missing_boot_drivers() -> list[str]:
    """txtsetup.sif's boot drivers (media 'x') that the CD does not carry.

    setupldr loads each one, and on BIOS a missing one stops the boot at a "Cannot load
    boot driver" message box that waits for a key: an unattended run then sits silent
    until its timeout. The fork pruned sacdrv, isapnp and floppy and left their rows, and
    that is exactly what the first run of this test found, 30 minutes later.
    """
    listing = (_BUILD / "boot" / "bootcdregtest.stripped.lst").read_text().lower()
    on_cd = set(re.findall(r"/([^/=\n]+)=", listing))
    section, missing = "", []
    for raw in (
        (PROJECT_DIR / "boot" / "bootdata" / "txtsetup.sif").read_text("latin-1").splitlines()
    ):
        line = raw.split(";")[0].strip()
        if line.startswith("["):
            section = line.strip("[]").lower()
            continue
        if section not in ("sourcedisksfiles", "sourcedisksfiles.amd64") or "=" not in line:
            continue
        name, fields = (part.strip() for part in line.split("=", 1))
        media = fields.split(",")[6].strip() if fields.count(",") >= 6 else ""
        if media == "x" and name.lower() not in on_cd:
            missing.append(name)
    return missing


def _uninstalled_service_images() -> list[str]:
    """hivesys.inf services whose image the CD carries but setup never copies to the disk.

    Setup copies reactos.cab plus txtsetup.sif's [SourceDisksFiles]; a NO_CAB driver with no
    row there exists on the CD and nowhere on the installed system. For a boot-start one,
    FreeLdr then stops the installed disk's first boot at a "Cannot load boot driver"
    message box (usbxhci). A boot-start image missing from the CD too is reported as well.
    """
    bootdata = PROJECT_DIR / "boot" / "bootdata"
    listing = (_BUILD / "boot" / "bootcdregtest.stripped.lst").read_text().lower()
    on_cd = set(re.findall(r"/([^/=\n]+)=", listing))
    # reactos.dff.<config>.dyn: the cab manifest with generator expressions resolved.
    dyn = max(
        (_BUILD / "boot" / "bootdata" / "packages").glob("reactos.dff.*.dyn"),
        key=lambda f: f.stat().st_mtime,
    )
    in_cab = set(re.findall(r'[/\\]([^/\\"\s]+)"?\s+\d+\s*$', dyn.read_text().lower(), re.M))
    section, copied = "", set()
    for raw in bootdata.joinpath("txtsetup.sif").read_text("latin-1").splitlines():
        line = raw.split(";")[0].strip()
        if line.startswith("["):
            section = line.strip("[]").lower()
        elif section in ("sourcedisksfiles", "sourcedisksfiles.amd64") and "=" in line:
            copied.add(line.split("=", 1)[0].strip().lower())
    services: dict[str, dict[str, str]] = {}
    for m in re.finditer(
        r'HKLM,"SYSTEM\\CurrentControlSet\\Services\\([^"\\]+)","(ImagePath|Start)",0x[0-9a-f]+,(.*)',
        bootdata.joinpath("hivesys.inf").read_text("latin-1"),
        re.I,
    ):
        services.setdefault(m.group(1), {})[m.group(2)] = m.group(3).strip().strip('"')
    missing = []
    for name, values in sorted(services.items()):
        image = values.get("ImagePath", "").split()[0:1]
        if not image:
            continue
        file = image[0].replace("\\", "/").rsplit("/", 1)[-1].lower()
        if file in in_cab or file in copied:
            continue
        if file in on_cd or int(values.get("Start", "0x3"), 16) == 0:
            missing.append(f"{name} ({file})")
    return missing


def _missing_display_services() -> list[str]:
    """txtsetup.sif [Display] services with no key in hivesys.inf.

    Setup sets the chosen display's `Services\\<service>` Start value and its `Device0` mode;
    a service the tree no longer ships stops the install at "Setup failed to update display
    registry settings" -- the VGA miniports were pruned and the rows still named them.
    """
    bootdata = PROJECT_DIR / "boot" / "bootdata"
    hive = bootdata.joinpath("hivesys.inf").read_text("latin-1").lower()
    section, missing = "", set()
    for raw in bootdata.joinpath("txtsetup.sif").read_text("latin-1").splitlines():
        line = raw.split(";")[0].strip()
        if line.startswith("["):
            section = line.strip("[]").lower()
            continue
        if section != "display" or "=" not in line:
            continue
        fields = [f.strip() for f in line.split("=", 1)[1].split(",")]
        if len(fields) > 2 and fields[2]:
            svc = fields[2].lower()
            if f"services\\{svc}\\device0" not in hive:
                missing.add(fields[2])
    return sorted(missing)


@pytest.fixture(autouse=True)
def restore_vm() -> None:
    """Overrides conftest's: that one depends on the shared livecd `vm` fixture, and
    would boot (and clean-rebuild) a whole desktop before this test ran."""


@pytest.fixture(scope="module")
def install_disk(arch: str, tmp_path_factory: pytest.TempPathFactory) -> Path:
    if arch != "amd64-bios":
        pytest.skip("the install smoke runs on amd64-bios (usetup installs a BIOS FreeLdr)")
    freshness = check_staleness(PROJECT_DIR, "amd64")
    assert freshness.fresh, f"vibeos-cli build --arch amd64: {freshness.reasons}"
    built = subprocess.run(
        ["bash", str(PROJECT_DIR / "tools" / "canon-run.sh"), "ninja", "bootcdregtest"],
        cwd=_BUILD,
        capture_output=True,
        text=True,
    )
    assert built.returncode == 0 and _ISO.is_file(), (
        f"ninja bootcdregtest failed ({built.returncode}):\n"
        f"{built.stdout[-3000:]}{built.stderr[-3000:]}"
    )
    missing = _missing_boot_drivers()
    assert not missing, f"txtsetup.sif lists boot drivers the CD does not carry: {missing}"
    images = _uninstalled_service_images()
    assert not images, f"hive services whose image setup never installs: {images}"
    services = _missing_display_services()
    assert not services, f"txtsetup.sif [Display] names services hivesys.inf lacks: {services}"
    work = tmp_path_factory.mktemp("install")
    disk = work / _DISK_NAME
    subprocess.run(
        ["qemu-img", "create", "-f", "qcow2", str(disk), _DISK_SIZE],
        check=True,
        capture_output=True,
    )
    return disk


def _boot(config: _SetupStage, work: Path) -> tuple[QemuVM, str, str | None]:
    """Launch, wait for the stage's marker. Returns (vm, serial, failure-or-None)."""
    # The same override every other boot honours (vm.get_arch_config); these configs are
    # built here, not through it.
    mem = os.environ.get("VIBEOS_QEMU_MEMORY", "").strip()
    if mem:
        config.memory = mem
    cli_cfg = cli_config.load(PROJECT_DIR)
    # The fleet's 30-minute QEMU lifetime cap is below a TCG install's stage-1 time (run 22
    # was killed mid-registry-page). Raise it for THIS VM only, as vm.py asks of a long run:
    # start() reads it for the watchdog and the lock file's deadline, then it is restored.
    cap = vm_mod.MAX_VM_LIFETIME_SECONDS
    vm_mod.MAX_VM_LIFETIME_SECONDS = max(cap, int(config.boot_timeout) + 900)
    # Every gated boot launches through the shared retry: a QEMU that never connects its
    # serial socket under host load is the HOST, and run 5 of this test died of exactly that.
    started, _attempts, launch_exc = launch_with_retry(
        lambda: QemuVM(
            config,
            work,
            backend=cli_cfg.get("qemu.backend", "native"),
            windows=cli_cfg.get("qemu.windows", {}),
            serial_log=work / f"{work.name}_serial.log",
        ),
        {},
        lambda msg: print(f"[install] {msg}"),
        "launch",
    )
    vm_mod.MAX_VM_LIFETIME_SECONDS = cap
    if launch_exc is not None:
        raise launch_exc
    assert started is not None
    vm = started
    print(f"\n[install] live serial: tail -f {vm.live_log_path}")
    try:
        return vm, _wait_marker(vm), None
    except (TimeoutError, ConnectionError, RuntimeError) as exc:
        # The loaders print nothing to serial before the kernel: a FreeLdr panic is only
        # on debugcon (`!!FREELDR-RUST-PANIC@file:line`), so carry its tail with the failure.
        raw = vm.read_debugcon() or b""
        tail = raw[-600:].decode("latin-1", "replace")
        return (
            vm,
            vm._boot_output or vm.serial.partial_output,
            f"{type(exc).__name__}: {exc}\ndebugcon tail: {tail!r}",
        )


def _wait_marker(vm: QemuVM) -> str:
    """The stage's marker, or fail as soon as a fatal marker shows.

    Not `wait_for_boot`: that waits for the marker or a timeout, and a bugchecked stage
    prints nothing more -- run 4 sat on 0x71 for ten minutes. Polling in slices lets a
    fatal marker end the stage within seconds.
    """
    config = vm.config
    deadline = time.monotonic() + config.boot_timeout
    need = getattr(config, "boots_required", 1)
    seen = ""  # markers already consumed from boots before the one that counts
    while True:
        try:
            out = seen + vm.serial.read_until(config.boot_marker, timeout=20)
            if out.count(_SMSS) < need:
                seen = out
                continue
        except TimeoutError:
            window = seen + vm.serial.partial_output
            vm._boot_output = window
            fatal = _fatal(window)
            if fatal is not None:
                # Let the bugcheck banner and flight recorder finish printing.
                try:
                    vm._boot_output = vm.serial.read_until_idle(
                        idle_seconds=5, timeout=30, min_bytes=0
                    )
                except TimeoutError:
                    pass
                raise RuntimeError(
                    f"fatal marker {fatal!r} before {config.boot_marker!r}"
                ) from None
            if time.monotonic() >= deadline:
                # The guest is alive and silent: where every vCPU is parked is the only
                # evidence a quiet wedge leaves, and it is only reachable before teardown.
                phase = f"install stage timeout waiting for {config.boot_marker!r}"
                note = vm.save_hang_state(phase)
                if note:
                    print(note)
                raise
            continue
        if config.desktop_settle_seconds > 0:
            try:
                out += vm.serial.read_until_idle(
                    idle_seconds=config.desktop_settle_seconds,
                    timeout=config.desktop_settle_seconds + 10,
                    min_bytes=0,
                )
            except TimeoutError:
                pass
        vm._boot_output = out
        return out


def _staged_disk(vm: QemuVM) -> Path:
    """Where QEMU actually wrote the disk: windows-host stages a copy."""
    staging = getattr(vm, "_win_staging", None)
    return staging / _DISK_NAME if staging is not None else vm.config.disk  # type: ignore[attr-defined]


# usetup prints every popup (usetup-rust PopupError). Unattended, a popup that waits for
# ENTER is a hang with the reason on screen only, so it ends the stage like a bugcheck.
_SETUP_POPUP = "[USETUP] error popup:"


def _fatal(serial: str) -> str | None:
    for marker in (*_FATAL_MARKERS, _SETUP_POPUP):
        if marker in serial:
            return marker
    return None


def _wait_reboot(vm: QemuVM, timeout: float) -> bool:
    """True once QEMU exits (`-no-reboot` turns the guest's reboot into an exit). False on
    a fatal marker -- a bugcheck in the shutdown leaves QEMU up and printing nothing more --
    or on the timeout."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if _wait_exit(vm, 5) is not None:
            return True
        try:
            vm.serial.read_until("\x00never\x00", timeout=1)
        except (TimeoutError, ConnectionError):
            pass
        if _fatal(vm.serial.partial_output) is not None:
            time.sleep(5)  # let the bugcheck report finish
            try:
                vm.serial.read_until("\x00never\x00", timeout=1)
            except (TimeoutError, ConnectionError):
                pass
            return False
    return False


def _wait_exit(vm: QemuVM, timeout: float) -> int | None:
    proc = vm._proc
    deadline = time.monotonic() + timeout
    while proc is not None and proc.poll() is None and time.monotonic() < deadline:
        time.sleep(1)
    return proc.poll() if proc is not None else None


@pytest.mark.timeout(7200)
def test_unattended_install_then_boot(install_disk: Path) -> None:
    root = install_disk.parent
    problems: list[str] = []

    # --- stage 1: text-mode setup from the CD ------------------------------------------
    vm, serial, failure = _boot(_SetupStage(disk=install_disk), root / "setup")
    installed = failure is None
    try:
        # usetup flushes and calls NtShutdownSystem(ShutdownReboot) next.
        if installed and not _wait_reboot(vm, 300):
            tail = serial + vm.serial.partial_output
            problems.append(
                "stage 1: setup reported success but the VM never rebooted "
                f"(fatal={_fatal(tail)}):\n{tail[-3000:]}"
            )
        # Once setup has said it succeeded, its hives are flushed and the boot code written,
        # so the installed disk is worth booting even if the shutdown after it crashed:
        # one run then reports both, rather than hiding stage 2 behind the reboot.
        written = _staged_disk(vm)
        if installed and written != install_disk:
            if vm._proc is not None and vm._proc.poll() is None:
                # Still running (it crashed instead of rebooting): pause it first. vm_stop
                # drains and flushes every block device, so the qcow2 on disk carries the
                # clusters QEMU had allocated but not yet written back.
                try:
                    vm.qmp.execute("stop")
                except Exception:
                    pass
            shutil.copyfile(written, install_disk)
    finally:
        vm.stop()
    pages = re.findall(r"\[USETUP\] page (\d+)", serial)
    assert installed, (
        f"stage 1 (text setup) failed: {failure}\nfatal={_fatal(serial)} pages={pages}\n"
        f"{serial[-6000:]}"
    )
    assert _SETUP_QUIT not in serial, f"setup quit instead of installing: pages={pages}"

    # --- stage 2: the installed disk, GUI setup, then the desktop -----------------------
    vm, serial, failure = _boot(_InstalledStage(disk=install_disk), root / "installed")
    try:
        if failure is not None:
            try:
                vm.screendump(root / "fail.ppm")
                from PIL import Image

                Image.open(root / "fail.ppm").save(
                    PROJECT_DIR / "qemu" / "amd64-bios_install.png", "PNG"
                )
            except Exception:
                pass
    finally:
        vm.stop()
    boots = serial.count(_SMSS)
    if failure is not None:
        problems.append(
            f"stage 2 (installed disk) failed after {boots} boot(s): {failure}\n"
            f"fatal={_fatal(serial)}\n{serial[-6000:]}"
        )
    elif _fatal(serial) is not None:
        problems.append(f"stage 2: fatal marker {_fatal(serial)!r} in the installed boot")
    elif boots != 2:
        # GUI setup on the first boot, the desktop on the second. One boot means the second
        # stage never ran (the hive still said setup was done); more than two is a reboot
        # loop that happened to reach Explorer.
        problems.append(f"stage 2: expected 2 boots (GUI setup, desktop), saw {boots}")
    assert not problems, "\n\n".join(problems)
