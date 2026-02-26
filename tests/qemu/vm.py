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
PROJECT_DIR = Path(__file__).resolve().parent.parent.parent.parent
QEMU_DIR = PROJECT_DIR / "qemu"


@dataclass
class ArchConfig:
    arch: str
    qemu_bin: str
    machine_args: list[str]
    memory: str
    boot_marker: str
    boot_timeout: float = 120.0
    min_boot_bytes: int = 10000

    def qemu_args(self, work_dir: Path) -> list[str]:
        """Build full QEMU command line. Subclasses override via callbacks."""
        raise NotImplementedError


@dataclass
class Arm64Config(ArchConfig):
    arch: str = "arm64"
    qemu_bin: str = "qemu-system-aarch64"
    machine_args: list[str] = field(default_factory=lambda: ["-M", "virt", "-cpu", "cortex-a72"])
    memory: str = "1G"
    # Empty marker = wait for serial idle (ARM64_UART_TRACE_ENABLE is off)
    boot_marker: str = ""
    boot_timeout: float = 120.0

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img = QEMU_DIR / "boot.img"
        uefi_fw = QEMU_DIR / "QEMU_EFI.fd"
        efivars_raw = work_dir / "efivars.fd"
        efivars = work_dir / "efivars.qcow2"

        # Copy efivars to temp dir first (avoids file lock from running QEMU),
        # then convert to qcow2 (required for savevm/loadvm snapshots)
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
            # file-posix.locking=off: allow coexistence with other QEMU instances
            "-drive", f"file={boot_img},format=raw,if=virtio,readonly=on,file.locking=off",
            "-device", "ramfb",
            # -display none instead of -nographic: the latter hijacks serial0
            # to stdio, stealing PL011 output from our chardev socket
            "-display", "none",
        ]


