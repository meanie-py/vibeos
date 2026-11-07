#!/usr/bin/env python3
"""Owned process that speaks real QMP/serial, including immediate replay STOP."""

import json
import os
import signal
import socket
import sys
import threading
import time
from pathlib import Path

if "--version" in sys.argv:
    print("QEMU native diagnostic fixture 1")
    raise SystemExit(0)
args = sys.argv[1:]
qmp_paths = [
    args[i + 1].split("unix:", 1)[1].split(",", 1)[0] for i, arg in enumerate(args) if arg == "-qmp"
]
serial = args[args.index("-chardev") + 1].split("path=", 1)[1].split(",", 1)[0]
events = Path(os.environ["DIAG_EVENTS"])
servers = []
for qmp in qmp_paths:
    server = socket.socket(socket.AF_UNIX)
    server.bind(qmp)
    server.listen()
    servers.append(server)
ser = socket.socket(socket.AF_UNIX)
ser.connect(serial)
ser.sendall(b"partial progress without boot marker\n")
Path(os.environ["DIAG_QEMU_PID"]).write_text(str(os.getpid()))


def monitor(server):
    conn, _ = server.accept()
    stream = conn.makefile("rwb", buffering=0)
    stream.write(b'{"QMP":{}}\n')
    for line in stream:
        request = json.loads(line)
        with events.open("a") as f:
            f.write(json.dumps(request) + "\n")
        if request["execute"] == "replay-break" and os.environ.get("DIAG_MODE") == "breakfire":
            stream.write(b'{"event":"STOP","timestamp":{"seconds":7}}\n')
        value = {}
        if request["execute"] == "human-monitor-command":
            value = "" if os.environ.get("DIAG_EMPTY_REGS") else "CPU#0\n PC=0000000040001000\n"
        stream.write((json.dumps({"return": value}) + "\n").encode())


for server in servers:
    threading.Thread(target=monitor, args=(server,), daemon=True).start()
signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
while True:
    time.sleep(0.05)
