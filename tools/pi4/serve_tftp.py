"""Read-only TFTP server for Pi 4B bring-up, run with WINDOWS python.

Why Windows python and not WSL: WSL2 is in NAT mode here (172.26.x.x), so the
Pi cannot reach it. The host's LAN address is 192.168.1.183 on "Ethernet", which
is the same subnet the Pi DHCPs into.

Serves one directory, read-only, no writes, no directory traversal. It exists to
close the bring-up feedback loop: the Pi's UEFI Shell pulls a new BOOTAA64.EFI
and ramdisk.img straight onto the SD card, so the card never leaves the board.

Implements RFC 1350 RRQ plus the RFC 2348 blksize, RFC 2349 tsize and RFC 7440
windowsize options.
blksize matters a lot: TFTP is lockstep, so a 140 MB ramdisk at the default 512
bytes is ~287k round trips. The EDK2 client negotiates 1408+, cutting that by ~3x.

Usage:  python serve_tftp.py [root] [bind-ip]
"""

import os
import socket
import struct
import sys
import time

OP_RRQ, OP_WRQ, OP_DATA, OP_ACK, OP_ERROR, OP_OACK = 1, 2, 3, 4, 5, 6
# Retransmit budget per block. A stalled client must end the transfer rather
# than pin the single-threaded server forever.
ACK_TIMEOUT = 2.0
MAX_RETRIES = 5


def send_error(sock, addr, code, message):
    sock.sendto(struct.pack("!HH", OP_ERROR, code) + message.encode() + b"\0", addr)


def parse_request(payload):
    # filename\0mode\0[opt\0value\0]...
    parts = payload.split(b"\0")
    if len(parts) < 2:
        return None, None, {}
    filename = parts[0].decode("latin1")
    mode = parts[1].decode("latin1").lower()
    options = {}
    rest = parts[2:]
    for i in range(0, len(rest) - 1, 2):
        if rest[i]:
            options[rest[i].decode("latin1").lower()] = rest[i + 1].decode("latin1")
    return filename, mode, options


def resolve(root, filename):
    """Map a TFTP filename to a path inside root, or None if it escapes."""
    # Clients send both separators; treat them alike, then confine to root.
    name = filename.replace("\\", "/").lstrip("/")
    full = os.path.normpath(os.path.join(root, name))
    if os.path.commonpath([os.path.abspath(full), os.path.abspath(root)]) != os.path.abspath(root):
        return None
    return full if os.path.isfile(full) else None


def serve_file(path, addr, options):
    """One transfer on its own ephemeral port, as TFTP requires."""
    size = os.path.getsize(path)
    blksize = 512
    window = 1
    oack = {}
    if "blksize" in options:
        try:
            requested = int(options["blksize"])
        except ValueError:
            requested = 512
        # 8..65464 is the RFC range. Above ~1468 a block no longer fits one
        # Ethernet frame and travels IP-fragmented, which is fine on one LAN.
        blksize = max(8, min(requested, 65464))
        oack["blksize"] = str(blksize)
    if "windowsize" in options:
        # RFC 7440: send up to N blocks per ACK. Lockstep TFTP moved the 140 MB
        # ramdisk at 0.2 MB/s (825s) because every block paid a full round trip.
        try:
            window = max(1, min(int(options["windowsize"]), 64))
        except ValueError:
            window = 1
        oack["windowsize"] = str(window)
    if "tsize" in options:
        oack["tsize"] = str(size)

    # A file that is an exact multiple of blksize still ends with an empty block.
    last = size // blksize + 1
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(ACK_TIMEOUT)
    started = time.time()
    try:
        if oack:
            payload = struct.pack("!H", OP_OACK)
            for key, value in oack.items():
                payload += key.encode() + b"\0" + value.encode() + b"\0"
            if await_ack(sock, addr, [payload], 0, 0) is None:
                return
        acked = 0
        with open(path, "rb") as handle:
            while acked < last:
                first = acked + 1
                top = min(acked + window, last)
                packets = []
                for n in range(first, top + 1):
                    handle.seek((n - 1) * blksize)
                    packets.append(struct.pack("!HH", OP_DATA, n & 0xFFFF) + handle.read(blksize))
                got = await_ack(sock, addr, packets, acked, top)
                if got is None:
                    print(f"  ! {os.path.basename(path)}: client stopped acking after block {acked}")
                    return
                acked = got
        elapsed = max(time.time() - started, 1e-6)
        print(f"  = {os.path.basename(path)}: {size} bytes in {elapsed:.1f}s "
              f"({size / elapsed / 1e6:.1f} MB/s, blksize {blksize}, window {window})", flush=True)
    finally:
        sock.close()


def await_ack(sock, addr, packets, acked, top):
    """Send a window of packets and return the highest block the client ACKs.

    Block numbers are 16 bits on the wire and wrap every 65536 blocks (96 MB at
    blksize 1468), so an ACK is mapped back into the window (acked..top) rather
    than compared directly. An ACK short of top means the client lost a block;
    the caller resends from there. None means the client went away.
    """
    for _ in range(MAX_RETRIES):
        for packet in packets:
            sock.sendto(packet, addr)
        try:
            while True:
                reply, source = sock.recvfrom(1024)
                if source != addr or len(reply) < 4:
                    continue
                opcode, block = struct.unpack("!HH", reply[:4])
                if opcode == OP_ERROR:
                    return None
                if opcode != OP_ACK:
                    continue
                for n in range(top, acked - 1, -1):
                    if n & 0xFFFF == block:
                        if n > acked or top == acked:
                            return n
                        break
                # A duplicate ACK for what was already acknowledged. After a loss
                # the client repeats it once per out-of-order block, so treating
                # each as a retransmit request (and a retry) would exhaust the
                # budget on one lost packet. Keep listening; the timeout resends.
        except socket.timeout:
            continue
    return None


def main():
    root = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
    bind = sys.argv[2] if len(sys.argv) > 2 else "0.0.0.0"
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind((bind, 69))
    print(f"TFTP read-only on {bind}:69 serving {root}", flush=True)
    while True:
        payload, addr = sock.recvfrom(65536)
        if len(payload) < 4:
            continue
        opcode = struct.unpack("!H", payload[:2])[0]
        if opcode == OP_WRQ:
            # Read-only on purpose: nothing should ever write to the bench dir.
            send_error(sock, addr, 2, "server is read-only")
            continue
        if opcode != OP_RRQ:
            continue
        filename, _mode, options = parse_request(payload[2:])
        path = resolve(root, filename) if filename else None
        print(f"> {addr[0]} RRQ {filename!r} opts={options}", flush=True)
        if not path:
            send_error(sock, addr, 1, "file not found")
            print("  ! not found", flush=True)
            continue
        serve_file(path, addr, options)


if __name__ == "__main__":
    main()
