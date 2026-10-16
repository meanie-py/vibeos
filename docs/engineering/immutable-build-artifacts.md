# Immutable build artifacts across lanes

Status: ON BY DEFAULT on all three architectures since 2026-10-16 (user's call: take
the amd64 pilot to every arch and every build, then widen the body set). The measurement
gate and a full three-architecture cold gate passed for the opt-in amd64 pilot first;
the per-arch recipe and the default are below under "Proposed interfaces". The design and
within-worktree sharding baseline were committed together as `eaa4c573a9a` on
`shared-rust-bodies` before this implementation began; that local commit is not a
claim of mainline landing. Step 1 owner: Codex. A handoff must name a replacement
owner and preserve the measurement artifacts.

Discovery is available through `vibeos-cli artifact-store inventory` and `plan`;
`plan --revision` replays tracked historical closures under an explicitly proposed
execution context. These commands report candidates, not cache hits. Measurements
are recorded under `disk-optimization-results/20261016-artifact-spike/`.

## Objective and scope

Let worktrees reuse unchanged build results without recompiling them or retaining
private copies of their intermediate trees. Reuse must follow actual build inputs,
not branch names, claims that a subsystem was untouched, or Cargo filename hashes.
The design must work on the current ext4 filesystem and eventually support remote
collaborators without a filesystem migration.

The first unit of reuse is a **completed nested Rust body build**: its static archive
and an input/output manifest. This is the boundary `reactos-link` already controls.
A second worktree consumes that archive directly and does not invoke nested Cargo
or materialize that body's dependency intermediates on a hit. Its outer Cargo
coordinator, native inputs, final PE, and boot media remain private.

This first step does not independently share every rlib. Several body archives can
contain the same dependency machine code, and a miss may still produce dependency
rlibs in a private worker. Individual rustc actions, direct CMake Rust libraries,
native compile actions, and final link actions are subsequent expansion points.
Do not promise total fleet savings or transparent Cargo replacement from the pilot.

## Working-tree implementation and constraints

- `sdk/lib/rust/reactos-link/src/lib.rs::build_rust_crate_once` constructs the
  nested Cargo command, including per-target ABI flags and compiler selection.
- The baseline `body_storage.rs` change shares mutable targets within one worktree, partitioned
  by configuration and body shard. Its lock covers Cargo, depfile reads, and copying
  the archive into the consumer's `OUT_DIR`. It is not a cross-worktree cache.
- `cargo_deps.rs` reads Cargo's `.d` and emits `cargo:rerun-if-changed` inputs.
  `sdk/cmake/gcc.cmake` and `tools/retarget_cargo_depfile.py` propagate the outer
  Cargo depfile to Ninja. Workspace manifests/configuration are separately watched.
- `tools/canon-run.sh` gives builds a canonical `/tmp/vibeos` source path.
  Per-worktree sccache servers resolve that path in their own mount namespaces;
  the compiler-cache storage is shared. Preserve this separation for new workers.
- `tests/qemu/cli/provenance.py`, the binary scanners, clean discovery, and retention
  currently reason about private outputs. Their integration is part of this project,
  not optional follow-up work after cache activation.
- Hardlink deduplication of writable outputs was retired after live-input truncation
  and stale-build failures. Never hardlink store objects into writable Cargo targets.
  Store objects must not become compiler output paths or mutable symlink targets.
- The previous within-lane measurement is documented in
  [disk-optimization.md](disk-optimization.md). It is not a measurement of this design.

## Architectural boundary

Keep Cargo as the body compiler in the first implementation. Place a lookup/publish
layer around the existing nested invocation, not around the entire outer `build.rs`:
outer build scripts also prepare resources, native code, and link directives whose
inputs are not part of a body-only result.

```text
outer module build
  -> resolve body request and eligible input closure
  -> hash immutable input snapshot -> action key
  -> acquire result lease and look up action
       hit: verify manifest/blobs, emit local input watches, return archive handle
       miss: compile snapshot in private worker -> validate -> publish -> return handle
  -> linker reads leased immutable archive
  -> private PE and boot media follow existing build/provenance checks
```

Separate three identities:

| Identity | Meaning |
|---|---|
| Recipe key | Schema, body request, compiler/platform/configuration and closure-discovery policy; useful for indexing, never sufficient for a hit |
| Action key | SHA-256 of a canonical action descriptor containing the recipe and complete input digests |
| Blob digest | SHA-256 of the actual output bytes; independent of the action that produced them |

