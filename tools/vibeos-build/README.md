# Native build infrastructure

`tools/build-infra` provisions the pinned-stable Rust binary offline, once per source
revision, then executes it directly. Source keys cover the manifest, lockfile, toolchain
pin and every file in `src/`. A per-checkout lock serializes provisioning; an edit during
compilation refuses execution. Compiler-environment cleanup is confined to the bootstrap;
commands receive their caller's environment. No Python is needed to launch the binary.

`vibeos-cli build` remains the build front door and the existing `host-jobs` broker owns
admission. This binary supplies build mechanisms; it does not install a second scheduler
or change concurrency. Add new build mechanisms to this library and command dispatcher,
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

Artifact adapters reuse a newline-delimited `--api-jsonl` process per Python process and
selected root. Calls are serialized, environment/source changes restart the server, and
a source change during a request refuses its result. Forked children close inherited
pipe copies and start their own server. The single-request `--api-json` interface remains.

The executor now keys native policy fingerprints and the compatibility transport alongside
its own sources. This deliberately changes action identities: old clients keep their old
objects, while new requests refill compatible actions. No live store is migrated, flushed
or rewritten. Build watches include the actual native executable and its source inputs.

The immutable artifact store/executor/runtime and Cargo orchestration remain Python
implementations for later batches in [the migration plan](../../docs/engineering/build-infra-rust-plan.md).
