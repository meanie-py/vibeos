# Trustworthy build reuse

Goal: small edits rebuild only affected outputs, without stale-binary false greens.
Keep current clean-build and landing requirements until invalidation is proven.

## First slice: measurement and controls

Implemented: `summary.json.command_timings` records subprocess stage, start offset,
duration, and exit status, including timeout and launch failures. The command journal
has the same fields. Durations include waits inside the subprocess; overlapping
commands must not be summed as wall time. This does not yet measure fleet waits
outside `Run.run`, or separate Cargo lock time from compilation.

Rust cache summaries retain non-cacheable counts/reasons, cache errors, and compilation
counts alongside hit/miss rates. These are server-reported statistics, not proof that
every compiler invocation in the build passed through that server.
Stats reset/collection now use the canonical wrapper, like compilation: previously
they queried the default daemon instead of the worktree-specific daemon. A regression
test checks both calls and their working directories. C/C++ cache counters remain
shared fleet telemetry and can include concurrent work; do not treat them as isolated
per-build counts.

The CLI test suite exercises the shipping CMake source-closure helper with a leaf,
shared dependency, build scripts, and an unrelated crate. This is a source-list control,
not yet an end-to-end incremental-build oracle.

## Next: establish an invalidation oracle

Implemented in `tools/test_build_invalidation.py`, registered in `check-tooling`:
a real offline CMake/Ninja/Cargo fixture compares executed warm and clean outputs.
It tests no-op rebuilds, leaf/shared/unrelated edits, embedded text, build-script
data and script edits, and added/removed shared modules. An invocation journal
asserts that only affected consumers run. Fixture outputs are touched after copying
so their completion timestamps, rather than compiler-output timestamps, drive Ninja.

Two negative controls intentionally omit non-Rust input edges: both produce stale
warm executables that differ from clean executables. This proves the oracle detects
the source-only helper's missing embedded-asset/build-script-data dependencies; it
does NOT certify that helper as a replacement for the broad production edges.

Leaf and shared package-version manifest edits are now covered, with and without
emitted depfiles: only their consumers run, and executed warm/cold outputs agree.
The initial 17 invalidation tests passed; subsequent coverage is recorded below.
Still required before narrowing production dependencies: workspace-inherited
dependency resolution, production generator/replay paths, spec files, and complete
directory-input tracking (native and bindgen controls are described below).
Use Cargo metadata and emitted dependency
information rather than expanding the existing manifest regex. Only then replace
the workspace-wide dependency trigger in `sdk/cmake/gcc.cmake`.

Collect unchanged, leaf-edit, and shared-edit build baselines; include cross-worktree
compiler-cache reuse without sharing generated artifacts. Add explicit fleet-wait and
Cargo-lock attribution plus rebuild reasons. Do not label zero cache requests a miss.

## Follow-through

1. After the invalidation oracle passes, align tool warnings and AGENTS.md around warm
   lane iteration and cold verification in the existing merge queue. Preserve all three
   architecture smokes and exact-tree landing checks.
2. Profile individual source scanners, run proven architecture-independent work once,
   and cache only against complete inputs: source, checker version, configuration, and
   applicable generated headers. Never use sibling generated artifacts.
3. Measure again before considering crate splits. Report queue, compiler, link/package,
   and verification time separately; no speedup claim from cache hit rate alone.

No dependency narrowing, verification removal, or clean-build policy change has been
made in the first slice. Those remain pending work, not implicitly completed steps.

## Initial measurement (2026-09-05)

Warm amd64 run `20260905-010653-build-amd64`, in the bughunt worktree with
the existing kernel fixes and new telemetry: Ninja 4.67s, prime 0.87s, reaffirm
1.33s. Completed checker timings: `scan_ud2` 39.88s, `scan_self_recursive`
27.34s, `verify_match_consts` 109.08s, `scan_extern_arity` 126.94s.
These are individual wall-time samples under fleet load, not a speedup comparison
or the full build duration. Source checks are enabled by default even for ordinary
`build`; distinguish them from compilation when reporting build latency.
This run started before the cache-server reporting fix, so its cache counters
must not be used to judge compiler-cache effectiveness.

The run subsequently completed with WARN in 1122.58s (18m43s), including advisory
source findings. `scan_orphan_modules` alone took 182.46s.

