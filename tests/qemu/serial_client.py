"""Serial output reader over QEMU Unix socket.

We act as the server: create the Unix socket, listen, then QEMU connects
to us when it starts. This guarantees we capture ALL serial output from
the very first byte — no race between QEMU boot and our connect.

CRITICAL (2026-05-16): a *lifetime background reader thread* drains the
socket continuously. ReactOS debug builds stream DbgPrint to a *polled*
16550 UART (the kernel busy-waits on LSR THRE per byte). If we only read
the socket during read_until()/read_available() calls, then between calls
QEMU back-pressures the full socket, THRE never clears, and the guest
spins forever in its serial driver — the vCPU pegs ~92% and every
post-boot test runs on a 1/20th-speed starved guest (this masqueraded as
a fake cross-arch "idle CPU storm"). The reader thread must NEVER stop
until close(), so the socket is always drained and the guest never stalls.
The public read_* methods just consume from the in-memory buffer.
"""

from __future__ import annotations

import os
import select
import socket
import threading
import time
from collections.abc import Callable


class SerialClient:
    """Read serial output from QEMU via a Unix socket we own.

    Usage:
        serial = SerialClient()
        serial.listen("/tmp/ser.sock")   # Create server socket BEFORE starting QEMU
        # ... start QEMU with -chardev socket,path=/tmp/ser.sock,server=off ...
        serial.accept(timeout=30)        # Wait for QEMU to connect (starts drainer)
        output = serial.read_until("READY", timeout=60)
    """

    def __init__(self) -> None:
        self._server: socket.socket | None = None
        self._conn: socket.socket | None = None
        self.partial_output: str = ""  # Last partial buffer on timeout

        # Shared buffer filled by the background reader thread.
        self._buf = ""
        self._pos = 0  # consumed offset (per read_* progress)
        self._last_data = time.monotonic()
        self._lock = threading.Lock()
        self._closed = False
        self._reader: threading.Thread | None = None
        self._sock_dead = False  # set when peer closes

        # Optional live-log tee. When set, the drainer appends every byte it
        # receives to this file handle and flushes immediately, so an agent can
        # `tail -f` boot progress in real time instead of guessing whether a
        # long TCG/cold-cache boot is making progress or wedged. Strictly a
        # side-channel: a write failure here must NEVER stall the drainer (see
        # the class docstring — a stalled drainer re-starves the polled guest).
        self._tee = None
        # VIBEOS_SERIAL_TIMESTAMPS=1 prefixes every tee'd line with the elapsed
        # seconds since the first serial byte ("[+ 12.345] ..."), turning any
        # live log into a boot-phase timing trace (tools/boot_phase_report.py
        # buckets it into firmware/freeldr/kernel/... phases). Tee-only: the
        # in-memory buffer that markers match against is never modified.
        self._tee_stamp = os.environ.get("VIBEOS_SERIAL_TIMESTAMPS", "") == "1"
        self._tee_t0: float | None = None
        self._tee_at_bol = True
        # Record/replay reads the guest's serial from a FILE QEMU writes (see
        # follow_file); None means the ordinary socket transport.
        self._file_path: str | None = None
        self._alive: Callable[[], bool] | None = None
        self._fd: int | None = None

    def listen(self, path: str) -> None:
        """Create Unix socket server. Call BEFORE starting QEMU."""
        try:
            os.unlink(path)
        except FileNotFoundError:
            pass

        self._server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self._server.bind(path)
        self._server.listen(1)
        self._server.setblocking(False)

    def listen_tcp(self, host: str = "127.0.0.1", port: int = 0) -> int:
        """Listen on a TCP port instead of a Unix socket (windows-host backend:
        the Windows QEMU connects back to us over WSL2 localhost forwarding).
        Returns the actually-bound port (pass 0 to let the OS pick). Call BEFORE
        starting QEMU. accept()/drain are transport-agnostic from here on."""
        self._server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._server.bind((host, port))
        self._server.listen(1)
        bound = self._server.getsockname()[1]
        self._server.setblocking(False)
        return bound

    def follow_file(self, path: str, alive: Callable[[], bool]) -> None:
        """Read the guest's serial from a regular file QEMU writes, not a socket.

        For record/replay only. A socket can refuse a write (EAGAIN) whenever this
        reader lags, and AF_UNIX stream charges every one-byte write a whole skb
        of buffer, so a short stall is enough. x86's 16550 then retries the byte
        from a HOST watch callback — at a moment the replay log cannot pin — and
        the replay later dies with "Missing character write event". Measured:
        every failing x86 replay had logged writes with res=-1, its first one at
        exactly the failing write; the clean traces had none. A regular file
        never refuses a write.

        `alive` reports whether QEMU still runs: a file has no EOF from a dead
        writer, so this is how the drainer learns the guest is gone. Call before
        accept(). The serial is output-only in this mode (see send()).
        """
        self._file_path = path
        self._alive = alive

    def set_tee(self, path: str, header: str = "") -> None:
        """Mirror all received serial bytes to `path` in real time (truncates).

        Call BEFORE accept(). The drainer flushes per chunk so an external
        `tail -f` sees boot progress live. Best-effort: if the file can't be
        opened the tee is silently disabled — it must never break serial drain.
        """
        try:
            # Held for the client's lifetime and closed in close(), so no context manager.
            self._tee = open(path, "w", encoding="latin-1", errors="replace", buffering=1)  # noqa: SIM115
            if header:
                self._tee.write(header)
                self._tee.flush()
        except OSError:
            self._tee = None

    def accept(self, timeout: float = 30.0) -> None:
        """Wait for QEMU to connect, then start the lifetime drainer thread."""
        if self._file_path is not None:
            self._accept_file(timeout)
            return
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
                self._reader = threading.Thread(
                    target=self._drain_loop, name="serial-drainer", daemon=True
                )
                self._reader.start()
                return

    def _accept_file(self, timeout: float) -> None:
        assert self._file_path is not None
        deadline = time.monotonic() + timeout
        while not os.path.exists(self._file_path):
            if time.monotonic() > deadline:
                raise TimeoutError(f"QEMU did not create {self._file_path} within {timeout}s")
            time.sleep(0.05)
        self._fd = os.open(self._file_path, os.O_RDONLY)
        self._reader = threading.Thread(
            target=self._file_drain_loop, name="serial-file-drainer", daemon=True
        )
        self._reader.start()

    def _file_drain_loop(self) -> None:
        fd, alive = self._fd, self._alive
        assert fd is not None and alive is not None
        while not self._closed:
            try:
                chunk = os.read(fd, 65536)
            except OSError:
                time.sleep(0.05)
                continue
            if chunk:
                self._ingest(chunk.decode("latin-1"))
                continue
            if not alive():
                # One last read: QEMU may have written its final bytes between
                # the empty read above and exiting.
                tail = os.read(fd, 1 << 20)
                if tail:
                    self._ingest(tail.decode("latin-1"))
                self._sock_dead = True
                break
            time.sleep(0.05)

    def _ingest(self, text: str) -> None:
        with self._lock:
            self._buf += text
            self._last_data = time.monotonic()
        # Live-log tee (side-channel, never allowed to break the drain).
        if self._tee is not None:
            try:
                self._tee.write(self._stamp_tee_text(text) if self._tee_stamp else text)
                self._tee.flush()
            except (OSError, ValueError):
                pass

    # ------------------------------------------------------------------ #
    # Background drainer — the load-bearing part. Always reading.
    # ------------------------------------------------------------------ #
    def _drain_loop(self) -> None:
        conn = self._conn
        assert conn is not None
        # NEVER break on a transient error: if this thread dies the socket
        # stops draining, QEMU back-pressures, and the polled-serial guest
        # re-starves (the exact bug this whole class exists to prevent).
        # Only stop on close() or a genuine peer EOF.
        while not self._closed:
            try:
                ready, _, _ = select.select([conn], [], [], 0.2)
            except (OSError, ValueError):
                if self._closed:
                    break
                time.sleep(0.05)
                continue
            if not ready:
                continue
            try:
                chunk = conn.recv(65536)
            except BlockingIOError:
                continue
            except OSError:
                if self._closed:
                    break
                time.sleep(0.05)
                continue
            if not chunk:
                self._sock_dead = True
                break
            text = chunk.decode("latin-1")
            self._ingest(text)
            # Keep the guest's kernel debugger / pager from wedging boot.
            tail = self._buf[-(len(text) + 80) :]
            try:
                if "kdb:>" in tail:
                    conn.sendall(b"cont\n")
                elif "any other key to continue ---" in tail:
                    conn.sendall(b" ")
            except OSError:
                pass

    def _stamp_tee_text(self, text: str) -> str:
        """Prefix each line-start in `text` with elapsed-seconds since the
        first serial byte. Only used for the tee file; cheap string work."""
        now = time.monotonic()
        if self._tee_t0 is None:
            self._tee_t0 = now
        stamp = f"[+{now - self._tee_t0:9.3f}] "
        out = []
        for ch_line in text.splitlines(keepends=True):
            if self._tee_at_bol:
                out.append(stamp)
            out.append(ch_line)
            self._tee_at_bol = ch_line.endswith(("\n", "\r"))
        return "".join(out)

    def _snapshot(self) -> tuple[str, float, bool]:
        with self._lock:
            return self._buf, self._last_data, self._sock_dead

    # ------------------------------------------------------------------ #
    # Public read API — unchanged signatures; now consume from the buffer.
    # ------------------------------------------------------------------ #
    def read_until(self, marker: str, timeout: float = 60.0) -> str:
        """Read buffered serial output until `marker` appears (from the
        current consume position). Returns everything from that position
        through the marker. Raises TimeoutError if not seen in `timeout`s.
        """
        start = self._pos
        deadline = time.monotonic() + timeout
        while True:
            buf, _, dead = self._snapshot()
            window = buf[start:]
            idx = window.find(marker)
            if idx >= 0:
                self._pos = start + idx + len(marker)
                return window[: idx + len(marker)]
            if dead:
                self.partial_output = window
                raise ConnectionError("Serial socket closed")
            if time.monotonic() >= deadline:
                self.partial_output = window
                raise TimeoutError(
                    f"Timed out waiting for marker {marker!r} after {timeout}s "
                    f"({len(window)} bytes captured). "
                    f"First 500: {window[:500]!r} ... Last 500: {window[-500:]!r}"
                )
            time.sleep(0.05)

    def read_until_idle(
        self, idle_seconds: float = 5.0, timeout: float = 120.0, min_bytes: int = 1024
    ) -> str:
        """Return buffered output once no new bytes arrive for `idle_seconds`
        (and at least `min_bytes` seen). Raises TimeoutError on overall timeout.
        """
        start = self._pos
        deadline = time.monotonic() + timeout
        while True:
            buf, last, dead = self._snapshot()
            window = buf[start:]
            if len(window) >= min_bytes and (time.monotonic() - last) >= idle_seconds:
                self._pos = len(buf)
                return window
            if dead and window:
                self._pos = len(buf)
                return window
            if time.monotonic() >= deadline:
                self.partial_output = window
                raise TimeoutError(
                    f"Timed out waiting for idle after {timeout}s "
                    f"({len(window)} bytes captured). "
                    f"First 500: {window[:500]!r} ... Last 500: {window[-500:]!r}"
                )
            time.sleep(0.1)

    def read_available(self, timeout: float = 0.1) -> str:
        """Return any buffered output since the last consume; waits up to
        `timeout`s for at least something to show up."""
        deadline = time.monotonic() + timeout
        while True:
            buf, _, _ = self._snapshot()
            if len(buf) > self._pos:
                out = buf[self._pos :]
                self._pos = len(buf)
                return out
            if time.monotonic() >= deadline:
                return ""
            time.sleep(0.02)

    def send(self, data: bytes) -> None:
        """Send data to serial port (interactive input)."""
        if self._file_path is not None:
            raise OSError("serial input is unavailable under record/replay (file transport)")
        assert self._conn is not None
        self._conn.sendall(data)

    def close(self) -> None:
        """Stop the drainer and close sockets."""
        self._closed = True
        if self._reader is not None:
            self._reader.join(timeout=2.0)
            self._reader = None
        for s in (self._conn, self._server):
            if s:
                try:
                    s.close()
                except OSError:
                    pass
        self._conn = None
        self._server = None
        if self._fd is not None:
            try:
                os.close(self._fd)
            except OSError:
                pass
            self._fd = None
        if self._tee is not None:
            try:
                self._tee.close()
            except OSError:
                pass
            self._tee = None
