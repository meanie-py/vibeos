"""Serial output reader over QEMU Unix socket.

We act as the server: create the Unix socket, listen, then QEMU connects
to us when it starts. This guarantees we capture ALL serial output from
the very first byte — no race between QEMU boot and our connect.
"""

from __future__ import annotations

import os
import select
import socket
import time


class SerialClient:
    """Read serial output from QEMU via a Unix socket we own.

    Usage:
        serial = SerialClient()
        serial.listen("/tmp/ser.sock")   # Create server socket BEFORE starting QEMU
        # ... start QEMU with -chardev socket,path=/tmp/ser.sock,server=off ...
        serial.accept(timeout=30)        # Wait for QEMU to connect
        output = serial.read_until("READY", timeout=60)
    """

    def __init__(self) -> None:
        self._server: socket.socket | None = None
        self._conn: socket.socket | None = None
        self.partial_output: str = ""  # Last partial buffer on timeout

    def listen(self, path: str) -> None:
        """Create Unix socket server. Call BEFORE starting QEMU."""
        # Remove stale socket file
        try:
            os.unlink(path)
        except FileNotFoundError:
            pass

        self._server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self._server.bind(path)
        self._server.listen(1)
        self._server.setblocking(False)

    def accept(self, timeout: float = 30.0) -> None:
        """Wait for QEMU to connect to our socket."""
        assert self._server is not None
        deadline = time.monotonic() + timeout
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(f"QEMU did not connect to serial socket within {timeout}s")
            ready, _, _ = select.select([self._server], [], [], min(remaining, 0.5))
            if ready:
                conn, _ = self._server.accept()
                conn.setblocking(False)
                self._conn = conn
                return

    def read_until(self, marker: str, timeout: float = 60.0) -> str:
        """Read serial output until marker string appears.

        Returns all output read (including text after the marker on the same line).
        Raises TimeoutError if marker not seen within timeout.
        Auto-continues past kdb debugger prompts (assertion failures, breakpoints).
        """
        assert self._conn is not None
        buf = ""
        deadline = time.monotonic() + timeout

        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                self.partial_output = buf
                raise TimeoutError(
                    f"Timed out waiting for marker {marker!r} after {timeout}s "
                    f"({len(buf)} bytes captured). "
                    f"First 500: {buf[:500]!r} ... Last 500: {buf[-500:]!r}"
                )

            ready, _, _ = select.select([self._conn], [], [], min(remaining, 1.0))
            if ready:
                try:
                    chunk = self._conn.recv(4096)
                except BlockingIOError:
                    continue
                if not chunk:
                    raise ConnectionError("Serial socket closed")
                buf += chunk.decode("latin-1")

                # Auto-continue past kdb debugger prompts and backtrace pagination
                tail = buf[-(len(chunk.decode("latin-1")) + 80):]
                if "kdb:>" in tail:
                    self._conn.sendall(b"cont\n")
                elif "any other key to continue ---" in tail:
                    self._conn.sendall(b" ")

                if marker in buf:
                    return buf

    def read_until_idle(self, idle_seconds: float = 5.0, timeout: float = 120.0,
                        min_bytes: int = 1024) -> str:
        """Read serial output until it goes idle (no new data for idle_seconds).

        Requires at least min_bytes before considering idle detection.
        Returns all captured output. Raises TimeoutError on overall timeout.
        """
        assert self._conn is not None
        buf = ""
        deadline = time.monotonic() + timeout
        last_data_time = time.monotonic()

        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(
                    f"Timed out waiting for idle after {timeout}s "
                    f"({len(buf)} bytes captured). "
                    f"First 500: {buf[:500]!r} ... Last 500: {buf[-500:]!r}"
                )

            # Check if we've been idle long enough
            since_last = time.monotonic() - last_data_time
            if len(buf) >= min_bytes and since_last >= idle_seconds:
                return buf

            wait_time = min(remaining, idle_seconds - since_last + 0.1)
            ready, _, _ = select.select([self._conn], [], [], max(wait_time, 0.1))
            if ready:
                try:
                    chunk = self._conn.recv(4096)
                except BlockingIOError:
                    continue
                if not chunk:
                    raise ConnectionError("Serial socket closed")
                buf += chunk.decode("latin-1")
                last_data_time = time.monotonic()

    def read_available(self, timeout: float = 0.1) -> str:
        """Read any available data with a short wait. Returns empty string if nothing."""
        assert self._conn is not None
        buf = ""
        deadline = time.monotonic() + timeout

        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            ready, _, _ = select.select([self._conn], [], [], min(remaining, 0.05))
            if ready:
                try:
                    chunk = self._conn.recv(4096)
                except BlockingIOError:
                    break
                if not chunk:
                    break
                buf += chunk.decode("latin-1")
            else:
                if buf:
                    break

        return buf

    def send(self, data: bytes) -> None:
        """Send data to serial port (for future interactive tests)."""
        assert self._conn is not None
        self._conn.sendall(data)

    def close(self) -> None:
        """Close connection and server sockets."""
        for s in (self._conn, self._server):
            if s:
                try:
                    s.close()
                except OSError:
                    pass
        self._conn = None
        self._server = None
