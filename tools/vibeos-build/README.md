# Native build infrastructure

`tools/build-infra` provisions the pinned-stable Rust binary offline, once per source
revision, then executes it directly. Source keys cover the manifest, lockfile, toolchain
pin and every file in `src/`. A per-checkout lock serializes provisioning; an edit during
compilation refuses execution. Compiler-environment cleanup is confined to the bootstrap;
commands receive their caller's environment. No Python is needed to launch the binary.

`vibeos-cli build` remains the build front door and the existing `host-jobs` broker owns
admission. This library also owns build preflight, reconciliation, target passes, artifact-store
acceptance and provenance through `cli_build*`; `cli_artifacts` owns shared command
supervision, logs and summaries. `vibeos-test::gate_*` composes builds and test lanes.
The Python entry points retain DTOs and external lease/context/task handles. Add new build mechanisms to this library and command dispatcher,
with output/refusal fixtures. Existing Python modules are compatibility adapters.

| Command | Purpose | Former entry point |
| --- | --- | --- |
| `cargo-depgraph --manifest FILE --packages NAME...` | Locked/offline all-feature resolved local package closure | `cargo_depgraph.py` |
| `retarget-depfile --source FILE --cargo-output FILE --target FILE --output FILE` | Replace only the rule prefix; preserve compiler prerequisites | `retarget_cargo_depfile.py` |
| `ninja-prune BUILD TARGET... [--canon-root DIR] [--delete]` | Keep every reachable edge's outputs, including side outputs | `ninja_closure_prune.py` |
| `link-guard LLD_ARGS...` | Run lld and reject unsafe order-dependent duplicate definitions | `link_guard.py` |
| `cell-manifest --write\|--check [--arch ARCH]` | Discover and serialize Cargo cells | `cell_manifest.py` |
| `media-manifest --write\|--check [--arch ARCH]` | Discover and serialize CD/CAB recipes | `media_manifest.py` |
| `sdk-recipe --write\|--check [--arch ARCH]` | Capture/factor SDK and host-tool recipes from Ninja | `sdk_recipe.py` |
| `build-bench --arches ARCH... [--clean] [--repeat N] [--output FILE]` | Run the build/lint trial graph with durable child receipts and independent resource samples | `tests/qemu/cli/buildbench.py` |
| `cargo-orchestrate --arch ARCH [--manifest] [--list] [--dry-run] [TARGET...]` | Plan Cargo cells, reuse SDK inputs, supervise producers and post-link/copy declared outputs | `cargo_orchestrate.py` |
| `strip-cd-media --lst FILE --out FILE --cache DIR --objcopy TOOL` | Strip media copies and publish checked PE checksums; `--file FILE` selects single-file mode | `strip_cd_media.py` |
| `ramdisk-tree LIST... --out DIR [--canon FROM TO]` | Validate and stage the case-folded ramdisk graft tree | `ramdisk_tree.py` |
| `artifact-executor [OPTIONS]` | Prepare and execute a captured body in the isolated worker | `artifact_executor.py` |
| `artifact-runtime request\|repair --supervisor FILE ...` | Serve supervised immutable-body consumers or exact missing-blob repair | `artifact_runtime.py` |
| `artifact-economics INVENTORY REPO OUTPUT [--eligibility-top N --scratch DIR]` | Attribute snapshots and model archive storage without changing the store | `artifact_economics.py` |

Use `--root DIR` to select a checkout. Manifest/recipe metadata operations select all
three architectures when `--arch` is omitted. Write operations render every selected
architecture before the first write. Check mode returns 1 on stale/missing output;
invalid input or failed extraction returns 2. Changed files are replaced atomically,
unchanged files retain timestamps, and an I/O failure does not promise multi-file rollback.

Explicit execution modes:

```sh
tools/build-infra sdk-recipe --run --arch amd64 --gen /path/to/generated
tools/build-infra media-manifest --assemble --arch arm64 --cd livecd --out /path/to/media
# --cells DIR masters from an explicitly selected cargo-orchestrate output set.
```

Recipe replay validates both recipe inputs before creating outputs and executes in recipe
order. SDK capture uses the selected checkout's canonical Ninja entry point. The media CLI
re-enters the canonical mount before mastering, preserving the stored command paths.
Media assembly retains CAB repacking, arm64 ramdisk generation, cell-source compatibility,
source-change detection and image provenance. External host tools remain required.

Ninja cleanup defaults to listing. Unknown required targets, malformed or missing includes,
escaping paths and symlinks refuse cleanup. The build adapter passes its declared optional
roots separately; every victim is checked before deletion. Only Ninja-logged outputs can
be victims. The linker adapter retains `VIBEOS_REAL_LLD`, `VIBEOS_LINK_GUARD=report` and
`VIBEOS_LINK_GUARD_LOG`; unsafe successful links have their output removed and fail.

