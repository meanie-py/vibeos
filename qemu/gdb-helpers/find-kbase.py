# Locate the runtime kernel base by scanning memory for the PE "MZ" signature
# AND verifying the image's export-directory name matches the kernel we loaded.
#
# ntoskrnl.exe loads at a fixed kernel-mode VA on each arch:
#   amd64:  somewhere in 0xfffff80000000000 .. 0xfffff80100000000 (Windows-style)
#   i386:   somewhere in 0x80000000 .. 0x90000000
#   arm64:  somewhere in 0xfffff80000000000 .. 0xfffff80100000000
#
# We scan in page-size steps. The usual PE image alignment is 64KB, but the
# i386 UEFI path can place ntoskrnl on a 4KB boundary (observed at 0x80491000).
#
# IMPORTANT: more than one PE image lives in the kernel scan range. On i386,
# HAL (hal.dll) loads *below* ntoskrnl (e.g. 0x80090000) and is the FIRST MZ a
# naive low-to-high scan hits — relocating ntoskrnl symbols to HAL's base makes
# every subsequent `bt`/breakpoint resolve to garbage. So we don't trust the
# first MZ: at each candidate we parse the PE export directory and only accept
# the image whose export Name matches the basename of VIBEOS_KERNEL (ntoskrnl).
#
# Cached in $kbase convenience variable; inspect.sh reads it back into the
# manifest so subsequent calls skip the scan.
#
# THE SAME TRAP EXISTS OUTSIDE THIS FILE. Fixing the scan does not help anyone
# resolving a kernel symbol by other means: `build-<arch>/ntoskrnl/*.exe` matches
# only the UP `ntoskrnl.exe`, while the media boots `ntkrnlmp.exe` from one
# directory deeper (`ntoskrnl/ntkrnlmp/`). A glob or tab-complete in the obvious
# place hands you the wrong kernel and it ANSWERS rather than erroring — measured:
# `ntoskrnl.exe+0x3BBA4` is `KeSetIdealProcessorThread+0x1c` in the UP image and
# `RtlRaiseStatus+0x44` in the MP one, and only the latter fits the bugcheck it
# came from. See docs/engineering/agent-reference.md.

import gdb
import os


def _rd(inferior, addr, n):
    try:
        return bytes(inferior.read_memory(addr, n))
    except gdb.MemoryError:
        return None


def _u16(b):
    return int.from_bytes(b, "little")


def _u32(b):
    return int.from_bytes(b, "little")


def _pe_info(inferior, base: int) -> tuple[str | None, int | None]:
    """Parse the PE at `base` and return its export-directory Name string,
    plus SizeOfImage, or (None, None) if it isn't a well-formed PE32 image.

    Layout (PE32):
      base+0x3c            -> e_lfanew (u32)
      base+e_lfanew        -> "PE\0\0"
      +4                   -> IMAGE_FILE_HEADER (20 bytes)
      +24                  -> IMAGE_OPTIONAL_HEADER32
      +24+0x38             -> SizeOfImage (u32)
      +24+0x60             -> DataDirectory[0] export table
      DataDirectory[0]     -> {ExportRVA (u32), Size (u32)}
      base+ExportRVA+0x0C  -> Name RVA (u32)
      base+NameRVA         -> NUL-terminated ASCII module name
    """
    mz = _rd(inferior, base, 2)
    if mz != b"MZ":
        return None, None
    e_lfanew_b = _rd(inferior, base + 0x3C, 4)
    if e_lfanew_b is None:
        return None, None
    e_lfanew = _u32(e_lfanew_b)
    if e_lfanew > 0x1000:  # sane bound; PE header sits in the first page
        return None, None
    sig = _rd(inferior, base + e_lfanew, 4)
    if sig != b"PE\x00\x00":
        return None, None
    opt = base + e_lfanew + 24
    size_b = _rd(inferior, opt + 0x38, 4)
    if size_b is None:
        return None, None
    size = _u32(size_b)
    # The export DataDirectory sits at a different optional-header offset for
    # PE32 (0x60) vs PE32+ (0x70): PE32+ widens ImageBase/SizeOf*Reserve/Commit
    # to 8 bytes (and drops BaseOfData), shifting NumberOfRvaAndSizes+the data
    # directories by 0x10. amd64/arm64 ntoskrnl is PE32+; i386 is PE32. Using the
    # PE32 offset on amd64 read a garbage export name → ntoskrnl never matched.
    magic_b = _rd(inferior, opt, 2)
    magic = _u16(magic_b) if magic_b is not None else 0
    dd_off = 0x70 if magic == 0x20B else 0x60
    exp_rva_b = _rd(inferior, opt + dd_off, 4)
    if exp_rva_b is None:
        return None, size
    exp_rva = _u32(exp_rva_b)
    if exp_rva == 0:
        return None, size
    name_rva_b = _rd(inferior, base + exp_rva + 0x0C, 4)
    if name_rva_b is None:
        return None, size
    name_rva = _u32(name_rva_b)
    if name_rva == 0:
        return None, size
    raw = _rd(inferior, base + name_rva, 32)
    if raw is None:
        return None, size
    nul = raw.find(b"\x00")
    if nul >= 0:
        raw = raw[:nul]
    try:
        return raw.decode("ascii").lower(), size
    except UnicodeDecodeError:
        return None, size


