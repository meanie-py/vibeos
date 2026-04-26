# Key Bugs Fixed (All Sessions)

## SVC & Syscall
- **[amd64] `_AddressOfReturnAddress()` broken with clang**: mingw defines it as `&(((void**)__builtin_frame_address(0))[1])`, assuming standard `push rbp; mov rbp, rsp` prologue. But clang generates non-standard prologues (pushes callee-saved regs before setting frame pointer), making `__builtin_frame_address(0)` return an address 64 bytes off from the return address. This caused `KiSystemCallHandler` to access the wrong KTRAP_FRAME, making ALL user-mode syscalls fail (SMSS couldn't create process heap). **Fix**: Pass TrapFrame as a parameter from `KiSystemCallEntry64` assembly (`lea rcx, [rsp + MAX_SYSCALL_PARAM_SIZE]`) instead of computing it from the broken intrinsic. Files: `trap.S`, `traphandler.c`.
- **SVC dispatch truncates 64-bit syscall return values**: PSYSCALL_ROUTINE typedef used NTSTATUS (32-bit). Win32k syscalls returning LONG_PTR got truncated then sign-extended. Fixed: ULONG_PTR return type in trap.c.
- **GDI batch not flushed on ARM64 syscall entry**: User-mode GDI32 batches PatBlt+DeleteObj in TEB. Without flush at syscall entry, DeleteObject's direct syscall destroys brushes before batched PatBlt runs → all colors white. Fixed: Added GdiBatchCount check + KeGdiFlushUserBatch() call to KiSvcDispatch in trap.c (matches amd64 traphandler.c:197).
- **NtContinue crash (Thread->TrapFrame=NULL)**: KiSvcDispatch now sets Thread->TrapFrame.

## Memory Management
- **[amd64] BAD_POOL_HEADER (0x19) from `core::ptr::write_bytes` buffer overflow**: C-to-Rust translation of `MmMakeSegmentResident` (section.rs) preserved `* sizeof(PFN_NUMBER)` from C's `RtlZeroMemory(Pages, sizeof(PFN_NUMBER) * N)`. But Rust's `core::ptr::write_bytes<T>(dst, val, count)` writes `count * sizeof(T)` bytes (element count, not byte count), causing 8x buffer overflow on amd64 (sizeof(PFN_NUMBER)=8). The overflow zeroed adjacent pool page POOL_HEADERs during bochsmp.sys loading. **Fix**: `write_bytes(Pages, 0, BYTES_TO_PAGES(ReadLength))` — remove the `* sizeof(PFN_NUMBER)`. **Key lesson**: C→Rust translations must convert `RtlZeroMemory`/`memset` byte counts to element counts for `write_bytes`. File: `ntoskrnl-rust/src/mm/section.rs`.
- **BUGCHECK 0xC2 from MiGetPfnEntry NULL dereference**: MmDeleteKernelStack and MiDeletePte call MiGetPfnEntry() without NULL check. When PFN bitmap doesn't cover a page (firmware memory holes), MiGetPfnEntry returns NULL. Fixed: NULL guards in MmDeleteKernelStack (procsup.c) and MiDeletePte (virtual.c).
- **MmDeleteProcessAddressSpace**: `#ifndef _M_AMD64` → `#if (_MI_PAGING_LEVELS < 3)` in procsup.c.
- **MiSessionCommitPageTables crash**: `#ifndef _M_AMD64` → `#if (_MI_PAGING_LEVELS < 3)`.
- **Kernel stack overflow (BUGCHECK 0x1E)**: 12KB stack overflows with deep winsrv/user32/gdi32 init in csrss. Fixed: MmCreateKernelStack forces GuiStack=TRUE on ARM64 with full 60KB commit (procsup.c). Also set Thread->LargeStack=TRUE and StackLimit to full size in thrdobj.c.

## Exception Handling
- **DUMMY_PSEH kernel bugcheck**: Kernel-mode AV accessing user-mode pointer in _SEH2_TRY crashes because _SEH2_EXCEPT is dead code. Fixed: KSEH-SKIP in KiDispatchException skips faulting instruction when data address < MmHighestUserAddress.
- **User exception dispatch fails on demand-paged stack**: MmIsAddressValid returns FALSE for valid-but-not-present pages. Fixed: call MmAccessFault(0x02) to fault-in user stack pages before writing exception frame.
- **ExitProcess crash in RtlDestroyHeap**: Stack page freed during process teardown while thread still running. Workaround: NtTerminateProcess directly in cmd.exe ARM64 path.
- **KSEH-SKIP register zeroing** (stubs.c): When KSEH-SKIP skips a faulting load instruction, now zeros the destination register. Handles LDR/LDRB/LDRH/LDRSW and LDP variants.
- **ExAllocatePoolWithQuotaTag Raise crash**: DUMMY_PSEH callers can't catch RtlRaiseStatus. Fixed: `Raise = FALSE` on ARM64 in expool.c.

## Cache & Code Generation
- **KeSweepICache missing ARM64 cache maintenance**: Only did ISB, missing DC CVAU + IC IALLU. ATL thunks executed stale I-cache → STATUS_ILLEGAL_INSTRUCTION. Fixed with full cache maintenance sequence in ke.h.

## Process & Thread
- **NtCurrentTeb() returns 0**: ARM64 `inline_ntcurrentteb.h` had `return 0`. Fixed: `mov %0, x18` inline asm.
- **BaseInitializeContext missing ARM64**: kernel32 `utils.c` had no ARM64 case for thread CONTEXT init. Fixed: set X0, X1, Sp, Pc, ContextFlags.
- **NtRaiseHardError bugcheck for missing DLLs**: ARM64 delay-imports become regular imports (llvm-dlltool limitation). csrss loading winsrv.dll triggers NtRaiseHardError(STATUS_DLL_NOT_FOUND) for missing deps. Fixed: skip NtRaiseHardError on ARM64 in ldrutils.c.

## Win32k / GDI / Fonts
- **g_FreeTypeLock NULL in font cleanup**: winsrv not loaded → NtUserInitialize never called → g_FreeTypeLock stays NULL. Fixed: NULL check in `IntGdiCleanupPrivateFontsForProcess`.
- **IntRealizeFont uninitialized pFontObj crash**: `FONTOBJ *pFontObj` uninitialized in freetype.c. Fixed: `= NULL`.
- **CsrConnectToUser NULL function pointer**: Fixed in csrsrv/api.c with NULL guard.

## CRT / Locale
- **msvcrt _create_locale("C") failure**: Fixed: hardcoded ASCII CPINFO fallback in `msvcrt/mbcs.c:create_mbcinfo`.
- **msvcrt DllMain never called (i386)**: Some processes have msvcrt mapped but DllMain not invoked → heap, TLS, locale all uninitialized. Fixed: static C-locale fallback structs (`cloc_static_locinfo`, `cloc_static_mbcinfo`) in `dll/win32/msvcrt/locale.c`. `get_locinfo()` and `get_mbcinfo()` return fallbacks when `data->locinfo`/`data->mbcinfo` is NULL. `get_current_locale_noalloc()` also uses fallbacks. `grab_locinfo()` gets NULL checks for `lc_category[].refcount` and `lc_time_curr`. Refcounts set to 0x7FFFFFFF (pinned).

## GDI
- **GdiGetHandleUserData NULL GdiHandleTable (i386)**: Process loads comctl32 without user32 → GdiHandleTable never initialized (set by GdiProcessSetup called from user32 init). Fixed: NULL check for GdiHandleTable in `win32ss/gdi/gdi32/misc/misc.c`.

## Registry / Profile
- **Missing ProfileList values (i386)**: userenv.dll requires `AllUsersProfile` and `DefaultUserProfile` in `HKLM\SOFTWARE\...\ProfileList`. Fixed: added to `boot/bootdata/hivesft.inf`.
- **Missing USERPROFILE environment variable (i386)**: shell32 needs `%USERPROFILE%` for Desktop creation. Fixed: added USERPROFILE, ALLUSERSPROFILE, HOMEDRIVE, HOMEPATH to `boot/bootdata/hivesys.inf`.

## Build
- **SYSTEM hive not rebuilt**: `ninja bootcd_hives` builds SETUPREG.HIV, NOT the `system` hive used by ramdisk. Use `ninja boot/bootdata/system` for the ramdisk hive.
