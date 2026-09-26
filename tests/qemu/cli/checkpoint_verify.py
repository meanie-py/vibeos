"""Destructive round-trip proof on a PRIVATE checkpoint attempt, never the seed."""

from __future__ import annotations

import re
import socket
import time

from .. import backend as be
from .checkpoint import CheckpointError, RestoredVM, pause, registers, restore


class Probe:
    """Small GDB remote client for stopped physical-memory probes only."""

    def __init__(self, vm: RestoredVM):
        if vm.gdb_port is None:
            raise CheckpointError("Verification requires a GDB port")
        self.sock = socket.create_connection((vm.gdb_host, vm.gdb_port), timeout=10)

    def packet(self, text: str) -> str:
        payload = text.encode("ascii")
        self.sock.sendall(b"$" + payload + b"#" + f"{sum(payload) % 256:02x}".encode())
        if self.sock.recv(1) != b"+":
            raise CheckpointError("GDB probe packet was not acknowledged")
        if self.sock.recv(1) != b"$":
            raise CheckpointError("GDB probe reply is malformed")
        reply = bytearray()
        while True:
            byte = self.sock.recv(1)
            if byte == b"#":
                break
            if not byte:
                raise CheckpointError("GDB probe connection closed")
            reply.extend(byte)
        checksum = b""
        while len(checksum) < 2:
            part = self.sock.recv(2 - len(checksum))
            if not part:
                raise CheckpointError("GDB checksum truncated")
            checksum += part
        if int(checksum, 16) != sum(reply) % 256:
            raise CheckpointError("GDB probe checksum mismatch")
        self.sock.sendall(b"+")
        return reply.decode("ascii")

    def physical(self, enable: bool) -> None:
        if self.packet(f"Qqemu.PhyMemMode:{int(enable)}") != "OK":
            raise CheckpointError("QEMU does not support physical-memory GDB probes")

    def read(self, address: int) -> str:
        response = self.packet(f"m{address:x},8")
        if not re.fullmatch(r"[0-9a-fA-F]{16}", response):
            raise CheckpointError(f"Cannot read probe RAM at 0x{address:x}: {response}")
        return response


def disk(vm: RestoredVM) -> bytes:
    """Read a QMP backup, never open an active qcow2 behind QEMU's caches."""
    target = (vm._win_staging or vm.work_dir) / "verify-disk.raw"
    blocks = vm.qmp.execute("query-block")["return"]
    block = next((b for b in blocks if b["device"] == "scratch"), None)
    if block is None:
        raise CheckpointError("Scratch block backend missing")
    with target.open("wb") as stream:
        stream.truncate(block["inserted"]["image"]["virtual-size"])
    path = be.win_path(target) if vm._backend == "windows-host" else str(target)
    vm.qmp.execute(
        "blockdev-add",
        **{
            "node-name": "verify-target",
            "driver": "raw",
            "file": {"driver": "file", "filename": path},
        },
    )
    vm.qmp.execute(
        "blockdev-backup",
        **{
            "job-id": "verify-copy",
            "device": "scratch",
            "target": "verify-target",
            "sync": "full",
            "auto-dismiss": False,
        },
    )
    deadline = time.monotonic() + 30
    while True:
        jobs = vm.qmp.execute("query-jobs")["return"]
        job = next((j for j in jobs if j["id"] == "verify-copy"), None)
        if job is None:
            raise CheckpointError("Disk verification job disappeared without a result")
        if job["status"] == "concluded":
            if job.get("error"):
                raise CheckpointError(f"Disk verification job failed: {job['error']}")
            break
        if time.monotonic() >= deadline:
            raise CheckpointError("Disk verification copy timed out")
        time.sleep(0.05)
    vm.qmp.execute("job-dismiss", id="verify-copy")
    vm.qmp.execute("blockdev-del", **{"node-name": "verify-target"})
    with target.open("rb") as stream:
        data = stream.read(512)
    target.unlink()
    if len(data) != 512:
        raise CheckpointError("Disk verification copy is truncated")
    return data


