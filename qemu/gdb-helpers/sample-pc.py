# sample-pc: read $pc + current process->ImageFileName, output one line.
#
# Designed to be invoked many times from a shell loop (qemu/sample-pc.sh)
# rather than driven from a single batch gdb session — gdb's batch-mode
# event loop doesn't reliably synchronize continue/interrupt cycles
# against a remote gdbstub, so the multi-sample in-one-batch approach
# kept reading stale state.
#
# Output format (one line, easy to grep / sort / uniq -c):
#   SAMPLE pc=0x<hex> proc=<imagename> sym=<symbol+offset>
#
# The shell wrapper aggregates these into a histogram.

import gdb
import os


def _sp_arch() -> str:
    return os.environ.get("VIBEOS_ARCH", "amd64")


def _sp_ptr_size() -> int:
    return 4 if _sp_arch() == "i386" else 8


def _sp_resolve_field_offset(struct_name: str, field: str) -> int | None:
    try:
        return int(gdb.parse_and_eval(f"&((struct {struct_name} *)0)->{field}"))
    except gdb.error:
        return None


def _sp_read_handle(addr: int, ptr_size: int) -> int | None:
    ctype = "unsigned long long" if ptr_size == 8 else "unsigned int"
    try:
        return int(gdb.parse_and_eval(f"*({ctype} *)({addr:#x})")) & ((1 << (ptr_size * 8)) - 1)
    except gdb.error:
        return None


def _sp_read_imagename(eproc_addr: int, off: int) -> str | None:
    out = []
    for i in range(16):
        try:
            b = int(gdb.parse_and_eval(f"*(unsigned char *)({eproc_addr + off + i:#x})")) & 0xFF
        except gdb.error:
            return None
        if b == 0:
            break
        if 0x20 <= b < 0x7F:
            out.append(chr(b))
        else:
            out.append("?")
    return "".join(out) or None


def _sp_current_thread() -> int | None:
    """Per-arch current-thread pointer.  DWARF lacks _KPCR/_KIPCR for arm64,
    so we fall back to raw offsets (TPIDR_EL1 → KPCR + 0x20 → CurrentPrcb;
    PRCB + 0x08 → CurrentThread).  Layout taken from
    sdk/lib/rust/ntkrnl-sys/src/bindgen_arm64.rs.
    """
    ptr_size = _sp_ptr_size()
    arch = _sp_arch()

    # Per-arch raw KPCR offsets — DWARF doesn't carry _KPCR/_KIPCR on any
    # of our targets so we walk the structs by hand.  Layouts taken from
    # sdk/lib/rust/ntkrnl-sys/src/bindgen_<arch>.rs.
    if arch == "arm64":
        # TPIDR_EL1 → KPCR; +0x20 → CurrentPrcb; PRCB +0x08 → CurrentThread.
        try:
            pcr = int(gdb.parse_and_eval("(unsigned long)$TPIDR_EL1")) & 0xFFFFFFFFFFFFFFFF
        except gdb.error:
            return None
        if not pcr:
            return None
        prcb = _sp_read_handle(pcr + 0x20, ptr_size)
        if not prcb:
            return None
        return _sp_read_handle(prcb + 0x08, ptr_size)

    if arch == "i386":
        # fs_base → KPCR; +0x20 → Prcb; PRCB +0x04 → CurrentThread.
        try:
            pcr = int(gdb.parse_and_eval("(unsigned long)$fs_base")) & 0xFFFFFFFF
        except gdb.error:
            return None
        if not pcr:
            return None
        prcb = _sp_read_handle(pcr + 0x20, ptr_size)
        if not prcb:
            return None
        return _sp_read_handle(prcb + 0x04, ptr_size)

    # amd64 — DWARF carries _KPCR for amd64 builds, so let gdb walk it.
    # If the DWARF cast stops working we'll need to add a raw-offset path
    # similar to the arm64/i386 ones above.
    for expr in (
        "((struct _KPCR *)$gs_base)->Prcb.CurrentThread",
        "((struct _KPCR *)$gs_base)->PrcbData.CurrentThread",
        "KeGetCurrentThread()",
    ):
        try:
            v = int(gdb.parse_and_eval(expr))
            if v:
                return v & 0xFFFFFFFFFFFFFFFF
        except gdb.error:
            continue
    return None


def _sp_current_process_name() -> str | None:
    ptr_size = _sp_ptr_size()
    cur_thread = _sp_current_thread()
    if cur_thread is None:
        return None
    proc_off = _sp_resolve_field_offset("_ETHREAD", "ThreadsProcess")
    if proc_off is None:
        return None
    eproc = _sp_read_handle(cur_thread + proc_off, ptr_size)
    if not eproc:
        return None
    img_off = _sp_resolve_field_offset("_EPROCESS", "ImageFileName")
    if img_off is None:
        return None
    return _sp_read_imagename(eproc, img_off)


def _sp_symbolicate(pc: int) -> str:
    try:
        out = gdb.execute(f"info symbol {pc:#x}", to_string=True).strip()
    except gdb.error:
        return "<gdb-error>"
    if "No symbol matches" in out or "no symbol" in out.lower():
        return f"<no-sym>@{pc:#x}"
    return out.split(" in ")[0].strip()


class SamplePcCmd(gdb.Command):
    """sample-pc: read $pc + current-process name, output one SAMPLE line."""

    def __init__(self):
        super().__init__("sample-pc", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        try:
            pc = int(gdb.parse_and_eval("(unsigned long)$pc")) & 0xFFFFFFFFFFFFFFFF
        except gdb.error as e:
            gdb.write(f"sample-pc: can't read $pc: {e}\n", gdb.STDERR)
            return
        sym = _sp_symbolicate(pc)
        proc = _sp_current_process_name() or "<unknown>"
        gdb.write(f"SAMPLE pc={pc:#018x} proc={proc} sym={sym}\n")


SamplePcCmd()