An action index maps an action key to a result manifest; manifests reference blobs.
Different actions may reference the same blob. A recipe index or a previous manifest
can suggest work to validate, but cannot establish freshness by itself.

## Input identity and eligibility

Use a versioned deterministic encoding with explicit field boundaries, sorted map
keys and path lists, preserved command argument order, and distinct absent/empty
environment values. Do not use debug formatting, delimiter concatenation, mtimes,
inode numbers, a Git commit alone, or the current storage helper's FNV digest as a
semantic cache key. Retain the full descriptor so `explain` can report differences.

The descriptor must contain:

| Input class | Required identity |
|---|---|
| Request | Package identity, selected target, crate type, profile, sorted requested features, default-feature switch, kernel/user mode, build-std settings |
| Rust configuration | Target triple or JSON **bytes**, effective flags with precedence resolved, applicable Cargo configuration, resolved closure and relevant lock entries, effective profiles/overrides, inherited workspace settings, patches/replacements and resolver semantics |
| Toolchain | Compiler and Cargo identities, host platform, sysroot/target libraries and tools actually used; pin-file text alone is insufficient |
| Sources | Full declared source roots of the selected package and conservative reachable dependency graph, including untracked inputs, additions, deletions, relative paths and file contents |
| Dependencies | Registry/Git/path source identities and content verification; all relevant normal/build/target-specific dependencies for the resolved request |
| Environment | Explicit effective environment passed to the worker, including code-affecting `VIBEOS_*`, `cfg`, and generator settings; exclude scheduling/logging values only through reviewed policy |
| Generated/native inputs | Declared producer action/output digests, external data and native libraries if supported by that action class |
| Execution policy | Store protocol version, discovery-policy version, command construction and path-normalization versions, platform/path mapping |

Resolve configuration at the actual command working directory. Kernel and user-mode
nested calls currently use different working directories and flag rules. A future
normalization must not accidentally enable SSE or change compiler selection.

### Resolved closure and manifest projection

Do not derive eligibility from an old `.d` file alone. It can omit newly introduced
files, unexecuted conditional branches, environment observations, manifest changes,
or undeclared build-script reads. It also only exists after a build.

For the pilot, construct the closure before lookup from pinned Cargo resolution and
complete package source roots. **Do not put whole workspace manifests or the whole
lockfile into every action key.** Their raw bytes remain discovery prerequisites and
provenance, but the action identity uses a versioned semantic projection:

- Resolved package/source identities, checksums, dependency edges and feature selection
  for this invocation, distinguishing host/build and target dependencies.
- The closure's lock entries and applicable configuration, including inherited workspace
  dependency/package settings, resolver behavior, selected profile and per-package/host
  overrides. Preserve settings that affect `env!`, `cfg`, codegen and output naming.
- Workspace membership, patches and replacements through their effect on that resolution
  and configuration. Adding an unrelated member or unrelated lock entry must preserve
  the key if those effects and the closure's source contents are unchanged.

Re-resolve after *any* workspace manifest, lock or searched configuration change. The
outer dependency watchers must still watch those whole files: an action hit can follow
rediscovery even when the action key is unchanged. A cached projection keyed only by
the previous closure would miss a newly influential patch or resolution change.

Hash source contents only for the selected dependency closure. In package source-root
inventories, represent manifests by the reviewed projection rather than accidentally
reintroducing their raw bytes into the action key. Keep full raw manifest/config bytes
in snapshot/provenance records to explain resolution. If a source observes a manifest
as data (for example `include_str!`), those bytes are a real compiler input and must
be hashed verbatim. Such actions are outside the initial normalized-manifest pilot
unless the observation is explicitly modeled. The same rule applies to build scripts
that inspect workspace membership when that action class is eventually supported.

Metadata resolution must be locked/offline after provisioning. Cargo metadata alone
is not assumed to reproduce the exact feature selection of a single-package build;
prove the projection against the actual pinned Cargo invocation using controlled
compiler-command observations and feature-variant fixtures. A conservative closure
superset is acceptable only if its scope is recorded and the unrelated-member test
still passes. Unsupported config/manifest constructs make a recipe ineligible until
modeled; do not silently drop them or quietly revert to whole-workspace hashing.
Keep a whole-manifest key only as a measurement comparator, never the production hit
policy. Normalize worktree-specific package paths to logical source roots without
merging distinct registry/Git identities.