## Scanner fast paths (2026-09-05)

Normalized reachable-file membership is now computed once per crate per orphan
scan, not for every file. This alone measured 178.34s: not a meaningful speedup
over the baseline under uncontrolled fleet load.

The shared module walker now avoids comment/module parsing when raw text cannot
contain the parser's literal module/include tokens. Reachable leaves are still
included in the live set. Generated-include opacity checking similarly avoids
comment parsing only when no `include` substring exists. Comments, strings,
dynamic includes, inline modules, and ambiguous inputs still take the existing
parser path; no new exemptions or persistent result caches were added.

Final whole-workspace orphan scan: 65.44s, versus 182.46s baseline (~2.8x faster).
Captured stdout was compared byte-for-byte with the earlier run and was identical;
both exited 1 with the existing six rootless-file findings. Intermediate module
fast-path-only run: 88.03s, also identical stdout. These are single-run measurements,
not a controlled fleet benchmark or a claim that the entire gate is 2.8x faster.
Dedicated tests protect live-file membership, orphan/stale-banner/phantom findings,
symlink normalization, per-scan membership reuse, and dynamic-include opacity.

## Targeted diagnostics and resolved package graph

`vibeos-cli check --arch amd64 --strict --only TOOL[,TOOL...]` runs selected
source scanners with their normal baseline comparison. It rejects empty/unknown
selections and cannot be combined with baseline rewriting. Its artifact identity
is `check-partial`, coverage is explicitly partial, omitted scanners are recorded,
and even passing selections yield WARN. It does not run the full check's binary,
Cargo-link or committed-drift guards, and does not change the full gate path.

`tools/cargo_depgraph.py --manifest sdk/lib/rust/Cargo.toml --packages
ntoskrnl-cargo kernel-rust` obtains the offline, locked, all-feature/all-target
resolved Cargo graph without writing the lockfile. It includes reachable build,
target-specific and patched local dependencies, rejects missing/ambiguous roots
and incomplete graphs, and reports package manifests. The live kernel example
contains 34 local packages in a graph of 364 packages. No production dependency
edges have been narrowed yet.

Nested build-script Cargo invocations are NOT visible as ordinary Cargo dependencies:
the kernel body must be supplied as a separate root. There is also a SECOND
workspace-wide trigger in `reactos-link::Linker::link` (the loop following
`workspace_root`): it emits `rerun-if-changed` for every workspace crate. Changing
CMake alone will not remove this work. Both layers need complete inputs, including
compiler-emitted `.d` files and explicit non-Rust build-script dependencies.

The host invalidation fixture explicitly orders initial/mutated input mtimes. One
run observed compiled outputs older than initial inputs; the host-clock cause was
not established. Initial inputs are placed in the past and edits newer than prior
fixture artifacts, avoiding sleeps/retries while preserving negative controls for
missing dependency edges. This is a dependency oracle, not a timestamp benchmark.

## Additive production depfiles

`add_cargo_module` now supplements its existing conservative edges with Cargo's
emitted `.d` prerequisites. `tools/retarget_cargo_depfile.py` changes only the rule's
output spelling to CMake's copied PE, preserving the dependency body verbatim.
Unexpected output rules and unsupported output-path spellings fail loudly. CMake
CMP0116 NEW handles modern path transformation; the 3.17 fallback uses Ninja's
build-root-relative output spelling. A mismatched absolute/relative spelling was
caught by the fixture's no-op assertion before production integration.

The fixture proves both embedded-asset and build-script-data changes rebuild the
right executable using emitted dependencies alone, with no manually listed asset
edge. It covers both policy modes and source/build paths containing spaces. This
closes the demonstrated input gap without narrowing any existing production edge.
The full clean gate `20260905-015449-gate-i386+amd64+arm64` passed every build,
strict source/binary check, tooling check, differential test, cohab check, desktop
smoke and network test. Total wall time was 25m11s; desktop boots took 36.71s
(i386), 41.26s (amd64), and 50.87s (arm64). Screenshots were inspected. The overall
WARN reflects the dirty worktree and absence of a push stamp, not a failed stage.
Package-closure narrowing and the reactos-link watcher change remain pending.

