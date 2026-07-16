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

## Recommended next spike

Add a candidate harness that:

1. substitutes one worker into an isolated generated class crate;
2. runs stable `cargo check` for the target ABI surface;
3. classifies diagnostics into deterministic repairs (signature/type aliases, const
   context, imports, pointer casts) versus model-owned body failures;
4. retries only the failed worker with a bounded diagnostic prompt;
5. routes to Sonnet after a fixed DeepSeek attempt budget;
6. measures first-pass compile, deterministic-repair compile, final compile, audit, token
   cost, and wall time over 20 representative methods.

That experiment will answer the real routing question: what fraction of ordinary Wine
methods can finish on DeepSeek automatically, rather than merely produce plausible Rust.
