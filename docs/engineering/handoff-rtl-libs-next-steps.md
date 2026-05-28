# Handoff: rtl-libs translation track — next steps

**Session anchor:** `rtl-libs` (worktree `vibeos-wt-rtl-priv`, branch
`rtl-priv-port`). Last commit (this session, 2026-05-28 evening):
`7741bafeb1e rtl-strings Wave PRC` + the libsupp postfix to come.

## 2026-05-28 session — x86 boot resolved through Phase 0; blocks on documented upstream PnP bug

Status at end of session, **fresh cold-build** of the worktree:

| arch | result | next |
|---|---|---|
| **arm64** | 6/6 + 5 skip ✅ | clean, ship-ready |
| **i386** | bugcheck 0x6B (PROCESS1_INIT, `\ArcName\multi(0)disk(0)cdrom(96)` NOT_FOUND) | upstream PnP/uniata IDE-binding bug, see `304abfa3cc1` |
| **amd64** | bugcheck 0x6B (same chain, same diagnosis) | same |

The boot **gets through Phase 0 init** (which the pre-allocator-fix
state could not), past system-driver init, into IopReassignSystemRoot,
hits the documented `IopCreateArcNamesCd CdRomCount=0` blocker.
Boot trace `[CDARC] matched signature, CdRomCount=0 EnabledDisks=0`
matches the symptom in `docs/engineering/handoff-smss-init-arc-names.md`
to the byte. The PCI IDE controller PDO (`PCI\VEN_8086&DEV_7010`)
enumerates with `NO RELATIONS` — uniata never binds children, no
CD-ROM PDO exists. Fix lives upstream of rtl-libs.

### What this session unstuck

**`095f0461ee8` postfix** — kernel-side libsupp.rs stubs + ntdll
`RtlGetProcessHeap` real fn. Without these, the kernel link
silently failed and Ninja left a stale `ntoskrnl.exe` in place →
smokes ran the wrong binary. The 3 Wave RES/THR/PRC "green" runs
mid-session were that stale binary.

**`fd05ea9f8c4` allocator fix** (cherry-pick of
`d99ffd568d7bf3 rtl-strings cabi: real allocator on the kernel
feature` from `worktree-smss-csrss`) — the cabi alloc paths were
design-stubbed to unconditionally return `STATUS_NO_MEMORY` (correct
for the diff harness, wrong for the live kernel). With this in,
`RtlAnsiStringToUnicodeString(alloc=TRUE)` from Phase 0 init no
longer bugchecks. Same fix landed earlier on
`worktree-smss-csrss`; rtl-priv-port needed it picked.

**`6873bdef7e6` testing harness** — 69 host unit tests in
`core_rs/{resource,thread,process,env,path}.rs` covering the
pure-decision parts of all the swaps. Found 1 test-input bug
(stack-layout no-guard tight case), 0 translation bugs. Runs
in 10ms with `cargo test -p rtl-strings --lib`.

### The remaining (upstream, documented) blocker

`docs/engineering/handoff-smss-init-arc-names.md` (commit
`304abfa3cc1`) and `2894e56860c` "i386 smoke green: HAL PCI bus
fallback + WDF boot-start + uniata critdev" both apply on this
branch. The `[CDARC] CdRomCount=0` symptom still manifests on a
fresh cold-build despite all three of: (1) the HAL PCI fallback
firing (`HAL: No PCI registry info, probing type 1 bus 0`),
(2) `Wdf01000` Start=0 in the registry, (3) `CriticalDeviceDatabase
PCI#CC_0101 → uniata` mapping present. So either the
`2894e56860c "10/10 GREEN"` claim was the same stale-cache artifact
that `d99ffd568d7`'s commit-message identified for "crp HEAD's
i386 GREEN", or there is a 2nd PnP-binding step that landed on a
different branch and hasn't been merged forward. Verify before
spending more cycles:

```
cd /home/curi/Projects/os-playground/vibeos
git checkout 28e83905bc6  # crp HEAD, no allocator fix
rm -rf build-i386/sdk/lib/rtl/{rust-target,librtl_rust.a} \
       build-i386/ntoskrnl/{libkernel_rust.a,ntoskrnl.exe}
ninja -C build-i386 livecd
timeout 90 bash qemu/run.sh --arch i386 --livecd | tail -30
```

