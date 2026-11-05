#!/usr/bin/env python3
"""Per-UID response fixture. Never invokes PowerShell or touches Windows."""

import os
import sys
from pathlib import Path

script = sys.argv[-1]
uid = os.environ["REAPER_UID"]
# Validate the exact UID-bound selector embedded by native backend::kill_by_uid.
needle = f"-name(?:\\s+|=){uid}(?=\\s|$)"
if needle not in script:
    print("refusing unexpected fixture UID selector", file=sys.stderr)
    raise SystemExit(88)
with Path(os.environ["REAPER_CALLS"]).open("a") as stream:
    stream.write(uid + "\n")
left = os.environ.get("REAPER_LEFT", "0")
print("confirmation unavailable" if left == "unknown" else "left=" + left)
