# rustconv

`rustconv` is vibeos's deterministic-first C/C++ to Rust translation pipeline. Start with
the repository protocol at
[`docs/engineering/rustconv-substrate-closure-protocol.md`](../../docs/engineering/rustconv-substrate-closure-protocol.md)
and use [`docs/engineering/rustconv-pipeline.md`](../../docs/engineering/rustconv-pipeline.md)
for internals, model modes, caching, sealing, and repair behavior.

From a worktree, force Python to load that worktree's package:

```bash
export PYTHONPATH="$PWD/tools/rustconv${PYTHONPATH:+:$PYTHONPATH}"
python -c 'import rustconv.__main__ as m; print(m.__file__)'
```

`translate` now enables fact, call and wide-string sealing by default. Paid C
translation also requires c2rust anchors for every selected function and runs the
source-owning CMake target's profile audit before payment. Missing planned/output
functions fail before assembly. These are bounded checks, not semantic equivalence:
constant/call argument checks currently cover direct sealed constant arguments.

Use `--replay-only` to assemble existing responses with no API fallback. A changed
body or seal namespace with purchased output refuses a cache-miss purchase; recover
or repair the existing artifact. `--no-resume` is explicit regeneration and requires
the owner's approval. Protection opt-outs are named `--no-c2rust`, `--no-seal-facts`,
`--no-seal-calls` and `--no-seal-wide`. Legacy paid `submit`/`batch` paths require
`--unsafe-unprotected`; use `translate` for the guarded pipeline.

The production-shaped **plain-C module** loop (driver / Win32 DLL / service) is:

```bash
# 1. Derive the bindgen island + audit every `case` label. ERROR findings are BLOCKING:
#    a constant missing from the island becomes a silent catch-all match arm (§9.112).
python -m rustconv.profile_derive drivers/network/dd/rtl8139 --emit report
python -m rustconv.profile_derive drivers/network/dd/rtl8139 --emit profile   # paste into gen_driver_types.sh
python -m rustconv.profile_derive drivers/network/dd/rtl8139 --scaffold-dir sdk/lib/rust/rtl8139-rust

# 2. All three envelopes and C anchors are on by default; trivial
#    bodies are lowered deterministically for $0 in the same run. Preview free first.
python -m rustconv translate drivers/network/dd/rtl8139/ndis.c --dry-run --prompt
python -m rustconv translate drivers/network/dd/rtl8139/ndis.c

# 3. Close the mechanical rustc roots for $0 BEFORE paying for any repair. All three arches:
#    a numeric cast inferred from one bakes that arch's pointer width.
python -m rustconv crate-repair rtl8139-rust --arches i386 amd64 arm64 --dry-run
```

The production-shaped shell32 loop is:

```bash
python -m rustconv --arch amd64 translate path/to/Class.cpp \
  --target shell32 --cpp-com --class CClass --mode mechanical \
  --model deepseek --seal-facts --seal-calls -o ~/vibeos-rust-staging/Class.rs
python -m rustconv candidate-check candidates.json --repair
python -m rustconv substrate-plan \
  tools/rustconv/.responses/candidate-oracles/<run>/report.json
```

Translation calls cost money. Preview with `--prompt`/`--dry-run`, preserve `.responses/`
and staging artifacts, replay cache hits after deterministic improvements, and use
`--no-resume` only when an approved model upgrade really must bypass an existing cache hit.

For `--cpp-com`, rustconv owns the complete class/interface/vtable scaffold and a
source-selected support closure. It also binds same-arity local overloads from declared
types and materializes trailing C++ default arguments before prompting. Ambiguity fails
before payment: never ask a model to guess compiler semantics, ABI, constants, or layout.
WIDL-declared interface inheritance also owns physical slot aliasing, so a COM map exposing
both `IContextMenu2` and `IContextMenu` produces one derived slot and deterministic QI routes.
C/C++ function-parameter arrays decay to pointers for both `name[]` and `name []` spellings;
declarator whitespace must never leak into a model prompt or generated Rust signature.
Explicit C tag keywords in WIDL signatures are normalized too: `struct tagNMHDR *` becomes
`*mut tagNMHDR`, never the invalid Rust spelling `*mut struct tagNMHDR`.

Class state may include by-value `CStringW`; rustconv owns its construction, assignment, and
destruction. Legacy SDK direction markers (`IN`, `OUT`, `OPTIONAL`) are stripped before
signature derivation. For callback-heavy classes, static window procedures and their Win32
ABI substrate are generated rather than modeled, including pointer-width export aliases
such as `GetWindowLongPtrW` -> `GetWindowLongW` on i386. A body may use an
`unsafe extern "system" fn(...)` callback type, but may not declare an ABI block or export.

Assembly re-normalizes every cache hit and must be replay-idempotent: a second normalization
cannot duplicate `CComPtr` guards or release-paired null assignments. The scanner recovers
same-line return/calling-convention prefixes that tree-sitter leaves outside a qualified C++
method node, so `HRESULT WINAPI CClass::FinalConstruct()` retains its HRESULT contract.

The kernel `CUnknownImpl<Interface>` model has a separate `cpp-com --kernel-owned`
mode. It derives private Rust storage, constructor initialization, the SDK-typed
vtable, atomic reference counting and destructor frees, then dispatches through
retained C-compatible method workers. Unsupported inheritance/lifetime shapes fail.
The PortCls pilot's object is entirely Rust-owned; its C++ bridge remains an unlinked
compiler oracle. QueryInterface, factories and platform hooks are explicit Rust
implementations, not assumed complete by generation alone. Commands and measured
coverage are in [the pilot report](../../docs/engineering/rustconv-overhaul-pilots.md).

Read historical cost and routing evidence without API calls or bank changes:

