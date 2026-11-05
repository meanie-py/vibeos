"""Host integration tests for the native serial service; no Pico required."""

import itertools
import json
import os
import pathlib
import signal
import socket
import subprocess
import time

import pytest

ROOT = pathlib.Path(__file__).resolve().parents[3]
TOOL = ROOT / "tools" / "bench-infra"


def wait_for(predicate, timeout=8):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(0.02)
    raise AssertionError("timed out waiting for serial service")


def state(bench):
    try:
        return json.loads((bench / "serial-current.json").read_text())
    except FileNotFoundError:
        return None


def recv_line(sock):
    result = bytearray()
    while not result.endswith(b"\n"):
        data = sock.recv(1)
        if not data:
            raise AssertionError("connection closed before authentication")
        result += data
    return bytes(result)


@pytest.mark.skipif(os.name == "nt", reason="SIGTERM lifecycle test requires POSIX")
def test_capture_reconnect_input_and_shutdown(tmp_path):
    listener = socket.socket()
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", 0))
    listener.listen(2)
    listener.settimeout(8)
    token = b"abcdefghijklmnop"
    token_file = tmp_path / "token"
    token_file.write_bytes(token + b"\r\n")
    env = {
        **os.environ,
        "PICO_DEVICE_TOKEN_FILE": str(token_file),
        "PICO_SERIAL_PORT": str(listener.getsockname()[1]),
    }
    proc = subprocess.Popen(
        [str(TOOL), "--root", str(ROOT), "serial", str(tmp_path), "127.0.0.1"],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    try:
        first, _ = listener.accept()
        with first:
            first.settimeout(8)
            assert recv_line(first) == b"PICOSERIAL 1 " + token + b"\n"
            first.sendall(b"OK\nfirst\x00\xff\n[picousb: 3 console bytes lost]\n")
            current = wait_for(lambda: s if (s := state(tmp_path)) and s["connected"] else None)
            directory = pathlib.Path(current["directory"])
            wait_for(
                lambda: (
                    (directory / "serial.bin").stat().st_size
                    >= len(b"first\x00\xff\n[picousb: 3 console bytes lost]\n")
                )
            )
            send = tmp_path / "serial-send.bin"
            send.write_bytes(b"input\x00\xff")
            received = bytearray()
            while len(received) < 7:
                received.extend(first.recv(7 - len(received)))
            assert bytes(received) == b"input\x00\xff"
            wait_for(lambda: not send.exists())
        second, _ = listener.accept()
        with second:
            second.settimeout(8)
            assert recv_line(second) == b"PICOSERIAL 1 " + token + b"\n"
            second.sendall(b"OK\nsecond\n")
            expected = b"first\x00\xff\n[picousb: 3 console bytes lost]\nsecond\n"
            wait_for(lambda: (directory / "serial.bin").read_bytes() == expected)
            rows = [
                tuple(map(int, row.split()))
                for row in (directory / "serial.times").read_text().splitlines()
            ]
            assert rows[0][0] == 0
            assert all(a[0] < b[0] and a[1] <= b[1] for a, b in itertools.pairwise(rows))
            assert rows[-1][0] < len(expected)
            proc.send_signal(signal.SIGTERM)
            assert proc.wait(timeout=8) == 0
            stopped = state(tmp_path)
            assert stopped is not None
            assert stopped["status"] == "STOPPED"
            assert not stopped["connected"]
    finally:
        if proc.poll() is None:
            proc.terminate()
            proc.wait(timeout=8)
        listener.close()


@pytest.mark.skipif(os.name == "nt", reason="POSIX executable and signal fixture")
def test_auth_rejection_is_terminal_and_redacted(tmp_path):
    listener = socket.socket()
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(8)
    token = b"abcdefghijklmnop"
    token_file = tmp_path / "token"
    token_file.write_bytes(token)
    proc = subprocess.Popen(
        [str(TOOL), "--root", str(ROOT), "serial", str(tmp_path), "127.0.0.1"],
        env={
            **os.environ,
            "PICO_DEVICE_TOKEN_FILE": str(token_file),
            "PICO_SERIAL_PORT": str(listener.getsockname()[1]),
        },
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        conn, _ = listener.accept()
        with conn:
            assert recv_line(conn) == b"PICOSERIAL 1 " + token + b"\n"
            conn.sendall(b"ERR\n")
        stdout, stderr = proc.communicate(timeout=8)
        assert proc.returncode != 0
        assert token not in stdout + stderr
        current = state(tmp_path)
        assert current is not None
        assert current["engine"] == "vibeos-bench"
        assert current["process_start_ticks"] is not None
        assert current["status"] == "FAILED"
        assert current["error"] == "console authentication failed"
        assert token.decode() not in json.dumps(current)
    finally:
        if proc.poll() is None:
            proc.terminate()
            proc.wait(timeout=8)
        listener.close()
