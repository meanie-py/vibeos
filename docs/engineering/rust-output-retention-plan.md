# Reduce private build output without losing debug information

Owner: bughunt lane. Keep `debug = 1`, frame pointers, translated sources, paid
responses, and ccache/sccache. Never share mutable output across worktrees.

## Storage pilot

`VIBEOS_RUST_STORAGE_PILOT=1 ./vibeos-cli build --arch amd64` opts three main
bodies (beep/null/condrv) into one worktree-local inner Cargo target directory.
Default remains the existing per-module layout. Only the builtin amd64 kernel
target, default features, no extra features/flags, and ordinary non-IR output
qualify. Everything else retains its existing layout.

Each body still has its OWN Cargo invocation and distinct final archive name.
Do not replace this with a multi-package invocation: that changes feature
resolution. Cargo serializes these inner invocations. The outer Cargo target
directory remains separate, avoiding a parent/child target-lock deadlock.
The pilot deliberately trades some parallelism for dependency storage reuse;
measure build time as well as bytes before expanding it.

Initial local measurement, 2026-09-06: the three old amd64 target subdirectories
occupy 274,608,128 allocated bytes total; the pilot directory occupies
101,076,992 bytes (about 63% less). This is a candidate saving, NOT space already
reclaimed: the old directories are retained. Old/new archives have different
hashes; this is not a byte-identity certificate. Boot and symbol/debug validation
are required before promotion. No profile or debug setting changed.

Cold-build discovery now includes nested `*-rustlib/<arch>` directories, not
only the outer workspace target. It preserves other architectures and shared
host build-script outputs. Without this fix, a nominal cold gate could reuse
the very nested artifacts being tested.

## Retirement and fleet retention

`prune --retain-gib N --idle-hours H` plans largest-first reclamation until the
aggregate private-output budget is met, keeping protected trees even when they
alone exceed the budget. Sizes count allocated blocks; activity age includes
descendant mtimes, not only the root directory mtime. Dry-run remains default.
This budget excludes shared compiler caches and run-log/image retention.
Without an explicit idle window, budgeted retention uses 24 hours. A local
dry-run with a 400 GiB budget and 24-hour window found 326.5 GiB retained and
zero eligible outputs: all nine populated trees were recent, active, current,
or the protected integration tree. This snapshot argues for reducing the
active working set, not aggressively evicting caches that will be rebuilt.

Updated CLI artifact consumers hold a shared host `maintenance.lock`; applying
prune requires its exclusive lock. This first implementation requires a quiet
fleet rather than trying to reclaim one idle lane while others build. It keeps
the integration tree protected by default, checks live compiler/QEMU state,
rejects symlinked deletion paths, and rechecks the kill switch before each
directory. It never targets source or paid translations.

Apply is disabled unless `<HOSTLOCK_DIR>/maintenance-owner` contains the exact
absolute path of the one pinned cleanup checkout. A stamp at
`<HOSTLOCK_DIR>/maintenance-disabled` stops subsequent directory deletions.
Both are outside worktrees. **No owner stamp is installed by this change.**

Rollout prerequisite: update every lane that can start artifact work and drain
older CLI/raw jobs before pinning the owner. Older clients and raw build tools
do not obey the new lock; process scanning cannot close their start/delete
race. The lock is cooperative, not a filesystem quota. A process killed while
its child keeps running also needs the live-process checks. Do not enable
scheduled cleanup until this fleet cutover has happened.

## Validation and remaining rollout

`20260906-175238-gate-i386+amd64+arm64` completed in 39m11s with every check
passing: three cold builds, source and binary checks, tooling, differential
tests, cohab coverage, three desktop smokes, and networking (15 tests).
All three screenshots were inspected. Its overall verdict is WARN solely
because the working tree was dirty: **no push stamp**. The initial warm amd64
pilot build was `20260906-174424-build-amd64` (8m02s); beep.sys retained
`.debug_info`, `.debug_line`, and related debug sections. Final targeted
verification: 445 CLI tests, three Rust storage-policy tests, and tooling
static/type checks passed. No debug profile setting was changed.

