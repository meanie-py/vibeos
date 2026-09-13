# Disk optimization, September 13, 2026

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