Expected: same `CdRomCount=0` symptom (proves it's pre-existing,
not introduced by my work). If crp HEAD actually boots to
`shell ready` cold, then my work introduced a regression and the
diff is the merge of allocator-fix + RES/THR/PRC.

### Repro (current rtl-priv-port HEAD)

```
cd /home/curi/Projects/os-playground/vibeos
git checkout rtl-priv-port
rm build-i386/ntoskrnl/{libkernel_rust.a,ntoskrnl.exe} \
   build-i386/sdk/lib/rtl/librtl_rust.a 2>/dev/null
ninja -C build-i386 livecd
timeout 90 bash qemu/run.sh --arch i386 --livecd | tail -30
# → ends with: *** BUGCHECK: 0x0000006B (0xc000003a, 0x2, 0, 0)

cargo test -p rtl-strings --lib  # → 69 passed
```

## Where the track stands (2026-05-28)

`sdk/lib/rtl/*.c` cohabitation-swapped to Rust so far (in
chronological order of swap):

| File | Wave | Commit(s) | Branch |
|---|---|---|---|
| unicode.c / nls.c / dos8dot3.c | RTL-1…4 | `5593d886` | continue-rust-port (in HEAD) |
| sid.c | SID-1+2 | `129afc07` / `4bad6d4a` | continue-rust-port |
| acl.c | ACL-1+2 | `f36faa9a` / `2cb60e96` | continue-rust-port |
| sd.c | SD-1+2 + swap | `76bc9d22` / `45e7ca71` | continue-rust-port |
| **priv.c** | **PRIV** | **`7a1b030cd38`** | **rtl-priv-port (NOT merged)** |
| **timezone.c** | **TZ** | **`33a0609a2b0`** | **rtl-priv-port (NOT merged)** |
| **env.c** | **ENV-1+2** | **`64ccdaeea0c`** | **rtl-priv-port (NOT merged)** |
| **security.c** | **SEC** | **`67f09af35e4`** | **rtl-priv-port (NOT merged)** |
| **resource.c** | **RES** | **`acce697403f`** | **rtl-priv-port (NOT merged) — x86 boot broken ⚠️** |
| **thread.c** | **THR** | **`3f99b5f7fe1`** | **rtl-priv-port (NOT merged) — x86 boot broken ⚠️** |
| **process.c** | **PRC** | **`7741bafeb1e`** | **rtl-priv-port (NOT merged) — x86 boot broken ⚠️** |
| (postfix: libsupp.rs kernel stubs + ntdll `RtlGetProcessHeap`) | postfix | (to commit next) | rtl-priv-port |

PATH-1 (path.c leaf fns, 6 fns, ~250 LOC) is **stashed** at
`stash@{0}` (`wave-path-1-wip`) — do NOT pop until x86 boot is
restored; PATH-1 builds clean but its smoke would be masked by the
RES/THR/PRC boot crash above.

rtl-difftest **87/87** (+4 env, +2 sec, +1 SD copy upgrade from
status-only to content);
3-arch smoke: i386 has a **pre-existing** idle-cpu regression unrelated
to any of the rtl-strings waves — see "Open follow-up debt" below.

## Immediate (≤ 30-min) chores

### 1. Merge rtl-priv-port → continue-rust-port

`rtl-priv-port` is **6 commits ahead** of `continue-rust-port`
(PRIV, docs-handoff, TZ, ENV, docs-update, SEC). From
the main worktree (`vibeos`):

```bash
cd /home/curi/Projects/os-playground/vibeos
# Verify nothing else changed under the PRIV files in continue-rust-port:
git fetch --all
git diff continue-rust-port..rtl-priv-port -- sdk/lib/rtl/CMakeLists.txt sdk/lib/rust/rtl-strings
# If the only diff is the PRIV commit, fast-forward-merge:
git checkout continue-rust-port
git merge --ff-only rtl-priv-port
```

