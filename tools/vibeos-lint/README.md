# vibeos-lint

One stable Rust host binary owns lint check logic. Source plugins live in `src/rules/`;
artifact and standalone diagnostic plugins live in `src/binary/`.

Use the front door, which provisions the binary offline:

```sh
./vibeos-cli check                             # source registry and baselines
./vibeos-cli check --strict --scope arch --arch amd64
./vibeos-cli lint --list-rules
./vibeos-cli lint --arch i386 --rules scan_self_recursive
./vibeos-cli lint --arch arm64 --rules verify_arm64_unwind
./vibeos-cli lint --arch amd64 --rules scan_flex_array_panic --explain CrateOrSymbol
./vibeos-cli lint --arch amd64 --rules scan_captures --path path/to/llvm-ir
./vibeos-cli lint --arch amd64 --rules scan_ir_mismatch --path path/to/module.ll
./vibeos-cli lint --arch amd64 --rules scan_bind_all_matches --package example-rust
```

`lint` uses check semantics: findings fail according to each plugin's policy; incomplete
coverage fails loudly. Standalone diagnostic checks do not become mandatory gate stages
merely by being available. Gate, static smoke assertions and cohab coverage invoke this
same binary through the existing Python CLI control plane.

Binary extraction is checked, content-addressed, locked and compressed. Its identity includes
executable and input contents, resolved paths, arguments, implementation and relevant environment.
Cache hits revalidate input/tool metadata; baselines and allowlists are applied after extraction.
Failed extraction never publishes a record; corruption is an error. `VIBEOS_BINARY_CACHE=0`
disables persistent reuse. Large disassemblies are released between checks.

Clang, rustc, Cargo and binutils remain compiler/tool authorities. Artifact drift checks
invoke existing Python **generators** into temporary outputs or request read-only recipes;
Rust compares their results with the committed artifacts. Generator implementation, the
Python CLI and migration reports are separate migrations. `rust_warning_census.py` remains
a reporting utility; its gate's deny policy lives in Rust.

For parity against historical scanner bytes (including retired imports):

```sh
python3 tools/lint_parity.py --oracle-revision REV --binary-rules scan_ud2 --arch amd64
```

This compares the same existing artifacts; it does not certify their freshness or boot health.
Coverage controls and failure cases live in Rust tests and `tests/qemu/cli/test_rustlints.py`.


Source execution defaults to at most eight workers and four million retained syntax nodes.
Trees are admitted after a second request; sources above 256 KiB remain weakly cached.
Active consumers and other fact caches are outside that node budget.
Masked comment views and lexer-cleaned text share a 128 MiB LRU retention budget, measured by
string buffer capacity. Active readers can exceed this budget; evicted views
are rebuilt on demand. Weak references share active views without retaining
their text buffers after eviction. Line indexes are lazy, with compact 32-bit
offsets for files up to 4 GiB and full-width offsets for larger inputs.
`VIBEOS_LINT_JOBS` overrides worker count with a positive integer;
`VIBEOS_LINT_AST_CACHE_NODES` overrides retention with a nonnegative integer (zero disables
retention). Library callers can use `Corpus::with_ast_cache_nodes`.

Set `VIBEOS_LINT_TIMINGS=1` for snapshot, shared-index and rule timings. Optional
`VIBEOS_LINT_CACHE_STATS=1` reports initialized payload sizes and syntax/text cache counters
to stderr without forcing new parses or views; these counts exclude allocator overhead.

The CLI uses pinned `mimalloc` by default. Linux builds also enable its C malloc override,
covering tree-sitter allocations. The library declares no global allocator. For allocator
comparisons, restore the system allocator without changing rules or worker limits:

```sh
cargo build --offline --locked --release --no-default-features \
  --manifest-path tools/vibeos-lint/Cargo.toml \
  --target-dir .cache/vibeos-lint/system-target
```

Run `.cache/vibeos-lint/system-target/release/vibeos-lint` directly for that control;
`vibeos-cli check` continues to build and use the default allocator.

On a fresh machine, fetch the allocator crates before the normal offline bootstrap:

```sh
cargo fetch --locked --manifest-path tools/vibeos-lint/Cargo.toml
```
