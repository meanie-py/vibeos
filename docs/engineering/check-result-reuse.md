# Check reuse and QEMU port handshakes

QEMU harness launches share a host-wide port handshake lock. It starts before
port selection and ends after QMP initialization succeeds. A failed launch is
stopped before releasing the lock. Guest boot and networking tests run outside
the lock, so parallel VMs remain parallel. Existing bind retries cover unrelated
Windows applications and older launchers that do not participate in the lock.
Both native and Windows builders participate; this also protects the scoped
Windows port-picker override from concurrent threads. Port-lock process tests
run every time check-tooling runs, including when other results are reused.
One launch also rejects duplicate candidate ports before assigning its forwards.

The CLI's standalone networking stage now passes `--skip-clean-build`, matching
the gate: smoke has already validated its media. The raw pytest clean-build
fixture previously rewrote the shared ISO while overlapping launches copied it.
A three-run probe caught two VMs with 44,040,192-byte and 35,586,048-byte ISO
copies against a 234,616,832-byte source; both stopped in the bootloader, while
the third completed all 15 networking tests. Their manifests were
`/tmp/vibeos-qemu-locks/1659809-1660401.json` and
`/tmp/vibeos-qemu-locks/1660217-1660987.json`; the failed probes were stopped by
their exact QEMU IDs after inspecting the copies. Raw pytest retains its explicit
clean-build behavior; CLI networking no longer starts a second writer.

The UD2 and self-recursion scanners cache per-binary analysis facts under this
checkout's `.cache/binary-facts`. Keys include binary contents and pathname,
scanner/helper implementation, actual analysis executable contents, architecture,
Python and relevant environment. The current binary inventory is always walked;
new, changed and deleted modules therefore participate immediately. Self-recursion
baselines are applied afresh to cached findings. Primitive verification also
reuses successful analysis of an identical kernel and tool inputs. Failed tool
execution, corrupt cache entries and inputs changing during extraction fail
explicitly. `VIBEOS_BINARY_CACHE=0` recomputes the analysis.

`check-tooling` records successful checks under the Git common directory in
`vibeos-tooling-receipts`. A receipt covers the current source contents (including
uncommitted and new source), execution command, environment, Python packages,
compiler/tool identities and declared generated inputs. Source hashes cover the
whole tracked tree conservatively. The rustconv suite additionally fingerprints
this checkout's generated include trees; it never reads a sibling's headers.
Runtime/process controls, live object checks and fixtures with undeclared external
Cargo inputs continue to run. A changed merge gets a different key. Identical
inputs can reuse results across worktrees without sharing mutable build output.
The live build-invalidation suite distributes its isolated temporary projects
over up to eight pytest-xdist workers (bounded by `lint_jobs()`). Install the
declared `.[qemu]` dependencies to provide xdist; a missing plugin fails the check.

Receipts expire after 24 hours and are published only after checking the inputs
again. Source/runtime changes during execution fail the check; generated headers
changing during a fresh run prevent publication of that suite's receipt. Output
reports how many checks were reused. `VIBEOS_TOOLING_CACHE=0` runs all checks
without reading or publishing receipts. Content verification deliberately avoids
timestamp-only shortcuts: it takes seconds, but removes repeated multi-minute
test executions in gate, submission preflight and integration.
