# Build invalidation

The Cargo link edges now consume compiler-discovered dependencies instead of the
entire Rust workspace. There were three separate causes of excessive work:

- CMake listed every workspace source as an input to all 39 Cargo-linked modules.
- `reactos-link` also watched every crate's source directory. Invoking an
  otherwise unrelated module therefore reran its build script and link work;
  it was not merely a cheap Cargo freshness check.
- Nested builds removed `RUSTC_WRAPPER`, disabling the sccache wrapper configured
  by `vibeos-cli`. The shared compiler-environment cleanup now preserves that
  wrapper while removing the outer compiler/target pins.

`reactos-link/src/cargo_deps.rs` reads each nested staticlib build's emitted `.d`
file and publishes those inputs through `cargo:rerun-if-changed`. This carries
transitive Rust sources, embedded assets, and build-script inputs into the outer
Cargo depfile. Manifests of the contributing packages are included explicitly:
Cargo tracks these internally, but the outer build must know to invoke it first.
Crates emitting both `staticlib` and `rlib` share a dependency file that can name
the `rlib`; both spellings refer to the same compilation's input closure.

CMake retargets the outer depfile to the copied PE output. Its explicit bootstrap
inputs cover the link crate, declared body crates, shared link engine, workspace
configuration, lockfile, and compiler pins. Generated artifacts and CMake target
prerequisites retain their producer dependencies. Native compiler dependency
files continue to cover actual headers and assembly inputs. Missing or malformed
nested dependency data stops the build.

A cold build does not require an old depfile: its missing output runs the build
edge, which produces the complete dependency file for subsequent edits. A new
Rust module is discovered through the edit introducing its module declaration.
An unrelated source edit no longer wakes the edge in either Ninja or Cargo.

The obsolete CMake `rust_kernel` prerequisite was also removed from both kernel
variants. It compiled a separate `kernel-rust` archive that neither Cargo link
consumed. The actual UP and MP builds still build and link their respective Rust
bodies, with their existing features and generated-header prerequisites.

`tools/test_nested_cargo_deps.py` builds and runs real executables through CMake,
Ninja and nested Cargo. It verifies transitive source edits, embedded assets,
build-script data, manifest-only feature changes, an unrelated edit that must not
rerun the link, and paths containing spaces. It exercises both archive-only and
archive-plus-rlib crates. A recording compiler wrapper proves that the nested
dependency compiler retains its wrapper after the shipping environment cleanup.
These tests run under `check-tooling`.

Generated outer Cargo depfiles on all three architectures now show
`ntoskrnl-rust/src/ke/time.rs` feeding only `ntoskrnl` and `ntkrnlmp`, and
`win32k-rust/src/window.rs` feeding only `win32k`. Previously both were explicit
inputs to all 39 Cargo link edges per architecture. The CMake kernel archive
removal and dependency propagation passed the full three-architecture gate at
`/tmp/vibeos-cli/20260912-175050-yx_10_7g-gate-i386+amd64+arm64/summary.json`.

The build-engine changes, including cache-wrapper preservation, passed the full gate:
`/tmp/vibeos-cli/20260912-175728-yyqcu3no-gate-i386+amd64+arm64/summary.json`.
All builds, desktop/network smokes, conformance, source/binary lints, tooling and
differential tests passed; the only warning was the uncommitted tree. The host
link-engine unit tests also passed (5 tests).

The media freshness validator now checks each Cargo-linked PE against that same
compiler-emitted input closure. Its former workspace-wide newest-source check
incorrectly rejected unrelated services after a selective kernel rebuild. Missing
or malformed dependency evidence fails loudly. Directory watches recurse as Cargo
does. Regression tests cover unrelated edits, real source and manifest edits,
escaped paths, directory watches, and missing inputs.

A real amd64 CLI rebuild after touching the kernel and win32k source inputs
relinked exactly ntoskrnl, ntkrnlmp, and win32k; all binary checks and the media
freshness validator passed:
`/tmp/vibeos-cli/20260912-182641-zm1ujdix-build-amd64/summary.json`.
This was a warm dependency-invalidation probe, not a replacement for the clean gate.
An i386 refresh also passed the new validator and binary checks:
`/tmp/vibeos-cli/20260912-182741-277su0zb-build-i386/summary.json`.

## Permit investigation

The delayed work was a detached WoW64 integration gate for `7e7f1275ff1b`,
submitted at 15:57 and landed at 18:21 on 2026-09-12. Its gate artifact is
`/tmp/vibeos-cli/queue-batch-01789249935459649416/20260912-175215-7kjhce67-gate-i386+amd64+arm64`.
The user confirmed that lane runs from cron. Agent pause did not stop it.

Whole-build permits have since been replaced with runnable compiler/linker
admission; see [host job scheduling](host-job-scheduling.md). The cron continues
normally, with integration work participating in the shared execution budget
once it updates its checkout.

## CMake option projections for Cargo links

The shared link engine previously emitted a watch on all of `CMakeCache.txt` when
reading `ENABLE_CCACHE`, `_WINKD_` or `GDB`. That put the cache into 366 AMD64 PE
depfiles in the batch-6 rollout tree. Switching `VIBEOS_WOW64_NO_STAGE` during guest
restaging then invalidated those links even though their inputs were unchanged.
The recorded reuse restage executed 400 Ninja edges and 886 artifact-store hits,
with no store producer calls (`phase6-warm-run02/build-amd64-restage.artifacts/`).

CMake now generates a separate Boolean input for each option under
`CMakeFiles/reactos-link-config/`, preserving its timestamp when its content is
unchanged. Each link watches only the options it reads. Existing true spellings
and unconfigured pure-Cargo defaults are preserved. A configured build with a
missing or malformed projection fails with a reconfigure diagnostic.

The real CMake/Ninja/Cargo fixture in `tools/test_nested_cargo_deps.py` reproduced
all three consumers rerunning after a media-only option change with the broad
watch. It now proves that unrelated reconfiguration runs none of them, each real
option rebuilds only its reader, and turning options off updates executable
behavior. Identical reconfiguration is also a no-op. Native refusal tests cover
missing, corrupt and unknown inputs.

CMake also rewrites `compile_commands.json` on identical reconfiguration. A real
CMake control recorded the same SHA-256 with a changed mtime. The link graph now
has one `copy_if_different` producer for a stable copy of that database. Ninja
restat leaves its consumers fresh when commands are unchanged. The Rust depfile
retargeter verifies exact byte equality before substituting this declared input
in the copied PE rule. Cargo retains its full original database watch, including
for direct invocations. Other prerequisites and phony rules remain intact.

The native provenance check compares the snapshot against the current database
before accepting a build or boot. A changed database therefore fails verification
until Ninja has propagated it, even if timestamps appear older than the PE.
Missing or unequal snapshot inputs also refuse depfile publication. The real
fixture counts both Ninja edges and Cargo build-script runs, checks executable
behavior after a genuine compile-command change, and confirms that identical
reconfiguration executes no link edge.