# The multiprocessor kernel's image stem. Every media carries both kernels and
# boots this one by default, so it outranks the UP image when both are mapped.
MP_KERNEL_STEM = "ntkrnlmp"


def _current_pc() -> int | None:
    """$pc, or None if the target has none right now."""
    try:
        return int(gdb.parse_and_eval("$pc")) & 0xFFFFFFFFFFFFFFFF
    except (gdb.error, ValueError):
        return None


def _scan(start: int, end: int, want: str, step: int = 0x1000):
    """Scan [start,end) for a PE whose export name matches `want`.

    Returns (base, name) of the match, or (None, None). We keep scanning past
    the first MZ — the first image found is usually HAL, not ntoskrnl.

    More than one image can match `want`, and taking the first one is wrong.
    Measured on arm64 (2026-09-13): the RUNNING kernel sits at 0xfffff80042190000
    (what the guest itself prints as [KRNLBASE], and where $pc was), but the scan
    returned a second ntoskrnl image at 0xfffff800505e4000 — 0xe million bytes
    higher, and not the one executing. Every symbol then relocates to that image,
    so `run-until.sh --break <symbol>` sets breakpoints at addresses the guest
    never executes and they SILENTLY never fire; `bt` resolves to garbage the
    same way. It looks like "breakpoints don't work on arm64".

    The likely mechanism is that the real image is skipped rather than rejected:
    `_pe_info` gives up when it cannot READ the export directory, and a paged-out
    export table in the live image is enough for that, while a fully-resident
    second copy parses fine.

    So $pc decides when it can. If we are stopped inside one of the candidate
    images, that image is the one executing — which is the question being asked —
    and no amount of export-name matching outranks it.
    """
    inferior = gdb.selected_inferior()
    pe_candidates = []
    name_matches = []
    for addr in range(start, end, step):
        mz = _rd(inferior, addr, 2)
        if mz != b"MZ":
            continue
        name, size = _pe_info(inferior, addr)
        if size is None:
            continue
        pe_candidates.append((addr, name, size))
        if name is not None and want in name:
            name_matches.append((addr, name, size))

    pc = _current_pc()
    if pc is not None:
        # Only a KERNEL image can win this way. Stopped inside the HAL (an idle i386 guest
        # halts in it), "the image containing $pc" is halmp, and relocating kernel symbols
        # there broke every helper. A nameless image may be a kernel whose export table is
        # paged out, so it still qualifies.
        kernels = (want, MP_KERNEL_STEM)
        containing = [
            c
            for c in pe_candidates
            if c[0] <= pc < c[0] + c[2]
            and (c[1] is None or any(k in c[1].lower() for k in kernels))
        ]
        if containing:
            # Innermost/highest base wins if images somehow nest.
            addr, name, _size = max(containing, key=lambda c: c[0])
            if name_matches and addr != name_matches[0][0]:
                gdb.write(
                    f"[find-kbase] $pc {pc:#x} is inside {addr:#x} "
                    f"('{name or '<no-export-name>'}'), NOT the first export-name "
                    f"match {name_matches[0][0]:#x} ('{name_matches[0][1]}') — "
                    "trusting $pc. Symbols relocated to the wrong image resolve to "
                    "garbage and breakpoints silently never fire.\n",
                    gdb.STDERR,
                )
            return addr, name or "<no-export-name>"

    # $pc is only decisive when it happens to be inside one of the images, and it
    # often is not (an EL1 vector, HAL, a driver). Fall back to the name — but
    # the MP kernel outranks the UP one, because EVERY media now carries BOTH and
    # boots MP by default (DefaultOS=LiveCD_SMP / ReactOS_Arm64_MP, 2026-09-12).
    # VIBEOS_KERNEL still names the UP image, so `want` is "ntoskrnl" and a naive
    # first-match lands on a kernel the guest is not running. Measured on arm64:
    # picked ntoskrnl at 0xfffff800505e4000 while ntkrnlmp was executing at
    # 0xfffff80042190000 — so every relocated symbol was wrong and every
    # breakpoint silently never fired.
    mp_matches = [c for c in pe_candidates if c[1] and MP_KERNEL_STEM in c[1]]
    if mp_matches and name_matches and mp_matches[0][0] != name_matches[0][0]:
        gdb.write(
            f"[find-kbase] both kernels are loaded; preferring MP "
            f"'{mp_matches[0][1]}' at {mp_matches[0][0]:#x} over "
            f"'{name_matches[0][1]}' at {name_matches[0][0]:#x} (media boots MP by "
            "default). Override with VIBEOS_KERNEL if this boot is UP.\n",
            gdb.STDERR,
        )
    if mp_matches and MP_KERNEL_STEM not in want:
        return mp_matches[0][0], mp_matches[0][1]
    if name_matches:
        return name_matches[0][0], name_matches[0][1]
    # No name match (stripped/rewritten export table?): fall back to the
    # largest valid PE image, not the first MZ. On i386 the first PE is commonly
    # kdcom/HAL below ntoskrnl, and relocating kernel symbols there makes every
    # follow-on helper read garbage.
    if pe_candidates:
        pe_candidates.sort(key=lambda item: item[2], reverse=True)
        addr, name, size = pe_candidates[0]
        name_s = name or "<no-export-name>"
        gdb.write(
            f"[find-kbase] no export name matched '{want}'; "
            f"falling back to largest PE {addr:#x} ('{name_s}', size={size:#x})\n",
            gdb.STDERR,
        )
        return addr, name_s
    return None, None


