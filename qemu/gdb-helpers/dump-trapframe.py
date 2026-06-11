# dump-trapframe <addr>: pretty-print a KTRAP_FRAME at addr per arch.
#
# Replaces the manual offset arithmetic on a 64-line `x/64gx` dump. Layouts
# mirror sdk/include/ndk/<arch>/ketypes.h — bumping NTDDI_VERSION or hot-
# patching that struct means this file needs an update too.
#
# Field selection is opinionated: we surface the registers and segments
# you actually want when triaging a kernel #PF/#GP/etc. — not every Dr*.

import gdb
import os


def _tf_read(addr: int, ctype: str) -> int:
    val = gdb.parse_and_eval(f"*({ctype} *)({addr:#x})")
    return int(val)


def _tf_read_u(addr: int, size: int) -> int:
    return _tf_read(addr, {1: "unsigned char", 2: "unsigned short",
                           4: "unsigned int", 8: "unsigned long long"}[size])


# (offset, size, label)
AMD64_FIELDS = [
    (0x000, 8, "P1Home"),
    (0x008, 8, "P2Home"),
    (0x010, 8, "P3Home"),
    (0x018, 8, "P4Home"),
    (0x020, 8, "P5"),
    (0x028, 1, "PreviousMode"),
    (0x029, 1, "PreviousIrql"),
    (0x02a, 1, "FaultIndicator"),
    (0x02b, 1, "ExceptionActive"),
    (0x02c, 4, "MxCsr"),
    (0x030, 8, "Rax"),
    (0x038, 8, "Rcx"),
    (0x040, 8, "Rdx"),
    (0x048, 8, "R8"),
    (0x050, 8, "R9"),
    (0x058, 8, "R10"),
    (0x060, 8, "R11"),
    (0x068, 8, "GsBase"),
    # Skip Xmm0..Xmm5 (+0x70..+0xd0).
    (0x0d0, 8, "FaultAddress"),
    # Skip Dr0..Dr7 + DebugControl block.
    (0x130, 2, "SegDs"),
    (0x132, 2, "SegEs"),
    (0x134, 2, "SegFs"),
    (0x136, 2, "SegGs"),
    (0x138, 8, "TrapFrame"),
    (0x140, 8, "Rbx"),
    (0x148, 8, "Rdi"),
    (0x150, 8, "Rsi"),
    (0x158, 8, "Rbp"),
    (0x160, 8, "ErrorCode"),
    (0x168, 8, "Rip"),
    (0x170, 2, "SegCs"),
    (0x178, 4, "EFlags"),
    (0x180, 8, "Rsp"),
    (0x188, 2, "SegSs"),
]
AMD64_SIZE = 0x190


# i386 KTRAP_FRAME — see sdk/include/ndk/i386/ketypes.h.
I386_FIELDS = [
    (0x000, 4, "DbgEbp"),
    (0x004, 4, "DbgEip"),
    (0x008, 4, "DbgArgMark"),
    (0x00c, 4, "DbgArgPointer"),
    (0x010, 4, "TempSegCs"),
    (0x014, 4, "TempEsp"),
    (0x018, 4, "Dr0"),
    (0x01c, 4, "Dr1"),
    (0x020, 4, "Dr2"),
    (0x024, 4, "Dr3"),
    (0x028, 4, "Dr6"),
    (0x02c, 4, "Dr7"),
    (0x030, 4, "SegGs"),
    (0x034, 4, "SegEs"),
    (0x038, 4, "SegDs"),
    (0x03c, 4, "Edx"),
    (0x040, 4, "Ecx"),
    (0x044, 4, "Eax"),
    (0x048, 4, "PreviousMode"),
    (0x04c, 4, "ExceptionList"),
    (0x050, 4, "SegFs"),
    (0x054, 4, "Edi"),
    (0x058, 4, "Esi"),
    (0x05c, 4, "Ebx"),
    (0x060, 4, "Ebp"),
    (0x064, 4, "ErrCode"),
    (0x068, 4, "Eip"),
    (0x06c, 4, "SegCs"),
    (0x070, 4, "EFlags"),
    (0x074, 4, "HardwareEsp"),
    (0x078, 4, "HardwareSegSs"),
    (0x07c, 4, "V86Es"),
    (0x080, 4, "V86Ds"),
    (0x084, 4, "V86Fs"),
    (0x088, 4, "V86Gs"),
]
I386_SIZE = 0x08c


# arm64: minimal subset — full struct is in sdk/include/ndk/arm64/ketypes.h.
ARM64_FIELDS = [
    (0x000, 8, "X0"),
    (0x008, 8, "X1"),
    (0x010, 8, "X2"),
    (0x018, 8, "X3"),
    (0x020, 8, "X4"),
    (0x028, 8, "X5"),
    (0x030, 8, "X6"),
    (0x038, 8, "X7"),
    (0x100, 8, "Fp"),
    (0x108, 8, "Lr"),
    (0x110, 8, "Sp"),
    (0x118, 8, "Pc"),
    (0x120, 4, "Spsr"),
    (0x124, 4, "Esr"),
    (0x128, 8, "FaultAddress"),
]


SCHEMA = {
    "amd64": (AMD64_FIELDS, AMD64_SIZE),
    "i386":  (I386_FIELDS,  I386_SIZE),
    "arm64": (ARM64_FIELDS, None),
}


def _arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


class DumpTrapFrameCmd(gdb.Command):
    """dump-trapframe <addr>: pretty-print KTRAP_FRAME at addr."""

    def __init__(self):
        super().__init__("dump-trapframe", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        a = arg.strip()
        if not a:
            gdb.write("dump-trapframe: usage: dump-trapframe <addr>\n", gdb.STDERR)
            return
        try:
            addr = int(gdb.parse_and_eval(a)) & 0xFFFFFFFFFFFFFFFF
        except gdb.error as e:
            gdb.write(f"dump-trapframe: bad address: {e}\n", gdb.STDERR)
            return

        arch = _arch()
        if arch not in SCHEMA:
            gdb.write(f"dump-trapframe: unsupported arch '{arch}'\n", gdb.STDERR)
            return

        fields, size = SCHEMA[arch]
        size_str = f" ({size:#x} bytes)" if size else ""
        gdb.write(f"KTRAP_FRAME @ {addr:#018x}  arch={arch}{size_str}\n")

        for off, sz, name in fields:
            try:
                val = _tf_read_u(addr + off, sz)
            except gdb.error as e:
                gdb.write(f"  +{off:#05x}  {name:<16} <unreadable: {e}>\n")
                continue
            width = sz * 2
            gdb.write(f"  +{off:#05x}  {name:<16} {val:#0{width + 2}x}\n")


DumpTrapFrameCmd()