The subsequent unchanged amd64 `build --no-check`, run
`20260905-021919-build-amd64`, took 77.69s total but only 1.405s in Ninja, with
zero requests to the correct worktree sccache server. This verifies a no-op for
this snapshot, not all edit scenarios. Binary checks account for most remaining
time. Per-worktree sccache counters can overlap between concurrent architectures.

## Further checker improvements

Comment stripping now scans plain chunks using native regular expressions while
preserving the previous nested-comment, string, escape and unterminated-input
semantics. Tests compare against the previous implementation on 10,000
deterministically generated inputs and large plain chunks. The targeted strict
check `20260905-022227-check-partial-amd64` found zero new findings in
`verify_match_consts`, `scan_extern_arity`, and `scan_orphan_modules`: 189.62s total
versus 256s previously, with the orphan scan taking 22.88s. These are uncontrolled
wall-time samples, not a whole-gate speedup claim.

IRQL scanner controls now build interprocedural summaries from their isolated
fixture workspace instead of repeatedly parsing the entire OS. Cross-file
raise/lower controls and a default-root regression test protect coverage; normal
CLI scanning still summarizes the full OS. All 52 scanner controls passed in
1.40s. No production source checks were removed.

The self-recursion binary checker now fails on disassembler errors, timeouts,
missing tools and empty build inputs instead of reporting clean. Regression tests
cover these failures and successful self-jump detection. These later Python-only
changes passed all 26 tooling checks and a live amd64 binary scan (28 existing
baselined self-jumps, zero new). The subsequent full warm gate
`20260905-023405-gate-i386+amd64+arm64` passed all build, tooling, differential,
strict lint, cohab, desktop and network stages in 20m08s. Screenshots were checked.
Again, the only gate warning was the dirty tree/no push stamp. Warm Ninja times
were 1.52s (amd64), 0.81s (arm64), and 9.67s (i386); strict checks still took
15–16 minutes per architecture. Do not compare this whole-gate duration with the
earlier cold gate as a controlled speedup benchmark.

Implemented next: explicit shared/per-architecture check partitioning:
`source_lint_specs(repo)` currently produces identical commands for each of the
three `check --strict --arch` children (45 scanners, 135 invocations in this
snapshot before partitioning). The gate now runs that complete source suite once after
this worktree's generated inputs are ready, while retaining Cargo-link,
committed-drift, binary and cohab guards for every requested architecture. Both
shared and per-architecture results must pass before stamping. `check --scope
source|arch --strict` exposes these explicitly partial partitions; the arch
partition requires `--arch`, and neither can rewrite baselines or combine with
`--only`. Ordinary unpartitioned `check` still runs its complete previous suite.
Tests cover partition coverage, invalid combinations, failure propagation,
once-only scheduling after all builds, and stamp refusal on either partition's
failure. Full gate `20260905-113218-gate-i386+amd64+arm64` passed every stage;
the sole warning was the dirty tree/no push stamp. The source child recorded
exactly 45 commands in 874s, with per-arch guards taking 80s (amd64), 32s (arm64),
and 86s (i386). All desktop screenshots were checked. Overall time was 26m00s
under integration-build contention: the build matrix finished at 687s. This
proves 90 duplicate source-scanner invocations removed, not an end-to-end latency
speedup against the earlier differently loaded runs.

The continuation also reproduced a CPU-broker fairness bug with real isolated
flocks: total pool capacity was mistaken for availability, allowing a newer
two-permit build to overtake an older waiter while another build held the other
two permits. Admission now follows ticket order; acquiring permits drops the
ticket immediately, preserving concurrent execution. All 30 host-lock/landing-lock
tests passed, including the formerly failing active-holder control. This does
not hot-patch already running processes or sibling worktrees.

An invalidation-fixture run exposed configure metadata dated ahead of Ninja's
generated manifest, causing 100 futile regeneration attempts. Initial generated
CMake metadata now shares the fixture's initial source epoch. Regeneration stays
enabled; no retry or dependency assertion was removed. A test deliberately dates
`CMakeSystem.cmake` one day ahead and verifies normalization and no-op behavior.
The suite now contains 18 invalidation tests. This is fixture clock ordering, not
a change to production build timestamps or proof of the host's clock-step cause.
The final tooling rerun passed all 26 checks with both the timestamp and admission
fixes included. The admission change was exercised by isolated real-flock tests;
already running fleet processes were not changed underneath the gate.

## Expanded invalidation controls

