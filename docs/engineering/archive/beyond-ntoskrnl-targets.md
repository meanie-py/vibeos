# Beyond ntoskrnl — Rust Translation Targets

Candidates for C→Rust translation outside ntoskrnl, ordered by strategic value. Excludes Wine-imported libraries and user applications (notepad, calc, etc.).

## Tier 1: Boot-Critical, Small, High Impact

These are small components on the boot-critical path. A bug here = no boot. Rust's safety guarantees have the highest leverage.

### HAL (hal/halx86) — 71 files, 23K lines
Hardware Abstraction Layer. Interrupt handling, timer, DMA, APIC, SMP init. Loaded by FreeLDR before ntoskrnl. Tightly coupled to kernel — same address space, same PE format, same calling conventions.

**Why translate:** HAL bugs are catastrophic (no IRQ = no scheduling). Already familiar territory — same build system, same FFI patterns as ntoskrnl Rust. The HAL is a `.dll` linked into kernel space, so it can use the same `ntkrnl-sys` bindings.

**Complexity:** Medium. Heavy inline asm (port I/O, MSR, CPUID), interrupt controllers (APIC/PIC). ARM64 HAL (`hal/halarm`) is only 2K lines — could do that first.

| Subcomponent | Files | Lines | Notes |
|---|---|---|---|
| generic/ | 23 | 7,498 | Core: timer, DMA, sysinfo, reboot |
| legacy/ | 8 | 4,790 | PC/AT: PIC, RTC, CMOS |
| mp/ | 7 | 3,058 | SMP bootstrap |
| acpi/ | 4 | 2,686 | ACPI tables, power management |
| apic/ | 7 | 1,908 | Local/IO APIC |
| pic/ | 4 | 1,572 | 8259 PIC |
| halarm/ | 20 | 2,103 | ARM HAL (tiny, good starter) |

### SMSS (base/system/smss) — 9 files, 6.5K lines
Session Manager. First user-mode process after kernel init. Creates CSRSS and WINLOGON. Small, well-defined, critical for boot.

**Why translate:** Small enough for a single translation pass. On the critical boot path (kernel → SMSS → CSRSS → winlogon → explorer). Native NT process (no Win32 dependencies), so pure Nt* API — clean FFI surface.

### CSRSRV (subsystems/csr/csrsrv) — 7 files, 7K lines
Client-Server Runtime subsystem server. Manages the CSRSS process — API dispatch, session management, thread/process support. Core infrastructure that SMSS creates.

**Why translate:** Small, critical, pure NT. CSRSS crash = system crash (BSOD 0xEF).

### ksecdd (drivers/crypto/ksecdd) — 5 files, 1.4K lines
Kernel Security Device Driver. Loaded during boot (`ksecdd.sys`). Provides kernel-mode crypto services.

**Why translate:** Tiny, boot-critical, security-sensitive. Perfect starter driver project.

## Tier 2: Core Infrastructure, Medium Size

### NTDLL (dll/ntdll) — 14 files, 11.7K lines
NT Layer DLL. Lowest user-mode library. Raw syscall wrappers (Nt*), PE loader (LDR), exception dispatch, runtime library. Every process loads ntdll.dll.

**Why translate:** Universal attack surface (loaded in every process). The LDR (5 files, 9.2K lines) is the most complex part — DLL loading, import resolution, TLS init. Exception dispatch is arch-specific (different for i386/amd64/arm64).

**Complexity:** High. Must produce a PE DLL that works as the first loaded module. Calling convention subtleties (user-mode, mixed cdecl/stdcall). Can't use Rust std (no allocator yet when ntdll loads).

### win32k — ntuser + ntgdi (98 files, 114K lines)
Kernel-side Win32 subsystem. Window manager (`ntuser/`, 59 files, 73K lines) + GDI rendering (`ntgdi/`, 39 files, 41K lines). Loaded into kernel space as `win32k.sys`.

**Why translate:** Largest kernel attack surface after ntoskrnl. GDI/window manager bugs are common security vulnerabilities. Same PE-COFF kernel-mode environment as ntoskrnl — can reuse all existing Rust infrastructure.

**Complexity:** Very high. 114K lines, deep internal state (window tree, GDI handle table, DC/brush/pen objects). Would be a multi-month project. Best approach: translate subsystem-by-subsystem (start with simple ntgdi functions like bitmap ops).

| Subcomponent | Files | Lines | Notes |
|---|---|---|---|
| ntuser/ | 59 | 73,165 | Window manager, messaging, input |
| ntgdi/ | 39 | 41,471 | GDI rendering, fonts, DCs |
| eng/ | 40 | 19,192 | Graphics engine, surface/palette |
| diblib/ | 29 | 2,715 | DIB operations |

### FreeLDR (boot/freeldr) — 116 files, 49.8K lines
Bootloader. Runs before kernel, handles multiboot, loads ntoskrnl, detects memory, sets up page tables. Architecture-specific boot code for i386/amd64/arm64.