def _is_running() -> bool:
    """True if any thread of the current inferior is still executing.

    Probing memory is NOT a reliable liveness test: QEMU's gdbstub happily
    services read_memory while the vCPUs are running (especially under
    -accel kvm), so a successful read does not imply the target is halted.
    The thread state API is authoritative."""
    try:
        for t in gdb.selected_inferior().threads():
            if t.is_running():
                return True
    except gdb.error:
        pass
    return False


def _ensure_stopped() -> None:
    """Halt the target before scanning. `target remote` to a KVM-accelerated
    QEMU does not always stop the vCPUs, and a moving target gives racy /
    partial memory reads — which is how the scan used to miss ntoskrnl and
    latch onto whichever lower image (kdcom/HAL) happened to be readable."""
    if not _is_running():
        return
    try:
        gdb.execute("interrupt", to_string=True)
    except gdb.error:
        pass
    # `interrupt` is asynchronous; give the stop event a chance to land so the
    # subsequent scan reads a frozen address space.
    for _ in range(50):
        if not _is_running():
            return
        try:
            gdb.execute("", to_string=True)  # pump event loop
        except gdb.error:
            pass


def find_kbase():
    arch = os.environ.get("VIBEOS_ARCH", "")
    want = os.path.splitext(os.path.basename(os.environ.get("VIBEOS_KERNEL", "ntoskrnl.exe")))[
        0
    ].lower()
    if arch == "amd64":
        ranges = [(0xFFFFF80000000000, 0xFFFFF80100000000)]
    elif arch == "arm64":
        ranges = [(0xFFFFF80000000000, 0xFFFFF80100000000)]
    elif arch == "i386":
        ranges = [(0x80000000, 0x90000000)]
    else:
        gdb.write(f"[find-kbase] unknown arch: {arch}\n", gdb.STDERR)
        return None

    _ensure_stopped()
    for lo, hi in ranges:
        hit, name = _scan(lo, hi, want)
        if hit is not None:
            gdb.write(f"[find-kbase] matched '{name}' at {hit:#x}\n")
            return hit, name
    return None, None


def _symbols_for(kernel_path: str, matched: str | None) -> str:
    """The image whose symbols belong at the base we found.

    _scan prefers the MP kernel's BASE when both are mapped, but VIBEOS_KERNEL still names the
    UP file, and loading the UP file's DWARF at the MP base made every symbol and struct offset
    wrong (list-threads: "DWARF struct offsets unavailable"). Follow the match to its sibling.
    """
    if not kernel_path or not matched:
        return kernel_path
    stem = os.path.splitext(os.path.basename(kernel_path))[0].lower()
    if MP_KERNEL_STEM in matched.lower() and MP_KERNEL_STEM not in stem:
        mp = os.path.join(os.path.dirname(kernel_path), MP_KERNEL_STEM, MP_KERNEL_STEM + ".exe")
        if os.path.isfile(mp):
            return mp
    return kernel_path


PE_PREFERRED_IMAGE_BASE = 0x00400000


class FindKbaseCmd(gdb.Command):
    """find-kbase: scan kernel VA range for the ntoskrnl PE header, set $kbase,
    AND relocate symbols so backtraces resolve correctly.

    DWARF in ntoskrnl.exe is recorded relative to the file's preferred
    ImageBase (0x400000). The runtime base is wherever the loader actually
    placed the image. Without relocation, every PC gdb sees is "??" because the
    file VA address space and the runtime address space are disjoint. Loading
    with -o <offset> fixes this in one shot.
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
        matched = None
        if not cached:
            kbase, matched = find_kbase()
            if kbase is None:
                gdb.write("[find-kbase] not found in expected ranges\n", gdb.STDERR)
                return

        offset = kbase - PE_PREFERRED_IMAGE_BASE
        gdb.execute(f"set $kbase = {kbase:#x}", to_string=True)
        gdb.execute(f"set $kreloc = {offset:#x}", to_string=True)

        kernel_path = _symbols_for(os.environ.get("VIBEOS_KERNEL", ""), matched)
        if kernel_path:
            os.environ["VIBEOS_KERNEL"] = kernel_path  # later helpers read it too
            # Reload with offset so all symbols match runtime VAs. We use
            # symbol-file (not add-symbol-file) to replace, not stack.
            gdb.execute(f"symbol-file {kernel_path} -o {offset:#x}", to_string=True)

        gdb.write(f"[find-kbase] kbase={kbase:#x} reloc={offset:#x}\n")


FindKbaseCmd()
