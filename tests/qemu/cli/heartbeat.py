"""Keep a long silent stretch talking, because silence is what gets it killed.

Background tasks on this host are reaped when they produce no output for a while.
Measured, with a control on both sides: silent + plain background died at ~8
minutes (twice, plus two builds sitting at one log line); the same launch path
emitting a line every 30s survived 36 minutes and exited normally; silent +
`setsid nohup` survived. Detachment works by removing the task from supervision,
but output is the trigger -- a supervised task that talks is spared too. Field
manual, "A gate can be RED for reasons that are not in the tree at all".

This exists because the alternative is a RULE, and a rule that says "remember to
emit output" gets broken by people who have read it: the moment you need it is the
moment you are thinking about something else. That is the `pkill` -> `vibeos-cli
kill` precedent, and AGENTS.md's own standard -- a wrapper script, or a remembered
recipe, is a bug report against the tool.

Four things here are measurements rather than taste, so don't "clean them up":

- The default interval is 30s because 30s is the PROVEN-survivor interval. The
  ~8-minute window is inferred from five samples of a supervisor none of us can
  read, so the margin is deliberate; `MAX_INTERVAL` refuses anything above 60s
  rather than letting a caller tune it close to a number we do not actually know.
- It writes to STDOUT, which is the stream the supervisor watches. A heartbeat
  into a log file or `summary.json` would be invisible to it -- the artifact is
  not what is being observed.
- It stays mid-stream. The final stdout line of a `vibeos-cli` run is reserved for
  `vibeos-cli: exit=N`, and consumers that want a salient line reverse-SEARCH for
  it rather than taking [-1] (gate.py:392 is the idiom).
- One line per 30s is ~12 lines across a 6-minute preflight, which is nothing
  against AGENTS.md's token-flooding rule -- but a per-second tick would defeat
  the reason stdout is a bounded summary. Each line carries the stage and the
  elapsed time so it is worth its space.
"""

from __future__ import annotations

import contextlib
import sys
import threading
import time
from collections.abc import Iterator
from typing import TextIO

#: The proven-survivor interval.
DEFAULT_INTERVAL = 30.0
#: Refuse to be tuned toward the observed kill window; see the module docstring.
MAX_INTERVAL = 60.0


def elapsed(seconds: float) -> str:
    s = int(seconds)
    return f"{s // 60}m{s % 60:02d}s" if s >= 60 else f"{s}s"


@contextlib.contextmanager
def beat(
    stage: str,
    *,
    label: str = "working",
    interval: float = DEFAULT_INTERVAL,
    stream: TextIO | None = None,
) -> Iterator[None]:
    """Print `[<label>] <stage> <elapsed>` to stdout while the body runs.

    The first line lands one interval in, not at t=0: a stretch that finishes
    quickly should stay silent, and only a stretch long enough to be at risk
    should cost any output at all.
    """
    if interval <= 0 or interval > MAX_INTERVAL:
        raise ValueError(f"heartbeat interval must be in (0, {MAX_INTERVAL}], got {interval}")
    out = stream if stream is not None else sys.stdout
    done = threading.Event()
    t0 = time.monotonic()

    def _tick() -> None:
        while not done.wait(interval):
            try:
                # One complete line per write, then flush: a captured pipe holds
                # a partial line forever otherwise, which is the same silence.
                out.write(f"[{label}] {stage} {elapsed(time.monotonic() - t0)}\n")
                out.flush()
            except (ValueError, OSError):
                return  # stream closed under us; the body still matters

    thread = threading.Thread(target=_tick, name=f"heartbeat:{stage}", daemon=True)
    thread.start()
    try:
        yield
    finally:
        done.set()
        thread.join(timeout=2.0)
