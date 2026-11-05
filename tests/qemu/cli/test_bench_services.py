"""Exercise bench services through the real native process and sockets."""

import http.client
import socket
import subprocess
import time
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
EXECUTABLE = ROOT / "tools" / "bench-infra"


def free_port(kind):
    with socket.socket(socket.AF_INET, kind) as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


@pytest.fixture
def launch(tmp_path):
    processes = []

    def start(service, port):
        process = subprocess.Popen(
            [str(EXECUTABLE), service, str(tmp_path), "127.0.0.1", str(port)],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        processes.append(process)
        return process

    yield start
    for process in processes:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        if process.stderr:
            process.stderr.close()


def connect_http(port, process):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise AssertionError(f"HTTP server exited: {process.stderr.read()!r}")
        try:
            return socket.create_connection(("127.0.0.1", port), timeout=0.2)
        except OSError:
            time.sleep(0.05)
    raise AssertionError("HTTP server did not start")


def test_http_ranges_keepalive_head_and_confinement(tmp_path, launch):
    (tmp_path / "image").write_bytes(b"0123456789")
    outside = tmp_path.parent / (tmp_path.name + "-outside")
    outside.write_bytes(b"secret")
    try:
        (tmp_path / "escape").symlink_to(outside)
    except (OSError, NotImplementedError):
        pass
    port = free_port(socket.SOCK_STREAM)
    process = launch("http", port)
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
    connection.sock = connect_http(port, process)
    try:
        connection.request("HEAD", "/image")
        response = connection.getresponse()
        assert response.status == 200 and response.getheader("Content-Length") == "10"
        assert response.read() == b""
        connection.request("GET", "/image", headers={"Range": "bytes=2-4"})
        response = connection.getresponse()
        assert response.status == 206 and response.getheader("Content-Range") == "bytes 2-4/10"
        assert response.read() == b"234"
        connection.request("GET", "/image", headers={"Range": "bytes=20-"})
        response = connection.getresponse()
        assert response.status == 416 and response.read() == b""
        connection.request("GET", "/escape")
        response = connection.getresponse()
        assert response.status == 404
        response.read()
    finally:
        connection.close()
        outside.unlink()


def test_tftp_oack_window_empty_tail_and_read_only(tmp_path, launch):
    (tmp_path / "image").write_bytes(b"12345678")
    port = free_port(socket.SOCK_DGRAM)
    process = launch("tftp", port)
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as client:
        client.settimeout(0.25)
        request = b"\x00\x01image\x00octet\x00blksize\x008\x00windowsize\x002\x00tsize\x000\x00"
        deadline = time.monotonic() + 30
        while True:
            if process.poll() is not None:
                raise AssertionError(f"TFTP server exited: {process.stderr.read()!r}")
            client.sendto(request, ("127.0.0.1", port))
            try:
                data, peer = client.recvfrom(1024)
                break
            except TimeoutError:
                if time.monotonic() >= deadline:
                    raise AssertionError("TFTP server did not start") from None
        assert data[:2] == b"\x00\x06" and b"tsize\x008\x00" in data
        assert peer[1] != port
        client.sendto(b"\x00\x04\x00\x00", peer)
        client.settimeout(5)
        first, source = client.recvfrom(1024)
        last, _ = client.recvfrom(1024)
        assert source == peer and first == b"\x00\x03\x00\x0112345678"
        assert last == b"\x00\x03\x00\x02"
        client.sendto(b"\x00\x04\x00\x02", peer)
        client.sendto(b"\x00\x02image\x00octet\x00", ("127.0.0.1", port))
        error, _ = client.recvfrom(1024)
        assert error[:4] == b"\x00\x05\x00\x02"
        assert (tmp_path / "image").read_bytes() == b"12345678"
