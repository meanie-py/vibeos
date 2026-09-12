# Lint execution

`build --arch X` compiles, checks the resulting binaries, and stamps boot media.
Source analysis runs through `check` or the gate. `build --check` opts into the
advisory source sweep; the old `--no-check` remains accepted. Smoke retains its
artifact freshness checks and desktop markers.

Source scanners and independent tooling suites use the CPUs available to the
process, rather than a fixed eight-worker default. `VIBEOS_LINT_JOBS=N` sets an
explicit budget; invalid values fail. Results remain ordered by registration,
regardless of completion order.

## Incremental source facts

`tools/lint_facts.py` stores pure function results in worktree-local SQLite
databases under `.cache/vibeos-lint/`. Keys cover input contents, scanner source,
declared helper dependencies, the cache implementation, Python, and tree-sitter
versions. The cache never trusts source mtimes. WAL supports overlapping readers
and writers, and serialized values cannot execute Python code. A failed extraction
does not publish an entry; cache errors remain explicit failures.

| Analysis | Reused work | Work repeated each invocation |
| --- | --- | --- |
| Extern declaration drift | Types, declarations, definitions, calls per file | Workspace type resolution and declaration/definition comparison |
| Extern arity | Declarations and definitions per file | Current module reachability and symbol joins |
| Calling conventions | Shared declaration/definition facts for local and import checks | Crate definitions, import libraries, specs, and provider comparison |
| IRQL balance | Call sets, wrapper facts, function findings | Cross-file propagation; findings are keyed by the current summary |
| Type alias contradictions | Alias declarations per file | Current architecture tables and width comparisons |
| Match constants | Constant names and local bindings; module edges | Current dependency closure and glob resolution |
| Octal, reference counts, wide literals, dropped prints | File-local findings | Current source inventory and baseline comparison |
| Invented constants | Rust declarations per file | Current C/header oracle comparison |
| Macro shadowing | Empty function definitions per file | Current crate-level macro relationships |
| Dropped assertions | Function assertion counts per file | Current C/Rust correspondence |
| Mask drift | Lexical regions | Current constant catalog and comparisons |

Other scanners keep their existing execution and cache policies. Generated-header
and binary checks are not treated as pure source facts. This is a full inventory
check, not a git-diff-only scan: added/deleted files and changes to distant
definitions participate in the next comparison.

The expensive AST selectors execute inside tree-sitter instead of visiting every
token in Python. IRQL propagation intersects indexed call sets rather than running
a growing regex alternation over the workspace. Type-width checking constructs
one alias table per architecture rather than copying it per declaration. Module
reachability facts are shared by its consumers; match-constant resolution reuses
source text and resolves each imported module once per invocation.

`VIBEOS_LINT_CACHE=0 ./vibeos-cli check` bypasses the shared fact cache for comparison
with recomputation. Scanners with older private caches retain their own controls.
Baselines are always read afresh. Neither cached findings nor cache hits bypass
the zero-new-findings comparison. Fatal scanner diagnostics override otherwise
baselined partial stdout.

## Build dependencies in the gate

Tooling and the audited source-only partition start alongside builds. Binary
checks and smoke start as each architecture finishes. The differential host tests
wait for the requested x86 builds: their C oracles consume amd64 objects and
headers, and their SEH layout probe consumes i386 headers. They do not need to
wait for ARM64. Remaining generated-input checks run after the build matrix.

CMake no longer scans the whole Rust workspace for every module. Cargo depfiles
carry each module's actual input closure, so concurrent Cargo output cannot race a
workspace glob. A cached configure without `build.ninja` is retried automatically
on the next build.


## Parallel Cargo links

The normal CMake build gives each outer Cargo link its own directory:
`sdk/lib/rust/target/<crate>-<arch>-cargo-link/`. Previously all outer links
shared `target/`, whose Cargo lock serialized Ninja's apparently parallel jobs.
The existing inner Rust library directories and package features are unchanged.
The new directories are included in the architecture-specific clean operation.

Ninja's `vibeos_cargo_links` pool controls fan-out; each Cargo invocation uses one
job so nested Cargo work does not multiply the scheduler budget. On 24 visible
CPUs the default permits six concurrent links per build and four concurrent
builds in the host's existing four-permit pool. All three architectures can start
together. Explicit host policy still takes precedence over default admission.
`VIBEOS_CARGO_LINK_JOBS` overrides the Ninja pool at configure time, and
`VIBEOS_MAX_CONCURRENT_BUILDS` overrides default host admission.

The CMake/Cargo test uses two real build scripts with a mutual-start barrier:
sharing one Cargo output lock fails; the shipping directory/pool setup passes.
The ucrtbase binary surface check also invokes `nm` once for its complete object
set rather than spawning one process per object. An unreadable member fails the
complete comparison instead of dropping that member from the evidence.

## Validation in the overhaul worktree

2026-09-12, after merging `origin/master` at `da59236b670`, with 24 visible CPUs:

- Full warm `check --strict`: 17 seconds, zero new findings.
  `/tmp/vibeos-cli/20260912-171919-r7am4lnk-check-all/summary.json`
- Shared-cache-disabled `check --strict`: 79 seconds, zero new findings.
  `/tmp/vibeos-cli/20260912-171652-y744a89x-check-all/summary.json`
- Full warm gate: 244 seconds. Three builds, three desktop smokes, network smoke,
  three conformance lanes, binary/source lints, differential tests and all 46
  tooling checks passed. Dirty-tree warning only; no push stamp.
  `/tmp/vibeos-cli/20260912-171618-rbo7cr9k-gate-i386+amd64+arm64/summary.json`

The handoff reported 180 seconds for the old parallel source sweep. The warm
strict result is about 10 times faster; this is an iteration result, not a claim
that an empty fact cache costs 17 seconds. Build outputs remained worktree-local.
The clean builds also completed, but their cache state differs from the earlier
serialized run, so these results do not assign a numerical build speedup.
