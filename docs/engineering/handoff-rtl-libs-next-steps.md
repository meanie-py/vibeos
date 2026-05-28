# Handoff: rtl-libs translation track — next steps

**Session anchor:** `rtl-libs` (worktree `vibeos-wt-rtl-priv`, branch
`rtl-priv-port`). Last commit `64ccdaeea0c rtl-strings Wave ENV`.

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

rtl-difftest **85/85** (+4 env: query / expand / set-seq / set-rand);
3-arch smoke: i386 has a **pre-existing** idle-cpu regression unrelated
to any of the rtl-strings waves — see "Open follow-up debt" below.

## Immediate (≤ 30-min) chores

### 1. Merge rtl-priv-port → continue-rust-port

`rtl-priv-port` is **4 commits ahead** of `continue-rust-port`
(PRIV, docs-handoff, TZ, ENV). From
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

Five remaining `sdk/lib/rtl/*.c` files have the sid/acl/sd shape —
low syscall density, structural data ops — and would fit the proven
byte-diff harness directly. Listed by ease + value:

| File | LOC | Zw* | RtlHeap | Notes |
|---|---|---|---|---|
| ~~timezone.c~~ | 175 | 0 | 0 | DONE Wave TZ `33a0609a2b0`. |
| **resource.c** | 326 | 0 | 0 | SRW-style reader/writer locks. Pointer-bearing but flat structs. |
| ~~env.c~~ | 593 | 0 | 0 | DONE Wave ENV `64ccdaeea0c`. |
| **atom.c** | 749 | 0 | 0 | Atom table data structure. Pointer-bearing tree. |
| **security.c** | 835 | 0 | 9 | Closes the SE family (sid/acl/sd/+SE helpers). Alloc-edge like RtlCopySD. |
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

**Recommended next:** `atom.c` (749 LOC) for the next byte-diff wave —
similar shape to env.c walk algorithms but pointer-bearing-tree heavy.
Alternative: `resource.c` (326 LOC) is smaller but exercises SRW locks
which add concurrency surface to the harness.

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

## Pointer trail

- This branch: `rtl-priv-port` @ `7a1b030cd38`
- Related memories: `[[project_rtl_strings_track]]`,
  `[[feedback_smoke_input_flake]]`,
  `[[feedback_worktree_build_reality]]`,
  `[[feedback_libcntpr_shared_source_crt]]` (the libcntpr-vs-librtl
  swap-pattern asymmetry, in case the next file needs the libcntpr
  pattern).
- Difftest harness: `sdk/lib/rust/rtl-difftest/build.rs` (per-file
  ref TU) + `tests/diff.rs` (per-fn fuzz tests). Pattern documented
  in the build.rs comments — copy the sid/acl/sd ref TU block when
  adding the next file.
