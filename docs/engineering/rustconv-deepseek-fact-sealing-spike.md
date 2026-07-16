# rustconv DeepSeek fact-sealing spike

Date: 2026-07-15

## Question

Can a deterministic harness make a cheap model useful for real Wine/ReactOS C++
translation without trusting it to copy constants or generate COM ABI artifacts?

This spike used `deepseek-v4-pro` on three real shell32 method bodies. The existing
`cpp_com` pipeline generated interfaces, vtables, object layout, factories, worker
signatures, helper bindings, and wide statics. DeepSeek saw only a body worker.

## Added experiment path

`rustconv translate --cpp-com --seal-facts` now creates a model-only view of each
worker:

1. `SUCCEEDED(expr)` and `FAILED(expr)` become exact signed-HRESULT comparisons.
2. Every numeric and symbolic constant occurrence outside comments/literals becomes
   an opaque `__RUSTCONV_FACT_NNNN__` identifier.
3. The response must contain the complete closed token set. Tokens may repeat because
   Rust sometimes needs a C value in both a type and initializer.
4. rustconv restores the original spelling before the existing constant/body audit.
5. The mode has its own response-cache namespace. `--cache-tag` provides another
   isolation dimension for A/B experiments.

The worker normalizer also now restores generated parameter names, replaces model type
aliases/return types with the generated signature when names and arity match, and
promotes a constant-only `let usize` to `const` when it is used as an array bound.

## Paid corpus

Artifacts are durable under
`~/vibeos-rust-staging/shell32-com/deepseek-fact-spike/` and the streamed raw responses
remain in `tools/rustconv/.responses/<timestamp>-<source>/`.

| Method | Unsealed result | Sealed result | What the harness learned |
|---|---|---|---|
| `CExeDropHandler::DragEnter` | Passed strict body audit and parsed | Passed after deterministic unused-parameter name restoration; parsed | DeepSeek preserved all four facts but renamed unused ABI parameters |
| `CExeDropHandler::Load` | Rejected: folded two source `+ 1` uses into one byte-count temporary | Raw sealed response preserved all three tokens; replay passes after generated-signature enforcement | Model rewrote `LPCOLESTR`/`DWORD`/`HRESULT` to primitive aliases even though it did not own the signature |
| `COpenControlPanel::GetPath` | Rejected: folded `38 + 1` to `39` | Fresh sealed run passed the full CLI audit and parsed; normalized worker passed a focused host compile | Rust duplicated the array-bound facts legitimately, then used a non-const local as an array bound |

Observed fresh sealed `GetPath` usage was 1,717 input / 4,588 output tokens, reported as
$0.01. `DragEnter` used 1,805 / 3,201 tokens. DeepSeek remains output-verbose even for
small workers, but the dollar cost is low.

## Result

The final harness accepts all three sealed paid responses when replayed through the
current deterministic transforms. The unsealed strict audit accepted one of three.
The two unsealed rejections were semantically reasonable constant folding rather than
wrong values, which is still useful evidence: sealing keeps the output mechanically
close enough for a strict, cheap oracle instead of asking an auditor to prove arbitrary
equivalence.

This is promising, not yet a bulk-translation green light. The focused `GetPath`
compile proves the observed normalization, but the production acceptance path still
needs to compile each candidate against the real target crate and feed bounded compiler
diagnostics into deterministic repair/model routing. Parsing plus lexical audits are not
enough for millions of lines.

## Follow-up spike: mechanical acceptance and cost evidence

The recommended candidate harness now exists:

- `rustconv candidate-check manifest.json` substitutes N workers into N isolated modules
  backed by an accepted shell32 module and runs one `cargo check` per architecture. Rustc
  diagnostics are attributed to the owning candidate and the complete oracle is durable
  under `.responses/candidate-oracles/`.