def verify(vm: RestoredVM, pause_seconds: int) -> dict:
    restore(vm)
    base_registers = registers(vm)
    base_disk = disk(vm)
    shared_data = 0xFFDF0000 if vm.config.arch.startswith("i386") else 0xFFFFF78000000000
    # A known data field, never firmware code/page tables: GDB code writes also
    # invalidate TCG translations, which is a separate debugger/JIT operation.
    address = shared_data + 0x14  # SharedSystemTime
    probe = Probe(vm)
    try:
        probe.physical(False)
        base_ram = probe.read(address)
        poison = "a5" * 8 if base_ram != "a5" * 8 else "5a" * 8
        if probe.packet(f"M{address:x},8:{poison}") != "OK" or probe.read(address) != poison:
            raise CheckpointError("RAM mutation did not take effect")
        pattern = "0x5a" if base_disk == b"\xa5" * 512 else "0xa5"
        reply = vm.qmp.hmp(f'qemu-io scratch "write -P {pattern} 0 512"')
        if disk(vm) == base_disk:
            raise CheckpointError(f"Disk mutation did not take effect: {reply}")
        # Keep the guest stopped while waiting; use monotonic time throughout.
        deadline = time.monotonic() + pause_seconds
        while time.monotonic() < deadline:
            time.sleep(min(0.25, max(0, deadline - time.monotonic())))
        probe.sock.close()
        restore(vm)
        probe = Probe(vm)
        probe.physical(False)
        if probe.read(address) != base_ram or disk(vm) != base_disk:
            raise CheckpointError("Snapshot did not roll back RAM and writable disk")
        if registers(vm) != base_registers:
            raise CheckpointError("Snapshot did not roll back all-vCPU registers")
        # SharedUserData.InterruptTime is guest timer-maintained memory, not
        # host wall time. Require a mapped, advancing value to make this proof
        # meaningful. Very early boot seeds may not have it yet: fail explicitly.
        probe.physical(False)
        # Kernel mapping is independent of the current process's user page tables.
        # sdk/include/xdk/{arm64,amd64,i386}/ke.h: SharedInterruptTime.
        clock_address = shared_data + 8
        before_clock = probe.read(clock_address)
        probe.sock.close()
        deadline = time.monotonic() + pause_seconds
        while time.monotonic() < deadline:
            time.sleep(min(0.25, max(0, deadline - time.monotonic())))
        resumed = time.monotonic()
        vm.qmp.execute("cont")
        time.sleep(0.5)
        pause(vm)
        ran_seconds = time.monotonic() - resumed
        probe = Probe(vm)
        probe.physical(False)
        after_clock = probe.read(clock_address)
        if before_clock == after_clock:
            raise CheckpointError("Guest InterruptTime did not advance; use a later capture marker")
        guest_seconds = (
            int.from_bytes(bytes.fromhex(after_clock), "little")
            - int.from_bytes(bytes.fromhex(before_clock), "little")
        ) / 10_000_000
        if not 0 < guest_seconds <= ran_seconds + 0.5:
            raise CheckpointError(
                f"Guest interrupt time jumped {guest_seconds:.3f}s "
                f"during {ran_seconds:.3f}s execution"
            )
        probe.sock.close()
        restore(vm)
        probe = Probe(vm)
        probe.physical(False)
        if probe.read(clock_address) != before_clock:
            raise CheckpointError("Snapshot did not restore guest InterruptTime")
        return {
            "ram_rollback": True,
            "disk_rollback": True,
            "all_vcpu_registers": True,
            "guest_interrupt_time_rollback": True,
            "host_pause_seconds": pause_seconds,
            "clock_before": before_clock,
            "clock_after_run": after_clock,
            "guest_elapsed_seconds": guest_seconds,
            "execution_seconds": ran_seconds,
        }
    finally:
        # Close without detach (which would resume the guest).
        probe.sock.close()