Start with audited bodies whose transitive graph has no build scripts, proc macros,
external native compilation, outside-root source includes, or undeclared generated
inputs. Builtin stable targets come first, including eligible i386 kernel bodies;
custom-target build-std and compiler-builtins generation require a separate recipe
for the particular actions that use them, not a blanket exclusion of i386 kernels.

Plain native DLL import attributes are different from bundled static libraries.
Pinned 1.92 compiler negative controls on Linux and amd64 Windows proved that an
absent default/dylib library permits a static archive, leaves its native symbol
undefined, and becomes a final-consumer requirement; the equivalent `kind="static"`
fails without the library. Discovery accepts narrowly parsed literal default/dylib
attributes and the existing raw-dylib `+verbatim` form. It rejects other modifiers,
unknown keys, bundled static libraries and conditional `cfg_attr(link(...))` until
modeled. These controls live in `tools/test_artifact_native_inputs.py`.

A source inspection of the working tree on 2026-10-15 found no `build.rs` or explicit
build/proc-macro setting in ntkrnl-sys, ntdll-sys, panic-common, panic-lean, cnt-rust-lib,
win32macros-rust and strsafe-rust. This is evidence about those packages, not proof
of any body's complete graph. Audit registry and optional dependencies per request.
Choose at least one eligible body with a large dependency closure as well as a small
fixture such as beep/null; a tiny-body demonstration alone cannot size the payoff.

Within each package root, hash all files except an explicit, validated output/VCS
exclusion policy. Include added files and directory membership; ignored/untracked
source is not inherently irrelevant. Reject source symlinks escaping declared roots
in the pilot. Output directories cannot also be declared input roots. Sources using
inputs outside these roots are ineligible until those inputs are declared and isolated.

Unsupported graphs execute through the existing private/sharded path with an explicit
`ineligible` reason. This is a supported dispatch decision, not a claimed cache hit.
Strict pilot mode fails on ineligibility so a test cannot quietly exercise the old path.
Expanding eligibility requires input-discovery tests and execution isolation, not just
trusting build-script `rerun-if-changed` declarations.

### Stable inputs during execution

Hash and compile the same bytes. A before/after hash of a live checkout is inadequate:
a file can change and change back while the compiler reads it.

Materialize an immutable source snapshot of the declared closure; calculate the key
from that snapshot. Run misses in a worker namespace with those sources at the
canonical path, private output mounts, provisioned read-only dependency/toolchain
roots, an explicit environment, and network access disabled. Do not let the worker
fall through to another lane's sources or arbitrary host config. The initial snapshot
implementation may copy source files; it must not hardlink mutable checkout files.
Record and budget snapshot bytes as well as output bytes.

The snapshot and worker isolation requirements are correctness invariants; their
implementation need not be a new general-purpose build service. Step 1 must compare
reuse of the existing canonical runner with a dedicated worker, and itemize what the
first eligible bodies actually read. Demonstrate which mount, environment, source
capture and compiler-server controls are needed, and measure their setup cost before
building a worker framework. A simpler executor is acceptable if it proves the same
stable-input and namespace guarantees. A source-only body can still race edits or
read the wrong sccache namespace, so absence of `build.rs` alone is not that proof.

Input discovery must finish before the worker starts. Reject unsupported external
reads by restricting visible input roots; determine the sandbox's necessary system
files/toolchain roots in the pilot and include their identity where output-relevant.
Audited recipes must exclude clock/random-dependent output. Source changes during
snapshot construction require retry or explicit failure; source inventory and content
capture must describe one coherent build input set. Integrate with checkout/build
mutation interlocks or obtain the snapshot from an immutable captured source tree.
The lane must revalidate its current inputs against that snapshot before accepting
the result as its current build. If they changed, restart; never stamp stale media.

## Store layout, protocol, and immutability

Proposed local layout, outside all worktrees and existing compiler caches:

```text
<configured-store>/v1/
  blobs/sha256/<prefix>/<digest>       # immutable output bytes
  actions/<action-key>.json           # immutable complete result manifest
  locks/<action-key>.lock             # stable lock inode, never GC'd while usable
  staging/<unique-worker-id>/         # incomplete private publication data
  leases/<lease-id>.json              # active consumers/publications
  refs/<worktree-id>/<arch>/...        # durable warm-consumer references
  quarantine/...                     # corruption/conflict evidence
```

