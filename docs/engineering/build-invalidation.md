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

## Follow-up after landing

Investigate permit ownership and lifecycle with paused agents. The user reported
that every other agent was paused, while `host-status` still showed the integration
conductor (PID 3835910) and its build children holding permits. This lane's final
builds recorded roughly 837 seconds waiting for CPU permits. That establishes what
the broker reported, not that the other lanes should have been active. Trace
background-process ownership, pause/cancellation behavior, and lease cleanup before
attributing this delay to legitimate concurrent work or simply raising limits.