The fixture now covers workspace-inherited package versions, lockfile-only Git
revision changes, and generated Rust built from a watched input directory. Git
dependencies use an isolated local repository and Cargo home; no network or
shared cache mutation is needed. Warm outputs are compared with executed cold
outputs, and invocation journals check affected consumers.

Two demonstrated constraints prevent depfile-only narrowing:

- A Cargo directory watch appears as a directory prerequisite in its emitted
  depfile. Ninja notices additions/removals but not an existing file's content
  edit. The negative control ships stale generated Rust on that edit. Emitting
  both the directory and every input file closes all three cases in the fixture.
  Production directory watches still need an audited input-expansion mechanism;
  this fixture does not change them or certify arbitrary directory trees.
  `reactos-link::Linker::link` currently emits precisely these `src` directory
  watches for workspace crates, so this is relevant to replacing its broad loop,
  not merely a hypothetical fixture limitation.
- Cargo's depfile does not supply the lockfile edge. Changing only the pinned
  Git revision leaves the warm executable stale without an explicit Ninja
  dependency. Adding that edge rebuilds all workspace consumers and matches
  cold output. Production `add_cargo_module` already preserves this edge; do
  not drop it during package-closure narrowing.

Workspace-inherited version edits rebuild all three fixture consumers, with and
without emitted depfiles. This proves inherited package metadata, not inherited
dependency/feature resolution. The generated Rust case uses `OUT_DIR` and
`include!`, not the production bindgen pipeline. No production trigger was
narrowed, no baseline changed, and no boot code changed in this test-only slice.
All 25 invalidation cases passed as part of all 26 `check-tooling` checks. No
OS build or desktop smoke was rerun for these test/documentation-only changes.

### Native compiler inputs

Twelve additional fixture cases compile and archive real C or preprocessed
assembly, then read the linked native symbol from a Rust executable. They
exercise source-only and included-header-only edits with three input policies:
no outer depfile, source-only build-script watches with a depfile, and explicit
source/header watches with a depfile. Only the affected leaf should run; warm
and cold executed outputs are compared.

The negative controls distinguish both missing layers. Without an outer edge,
Ninja never invokes Cargo. Without a Cargo header watch, even deleting the copied
executable to force that invocation still leaves the native value stale. Explicit
header watches plus the emitted depfile close both layers in this fixture.

This uses fixture-owned host compiler calls, not production `reactos-link` or
cc-rs header discovery. Production native include-closure auditing, actual bindgen
generation, spec-file inputs, and workspace-inherited dependency resolution remain
required before narrowing production triggers. No production code is changed.
All 37 invalidation cases passed within all 26 `check-tooling` checks. OS builds
and desktop smokes were not rerun for this test/documentation-only batch.

### Production native discovery and real bindgen controls

`reactos-link::native_deps::compile` now obtains a Make depfile from the same
GCC/Clang invocation that compiles a native source, preserving target, language,
defines, and include ordering. `-MD` includes system headers too; `-MMD` would
omit SDK headers passed through `-isystem`. Every discovered file becomes an
explicit Cargo watch. Missing, empty, or unsupported depfiles hard-fail, and an
old depfile is removed before compilation so it cannot mask missing output.
The production C-shim, individual assembly, spec-stub, and assembly-archive
cc-rs paths use this helper. Archive member order and link ordering are retained.

Two host fixtures call this shipping helper through the actual `reactos-link`
dependency. Editing a transitive system header with a space in its name rebuilds
only the C/assembly consumer and matches a cold executable. Parser unit controls
cover escaped spaces/hash/dollar paths, continued lines, and malformed rules.

Four more controls run real bindgen, include its output into Rust, and change only
a nested header's constant and struct layout. With the generator's own depfile,
warm output matches cold output; without it, the generator and final executable
stay stale. Bindgen emits an absolute output target: CMP0116 NEW makes that match
Ninja's target, while OLD requires retargeting with the shipping depfile helper.
Both policies have positive and negative controls. Using OLD with an unretargeted
absolute depfile instead rebuilds on every no-op (the fixture's no-op assertion
caught this).
The host fixture gate therefore requires `bindgen`/libclang and Cargo's cached
cc-rs dependencies in addition to the existing CMake/Ninja/Rust/native toolchain;
missing prerequisites fail the test rather than skipping this coverage.

