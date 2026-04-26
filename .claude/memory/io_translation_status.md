---
name: IO subsystem translation status
description: Current state of C-to-Rust translation for ntoskrnl IO manager files, including known bugs and workarounds
type: project
---

IO subsystem translation status as of 2026-03-19.

**Active Rust files (19 + driver on all arches):**
adapter, bootlog, controller, debug, driver, deviface, error, iocomp, ioevent,
iomdl, iorsrce, iotimer, iowork, irq, rawfs, remlock, symlink, util, volume,
ramdisk (i386/amd64 only)

**pnpmgr active (amd64/arm64):** pnputil, pnpirp, pnpdma, pnpnotify, pnpmap,
devnode, plugplay, pnpreport, pnproot, pnpinit, pnpmgr (with IopGetBusTypeGuidIndex_c
and IopQueryDeviceCapabilities_c shims)

**Still in C with retranslated Rust (not yet activated):**
- file.rs (2891 lines) — retranslated, compiles clean, untested runtime
- iofunc.rs (2739 lines) — retranslated, compiles clean, untested runtime
- iomgr.rs (1110 lines) — retranslated, hangs on amd64
- ramdisk.rs — 2 bugs fixed (InitializeObjectAttributes, sprintf null), 0x6B remains on ARM64

**Still in C with known bugs:**
- irp.rs — 3 bugs found+fixed (NT_ERROR, KTHREAD offset, PacketType), more bugs in IopCompleteRequest
- device.rs — LLVM captures(none) optimizer issue
- devaction.rs — deep internal function graph bugs
- arcname.rs — partial split, i386 PE layout issue

**Key bug patterns found:**
1. NT_ERROR(x) is `(x>>30)==3`, NOT `x<0` (which is !NT_SUCCESS)
2. Bindgen `__bindgen_anon_1` can point to wrong union member at unexpected offsets
3. InitializeObjectAttributes is a C macro — must be inlined in Rust, not extern
4. sprintf/printf format strings need explicit null terminators in Rust byte arrays
5. LLVM cross-CGU captures(none) eliminates output pointer writes — use C shims
6. i386 PE section layout sensitivity — adding Rust modules can crash unrelated code

**Why:** Tracking progress to avoid repeating work across sessions.
**How to apply:** Check this before starting IO translation work. Update after changes.
