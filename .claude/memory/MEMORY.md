# ReactOS ARM64 Port - Key Notes

## Memory Index
- [io_translation_status.md](io_translation_status.md) — IO subsystem C→Rust translation progress, known bugs, workarounds
- [feedback_codex_cli.md](feedback_codex_cli.md) — Use OpenAI Codex CLI for C→Rust translations (cross-model, good quality)
- [feedback_no_auto_commit.md](feedback_no_auto_commit.md) — Don't commit without asking first
- [feedback_extern_c_abi.md](feedback_extern_c_abi.md) — extern "C" vs extern "system" ABI mismatch is #1 i386 crash cause
- [feedback_flexible_array.md](feedback_flexible_array.md) — Bindgen flexible arrays must use pointer arithmetic, not indexing
- [feedback_llvm_workarounds.md](feedback_llvm_workarounds.md) — LLVM i386 is more aggressive than amd64, structural fixes needed
- [feedback_verify_abi.md](feedback_verify_abi.md) — verify_abi.py tool for detecting extern "C" vs extern "system" mismatches
- [feedback_cgu1.md](feedback_cgu1.md) — CGU=1 investigation: ABI mismatches fixed, interprocedural contagion remains (15 ud2s)
- [feedback_no_blind_revert.md](feedback_no_blind_revert.md) — Never git checkout/restore to discard changes, always stash
- [feedback_inline_never.md](feedback_inline_never.md) — shim_exports #[inline(never)] is load-bearing, don't remove
- [feedback_volatile_static.md](feedback_volatile_static.md) — Volatile<T> on #[no_mangle] statics breaks linker sections

## CURRENT STATUS (as of 2026-03-22)
**RAII guard migration + output pointer hardening session.** ~300+ guard instances (9 types), ~500+ write_volatile conversions, Result pattern for Nt* syscalls. Pre-existing ud2 miscompilation discovered in IopAcquireFileObjectLock + 7 others (captures(none) on clean builds, masked by incremental). New tools: scan_ud2.py, lint_kernel.py, verify_sacred_primitives.sh. Smoke test conftest forces clean Rust rebuild. Key findings documented in rust-kernel-patterns.md (4 failed approaches, migration ceiling analysis, 3 LLVM defense layers).

## PREVIOUS STATUS (as of 2026-03-21)
**ALL 3 ARCHES PASS SMOKE TESTS 4/4.** Boot marker raised to "shell ready" (co_IntShowDesktop in win32k). Known regression: amd64 taskbar doesn't render (explorer.exe starts, desktop paints, but shell UI missing — see ~amd64-taskbar-regression.md). Key progress this session: 154 wrong constants fixed, iofunc.rs completed (Codex), devaction.rs PnP hang fixed (Codex: DEVICE_RELATIONS flexible-array bug), i386 boot fixed (Codex: extern "C" vs extern "system" ABI), ARM64 boot fixed (Codex: #[inline(never)] on PspCreateThread), pnpmgr enabled on i386 (Gemini: ABI audit + macro inlining), arcname.rs + arb/*.rs enabled on all arches. New tools: verify_abi.py, verify_ffi.py.

## PREVIOUS STATUS (as of 2026-03-19)
**amd64 + ARM64 pass smoke tests.** i386 has pre-existing regression. pnpinit.rs + pnpmgr.rs enabled on amd64/arm64, driver.rs enabled on all arches, debug.rs translated, irp.rs 3 bugs found, file.rs + iofunc.rs retranslated.

## PREVIOUS STATUS (as of 2026-03-13, last all-green)
**ALL 3 ARCHITECTURES PASS SMOKE TESTS 4/4.** i386 + amd64 + ARM64 all boot to desktop. Key fixes this session: (1) ARM64 MmProbeAndLockPages `let mut Operation = IoReadAccess` variable shadowing bug — LLM translated parameter reassignment as new local, so kernel-mode MDL locking still checked write permissions → STATUS_ACCESS_VIOLATION → BUGCHECK 0x1E. Fix: `mut Operation` parameter + `Operation = IoReadAccess`. (2) amd64 Rust usercall.rs has undiagnosed return-address corruption — workaround: use C usercall.c for amd64. (3) All debug instrumentation removed from 7 files.

