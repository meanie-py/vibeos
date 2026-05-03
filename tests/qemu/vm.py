"""QEMU VM lifecycle manager for ReactOS test sessions."""

from __future__ import annotations

import atexit
import os
import shutil
import signal
import subprocess
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path

from .qmp_client import QmpClient
from .serial_client import SerialClient

# Project root: os-playground/
PROJECT_DIR = Path(__file__).resolve().parent.parent.parent  # vibeos/ repo root
QEMU_DIR = PROJECT_DIR / "qemu"


@dataclass
class ArchConfig:
    arch: str
    qemu_bin: str
    machine_args: list[str]
    memory: str
    boot_marker: str
    boot_timeout: float = 120.0
    min_boot_bytes: int = 1000
    qmp_timeout: float = 10.0
    # How long to wait after boot_marker for the desktop to come up.
    # The boot marker proves the kernel booted; this extra time lets
    # user-mode processes (SMSS, csrss, winlogon, explorer) finish.
    desktop_settle_seconds: float = 0.0
    # Milestones expected to appear in the boot output (substring match).
    # Validated by test_milestones after wait_for_boot completes.
    # Order is informational — we report which milestones were reached / missed.
    required_milestones: list[str] = field(default_factory=list)

    def qemu_args(self, work_dir: Path) -> list[str]:
        """Build full QEMU command line. Subclasses override via callbacks."""
        raise NotImplementedError


@dataclass
class Arm64Config(ArchConfig):
    arch: str = "arm64"
    qemu_bin: str = "qemu-system-aarch64"
    machine_args: list[str] = field(default_factory=lambda: [
        "-M", "virt", "-cpu", "cortex-a72",
        "-accel", "tcg,thread=multi",
    ])
    memory: str = "1G"
    # Wait for SMSS user-mode dispatch to be set up. This proves the kernel
    # got to user-mode entry — currently the highest reliably-reached point
    # on ARM64. SMSS doesn't currently reach its _main (LdrInitializeThunk
    # appears to fail silently). Idle detection alone is too weak — every
    # silent kernel passes.
    boot_marker: str = "[PS] PspUserThreadStartup: returning"
    boot_timeout: float = 300.0
    min_boot_bytes: int = 5000
    qmp_timeout: float = 10.0
    desktop_settle_seconds: float = 30.0
    # Aspirational chain — test_milestones reports each as reached/missed.
    # Currently the [SMSS]+ markers do not fire on ARM64; that's the bug.
    required_milestones: list[str] = field(default_factory=lambda: [
        "[KERNEL] KiSystemStartup entered",
        "IoInitSystem returning TRUE",
        "[PS] PspUserThreadStartup: returning",
        "[SMSS] _main entered",
        "[CSRSS] _main entered",
        "[WINLOGON] WinMain entered",
        "[EXPLORER] _tWinMain entered",
        "shell ready",
    ])

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img = QEMU_DIR / "boot.img"
        uefi_fw = QEMU_DIR / "QEMU_EFI.fd"
        efivars_raw = work_dir / "efivars.fd"
        efivars = work_dir / "efivars.qcow2"

        shutil.copy2(QEMU_DIR / "efivars.fd", efivars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2",
             str(efivars_raw), str(efivars)],
            check=True, capture_output=True,
        )
        efivars_raw.unlink()

        return [
            *self.machine_args,
            "-m", self.memory,
            "-drive", f"if=pflash,format=raw,file={uefi_fw},readonly=on",
            "-drive", f"if=pflash,format=qcow2,file={efivars}",
            "-drive", f"file={boot_img},format=raw,if=virtio,readonly=on,file.locking=off",
            "-device", "ramfb",
            "-display", "none",
        ]


