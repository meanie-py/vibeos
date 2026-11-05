#!/usr/bin/env python3
"""Socket fixture for native VM supervision; never boots guest code."""

import json
import os
import signal
import socket
import sys
import threading
import time
from pathlib import Path

attempts = os.environ.get("FAKE_QEMU_ATTEMPTS")
if attempts:
    attempt_path = Path(attempts)
    attempt = int(attempt_path.read_text()) + 1 if attempt_path.exists() else 1
    attempt_path.write_text(str(attempt))
    if attempt <= int(os.environ.get("FAKE_QEMU_FAILURES", "0")):
        print(
            os.environ.get("FAKE_QEMU_FAILURE_MESSAGE", "Could not set up host forwarding rule"),
            file=sys.stderr,
        )
        sys.exit(1)

args = sys.argv[1:]
qmp = args[args.index("-qmp") + 1].split("unix:", 1)[1].split(",", 1)[0]
serial = args[args.index("-chardev") + 1].split("path=", 1)[1].split(",", 1)[0]
events = Path(os.environ["FAKE_QEMU_EVENTS"])
script = (
    json.loads(Path(os.environ["FAKE_QMP_SCRIPT"]).read_text())
    if os.environ.get("FAKE_QMP_SCRIPT")
    else {}
)


def event(name):
    with events.open("a") as stream:
        stream.write(name + "\n")


server = socket.socket(socket.AF_UNIX)
server.bind(qmp)
server.listen()
ser = socket.socket(socket.AF_UNIX)
ser.connect(serial)
ser.sendall(b"before READY after\xff\n")
event(f"pid={os.getpid()}")


def monitor():
    conn, _ = server.accept()
    stream = conn.makefile("rwb", buffering=0)
    stream.write(b'{"QMP":{}}\n')
    for line in stream:
        request = json.loads(line)
        command = request["execute"]
        if command == "human-monitor-command":
            command = request["arguments"]["command-line"]
            event(command)
            if command == "info registers -a":
                result = "CPU#0\nPC=fffff80000100000 SP=fffff80000200000\n"
            elif command.startswith("x /"):
                result = "fffff78000000008: 0x10 0x0 0x0"
            else:
                result = ""
            scripted = script.get(command)
            if scripted is not None:
                if isinstance(scripted, list):
                    result = scripted.pop(0) if len(scripted) > 1 else scripted[0]
                else:
                    result = scripted
            if command.startswith("loadvm"):
                ser.sendall(b"RESTORED\n")
        else:
            event(command)
            result = {}
        stream.write((json.dumps({"return": result}) + "\n").encode())


threading.Thread(target=monitor, daemon=True).start()
signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
while True:
    time.sleep(0.05)
