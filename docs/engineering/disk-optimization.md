# Disk optimization, September 13, 2026

Next proposed step: [immutable build artifacts across lanes](immutable-build-artifacts.md).
That design covers cross-worktree reuse, dependency identity, reader lifetime, retention,
and phased implementation. It is documentation only; the storage changes below remain
worktree-local.

Direct CMake static-library builds now use four Cargo target directories per
architecture and worktree. 111 declarations were migrated with
`tools/migrate_rust_storage.py`; the outer Cargo link coordinators and their
nested builds retain separate storage. No debug settings changed.

Each static library still has its own `cargo build -p` invocation. Cargo feature
unification follows the packages selected in an invocation, not previous builds
in its output directory ([Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)).
Sharing a directory serializes its Cargo invocations. Four shards preserve some
parallelism; a configure-time ownership check rejects duplicate primary archive
names, preventing competing owners from replacing an unhashed archive before
CMake copies it. No output is linked or shared across worktrees.

`./vibeos-cli build --arch i386 --rust-storage-shards 0` selects the former layout;
`--rust-storage-shards 4` selects the new layout. This setting persists in that
architecture's CMake cache. New configurations default to four. A clean build
removes discovered obsolete targets and rebuilds the selected layout.

## Measurement

The same-source i386 comparison in `vibeos-diskopt`:

| Measurement | Private targets | Four shards |
|---|---:|---:|
| Build directory allocated file bytes | 7.45 GiB | 3.62 GiB |
| Ninja build time | 98.79 s | 97.14 s |
| `.rlib` bytes | 2.45 GiB | 0.52 GiB |
| `.rmeta` bytes | 2.43 GiB | 0.51 GiB |

The build-directory reduction is 3.83 GiB (51%). The first build used a newly
configured directory; the second explicitly used `--clean`. Both used shared
compiler caches. The second run had warm binary-scanner caches, so overall CLI
wall time is not a fair compile-time comparison. A subsequent no-change build
completed in 5 seconds. These timings are observations, not a fleet latency SLA.

Baseline: `/tmp/vibeos-cli/20260913-002736-o02ws6ne-build-i386/summary.json`.
Four shards: `/tmp/vibeos-cli/20260913-003247-fpppz0a9-build-i386/summary.json`.
Warm: `/tmp/vibeos-cli/20260913-003550-xfyp8pw1-build-i386/summary.json`.
Allocated-byte reports are in `disk-optimization-results/`. Reproduce a local
inventory with `python3 tools/rust_storage_report.py --arch i386`; this explicitly
reads artifact bytes and should not run on every build.

The handoff's repeated Cargo filenames did **not** imply byte-identical rlibs in
this fresh baseline: 330 rlibs had 326 distinct SHA-256 hashes. Metadata was highly
duplicated (326 files, 85 distinct hashes). Sharing compiler output still removes
the redundant storage. The report counts existing hardlinked inodes once.

Static archives contributed 1.24 GiB, including 0.62 GiB of identical copies.
Those include CMake's copies of final Cargo archives; they remain independent.
Deleting `.rmeta` after a link would force subsequent compilation to regenerate
metadata. This change reuses it instead.

Workspace Cargo outputs are separate: the initial i386 build also materialized
15.74 GiB under `sdk/lib/rust/target`. This change does not claim to reclaim that
working set. Further nested-build grouping needs to protect same-name feature
variants and measure lock contention separately.

## Retention

`./vibeos-cli retention --enable --retain-gib 100 --min-free-gib 150 --idle-hours 24`
installs an hourly user-systemd timer from this checkout. The 150 GiB headroom
was selected by the user. The 100 GiB fleet-output budget prevents unlimited
retention even when the filesystem has ample free space. These are reclamation
targets, not quotas: active/recent outputs can prevent either from being met.
Shared compiler caches and other workloads are outside the deletion scope.

The timer runs at low CPU/I/O priority. It includes idle integration scratch and
the cleanup checkout's own generated outputs, but deletes only the existing
prune allowlist. It requires exclusive fleet maintenance ownership and at least
one hour of idleness (24 by default). Every run checks that all registered lanes
implement the maintenance interlock. Busy fleets skip that hourly opportunity.
The policy and systemd units live outside worktrees. Keep the pinned checkout
available while scheduled retention is enabled.

`./vibeos-cli retention --disable` stops the timer and installs the global
`maintenance-disabled` kill switch. `--enable` from the existing owner can
re-enable it or change budgets after a fresh quiet-fleet compatibility check.
Plain `retention` prints the policy. `disk-status` reports whether its timer is
active. Manual previews support the same policy:

```
./vibeos-cli prune --retain-gib 100 --min-free-gib 150 --idle-hours 24 --include-integration
```

Activation is currently blocked, not silently bypassed. Source inspection found
missing/incompatible maintenance admission in:

- `vibeos-astra-smp39`
- `vibeos-conformance-integration`
- `vibeos-fex/.claude/worktrees/fex-m4`

These lanes must adopt compatible admission or be retired by their owners, and
older/raw clients must drain before activation. No maintenance owner, policy,
timer, cross-worktree deletion, or cache-cap change has been installed here.

## Validation

`20260913-003645-42vryur_-gate-i386+amd64+arm64` completed all three clean
builds, strict source/binary checks, cohab coverage, differential tests, desktop
smokes and networking. All screenshots were inspected. Conformance ratchets
held on all three architectures; ARM64's ExPools count dropped on the first
attempt and recovered on the gate's serial retry
(`20260913-004354-19pdcqmj-conformance-arm64`).