**Why translate:** Boot security is critical (a compromised bootloader owns everything). FreeLDR runs in a constrained environment (no OS services), so `#![no_std]` Rust is a natural fit.

**Complexity:** Very high. 16-bit real mode code (i386 boot sectors — these stay in ASM). PE loading, FAT/ISO9660/ext2 filesystem reading, memory map construction, EFI support. Would need a custom Rust target or `#![no_std]` with manual memory management.

## Tier 3: Important System Services

### Winlogon (base/system/winlogon) — 11 files, 7.9K lines
Handles user login, desktop creation, SAS (Ctrl+Alt+Del), auto-logon. Security-sensitive (credential handling).

### SCM / services.exe (base/system/services) — 9 files, 12.2K lines
Service Control Manager. Manages all Windows services. Controls startup order, dependencies, failure recovery.

### winsrv (win32ss/user/winsrv) — 38 files, 25K lines
Win32 subsystem server (runs inside CSRSS). Console management, hard error handling, shutdown coordination. Split into usersrv (user operations) and consrv (console).

### umpnpmgr (base/services/umpnpmgr) — 4 files, 7K lines
User-mode PnP manager. Handles device installation, driver loading from user-mode side. Companion to the kernel PnP manager we already translated.

## Tier 4: Drivers (Selective)

→ Moved to **`driver-translation-audit.md`** (the `drivers/` fleet status +
candidate targets + the cdrom KMDF / storage read-stack sequencing live there).

## Size Comparison: ntoskrnl vs win32k

| | C files | C lines | Complexity |
|---|---|---|---|
| **ntoskrnl (total)** | 406 | 298K | Very high — arch-specific asm, spinlocks, IRQLs, SEH everywhere |
| **ntoskrnl Rust (translated)** | 310 .rs | 364K | ~80% of ntoskrnl is now Rust |
| **win32k kernel core (ntuser+ntgdi+eng)** | 138 | 134K | High — massive global state, less asm |
| **win32ss (all)** | 492 | 338K | Comparable to full ntoskrnl |

### win32k Complexity vs ntoskrnl ke/io

| Factor | ntoskrnl ke/io | win32k ntuser/ntgdi |
|---|---|---|
| SEH blocks | 1,050 across ntoskrnl | 526 (ntuser 344 + ntgdi 182) |
| Largest file | iofunc.c 4.6K | menu.c 6.7K, freetype.c 7.9K |
| Global mutable state | Heavy per-subsystem | Very heavy (286 statics in ntuser) |
| Inline asm | Yes (ke/ arch-specific) | Minimal |
| Spinlocks/IRQLs | Pervasive | Some (win32k lock, DC lock) |
| Bindgen traps | Yes (DEVICE_RELATIONS) | Likely (GDI object arrays) |

