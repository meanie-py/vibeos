"""Requested serial logs fail loudly while the native drainer stays alive."""

import socket
import time

import pytest

from tests.qemu.serial_client import SerialClient


def peer(client, path):
    client.listen(path)
    writer = socket.socket(socket.AF_UNIX)
    writer.connect(str(path))
    client.accept(timeout=1)
    return writer


def test_dev_full_tee_failure_beats_ready_and_remains_sticky(tmp_path):
    client = SerialClient()
    writer = None
    try:
        client.set_tee("/dev/full")
        writer = peer(client, tmp_path / "serial.sock")
        writer.sendall(b"READY\n")
        with pytest.raises(OSError, match=r"serial tee /dev/full:"):
            client.read_until("READY", timeout=1)
        assert "READY" in client.partial_output
        # Snapshot remains readable even though every consuming wait refuses.
        text, _, dead = client._snapshot()
        assert "READY" in text and not dead
        writer.sendall(b"after failure\n" + b"x" * 131072 + b"END\n")
        deadline = time.monotonic() + 2
        while not client._snapshot()[0].endswith("END\n") and time.monotonic() < deadline:
            time.sleep(0.01)
        text, _, dead = client._snapshot()
        assert text.endswith("END\n") and not dead
        for consume in (
            lambda: client.read_until("END", timeout=1),
            lambda: client.read_until_idle(idle_seconds=0, timeout=1, min_bytes=0),
            lambda: client.read_available(timeout=0),
        ):
            with pytest.raises(OSError, match=r"serial tee /dev/full:"):
                consume()
    finally:
        if writer is not None:
            writer.close()
        client.close()


def test_tee_reopen_failure_is_not_silently_disabled(tmp_path):
    client = SerialClient()
    writer = None
    target = tmp_path / "requested.log"
    try:
        client.set_tee(target)
        target.unlink()
        writer = peer(client, tmp_path / "serial.sock")
        writer.sendall(b"READY\n")
        with pytest.raises(OSError, match=r"serial tee .*requested\.log"):
            client.read_until("READY", timeout=1)
        deadline = time.monotonic() + 1
        while "READY" not in client._snapshot()[0] and time.monotonic() < deadline:
            time.sleep(0.01)
        assert "READY" in client._snapshot()[0]
    finally:
        if writer is not None:
            writer.close()
        client.close()
