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

## Next routing experiment

Run 20 representative methods stratified by cheap static features (size, branches, loops,
calls, pointers, gotos). For each, record template eligibility, first-pass envelope/audit,
raw compile, deterministic-repair compile, tokens/cost, and whether a model-owned failure
requires Sonnet. That gives the first evidence-backed router rather than hard-coding a
DeepSeek optimism threshold from one easy worker.
