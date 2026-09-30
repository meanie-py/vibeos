# lm [addr]: list loaded kernel modules (PsLoadedModuleList), or name the one holding addr.
#
# A fault PC in system-PTE space (0xf2bbb6b4 on i386, say) belongs to a driver the kernel
# loaded at run time, and nothing else in the toolkit could say which: find-kbase only
# relocates ntoskrnl, and the serial log records which drivers load but not where.
# Offset-free apart from LDR_DATA_TABLE_ENTRY's head, which has been fixed since NT 3.1:
# three LIST_ENTRYs, then DllBase, EntryPoint, SizeOfImage, FullDllName, BaseDllName.
# Needs find-kbase to have relocated ntoskrnl's symbols first.
# Every helper is sourced into ONE gdb Python namespace, so module-level names are prefixed:
# debug_block.py also defines `_read` (returning bytes), and the later source wins.
import os

import gdb


def _lm_ptr_size() -> int:
    return 4 if os.environ.get("VIBEOS_ARCH", "").startswith("i386") else 8


def _lm_read(addr: int, n: int) -> int | None:
    try:
        return int.from_bytes(bytes(gdb.selected_inferior().read_memory(addr, n)), "little")
    except gdb.MemoryError:
        return None


def _lm_ustr(addr: int, ps: int) -> str:
    length = _lm_read(addr, 2)
    buf = _lm_read(addr + ps, ps)  # Buffer follows Length/MaximumLength, pointer-aligned
    if not length or not buf:
        return "?"
    try:
        raw = bytes(gdb.selected_inferior().read_memory(buf, min(length, 512)))
    except gdb.MemoryError:
        return "?"
    return raw.decode("utf-16-le", "replace")


def _lm_list_head() -> int | None:
    for name in ("PsLoadedModuleList", "_PsLoadedModuleList"):
        try:
            return int(gdb.parse_and_eval(f"(unsigned long long)&{name}"))
        except gdb.error:
            continue
    return None


def _lm_modules():
    ps = _lm_ptr_size()
    head = _lm_list_head()
    if head is None:
        raise gdb.GdbError("lm: no PsLoadedModuleList symbol (run find-kbase first)")
    dll_base, size_of_image = 6 * ps, 8 * ps
    full_name = (size_of_image + 4 + ps - 1) & ~(ps - 1)  # UNICODE_STRING, pointer-aligned
    base_name = full_name + 2 * ps  # 0x2c on i386, 0x58 on 64-bit
    node = _lm_read(head, ps)
    seen = 0
    while node and node != head and seen < 512:
        base = _lm_read(node + dll_base, ps) or 0
        size = _lm_read(node + size_of_image, 4) or 0
        yield base, size, _lm_ustr(node + base_name, ps)
        node = _lm_read(node, ps)
        seen += 1


class LmCmd(gdb.Command):
    """lm [addr]: list loaded modules, or name the module (and RVA) containing addr."""

    def __init__(self):
        super().__init__("lm", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        want = int(gdb.parse_and_eval(arg)) & 0xFFFFFFFFFFFFFFFF if arg.strip() else None
        hit = False
        for base, size, name in _lm_modules():
            if want is None:
                gdb.write(f"{base:#018x} {size:#10x} {name}\n")
            elif base <= want < base + size:
                gdb.write(f"{want:#x} = {name}+{want - base:#x} (base {base:#x})\n")
                hit = True
        if want is not None and not hit:
            gdb.write(f"{want:#x} is in no loaded module\n")


LmCmd()
