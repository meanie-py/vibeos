"""Read-only HTTP server with byte ranges, for iPXE `sanboot` on the Pi 4B bench.

Run with WINDOWS python, for the same reason as serve_tftp.py: WSL2 in NAT mode
is not reachable from the LAN.

Why this exists: writing the 140 MB ramdisk to the SD card through the Pi's
firmware runs at ~0.18 MB/s (~13.5 min), and freeldr then needs ~3.5 min to
read it back. The network is not the limit (iPXE pulls the same file in 19 s).
`sanboot http://host:port/boot.img` makes iPXE install the image as a UEFI block
device and boot its EFI\\BOOT\\BOOTAA64.EFI; freeldr reads block devices with
its own FAT driver, so freeldr.ini and ramdisk.img come over the network and
the card is never touched. sanboot needs HEAD (for the size) and Range GETs,
which the standard library's http.server does not implement.

Serves one directory, read-only, no listings, no traversal.
Needs an inbound TCP rule for the port (the bench's TFTP rule is UDP 69 only).

Usage:  python serve_http.py [root] [bind-ip] [port]
"""

import http.server
import os
import re
import sys

RANGE = re.compile(r"bytes=(\d*)-(\d*)$")
CHUNK = 1 << 20


class Handler(http.server.BaseHTTPRequestHandler):
    root = "."
    # Keep-alive: a sanboot reads the image as thousands of small ranges (freeldr
    # asks for 62 KB at a time), and under HTTP/1.0 each one paid a new TCP
    # handshake and slow start. Every response below carries a Content-Length.
    protocol_version = "HTTP/1.1"
    # Headers and body go out as separate writes; don't let Nagle hold the body
    # back for the client's delayed ACK of the headers.
    disable_nagle_algorithm = True

    def _resolve(self):
        rel = self.path.split("?", 1)[0].lstrip("/")
        full = os.path.realpath(os.path.join(self.root, rel))
        if not full.startswith(os.path.realpath(self.root) + os.sep) or not os.path.isfile(full):
            self.send_error(404)
            return None
        return full

    def _span(self, size):
        """(start, end_inclusive, partial) or None after sending 416."""
        header = self.headers.get("Range")
        if not header:
            return 0, size - 1, False
        m = RANGE.match(header.strip())
        if not m or (m.group(1) == "" and m.group(2) == ""):
            self.send_error(416)
            return None
        if m.group(1) == "":  # suffix: last N bytes
            start, end = max(0, size - int(m.group(2))), size - 1
        else:
            start = int(m.group(1))
            end = min(int(m.group(2)), size - 1) if m.group(2) else size - 1
        if start >= size or start > end:
            self.send_response(416)
            self.send_header("Content-Range", f"bytes */{size}")
            self.send_header("Content-Length", "0")
            self.end_headers()
            return None
        return start, end, True

    def _serve(self, body):
        full = self._resolve()
        if full is None:
            return
        size = os.path.getsize(full)
        span = self._span(size)
        if span is None:
            return
        start, end, partial = span
        self.send_response(206 if partial else 200)
        self.send_header("Accept-Ranges", "bytes")
        self.send_header("Content-Length", str(end - start + 1))
        if partial:
            self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
        self.end_headers()
        if not body:
            return
        with open(full, "rb") as f:
            f.seek(start)
            left = end - start + 1
            while left:
                data = f.read(min(CHUNK, left))
                if not data:
                    break
                self.wfile.write(data)
                left -= len(data)

    def do_HEAD(self):
        self._serve(body=False)

    def do_GET(self):
        self._serve(body=True)

    def log_message(self, format, *args):
        # Range requests arrive by the thousand during a sanboot; log only
        # whole-file fetches and errors.
        if self.headers.get("Range") is None or not str(args[1]).startswith("2"):
            sys.stdout.write(f"{self.client_address[0]} {format % args}\n")
            sys.stdout.flush()


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    bind = sys.argv[2] if len(sys.argv) > 2 else "0.0.0.0"
    port = int(sys.argv[3]) if len(sys.argv) > 3 else 8069
    Handler.root = root
    server = http.server.ThreadingHTTPServer((bind, port), Handler)
    print(f"serving {os.path.abspath(root)} on {bind}:{port}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
