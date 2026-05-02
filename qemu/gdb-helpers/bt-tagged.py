# Backtrace with translation-boundary tags: [Rust] / [C] / [ASM] / [???].
#
# GDB's stock `bt` output drops the source-file extension, which is exactly
# the signal we need to spot Rust↔C boundary issues fast (KeGdiFlushUserBatch,
# COMPUTE_HASH_CHAR, the wait-block setup chain).
#
# Tag rules (see kernel-debugging-infra.md §8):
#     .rs              -> [Rust]
#     .c, .cpp         -> [C]
#     .S, .asm, .s     -> [ASM]
#     no DWARF info    -> [???]   ← the genuinely valuable tag
#
# global_asm! blocks live in .rs files and tag as [Rust]; that's correct —
# the tag is provenance, not implementation language.

import gdb
import os


def _tag_for_source(src: str | None) -> str:
    if src is None:
        return "[???]"
    ext = os.path.splitext(src)[1].lower()
    if ext == ".rs":
        return "[Rust]"
    if ext in (".c", ".cpp", ".cc", ".cxx", ".h", ".hpp"):
        return "[C]"
    if ext in (".s", ".asm"):
        return "[ASM]"
    return "[???]"


def _frame_source(frame: gdb.Frame) -> str | None:
    sal = frame.find_sal()
    if sal is None or sal.symtab is None:
        return None
    return sal.symtab.filename


def _frame_label(frame: gdb.Frame, idx: int) -> str:
    pc = frame.pc()
    name = frame.name() or "<no symbol>"
    src = _frame_source(frame)
    tag = _tag_for_source(src)
    sal = frame.find_sal()
    line = sal.line if (sal and sal.line) else 0
    src_short = os.path.basename(src) if src else "??"
    return f"#{idx:<2} {tag:<7} {pc:#018x}  {name}  ({src_short}:{line})"


class BtTaggedCmd(gdb.Command):
    """bt-tagged [N]: backtrace with [Rust]/[C]/[ASM]/[???] tags. N defaults to 30."""

    def __init__(self):
        super().__init__("bt-tagged", gdb.COMMAND_USER)

    def invoke(self, arg: str, from_tty: bool) -> None:
        max_frames = int(arg.strip()) if arg.strip() else 30
        try:
            frame = gdb.newest_frame()
        except gdb.error as e:
            gdb.write(f"bt-tagged: no frame ({e})\n", gdb.STDERR)
            return
        idx = 0
        while frame is not None and idx < max_frames:
            gdb.write(_frame_label(frame, idx) + "\n")
            try:
                frame = frame.older()
            except gdb.error:
                break
            idx += 1


BtTaggedCmd()