```bash
python -m rustconv.cost_ledger --worktrees . \
  --output ~/vibeos-rust-staging/rustconv-speedrun/ledger.json
```

Run from the repository root with `PYTHONPATH` set as above. Explicit bank paths can
supplement local-worktree discovery. The report reconciles archive/outcome records,
retains per-call provenance, and marks unavailable retry, timing and billing evidence.
See the [measurement and scaling proposal](../../docs/engineering/active-lanes/rustconv-speedrun-scale.md)
for interpretation; recorded estimates are not invoices or semantic acceptance rates.

Host pilots and per-attempt accounting (experimental):

```bash
# C: compiler-led candidate, no model calls. Use a NEW persistent directory.
python -m rustconv.pilot_anchor --repo . \
  --output ~/vibeos-rust-staging/base64-anchor
python -m rustconv.pilot_base64 ~/vibeos-rust-staging/base64-anchor/libbase64-anchor.so \
  --repo . --seed 20260922 --random-cases 500 --max-length 4096 --mutation-check

# C++: replay an existing purchase, then compile and grade the saved candidate.
python -m rustconv.pilot replay tools/rustconv/.responses/host-pilots/txt2nls/nonthinking
python -m rustconv.pilot_repair --repo . \
  --output-dir ~/vibeos-rust-staging/txt2nls-replay
rustc --edition=2021 -O ~/vibeos-rust-staging/txt2nls-replay/txt2nls-faithful.rs \
  -o ~/vibeos-rust-staging/txt2nls-replay/txt2nls-faithful
python -m rustconv.pilot_txt2nls --repo . \
  --candidate ~/vibeos-rust-staging/txt2nls-replay/txt2nls-faithful \
  --all-codepages --mutations
```

`pilot generate-txt2nls --budget /persistent/path/budget.json --cap-usd 5` is a
**paid**, purchase-once experiment. It refuses existing work across registered local
banks. This checkout already has a purchase: replay/repair it. `retry-empty` permits
one explicit non-thinking attempt only after a known-charge failure with zero code
bytes. It cannot regenerate an existing candidate. The pilot is not the production
COM/OS translation path; a fact-token pass is not behavioral acceptance.

For regular synchronous DeepSeek calls, set both `RUSTCONV_BUDGET_FILE` and
`RUSTCONV_BUDGET_USD` to activate the persistent shared cap. The guard reserves full
context plus maximum output at conservative peak prices before API contact, serializes
reservations across processes, disables hidden SDK retries, and holds unknown charges.
Other models refuse a capped call until they have a verified reservation bound.
The separate semdiff review API also refuses to bypass an enabled cap.
Uncapped existing workflows remain uncapped. Do not change the cap midway through a run.

Each `submit_single` now preserves a unique `.responses/attempts/<id>/` with request,
raw stream, candidate, result, and separate validation records; DeepSeek reasoning is
stored separately when supplied. The cost ledger joins these to legacy outcomes by
attempt ID and counts intermediate retries. Missing/interrupted usage stays unknown;
recorded charges are estimates, not provider invoices. Historical overwritten attempts
cannot be reconstructed. See the [pilot results](../../docs/engineering/active-lanes/rustconv-speedrun-pilots.md)
for measured outcomes and limitations.

### Offline compiler candidates and cohort grading

`python -m rustconv.anchor_candidate FILE --module MODULE --arch amd64 --output BANK_DIR`
uses that architecture's real compile command and refuses existing output, known prior
translations, blocking profile findings, or missing compiler support. It never falls back
to a model. `profile_derive --source FILE` restricts the audit to a proven CMake member and
records partial scope; it does not certify unselected translation units.

Normalize saved compiler output with `python -m rustconv.anchor_lower RAW_RS --source FILE
--arch amd64 --output NEW_RS`. This derives C scalar widths from the target compiler rather
than Linux's Rust aliases. Later, use `--abi-json PREVIOUS_NORMALIZATION.json` instead of
`--source/--arch` to replay after the C source has been removed from the build. The saved
record must match the raw candidate's hash. Neither operation certifies semantic equivalence.

The offline C++ cohort lives in `rustconv.cpp_cohort`: banked selected-method graders,
a compiler-active numeric-macro audit, and bounded `std::stol`/`substr` helpers tested against
the genuine C++ library. Their reports retain rejected and unsupported methods. See its
README for individual commands and limits.

`vibeos-cli difftest --only cnt-difftest --case diff_memmove` runs an exact existing
comparison while retaining all generated-oracle checks. Missing or duplicate cases fail.
`vibeos-cli build --arch arm64 --extra-target jscript` additionally builds a DLL outside
ARM64's boot manifest while keeping the normal full boot build and its guards.

Run the frozen 12-source benchmark with `python -m rustconv.cohort_runner`; its
`prepare`, `verify`, and `run` commands pin and replay existing artifacts. It recompiles
the candidate sources, keeps development and held-out results separate, and refuses
measurement failures. Reproduction, outcomes, and remaining coverage gaps are in
[the cohort report](../../docs/engineering/active-lanes/rustconv-speedrun-cohort.md).

### Generated scalar and buffer difftests

`rustconv difftest-contract CONTRACT.json --out NEW_DIRECTORY` compiles genuine C and
existing Rust against a reviewed input contract, compares isolated executions, and requires
sensitivity mutants to fail. Contracts cover scalar/buffer cases, nullable pointers, typed
`char **` outputs, bounded grammar-generated immutable and mutable ASCII or UTF-16 strings, and
bounded libc-owned C-string results. It spends no model
budget. See the
[contract guide and shipping pilots](data/difftest/README.md) for supported types,
input generation, source provenance, and explicit host-only limitations.