@dataclass
class I386Config(ArchConfig):
    arch: str = "i386"
    qemu_bin: str = "qemu-system-i386"
    machine_args: list[str] = field(default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"])
    memory: str = "512M"
    # bootcd.ini defaults to Setup_Debug with /DEBUG /DEBUGPORT=COM1 /SOS
    boot_marker: str = ""
    boot_timeout: float = 180.0
    min_boot_bytes: int = 500

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-i386" / "bootcd.iso"
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
    # livecd.ini defaults to LiveCD_Debug with /DEBUG /DEBUGPORT=COM1 /SOS
    # Must use a marker — desktop produces continuous debug output (never goes idle)
    boot_marker: str = "SwitchDesktop OK"
    boot_timeout: float = 180.0
    min_boot_bytes: int = 10000

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-amd64" / "livecd.iso"
        scratch = work_dir / "scratch.qcow2"
        return [
            *self.machine_args,
            "-m", self.memory,
            "-vga", "std",
            "-cdrom", str(iso),
            "-display", "none",
            "-no-reboot",
        ]


@dataclass
class Arm64TestKernelConfig(Arm64Config):
    """ARM64 config with /TESTKERNEL appended to freeldr.ini boot options.

    Copies boot.img to the work directory and patches the Options line
    so the kernel enters the serial test dispatch loop instead of launching SMSS.
    """
    arch: str = "arm64-test"
    boot_marker: str = "TESTKERNEL READY"
    boot_timeout: float = 180.0

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img_src = QEMU_DIR / "boot.img"
        boot_img = work_dir / "boot-test.img"
        uefi_fw = QEMU_DIR / "QEMU_EFI.fd"
        efivars_raw = work_dir / "efivars.fd"
        efivars = work_dir / "efivars.qcow2"

        # Copy boot.img so we can patch it without modifying the original
        shutil.copy2(boot_img_src, boot_img)

        # Extract freeldr.ini, append /TESTKERNEL, write back
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

        # Convert efivars to qcow2
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

        # Unix sockets have a 104-byte sun_path limit on macOS.
        # Use a short /tmp directory for socket files.
        self._sock_dir = Path(tempfile.mkdtemp(prefix="ros-"))
        atexit.register(lambda d=self._sock_dir: shutil.rmtree(d, ignore_errors=True))

        self.qmp_path = self._sock_dir / "qmp.sock"
        self.serial_path = self._sock_dir / "ser.sock"
        self.scratch_path = work_dir / "scratch.qcow2"

        self.qmp = QmpClient()
        self.serial = SerialClient()
        self._proc: subprocess.Popen | None = None
        self._boot_output: str = ""

    def start(self) -> None:
        """Start QEMU process, connect QMP + serial, wait for boot."""
        # Create scratch qcow2 for savevm/loadvm support
        subprocess.run(
            ["qemu-img", "create", "-f", "qcow2", str(self.scratch_path), "64M"],
            check=True,
            capture_output=True,
        )

        # Start serial server socket BEFORE launching QEMU.
        # QEMU will connect to us as a client, guaranteeing we capture
        # every byte from the very start of boot.
        self.serial.listen(str(self.serial_path))

        # Build QEMU command
        cmd = [
            self.config.qemu_bin,
            # QMP control socket (QEMU is server)
            "-qmp", f"unix:{self.qmp_path},server,nowait",
            # Serial: QEMU connects to OUR server socket (server=off)
            "-chardev", f"socket,id=ser0,path={self.serial_path},server=off,reconnect=1",
            "-serial", "chardev:ser0",
            # Scratch disk for snapshots
            "-drive", f"file={self.scratch_path},format=qcow2,if=none,id=scratch",
            "-device", "virtio-blk-device,drive=scratch" if self.config.arch.startswith("arm64")
                else "virtio-blk-pci,drive=scratch",
            # No monitor (we use QMP)
            "-monitor", "none",
            # Architecture-specific args
            *self.config.qemu_args(self.work_dir),
        ]

        # Log command for debugging
        print(f"[QemuVM] {' '.join(cmd)}")

        # Start QEMU process
        self._proc = subprocess.Popen(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            preexec_fn=os.setpgrp,
        )

        # Accept QEMU's connection to our serial socket
        self.serial.accept(timeout=15.0)

        # Connect QMP for VM control
        self._wait_for_socket(self.qmp_path, timeout=10.0)
        self._connect_with_retry(
            lambda: self.qmp.connect(str(self.qmp_path)),
            "QMP",
        )

    def wait_for_boot(self) -> str:
        """Wait for boot to complete. Returns all boot output.

        Uses boot_marker if set, otherwise waits for serial output to go idle
        (no new data for several seconds after receiving substantial output).
        """
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
        return self._boot_output

    def save_snapshot(self, name: str = "ready") -> None:
        """Save VM state via QMP/HMP savevm."""
        result = self.qmp.hmp(f"savevm {name}")
        if "Error" in result:
            raise RuntimeError(f"savevm failed: {result}")
        # savevm can take a moment — wait for completion by querying status
        time.sleep(0.5)

    def restore_snapshot(self, name: str = "ready") -> None:
        """Restore VM state via QMP/HMP loadvm."""
        result = self.qmp.hmp(f"loadvm {name}")
        if "Error" in result:
            raise RuntimeError(f"loadvm failed: {result}")
        # After loadvm, serial socket may have buffered stale data — drain it
        time.sleep(0.5)
        self.serial.read_available(timeout=0.2)

    def serial_output(self, timeout: float = 1.0) -> str:
        """Read available serial output since last read."""
        return self.serial.read_available(timeout=timeout)

    def screendump(self, path: str | Path) -> Path:
        """Save a screenshot of the VM display via QMP screendump.

        Returns the path to the saved PPM file.
        """
        path = Path(path)
        result = self.qmp.execute("screendump", filename=str(path))
        # Give QEMU a moment to write the file
        time.sleep(0.5)
        if not path.exists():
            raise RuntimeError(f"screendump failed: {result}")
        return path

    def stop(self) -> None:
        """Kill QEMU process, clean up sockets."""
        # Close clients first
        self.serial.close()
        self.qmp.close()

        if self._proc is not None:
            try:
                # Try graceful quit via signal
                os.killpg(os.getpgid(self._proc.pid), signal.SIGTERM)
                self._proc.wait(timeout=5)
            except (ProcessLookupError, subprocess.TimeoutExpired):
                try:
                    os.killpg(os.getpgid(self._proc.pid), signal.SIGKILL)
                    self._proc.wait(timeout=2)
                except (ProcessLookupError, subprocess.TimeoutExpired):
                    pass
            self._proc = None

        # Clean up socket directory
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
            # Check if QEMU process died
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