- `--repair` applies bounded repairs whose authority is either the generated worker
  signature, the accepted base worker, or Rust lexical structure. Current repairs cover
  constant array bounds, generated signatures, call-argument casts, local initializers,
  and mutable reassignment. It does not ask an LLM to interpret rustc.
- `rustconv translate` appends every deterministic, cached, generated, rejected, and API
  error outcome to `.responses/outcomes.jsonl`, including content identity, cheap routing
  features, tokens, cost, latency, model, mode, and sealing flags.
- `rustconv dup-census` measures exact and alpha-normalized C/C++ function duplication.
  Alpha results are deliberately census-only; no translation is instantiated from them.

### Duplication census

| Scope | Functions | Exact duplicate copies | Alpha-only copies |
|---|---:|---:|---:|
| `dll/win32/shell32` | 2,854 | 150 (5.3%) | 6 (0.2%) |
| `dll/win32` | 47,747 | 2,806 (5.9%) | 570 (1.2%) |

The broader scan covered 1,542 files. It had 1,559 AST alpha fallbacks and 193 bodies the
existing scanner did not extract. The exact reuse pool is real but not large enough to be
the whole strategy. The census also showed many tiny repeated shapes, strengthening the
case for deterministic zero-LLM templates. Alpha-renamed substitution remains unsafe until
scope, signature, substrate, and identifier re-instantiation are proven.

### Call sealing

`--seal-calls` now composes with `--seal-facts`. It replaces every known runtime call
occurrence, including deterministic `*_body` helper rewrites, with a unique opaque token.
The response must contain every token exactly once in executable code; missing, duplicated,
comment-only, or invented tokens fail before normalization. The mode has its own response
cache namespace.

Two fresh fact+call-sealed `GetPath` trials were produced. The first happened while exposing
a CLI bug: `--prompt` claimed to imply `--dry-run` but did not. No output was lost; the paid
artifact was moved intact to durable staging, and `cmd_translate` now forces dry-run before
creating an archive/backend whenever `--prompt` is set.

| Trial | Input / output | Cost | Raw x3 compile | Deterministic repair x3 |
|---|---:|---:|---|---|
| fact+call 1 | 1,828 / 4,026 | $0.00494 | E0308 FFI casts | PASS |
| fact+call 2 | 1,871 / 3,387 | $0.00425 | E0308 FFI casts | PASS |

Both responses passed the closed fact/call envelopes and strict body audit before compile.
The batched oracle then exposed only mechanical Rust issues. Trial 1 needed generated
signature restoration, three accepted-base call casts, and one accepted-base initializer.
Trial 2 needed the signature, the same three casts, and one lexical mutable-reassignment
repair. The accepted baseline stayed unchanged, and all three candidates compiled on i386,
amd64, and arm64 in one oracle run per architecture.

The current positive result is therefore narrower and stronger than “DeepSeek writes good
Rust”: on this representative worker, DeepSeek preserved the model-owned control flow while
the harness owned constants, calls, signature, compile acceptance, and bounded repairs.

### What to take seriously from the cost-floor brainstorm

1. **Take now:** exact duplicate mining, zero-LLM body templates, call sealing, batched
   compilation, a compounding deterministic repair library, an outcome ledger, and a static
   difficulty router once the ledger has enough examples.
2. **Measure next:** using sealed facts as a dependency manifest to remove definition-heavy
   prompt context. Sealing alone did not shrink this prompt: the call contract added input
   tokens. Context removal, not token replacement, is the hypothesized saving.
3. **Use selectively:** differential execution for truly host-runnable pure helpers. Most
   shell/COM methods are stateful and cannot use it cheaply.
4. **Deprioritize:** a second cheap-model “judge.” Compile plus closed lexical/ABI contracts
   are stronger and cheaper; another model call should only address residual semantic cases.
5. **Park:** fine-tuning until the ledger quantifies the residual that templates, DeepSeek,
   and deterministic repair cannot absorb. The accepted corpus is valuable, but clean
   function-pair extraction and evaluation are prerequisites, not a free by-product.

