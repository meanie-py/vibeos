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