The library contains the link policy below both higher-level tool libraries, avoiding a
linter/generator/build dependency cycle. `vibeos-codegen` calls the manifest library for
Cargo-cell refresh directly and shares SDK recipe factoring. The linter provisioner
fingerprints this transitive library too. Native helper paths are wired into CMake's
depfile edge and the linker shim; the build CLI calls native cleanup with its optional
root policy. Legacy imports use `build_infra_compat.py`, a JSON/DTO transport with no
Python algorithm fallback.

Validation lives in the crate's tests and `tools/test_build_infra.py`, plus the existing
helper tests through their adapters. Real host CMake/Ninja/Cargo invalidation tests invoke
the native depfile command. Frozen Python fixtures cover link policy, cell commands,
Unicode manifest formatting, Ninja closures and SDK factoring/replay; synthetic tools
exercise CAB and arm64 mastering without claiming an OS boot. Three-architecture live
configured-graph comparisons are distinct from cold OS build/smoke coverage.

Artifact configuration audits, Cargo input discovery, verified source snapshots, tracked
historical capture and storage economics now live in the native library. Python imports
retain their signatures through the JSON transport. Canonical identity bytes preserve
sorted ASCII JSON, arbitrary-size integers and Python float spelling. Discovery still
provides evidence only; it never authorizes a cache hit or compiler execution by itself.

Metadata adapters reuse a newline-delimited `--api-jsonl` process per Python process
and selected root. Environment/source changes restart that metadata server; changes during
a request refuse its result. Forked children close inherited pipe copies. The single-request
`--api-json` interface remains. Owned sessions, consumer guards and transactions use separate
bounded connections so contenders cannot block a producer's publication on the same stream.
These connections retain explicitly inheritable admission descriptors until work ends;
long-lived metadata processes do not retain them. Supervisor configuration audits obtain
the parent's current environment at each boundary without restarting its owned lease.

The executor now keys native policy fingerprints and the compatibility transport alongside
its own sources. This deliberately changes action identities: old clients keep their old
objects, while new requests refill compatible actions. No live store is migrated, flushed
or rewritten. Build watches include the actual native executable and its source inputs.

Store locking, publication, sessions/refs, repair, recovery and GC now live in Rust, as do
the isolated executor and supervisor. The v1 store layout and mixed-client publication
contract are preserved. Lock order remains consumer → action → GC; GC never waits on a
producer. Partial publications expose no action index, and failed or corrupt requests
retain evidence. Python store builders remain callbacks over a native two-phase transaction;
Python does not implement publication or verification.

Cargo consumers call `tools/build-infra artifact-runtime request` directly. An explicit
`VIBEOS_ARTIFACT_TOOL` overrides that helper path; failure is final. The Python executor and
runtime entry points exec the same native commands. The native supervisor owns leases,
configuration audits, receipts and acceptance. The existing host-jobs broker and externally
held disk reservation continue to own resource admission.

Native helpers watch their owner through pidfds and process-start identity, avoiding PID
reuse and thread-lifetime mistakes. Executor cancellation stops only its owned process
session. A namespace guardian observes abrupt executor death and reaps the isolated worker;
compiler subprocesses receive the explicit worker environment rather than supervision
metadata. Missing isolation or lifetime support fails the operation.

Private Rust `RunHost`/`RuntimeHost` test implementations replace Python algorithm monkeypatches;
production APIs have no fault-injection switches. See the
[coverage mapping](../../docs/engineering/artifact-native-test-coverage.md) for preserved
failure controls and real-process tests. Build-policy extraction is implemented; rollout measurements remain tracked in
[the migration plan](../../docs/engineering/build-infra-rust-plan.md).

Media stripping and ramdisk tree staging run directly from CMake and `mkramdisk.sh`.
Ramdisk planning rejects missing inputs, escaping destinations and file/directory
collisions before replacing an existing tree; later lists retain overlay precedence.
Strip cache names remain `sha1(source-path)[:12]-basename` for FEX compatibility. SHA-256
checks source, tool, native policy and twin bytes before reuse, so old metadata-only
entries refresh once. Source/tool changes during production cannot replace a completed
twin. Malformed cache manifests and unsuccessful or invalid producer output fail the
operation. The checksum uses actual file length; the old helper incorrectly counted
padding bytes on unaligned images. Original build binaries retain their debug sections.
Payload selection, stamps, foreign-build freshness and collision policy remain owned by
`tools/payloads/compile.py media-manifest`; the native strip command calls that explicit
interface and propagates failures. Payload tooling remains tracked for its later phase.

Cargo orchestration now owns legacy and committed-manifest planning, i386 target-content
isolation, generated SDK/version headers, native SDK recipe replay, bounded parallel
producer groups, post-link/copy ordering, verification and source provenance. It reuses
the host-jobs broker's existing lock and source stamp. Each producer has an owned worker
session, streams live output and exits with its parent; parallel cells do not share a
process-global signal handler. The compiler, pefixup, strip and the existing i386 forwarding
repair tool remain explicit tool authorities; their failures abort the build. No Python
orchestration algorithm remains behind the compatibility facade.

