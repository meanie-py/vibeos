# Native artifact coverage

`tools/test_artifact_store.py`: 31 original Python test definitions (34 parametrized
cases) become 28 definitions (31 cases). Three fault-injection-only definitions move
entirely below the native implementation boundary. Three other tests retain public
observable checks and move only their instrumentation below that boundary.

Store native destination: `tools/vibeos-build/src/artifact_store.rs`, `#[cfg(test)] mod tests`.
No production RPC fault-injection flags are added.

| Original Python test | Shipping native coverage / retained facade check |
| --- | --- |
| `test_digest_collision_compares_actual_bytes` | `digest_collisions_compare_bytes_and_cannot_replace_the_first_blob`: private test digest override forces two different byte strings to the same hash; publication refuses the second without replacing the first or creating its index. |
| `test_kill_after_blob_rename_never_exposes_partial_action` | `publication_crash_exposes_no_index_and_gc_recovers_without_waiting`: spawns the native unit-test executable with a private test-only crash worker; exits after blob rename and before index creation; asserts no index, orphan/stage/stale-lease GC. Helper `killed_during_publication` is removed with its Python monkeypatch. |
| `test_disk_full_manifest_publication_exposes_no_action` | `disk_full_during_manifest_write_exposes_no_action`: private action-index write hook returns ENOSPC; no index is visible and orphan blob/stage remain collectable. |
| `test_restore_refs_verifies_shared_action_once` | Exact hash count in `restore_refs_hashes_each_distinct_action_once_and_gc_can_drop_dead_refs`; Python `test_restore_refs_returns_one_verified_handle_per_shared_action` retains shared-handle deduplication and receipt count, then corrupts bytes between calls and requires a new restore to detect them. |
| `test_gc_hashes_only_the_blobs_it_deletes` | Exact zero-kept-blob hash count in `publication_hit_verification_and_gc_keep_bytes_immutable`; Python `test_gc_keeps_protected_blobs_without_certifying_their_content` proves GC leaves a changed protected blob alone while consumer lookup detects corruption. |
| `test_clone_or_copy_copies_and_falls_back_when_the_kernel_refuses` | `copy_range_fallback_restarts_both_files_only_for_allowed_errors` forces EXDEV/EINVAL/ENOSYS/EOPNOTSUPP and checks restart/truncation, then ENOSPC/EIO and checks refusal. Python `test_clone_or_copy_transfers_open_fds_through_the_native_tool` transfers >3 MiB through actual inherited descriptors and verifies bytes and parent FD lifetime. |

All other original Python tests are retained, including multiprocess single-producer,
parallel distinct actions, killed in-flight producer, supervisor attachment, durable
refs, threaded once-per-supervisor verification, corruption/quarantine, missing-blob
repair, live leases and PID reuse, filesystem accounting, GC callbacks, and runtime
scheduled-GC controls.

## Executor controls

`tools/test_artifact_executor.py` keeps public preparation/mutation failures, actual
namespace isolation, provision identity, relocation, normalized graph and three-arch
flag controls. Its former mocked run-success/failure/postcheck cases now execute
`artifact_executor.rs::tests::supervision_success_failure_and_postcheck_controls_preserve_final_evidence`
through a private `RunHost`. Native integrity tests cover the complete prepared-input
mutation matrix. Real signal and abrupt-owner-death tests use owned worker processes,
check inherited admission-lock release and prove an unrelated process survives.

## Supervisor controls

The old Python FakeAPI model is replaced by private `RuntimeHost` implementations in
`artifact_runtime_tests.rs`, using the shipping native store. Controls cover miss/hit,
clean and verification rebuild counts, nondeterminism, request/tool/config/source changes,
worker refusal, binding failures, complete failure receipts, zero-request final audits,
outer failure, expired owners, unsupported requests, explicit ineligibility, protocol
validation, clean-only per-arch retirement, missing-only exact repair, source/tool changes
during repair, shared-reference revalidation, scheduled GC and held-permit broker avoidance.

Additional tests use real config files to reject preflight and post-hit changes, accept
inert edits without changing the action, retain foreign/standalone/current-lease refs,
and preserve a failed request receipt when final tool validation also fails. Public
Python tests retain the environment-change callback, supervisor/receipt contracts and
CLI disk-admission routing. Fault injection is confined to Rust tests.

`test_artifact_runtime_real.py` compiles real isolated bodies from two independent
checkout roots and checks four consumers share one action. `test_artifact_repair.py`
reproduces missing-blob repair through the real worker. `test_artifact_ninja.py` and
`test_artifact_consumer.py` exercise the shipping Rust consumer's direct native command
protocol and CMake/Ninja invalidation, including deliberately insufficient-watch controls.
`test_artifact_inputs.py` checks bounded resource helpers retain inherited permits while
the persistent metadata server releases them. These host controls do not replace the
integrator's cold OS build and desktop smoke gate on all three architectures.