Configure one explicit absolute store root through `vibeos-cli`; do not infer it from
each worktree's parent. Bind it at a common canonical path in worker/link namespaces
so different user home paths do not leak into archive/link identities. Host-side
receipts refer to digest identities, with paths resolved by store configuration.
Initial deployment is a trusted single-host user store on a local filesystem with
tested lock/rename semantics; network filesystems are not an implicit supported mode.

Required manifest fields: schema version, action key, full canonical descriptor or
its retained object, input inventory, output roles/names/digests/sizes, dependency
action references when applicable, compiler/platform identity, and publication
provenance. Timestamps and diagnostic logs are metadata, not action identity.
Only declared regular-file outputs are publishable. Reject absolute output names,
path traversal, symlinks and special files. Body v1 publishes an archive plus the
relocatable dependency manifest; optional debug companions must be explicit outputs.

Publication sequence:

1. Acquire the per-action build lock; recheck the action index after waiting.
2. Build on a miss using admitted private scratch. Do not hold a global store lock
   during compilation or wait for a compiler permit while holding a shard lock.
3. Validate expected artifacts exist, collect/check the dependency inventory, hash
   outputs, and compare observed dependencies with the declared snapshot. Undeclared
   inputs invalidate the action; do not publish a partial result.
4. Copy outputs into store staging on the store filesystem, verify bytes, flush files,
   and atomically publish blobs with no replacement. Verify any existing digest path.
5. Publish the manifest last, flush its containing directory, and create the consumer
   reference/lease under the GC coordination protocol before releasing protection.
6. Return an artifact handle. Clean scratch only after successful publication and
   receipt creation; retain bounded failure diagnostics through the normal policy.

Never modify an existing blob, even to strip debug information. Readers never see
staging. Read-only permissions prevent accidental writes but do not constitute a
security boundary against the same OS user; all compiler output paths must be private.
No hardlinks from writable trees to store objects. A future separately owned service
can provide stronger enforcement without changing digest identities.

If an independent rebuild of the same action produces different bytes, retain both
diagnostic results, mark the action conflicting, and fail verification. Do not overwrite
the index or equate PE semantic equivalence with reproducible archive bytes. Investigate
path leakage, timestamps, nondeterminism, or missing inputs before broadening rollout.

## Reader lifetime and build-system integration

Direct store reads are a deliberate storage/complexity trade-off. Copying an archive
into each consumer's `OUT_DIR` would simplify lifetime management: after a verified
copy completes, that consumer no longer needs to pin the store blob for its linker.
However, it retains one archive per consumer/variant and reduces the expected saving.
The post-sharding inventory must measure this cost separately, including existing
snapshot copies; the review's archive count/size is a hypothesis until remeasured
with scope, inode accounting and content identity recorded.

Keep copy-on-hit as an explicit experimental comparison or rollback mode, never an
automatic response to a lease failure. It still needs a lease during copying, normal
dependency validation and provenance. If direct-reader lifetime tests fail, disable
direct mode; compare the simpler mode's measured net savings before choosing it.
A copying pilot does not satisfy the direct-sharing milestone by relabeling a hit.

`build_rust_crate_once` should return an artifact handle rather than a bare `PathBuf`.
The handle contains archive identity/path, normalized local input watches, result
manifest identity and its reference. Legacy mode can wrap its private archive in the
same interface. Keep command construction in one place to avoid cache-mode ABI drift.

The build script ends **before** outer Cargo invokes the linker. An RAII lease dropped
at the end of `build.rs` is therefore unsafe. The CLI build supervisor must own a lease
through all Cargo/link/media work, and each successful outer build must retain a durable
reference while its build-script output can be reused. Direct engine invocations must
create an equivalent durable reference before returning a path, or explicitly refuse
store mode without a supervising context. Do not rely on a short TTL for link safety.

Cargo can reuse build-script output without executing it. Before running Cargo, restore
and lease all referenced manifests/blobs from the lane's receipts. If a reference is
missing or invalid, force the corresponding action/build-script rebuild through a
tested mechanism; never leave a fresh Cargo fingerprint pointing at a missing file.
Deleting the whole output directory is not the default repair strategy.

On a hit, emit the same source/config/environment dependency watches as on a miss,
mapped to **this worktree**. Include source-root membership watches so added files
invalidate the caller. The durable manifest replaces dependence on a worker-local `.d`;
never replay paths into another worktree or deleted scratch. Store all watched inputs,
including relevant `rerun-if-env-changed` keys. Changing cache mode/root/policy must
invalidate the outer script as well. If Cargo/Ninja directory watches cannot reliably
represent membership, add a CLI-owned input-inventory prerequisite before enabling hits.