Frozen Python planning/header controls and real producer/cancellation tests live in
`cargo_orchestrate.rs` and `tools/test_cargo_orchestrate_native.py`. Media controls in
`tools/test_strip_cd_media.py` include real LLVM stripping of all three PE architectures,
cache content/policy changes, producer failure and atomic-output refusal; the ramdisk
suite retains overlay/collision controls and checks destination/alias protection.

Build policy planning now lives in `build_policy.rs`. Capacity validation and arithmetic
are shared from `tools/host-jobs/src/build_policy.rs`; that broker remains the single
admission authority. No capacity, memory, priority, cache-size or Cargo fanout defaults
change in this port. Invalid or unrepresentable capacity fails closed. The launcher,
Python transport and linter fingerprints include the shared policy source.

Native plans select fleet cache roots, preserve wrapper chaining and parent ownership,
construct cache/build/broker environments, parse cache counters, select Ninja limits,
read CMake settings and avoid redundant storage reconfiguration. The 114-case frozen
Python oracle is in `tests/fixtures/build-policy-python.json`. Parent environment and
CPU observations cross the adapter explicitly; long-lived metadata helpers do not
retain inherited build permits. Broker provisioning captures compiler output separately
from JSON protocol replies and preserves the existing source stamp and flock.

The native benchmark uses the same `vibeos-cli build` and lint entry points as normal
work. It refuses dirty source, duplicate outputs, missing/failed child summaries, absent
media and arm64 boot files, OOM counter increases, and changed source or capacity during
measurement. Workers preserve raw signal status in separate owned receipts and stream
live logs. Samples include host meminfo/vmstat/PSI, available cgroup counters and summed
RSS of the sampled child process trees; shared pages can count in several processes and
short peaks between samples can be missed. Host availability includes unrelated work.
These are observations with their method, not a claimed speedup or a private memory cap.

Python retains argument routing, Run/Stage rendering and the existing build control
flow. `buildpolicy.py` retains actual policy-lock lifetimes, heal/mirror and drained
activation/pause orchestration; `hostlock.py` retains legacy/QEMU admission adapters.
`cachecfg.py` contains DTO/presentation only, `hostjobs.py` delegates provisioning and
environment/status mechanisms, and `buildbench.py` delegates trial execution/sampling.
Those control-plane adapters remain for the CLI/QEMU phases of the tooling migration.

Dedicated command workers now keep a private runner session behind a controller. The
controller observes exit with `waitid(..., WNOWAIT)`, so the runner's PID remains reserved
until its group has stopped. Killing a runner cannot strand its ordinary grandchildren;
stubborn groups escalate from TERM to KILL. Detached services keep their own sessions
and are preserved, including on command failure. Receipt descriptors close before compiler
exec, worker argv is forwarded literally, and launch diagnostics travel in structured
receipts. Cache discovery uses the caller's execute permission rather than merely finding
any execute bit on the file.

The `retarget-depfile` command accepts paired `--dependency-source` and
`--dependency-snapshot` options. It checks exact byte equality before substituting
that prerequisite in the copied PE rule; other inputs and Cargo's original depfile
are preserved. CMake uses this for its stable compile-command snapshot, whose
producer runs before Cargo links. Provenance calls `depfile_check_compile_snapshot`
before accepting a Cargo-linked media tree, so an unbuilt command change fails
rather than being hidden by the snapshot. Missing or unequal inputs never publish
a new depfile.

Payload mechanisms live in `payload_spec`, `payload_build`, `payload_pe`,
`payload_closure`, `payload_modrun`, `payload_providers` and `payload_firmware`.
The existing compiler, PE checker, closure, modrun and firmware-fetch commands
are Python transport adapters. Native providers cover host applications, vendor
drivers, modrun suites and verified Pi Wi-Fi firmware. New payload mechanisms
belong in these modules; application contracts remain declarative YAML. Firmware
hash failures preserve existing cached files, and guest-set staging still
requires the other architecture's build receipt and complete dependency closure.

Shared CLI infrastructure is native: `cli_artifacts` owns atomic run/summary
publication, serial/failure archives, ledger writes and complete command logs.
Each command runs through the existing pinned process-group controller, with
stdout/stderr spooled to its own files. A command timeout is distinct from a
program that returns 124. Parallel commands use separate resource connections;
metadata requests cannot serialize their execution. Interrupted runs retain an
explicit incomplete marker rather than a false green.

`cli_arguments.json` is the CLI grammar and handler routing table. Add flags and
commands there, including their help snapshots, rather than growing the legacy
Python parser. The Python front door materializes Namespace and Path DTOs from
the native parser. The frozen pre-port argument corpus checks every command and
option family. `tooling_receipts` owns success-only cache lookup, source/generated
fingerprints, locks and revalidation; interpreter/package enumeration remains a
runtime discovery adapter. New tooling check registrations live in
`tools/vibeos-test/src/tooling_checks.json` and need a separately audited receipt
input contract in `tooling_policy.json` before they can be reused.
