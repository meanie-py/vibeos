# pool-tag <addr>: walk backwards from addr to find the enclosing POOL_HEADER
# and decode its tag, size, and pool type.
#
# Useful when gdb hands you a kernel pointer and you want to know what kind
# of object it points at without having to look up bindgen offsets by hand.
#
# Heuristics:
# - On amd64/arm64 (8-byte pointers), POOL_HEADER is 16 bytes. We try
#   addr-0x10 first.
# - On i386, POOL_HEADER is 8 bytes. We try addr-0x08.
# - For object-manager allocations the body is typically at POOL_HEADER + 0x10
#   + 0x30 (OBJECT_HEADER) on amd64, so we also try addr-0x40 to find the
#   pool header for an object body. Likewise -0x30 on i386 (pool 8 + obj 0x28).
#
# We accept a candidate pool header when:
# - PoolTag bytes are all printable ASCII
# - PoolType is in [0..7]  (NonPaged through PagedCacheAligned with Session bit)
# - BlockSize > 0

import gdb
import os
import string


PRINTABLE = set(string.ascii_letters + string.digits + " _-")


POOL_TYPES = {
    0: "NonPagedPool",
    1: "PagedPool",
    2: "NonPagedPoolMustSucceed",
    3: "DontUseThisType",
    4: "NonPagedPoolCacheAligned",
    5: "PagedPoolCacheAligned",
    6: "NonPagedPoolCacheAlignedMustS",
    7: "MaxPoolType",
    32: "NonPagedPoolSession",
    33: "PagedPoolSession",
    36: "NonPagedPoolCacheAlignedSession",
    37: "PagedPoolCacheAlignedSession",
}


def _pt_read_u32(addr: int) -> int | None:
    try:
        v = gdb.parse_and_eval(f"*(unsigned int *)({addr:#x})")
        return int(v) & 0xFFFFFFFF
    except gdb.error:
        return None


def _decode_tag(tag_bytes: int) -> str | None:
    chars = []
    for shift in (0, 8, 16, 24):
        b = (tag_bytes >> shift) & 0xFF
        c = chr(b)
        if c not in PRINTABLE:
            return None
        chars.append(c)
    return "".join(chars)


def _try_decode(header_addr: int) -> dict | None:
    ulong1 = _pt_read_u32(header_addr)
    if ulong1 is None:
        return None
    pool_tag = _pt_read_u32(header_addr + 4)
    if pool_tag is None:
        return None

    tag = _decode_tag(pool_tag)
    if tag is None:
        return None

    prev_size = ulong1 & 0x1FF
    pool_index = (ulong1 >> 9) & 0x7F
    block_size = (ulong1 >> 16) & 0x1FF
    pool_type = (ulong1 >> 25) & 0x7F

    if block_size == 0:
        return None
    if pool_type & 0x18:  # bits 3-4 are reserved/free; reject obvious garbage
        return None

    return {
        "header_addr": header_addr,
        "tag": tag,
        "block_size_bytes": block_size * 16,  # chunks of 16 on 64-bit
        "prev_size_bytes": prev_size * 16,
        "pool_type": pool_type,
        "pool_type_name": POOL_TYPES.get(pool_type, f"Unknown({pool_type})"),
        "pool_index": pool_index,
        "raw_ulong1": ulong1,
        "raw_tag": pool_tag,
    }


def _candidate_offsets(arch: str) -> list[int]:
    # Distance from a "thing the user pointed at" back to the POOL_HEADER.
    # Try the most-specific first; whichever decodes cleanly wins.
    if arch in ("amd64", "arm64"):
        return [
            0x10,   # raw pool body
            0x40,   # object body: pool 0x10 + object header 0x30
            0x50,   # object body: pool 0x10 + object header 0x30 + creator info 0x10
        ]
    # i386
    return [
        0x08,
        0x28,   # pool 0x08 + object header 0x20
        0x38,
    ]


def _arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


class PoolTagCmd(gdb.Command):
    """pool-tag <addr>: identify the enclosing POOL_HEADER for a kernel address."""

    def __init__(self):
        super().__init__("pool-tag", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        a = arg.strip()
        if not a:
            gdb.write("pool-tag: usage: pool-tag <addr>\n", gdb.STDERR)
            return
        try:
            addr = int(gdb.parse_and_eval(a)) & 0xFFFFFFFFFFFFFFFF
        except gdb.error as e:
            gdb.write(f"pool-tag: bad address: {e}\n", gdb.STDERR)
            return

        arch = _arch()
        for back in _candidate_offsets(arch):
            cand = addr - back
            info = _try_decode(cand)
            if info is None:
                continue
            gdb.write(
                f"POOL_HEADER @ {info['header_addr']:#018x}  "
                f"(target = header + {back:#x})\n"
            )
            gdb.write(f"  Tag             '{info['tag']}'\n")
            gdb.write(f"  BlockSize       {info['block_size_bytes']} bytes\n")
            gdb.write(f"  PrevSize        {info['prev_size_bytes']} bytes\n")
            gdb.write(
                f"  PoolType        {info['pool_type']} ({info['pool_type_name']})\n"
            )
            gdb.write(f"  PoolIndex       {info['pool_index']}\n")
            return

        gdb.write(
            f"pool-tag: no plausible POOL_HEADER within {max(_candidate_offsets(arch)):#x} "
            f"bytes before {addr:#018x}\n"
        )


PoolTagCmd()