## Project State
- Branch: `arm64-port`, **zero build failures for all three architectures** (arm64, amd64, i386)
- **Unified llvm-mingw toolchain**: `toolchain-llvm-mingw.cmake` builds i386/amd64/arm64 from macOS or Linux (WSL2)
- **ARM64**: Milestones 1-31 achieved + **Rust SList + Bitmap**, full desktop, virtio input, serial console
- **Rust in Kernel**: SList + Bitmap + AVL Table + **Pool Allocator** + **Full ob subsystem** + **Full se subsystem** + **Full ps subsystem** + **Tier 1 subsystems** (lpc, dbgk, po, inbv, wmi, vdm, vf, rtl = 26 files) + **Tier 2: CC + FSRTL** (24 files, KD64 kept in C) + **MM Tier 1+2** (20 files) + **MM Tier 3b** (10 files) + **Full EX subsystem** (31 files) + **Full KE subsystem** (62 files). All 3 arches via kernel-rust umbrella. GNU/LLVM targets, not MSVC.
- **Rust CC + FSRTL subsystems**: CC (7 files: cacheman, copy, fs, lazywrite, mdl, pin, view) + FSRTL (17 files). i386 + arm64 smoke tests pass. amd64 has pre-existing baseline issue (STATUS_IN_PAGE_ERROR in uniata.sys during bochsmp.sys loading — DUMMY_PSEH can't catch kernel-mode exceptions). Key fixes: ROS_SHARED_CACHE_MAP/ROS_VACB struct field ordering in view.rs; `core::ptr::write_bytes` buffer overflow in MmMakeSegmentResident (section.rs) — C's `RtlZeroMemory` takes byte count but Rust's `write_bytes` takes element count, LLM translation preserved `* sizeof(PFN_NUMBER)` causing 8x overflow that zeroed adjacent pool headers.
- **Rust MM Tier 1+2**: 20 files total (ARM3: contmem, drvmgmt, dynamic, hypermap, iosup, kdbg, largepag, mmdbg, mmsup, ncache, special, syspte, vadnode, zeropage; mm: balance, freelist, marea, mmfault, mminit, pagefile, region, rmap, shutdown). i386 + arm64 smoke tests pass. Key fix: MMPTE_LIST bit layout differs per arch — ARM64 has OneEntry at bit 6 and NextEntry at bits [63:28] (36-bit), not bit 1 and bits [63:32] like amd64.
- **Rust MM Tier 3b**: 10 files total (ARM3: mdlsup, mminit, pagfault, pfnlist, procsup, section, session, sysldr, virtual; mm: section). All 3 arches build + i386/arm64 smoke tests pass. wslist.cpp stays in C (no Rust translation). section_pe.c stays in C (PE format parser, not translated). `virtual` is a Rust reserved keyword — use `#[path = "virtual.rs"] pub mod r#virtual;` in mod.rs. VEH recursive exception fix: `#ifndef _NTOSKRNL_` guards in sdk/lib/rtl/{amd64,i386}/except.c around VEH handler calls.
- **KD64 kept in C**: Rust KD64 causes recursive exception flood on amd64 (INT3 → KdpTrap → KdpReport → KdSendPacket infinite nesting). Root cause unknown — code logic appears identical to C. Key shim file: `kd64_rust_shims.c` with `#ifndef KD64_USE_C` guard for KdpDprintf and `!defined(CC_USE_C)` for ExpKdbgExt* KDBG functions.
- **Rust Tier 1 subsystems**: 26 files (lpc 6, dbgk 2, po 4, inbv 2, wmi 2, vdm 2, vf 2, rtl 6). All 3 arches build + smoke tests pass. Key shim file: `tier1_rust_shims.c` (IO macros, LPC inlines, HAL dispatch table, VdmState, KeRaiseIrql/KeLowerIrql on i386).
- **Rust ps subsystem**: All 13 C files translated to Rust (apphelp, debug, job, kill, process, psmgr, psnotify, query, quota, security, state, thread, win32). All 3 arches build + smoke tests pass. Key fixes: i386 fat-LTO ud2 workaround via SepInitTokensShim C shim, missing TEB→PEB linkage in PspCreateProcess (ArbitraryUserPointer = Process->Peb).
- **Rust EX subsystem**: All 31 C files translated to Rust (atom, callback, dbgctrl, efi, event, evtpair, exintrin, fmutex, handle, harderr, hdlsterm, init, interlocked, keyedevt, locale, lookas, mutant, profile, pushlock, resource, rundown, sem, shutdown, sysinfo, time, timer, uuid, win32k, work, xipdisp, zone). All 3 arches build + i386/arm64 smoke tests pass. Key shim file: `ex_rust_shims.c` (~25 FORCEINLINE/macro shims using Phase 1 rename pattern). Key fixes: pushlock contradictory `#[cfg]` guards (LLM emitted `#[cfg(x86)] #[cfg(not(x86))]` on same fn → compiled on NO arch), resource.rs cli/sti arch-gated for ARM64, rundown/pushlock/exintrin i386 fastcall exports via `fastcall_export!` macro, ~15 globals needed `#[no_mangle] pub`.
- **Rust KE subsystem**: All 62 C files translated to Rust (26 generic: apc, balmgr, bug, clock, config, devqueue, dpc, eventobj, except, freeze, gate, gmutex, ipi, krnlinit, mutex, processor, procobj, profobj, queue, semphobj, spinlock, thrdobj, thrdschd, time, timerobj, wait; 15 i386: abios, context, cpu, exp, freeze, irqobj, kiinit, ldt, mproc, mtrr, patpge, thrdini, traphdlr, usercall, v86vdm; 16 amd64: context, cpu, except, freeze, interrupt, ipi, irql, kiinit, krnlinit, mproc, spinlock, stubs, thrdini, traphandler, usercall, xstate; 5 arm64: kiinit, stubs, thrdini, trap, usercall). All 3 arches build + i386/arm64 smoke tests pass. Key shim file: `ke_rust_shims.c` (~2080 lines, ~70 FORCEINLINE shims from ke_x.h). Key fixes: i386 calling convention decorations (fastcall for KxAcquireSpinLock/KxReleaseSpinLock, stdcall for KiStartUnexpectedRange/DbgUnLoadImageSymbols, cdecl for KiExceptionExit), static mut fn pointers→extern fn declarations, NtCurrentProcess macro→ntkrnl_sys::macros function.
- **Rust Pool Allocator**: All 3 arches. All smoke tests pass. Two critical bugs fixed: MMFREE_POOL_ENTRY Owner marking range, MMPFN struct size.
- **Rust ob subsystem**: All 12 C files translated to Rust. All 3 arches build + smoke tests pass.
- **Rust se subsystem**: All 18 C files translated to Rust (access, accesschk, acl, audit, client, debug, objtype, priv_, sd, semgr, sid, sqos, srm, subject, token, tokenadj, tokencls, tokenlif). All 3 arches build + smoke tests pass. Key fixes: ~200 constants added to ntkrnl-sys, probe.h macro shims, cross-file #[no_mangle] for Sep* functions, NTAPI on all shim Phase 5 functions for i386 stdcall.
- **kernel-rust umbrella crate** (`sdk/lib/rust/kernel-rust/`): Single `staticlib` that depends on `reactos-pool` (rlib) + `ntoskrnl-rust` (rlib). One panic_handler, one set of compiler_builtins.
- **rustconv pipeline**: Multi-stage C-to-Rust translation. Stage 1: C→Rust (Sonnet). Stage 2: probe migration (Haiku). Stage 3: SEH idiomation (planned). See [rustconv.md](rustconv.md) for details.
- **Rust probe API** (`ntkrnl-sys/src/probe.rs`): Result-returning probe functions replacing bare ProbeForRead/Write. SE (12 files, 62 sites) + OB (6 files, 35 sites) migrated via `rustconv probe-migrate` pipeline. All 3 arches build + smoke tests pass.
- **SEH idiomation pipeline** (planned): Function-level C+Rust paired transformation using dependency graph (SQLite cache), RAII guards, safe_copy_to/from_user, Result<T,NTSTATUS> patterns. Will use tree-sitter for both C and Rust AST extraction. Targets SE+OB first as control, then PS.
- **i386**: **Desktop boots — smoke test passes 4/4.** LiveCD used for smoke tests. Key fixes: ExReturnPoolQuota process-pointer NULL-out, KeWaitForMultipleObjects WaitBlock init from KxMultiThreadWait macro, 50ms delay HACK in co_IntInitializeDesktopGraphics (winsta.c), plus all previous fixes (MXCSR, FX_SAVE_AREA, etc.).
- **amd64**: **Desktop boots — smoke test passes 4/4.** LiveCD used for smoke tests. Uses C usercall.c (Rust version has undiagnosed return-address corruption). Smoke test marker: `explorer.exe created`.
- ARM64 boot: FreeLDR → ntoskrnl → Phase1 → IoInitSystem → SMSS → win32k → csrss → winlogon → explorer
- ARM64 stable with interactive prompt, zero crashes, zero bugchecks

## Multi-Arch Boot (i386 + amd64)
- **Unified toolchain**: `toolchain-llvm-mingw.cmake` — one file for i386/amd64/arm64
- **Build dirs**: `build-i386/`, `build-amd64/`, `build-arm64/`
- **QEMU**: `qemu/run.sh --arch i386|amd64|arm64` — boots from livecd.iso (i386/amd64) or UEFI ramdisk (arm64)
- **FREELDR_SECTION_COUNT** (mm.h): Clang/LLD produces more sections than GCC. i386→5, amd64→6 (.text .rdata .buildid .data [.pdata] .reloc)
- **BSS zeroing fix** (entry.S for both i386 and amd64): LLD merges .bss into .data (VirtualSize > RawDataSize). `__bss_start__`/`__bss_end__` markers don't work with PE $ suffix ordering. Fix: scan PE section headers at runtime, zero VirtualSize-RawDataSize gaps.
  - i386: uses `___ImageBase` (triple underscore — LLD adds i386 `_` prefix)
  - amd64: uses `__ImageBase` (double underscore — no prefix on 64-bit)
- **LLD i386 forwarder decoration bug**: LLD prepends `_` to PE export forwarder targets on i386. Fix in FreeLDR (peloader.c) and ntoskrnl (sysldr.c): strip leading `_` from forwarder DLL name.
- **NO_ROSSYM for USE_LLVM_MINGW** (gcc.cmake): rsym host tool crashes on Clang-generated binaries. Skip rsym, use dwarf-based build.
- **PSEH_CLANG for i386** (gcc.cmake): Setjmp/longjmp-based SEH for i386 Clang. Always catches via longjmp, evaluates filter in original function context (avoids PSEH3's ESP-relative issue). Files: `sdk/lib/pseh/i386/pseh_clang.c`, `pseh_clang_asm.S`, new section in `pseh2.h`. ARM64/amd64 keep DUMMY_PSEH.
- **MXCSR sanitization**: FXRSTOR can #GP if MXCSR has reserved bits set. Fix in `ke_rust_shims.c:Ke386FxStore` and `ctxswitch.S:Ke386LoadFpuState`: sanitize MXCSR before FXRSTOR using MXCSR_MASK (or default 0xFFBF).
- **FX_SAVE_AREA struct bug** (cpu.rs): LLM translation had wrong field order `{Cr0NpxState, NpxSavedCpu, Data[512]}` vs correct C layout `{U[512], NpxSavedCpu, Cr0NpxState}`. Fixed.

## i386 Desktop Boot Fixes (Milestone 32)
- **KernelCallbackTable guard** (win32ss/user/ntuser/main.c): Check Process->Peb->KernelCallbackTable before co_IntClientThreadSetup. Prevents crash when user32 DllMain hasn't run yet.
- **Exception swallowing** (ntdll/dispatch/i386/dispatch.S): For DUMMY_PSEH builds, swallow software exceptions (code < 0xC0000000) in KiUserExceptionDispatcher instead of raising second-chance. Hardware exceptions (ACCESS_VIOLATION etc.) still propagate normally.
- **Explorer.exe path** (winlogon.c): Use `GetWindowsDirectoryW()` to build full path — explorer.exe is in `%SystemRoot%\` not `%SystemRoot%\System32\`.
- **Default boot option** (bootsup.c): Changed default from `/FASTDETECT` to `/DEBUG /DEBUGPORT=COM1 /BAUDRATE=115200 /SOS` for serial debug output.
- **QEMU i386**: `qemu-system-i386 -nodefaults -cpu pentium3 -m 256 -vga std -hda HDD -boot order=c -nographic -serial mon:stdio`. Use `sh -c` wrapper to avoid sandbox drive conflicts. 2-stage install: CD→HDD format+copy, then HDD boot.
- **msvcrt DllMain-never-called fix** (lock.c, heap.c, locale.c): Some service processes (PID=276) have msvcrt.dll mapped but DllMain never called. Fixed 3 subsystems:
  - **lock.c**: `_lock()` infinite recursion when `_LOCKTAB_LOCK` uninitialized → lazy init `_LOCKTAB_LOCK` in `_lock()` before recursive call
  - **heap.c**: `heap` handle is NULL → lazy init `heap = GetProcessHeap()` in `msvcrt_heap_alloc/realloc/free/size`
  - **locale.c**: `update_thread_locale` early return when `MSVCRT_locale == NULL`; `grab_locinfo` NULL-checks `lc_category[i].refcount` and `lc_time_curr`; static `cloc_static_locinfo`/`cloc_static_mbcinfo` fallbacks in `get_current_locale_noalloc`
- **GdiGetHandleUserData guard** (gdi32/misc/misc.c): NULL check for GdiHandleTable (set by user32 init, not gdi32). Processes loading gdi32 without user32 had NULL KCT.
- **Registry ProfileList** (hivesft.inf): Added `AllUsersProfile` and `DefaultUserProfile` values. **Environment vars** (hivesys.inf): Added USERPROFILE, ALLUSERSPROFILE, HOMEDRIVE, HOMEPATH.
- **Result**: Zero unhandled exceptions, zero bugchecks. Screenshot: `qemu/i386_desktop.png`

## amd64 Desktop Boot Fixes (DUMMY_PSEH)
- **Root cause**: DUMMY_PSEH makes ALL `_SEH2_EXCEPT` handlers dead code on amd64 (same as ARM64/i386). Three bugs:
- **RtlRaiseException swallowing** (sdk/lib/rtl/amd64/except_asm.S): When `RtlDispatchException` found no handler, `RtlRaiseException` called `ZwRaiseException(FirstChance=FALSE)` → direct second-chance → process killed. Fix: for continuable software exceptions (code < 0xC0000000, not NONCONTINUABLE), call `ZwContinue` instead.
- **KiUserExceptionDispatcher swallowing** (dll/ntdll/dispatch/amd64/dispatch.S): Same pattern — defense in depth for kernel-dispatched user-mode exceptions.
- **DbgPrint output eaten** (sdk/lib/rtl/debug.c): `vDbgPrintExWithPrefixInternal` checks `RtlpCheckForActiveDebugger()` (PEB.BeingDebugged). If TRUE, raises `DBG_PRINTEXCEPTION_C` via `RtlRaiseException`. The `_SEH2_EXCEPT` should set `Handled=FALSE` to fall through to `DebugPrint` (the actual `int 0x2d` serial output). With DUMMY_PSEH, EXCEPT is dead → swallowed exception leaves `Handled=TRUE` → `DebugPrint` never called → all user-mode DbgPrint silently lost. Fix: `#ifndef _USE_DUMMY_PSEH` around the debugger notification block.
- **Page table hierarchy** (ntoskrnl/mm/i386/page.c): `MiEnsureKernelPdeValid` needed for amd64 4-level paging — was only implemented for 2-level.
- **Font loading** (winlogon.c): Removed `_M_ARM64` guard so amd64 also loads system fonts.
- **Spooler load timing** (gdi32/objects/dc.c): Moved spooler DLL load after display driver check.
- **Result**: Full LiveCD desktop. Screenshot: `qemu/amd64_desktop.png`

## i386 Boot Fixes (GAS Intel Syntax Bugs)
- **Critical pattern**: In GAS `.intel_syntax noprefix`, `mov eax, label` does MEMORY LOAD (reads bytes at label), NOT address load. Must use `lea eax, [label]` or `mov eax, offset label`.
- **systimer.S** (3 fixes): `lea eax, [handler]` instead of `mov eax, handler` / `push handler` for IDT handler setup
- **trap.s:135** (KiInterruptTemplate): `lea eax, [@KiInterruptTemplateHandler@8]` — was loading prologue bytes (0x57E58955 = push ebp;mov ebp,esp;push edi) as address → crash on every hardware interrupt
- **scsiport.spec**: Removed i386 forwarded exports (HAL.xxx, NTOSKRNL.xxx) to avoid lld `_` prefix bug. Use stubs.c functions for all architectures.
- **sysldr.c MiSnapThunk**: Strip leading `_` from PE forwarded export DLL names (lld workaround)
- **generictable.c**: Guard against corrupt splay tree root (DUMMY_PSEH can't catch page faults in SEH-protected code paths)

## kernel-rust Umbrella Crate + ob/obhandle Wiring
- **kernel-rust** (`sdk/lib/rust/kernel-rust/`): Single `staticlib` that depends on `reactos-pool` (rlib) + `ntoskrnl-rust` (rlib). One panic_handler, one set of compiler_builtins. Avoids duplicate symbol errors from multiple Rust staticlibs.
- **CMakeLists.txt**: `cargo build -p kernel-rust` replaces `cargo build -p reactos-pool`. Uses `RUST_KERNEL_LIB`/`rust_kernel` target names.
- **ntos.cmake**: `obhandle.c` commented out, `ob_rust_shims.c` added (19 FORCEINLINE/macro stubs for Rust FFI).
- **ob/obhandle.rs** (5,910 lines): Sonnet-translated, `#[no_mangle] pub unsafe extern "system"` on all exported functions. `ObGetProcessHandleCount` + `ObpSetHandleAttributes` needed manual export annotation.
- **MiAddressToPte fix** (mm.rs): Critical bug in committed code — `(addr >> 12) * 8 + PTE_BASE` overflows for kernel VAs on 64-bit. Fix: `(addr >> 9) & (0xFFFFFFFFF << 3) + PTE_BASE` (39-bit mask). Also fixed MiPteToAddress (signed shift), MiPdeToPte (was calling MiAddressToPte instead of MiPteToAddress), added MiPdeToAddress.
- **expool.rs fix**: Changed `KeAcquireSpinLockForDpc` → `KeAcquireSpinLock` (and release) for `ExpLargePoolTableLock`. ForDpc produced WARNING spam flooding serial output.
- **Stale build artifacts gotcha**: CMake DEPENDS tracks source files but cargo caches its own artifacts. Delete `rust-kernel-target/` + `libkernel_rust.a` to force fresh Rust build after source changes.
- **Status**: All 3 arches build, ARM64 smoke tests pass, amd64 Phase 1 MM init completes. Boot continues past pool init (no hang/crash). Full desktop boot not yet verified with Rust pool (amd64 desktop boot was with C pool).

## ARM64 Milestones (summary)
- M19-21: Serial UART console, framebuffer display, KeUserModeCallback
- M22-23: WM_PAINT, font rendering (FreeSans.ttf), GDI batch flush
- M27: Explorer.exe desktop shell (blue desktop, taskbar, clock)
- M28: Interactive serial console, scheduler starvation fixes
- M29: Notepad with keyboard input via serial
- M30: Mouse input via serial UART (host→PL011→arm64input.sys→RIT)
- M31: Virtio-input driver (real mouse+keyboard from QEMU `--gui`)
- M11-18: Win32k, winlogon, csrss, kernel32, CreateProcessW, cmd.exe

## Key Bugs Fixed
See [bugfixes.md](bugfixes.md) for detailed list. Most common patterns:
- `#ifndef _M_AMD64` → `#if (_MI_PAGING_LEVELS < 3)` (ARM64 4-level paging)
- DUMMY_PSEH: _SEH2_EXCEPT never runs → must validate before faulting ops
- Missing NULL checks for ARM64-specific uninitialized state
- **C macro→function shim bugs**: KxMultiThreadWait macro sets local `WaitBlock` variable; function shim can't. Must manually set `WaitBlock = WaitBlockArray.add(Count-1)` after each call.
- **ExReturnPoolQuota missing NULL-out** (expool.rs): C code clears process pointer after quota return to prevent double-return from lookaside; Rust translation omitted this line → QUOTA_UNDERFLOW bugcheck.
- **MmProbeAndLockPages Operation shadowing** (mdlsup.rs): LLM translated `Operation = IoReadAccess` (parameter reassignment) as `let mut Operation = IoReadAccess` (new local). Kernel-mode MDL locking still checked write permissions → BUGCHECK 0x1E on ARM64.
- **Stale Rust build artifacts**: CMake doesn't track Rust source changes. Must `rm libkernel_rust.a` + deps to force cargo rebuild after modifying Rust sources.

## Ramdisk Boot Setup
- `qemu/mkramdisk.sh` creates ramdisk.img (80MB+ FAT32) + boot.img (160MB)
- `freeldr.ini`: `SystemPath=ramdisk(0)\reactos`, `Options=/RDPATH=\ramdisk.img`
- Update: `mcopy -o -i qemu/ramdisk.img <file> ::<path>` then `mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img`
- **Do NOT mcopy uefildr.efi into boot.img** after ramdisk — corrupts FAT32
- **SYSTEM hive**: `ninja boot/bootdata/system` (not `ninja bootcd_hives`)

## ARM64 SVC Dispatch
- `KiSvcDispatch` in `trap.c`: x8=syscall ID, SSDT lookup, PreviousMode from SPSR
- Thread->TrapFrame linking: saved/restored around syscall dispatch
- GDI batch flush for Win32k syscalls (TableIndex==1, user-mode)

## Rust SList (Cross-Platform — All 3 Architectures)
- **Workspace**: `sdk/lib/rust/` — Cargo workspace with `slist` + `slist-loom` members
- **Targets**: `aarch64-pc-windows-gnullvm`, `x86_64-pc-windows-gnu`, `i686-pc-windows-gnu`
- **Replaces**: arm64/slist.S (LDXP/STXP), amd64/slist.S (CMPXCHG16B), i386/interlck.S (CMPXCHG8B)
- **Architecture**: `header.rs` (cfg per pointer width) → `algorithm.rs` (pure transitions, unchanged) → `atomics.rs` (unified load/try_store/cancel per target_arch) → `lib.rs` (cfg per arch for FFI + calling convention)
- **i386 specifics**: Pop in `global_asm!` (fault labels required by KiCheckForSListFault: bytes 0x8B 0x18). Push/Flush as `#[no_mangle] extern "stdcall"`. Flush preserves sequence (i386 asm convention).
- **amd64**: Always Header16 (dropped Header8 path). Empty fault label symbols for ntdll.spec.
- **64-bit HeaderType/Init fix**: `pack()` ORs in bits 0-1 when non-empty — required by `RtlFirstEntrySList` (slist.c:56).
- **CMake**: `sdk/lib/rtl/CMakeLists.txt` — builds for all architectures (no ARM64-only guard)
- **Verification**: 20 unit tests + 9 Kani proofs + 3 Loom concurrency tests + 29 kernel C tests pass
- **Rust inline asm gotcha**: LLVM reserves rbx (amd64) and ebx (i386 PIC). Use `xchg` workaround for CMPXCHG16B/CMPXCHG8B. `#[no_mangle] extern "stdcall"` correctly produces `_Name@N` on i686-gnu; `#[export_name]` with pre-decorated names gets double-decorated.

## Test Suite
- **Rust host tests**: `cd reactos/sdk/lib/rust && cargo test` (unit + Kani + Loom)
- **QEMU kernel tests**: `pytest tests/qemu/test_kernel.py --test-kernel` (28 tests via serial dispatch)
- **QEMU smoke tests**: `pytest tests/qemu/test_smoke.py --arch arm64|i386|amd64` — see [smoke-tests.md](smoke-tests.md)
  - Boot media: livecd.iso (i386/amd64), UEFI ramdisk (arm64)
  - North star: `explorer.exe created` (i386/amd64), idle detection (arm64)
  - Debugcon triage: 0xE9 port markers decoded against JSON trace catalogs on failure
  - KVM acceleration for i386/amd64 via `-accel kvm -cpu host`
- **Python venv**: `reactos/.venv/bin/python -m pytest ...`

## Build System
- **Linux (WSL2)**: Source at `/home/curi/Projects/os-playground/reactos/`
- **Toolchain**: llvm-mingw at `/home/curi/Projects/os-playground/llvm-mingw/` (x86_64-linux)
- **Unified toolchain**: `reactos/toolchain-llvm-mingw.cmake` — i386/amd64/arm64 from Linux
- **Build dirs**: `build-arm64/` (0 failures), `build-amd64/` (0 failures), `build-i386/` (0 failures)
- **16-bit assembly**: System `i686-w64-mingw32-gcc` for boot sectors/frldr16/MVDM
- **KVM acceleration**: Available in WSL2 (`/dev/kvm`), used for i386/amd64 QEMU configs
- **Smoke tests**: All 3 arches pass 4/4 (as of 2026-03-13). See [smoke-tests.md](smoke-tests.md)
- **Key asm.inc fixes**: Uppercase macros guarded by `#ifdef __clang__` (GNU as is case-insensitive), `retf` → `.byte 0xCB`, `.pushframe code` → `.seh_pushframe @code`

## ARM64 Input Pipeline
- **arm64input.sys**: PointerClass0 + KeyboardClass0 (keyboard IOCTL + mouse IOCTL + virtio MMIO)
- **serialterm.c**: Escape sequence parser for mouse, raw bytes for keyboard
- Host control: `qemu/mouse_click.py <x> <y>` via Unix socket `qemu/serial.sock`