## Stratified 20-method routing experiment

The follow-up corpus used 20 real shell32 workers across `CActiveDesktop`,
`CExeDropHandler`, and `COpenControlPanel`: 10 zero-LLM template shapes and 10 model-tier
bodies spanning low, medium, and high static difficulty. Artifacts and the manifest are
durable under `~/vibeos-rust-staging/shell32-com/routing-corpus-v1/`.

Nine fresh DeepSeek calls plus one reused sealed `GetPath` response consumed 18,053 input
and 38,651 output tokens. Fresh spend was $0.04757; including the earlier cached `GetPath`
call makes the ten-body historical cost $0.05182. All ten model responses passed fact
sealing, call sealing, and the strict body audit before compilation.

| Route | Raw x3 compile | After bounded repair | Fresh cost |
|---|---:|---:|---:|
| deterministic template | 10/10 | 10/10 | $0 |
| DeepSeek body worker | 3/10 | 7/10 | $0.04757 |
| total | 13/20 | 17/20 | $0.04757 |

The useful router split is already visible: low model-tier workers were 4/4 repair-green,
medium were 2/3, and high were 1/3. The three remaining failures were not constant or
control-flow corruption: `Drop` and `Load` retained C API spellings that the isolated Rust
substrate did not expose, while `Open` needed Rust ABI/type adaptation.

### Cross-language semantic skeleton

`rustconv skeleton-check corpus.json` now parses each source C/C++ worker and candidate Rust
worker with tree-sitter and compares a normalized ordered control skeleton. It records
branch/loop/return shape and a fuzzy sequence score while ignoring language-specific syntax
and expressions. This corpus was 20/20 exact on normalized control skeletons; fuzzy scores
ranged from 0.8325 to 1.0.

That is useful evidence, but deliberately not an acceptance threshold. A compile-red
`COpenControlPanel::Open` still scored 0.9417: structural similarity can catch a dropped
branch or loop, but cannot prove FFI types, pointer semantics, call arguments, or ownership.
The correct role is a cheap veto alongside closed fact/call sets, never a replacement for
the compiler or semantic tests.

### Sonnet escalation result

The same three DeepSeek residuals were retried with `claude-sonnet-4-5-20250929`. The first
attempt exposed a harness bug: statement-position `TRACE`/`ERR` macros were omitted from the
sealed call set but remained visible to the model, so Sonnet treated them as runtime calls
and renumbered the envelope. rustconv now removes complete debug-macro statements from the
model-only body and dependency view before fact/call sealing; nested or conditional uses are
left intact. The fixed prompt retained exactly ten call tokens for `Drop`.

All three fixed Sonnet calls passed the deterministic envelopes and audit for about $0.05,
but compiled 0/3 both raw and after the existing bounded repairs. `Drop` and `Load` failed
mostly on the same unavailable C API spellings; `Open` emitted a body-local duplicate
`SHELLEXECUTEINFOW`, producing a distinct Rust type, plus an integer mismatch. This is the
important economic result: escalating model strength did not repair missing deterministic
substrate. The next investment should be generated FFI/type/callee context and mechanical
C-to-Rust call adaptation, not routine Sonnet retries for unresolved-symbol diagnostics.

### Initial routing policy

1. Instantiate known templates without a model.
2. Send low/medium non-template workers to sealed DeepSeek mechanical mode.
3. Require exact fact/call closure, strict body audit, semantic-skeleton compatibility, and
   x3 compile after bounded repair.
4. Classify unresolved symbols and duplicate local ABI types as harness/substrate work.
   Do not pay a stronger model until the deterministic substrate is complete.
5. Reserve Sonnet for residual semantic or structural failures after that substrate gate,
   especially high-difficulty workers.

### Deterministic substrate replay (2026-07-16)