The gate itself was **TOOLING_FAIL**: it caught an overbroad subprocess mock and
a typed-dictionary error in the new retention tests while those tests were still
being edited. Both were corrected; targeted tests and CLI type checking passed.
The final standalone `check-tooling` rerun passed all 54 checks with stable
inputs (log: `/tmp/diskopt-final-tooling.log`).
That local run used an uncommitted working tree and did not mint a push stamp;
the merge queue performs the landing gate on the integrated commit.

The retired `tools/dedup-build-cache.sh` now fails immediately, including with
`--apply --force`. Its former idleness check could not make the hardlinks safe
for later builds. The supported mechanisms are local Cargo storage reuse and
delete-only retention.

`llvm-readobj --sections` confirmed `.debug_info` and `.debug_line` remain in
i386/amd64 `partmgr.sys` and ARM64 `gdi32.dll`. Generated systemd units also
passed `systemd-analyze --user verify`. An actual activation attempt returned
`RETENTION_REFUSED: fleet activity in progress`; no host policy was installed.


## Nested body storage

`reactos-link` now groups nested body builds into four worktree-local shards per
architecture, target specification, compiler pin, and effective Rust flags.
Each body retains a separate Cargo invocation, preserving feature selection.
Outer link coordinators retain their independent target directories.
Explicit `VIBEOS_EMIT_LLVM_IR=1` diagnostic builds retain per-module directories
so `tools/audit-captures.sh` continues to find the exact module it audits.

Use `./vibeos-cli build --arch amd64 --rust-body-shards N` to persist a shard
count in that architecture's CMake cache: `0` selects private targets; `1`–`16`
select sharing. This is separate from `--rust-storage-shards`, which controls
CMake's direct staticlib builds. The old three-body pilot still applies when
body sharing is disabled and its environment switch is explicitly enabled.

A shard lock covers Cargo, reading the resulting dependency closure, and copying
the archive into the consumer's `OUT_DIR`. The linker uses that independent copy,
never the shared primary filename. Feature variants of the same body therefore
cannot replace an archive while another consumer links it. The directory key
includes target JSON **contents**, not just its basename. Exact-byte identity
markers make hash collisions fail loudly. No cross-worktree outputs or hardlinks
are introduced, and existing cold-target discovery includes the new shards.

`python3 tools/rust_storage_report.py --workspace --no-hash` inventories allocated
file bytes, counting each inode once, including consumer snapshots. Omit
`--no-hash` to additionally measure byte-identical artifacts. Repeated Cargo
filenames are not content hashes and must not be reported as identical bytes.
Compare private/shared runs from the same source, including clean and unchanged
build times; retain the CLI artifact directories with the measurements. Old
private architecture outputs are removed by the normal `--clean` discovery;
other architectures and host build outputs remain protected.

Regression coverage includes real Cargo feature variants linked after both
same-name builds, lock exclusion, independent snapshots, collision rejection,
and Ninja propagation of dependency source, manifest, asset, and generated-input
changes.

Measured amd64 comparison, 2026-10-15 (same OS and engine sources):

| Allocated file bytes | Private bodies | Four body shards |
|---|---:|---:|
| Workspace Cargo targets, including retained host outputs and snapshots | 21.612 GiB | 11.766 GiB |
| CMake build directory | 5.276 GiB | 5.207 GiB |
| Unchanged build, binary scans omitted in both | 9 s | 9 s |

The workspace reduction is 9.846 GiB (45.6%). `pe_equiv --code` found identical
loaded sections, addresses, imports, exports, and resources in all ten sampled
PEs: ntoskrnl/ntkrnlmp, hal/halmp, ntdll, kernel32, win32k, beep, null, and condrv.
Debug sections and build IDs are excluded by that checker. This is an amd64
measurement, not a fleet-wide extrapolation or proof that four shards is optimal.

Ninja took 900.89 s for the initial private build and 619.92 s for the shared
clean rebuild. The initial build also provisioned C outputs; compiler caches and
fleet load were not controlled. These are observed times, not an isolated speedup
claim. Both used shared compiler caches. A repeated layout option initially forced
CMake to rerun and caused a 170.16 s relink; the CLI now skips that reconfigure
when the persisted shard counts already match. The 9 s warm figures use that fix.

Raw inventories, CLI summaries, and the ten-PE comparison are retained in
`disk-optimization-results/20261015-body-storage/`. The cold runs are
`20261015-175513-gkinw5el-build-amd64` (private) and
`20261015-182150-t53sike9-build-amd64` (shared); warm runs are
`20261015-182057-6_sbs_hy-build-amd64` and
`20261015-183303-u2zk1_x9-build-amd64`. Shared-build binary scanning was deferred
to the full gate.

Validation completed in `20261015-191430-26o27ggr-gate-i386+amd64+arm64`:
all three architecture builds, desktop smokes, strict checks, cohab coverage,
136 tooling checks, differential tests, amd64 BIOS boot and 16 network tests
passed. Conformance passed 102/102 on i386 and ARM64 and 105/105 on amd64.
The gate recorded amd64 WoW64 ladder coverage; ARM64's optional FEX ladder
was not staged. Screenshots were inspected. The final verdict is WARN solely
because the uncommitted tree cannot receive a push stamp; nothing was pushed.

The preceding clean builds passed on all three architectures; their summaries
are also retained beside the final gate summary. The first gate parent was
interrupted by a server restart while its clean build children continued. A
subsequent gate was terminated with exit 143 before finalization; its completed
checks are not presented as a complete gate. The final gate reused the clean
outputs and finished normally. These interruptions and admission waits make
full-gate wall time unsuitable as a storage-performance comparison.
