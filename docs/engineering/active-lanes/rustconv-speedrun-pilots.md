# rustconv scaling pilots — 2026-09-22

This implements the first experiment after [Task 0](rustconv-speedrun-scale.md), using
Sol for the two independent pilot implementations, Luna for the budget primitive, and
root-run differential grading. Neither pilot changes OS build selection. Production seal
defaults remain enabled. No existing translation was bought again.

## Measured results

| Route / target | Result | API cost estimate |
|---|---|---:|
| C: mbedTLS `base64.c`, c2rust anchor with two stable-Rust adaptations | 6,412/6,412 cases match genuine C; both deliberately wrong C mutants detected | $0 |
| C++: `sdk/tools/txt2nls/main.cpp`, first DeepSeek attempt | 32,768 output tokens, zero code, 244.023 s | $0.13780272 |
| C++: explicit non-thinking trial after zero-code failure | 6,363 output tokens, 31.986 s; fact-token check passes, compilation fails | $0.03313464 |
| Same C++ purchase, compile repairs only | Builds; 0/123 valid cases pass. All six negatives reject, which does not establish correct error paths | $0 additional |
| Same C++ purchase, numeric-parser repair | 128/129 cases pass; all 117 real codepages match byte-for-byte; leading-keyword-space case differs | $0 additional |
| Same C++ purchase, source tokenizer quirk restored | **129/129 pass**, zero unsupported; five injected mutants detected | $0 additional |

Total conservatively accounted spend: **$0.17093736 of $5**. No pending reservations.
These are token-usage estimates at peak cache-miss rates, not invoice measurements;
cache hits/off-peak prices may reduce the provider charge. Rates were checked against
[DeepSeek pricing](https://api-docs.deepseek.com/quick_start/pricing/).
The two API calls total 276.009 seconds. This is API time, not complete project time:
agent engineering, compiler setup, grading and repairs are separate work.
One comparison does not establish a fleet-wide model/reasoning policy.

Evidence under `~/vibeos-rust-staging/rustconv-speedrun/`:

- `budget.json` and `pilot-ledger.json`: independent persistent settlements and the
  ledger reconstructed from immutable attempt records; both sum to $0.17093736.
- `c-pilot/generated2/generation.json`: genuine c2rust output, transformations and compiler
  commands. `c-pilot/heldout-grade.json`: root's seed 20260922, 500 random cases up to
  4,096 bytes, boundary cases and mutation results. Grade execution took 1.89 seconds.
- `cpp-pilot/{compile-only,repaired,faithful}-grade.json`: genuine reference outcomes, complete
  output hashes and negative-case exit statuses. Five distinct C++ mutants were detected.
  `final-manifest.json` pins reference headers and all input hashes; `repair-manifest.json`
  pins every derivative and compiler command. Final candidate grading took 4.69 seconds.
- Canonical purchases are `.responses/attempts/a7f4e369ad38456eb8eec0a707688b32/` and
  `.responses/attempts/e37ebbd5ad98472e9f816110a15f7a69/` beneath `tools/rustconv`.
  Original output remains untouched. Separate accounting receipts recover the first
  call's recorded settlement; its initial result predates the added cost field.

## What failed and what the experiment supports

The C anchor needed removal of c2rust's obsolete `feature(asm)` attribute and a
compiler-accepted spelling for its empty inline-assembly barrier. Its algorithms were
not rewritten. The grade compares returns, output lengths, entire destination buffers,
and guard bytes with genuine C, including invalid input and insufficient capacities.
This is evidence to extend compiler-led lowering to more C modules before paying for
whole-function rewrites. It is not a repository-wide acceptance estimate.

The C++ model omitted `MAXIMUM_LEADBYTES` despite `#define MAXIMUM_LEADBYTES 12` being
present in the actual request. Preserving expression tokens did not preserve declarations.
Additional compile repairs addressed packed-field references, unwind-safety bounds and
an integer-width comparison. More seriously, Rust's `from_str_radix(..., 0)` is not C++
`std::stol(..., base=0)`: the buildable candidate rejected every valid table. The repair
implements decimal/octal/hex prefixes and partial numeric consumption. A held-out case
then exposed the source's unusual `substr(start, end)` call: its second argument is a
length, while the generated Rust treated it as an end offset.

The actionable direction is to own declarations and standard-library semantics in a
compiler-derived substrate, and give models smaller residual tasks. Compile success and
closed token sets cannot be acceptance criteria. Keep whole-TU model generation as a
measured baseline rather than assume it scales safely.

## Mechanisms now available

- Every synchronous `submit_single` has a unique immutable attempt directory, independent
  of legacy resume namespaces. Raw streamed content, candidate, usage, elapsed time and
  later audits remain separate. Run archives and individual filenames cannot collide on
  same-second invocations or repeated preambles. Legacy consumers retain numeric prefixes.
- `RUSTCONV_BUDGET_FILE` plus `RUSTCONV_BUDGET_USD` activates a persistent cross-process
  cap. Reserve the full documented input context plus maximum output before calling;
  disable hidden DeepSeek SDK retries; settle known usage conservatively; retain unknown
  charges. The guard refuses models without a verified bound and prevents the separate
  semdiff review API from bypassing an enabled cap. It is opt-in for existing
  workflows, and was enabled for both pilot calls.
- The ledger joins new attempts to legacy records by exact attempt ID, counts intermediate
  retries, rejects changed candidates/conflicting identities, and exposes incomplete
  requests. It does not reconstruct overwritten historical purchases.
- `pilot_anchor`, `pilot_base64`, `pilot_txt2nls` and `pilot` expose generation, reference
  grading, mutation checks and zero-cost replay. C++ purchase claims prevent rerunning the
  pilot against existing output; `retry-empty` requires zero code and known prior usage.
  An unsupported reference case prevents a successful full-corpus grade.

`pilot_repair` replays the three C++ repair stages against the pinned original purchase;
it refuses changed input and does not overwrite differing output. This is a specific
repair recipe with measured evidence, not a general C++ semantic lowerer.

Commands are in [rustconv's README](../../../tools/rustconv/README.md).

## Scope and remaining obligations

Both pilots run on this Linux x86-64 host. Base64 is explicitly enabled with
`MBEDTLS_BASE64_C`, which is disabled in the shipping configuration; it is an opt-in
behavioral experiment. It does not establish constant-time behavior, guest ABI correctness
or a shipping cutover. txt2nls is a standalone host tool, not COM or a Windows DLL; its
experimental prompt permits standard-library adaptation rather than applying production
call sealing. Invalid-input tests avoid undefined reference behavior and do not cover
all exception/range failures. Finite differential corpora are not equivalence proofs.

The next expansion should add C modules with different pointer/arithmetic/configuration
shapes and C++ API/lifetime families, measuring accepted output and repair effort separately.
Still owed from the larger plan: a deterministic-checker false-positive sweep, recoverable
cross-policy resume, typed constant-placement checks beyond existing call-argument guards,
checker/candidate-signature retry routing, and stage timing through guest integration.
These pilots supply mechanisms and counterexamples; they do not complete those tasks or
justify extrapolating cents per line to millions of lines.

## Verification

Final `vibeos-cli check-tooling`: all 59 checks pass (ruff, pyright and pytest).
Log: `~/vibeos-rust-staging/rustconv-speedrun/tooling-complete-fixed.log`.
`vibeos-cli check --strict --scope source-deferred` reports no new failures; its WARN
marks partial coverage, not a full gate. Evidence: run
`20260922-174221-qhu3239g-check-source-deferred-all`. OS smokes were not run for these
tooling-only and host-pilot changes. Nothing was committed, pushed or integrated into OS modules.
