#!/usr/bin/env python3
"""Real protocol/process fixture; no guest code and no Python VM ownership."""

import json
import os
import signal
import socket
import sys
import threading
import time
from pathlib import Path

if "--version" in sys.argv:
    print("QEMU checkpoint native fixture 1")
    raise SystemExit(0)
args = sys.argv[1:]
qmp = args[args.index("-qmp") + 1].split("unix:", 1)[1].split(",", 1)[0]
serial = args[args.index("-chardev") + 1].split("path=", 1)[1].split(",", 1)[0]
events = Path(os.environ["CP_EVENTS"])


def event(name):
    with events.open("a") as f:
        f.write(name + "\n")


server = socket.socket(socket.AF_UNIX)
server.bind(qmp)
server.listen()
ser = socket.socket(socket.AF_UNIX)
ser.connect(serial)
ser.sendall(os.environ.get("CP_SERIAL", "READY\n").encode())


def monitor():
    conn, _ = server.accept()
    stream = conn.makefile("rwb", buffering=0)
    stream.write(b'{"QMP":{}}\n')
    for line in stream:
        request = json.loads(line)
        op = request["execute"]
        result = {}
        if op == "human-monitor-command":
            op = request["arguments"]["command-line"]
            if op.startswith(("savevm ", "loadvm ")):
                result = os.environ.get("CP_SNAPSHOT_ERROR", "")
            elif op == "info registers -a":
                result = os.environ.get("CP_REGISTERS", "CPU#0\nPC=1111 SP=2222")
            else:
                result = ""
        elif op == "query-status":
            result = {"running": False, "status": os.environ.get("CP_STOP_STATE", "paused")}
        elif op == "query-cpus-fast":
            result = [{"cpu-index": i} for i in range(int(os.environ.get("CP_VCPUS", "1")))]
        event(op)
        stream.write((json.dumps({"return": result}) + "\n").encode())
        if op == "quit":
            os._exit(0)


threading.Thread(target=monitor, daemon=True).start()
signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
while True:
    time.sleep(0.05)
