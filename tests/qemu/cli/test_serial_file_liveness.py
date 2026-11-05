"""Standalone file liveness callbacks remain live during native blocking waits."""

import threading
import time

import pytest

from tests.qemu.serial_client import SerialClient


def test_file_alive_change_interrupts_an_inflight_native_read(tmp_path):
    path = tmp_path / "serial"
    path.write_bytes(b"partial progress")
    running = threading.Event()
    running.set()
    client = SerialClient()
    timer = None
    try:
        client.follow_file(path, running.is_set)
        client.accept(timeout=1)
        timer = threading.Timer(0.05, running.clear)
        timer.start()
        began = time.monotonic()
        with pytest.raises(ConnectionError, match="closed"):
            client.read_until("never", timeout=0.8)
        assert time.monotonic() - began < 0.5
        assert client.partial_output == "partial progress"
    finally:
        if timer is not None:
            timer.join(timeout=1)
        client.close()


def test_file_final_marker_is_drained_before_producer_death(tmp_path):
    path = tmp_path / "serial"
    path.write_bytes(b"before\n")
    running = threading.Event()
    running.set()
    client = SerialClient()

    def producer():
        time.sleep(0.05)
        with path.open("ab") as stream:
            stream.write(b"FINAL marker\n")
            stream.flush()
        running.clear()

    thread = threading.Thread(target=producer)
    try:
        client.follow_file(path, running.is_set)
        client.accept(timeout=1)
        thread.start()
        assert client.read_until("FINAL marker", timeout=1) == "before\nFINAL marker"
        assert client.read_available(timeout=0.5) == "\n"
        began = time.monotonic()
        assert client.read_available(timeout=1) == ""
        assert time.monotonic() - began < 0.5
    finally:
        thread.join(timeout=1)
        client.close()


def test_file_idle_wait_drains_final_bytes_even_below_minimum(tmp_path):
    path = tmp_path / "serial"
    path.write_bytes(b"last bytes")
    running = threading.Event()
    running.set()
    client = SerialClient()
    timer = None
    try:
        client.follow_file(path, running.is_set)
        client.accept(timeout=1)
        timer = threading.Timer(0.05, running.clear)
        timer.start()
        assert client.read_until_idle(idle_seconds=60, timeout=1, min_bytes=100000) == "last bytes"
    finally:
        if timer is not None:
            timer.join(timeout=1)
        client.close()


def test_missing_file_accept_observes_producer_death(tmp_path):
    path = tmp_path / "not-created"
    running = threading.Event()
    running.set()
    client = SerialClient()
    timer = threading.Timer(0.05, running.clear)
    try:
        client.follow_file(path, running.is_set)
        timer.start()
        began = time.monotonic()
        with pytest.raises(ConnectionError, match="exited"):
            client.accept(timeout=1)
        assert time.monotonic() - began < 0.5
        assert not path.exists()
    finally:
        timer.join(timeout=1)
        client.close()