Keep the existing outer Cargo -> retargeted depfile -> Ninja chain. Test changes with
ordinary `vibeos-cli build`, not only an explicit forced engine invocation: otherwise
the cache could be correct while Ninja never calls it after an input edit.

Before activation, adapt:

- `provenance.py`: include artifact manifests/digests in build receipts, validate input
  identity and current media, and map static archives to their consumers without
  inferring ownership from private directory names. Do not substitute a warm receipt
  or store hit for the existing freshness and gate checks.
- `workspace_cargo_targets`, clean/prune: keep the shared store outside deletion walks;
  release lane refs only under existing maintenance/build interlocks.
- Binary scanners and symbol/debug tools: scan the exact consumed archive or final PE,
  not whichever same-named archive a directory walk finds. Preserve debug information.
- `VIBEOS_EMIT_LLVM_IR`: keep the existing private diagnostic path initially; do not
  claim IR coverage from an archive-only cache hit.
- `cachecfg.py`/canonical runner: a miss worker needs its own namespace-correct sccache
  server identity. Reusing the parent's server would resolve its sources in the parent
  namespace. Preserve shared compiler-cache storage without sharing the daemon context.

## Retention and failure behavior

GC takes a short global coordination lock shared with lookup-and-pin, publication and
reference changes. It marks manifests reachable from active leases and durable lane
refs, then their blobs. Remove only unreferenced objects, with an age grace period;
never truncate a blob. An active reader pins before releasing the coordination lock.
The implementation must define one lock order and stress-test it; GC must not wait
for per-action compilation locks while holding the global lock.

Leases identify owner process **and start identity**, not PID alone. A supervisor
heartbeat is useful but absence alone must not reclaim a live build's inputs. Durable
refs survive daemon restarts. Stale staging/ref reclamation is bounded, visible, and
uses the repository's maintenance protocol. Disk admission covers source snapshots,
worker targets and publication copies; store size counts unique blobs once. A byte
budget does not authorize evicting a live or pinned result.

| Condition | Required behavior |
|---|---|
| Action absent | Ordinary miss; compile and publish if eligible |
| Graph unsupported | Explicit ineligible reason and existing build path; strict mode fails |
| Missing blob for an indexed action | Broken entry; invalidate reference, report it and rebuild through the normal mechanism |
| Digest/schema inconsistency or conflicting result | Fail loud, quarantine evidence; no silent private fallback |
| Build/compiler failure | Preserve failure, publish nothing |
| Disk full or worker killed | No visible partial action; retry can reclaim verified stale staging |
| Worker/source inputs change | Reject current result for this lane and re-evaluate inputs |
| Optional remote unavailable | Explicit local miss policy; remote-required mode fails |

## Proposed interfaces and telemetry

The first implementation exposes these commands through the existing CLI:

- `build --arch <a>` uses the shared store at `$VIBEOS_ARTIFACT_STORE_ROOT`, default
  `~/vibeos-artifact-store`, for the six audited body packages (CRT startup, CRT entry
  bridge, DLL entry shim, console utilities, beep and kernel). `--artifact-store PATH`
  names another store; `--no-artifact-store`, or an incremental build, keeps every body
  on the existing storage. The store must be outside every checkout.
- **Per-arch recipe.** A body is eligible on a builtin triple only (x86_64-pc-windows-gnu,
  i686-pc-windows-gnu, aarch64-pc-windows-gnullvm), with exactly the RUSTFLAGS
  reactos-link gives it there (`FLAGS` in `tools/artifact_executor.py`: frame pointers
  for kernel-mode bodies, `-sse,-sse2` for every i386 body). Target and flags are part
  of the action context, so the worker compiles exactly that. The amd64 recipe is
  byte-identical to the pilot's, so its actions stay hits. reactos-link keeps a copy of
  the flags table (`artifact_store.rs`, held equal by `test_artifact_config.py`): a
  body on a custom target JSON (the i386 kernel, `i686-kernel`) or with other flags
  takes the private path instead of failing at the store. So does a build with no
  vibeos-cli supervisor (a raw `canon-run.sh ninja`), with a cargo warning.
- **Requested-target config.** The request audit holds the requested triple's
  `[target.<triple>]` section to `linker`, plus `rustflags` only when the request
  sets RUSTFLAGS itself, which replaces them; the worker reads no Cargo config.
