"""QEMU Machine Protocol (QMP) client over Unix socket."""

from __future__ import annotations

import json
import socket
import threading
import time
from collections import deque
from typing import Any


class QmpError(Exception):
    """QMP command returned an error."""

    def __init__(self, command: str, error: dict):
        self.command = command
        self.error = error
        super().__init__(f"QMP {command}: {error.get('class', '?')}: {error.get('desc', '?')}")


class QmpClient:
    """QEMU Machine Protocol client over Unix socket.

    QMP is a JSON-based protocol. Each message is a single JSON object
    terminated by newline. The server sends a greeting on connect, then
    the client must send qmp_capabilities to enter command mode.
    """

    def __init__(self) -> None:
        self._sock: socket.socket | None = None
        self._server: socket.socket | None = None
        self._buf = b""
        # QMP events, kept instead of dropped. _recv_json has to skip them to find
        # a command's reply, and it used to discard them on the way past — which
        # silently threw away the only notification QEMU sends for things nobody
        # asked about, STOP among them.
        #
        # That cost a real answer: a record/replay `replay-break` stops the guest
        # and announces it with STOP, and with the event discarded a replay paused
        # at a breakpoint is indistinguishable from a replay wedged, because both
        # present as a frozen serial log and a busy CPU. Keeping them lets a caller
        # ask what happened rather than infer it from where output stopped.
        #
        # Bounded: a long boot emits plenty, and this is a diagnostic tail, not a
        # log. Callers that need every event should consume as they go.
        self.events: deque[dict] = deque(maxlen=256)
        # One command is a send + its reply; a second thread interleaving there
        # reads the other's answer. Re-entrant so a caller can hold it across a
        # multi-command sequence that depends on monitor state (`cpu N` then `x`),
        # which is what the PC sampler and the frame walk both do.
        self.lock = threading.RLock()

    def connect(self, path: str, timeout: float = 10.0) -> dict:
        """Connect and complete QMP handshake (greeting + qmp_capabilities).

        Native path: we are the TCP/Unix client connecting to QEMU's QMP server
        (`-qmp unix:...,server`)."""
        self._sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self._sock.settimeout(timeout)
        self._sock.connect(path)
        return self._handshake()

    def listen_tcp(self, host: str = "127.0.0.1", port: int = 0) -> int:
        """Listen on a TCP port for QEMU to connect to us (windows-host backend:
        QEMU's QMP chardev is `server=off`, connecting back over WSL2 localhost
        forwarding). Returns the bound port. Call BEFORE starting QEMU; then
        accept() once QEMU is up. QEMU is still the QMP protocol server (it sends
        the greeting), regardless of which side opened the TCP connection."""
        self._server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._server.bind((host, port))
        self._server.listen(1)
        return self._server.getsockname()[1]

    def accept(self, timeout: float = 30.0) -> dict:
        """Accept QEMU's QMP connection (after listen_tcp) and complete the
        handshake. Returns the greeting."""
        assert self._server is not None, "call listen_tcp() before accept()"
        self._server.settimeout(timeout)
        conn, _ = self._server.accept()
        conn.settimeout(timeout)
        self._sock = conn
        return self._handshake()

    def _handshake(self) -> dict:
        """Read the QMP greeting and enter command mode (qmp_capabilities)."""
        greeting = self._recv_json()
        self._send_json({"execute": "qmp_capabilities"})
        resp = self._recv_json()
        if "error" in resp:
            raise QmpError("qmp_capabilities", resp["error"])
        return greeting

    def execute(self, command: str, **args: Any) -> dict:
        """Send QMP command, return response. Raises QmpError on error."""
        msg: dict[str, Any] = {"execute": command}
        if args:
            msg["arguments"] = args
        with self.lock:
            self._send_json(msg)
            resp = self._recv_json()
        if "error" in resp:
            raise QmpError(command, resp["error"])
        return resp

    def hmp(self, command: str) -> str:
        """Execute HMP command via human-monitor-command passthrough."""
        resp = self.execute("human-monitor-command", **{"command-line": command})
        return resp.get("return", "")

    def saw_event(self, name: str) -> dict | None:
        """The most recent retained event called `name`, or None.

        Answers "did the guest stop?" without the caller reaching into a deque and
        guessing at QMP's schema. Only as far back as the retention bound, so a
        None means "not in the recent tail", not "never happened".
        """
        for obj in reversed(self.events):
            if obj.get("event") == name:
                return obj
        return None

    def wait_event(self, name: str, timeout: float) -> dict | None:
        """Block until QEMU sends event `name` (or one is already retained), else None.

        Only for a connection nobody else is issuing commands on: it consumes
        whatever arrives, and a command reply read here would be lost.
        """
        if (ev := self.saw_event(name)) is not None:
            return ev
        assert self._sock is not None
        deadline = time.monotonic() + timeout
        old = self._sock.gettimeout()
        try:
            while (left := deadline - time.monotonic()) > 0:
                self._sock.settimeout(min(left, 5.0))
                try:
                    obj = self._recv_one_json()
                except TimeoutError:
                    continue
                if "event" in obj:
                    self.events.append(obj)
                    if obj["event"] == name:
                        return obj
        finally:
            self._sock.settimeout(old)
        return None

    def close(self) -> None:
        """Close the connection and (if listening) the server socket."""
        for attr in ("_sock", "_server"):
            s = getattr(self, attr)
            if s:
                try:
                    s.close()
                except OSError:
                    pass
                setattr(self, attr, None)

    def _send_json(self, obj: dict) -> None:
        assert self._sock is not None
        data = json.dumps(obj).encode() + b"\n"
        self._sock.sendall(data)

    def _recv_json(self) -> dict:
        """Read one JSON object from the socket, skipping QMP events."""
        while True:
            obj = self._recv_one_json()
            # Skip asynchronous events (they have "event" key) — but keep them.
            if "event" not in obj:
                return obj
            self.events.append(obj)

    def _recv_one_json(self) -> dict:
        """Read exactly one JSON object from the socket."""
        assert self._sock is not None
        while True:
            # Try to parse a complete JSON object from buffer
            nl = self._buf.find(b"\n")
            if nl >= 0:
                line = self._buf[:nl]
                self._buf = self._buf[nl + 1 :]
                if line.strip():
                    return json.loads(line)
                continue

            # Need more data
            chunk = self._sock.recv(4096)
            if not chunk:
                raise ConnectionError("QMP socket closed")
            self._buf += chunk
