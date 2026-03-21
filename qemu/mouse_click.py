#!/usr/bin/env python3
"""
Send mouse click events to ReactOS ARM64 via QEMU serial socket.

Usage:
    python3 mouse_click.py <x> <y>          # Left click at pixel (x, y)
    python3 mouse_click.py <x> <y> right    # Right click at pixel (x, y)
    python3 mouse_click.py <x> <y> move     # Move cursor only

Screen resolution is assumed 1024x768 (QEMU ramfb default).
Coordinates are converted to 0-65535 absolute range.

Protocol: ESC [ M <action> <x_lo> <x_hi> <y_lo> <y_hi>
  action: 0=move, 1=left-down, 2=left-up, 3=right-down, 4=right-up
"""
import socket
import struct
import sys
import time
import os

SOCKET_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "serial.sock")
SCREEN_W = 800
SCREEN_H = 600

def make_mouse_event(action, x_abs, y_abs):
    """Build ESC[M escape sequence for one mouse event."""
    x_lo = x_abs & 0xFF
    x_hi = (x_abs >> 8) & 0xFF
    y_lo = y_abs & 0xFF
    y_hi = (y_abs >> 8) & 0xFF
    return b'\x1b[M' + bytes([action, x_lo, x_hi, y_lo, y_hi])

def pixel_to_abs(px, py):
    """Convert pixel coordinates to 0-65535 absolute range."""
    ax = min(65535, max(0, int(px * 65535 / (SCREEN_W - 1))))
    ay = min(65535, max(0, int(py * 65535 / (SCREEN_H - 1))))
    return ax, ay

def send_events(events):
    """Send a list of byte sequences to the serial socket."""
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.connect(SOCKET_PATH)
    for ev in events:
        sock.sendall(ev)
        time.sleep(0.05)  # Delay between events
    sock.close()

def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)

    px = int(sys.argv[1])
    py = int(sys.argv[2])
    action = sys.argv[3] if len(sys.argv) > 3 else "left"

    ax, ay = pixel_to_abs(px, py)
    print(f"Pixel ({px}, {py}) -> Abs ({ax}, {ay})")

    if action == "move":
        events = [make_mouse_event(0, ax, ay)]
    elif action == "right":
        events = [
            make_mouse_event(0, ax, ay),   # Move
            make_mouse_event(3, ax, ay),   # Right down
            make_mouse_event(4, ax, ay),   # Right up
        ]
    else:  # "left" or default
        events = [
            make_mouse_event(0, ax, ay),   # Move
            make_mouse_event(1, ax, ay),   # Left down
            make_mouse_event(2, ax, ay),   # Left up
        ]

    send_events(events)
    print(f"Sent {action} click at ({px}, {py})")

if __name__ == "__main__":
    main()