- Add `--artifact-store-verify` to independently rebuild and compare selected actions
  once per supervisor session. A command/environment nonce forces requests even when
  Ninja and Cargo would otherwise be fresh. An empty store requires two independent
  builds before a verification receipt can claim a comparison.
- `gate --artifact-store /absolute/store [--artifact-store-verify]` forwards the
  selection to every arch's build (the builds use the default store anyway); the final
  warm amd64 media restage retains the store without repeating verification.
- `artifact-store build-body --store /absolute/store --body NAME --output REPORT_DIR`
  exercises one body under disk/CPU admission. `--features`, `--no-default-features`
  and `--kernel` specify the request; `--clean` and `--verify` exercise rebuild modes.
- `artifact-store status --store PATH` reports filesystem accounting, not content
  integrity. `artifact-store gc --store PATH` is a dry run; `--apply` collects only
  unreferenced objects after the grace interval. Live sessions and durable references
  protect objects independently of their age.
- `artifact-store repair --store PATH --action SHA256 --output REPORT_DIR` explicitly
  rebuilds a missing indexed blob. The current source, compiler and recipe must match
  the original action exactly; output bytes must match its recorded digest. Existing
  corrupt blobs are rejected, and references/index entries are preserved. Normal
  builds fail on missing blobs until repair succeeds; repair never serves consumers.

Store path and supervisor-verified toolchain identity are persistent Cargo command
inputs. Session capabilities are private, short-lived inputs, not dependency watches.
Full toolchain hashes are checked before and after the supervised build; misses also
retain a worker postcheck. This avoids expanding every consumer's dependency file
with the entire compiler installation.

The supervisor audits ambient Cargo configuration before and after the outer build,
including when all consumers are fresh. Its bounded cwd scope covers every arch's
build directory, Rust workspace and direct package directories, their ancestors,
the canonical mount's ancestors and effective Cargo home. Each request supplies
its actual nested Cargo cwd and must fall within that audited scope. Only inert
target settings are allowed; profile, build, environment and target-flag overrides
outside the explicit recipe fail. Missing-config membership is checked by this
supervisor scan, not recursive Cargo watches on `/` or the home directory.

The local prototype fingerprints native tool contents but still includes their
resolved installation paths in the tool descriptor. Worktrees using the same local
installation can share; identical installations at different paths can miss. Before
remote collaborator reuse, separate physical mount locations from logical tool
identity and prove that normalization with independently installed toolchains.
Remote transport and authentication are not implemented by this local store.

Reference retirement currently accompanies a store-enabled clean build:
only after private Cargo outputs are successfully removed does the supervisor drop
their durable references. Standalone body references remain. Removing worktrees or
pruning outputs outside this path conservatively leaves references behind; automatic
retirement for those lifecycle operations remains required before broad rollout.
Never reclaim those references solely because a lane is old.

`--clean` must not silently become an action-cache hit. Preserve its independent-rebuild
intent by bypassing action hits on clean gates; compiler caches follow existing policy.
Test action hits separately after clean builds. Verification receipts must state whether
compiler caches were used; periodically test reproduction with them disabled too.

Write machine-readable per-action events into CLI artifacts: eligibility/reason, key,
hit/miss, lookup/hash/wait/compile/publish time, bytes read/written, referenced/new blob
bytes, scratch peak, and failure stage. Counters from other sccache daemons are not
per-action measurements. Report physical fleet bytes and each lane's retained bytes
separately; shared logical references are not additional physical savings.

## Implementation sequence and acceptance

1. **Discovery, payoff and execution spike (Codex).** Audit small and large eligible
   body graphs; implement the semantic descriptor/inventory with explanatory diffs and
   unrelated-member tests. Validate the minimum worker isolation requirements and their
   cost. No hits enabled. Produce the measurement report and pass the decision gate below
   before building store primitives. If no useful body is eligible, identify the needed
   generator recipe and cost rather than assuming a store will pay for itself.
2. **Store primitives, conditional on the step 1 evidence.** Implement SHA-256 manifests/blobs, atomic publication,
   collision/corruption handling, action locks, supervisor leases and durable refs.
   Choose a maintained stable-compatible hashing/serialization implementation through
   the normal dependency policy; do not hand-write cryptography. Add crash/concurrency tests.
