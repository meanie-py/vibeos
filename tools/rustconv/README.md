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
