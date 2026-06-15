# rustconv — Pipeline Internals

Deep reference for the `tools/rustconv/` C-to-Rust translation pipeline: the
pipeline stages, key files, post-processor rules, the probe-migration and
Stage-3 design, and the Codex cross-model path. The high-level "what it is",
the CLI command list, the money-warning, the cost table, and the static-analysis
commands live in `AGENTS.md` → "rustconv — C-to-Rust Translation Pipeline".

## Pipeline

```
C source → [scanner] tree-sitter parse → [resolver] match vs ntkrnl-sys catalog
→ [prompt_builder] system prompt + C + imports + exports → [batch_runner] LLM API
→ [postprocess] safe mechanical fixes → [assembler] write crate → [checker] cargo check ×3
```

`rustconv translate` accepts HAL paths and is HAL-aware (separate
`system_prompt_hal.md`, hal_ddk imports, auto-CMake edit). Output quality is
~70% useful for HAL — review before integrating (memory
`feedback_rustconv_hal_target.md`).

## Key Files

```
tools/rustconv/
├── rustconv/
│   ├── scanner.py         # tree-sitter C parser (functions, globals, SEH, arch ifdefs)
│   ├── catalog.py         # Index ntkrnl-sys symbols (~9,700 per arch)
│   ├── resolver.py        # Match identifiers → imports + gap warnings
│   ├── spec_parser.py     # Parse ntoskrnl.spec / hal.spec
│   ├── prompt_builder.py  # Assemble system + per-file prompts
│   ├── batch_runner.py    # Anthropic + Gemini API client
│   ├── postprocess.py     # Safe mechanical fixes (type names, imports, enums)
│   ├── assembler.py       # Write to ntoskrnl-rust/src/{subsys}/{file}.rs
│   ├── checker.py         # cargo check ×3 targets → structured errors
│   ├── probe_migrate.py   # Stage 2: ProbeFor{Read,Write} → Result<T>
│   ├── verify_constants.py / verify_abi.py / verify_ffi.py  # Static-analysis verifiers
│   └── config.py          # Paths, model config, token budgets, Target dataclass (kernel/hal)
├── templates/
│   ├── system_prompt.md       # Kernel translation rules (~5K tokens, prompt-cached)
│   ├── system_prompt_hal.md   # HAL-flavor translation rules
│   ├── function_prompt.md     # Per-function prompt template
│   └── probe_migrate_prompt.md  # Stage 2 migration rules
└── data/
    ├── gap_functions.toml         # FORCEINLINE/macro functions not in bindgen
    └── trace_catalog_{arch}.json  # debugcon byte-marker decoder catalogs
```

## Post-Processor Safety

`postprocess.py` only does context-free transforms that are always correct:
- Type name underscore-prefixing (`OBJECT_HEADER` → `_OBJECT_HEADER`)
- Import fixes (`ddk::*` warning, pool type dedup)
- Enum variant name prefixing (`ObCreateHandle` → `_OB_OPEN_REASON_ObCreateHandle`)

Union field access (`__bindgen_anon_N`) is NOT done by the post-processor — it's
context-dependent and handled by the system prompt teaching the LLM to generate
correct accessors directly.

## Probe Migration Stage (`probe-migrate`)

Second pipeline stage that migrates bare `ProbeForRead`/`ProbeForWrite` calls in
already-translated Rust code to use the Rust-native probe API
(`ntkrnl-sys/src/probe.rs`). Operates file-by-file with
preprocess→LLM→postprocess pattern.

```bash
python -m rustconv probe-migrate se/sqos.rs           # Single file
python -m rustconv probe-migrate se/ -m haiku          # Batch subsystem
python -m rustconv probe-migrate se/sqos.rs --scan-only  # Show probe sites
python -m rustconv probe-migrate se/sqos.rs --dry-run    # Preview without writing
```

Key files:
- `rustconv/probe_migrate.py` — scanner, preprocessor, postprocessor, runner
- `templates/probe_migrate_prompt.md` — 10 migration rules for LLM
- `sdk/lib/rust/ntkrnl-sys/src/probe.rs` — Rust-native probe API

The probe API returns `Result<T, NTSTATUS>` instead of raising exceptions:
```rust
probe_read_user::<T>(ptr, mode) -> Result<T, NTSTATUS>
probe_write_user::<T>(ptr, value, mode) -> Result<(), NTSTATUS>
probe_user_buffer(buf, len, align, mode) -> Result<(), NTSTATUS>
probe_capture_unicode_string_header(ptr, mode) -> Result<UNICODE_STRING, NTSTATUS>
```

**Status:** SE and OB subsystems fully migrated to Rust-native probe API. All 3
arches build + smoke tests pass. Other subsystems still use bare
`ProbeForRead`/`ProbeForWrite` calls — see memory
`feedback_translation_philosophy.md` for staging.

## Multi-Stage Pipeline Architecture

The translation pipeline has multiple stages, each refining the previous output:

```
Stage 1: C → Rust        (rustconv submit/batch)   — bulk transliteration
Stage 2: Probe migration (rustconv probe-migrate)  — ProbeForRead/Write → Result<T>
Stage 3: Result+Drop-RAII idiomation (planned)     — de-transliterate to idiomatic Rust
```