3. **One body across two worktrees.** Integrate the artifact handle and full invalidation
   path. Worktree A builds/publishes; B consumes the identical blob without nested Cargo,
   without an `OUT_DIR` archive copy and without private body intermediate targets.
   Link and boot using the ordinary front door. Keep all other bodies on existing storage.
4. **Retention and correctness gates.** Finish scanner/provenance/clean/GC integration,
   run the matrix below, independently rebuild and compare, then gate all three arches.
   Record unsupported configurations explicitly; pilot eligibility may remain amd64-only.
5. **Measured expansion.** Extend to more audited bodies, then direct CMake staticlibs.
   Benchmark scratch reuse under the broker versus disposable miss workers. Reuse mutable
   scratch only as a worker-local optimization with snapshot/config validation; readers
   must never consume scratch paths. Tune budgets/shards from measurements.
6. **Individual actions and remote backend.** Specify a separate action-graph integration
   for rustc/rlib outputs, generators, native compiles and links. Cargo's mutable target
   bookkeeping cannot be replaced by symlinking its `deps/` to this store. Do not depend
   on nightly-only build-plan interfaces or undocumented fingerprint mutation. Evaluate
   the existing orchestrator boundary and maintained build-system approaches before
   committing to owning a complete Cargo scheduler.

The pilot is successful only if two differently located worktrees reuse the same
immutable artifact, the consumer retains no body intermediates on a hit, and an edit
to a shared dependency invalidates every affected consumer while unrelated package
source edits, member additions and lock entries retain the hit when resolution is
unchanged. Record identity, command count and allocated-byte evidence.

### Measurement gate before store primitives

Produce a machine-readable report plus a short interpretation under
`disk-optimization-results/`, naming the source revisions/dirty-input digests, worktrees,
architectures, effective shard settings, timestamp, commands and accounting method.
Do not prune other lanes or modify their builds to obtain these measurements.

1. **Post-sharding inventory.** Measure current sharded outputs by role: body dependency
   rlibs/rmeta, body primary archives, consumer snapshots, outer link-coordinator host
   dependencies, direct CMake builds, and other outputs. Count allocated blocks and
   unique inodes separately from logical file sizes. Report old-layout leftovers and
   mixed configurations separately rather than attributing them to the new layout.
2. **Content identity.** Use size/name grouping only to narrow candidates, then SHA-256
   actual bytes before claiming duplicates. Equal Cargo hashed filenames are not content
   hashes and have already differed in sampled copies. Conversely, different names can
   contain equal bytes. Do not reuse the earlier filename-grouped 52.7 GiB estimate as a
   verified duplicate total. For live outputs, use quiescent snapshots or detect/reject
   changes during reading; record incomplete coverage rather than treating it as zero.
3. **Reachable savings.** Attribute bytes to the actions that would avoid producing them,
   not just to same-content files. Shared shard intermediates must not be counted as
   reclaimable while other miss/ineligible actions still need them. Count consumer copies
   eliminated by direct reads, unique retained store archives, source snapshots, retained
   worker scratch and GC references. Provide a conservative net steady-state estimate
   and peak disk requirement for direct and copying modes. Outer coordinator dependencies
   remain outside body-only savings; identify the largest remaining category explicitly.
4. **Cross-lane reuse potential.** Calculate projected action keys for eligible requests
   at the currently registered lanes' source states, including dirty inputs when available.
   Record lane bases and toolchain/configuration differences. Compare whole-manifest and
   projected-closure keys using the same requests; report request-, byte- and measured
   compile-cost-weighted match rates. Add a historical replay across recent base changes
   to distinguish same-base sharing from reuse that survives normal merges. Key matches
   are projected eligibility, not observed cache hits or proof of reproducibility. Missing
   historical toolchains/generated inputs make a sample unknown, not a hit or a miss.
5. **Latency and contention.** Measure hash/resolution/snapshot/setup overhead on small
   and large bodies. Compare equivalent four-lane miss demand for one action against
   the current sharded path, with fixed compiler-cache state and broker capacity. Before
   implementing store locks, report this as a measured single-build cost plus an explicit
   contention model, not an observed deduplicated run. The actual four-reader/single-builder
   experiment is required once the pilot exists; also measure four distinct misses and
   a shared-dependency change. The earlier 9 s unchanged builds say nothing about this cost.