The three DeepSeek residuals were replayed from the durable cache after extending the
bounded repair layer; no model calls were made. The same corpus now compiles **20/20 on
i386, amd64, and arm64** after deterministic repair, including 10/10 model-tier workers.

The new repairs are deliberately closed transformations rather than rustc-suggestion
application:

- `std` paths become `core` in the `no_std` candidate crate;
- known wide-length and shell-allocation spellings resolve to helpers already present in
  the accepted module;
- C `memcpy` becomes `core::ptr::copy_nonoverlapping` over `u8`, preserving the original
  byte count exactly;
- generated COM out parameters and anonymous-union member paths receive their ABI-owned
  casts/field paths;
- only the exact fact-sealed `L"%ls "` and `L"\"%ls\" "` strsafe formats lower to the
  existing append helper; arbitrary printf formats remain red;
- ABI struct field names, pointer nulls, and signed pointer offsets are restored only from
  the accepted base initializer/declarations.

The durable successful oracle is
`.responses/candidate-oracles/20260716-004449-286078/report.json`. The repair report names
every transformation per worker, so an acceptance can be audited without inspecting a
compiler scratch tree.

This does not make compilation a semantic proof. It does establish the economic result the
spike was looking for: the tested DeepSeek generation tier reaches 10/10 once common C/COM
surface differences are owned deterministically. Body-local `struct`/`union`/`enum`
declarations are now rejected as model-owned ABI surface; generated scaffold types remain
the only authority. The next corpus should therefore broaden source classes and exercise
these repairs without an already accepted per-function implementation before changing the
production router threshold.

### Out-of-sample class pilot: `CCopyAsPathMenu`

`CCopyAsPathMenu.cpp` was the first test with no accepted Rust implementation of the same
class. Its initial preflight failed loud on `CComPtr<IDataObject> m_pDataObject`. The
deterministic scaffold now maps a `CComPtr<T>` object field to one raw ABI slot, initializes
it null, routes assignment through a generated AddRef-before-Release helper, and releases it
after the translated destructor body. Sibling method calls and C++ reference declarators are
also normalized to generated Rust worker names/signatures before prompting.

The real 162-line class had two trivial bodies lowered for $0 and eight fact/call-sealed
DeepSeek workers. A parallel audit originally stopped on the first rejected future even
though the other paid responses had finished. All raw streams survived in
`.responses/20260716-002746-CCopyAsPathMenu.cpp/`; seven were recovered by deterministic
replay and the eighth had already committed. No function was regenerated. rustconv now
normalizes declaration-only model output before enforcing executable call tokens, seals
rewritten local/sibling worker calls under their actual Rust names, and drains every parallel
future so accepted paid artifacts reach the resume cache before a combined failure is
reported. Only the two futures processed before the old abort have recorded usage/cost
metadata ($0.00667); the total provider charge for the other completed streams is unknown.

`candidate-check` now supports `full_module: true`, so the assembled class can be compiled
without pretending an accepted per-function base exists. The resulting x3 oracle is
`.responses/candidate-oracles/20260716-004452-273547/report.json`. It is correctly red with
62 diagnostics on each architecture. The residual is now an explicit substrate manifest:
`CStringW`, `IShellItemArray`/complete `IShellItem`, menu/invoke-command structs and handle
aliases, Win32 clipboard/menu externs and constants, `IID_PPV_ARG`/`MAKE_HRESULT` lowering,
and the file-scope `CMVERBMAP` table. Three ordinary Rust casts remain after that surface.

That is the useful out-of-sample result: DeepSeek preserved and audited all eight workers,
but a complete new class still cannot be accepted until rustconv generates the header/global/
RAII substrate it currently borrows from an accepted module. Do not escalate this class to
Sonnet; model capacity is not the blocker. The next investment is deterministic per-class
substrate/addendum generation, starting with header-derived Win32 types/externs and exact
lowering for `IID_PPV_ARG`, then a reusable owned-wide-string replacement for `CStringW`.
