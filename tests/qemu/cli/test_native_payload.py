"""Shipping native payload policies exercised through real adapter I/O fixtures."""

from __future__ import annotations

import contextlib
import socketserver
import threading
from pathlib import Path
from types import SimpleNamespace
from typing import Any, cast

import pytest

from tests.qemu import test_payload as payload


class VM:
    def __init__(self, root: Path, pixels=()):
        self.work_dir = self.artifact_dir = root
        self._boot_output = "done VERDICT=PASS\n"
        self.pixels = iter(pixels)
        self.commands = []
        self.qmp = self
        self.running = True
        self.sleeps = []
        self.config = type("Config", (), {"payload_hostfwd_port": 0, "echo_hostfwd_port": 0})()
        self.net_host = "127.0.0.1"

    def screendump(self, path):
        Path(path).write_bytes(b"P6\n1 1\n255\n" + bytes([next(self.pixels)] * 3))

    def execute(self, command, **arguments):
        self.commands.append((command, arguments))
        return {"return": {"running": self.running}}


@pytest.fixture
def vm(tmp_path, monkeypatch):
    instance = VM(tmp_path)
    monkeypatch.setattr(payload, "time", SimpleNamespace(sleep=instance.sleeps.append))
    return instance


def test_render_gate_reads_the_captured_pixels_and_refuses_a_static_frame(vm):
    spec = {"test": {"renders": {"settle": 3, "gap": 2}}}
    vm.pixels = iter([0, 255])
    payload.test_payload_renders(vm, "amd64", "demo", spec, cast(Any, None))
    assert vm.sleeps == [3, 2]
    vm.pixels = iter([0, 0])
    with pytest.raises(AssertionError, match="nothing is animating"):
        payload.test_payload_renders(vm, "amd64", "demo", spec, cast(Any, None))


def test_input_gate_sends_down_up_events_and_checks_survival(vm):
    vm.pixels = iter([0, 255, 0, 0])
    spec = {"test": {"input": {"drive": [{"keys": ["esc", "ret"]}], "settle": 0}}}
    payload.test_payload_input_reaches_app(vm, "amd64", "demo", spec, cast(Any, None))
    keys = [
        a["events"][0]["data"]
        for op, a in vm.commands
        if op == "input-send-event" and a["events"][0]["type"] == "key"
    ]
    assert [k["key"]["data"] for k in keys] == ["esc", "esc", "ret", "ret"]
    assert [k["down"] for k in keys] == [True, False, True, False]
    assert vm.commands[-1][0] == "query-status"
    assert len([op for op, _ in vm.commands if op == "input-send-event"]) == 41


def test_input_retry_uses_on_retry_and_failure_captures_evidence(vm, monkeypatch):
    vm.pixels = iter([0, 255, 0, 255, 0, 0])
    spec = {
        "test": {
            "input": {
                "focus": "none",
                "drive": [{"keys": ["ret"]}],
                "on_retry": [{"keys": ["esc"]}],
                "retries": 2,
            }
        }
    }
    payload.test_payload_input_reaches_app(vm, "amd64", "demo", spec, cast(Any, None))
    events = [
        a["events"][0]["data"]["key"]["data"] for op, a in vm.commands if op == "input-send-event"
    ]
    assert events == ["ret", "ret", "esc", "esc", "ret", "ret"]
    vm.running = False
    vm.pixels = iter([0, 255, 0, 0])
    with pytest.raises(AssertionError, match="no longer running"):
        payload.test_payload_input_reaches_app(vm, "amd64", "demo", spec, cast(Any, None))


def test_unknown_drive_and_qmp_errors_are_loud(vm):
    with pytest.raises(AssertionError, match="unknown drive"):
        payload._run_drive(vm, [{"typo": 1}])
    vm.execute = lambda *_a, **_kw: {"error": {"desc": "input rejected"}}
    with pytest.raises(AssertionError, match="input rejected"):
        payload._send_key(vm, "ret")


def test_launch_honors_the_payload_own_verdict_and_rejects_unknown_graders(vm):
    payload.test_payload_launched(
        vm, "amd64", "demo", {"test": {"launch_marker": "done", "pass_marker": "VERDICT=PASS"}}
    )
    with pytest.raises(AssertionError, match="lacks"):
        payload.test_payload_launched(
            vm, "amd64", "demo", {"test": {"launch_marker": "done", "pass_marker": "other verdict"}}
        )
    with pytest.raises(ValueError, match="unknown payload grader"):
        payload.test_payload_launched(
            vm, "amd64", "demo", {"test": {"launch_marker": "done", "grader": "typo.py"}}
        )


class HTTPHandler(socketserver.StreamRequestHandler):
    def handle(self):
        path = self.rfile.readline().split()[1].decode()
        while self.rfile.readline() not in (b"\r\n", b"\n", b""):
            pass
        state = cast(Any, self.server).state
        with state["lock"]:
            state["requests"] += 1
            requests = state["requests"]
        if path == "/status":
            body = f"requests={0 if state['counter_wrong'] else requests} alive=1".encode()
        elif path.startswith("/data"):
            size = int(path.split("=")[1])
            body = bytes(((i * 7 + 13) & 255) for i in range(size))
            if state["corrupt"]:
                body = b"wrong"
        else:
            body = b"hello"
        self.wfile.write(b"HTTP/1.0 200 OK\r\n\r\n" + body)


@contextlib.contextmanager
def server():
    class Server(socketserver.ThreadingTCPServer):
        allow_reuse_address = True
        daemon_threads = True
        state: dict[str, Any]

    with Server(("127.0.0.1", 0), HTTPHandler) as instance:
        instance.state = {
            "requests": 0,
            "lock": threading.Lock(),
            "corrupt": False,
            "counter_wrong": False,
        }
        worker = threading.Thread(target=instance.serve_forever, daemon=True)
        worker.start()
        try:
            yield instance
        finally:
            instance.shutdown()
            worker.join(timeout=5)


def test_hammer_checks_bytes_counters_and_guest_survival(vm):
    spec = {
        "test": {
            "net": True,
            "tcp_hammer": {"threads": 3, "requests": 5, "data_sizes": [1, 512, 4096]},
        }
    }
    with server() as instance:
        vm.config.payload_hostfwd_port = instance.server_address[1]
        payload.test_payload_tcp_hammer(vm, "amd64-net", "httpd", spec)
        assert instance.state["requests"] == 16
        assert vm.commands[-1][0] == "query-status"
        instance.state["corrupt"] = True
        with pytest.raises(AssertionError, match="body mismatch"):
            payload.test_payload_tcp_hammer(vm, "amd64-net", "httpd", spec)
        instance.state["corrupt"] = False
        instance.state["counter_wrong"] = True
        with pytest.raises(AssertionError, match="phantom successes"):
            payload.test_payload_tcp_hammer(vm, "amd64-net", "httpd", spec)