@dataclass
class I386Config(ArchConfig):
    arch: str = "i386"
    qemu_bin: str = "qemu-system-i386"
    machine_args: list[str] = field(default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"])
    memory: str = "512M"
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    boot_timeout: float = 360.0
    min_boot_bytes: int = 500
    desktop_settle_seconds: float = 5.0
    required_milestones: list[str] = field(default_factory=lambda: [
        "[SMSS] _main entered",
        "[CSRSS] _main entered",
        "[WINLOGON] WinMain entered",
        "[EXPLORER] _tWinMain entered",
        "shell ready",
    ])

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-i386" / "livecd.iso"
        return [
            *self.machine_args,
            "-m", self.memory,
            "-vga", "std",
            "-cdrom", str(iso),
            "-display", "none",
            "-no-reboot",
        ]


@dataclass
class Amd64Config(ArchConfig):
    arch: str = "amd64"
    qemu_bin: str = "qemu-system-x86_64"
    machine_args: list[str] = field(default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"])
    memory: str = "512M"
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    boot_timeout: float = 360.0
    min_boot_bytes: int = 1000
    desktop_settle_seconds: float = 5.0
    required_milestones: list[str] = field(default_factory=lambda: [
        "[SMSS] _main entered",
        "[CSRSS] _main entered",
        "[WINLOGON] WinMain entered",
        "[EXPLORER] _tWinMain entered",
        "shell ready",
    ])

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-amd64" / "livecd.iso"
        return [
            *self.machine_args,
            "-m", self.memory,
            "-vga", "cirrus",
            "-cdrom", str(iso),
            "-display", "none",
            "-no-reboot",
        ]


@dataclass
class Arm64TestKernelConfig(Arm64Config):
    """ARM64 config with /TESTKERNEL appended to freeldr.ini boot options."""
    arch: str = "arm64-test"
    boot_marker: str = "TESTKERNEL READY"
    boot_timeout: float = 180.0
    desktop_settle_seconds: float = 0.0

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img_src = QEMU_DIR / "boot.img"
        boot_img = work_dir / "boot-test.img"
        uefi_fw = QEMU_DIR / "QEMU_EFI.fd"
        efivars_raw = work_dir / "efivars.fd"
        efivars = work_dir / "efivars.qcow2"

        shutil.copy2(boot_img_src, boot_img)

        ini_path = work_dir / "freeldr.ini"
        subprocess.run(
            ["mcopy", "-o", "-i", str(boot_img), "::/freeldr.ini", str(ini_path)],
            check=True, capture_output=True,
        )
        ini_text = ini_path.read_text()
        if "/TESTKERNEL" not in ini_text:
            ini_text = ini_text.replace(
                "/SOS /RDPATH=",
                "/SOS /TESTKERNEL /RDPATH=",
            )
            ini_path.write_text(ini_text)
        subprocess.run(
            ["mcopy", "-o", "-i", str(boot_img), str(ini_path), "::/freeldr.ini"],
            check=True, capture_output=True,
        )

        shutil.copy2(QEMU_DIR / "efivars.fd", efivars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2",
             str(efivars_raw), str(efivars)],
            check=True, capture_output=True,
        )
        efivars_raw.unlink()

        return [
            *self.machine_args,
            "-m", self.memory,
            "-drive", f"if=pflash,format=raw,file={uefi_fw},readonly=on",
            "-drive", f"if=pflash,format=qcow2,file={efivars}",
            "-drive", f"file={boot_img},format=raw,if=virtio,readonly=on,file.locking=off",
            "-device", "ramfb",
            "-display", "none",
        ]


ARCH_CONFIGS: dict[str, type[ArchConfig]] = {
    "arm64": Arm64Config,
    "arm64-test": Arm64TestKernelConfig,
    "i386": I386Config,
    "amd64": Amd64Config,
}


def get_arch_config(arch: str) -> ArchConfig:
    cls = ARCH_CONFIGS.get(arch)
    if cls is None:
        raise ValueError(f"Unknown architecture: {arch!r}. Choose from: {list(ARCH_CONFIGS)}")
    return cls()


class QemuVM:
    """Manages a single QEMU VM instance with QMP + serial control."""

    def __init__(self, config: ArchConfig, work_dir: Path) -> None:
        self.config = config
        self.work_dir = work_dir
        self.work_dir.mkdir(parents=True, exist_ok=True)

        self._sock_dir = Path(tempfile.mkdtemp(prefix="ros-"))
        atexit.register(lambda d=self._sock_dir: shutil.rmtree(d, ignore_errors=True))

        self.qmp_path = self._sock_dir / "qmp.sock"
        self.serial_path = self._sock_dir / "ser.sock"
        self.scratch_path = work_dir / "scratch.qcow2"
        self.debugcon_path = work_dir / "debugcon.bin"

        self.qmp = QmpClient()
        self.serial = SerialClient()
        self._proc: subprocess.Popen | None = None
        self._boot_output: str = ""

    def start(self) -> None:
        """Start QEMU process, connect QMP + serial."""
        subprocess.run(
            ["qemu-img", "create", "-f", "qcow2", str(self.scratch_path), "64M"],
            check=True, capture_output=True,
        )

        self.serial.listen(str(self.serial_path))

        cmd = [
            self.config.qemu_bin,
            "-qmp", f"unix:{self.qmp_path},server,nowait",
            "-chardev", f"socket,id=ser0,path={self.serial_path},server=off,reconnect=1",
            "-serial", "chardev:ser0",
            # Scratch disk for snapshots (writable qcow2 backend for savevm).
            # i386: if=none without a device — FreeLDR crashes on unpartitioned
            # BIOS-visible disks; QEMU savevm only needs a block backend.
            "-drive", f"file={self.scratch_path},format=qcow2,if=none,id=scratch",
            *(
                ["-device", "virtio-blk-device,drive=scratch" if self.config.arch.startswith("arm64")
                     else "virtio-blk-pci,drive=scratch"]
                if self.config.arch != "i386" else []
            ),
            "-monitor", "none",
            *(
                ["-debugcon", f"file:{self.debugcon_path}"]
                if not self.config.arch.startswith("arm64") else []
            ),
            *self.config.qemu_args(self.work_dir),
        ]

        print(f"[QemuVM] {' '.join(cmd)}")

        self._proc = subprocess.Popen(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            preexec_fn=os.setpgrp,
        )

        self.serial.accept(timeout=15.0)

        self._wait_for_socket(self.qmp_path, timeout=10.0)
        self._connect_with_retry(
            lambda: self.qmp.connect(str(self.qmp_path), timeout=self.config.qmp_timeout),
            "QMP",
        )

    def wait_for_boot(self) -> str:
        """Wait for boot to complete. Returns all boot output.

        Uses boot_marker if set, otherwise waits for serial output to go idle.
        After detection, waits desktop_settle_seconds for user-mode to finish.
        """
        try:
            if self.config.boot_marker:
                self._boot_output = self.serial.read_until(
                    self.config.boot_marker,
                    timeout=self.config.boot_timeout,
                )
            else:
                self._boot_output = self.serial.read_until_idle(
                    idle_seconds=5.0,
                    timeout=self.config.boot_timeout,
                    min_bytes=self.config.min_boot_bytes,
                )
        except TimeoutError:
            # Save whatever serial output was captured before the timeout
            self._boot_output = self.serial.partial_output
            raise

        # Wait for desktop to come up (user-mode processes need time after
        # the kernel finishes loading boot drivers).
        settle = self.config.desktop_settle_seconds
        if settle > 0:
            print(f"[QemuVM] Waiting {settle:.0f}s for desktop to settle...")
            # Keep draining serial during settle (some arches produce late output)
            try:
                extra = self.serial.read_until_idle(
                    idle_seconds=settle,
                    timeout=settle + 10,
                    min_bytes=0,
                )
                self._boot_output += extra
            except TimeoutError:
                pass  # Settle time elapsed — that's fine

        return self._boot_output

    def save_snapshot(self, name: str = "ready") -> None:
        """Save VM state via QMP/HMP savevm."""
        result = self.qmp.hmp(f"savevm {name}")
        if "Error" in result:
            raise RuntimeError(f"savevm failed: {result}")
        time.sleep(0.5)

    def restore_snapshot(self, name: str = "ready") -> None:
        """Restore VM state via QMP/HMP loadvm."""
        result = self.qmp.hmp(f"loadvm {name}")
        if "Error" in result:
            raise RuntimeError(f"loadvm failed: {result}")
        time.sleep(0.5)
        self.serial.read_available(timeout=0.2)

    def serial_output(self, timeout: float = 1.0) -> str:
        """Read available serial output since last read."""
        return self.serial.read_available(timeout=timeout)

    def screendump(self, path: str | Path) -> Path:
        """Save a screenshot via QMP screendump."""
        path = Path(path)
        result = self.qmp.execute("screendump", filename=str(path))
        time.sleep(0.5)
        if not path.exists():
            raise RuntimeError(f"screendump failed: {result}")
        return path

    def read_debugcon(self) -> bytes:
        """Read raw debugcon (bochs debug port 0xE9) output."""
        if self.debugcon_path.exists():
            return self.debugcon_path.read_bytes()
        return b""

    def debugcon_report(self) -> str | None:
        """Decode debugcon output into a triage report."""
        raw = self.read_debugcon()
        if not raw:
            return None
        try:
            from .debugcon import debugcon_report as _report
            return _report(raw, self.config.arch)
        except Exception:
            return f"Debugcon: {len(raw)} bytes (catalog unavailable)"

    def stop(self) -> None:
        """Kill QEMU process, clean up sockets."""
        self.serial.close()
        self.qmp.close()

        if self._proc is not None:
            try:
                os.killpg(os.getpgid(self._proc.pid), signal.SIGTERM)
                self._proc.wait(timeout=5)
            except (ProcessLookupError, subprocess.TimeoutExpired):
                try:
                    os.killpg(os.getpgid(self._proc.pid), signal.SIGKILL)
                    self._proc.wait(timeout=2)
                except (ProcessLookupError, subprocess.TimeoutExpired):
                    pass
            self._proc = None

        shutil.rmtree(self._sock_dir, ignore_errors=True)

    def _connect_with_retry(self, connect_fn, name: str, timeout: float = 10.0) -> None:
        """Retry a connect function until it succeeds or QEMU dies."""
        deadline = time.monotonic() + timeout
        last_err = None
        while time.monotonic() < deadline:
            if self._proc and self._proc.poll() is not None:
                self._raise_start_error(f"QEMU exited with code {self._proc.returncode}")
            try:
                connect_fn()
                return
            except (ConnectionRefusedError, ConnectionError, OSError) as e:
                last_err = e
                time.sleep(0.2)
        self._raise_start_error(f"Failed to connect to {name}: {last_err}")

    def _wait_for_socket(self, path: Path, timeout: float = 10.0) -> None:
        """Wait for a Unix socket file to appear."""
        deadline = time.monotonic() + timeout
        while not path.exists():
            if time.monotonic() > deadline:
                self._raise_start_error(f"Socket {path} did not appear within {timeout}s")
            if self._proc and self._proc.poll() is not None:
                self._raise_start_error(f"QEMU exited with code {self._proc.returncode}")
            time.sleep(0.1)

    def _raise_start_error(self, msg: str) -> None:
        """Raise RuntimeError with QEMU stderr for diagnostics."""
        stderr = ""
        if self._proc and self._proc.stderr:
            try:
                stderr = self._proc.stderr.read().decode("utf-8", errors="replace")
            except Exception:
                pass
        raise RuntimeError(f"{msg}\nQEMU stderr:\n{stderr}")