### 2. Clean up the main worktree's uncommitted PRIV dups

During Wave PRIV I copied the modified files to the main worktree to
drive the warm-build smoke (see
`[[feedback_worktree_build_reality]]`). After the merge above, the
main worktree's working tree should match HEAD — verify with `git
status -s` in `/home/curi/Projects/os-playground/vibeos`. If
anything diverged in the meantime, **stash, do not `git restore`**
(per `[[feedback_no_blind_revert]]`).

The four files involved:
- `sdk/lib/rtl/CMakeLists.txt`
- `sdk/lib/rust/rtl-strings/src/cabi/mod.rs`
- `sdk/lib/rust/rtl-strings/src/core_rs/mod.rs`
- `sdk/lib/rust/rtl-strings/src/core_rs/priv_mod.rs`

## Open follow-up debt (from prior waves)

### i386 idle-cpu regression on continue-rust-port HEAD (PRE-EXISTING; unrelated to rtl-strings)

**Symptom:** post-`64ccdaeea0c` (Wave ENV) i386 smoke fails
`test_idle_cpu_not_spinning` at 57–90% steady-state host CPU
(threshold <40%). Same msgqueue spam pattern as the amd64 SD-wave:
`(\win32ss\user\ntuser\msgqueue.split2.c:87) err: Sending to
Suspended Thread Msg 121` (WM_ENTERIDLE to a suspended thread,
~2k–5k messages per session) starting right after `[KERNEL]
SwitchDesktop OK`.

**Critical finding — NOT caused by any rtl-strings wave:** verified
2026-05-28 via a `git stash`-based revert experiment in the main
worktree. Stashed the entire Wave ENV diff (8 files), rebuilt i386
livecd with env.c re-enabled and the prior C path active, re-ran
`pytest -k idle_cpu`. Result: **92.7% host CPU steady-state** (vs
57.3% with the wave applied). Stash restored, no data lost.

The misleading green "pre-wave i386" runs that initially suggested
ENV was the cause turned out to be **12 days stale** (May 16, before
several other changes landed on continue-rust-port). Don't trust
`qemu/runs/i386-*` directory listings without checking `stat -c '%y'`
against the commit you're comparing against.

**Diagnosis so far:** the spam triggers in win32k's
`MsqSendMessage` path when explorer (or one of its children) sends
WM_ENTERIDLE to a thread that's transitioned to suspended. Pre-wave
i386 was spam-free; some change on `continue-rust-port` between the
May 16 GREEN run and the May 28 HEAD (`28e83905bc6`) introduced it.
Out of scope for the rtl-strings track.

**Until fixed:** i386 smoke fails the idle-cpu gate, every other
test passes (boot, screenshot, snapshot, sacred primitives, no
unexpected ud2, etc.). The wave is `git commit`-clean and harness-
green except for this one pre-existing gate.

**Investigation plan (out of scope for rtl-strings):**
1. `git bisect` from May 16 GREEN i386 (`qemu/runs/i386-20260516-073104`
   spam=0) to current `28e83905bc6` against the idle-cpu test alone.
   ~10–15 builds at ~3 min each = ~45 min wall time.
2. Likely suspect by date: HAL PCI fallback (`2894e568`), recent WDF
   lazy-init, or one of the smss-csrss / cgu-shift series — all are
   the kind of post-boot user-mode subsystem changes that could shift
   the explorer / suspended-thread interaction.
3. Whatever's found is its own bug — the rtl-strings cohabitation
   pattern stays clean.

### amd64 SD-wave msgqueue.split2.c:87 spam

**Symptom:** post-`45e7ca71` (Wave SD swap), amd64 boot reaches
`[EXPLORER] _tWinMain entered` and paints the desktop cleanly, BUT
spams `(\win32ss\user\ntuser\msgqueue.split2.c:87) err: Sending to
Suspended Thread Msg 121` at ~1900 lines/sec from late winlogon
onward. Serial never goes idle → the 120s desktop-settle phase
times out. `pytest --timeout 300` cuts in the middle of settle;
`--timeout 600` sees the full pass.

**Diagnosis so far:** i386 + arm64 are spam-free; only amd64. The
diff harness is 81/81 byte-identical to genuine sd.c, so the
divergence is in some SD walking pattern that lsass / services
exercise and that `mk_relative_sd` / `mk_absolute_sd` don't cover.
**Diff-coverage gap, not a modelled-logic bug.**

**Reproducer (in main worktree):**
```bash
cd /home/curi/Projects/os-playground/vibeos
# fast: pytest --timeout 300 (will fail at settle)
.venv/bin/python -m pytest tests/qemu/test_smoke.py --arch amd64 \
    --timeout 300 --skip-clean-build -q