**Bottom line:** win32k is **easier than ke/ but harder than io/** in different ways. Less asm and fewer calling convention issues, but the window manager is one giant mutable state machine (windows, desktops, message queues, input threads, hooks, timers all cross-referencing via global lists — similar to MM, the hardest ntoskrnl subsystem). FreeType font engine (7.9K lines) should probably stay in C.

**Best entry points:** `gdi/eng/` (19K lines, pure computation graphics primitives) or ntgdi bitmap/brush ops. Window manager (ntuser/) would be hardest.

## NTDLL Deep Dive — 14 files, 11.7K lines

NTDLL is the lowest user-mode library — loaded into every process before anything else. It provides the raw NT syscall wrappers, PE loader, and exception dispatch.

| Subcomponent | Files | Lines | Description |
|---|---|---|---|
| **ldr/** | 5 | 9,226 | PE loader — DLL loading, import resolution, TLS init, load order |
| **rtl/** | 3 | 1,532 | Runtime helpers (user-mode libsupp, version) |
| **dbg/** | 1 | 387 | Debug UI helpers (DbgUi*) |
| **dispatch/** | 1 | 89 | Syscall dispatcher stub |

**Complexity:** Medium-high. Only 30 SEH blocks (much less than ntoskrnl). The PE loader (`ldr/`) is 79% of the code and the hardest part — manages the module load order list, handles DLL forwarding, TLS callbacks, activation contexts, and the infamous "loader lock". It runs before the heap is initialized, so memory management is tricky.

**Key challenge:** NTDLL can't use Rust std — it IS the lowest runtime. Would need `#![no_std]` with manual allocation (or hooking into RtlAllocateHeap after heap init). The syscall dispatchers are arch-specific ASM (already separate `.asm` files).

**Security value:** Very high — every process loads ntdll.dll, so bugs here are universal attack surface. The loader is a classic source of DLL hijacking and preloading vulnerabilities.

## Remaining ntoskrnl Components (Not Yet Translated)

These are the parts of ntoskrnl still in C:

| Component | Files | Lines | Notes |
|---|---|---|---|
| **config/ (Registry CM)** | 24 | 18.8K | Registry hive manager. Complex B-tree + cell allocator. Depends on cmlib (8.8K shared library). |
| **kd+kd64/ (Debugger)** | 20 | 9.1K | Kernel debugger. KD64 Rust version caused recursive exception flood — kept in C. |
| **fstub/ (Disk partitioning)** | 4 | 5.8K | Disk layout, partition table parsing. |
| **cache/section/ (Section objects)** | 13 | 6K | Section object internals (shared with MM). |

## Security Stack — LSASS + SAM + Auth

| Component | Files | Lines | Description |
|---|---|---|---|
| **lsasrv** (dll/win32/lsasrv) | 16 | 14.9K | LSA Server — local security authority, policy, privileges |
| **samsrv** (dll/win32/samsrv) | 12 | 16.5K | SAM Server — user account database |
| **msv1_0** (dll/win32/msv1_0) | 7 | 3.3K | NTLM authentication package |
| **secur32** (dll/win32/secur32) | 13 | 7.6K | Security Support Provider Interface (SSPI) |
| **advapi32** (dll/win32/advapi32) | 30 | 32.1K | Advanced API — registry, security, crypto, service control (ReactOS-native, NOT Wine) |
| **lsass** (base/system/lsass) | 1 | 87 | LSASS process (thin wrapper, loads lsasrv) |

This is the entire authentication/authorization chain. All ReactOS-native code. High security value for translation but needs user-mode Rust infrastructure.

## Shared Libraries (Used by Both Kernel and User Mode)

| Component | Files | Lines | Description |
|---|---|---|---|
| **sdk/lib/rtl** | 81 | 57.1K | Runtime Library — heap, unicode, path, registry, thread pool. Shared between ntoskrnl and ntdll. Key files: heap.c (4.2K), actctx.c (6.2K), unicode.c (2.9K), path.c (3K) |
| **sdk/lib/cmlib** | 13 | 8.8K | Registry hive format library — used by ntoskrnl/config, FreeLDR, and setup tools |
| **sdk/lib/crt** | 596 | 53.7K | C runtime — mostly imported/standard, not a translation target |

RTL is interesting because we've already translated ntoskrnl's RTL subset — translating sdk/lib/rtl would give Rust RTL to ALL components (kernel + user-mode).

## NOT Recommended for Translation

| Component | Reason |
|---|---|
| **explorer.exe** | 22 files, 14K lines of C++ — uses COM, OLE, Shell APIs. Would need C++ interop, not just C FFI |
| **subsystems/mvdm (ntvdm)** | DOS/16-bit emulator, 34K lines. Legacy subsystem, low priority |
| **Wine-derived DLLs** | user32, gdi32, kernel32 client-side — maintained as Wine imports |

(Not-recommended *drivers* — tcpip/lwIP, ACPICA core, btrfs, ext2 — are in
`driver-translation-audit.md`.)

## Translation Infrastructure Reuse

The existing rustconv pipeline + ntkrnl-sys bindings can be directly reused for:
- **HAL** — same kernel address space, same calling conventions, same PE format
- **win32k** — same kernel address space, loaded as .sys driver
- **Kernel-mode drivers** (.sys) — same PE-COFF, same `extern "system"` FFI

Would need new infrastructure for:
- **User-mode DLLs** (ntdll, kernel32) — different Rust target, different linking, different runtime
- **User-mode executables** (smss, winlogon, services) — simpler than DLLs but still need NT API bindings
- **FreeLDR** — custom `#![no_std]` environment, possibly new Rust target

## Suggested Roadmap

**Phase 1 — Finish ntoskrnl + kernel-adjacent (reuses existing infra):**
1. ntoskrnl config/ (18.8K lines — registry CM, last major untranslated subsystem)
2. ntoskrnl fstub/ (5.8K lines — disk partitioning)
3. HAL/halarm (2K lines — immediate, same build system)
4. ksecdd.sys (1.4K lines — tiny boot driver)
5. HAL/halx86 generic (7.5K lines — core HAL)

**Phase 2 — win32k (same kernel-mode infra):**
6. win32k gdi/eng (19K lines — graphics primitives, pure computation)
7. win32k gdi/ntgdi selective (bitmap, brush, pen — ~10K lines)
8. win32k user/ntuser selective (input, cursor, hooks — ~15K lines)
9. win32k full (114K lines total — long campaign)

**Phase 3 — User-mode foundation (needs new Rust target):**
10. SMSS (6.5K lines — first user-mode Rust process)
11. NTDLL ldr/ (9.2K lines — PE loader, high security value)
12. CSRSRV (7K lines — CSRSS infrastructure)

**Phase 4 — Security stack (high value, medium size):**
13. msv1_0 (3.3K lines — NTLM auth package)
14. lsasrv (14.9K lines — LSA server)
15. samsrv (16.5K lines — user account database)
16. advapi32 (32.1K lines — ReactOS-native, not Wine)

**Phase 5 — Drivers + shared libs:**
17. fastfat.sys (68.8K lines — primary filesystem)
18. sdk/lib/rtl (57.1K lines — shared runtime, benefits ALL components)
19. cdfs.sys (32.4K lines — LiveCD filesystem)
