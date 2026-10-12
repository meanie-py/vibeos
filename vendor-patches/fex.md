# Vendored FEX-Emu snapshot (vibeos)

**Upstream:** https://github.com/FEX-Emu/FEX, tag **FEX-2607**
(commit `1cc4b93e7a71c883ec021b71359f136394dc1f3c`, snapshot 2026-07-08). **License: MIT** (`LICENSE`).

**Two vibeos patches are applied** (rows below). This section said "Unmodified. No vibeos patches
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
