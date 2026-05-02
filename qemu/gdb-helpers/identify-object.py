# identify-object <addr>: best-effort answer to "what is this kernel pointer?"
#
# Combines:
#   - DISPATCHER_HEADER.Type at +0x00 (1 byte)
#   - Pool header tag (via the same scan logic as pool-tag.py)
#   - For ProcessObject: read ImageFileName at the body-relative offset
#     observed in the current kernel (amd64=0x260). Empirically stable but
#     not authoritative — bumping NTDDI_VERSION can shift it.
#
# Use this when gdb hands you a "Thread" pointer that's misbehaving and
# you want to know whether it's actually a thread, a process, an event,
# or just garbage.

import gdb
import os
import string


PRINTABLE = set(string.ascii_letters + string.digits + " _-")


KOBJECT_TYPE = {
    0x00: "EventNotificationObject",
    0x01: "EventSynchronizationObject",
    0x02: "MutantObject",
    0x03: "ProcessObject",
    0x04: "QueueObject",
    0x05: "SemaphoreObject",
    0x06: "ThreadObject",
    0x07: "GateObject",
    0x08: "TimerNotificationObject",
    0x09: "TimerSynchronizationObject",
    0x0a: "Spare2Object",
    0x0b: "Spare3Object",
    0x0c: "Spare4Object",
    0x0d: "Spare5Object",
    0x0e: "Spare6Object",
    0x0f: "Spare7Object",
    0x10: "Spare8Object",
    0x11: "Spare9Object",
    0x12: "ApcObject",
    0x13: "DpcObject",
    0x14: "DeviceQueueObject",
    0x15: "EventPairObject",
    0x16: "InterruptObject",
    0x17: "ProfileObject",
    0x18: "ThreadedDpcObject",
}


# Offset of EPROCESS.ImageFileName from the body start (= KPROCESS start).
# Observed empirically on the current amd64 kernel build. Update if the
# kernel struct layout changes.
EPROCESS_IMAGEFILENAME_OFFSET = {
    "amd64": 0x260,
    "i386":  0x174,   # observed on legacy 32-bit Windows; verify per build
    "arm64": 0x260,   # mirrors amd64; verify when arm64 is exercised
}


def _arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


def _io_read_u(addr: int, size: int) -> int | None:
    try:
        ctype = {1: "unsigned char", 2: "unsigned short",
                 4: "unsigned int", 8: "unsigned long long"}[size]
        v = gdb.parse_and_eval(f"*({ctype} *)({addr:#x})")
        return int(v) & ((1 << (size * 8)) - 1)
    except gdb.error:
        return None


def _io_read_string(addr: int, max_len: int = 16) -> str:
    out = []
    for i in range(max_len):
        b = _io_read_u(addr + i, 1)
        if b is None or b == 0:
            break
        c = chr(b)
        if c not in string.printable or c in "\r\n\t\x0b\x0c":
            break
        out.append(c)
    return "".join(out)


def _decode_tag(tag_bytes: int) -> str | None:
    chars = []
    for shift in (0, 8, 16, 24):
        b = (tag_bytes >> shift) & 0xFF
        c = chr(b)
        if c not in PRINTABLE:
            return None
        chars.append(c)
    return "".join(chars)


def _try_pool_header(header_addr: int) -> dict | None:
    ulong1 = _io_read_u(header_addr, 4)
    if ulong1 is None:
        return None
    pool_tag = _io_read_u(header_addr + 4, 4)
    if pool_tag is None:
        return None
    tag = _decode_tag(pool_tag)
    if tag is None:
        return None
    block_size = (ulong1 >> 16) & 0x1FF
    pool_type = (ulong1 >> 25) & 0x7F
    if block_size == 0 or pool_type & 0x18:
        return None
    return {"tag": tag, "block_bytes": block_size * 16, "pool_type": pool_type}


def _find_pool_header(addr: int, arch: str) -> dict | None:
    candidates = [0x10, 0x40, 0x50] if arch in ("amd64", "arm64") else [0x08, 0x28, 0x38]
    for back in candidates:
        info = _try_pool_header(addr - back)
        if info:
            info["body_offset"] = back
            info["header_addr"] = addr - back
            return info
    return None


class IdentifyObjectCmd(gdb.Command):
    """identify-object <addr>: identify a kernel pointer (process/thread/etc)."""

    def __init__(self):
        super().__init__("identify-object", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        a = arg.strip()
        if not a:
            gdb.write("identify-object: usage: identify-object <addr>\n", gdb.STDERR)
            return
        try:
            addr = int(gdb.parse_and_eval(a)) & 0xFFFFFFFFFFFFFFFF
        except gdb.error as e:
            gdb.write(f"identify-object: bad address: {e}\n", gdb.STDERR)
            return

        arch = _arch()
        gdb.write(f"identify-object @ {addr:#018x}  arch={arch}\n")

        # 1. DISPATCHER_HEADER.Type at +0x00
        type_byte = _io_read_u(addr, 1)
        if type_byte is None:
            gdb.write("  Header.Type     <unreadable>\n")
        else:
            type_name = KOBJECT_TYPE.get(type_byte, f"Unknown({type_byte:#x})")
            gdb.write(f"  Header.Type     {type_byte:#04x}  {type_name}\n")

        # 2. Pool header
        pool = _find_pool_header(addr, arch)
        if pool:
            gdb.write(
                f"  PoolHeader      @ {pool['header_addr']:#018x}  "
                f"tag='{pool['tag']}'  size={pool['block_bytes']}B  "
                f"(body offset {pool['body_offset']:#x})\n"
            )
        else:
            gdb.write("  PoolHeader      <not found within 0x50 bytes>\n")

        # 3. Type-specific drilldowns
        if type_byte == 0x03:  # ProcessObject
            off = EPROCESS_IMAGEFILENAME_OFFSET.get(arch)
            if off is not None:
                name = _io_read_string(addr + off, max_len=16)
                if name:
                    gdb.write(f"  ImageFileName   '{name}'  (at body+{off:#x})\n")
                else:
                    gdb.write(f"  ImageFileName   <empty/unreadable at body+{off:#x}>\n")

        if type_byte == 0x06:  # ThreadObject
            # Defer Cid printing — offset is layout-sensitive across builds.
            # Most useful next steps for a thread are bt + dump-thread anyway.
            gdb.write("  (use dump-thread for full KTHREAD field walk)\n")


IdentifyObjectCmd()