The full private working set remains approximately 37 GiB; the three-driver
pilot demonstrates a local grouping saving, not a fleet-wide footprint win.
Its final allocated size is 101,081,088 bytes. The cold gate removed and rebuilt
this worktree's generated targets; no fleet prune was applied and no owner
stamp was installed.

- [x] Gate the pilot on all three architectures; inspect screenshots/debug data.
- [ ] Measure warm rebuild time and a same-source pilot-on/off comparison.
- [ ] Restore validated cache-wrapper propagation into nested stable Cargo
      invocations: `reactos-link::build_rust_crate` currently scrubs
      `RUSTC_WRAPPER` on both the builtin-kernel and user-target paths without
      restoring it. Shared cache configuration therefore does not mean every
      nested invocation uses it. Keep the toolchain scrub, test wrapper routing,
      and measure request/hit deltas with isolated counters before claiming coverage.
- [ ] Account for outer Cargo locking when evaluating concurrency: during the
      pilot cold gate, waiting Cargo processes held an fd to
      `/tmp/vibeos/sdk/lib/rust/target/release/.cargo-lock`. Separate per-arch
      build leases do not imply independent Cargo execution. The CMake path
      invokes Cargo per cell; the existing `cargo_build_many` orchestrator is
      not called by `vibeos-cli build`. Do not claim that the three-driver
      storage grouping fixes this outer scheduling bottleneck.
- [ ] Expand only proven compatible groups; preserve feature/codegen variants.
- [ ] Promote a layout before retiring its former directories. Do not retire
      old outputs while the pilot remains opt-in: ordinary builds still use them.
- [ ] Update all lanes, pin the maintenance owner, choose a fleet budget and
      idle window from working-set measurements, then enable retention.
- [ ] Add targeted obsolete-layout retirement with per-arch validation evidence;
      the existing whole-worktree prune is not a substitute for that proof.
- [ ] Bound successful/failed run artifacts separately, preserving failure evidence.

Follow-on implementation: smoke/raw-boot callers give QemuVM an owned
`bootN/serial.log` path. The summary lists every boot's result and log; the
top-level archive selects the first failure (or first success), never a later
boot's overwritten per-arch file. Other QemuVM callers retain their existing
live-log convention for compatibility and still need migration. Regression
tests cover overlapping streams, repeated boots, and launch failures without
borrowing another boot's evidence.

SMP media copies now write sparse all-zero extents while retaining exact bytes
and independent files. This is neither a hardlink nor a shared backing image;
the source can change after copying without affecting the owned copy. Local
ARM64 source measurement: 293,601,280 logical/allocated bytes, with 145,752,064
bytes in all-zero 1 MiB chunks. Actual savings depend on filesystem support and
the image contents. A real SMP-copy check produced 147,390,464 allocated bytes
(140.6 MiB instead of 280 MiB), equal logical length, and exactly one changed
byte: the intended UP-to-MP selector. Windows staging currently copies these files separately
and is not claimed to preserve sparsity; its normal stop path already removes
the per-run staging directory. Cross-run immutable backing images/overlays and
artifact retention remain follow-ups, not enabled cleanup.

Follow-up verification: 458 CLI tests plus lint/type checks passed. ARM64's
`20260906-193550-smoke-arm64` passed two independent desktop boots in 1m22s;
both full logs retain the required markers, and the top archive exactly matches
boot0. amd64's `20260906-193748-smoke-amd64` passed in 37s and i386's
`20260906-194035-smoke-i386` passed in 34s. Screenshots were
inspected. These are warm-build checks, not a clean-tree gate; amd64 networking
was omitted from this targeted logging test and remains mandatory in the train.

The handoff's cache-eviction explanation remains unverified.
Keep failure evidence and active images protected. Verify sccache request/hit
and eviction behavior before changing its cap (`SCCACHE_CACHE_SIZE`); do not
infer a broken shared cache merely from large private Cargo targets.

Until those checks complete, the pilot and retention machinery are migration
infrastructure, not a claim that the 37 GiB working set has been reclaimed.
