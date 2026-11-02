# Build infrastructure migration to Rust

Batches 1–4 implemented on `lint-rs`; batch 5 is in progress and batch 6 remains planned.
Batch 3 adds native configuration audits, input projections, verified snapshots,
historical capture and artifact economics (2026-11-02).
The native commands and compatibility boundaries are documented in
[`tools/vibeos-build/README.md`](../../tools/vibeos-build/README.md). Host capacity
is unchanged; no performance gain is claimed before measurement.

The target is one stable Rust build-infrastructure binary with a reusable library,
provisioned by a `tools/build-infra` launcher. Keep `vibeos-cli build` as the front door
and migrate its build mechanisms behind that interface. Reuse the existing Rust
`tools/host-jobs` broker; do not introduce another admission scheduler. Lint logic stays
in `vibeos-lint`, generation in `vibeos-codegen`, and compiler/linker authority in the
existing tools. The later CLI/QEMU migrations retain their own scope.

## Inventory and boundaries

The initial perimeter below is 23 Python files / 11,168 lines in this checkout at
`93d5def2dfe`. These are source lines, not independent tools or a work estimate. Re-read
callers and tests after merging current master before implementing each batch.

| Area | Current files | Destination / boundary |
| --- | --- | --- |
| Dependency and cleanup helpers | `cargo_depgraph.py`, `retarget_cargo_depfile.py`, `ninja_closure_prune.py`, `link_guard.py` | Native graph/depfile/cleanup commands; duplicate-symbol policy lives in the shared low-level library with a native linker adapter |
| Build and SDK manifests | `cell_manifest.py`, `media_manifest.py`, `sdk_recipe.py` | Native parse/validate/emit/replay modules; reuse codegen's CMake and compile-recipe facts where contracts match |
| Cargo cell execution | `cargo_orchestrate.py` | Native execution plans and supervised per-arch execution |
| Immutable artifact store | `artifact_config.py`, `artifact_inputs.py`, `artifact_worker.py`, `artifact_executor.py`, `artifact_store.py`, `artifact_runtime.py` | Shared native descriptor/snapshot/store/executor/runtime modules |
| Store analysis | `artifact_history.py`, `artifact_economics.py` | Native consumers of the same descriptors; keep report formats compatible |
| Media staging | `strip_cd_media.py`, `ramdisk_tree.py` | Native bounded staging/copy plans; leave guest boot and hardware payload tools for their phases |
| Build CLI adapters | `tests/qemu/cli/{build,buildpolicy,cachecfg,buildbench,hostjobs}.py` | Extract build mechanisms; retain Python argument routing, Run/Stage reporting and the host-jobs adapter until the CLI phase |

Related contracts: [build invalidation](build-invalidation.md),
[immutable artifacts](immutable-build-artifacts.md),
[capacity policy](build-capacity-trial-plan.md), and
[build system](build-system.md). Existing Rust `reactos-link`, pefixup and host-jobs
remain owners of their present responsibilities.

## Delivery order

Each batch must be independently usable and landed through the integrator. Ports keep
existing entry points as thin adapters while callers move. After a caller switches,
its Python implementation stops owning the algorithm; an old revision remains the
parity oracle. Do not maintain two evolving implementations.

1. **Native foundation and dependency helpers.** Define typed requests, plans, errors,
   architecture selection and subprocess results; add the binary/launcher. Port the
   Cargo metadata closure and depfile retargeting first, then Ninja closure pruning
   and the linker adapter. Cover escaped paths, side outputs and malformed input.
   Switch their CMake/build callers only after fixtures and refusal tests pass.
   Cargo metadata and compiler depfiles remain authoritative; no hand-guessed closure.
2. **Manifests and SDK recipes.** Port cell/media discovery and deterministic output,
   then SDK/host-tool recipe capture and replay. Separate inspect/check from write/run.
   Compare all three configured graphs against the frozen Python outputs. Remove the
   two Python reporter calls from native Cargo-cell conversion once their native
   replacements are available, without adding a dependency cycle between the two
   libraries. Factor shared facts into a lower layer if necessary.
3. **Artifact descriptors and snapshots (implemented).** Port config audit, input discovery, verified
   capture, historical capture and economics/report consumers. Compare canonical JSON
   bytes and key derivation under a fixed fixture implementation identity, including
   Unicode escaping, integer handling, environment,
   symlinks, renamed libraries, embedded inputs and mutation during capture. Discovery
   remains a candidate calculation; it never independently authorizes a cache hit.
