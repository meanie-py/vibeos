#!/usr/bin/env python3
"""External diagnostic-process fixture; Native Vm still owns launch and lifetime."""

import json
import os
import sys
import time
from pathlib import Path

request = json.load(sys.stdin)
op = sys.argv[-1]
mode = os.environ.get("DIAG_MODE", "none")
if op == "vm_diagnostic_prepare":
    value = {"argv": request["argv"], "sidecars": []}
    if mode in ("break", "breakfire"):
        value["rr_break_icount"] = 200
    if mode == "budgets":
        value.update(boot_timeout_min=1800, hang_silence_min=600)
    if mode in ("finish", "finish-fail"):
        value["sidecars"] = [{"kind": "pc"}, {"kind": "rr", "value": "check"}]
    if mode == "two-results":
        value["sidecars"] = [{"kind": "pc"}, {"kind": "rr"}]
    if mode in ("fail", "hold"):
        value["sidecars"] = [{"kind": "fixture"}]
    print(json.dumps({"ok": value}), flush=True)
else:
    Path(os.environ["DIAG_PID"]).write_text(str(os.getpid()))
    if mode in ("finish", "finish-fail") and request["kind"] == "rr":
        time.sleep(0.3)
        Path(os.environ["DIAG_FINISHED"]).write_text("complete")
        if mode == "finish-fail":
            print(
                json.dumps(
                    {"error": {"kind": "FixtureError", "message": "controlled completion failure"}}
                ),
                flush=True,
            )
            raise SystemExit(7)
        print(json.dumps({"ok": {"compared": 1}}), flush=True)
        raise SystemExit(0)
    if mode == "two-results":
        # Different result lengths expose sharing a file offset between workers.
        if request["kind"] == "rr":
            time.sleep(0.2)
        print(
            json.dumps(
                {
                    "ok": {
                        "kind": request["kind"],
                        "samples": 1 if request["kind"] == "rr" else 123456,
                    }
                }
            ),
            flush=True,
        )
        raise SystemExit(0)
    if mode == "fail":
        print("diagnostic sentinel", file=sys.stderr, flush=True)
        print(
            json.dumps({"error": {"kind": "FixtureError", "message": "diagnostic sentinel"}}),
            flush=True,
        )
        raise SystemExit(7)
    while True:
        time.sleep(0.05)
