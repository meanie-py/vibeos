# IO Manager — Disabled Rust Translations

Files that have Rust translations but remain in C due to bugs. Each entry documents
the known issues and what's needed to fix them.

## irp.rs (irp.c — 2027 lines)

**Status:** 3 bugs found and fixed, still hangs on amd64. More bugs in IopCompleteRequest.

**Bugs found (all fixed in source):**
1. **`NT_ERROR` macro wrong** — Defined as `status < 0` which catches both warnings
   (0x80xxxxxx) AND errors (0xC0xxxxxx). The correct definition is
   `((status as u32) >> 30) == 3` which only catches errors (severity 3).
   This changes IRP completion control flow: warning status codes like
   STATUS_BUFFER_OVERFLOW take the error path, silently dropping IRP completions.

2. **KTHREAD offset in KeInitializeApc** (lines 608, 1315, 1333) —
   `Tcb.__bindgen_anon_1` gives ApcState (offset 72 on amd64), not KTHREAD start.
   Fixed to `&mut (*Thread).Tcb as *mut _`.

3. **PacketType constant wrong** (line 624) — `IoPagingPriorityHigh` (2) instead of
   `IopCompletionPacketIrp` (0). Affects IO completion port packet identification.

**Remaining issue:** IopCompleteRequest (~300 lines) has additional bugs causing
amd64 hang at mountmgr.sys. The function handles buffered IO copy-back, file object
dereferencing, completion port posting, user APC insertion, and MDL cleanup — all
with complex struct field accesses through bindgen unions. Needs careful line-by-line
retranslation or a C shim.

**Category:** Translation bugs (not compiler/LLVM issues).

---

## iomgr.rs (iomgr.c — 765 lines)

**Status:** Retranslated with improved prompt. Hangs on amd64, untested elsewhere.

**Bugs found:**
1. **LLVM `ud2` after IopInitializePlugPlayServices** — LLVM eliminates code after
   the call because the return value is discarded. Fixed with `core::hint::black_box()`.
2. **Missing `#[no_mangle]` on globals** — IopFileMapping, IopSecurityResource,
   IopLoaderBlock needed `#[no_mangle] pub`.
3. **InitializeObjectAttributes as extern** — This is a C macro, not a function.
   Fixed by inlining the struct initialization.

**Remaining issue:** Still hangs on amd64 after all fixes. Likely more LLVM
cross-CGU optimization issues or additional translation bugs in IoInitSystem
(the main IO initialization function).

**Category:** Mix of LLVM issues and translation bugs.

---

## device.rs (device.c — 1973 lines)

**Status:** Not recently tested. Known "LLVM optimizer bug" from previous sessions.

**Known issue:** LLVM eliminates success path after ObCreateObject output-pointer
pattern (same captures(none) issue as IopGetBusTypeGuidIndex).

**Category:** LLVM compiler issue. Needs C shim for affected functions.

---

## file.rs (file.c — 4202 lines)

**Status:** Crashes both arches. Not recently investigated.

**Known issue:** NtCreateFile and related Nt* syscalls have translation bugs.
Large file with complex SEH patterns and user-mode pointer validation.

**Category:** Translation bugs. Likely needs retranslation with improved prompt.

---

## iofunc.rs (iofunc.c — 4646 lines)

**Status:** SMSS crash. Not recently investigated.

**Known issue:** Nt* syscalls (NtReadFile, NtWriteFile, NtDeviceIoControlFile, etc.)
have translation bugs causing SMSS to crash during session initialization.

**Category:** Translation bugs. Largest IO file, will need careful work.

---

## ramdisk.rs (ramdisk.c — 307 lines)

**Status:** Retranslated. Works on i386/amd64 (never called). Crashes on ARM64.

**Bugs found (fixed in source):**
1. **`InitializeObjectAttributes` as extern** — Called a C shim that had calling
   convention mismatch on ARM64. Fixed by inlining the struct initialization.
2. **`sprintf` format string missing null** — `[b'X', b':', b'%', b's']` had no `\0`,
   causing `sprintf` to read past the buffer. Fixed by adding null terminator.

**Remaining issue:** After fixing the above, bugcheck changes from 0x9C
(RAMDISK_BOOT_INITIALIZATION_FAILED) to 0x6B (PROCESS1_INITIALIZATION_FAILED).
The ramdisk init now succeeds but something in the subsequent SystemRoot/drive
letter setup is wrong. Needs further investigation.

**Category:** PE binary layout issue (compiler), NOT a translation bug. Even with
both C and Rust compiled (C wins), ARM64 crashes — the extra Rust code in the
binary changes PE layout enough to trigger 0x6B. Same pattern as i386 PE sensitivity.
Translation bugs (InitializeObjectAttributes, sprintf null, SharedUserData) are
all fixed but won't help until the layout issue is resolved.

---

## driver.rs (driver.c — 2314 lines)

**Status:** Active on amd64 (cfg(not(aarch64))). Crashes on ARM64 Phase0.

**Known issue:** Phase0 crash on ARM64 only. Works on amd64. Needs ARM64 debugging.

**Category:** Unknown — could be translation or ARM64-specific.

---

## arcname.rs (arcname.c — 994 lines)

**Status:** Partially split (2 functions Rust, 2 functions C). Crashes on i386.

**Known issue:** The i386 crash is likely the PE section layout sensitivity issue
that affects all i386 Rust modules. May work if the i386 layout issue is resolved.

**Category:** Likely i386 PE layout issue (compiler), not translation bug.

---

## devaction.rs (devaction.c — 2750 lines)

**Status:** Stays in C on all arches.

**Known issue:** Deep bugs in internal static function graph (PipDeviceActionWorker
and callees). Even with ALL exported functions delegated to C shims, the hang
persists because C shims call Rust internal functions. The entire file's internal
logic has undiagnosed bugs.

**Category:** Translation bugs (many interacting static functions).

---

## Common bug patterns found across all files

1. **NT_ERROR vs NT_SUCCESS** — `NT_ERROR(x)` is `(x >> 30) == 3`, NOT `x < 0`.
   `x < 0` is `!NT_SUCCESS(x)` which includes warnings.
2. **Bindgen anonymous union offsets** — `__bindgen_anon_1` can point to the wrong
   union member, adding unexpected offsets (e.g., KTHREAD.ApcState at offset 72).
3. **Wrong enum constants** — Match arms using sequential ordinals instead of actual
   enum values. DeviceProperty, KeyValueInformationClass both hit this.
4. **LLVM captures(none)** — Output pointer writes eliminated across codegen units.
   Workaround: C shim wrappers.
5. **Wide string encoding** — `b"..."` byte strings used as UTF-16. Must be `[u16; N]`.
6. **Static lifetime** — Inline `[u16; N]` arrays as temporaries → dangling pointers.
