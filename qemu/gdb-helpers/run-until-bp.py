# bounded-continue <secs>: `continue` the inferior, but give up cleanly after
# <secs> if no breakpoint is hit.
#
# Why this exists: a bare batch-mode `continue` blocks forever waiting for the
# breakpoint. run-until.sh used to wrap the whole gdb process in a host-side
# `timeout`, which SIGTERMs gdb *mid-continue* when the breakpoint never fires —
# leaving the target running, so every queued `--then`/cleanup command then died
# with "Cannot execute this command while the target is running" plus a Python
# KeyboardInterrupt traceback. Confusing, and indistinguishable from a real bug.
#
# Instead we bound the wait from inside gdb: a daemon timer raises SIGINT on
# gdb's own pid after <secs>, which interrupts `continue` exactly like a Ctrl-C
# and returns control with the target cleanly STOPPED. We then report whether a
# breakpoint actually hit, so the caller can tell "hit" from "timed out" and all
# subsequent commands run against a stopped (not running) target.

import gdb
import os
import signal
import threading


class BoundedContinueCmd(gdb.Command):
    """bounded-continue <secs>: continue, but interrupt cleanly after <secs>."""

    def __init__(self):
        super().__init__("bounded-continue", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        try:
            secs = float(arg.strip()) if arg.strip() else 120.0
        except ValueError:
            secs = 120.0

        hit = {"v": False}

        def on_stop(evt):
            # A BreakpointEvent means we stopped because a breakpoint fired,
            # not because our timer SIGINT'd us.
            if isinstance(evt, gdb.BreakpointEvent):
                hit["v"] = True

        gdb.events.stop.connect(on_stop)
        # Raising SIGINT on our own pid is exactly what Ctrl-C does: it unblocks
        # the synchronous `continue` and stops the target. A plain timeout-kill
        # of the whole gdb cannot do this gracefully.
        timer = threading.Timer(secs, lambda: os.kill(os.getpid(), signal.SIGINT))
        timer.daemon = True
        timer.start()
        try:
            gdb.execute("continue")
        except (gdb.error, KeyboardInterrupt):
            # SIGINT during continue surfaces here in some gdb builds; the target
            # is stopped regardless, which is all we need.
            pass
        finally:
            timer.cancel()
            try:
                gdb.events.stop.disconnect(on_stop)
            except Exception:
                pass

        if hit["v"]:
            gdb.write("\n=== BREAKPOINT HIT ===\n")
        else:
            gdb.write(
                f"\n=== BREAKPOINT NOT HIT (timed out after {secs:.0f}s; "
                f"target interrupted, now stopped) ===\n"
            )


BoundedContinueCmd()
