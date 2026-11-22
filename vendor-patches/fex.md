# Vendored FEX-Emu snapshot (vibeos)

**Upstream:** https://github.com/FEX-Emu/FEX, tag **FEX-2607**
(commit `1cc4b93e7a71c883ec021b71359f136394dc1f3c`, snapshot 2026-07-08). **License: MIT** (`LICENSE`).

**Ten vibeos patches are applied** (rows below). `vendor/fex/CLAUDE.md` is upstream's
policy for contributions TO FEX: these local patches may be agent-written, and none of them
is ever sent upstream (owner decision, 2026-11-02). This section said "Unmodified. No vibeos patches
are applied to this tree" from 2026-07-08 until 2026-10-03, and that was false for
all but the first six days of it: the patch below landed 2026-07-14 and nobody
updated the record. The claim was then reused downstream as evidence — a design doc
argued a fault was upstream behaviour because `git status --porcelain external/fex`
came back empty, which measures *working-tree dirtiness*, not divergence from
upstream, and is clean for a committed patch. **The oracle for "is this tree
modified" is `git log -- 'vendor/fex/*'` or this file, never `git status`.**

| commit | file | what |
|---|---|---|
| `6afc128553f` (2026-07-14) | `Source/Windows/WOW64/Module.cpp` | NULL-thread guard at the top of `BTCpuResetToConsistentStateImpl`. An exception delivered before `BTCpuThreadInit` populates the THREAD_STATE TLS slot cannot be FEX-owned, but every handler below derefs `Thread`, so with `Thread == nullptr` each attempt AVs, re-enters the VEH and recurses to stack overflow (~1600 frames, `arm64-20260714-181535`). Returns the exception to the next handler — but first still runs `OvercommitTracker::HandleAccessViolation`, which is the one thread-independent handler and must stay reachable or on-demand commit breaks for pre-thread faults. Marked in-source with a `VIBEOS PATCH` comment. |
| commit-on-touch (2026-10-11) | `FEXCore/include/FEXCore/Utils/AllocatorHooks.h`, `FEXCore/Source/Utils/Allocator.cpp`, `Source/Windows/Common/OvercommitTracker.h`, `Source/Windows/Common/CallRetStack.h`, `FEXCore/Source/Interface/Core/Core.cpp`, `FEXCore/Source/Interface/Core/JIT/JIT.cpp`, `Source/Windows/WOW64/Module.cpp`, `External/rpmalloc/rpmalloc/rpmalloc.c` | The vibeos kernel charges commit against a limit with no pagefile behind it (`ntoskrnl-rust/src/ARM3/commit.rs`), and FEX commits its code buffers (16 MB, doubling) and pools (4 MB chunks) up front: six guests at a 1 GB boot exhausted the limit and died on the refused allocation. `FEXCore::Allocator::LazyCommitTrack`, when a frontend installs it, turns a committed `VirtualAlloc` into a reservation the frontend's `OvercommitTracker` commits page by page on first touch (`VirtualFree` unmarks it); WOW64/Module.cpp installs it beside its tracker (null, the default, is upstream's eager commit). The tracker now commits only pages still `MEM_RESERVE`, never past that run, with the reservation's own protection (code buffers are reserved RWX), and reports a refused commit as unhandled. The 4 MB per-thread call-ret stack is likewise left reserved: its access-violation handler commits a reserved page of the stack proper and retries (guard pages keep meaning imbalance), and FEXCore's three zeroings of it no longer recommit. FEX's Windows CRT `malloc` is rpmalloc, which committed whole 4 MB medium and 64 MB large pages per allocation: on Windows those pages now commit only the OS pages under each block as it is first handed out (small 64 KB pages unchanged), and a refused commit makes `malloc` return NULL. Not commit-on-touch, because the kernel reads and writes malloc'd buffers in syscalls. `VIBEOS PATCH` at each site. |
| checked commit (2026-11-02) | `External/rpmalloc/rpmalloc/rpmalloc.c` | rpmalloc's `memory_commit` returns `void` and Windows `os_mcommit` only asserts on a refused `VirtualAlloc` (`ENABLE_ASSERTS=0`), so every caller wrote its span or page header into memory that was never committed: at the commit limit the CPU layer died on the access violation (iexplore under FEX at 1024M, `20261017-141001-or20qi28`). `memory_commit_checked` reports the refusal on Windows, and a new span's first page, each later span page, a decommitted free page being reused and a huge allocation now leave their structure unchanged and make `malloc` return NULL. Still unchecked: `heap_allocate_new`, a thread's own heap (a few pages), whose callers store it in TLS and have no failure path. FEX's own `FEX_rp_memory_commit` hook is not involved: `AllocatorHooks.cpp` installs it only off Windows. `VIBEOS PATCH (checked commit)` at each site. |
| uniprocessor TSO (2026-11-07) | `Source/Windows/Common/UniprocessorTSO.h` (new), `Source/Windows/WOW64/Module.cpp` (and vibeos's `tools/fex/wowx64fex/Module.cpp`) | TSO emulation compiles every guest load and store as an ordered access so x86 threads on two cores see x86 store order. With one processor no two threads run at once, so both CPU layers now set `TSOEnabled=0` before creating their context when the PEB reports one processor, unless `FEX_TSOENABLED` is set or the boot's tripwire has `FEX_TSO=1`. Each process prints `TSO emulation off/on`. Measured neutral on our QEMU VMs, as it must be there: TCG's x86 backend emits nothing for acquire/release barriers (`tcg_out_mb` acts only on store-load), so the gain is real hardware's. Ladder green both ways (`20261107-111809-kla30zjb`, `-g0shcxag`). `VIBEOS PATCH (uniprocessor TSO)` at each site. The same header (2026-11-08) also hands FEX's config loader this boot's `FEX_<OPTION>=<value>` tripwire tokens beneath the process's environment (`EnvironmentWithTripwireOptions`, both layers' `LoadConfig` call), so a boot can A/B any FEX option without a config file in the image: `VIBEOS PATCH (tripwire options)`. |
| compile hook (2026-11-08) | `FEXCore/include/FEXCore/HLE/SyscallHandler.h`, `FEXCore/Source/Interface/Core/Core.cpp` | `SyscallHandler::BlockCompiled(Thread, GuestRIP, Ticks)`, default empty, which `CompileBlock` calls for each block it compiled itself with the cycle-counter ticks the call took. vibeos's wowx64fex uses it to split compile time by guest image (`jit image` lines at process exit), which showed 95% of appstorm64's 265 s of compile in system-DLL code every process compiles again. `VIBEOS PATCH (compile hook)` at each site. |
| lazy code cache on Windows (2026-11-21) | `FEXCore/Source/Interface/Core/CodeCache.cpp` | Upstream's Windows path relocated the mapped cache file in place, all at load (`TODO: Implement lazy mapping on Windows`). Every block refers to the per-process dispatcher, so that wrote every page and gave each process a private copy of the whole cache. Now `LoadCache` reserves a separate, inaccessible code buffer, and `FinalizeCodePages` commits it read-write, copies, relocates and publishes it read-write-execute a page range at a time, from the frontend's access-violation handler. Windows has no `mremap`, so the protection change is the atomic step. It uses `::VirtualProtect` because `Allocator::VirtualProtect` passes a null old-protection pointer, which `NtProtectVirtualMemory` refuses, and ignores the failure. `FEX_ENABLELAZYCODECACHINGWIP=0` keeps eager finalization. `VIBEOS PATCH (lazy code cache on Windows)` at each site. |
| machine-wide code cache (2026-11-21) | `Source/Common/Config.cpp` | `GetCacheDirectory` on Windows is `<SystemRoot>\fex-emu\` (from `KUSER_SHARED_DATA.NtSystemRoot`), not `%LOCALAPPDATA%\fex-emu\`. A cache exists so that every process reuses the system-DLL code that others compiled, and services have no `LOCALAPPDATA` (upstream fell back to the current directory). `VIBEOS PATCH (machine-wide code cache)`. |
| guest-width images (2026-11-21) | `Source/Windows/Common/ImageTracker.cpp` | vibeos's x64-guest CPU layer (`tools/fex/wowx64fex`) links this file built for the x86 guest. On a PE32+ image, the 32-bit load-config struct read the wrong field as `VolatileMetadataPointer`. The struct now follows the image's `OptionalHeader.Magic`. `VIBEOS PATCH (guest-width images)`. |
| cache generator (2026-11-21) | `Source/Tools/FEXOfflineCompiler/Main.cpp` | Builds `fexcachegen64.exe`, which runs in the guest (`process-all`) and turns the code maps the CPU layers wrote into one cache per x86-64 binary. Changes: it takes `ImageBase` from the image's own optional header and lets a 64-bit image sit anywhere; it opens images `GENERIC_EXECUTE` and treats NULL as `CreateFileMapping`'s failure; it compiles with the host configuration wowx64fex runs guests with; it commits on touch, as the CPU layers do; it registers its fault handler once per process; it tolerates code maps that a running process holds open; and it logs to its own stdout. **Syscall ABI `OS_GENERIC` (cache config):** under `OS_LINUX64` the compiled code stores the handler's return value into RAX, but the Windows CPU layers return 0 and write the NTSTATUS themselves. So every cached syscall "succeeded", and pass 2 hung in ntdll. `VIBEOS PATCH (cache generator)`, `(x64 cache generator)`, `(cache config)` and `(cache generator logging)` at each site. |
| 64-bit segment selector loads (2026-11-22) | `FEXCore/Source/Interface/Core/OpcodeDispatcher.cpp` | `MOVSegOp` failed decode for a load of FS or GS in 64-bit mode ("We don't support modifying FS selector in 64bit mode!"). A block that fails decode is never compiled, and `ExitFunctionLink` then returns 0 for its address, so the dispatcher branched to host pc 0. ReactOS's amd64 `RtlpRestoreContextInternal` reloads DS/ES/FS from the CONTEXT at the end of every user-mode unwind, so every x64 guest that caught an exception died there (`20261122-133824-aJpIbv`). In 64-bit mode the load now stores the 16-bit selector and leaves the base untouched, which is what FEX's own comment above it says it measured on a Zen+ CPU. Under the x64 Windows guest, GS keeps the TEB base, and nothing uses FS. `VIBEOS PATCH (64-bit segment selector loads)`. |

Everything else is upstream. To verify at any time:
`grep -rn "VIBEOS PATCH" vendor/fex/` should report exactly the sites in the rows above, and
`git log --oneline -- 'vendor/fex/*' 'external/fex/*'` should show only the vendoring
commit, the relocation, the vendor-registry stamp, and those rows' commits.

If a local change ever becomes unavoidable, add a row here AND a `VIBEOS PATCH`
comment at the site, in the same commit.

The one addition is `vendor/fex/VENDORED`, a generated marker written by
`tools/vendored.py --write-stamps` (see `vendor.yaml`). It adds a file rather than
touching one, so every upstream byte is still upstream's; a re-vendor drops it and
`--write-stamps` puts it back.

**Pruned** (build-irrelevant for the vibeos wow64/arm64ec targets — huge test
corpora and gated-off externals; see `tools/fex/build-fex.sh` for the exact
configure that this subset was proven against):

- `External/fex-gvisor-tests-bins` (727M), `External/fex-posixtest-bins` (38M),
  `External/fex-gcc-target-tests-bins` (33M) — test binaries
- `External/vixl` (412M) — replaced by FEX's own CodeEmitter; only needed for
  `ENABLE_VIXL_DISASSEMBLER`/`ENABLE_VIXL_SIMULATOR` (both OFF)
- `External/tracy`, `External/Vulkan-Headers`, `External/zydis`,
  `External/Catch2`, `External/jemalloc_glibc`, `External/drm-headers`,
  `unittests/` — profiler / thunks / disassembler / tests / glibc allocator,
  all gated off in our configure
- `.git*`

**Build:** `tools/fex/build-fex.sh` (out-of-band — deliberately NOT part of the
ReactOS cmake graph; see `docs/engineering/fex-bringup-plan.md` rung 1).
Products land in `<builddir>/Bin/`:

- `libwow64fex.dll` — the 32-bit-x86-guest WoW64 BTCpu plugin
  (imports **ntdll.dll + wow64.dll only**)
- `FEXOfflineCompiler.exe` — standalone JIT driver (imports KERNEL32/ADVAPI32 +
  UCRT api-sets → needs `ucrtbase.dll` + apiset resolution on vibeos)

A second consumer links these archives without touching this tree:
`libwowx64fex.dll`, our x86-64-guest CPU layer (`tools/fex/wowx64fex`, built by
`tools/fex/build-hello-compute.sh`). A re-vendor must re-prove that link too.

To re-vendor a newer tag: shallow-clone upstream with submodules, re-apply the
prune list above, re-run the build proof, update this file.
