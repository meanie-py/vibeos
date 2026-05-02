# Locate the runtime kernel base by scanning memory for the PE "MZ" signature.
#
# ntoskrnl.exe loads at a fixed kernel-mode VA on each arch:
#   amd64:  somewhere in 0xfffff80000000000 .. 0xfffff80100000000 (Windows-style)
#   i386:   somewhere in 0x80000000 .. 0x90000000
#   arm64:  somewhere in 0xfffff80000000000 .. 0xfffff80100000000
#
# We scan in 0x10000 steps because Windows aligns image bases to 64KB.
# Once found, gdb gets `add-symbol-file <kernel_image> <runtime_text>` so
# every subsequent `bt`/`dump-thread`/etc. has DWARF symbols correctly mapped.
#
# Cached in $kbase convenience variable; inspect.sh reads it back into the
# manifest so subsequent calls skip the scan.

import gdb
import os


def _scan(start: int, end: int, step: int = 0x10000) -> int | None:
    inferior = gdb.selected_inferior()
    for addr in range(start, end, step):
        try:
            mem = inferior.read_memory(addr, 2)
            if bytes(mem) == b"MZ":
                return addr
        except gdb.MemoryError:
            continue
    return None


def find_kbase() -> int | None:
    arch = os.environ.get("VIBEOS_ARCH", "")
    if arch == "amd64":
        ranges = [(0xFFFFF80000000000, 0xFFFFF80100000000)]
    elif arch == "arm64":
        ranges = [(0xFFFFF80000000000, 0xFFFFF80100000000)]
    elif arch == "i386":
        ranges = [(0x80000000, 0x90000000)]
    else:
        gdb.write(f"[find-kbase] unknown arch: {arch}\n", gdb.STDERR)
        return None

    for lo, hi in ranges:
        hit = _scan(lo, hi)
        if hit is not None:
            return hit
    return None


PE_PREFERRED_IMAGE_BASE = 0x00400000


class FindKbaseCmd(gdb.Command):
    """find-kbase: scan kernel VA range for PE header, set $kbase, AND
    relocate symbols so backtraces resolve correctly.

    DWARF in ntoskrnl.exe is recorded relative to the file's preferred
    ImageBase (0x400000). The runtime base is wherever the loader actually
    placed the image (typically 0xfffff80000400000 on amd64). Without
    relocation, every PC gdb sees is "??" because the file VA address space
    and the runtime address space are disjoint. Loading with -o <offset>
    fixes this in one shot.
    """

    def __init__(self):
        super().__init__("find-kbase", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        # Allow callers to pass a cached kbase to skip the scan.
        cached = arg.strip()
        if cached:
            try:
                kbase = int(cached, 0)
            except ValueError:
                gdb.write(f"[find-kbase] invalid arg '{cached}'\n", gdb.STDERR)
                return
        else:
            kbase = find_kbase()
            if kbase is None:
                gdb.write("[find-kbase] not found in expected ranges\n", gdb.STDERR)
                return

        offset = kbase - PE_PREFERRED_IMAGE_BASE
        gdb.execute(f"set $kbase = {kbase:#x}", to_string=True)
        gdb.execute(f"set $kreloc = {offset:#x}", to_string=True)

        kernel_path = os.environ.get("VIBEOS_KERNEL", "")
        if kernel_path:
            # Reload with offset so all symbols match runtime VAs. We use
            # symbol-file (not add-symbol-file) to replace, not stack.
            gdb.execute(f"symbol-file {kernel_path} -o {offset:#x}", to_string=True)

        gdb.write(f"[find-kbase] kbase={kbase:#x} reloc={offset:#x}\n")


FindKbaseCmd()