These are additive safeguards, not a complete graph replacement. Work left at
that stage included:
compiler-command replay and committed-flags C paths, native header generators,
binding regeneration/postprocessing for the committed per-arch islands, spec
inputs, directory membership/shadowed-header changes, and inherited dependency
resolution. In particular, `generate_ks_inc` treated an existing output as
memoized across build-script runs; input discovery alone could not make that
generator fresh. The next slice addresses that shortcut. No broad production
edge is removed.

The first gate's tooling run exposed another fixture setup clock edge: a copied
executable was 430 ms older than its initial source. Setup edits made after the
constructor were incorrectly treated as post-build mutations. Initial setup now
keeps the past epoch until the first compile, which explicitly reconfigures to
consume initial CMake edits. Only edits after a successful build use the strict
newer-than-artifacts epoch. A deterministic forward-clock setup control preserves
the no-op assertion and proves a later genuine edit still rebuilds. Production
timestamps and dependency edges are untouched by this fixture correction.
The corrected tree passed all 26 `check-tooling` checks, including 44 invalidation
cases and the native depfile parser's Rust unit tests. The original gate's tooling
failure remains in its artifact record; the separate rerun verifies the fix.

Clean gate evidence: `/tmp/vibeos-cli/20260905-225220-gate-i386+amd64+arm64/summary.json`.
All three builds, desktop smokes (screenshots inspected), architecture partitions,
the shared source partition, cohab checks, network tests, and difftests passed.
The gate's overall verdict is `FAIL: TOOLING_FAIL` from the initial fixture-clock
failure above, not an OS failure; the corrected tooling rerun passed separately.
No green gate stamp is claimed for this dirty-tree run. The previously queued
tip `2e016c6595fe` landed independently as `41589edd0c53` through the conductor's
green integration gate; this new production/test batch is a separate local change.

### Compiler replay and invocation-scoped native generation

The compiler-command replay, committed-flags C, geninc compiler, and host-tool
compiler paths now use the same depfile discovery as cc-rs. Command tracing sees
the augmented command. Relative prerequisites are resolved from the compiler's
working directory before Cargo watches them; compile-db entries need not execute
from the build-script directory. Failed compiler commands retain their failure
status rather than being obscured by an absent depfile error.

`generate_ks_inc` now uses `InvocationOutput`: it regenerates on the first request
in each new Linker invocation, then reuses that result in memory. A surviving
file in OUT_DIR no longer bypasses compilation, dependency discovery, or updated
constants. The reuse avoids recompiling once per assembly consumer.

Four C/assembly controls exercise the shipping cc-rs and raw-command paths,
including a different compiler cwd and a transitive system header with spaces.
Two generator controls compile and execute a real C generator whose header sets
the generated Rust constant. The disk-existence negative control stays stale
even when Cargo sees the edit; the invocation cache matches cold output and a
journal proves two requests invoke the generator only once per build script.
These test the shipping cache/discovery mechanism, not the full SDK geninc
layout extraction or a complete cmake-off OS image.

Still owed before dependency narrowing: runtime template/include discovery in
native-hpp and other generators, complete cmake-off generator validation,
committed bindgen-island regeneration/postprocessing, spec inputs, directory
membership/shadowing, and workspace-inherited dependency resolution. Broad
production watches remain in place.

The first tooling run in this slice failed two older controls (bindgen without
header tracking and directory-only removal tracking). Fixture time ordering now
also covers membership changes and generated outputs: additions/removals stamp
the parent directory, existing-file edits do not, and an executed CMake edge
stamps its output after current fixture inputs. Missing edges never execute that
completion step and remain stale. A deterministic day-ahead input control proves
no-op stability after an executed rebuild; all eight focused clock/bindgen/
directory cases passed, including the negative controls. This changes only the
fixture clock model, not production timestamps, graph edges, or assertions.

Verification: the corrected tree passed all 26 tooling checks, including 49
invalidation cases. The clean gate at
`/tmp/vibeos-cli/20260905-234449-gate-i386+amd64+arm64/summary.json` passed all
three builds, desktop smokes (screenshots inspected), architecture/source
checks, cohab, difftests, and all 15 network tests. Its overall `TOOLING_FAIL`
records the first fixture failures above; the subsequent full tooling rerun
passed separately. No clean green gate stamp or master landing is claimed for
this local batch.