4. **Artifact store and execution (implemented).** Port atomic publication, sessions/refs, lock and
   lease protocols, recovery/repair and GC; then the isolated executor and supervisor.
   Preserve lock order (consumer -> action -> GC), release permits on failure, and
   retain externally held disk admission. Exercise competing writers, cancellation,
   owner death/PID reuse, corrupt objects and incomplete publication. Use a disposable
   store for mixed-client tests. Deliberately version changed implementation identity;
   budget the resulting cache refill and retain old objects for old clients. Do not
   flush or rewrite the live fleet store to make parity easier.
5. **Cargo orchestration and media staging (in progress).** Port target planning, generated SDK reuse,
   build/post-link/copy ordering, strip/media/ramdisk staging and provenance. Require
   declared outputs to exist and verify hashes/dependencies. Wire the native mechanisms
   under the existing build CLI; preserve refusal of stale media and mutable sibling
   build inputs. Missing tools, failed producers and invalid dependency data stop the
   requested operation instead of falling back to an unchecked build path.
6. **Build policy extraction and rollout.** Move build-specific job/cache/environment
   planning behind the native API while keeping the CLI's reporting contract. Preserve
   host-jobs protocol, memory limits and lease ownership. Move any policy genuinely
   shared by scheduling clients into the existing broker/library rather than copying
   it into the new binary. Complete cold/warm and cross-lane trials, then retire obsolete
   algorithm implementations and document the remaining CLI adapters.

Batches 1–2 form the first implementation chunk. They close the generator's remaining
Python manifest calls and exercise the new native build interface before touching
shared cache state. Parallelize independent modules only after shared types, path
rules and error semantics are established; integrate shared files and run gates centrally.

## Verification and performance evidence

Preserve and adapt the existing `test_cargo_depgraph`, `test_nested_cargo_deps`,
`test_retarget_cargo_depfile`, `test_ninja_closure_prune`, `test_link_guard`,
`test_build_invalidation`, `test_cell_manifest`, `test_sdk_recipe`,
`test_cargo_orchestrate`, `test_ramdisk_tree` and `test_artifact_*` coverage. Compatibility
adapters must exercise shipping native implementations, with no Python fallback.

Run `vibeos-cli check`, relevant native/adapter tests and `check-tooling` for Python
changes before each handoff. Changes on the actual build/link/media path also need
cold build artifact verification and independent desktop smokes on all three arches;
coordinate the full gate with the integrator. A parser-only batch can land before the
callers switch without requiring an unrelated OS rebuild.

Use the existing `vibeos-cli build-bench` and durable run summaries for the measurement
ladder: no-op warm build, one body-source edit, one transitive/embedded/generated-input
edit, cold lane with warm shared store, and concurrent cold build plus full lint check.
Record the revision, toolchain, cache state, broker policy, outputs, elapsed time,
permit wait and process-tree memory. Independently sample `/proc`/cgroup memory and
host memory/pressure; a tool's own peak report is not sufficient evidence about itself.
Do not compare two warm-up histories as though only the implementation changed.

Accept a batch when behavior/refusals and artifacts match, invalidation tests still
reject stale output, and resource usage has no unexplained regression. Investigate
measured bottlenecks before changing concurrency or cache policy. Faster host parsing
may help; compile/link time will still belong to the toolchain.

## Lint/generator audit behind this plan

Fresh `origin/master` was `8e3fb97cca5` when audited. Compared with our merged baseline
`bbe71c54f57`, the only newly added Python file was a rustconv test. The newest lint,
`scan_minmax_narrowing` (`f5f50c8c7e3`), is a Rust plugin. The existing
`tests/qemu/cli/test_rustlints.py::test_plugin_inventory_matches_gate` requires the
source gate registry to equal the native routing inventory.

Older Python ad-hoc scanners (`scan_arch_cfg`, `scan_probe_then_deref`,
`scan_sibling_drift`) and the layout oracles (`verify_offsets*`) still exist; they
predate the lint migration and are not evidence of newly added Python gate rules.
Track their eventual consolidation with diagnostics/translation tooling rather than
calling the entire Python estate retired. AGENTS.md and both native tool READMEs now
direct new lint and generator implementations to Rust; existing Python compatibility
entry points do not authorize new Python algorithms.
