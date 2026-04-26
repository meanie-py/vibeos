---
name: IO subsystem Rust translation status
description: Current state of iomgr C-to-Rust translation — 20/27 files active, remaining 7 have runtime bugs
type: project
---

## IO IOMGR — 20/27 files active in Rust (as of 2026-03-15)

**Active (Rust replacing C, all 3 arches pass smoke tests):**
adapter, bootlog, controller, deviface, error, iocomp, ioevent, iomdl, iorsrce, iotimer, iowork, irq, rawfs, remlock, symlink, util, volume + driver/ramdisk (non-ARM64)

**Why:** iorsrce had KEY_INFORMATION_CLASS enum mixup (FullInfo=2 not 1). deviface had 28 `b"..."` byte-string-as-wide-string bugs. Both fixed.

**How to apply:** arcname.rs provides IopReassignSystemRoot/IopVerifyDiskSignature while C arcname.c provides IopCreateArcNames — use `#define ARCNAME_RUST_REASSIGN` + `#ifndef` guards. Currently arcname.rs is DISABLED (crashes i386).

**Blocked (7 files, translated but disabled):**
- **file.rs** — 20 functions translated by agent, crashes both arches. NtCreateFile/NtOpenFile/IoCheckShareAccess etc. have subtle runtime bugs.
- **iofunc.rs** — 8 Nt* syscalls translated, 3 bugs fixed (FILE_WRITE_TO_END_OF_FILE, buffer zeroing, FileMaximumInformation). Untested standalone — need to validate.
- **arcname.rs** — IopReassignSystemRoot/IopVerifyDiskSignature pass amd64 but crash i386. Calling convention fixed (extern "C" → extern "system"), IsEqualGUID inlined.
- **irp.rs** — BUGCHECK 0x1E (STATUS_IN_PAGE_ERROR in KiInsertQueueApc). Unknown root cause.
- **device.rs** — LLVM cross-CGU `captures(none)` optimizer bug. C shims written but untested.
- **iomgr.rs** — LLVM codegen bug in IopInitLookasideLists loop.
- **debug.c** — not translated (535 lines, debug-only).

**Key lessons:**
- Agent-translated complex IO dispatch functions (NtReadFile, NtCreateFile etc.) consistently compile but have subtle runtime bugs
- The rustconv LLM pipeline hits 32K token limits on large files → truncated translations
- `b"..."` byte strings as wide strings is a systematic LLM translation bug
- Enum constant mixups (KEY_INFORMATION_CLASS vs KEY_VALUE_INFORMATION_CLASS) are hard to catch
- Running multiple agents concurrently that modify mod.rs/ntos.cmake causes conflicts
