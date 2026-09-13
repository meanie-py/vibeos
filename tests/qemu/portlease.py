"""Serialize port selection through QEMU's bind handshake across worktrees.

The harness runs on Linux even when QEMU runs on Windows. Keep this lock only
until QEMU confirms initialization, never through guest boot. A lock file is
permanent: unlinking it would let another process lock a different inode.
"""

import fcntl
import json
import math
import os
import time
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path


@contextmanager
def launch_lock(*, timeout: float = 120.0) -> Iterator[None]:
    """Hold the host-wide launch handshake lock, or fail loudly on timeout.

    Each caller opens its own description, so flock excludes threads as well as
    processes. The descriptor cannot survive exec; a killed harness releases
    ownership automatically. Metadata describes the last holder for diagnosis,
    but only the kernel lock establishes ownership.
    """
    if not math.isfinite(timeout) or timeout < 0:
        raise ValueError("QEMU port handshake timeout must be finite and nonnegative")
    root = Path(os.environ.get("VIBEOS_HOSTLOCK_DIR", "/tmp/vibeos-qemu-locks/host"))
    root.mkdir(parents=True, exist_ok=True)
    path = root / "qemu-port-handshake.lock"
    deadline = time.monotonic() + timeout
    with path.open("a+", encoding="utf-8") as lock:
        while True:
            try:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    lock.seek(0)
                    owner = lock.read(4096).strip() or "owner metadata unavailable"
                    raise TimeoutError(
                        f"QEMU port handshake timed out after {timeout:g}s waiting for "
                        f"{path}; last holder: {owner}"
                    ) from None
                time.sleep(min(0.025, remaining))
        try:
            lock.seek(0)
            lock.truncate()
            json.dump({"pid": os.getpid(), "cwd": str(Path.cwd()), "since": time.time()}, lock)
            lock.flush()
            yield
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
