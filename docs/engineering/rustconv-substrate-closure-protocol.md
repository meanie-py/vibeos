# rustconv substrate-closure protocol

This is the acceptance loop for a translated worker batch or full module. The model owns
only source behavior that requires judgment. Source/header facts, ABI, interfaces, vtables,
object layout, constants, imports, and compiler acceptance are deterministic authorities.
The protocol is language-neutral after translation: C and C++ use the same diagnostic
clustering and authority routing; the C++ frontend additionally owns ATL/COM discovery.

## Protocol

1. Generate or replay workers into a durable path under `~/vibeos-rust-staging/`. Never put
   paid output in `/tmp`, and never regenerate an existing worker without approval.
2. Assemble the complete candidate. For C++/COM, rustconv generates interfaces, vtables,
   `QueryInterface`, refcount/factory scaffolding, class state, simple source/SDK PODs, and
   literal header facts. None of those pass through the model.
3. Compile one synthetic crate across i386, amd64, and arm64:

   ```bash
   python -m rustconv candidate-check candidates.json --repair
   ```

4. Cluster the resulting compiler cascade into distinct missing authorities:

   ```bash
   python -m rustconv substrate-plan \
     tools/rustconv/.responses/candidate-oracles/<run>/report.json
   ```

   `candidate-check` also embeds this analysis in `report.json`. Add `--refresh` after
   changing the analyzer, or `--json` for machine-readable routing.
5. Close roots in authority order, then replay cached workers for $0:

   1. import an existing canonical shared definition;
   2. promote a duplicated module-local definition to shared authority;
   3. reuse an existing project binding such as `com-sys`;
   4. generate a simple SDK/header fact or POD layout;
   5. generate a source-local definition or an explicit C/C++ bridge;
   6. repair candidate scope/import mistakes;
   7. leave ambiguous or unsupported roots in a loud backlog.

6. Repeat until substrate-clean. Only then classify residual failures as bounded mechanical
   repair, deterministic CFG/ownership decomposition, or a stronger-model candidate.
7. Integration acceptance is still the real module build/link and the required three-arch
   smoke gate. A green synthetic crate is necessary evidence, not permission to cut over.

## Integration boundary rules

- Synthetic compilation proves generated Rust consistency, not PE/COFF linkage. Inline-only
  C++ helpers, mangled source-local symbols, and calling-convention decoration become visible
  only in the real shell32 link. Export a narrow explicit `extern "C"`/`WINAPI` bridge where
  the source definition is the sole authority, and use i386 as the decoration canary.
- Hybrid translation-unit cohab is valid when Rust alone owns activation and published
  vtables, while the retained C++ file owns a companion class or bridge. Never publish two
  implementations of the same class factory/vtable.
- Cached workers are normalized on every assembly. Every deterministic normalization must
  therefore be idempotent; replay tests must cover both raw model spelling and already-fixed
  spelling.
- `CComPtr` copy construction owns an `AddRef`, including copies from parameters and members.
  The generated guard owns scope-exit `Release`; model-written guards and ambiguous
  `Attach`/`Detach` ownership remain hard failures.
- Source-local numeric tables may be generated only from parsed initializer/header facts.
  Preserve row order, sentinel rows, and expression values; otherwise use a bridge or leave
  the authority red.

## Candidate manifest

The optional `source` field lets the authority index distinguish source-local definitions
from SDK/project gaps. Always include it for full-module runs.

```json
{
  "candidates": [
    {
      "name": "shelllink-full",
      "candidate": "/home/user/vibeos-rust-staging/shell32/CShellLink.rs",
      "source": "dll/win32/shell32/CShellLink.cpp",
      "full_module": true
    }
  ]
}
```

## Stop rules

- Do not grind hundreds of rustc sites independently. The unit of work is a distinct root
  plus its authority and impact count.
- Do not ask a model for a constant, SDK layout, WIDL interface, vtable, calling convention,
  COM offset, or known import.
- Do not accept generated PODs unless every field and fixed-array bound is proven. Unions,
  bitfields, expressions, conflicts, conditional layouts, and unparsed annotations stay red.
- Do not route an unresolved-symbol cascade to Sonnet. Stronger models are for residual
  semantic/structural judgment after the deterministic substrate is complete.
- Stop class integration when the remaining roots form another substantial companion class.
  Preserve the paid module and translate that closure as its own bounded unit; do not grow a
  guessed synthetic substrate until the first class happens to compile.
- Do not accept from AST similarity. `skeleton-check` is a veto for dropped/invented control
  flow; fact/call seals, body audit, x3 compilation, link, and smoke remain mandatory.

## Current scale evidence

On the 91-worker `CShellLink` full module, unchanged cached workers plus deterministic
closure reduced the compiler result from 743 diagnostics per architecture and 164 distinct
unresolved roots to 492 diagnostics and 51 roots. The SDK POD pass generated six previously
missing SDK record families, including their embedded dependencies and fixed arrays. This is
not an integration result; it demonstrates that one substrate addition removes repeated
failures across the whole class and all three architectures without another model call.
