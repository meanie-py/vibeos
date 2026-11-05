#!/usr/bin/env python3
"""Owned QMP/serial fixture for EOF, stderr pressure and snapshot boundaries."""

import json
import os
import signal
import socket
import sys
import threading
import time
from pathlib import Path

args = sys.argv[1:]
qmp = args[args.index("-qmp") + 1].split("unix:", 1)[1].split(",", 1)[0]
serial = args[args.index("-chardev") + 1].split("path=", 1)[1].split(",", 1)[0]
events = Path(os.environ["SERIAL_EVENTS"])
# Exercise the real native file descriptor selected before the child starts.
sys.stderr.write(os.environ.get("SERIAL_STDERR", ""))
sys.stderr.write("x" * int(os.environ.get("SERIAL_STDERR_BYTES", "0")))
sys.stderr.flush()
server = socket.socket(socket.AF_UNIX)
server.bind(qmp)
server.listen()
ser = socket.socket(socket.AF_UNIX)
ser.connect(serial)
serial_lock = threading.Lock()
running = threading.Event()
running.set()


def send(text):
    with serial_lock:
        ser.sendall(text.encode("latin1"))


send(os.environ.get("SERIAL_INITIAL", "before READY\n"))
stop_line = os.environ.get("SERIAL_ON_STOP", "")
load_line = os.environ.get("SERIAL_ON_LOAD", "RESTORED\n")


def chat():
    duration = float(os.environ.get("SERIAL_CHATTY_SECONDS", "0"))
    end = time.monotonic() + duration
    while time.monotonic() < end:
        if running.is_set():
            send("payload still running\n")
        time.sleep(0.005)


threading.Thread(target=chat, daemon=True).start()


def monitor():
    global stop_line
    conn, _ = server.accept()
    stream = conn.makefile("rwb", buffering=0)
    stream.write(b'{"QMP":{}}\n')
    for line in stream:
        request = json.loads(line)
        op = request["execute"]
        if op == "human-monitor-command":
            op = request["arguments"]["command-line"]
        with events.open("a") as f:
            f.write(op + "\n")
        result = "" if request["execute"] == "human-monitor-command" else {}
        if op == "stop":
            running.clear()
            if stop_line:
                send(stop_line)
                stop_line = ""
        elif op == "cont":
            running.set()
        elif op.startswith("loadvm"):
            if running.is_set():
                result = "Error: fixture load on running guest"
            else:
                send(load_line)
        elif op == "query-status":
            result = {
                "status": "running" if running.is_set() else "paused",
                "running": running.is_set(),
            }
        stream.write((json.dumps({"return": result}) + "\n").encode())
        if op == "fixture-exit":
            time.sleep(0.02)
            os._exit(int(os.environ.get("SERIAL_EXIT_CODE", "1")))


threading.Thread(target=monitor, daemon=True).start()
signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
while True:
    time.sleep(0.05)