# tail the spam:
ls -t qemu/runs/amd64-* | head -1 | xargs -I {} \
    grep -c msgqueue.split2.c:87 {}/serial.log
# the spam starts around line ~3500 after WIN32K shell ready
```

**Investigation plan:**
1. Capture an lsass / services SD trace (gdb breakpoint on
   `RtlValidSecurityDescriptor` / `RtlSelfRelativeToAbsoluteSD2` /
   `RtlMakeSelfRelativeSD` in amd64 build).
2. Compare the SD bytes lsass passes against what `mk_relative_sd`
   generates. Likely candidates:
   - SDs with gaps between SDREL header and payload (`DataStart >
     Sd + sizeof(SDREL)`) — the C code's `MoveDelta` assumption
     breaks but my diff harness always packs payload tight.
   - SDs with non-contiguous payload sections.
   - Edge cases where one field is present but its offset is 0
     (the C `if (offset == 0)` check vs Rust).
3. Extend `mk_relative_sd` in `rtl-difftest/tests/diff.rs` to
   produce that pattern, watch the diff catch it, fix
   `core_rs/sd.rs` / `cabi/mod.rs`, re-smoke.

**Until investigated:** smoke amd64 needs `--timeout 600`. Do not
weaken the test markers (per `[[smoke-tests]]`).

## Next translation candidates (ranked)

| File | LOC | Zw* | RtlHeap | Notes |
|---|---|---|---|---|
| ~~timezone.c~~ | 175 | 0 | 0 | DONE Wave TZ `33a0609a2b0`. |
| **resource.c** | 326 | 0 | 0 | SRW-style reader/writer locks. Heavy `NtCreateSemaphore` / `NtWaitForSingleObject` — translate-only shape (priv.c-like), no byte-diff. |
| ~~env.c~~ | 593 | 0 | 0 | DONE Wave ENV `64ccdaeea0c`. |
| **atom.c** | 749 | 0 | 0 | **Layout puzzle** — `RTL_ATOM_TABLE` has a `FAST_MUTEX` (kernel) vs `RTL_CRITICAL_SECTION` (user) union ⇒ different `NumberOfBuckets`/`Buckets` offsets per consumer. Needs ~3 C accessor helpers per libsupp.c (kernel + user) for layout-opaque tail access. See "Skipped this session" below for the rabbit hole. |
| ~~security.c~~ | 835 | 0 | 9 | DONE Wave SEC `67f09af35e4`. |
| **path.c** | 3040 | 1 | 20 | Big but structural. Would need to split SD-style into 2-3 waves. |

Files that are pure syscall plumbing (priv.c-shape, use
"translate + smoke" without byte-diff):

| File | LOC | Zw* | Notes |
|---|---|---|---|
| process.c | 512 | 30 | Heavy ZwOpenProcess / etc plumbing. |
| thread.c | 352 | 9 | Thread-creation wrappers. |
| registry.c | 1351 | 24 | Wraps NtCreateKey / NtSetValueKey / etc. |

Files DEFERRED (wrong shape for byte-diff, behavioral / property
track):

- **heap.c** + heapdbg.c + heappage.c + heapuser.c — stateful
  allocator (locks / VM / free-lists). Owed to the Kani/property
  track per `[[project_rtl_strings_track]]`.

### Recommended next session (in order)

1. **Merge first.** `git checkout continue-rust-port && git merge --ff-only rtl-priv-port` from main worktree. 6 commits land in one fast-forward. Validate `git status -s` is clean afterwards (priv_mod.rs / env.rs / sec.rs / timezone.rs leave the untracked list; AGENTS.md stays).

2. **resource.c (326 LOC, translate-only).** Smallest unswapped file. priv.c-style: `extern "system" fn` for each `NtCreateSemaphore` / `NtWaitForSingleObject` / `NtReleaseSemaphore` / `NtClose`, mirror `RTL_RESOURCE` struct layout (8 fields, all flat — `RTL_CRITICAL_SECTION` Lock, two HANDLE semaphores, four counters, owner thread). No byte-diff possible (concurrency primitives), smoke is the only net. ~1h wave. Pattern: same as priv.c → no `core_rs/`, single block in `cabi/mod.rs`, comment in `CMakeLists.txt`, ship.

3. **process.c (512 LOC, translate-only) OR thread.c (352 LOC).** Both are syscall wrappers. process.c has 30 Zw\* calls (NtCreateProcess, NtTerminateProcess, RtlCloneUserProcess, etc.). thread.c has 9 — easier first if going translate-only. Same priv.c pattern.

4. **atom.c — if you want the puzzle.** The layout differs per consumer. Cleanest fix: add `RtlpAtomTableNumberOfBuckets(*table) -> u32` + `RtlpAtomTableBuckets(*table) -> **Entry` + `RtlpAtomTableSetSignature/SetNumberOfBuckets` accessor helpers to BOTH `ntoskrnl/rtl/libsupp.c` AND `dll/ntdll/rtl/libsupp.c`. Each consumer uses ITS native struct layout. Rust port treats `RTL_ATOM_TABLE` as opaque; only `RTL_ATOM_TABLE_ENTRY` (no union, flat layout) is replicated in Rust. ~2-3h wave including diff harness with mocked Rtlp\* helpers.

5. **path.c — split into sub-waves.** 3040 LOC. Suggested split: PATH-1 (validation + canonicalization helpers, the byte-diff'able core), PATH-2 (full path resolution + RtlNtPathNameToDosPathName + DOS path arg handling), PATH-3 (the file-system roundtrip wrappers — mostly translate-only). Don't tackle in one shot.

## Conventions reminder (for the next session)

- **Worktree path discipline** — all Edit/Write/Read MUST use
  `/home/curi/Projects/os-playground/vibeos-wt-rtl-priv/...`
  absolute paths. Repo-root paths leak into main.
  (`[[feedback_worktree_path_discipline]]`)
- **Warm build only** — vibeos has no cold-cmake-build path; use
  `build-i386` / `build-amd64` / `build-arm64` from the main
  worktree, copying modified files over for the smoke run, then
  commit in the worktree. (`[[feedback_worktree_build_reality]]`)
- **Background bash mis-reports rc** in this worktree — when
  building, use foreground `dangerouslyDisableSandbox: true` and
  judge by artifacts, not exit codes. Same memory.
- **`core_rs/` is `forbid(unsafe_code)`** — inline asm + raw FFI
  must live in `cabi/mod.rs` only.
- **Never weaken smoke markers**; never revert a `.rs` to C;
  never delete a `.rs` without explicit user approval. See
  AGENTS.md.
- **Stage commits explicitly by path; never `git add -A`.** The
  always-excluded pre-existing mods are: `AGENTS.md`,
  `boot/bootdata/livecd.ini`, `drivers/input/i8042prt/*.c`,
  `ntoskrnl-rust/.../pnpmgr/devaction.rs`, `arm64input-rust/`,
  `win32k-(kernel-)?rust`, `win32k-sys`, `sdk/lib/rust/Cargo.toml`
  workspace edit, `Cargo.lock` for unrelated edges.

## Skipped this session (atom.c rabbit hole)

Looked at atom.c (749 LOC) for the next byte-diff wave; bailed after
mapping the per-flavor layout puzzle. Notes for whoever picks it up:

- `RTL_ATOM_TABLE` (ndk/rtltypes.h:1704) has TWO unions whose
  per-arm sizes differ between kernel and user mode:
  - kernel: `FAST_MUTEX` + `PHANDLE_TABLE` (~24 bytes)
  - user:   `RTL_CRITICAL_SECTION` + `RTL_HANDLE_TABLE` (~76 bytes)
- librtl is compiled with `-D_NTOSKRNL_` (no `NTOS_MODE_USER`), so
  atom.c's view of `Table->NumberOfBuckets` and `Table->Buckets[i]`
  is the KERNEL layout's offset.
- But `dll/ntdll/rtl/libsupp.c` allocates the table with USER-mode
  sizeof, then writes to `Table->CriticalSection` (user offset).
- This is an existing latent inconsistency that ReactOS apparently
  doesn't hit at scale (atom tables are rarely used from ntdll).
- Clean fix without inheriting the inconsistency: 3 C accessor
  helpers per libsupp.c (kernel + user) for layout-opaque access:
  `RtlpAtomTableGetSignature/SetSignature`,
  `RtlpAtomTableGetNumberOfBuckets/SetNumberOfBuckets`,
  `RtlpAtomTableBuckets(*table) -> **Entry` (returns the bucket
  array head, sized by the consumer's native struct).
- Then Rust treats `RTL_ATOM_TABLE` as opaque, but mirrors
  `RTL_ATOM_TABLE_ENTRY` (which has no union — flat layout) directly.
- Diff harness can stub `Rtlp*` helpers with C arrays + no-op locks.

## Reusable infrastructure built this session

- **Heap-backed Nt\*VirtualMemory** in `rtl-difftest/build.rs` (env
  ref TU). `posix_memalign + vm_header_t` page-aligned alloc + free,
  with a per-region size header that `NtQueryVirtualMemory` reads
  back faithfully. Replaces the prior `STATUS_NOT_IMPLEMENTED` stubs
  so `RtlSetEnvironmentVariable` realloc paths run end-to-end.

- **Heap-backed RtlAllocateHeap** in the SD ref TU. Used to be a
  `NULL`-returning stub to pin the `RtlCopySecurityDescriptor`
  `NO_MEMORY` edge; now `posix_memalign`-backed. `RtlCopySecurityDescriptor`
  upgraded from a NO_MEMORY stub to the faithful port, and its
  difftest from status-only to byte-content diff.

- **x86_64 SysV BOOLEAN-return ABI workaround** in the env ref TU,
  reused by sec ref TU. Rust `extern "system" fn -> u8` doesn't
  zero-extend EAX. C callers doing `if (RtlEqualUnicodeString(...))`
  generate `test %eax, %eax` against the full register and read
  garbage upper bits. Workaround: wrap the call in
  `(unsigned int)X & 1u` to force `movzbl` between the cabi call
  and the C test. Real kernel build (MS x64 / i386 stdcall) is
  unaffected. See env build.rs + `refsec_zext_vsid` in sec body.

- **PEB accessor C helpers** in `{ntoskrnl,dll/ntdll}/rtl/libsupp.c`:
  `RtlpGetPebEnvironment` / `RtlpSetPebEnvironment`. Lets Rust shims
  fetch/set `Peb->ProcessParameters->Environment` without mirroring
  the per-arch PEB layout. Same pattern is what atom.c needs at scale.

## Pointer trail

- This branch: `rtl-priv-port` @ `67f09af35e4` (head); 6 commits
  ahead of `continue-rust-port`.
- Related memories: `[[project_rtl_strings_track]]`,
  `[[feedback_smoke_input_flake]]`,
  `[[feedback_worktree_build_reality]]`,
  `[[feedback_libcntpr_shared_source_crt]]` (the libcntpr-vs-librtl
  swap-pattern asymmetry, in case the next file needs the libcntpr
  pattern).
- Difftest harness: `sdk/lib/rust/rtl-difftest/build.rs` (per-file
  ref TU) + `tests/diff.rs` (per-fn fuzz tests). Pattern documented
  in the build.rs comments — copy the sec ref TU block (newest, has
  the heap-backed `RtlAllocateHeap` + Nt-token stubs + BOOLEAN
  zero-extend workaround pre-wired) when adding the next file.
