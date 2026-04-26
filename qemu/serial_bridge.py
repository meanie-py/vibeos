#!/usr/bin/env python3
"""
Serial bridge for QEMU ARM64 ReactOS.

Launches QEMU with -serial stdio, captures serial output to a log file,
and accepts input via a Unix socket at /tmp/qemu_serial_input.sock.

Usage:
    python3 qemu/serial_bridge.py          # Launch QEMU + bridge

Inject input:
    python3 -c "import socket; s=socket.socket(socket.AF_UNIX); s.connect('/tmp/qemu_serial_input.sock'); s.sendall(b'hello\\r'); s.close()"
"""

import subprocess
import os
import sys
import socket
import threading
import time
import signal

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_DIR = os.path.dirname(SCRIPT_DIR)
BUILD_DIR = os.path.join(PROJECT_DIR, "build-arm64")
QEMU_DIR = SCRIPT_DIR

BOOT_IMG = os.path.join(QEMU_DIR, "boot.img")
UEFI_FW = os.path.join(QEMU_DIR, "QEMU_EFI.fd")
EFIVARS = os.path.join(QEMU_DIR, "efivars.fd")
SERIAL_LOG = os.path.join(QEMU_DIR, "serial.log")
INPUT_SOCK = "/tmp/qemu_serial_input.sock"

def main():
    # Clean up old socket
    try:
        os.unlink(INPUT_SOCK)
    except FileNotFoundError:
        pass

    # Create input socket server
    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(INPUT_SOCK)
    srv.listen(5)
    print(f"[bridge] Input socket: {INPUT_SOCK}", flush=True)

    # Launch QEMU with -serial stdio
    cmd = [
        "qemu-system-aarch64",
        "-M", "virt",
        "-cpu", "cortex-a72",
        "-m", "1G",
        "-drive", f"if=pflash,format=raw,file={UEFI_FW},readonly=on",
        "-drive", f"if=pflash,format=raw,file={EFIVARS}",
        "-drive", f"file={BOOT_IMG},format=raw,if=virtio",
        "-device", "ramfb",
        "-serial", "stdio",
        "-monitor", "telnet:localhost:4444,server,nowait",
        "-nographic",
    ]

    print(f"[bridge] Launching QEMU...", flush=True)
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=0,
    )
    print(f"[bridge] QEMU PID: {proc.pid}", flush=True)

    # Serial output reader thread: reads stdout, writes to log
    def output_reader():
        with open(SERIAL_LOG, "wb") as log:
            while True:
                data = proc.stdout.read(1)
                if not data:
                    break
                log.write(data)
                log.flush()
        print("[bridge] Output reader exiting", flush=True)

    # Input socket handler: accepts connections, forwards bytes to QEMU stdin
    def input_handler():
        while True:
            try:
                conn, _ = srv.accept()
                print("[bridge] Input client connected", flush=True)
                while True:
                    data = conn.recv(256)
                    if not data:
                        break
                    proc.stdin.write(data)
                    proc.stdin.flush()
                    print(f"[bridge] Injected {len(data)} bytes: {repr(data[:50])}", flush=True)
                conn.close()
                print("[bridge] Input client disconnected", flush=True)
            except Exception as e:
                print(f"[bridge] Input handler error: {e}", flush=True)
                break
        print("[bridge] Input handler exiting", flush=True)

    ot = threading.Thread(target=output_reader, daemon=True)
    it = threading.Thread(target=input_handler, daemon=True)
    ot.start()
    it.start()

    # Wait for QEMU to exit
    try:
        proc.wait()
    except KeyboardInterrupt:
        proc.terminate()

    print(f"[bridge] QEMU exited with code {proc.returncode}", flush=True)
    srv.close()
    try:
        os.unlink(INPUT_SOCK)
    except:
        pass

if __name__ == "__main__":
    main()