**Decision rule:** do not start step 2 until this report identifies a material,
eligible source of duplication, demonstrates that projected keys survive unrelated
workspace additions, and estimates net savings after store/snapshot/scratch overhead.
The report must declare numeric targets for net retained GiB, projected weighted reuse,
hit/setup latency and four-lane miss latency *before* implementing the store. Explain
how those targets follow from the measured disk pressure and current build baseline;
then judge the prototype against them, without lowering targets to fit its result.
Until these measurements exist, this document claims no quantified payoff. If outer
coordinator artifacts dominate, or body reuse cannot beat its overhead, revise the
first action boundary or choose the simpler mode before spending on store primitives.
This is an engineering evidence gate, not an extra user-permission requirement.

### Required test matrix

| Area | Cases and expected evidence |
|---|---|
| Identity | Own/dependency source edits and semantically relevant manifest/lock/config changes, features/default-features, inherited dependencies/profiles/overrides, compiler/sysroot and target JSON changes cause misses; a newly influential patch/member must be discovered |
| Discovery | Add/remove/rename a module, untracked include, directory membership, dirty source, excluded-output boundary and escaping symlink; no stale hit |
| Generated inputs | Unsupported script/proc macro is rejected; supported future generators invalidate on inputs, tool and environment changes |
| No-op/unrelated edit | Second lane hits; unrelated source edits, adding/removing an unrelated workspace member, unrelated lock entries and manifest formatting preserve the key when resolved inputs are unchanged; rediscovery still runs |
| Invocation boundary | Real Cargo/Ninja incremental builds rerun after dependency edits on hit and miss paths; negative-control fixture demonstrates the test detects missing watches |
| Variants | Same archive name with different features/ABI/target yields independent results; link both variants after publication and run/check their behavior |
| Concurrency | Four same-key cold requests produce one builder and four safe consumers; four distinct misses retain parallelism; GC racing lookup, build-script exit before link, paused consumer, crash and PID reuse are safe |
| Atomicity | Kill during copy/manifest publication, disk exhaustion, truncated blob, poisoned manifest, missing object; no partial or silently accepted result |
| Reproducibility | Independent worker same-key rebuild, source-path relocation, source edit during snapshot/build, identical mtime with changed bytes, compiler-cache-disabled verification |
| Integration | Receipt survives restart; missing object forces valid rebuild; cold bypass works; scanner corpus and symbols remain covered; IR diagnostic layout remains usable |
| OS behavior | Existing `check-tooling`, strict source/binary checks, differential tests, PE equivalence and full three-architecture gate; screenshots inspected |

Extend `tools/test_nested_cargo_deps.py` and `tools/test_rust_storage.py` rather than
testing only a synthetic lookup function. Add store lifecycle/GC tests where the real
supervisor protocol lives. Tests must prove a cache hit actually skipped nested Cargo
and still linked the correct result. A matching exit code or filename is insufficient.

Measure baseline private/sharded storage and the pilot with the same sources and
same compiler-cache policy: first miss, second worktree hit, no-op, leaf edit, shared
dependency edit, and several concurrent lanes. Include source snapshots, store blobs,
manifests, private outputs and scratch high-water marks. Set rollout thresholds from
these observations, not the earlier 45.6% within-lane result.

## Remote evolution and rollout boundaries

Keep transport out of the compiler contract: local lookup first, optional remote fetch
of the same action/manifest/blob protocol, then local admitted compilation. Verify all
downloaded blobs before publication. Hashes prove byte integrity, not that the producer
built the claimed action: remote action mappings require authenticated trusted writers,
provenance and namespace policy. Untrusted contributions must not populate trusted
release caches. Do not upload private sources, secrets or logs by default; shared
debug-bearing artifacts can contain source paths and must follow explicit project policy.

Do not enable remote publication in the initial local pilot. Offline collaborators
must retain a functional local build path, with unsupported/offline behavior explicit.
Portable identity covers equivalent toolchains/platforms, not arbitrary cross-host ABI
compatibility. Storage compression and remote eviction are independent of blob identity.

Rollback is `vibeos-cli build --arch <a> --no-artifact-store`; it reconciles the CMake
mode to off and regenerates consumers through the existing sharded path. It must work
without deleting the store or disturbing another lane. **Owed** before widening the body
set: automatic reference retirement when a lane is pruned or removed (its durable refs
now outlive it and pin their blobs; the store is small, so this leaks bytes, not
correctness), and the per-request latency (about 0.8 s, mostly discovery and
revalidation, overlapped across a parallel build). No migration
of existing mutable targets into the trusted store by filename, and no pruning of other
lanes as part of this project.