**Stage 3 is NOT "SEH idiomation."** There is no SEH to convert: `USE_DUMMY_PSEH`
makes `_SEH2_EXCEPT` handlers dead code on all 3 arches (see DUMMY_PSEH section
in AGENTS.md), and the kernel is `panic=abort` with no unwinding. Stage 1 already
produced Rust where the exception path doesn't execute. Stage 3's job is to
de-transliterate the *normal* path: replace the C goto-cleanup cascades and
manual lock/unlock pairs with `Drop`-based RAII (`Drop` *is* the `finally`, and
it is sound here precisely because every scope exit is an explicit `return` — no
unwind), bare `NTSTATUS` i32 with `Result<T, NTSTATUS>`, and magic integer
constants with `enum`/newtype/`bitflags!`.

The shape per function: an idiomatic `<snake_name>_rs(...) -> Result<T, NTSTATUS>`
core (suffix `_rs`; distinct from the existing `_rs_<CName>` inline-shim
*prefix*), with the existing `#[no_mangle] extern "system"` C symbol collapsed to
a thin trampoline (`Ok(v)` → `write_volatile` outparams + `STATUS_SUCCESS`,
`Err(s)` → `s`). The trampoline MUST keep `write_volatile` on every
output-pointer write (a naive `*out = v` reintroduces the captures(none)/ud2
miscompile) and keep the `extern "system"` ABI. Adoptable one function at a time
so smoke stays green throughout.

Stage 3 will operate **function-by-function** (not file-by-file) using paired
C+Rust function context, a dependency graph (SQLite cache), and Rust-native API
surface for RAII guards, safe user-mode copies, and Result patterns. The LLM sees
the original C alongside the current Rust translation, plus callees and type
context.

## Codex CLI (Cross-Model Translation)

OpenAI Codex CLI (`codex`) can be used as an alternative to rustconv for
C-to-Rust translations. Cross-model translation catches different blind spots
than same-model review.

```bash
# Non-interactive translation
codex exec --full-auto "Translate these C functions to Rust: ..."

# Default model works (o3 requires paid OpenAI API, not ChatGPT account)
codex exec --full-auto "$(cat prompt.md)"
```

**Key prompt rules to include:**
- `#[no_mangle] pub unsafe extern "system" fn` for all exported functions
- Pool tags: `u32::from_le_bytes(*b"XXXX")` (C multi-char literals are big-endian)
- `NT_ERROR(x)` = `((x as u32) >> 30) == 3`, NOT `x < 0`
- `extern "system"` for FFI (stdcall on i386)
- DUMMY_PSEH: skip `_SEH2_EXCEPT` handlers entirely
- `core::ptr::addr_of_mut!` for KeWaitForSingleObject event pointers
- `IO_STATUS_BLOCK.Information`: use `as _` for cross-arch portability

**Post-integration checklist:**
1. Run `verify_constants.py` on the output to catch wrong constant values
2. Run `verify_ffi.py` to check ABI mismatches and C macro usage
3. Use RAII guards (`PfnLockGuard`, `SpinLockGuard`, `IrqlGuard`) instead of manual lock pairs
4. Use `drop(guard)` at the exact original release point — do NOT let guards drop at function end
5. Prefer `Result<(T, NTSTATUS), NTSTATUS>` returns over output pointers for internal Nt* functions — preserve exact NTSTATUS, don't hardcode STATUS_SUCCESS
6. Use `core::ptr::write_volatile` for ALL writes to output pointer parameters — prevents LLVM captures(none) elimination
7. Run `python3 tools/scan_ud2.py --check build-i386` after building — catches LLVM miscompilation (ud2 in named functions)
8. Run `python3 tools/lint_kernel.py` to find unprotected output pointer writes
9. Fix type casts (`PVOID`, `as _` for arch-dependent sizes)
10. Build all 3 arches + run smoke tests (use `--skip-clean-build` for fast iteration, omit for final verification)

## Developing the pipeline itself (lint / types / tests)

`rustconv` is a maintained Python package, not a scratch script — it carries its
own dev tooling in `tools/rustconv/pyproject.toml`:

- **ruff** (`[tool.ruff]`): lint (`E/F/I/UP/B/SIM/RUF`) + format. Same ruleset as
  the cli and `agent-swarm` so the whole tool surface lints under one standard.
- **pyright** (`[tool.pyright]`, basic mode): run it pointed at the venv
  interpreter so deps resolve — `pyright --pythonpath .venv/bin/python rustconv`.
- **pytest** (`tests/`): pure, no-network unit tests. `test_semdiff_pure` pins the
  semdiff Rust-function/JSON extraction; `test_config_cost` the cost/model math;
  `test_cli_invariants` guards the import hygiene (every submodule imports,
  `__main__` has zero inline imports, and the analysis subcommands that share
  symbol names — `run_oracle`, `extract_file`, `print_summary`, … — stay
  module-qualified so they can't collide on a naive re-hoist).

Run the whole surface (rustconv + cli + `tools/*.py` + agent-swarm) in one shot:

```bash
./vibeos-cli check-tooling            # ruff + ruff format --check + pyright + pytest
./vibeos-cli check-tooling --no-tests # fast lint+types only
```

**Import policy:** hoist imports to module top. The only imports kept inside
functions / guarded by `try/except` are *genuinely optional* third-party deps —
`google.genai` (the `gemini` extra), `PIL`, `shellcheck_py`, and the
`tomllib`→`tomli` 3.10 backport — where a top-level import would break
`pip install` without the extra. That guard is the correct idiom, not a smell.
