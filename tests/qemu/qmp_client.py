"""QEMU Machine Protocol (QMP) client over Unix socket."""

from __future__ import annotations

import json
import socket
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
        self._buf = b""

    def connect(self, path: str, timeout: float = 10.0) -> dict:
        """Connect and complete QMP handshake (greeting + qmp_capabilities)."""
        self._sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self._sock.settimeout(timeout)
        self._sock.connect(path)

        # Read greeting
        greeting = self._recv_json()

        # Send qmp_capabilities to enter command mode
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
        self._send_json(msg)
        resp = self._recv_json()
        if "error" in resp:
            raise QmpError(command, resp["error"])
        return resp

    def hmp(self, command: str) -> str:
        """Execute HMP command via human-monitor-command passthrough."""
        resp = self.execute("human-monitor-command", **{"command-line": command})
        return resp.get("return", "")

    def close(self) -> None:
        """Close socket."""
        if self._sock:
            try:
                self._sock.close()
            except OSError:
                pass
            self._sock = None

    def _send_json(self, obj: dict) -> None:
        assert self._sock is not None
        data = json.dumps(obj).encode() + b"\n"
        self._sock.sendall(data)

    def _recv_json(self) -> dict:
        """Read one JSON object from the socket, skipping QMP events."""
        while True:
            obj = self._recv_one_json()
            # Skip asynchronous events (they have "event" key)
            if "event" not in obj:
                return obj

    def _recv_one_json(self) -> dict:
        """Read exactly one JSON object from the socket."""
        assert self._sock is not None
        while True:
            # Try to parse a complete JSON object from buffer
            nl = self._buf.find(b"\n")
            if nl >= 0:
                line = self._buf[:nl]
                self._buf = self._buf[nl + 1:]
                if line.strip():
                    return json.loads(line)
                continue

            # Need more data
            chunk = self._sock.recv(4096)
            if not chunk:
                raise ConnectionError("QMP socket closed")
            self._buf += chunk
