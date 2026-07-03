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

## Stage 3 — Result + Drop-RAII de-transliteration

Stage 1 output is a faithful C *transliteration*: the borrow checker, type system, and ownership model are essentially unused (the fattest subsystems ke/mm/ps/ob/ex score ~2–3/10 idiomatic across ~447K LOC — a few hundred `Option`/`Result`, thousands of whole-function `unsafe extern` blocks, ~5000 1:1 `#[no_mangle]` symbols). Stage 3 de-transliterates the *normal* path into idiomatic Rust while smoke stays green throughout.

**Stage 3 is NOT "SEH idiomation."** `USE_DUMMY_PSEH` makes `_SEH2_EXCEPT` handlers dead code on all arches and the kernel is `panic=abort` with no unwinding — there is no exception control flow to idiomate. Drop-as-`finally` is sound *precisely because* of this: every scope exit is an explicit `return`, so guards fire deterministically.

**The seven moves (incremental, per-function, smoke-green throughout):** (1) magic values → `enum`/newtype/`bitflags!`, run every new constant through `verify_constants.py`; (2) remove C control flow (labeled-`loop`-as-goto, goto-cleanup cascades); (3) break up 300–600-line files — safe *within* `ntoskrnl-rust` (`#[no_mangle]` is flat namespace); spinning a fn into a *new rlib* triggers the `lto=false`/`cgu=256` firewall — don't do casually; (4) tighten `unsafe` scope; (5) introduce `<snake_name>_rs(...) -> Result<T, NTSTATUS>` cores (suffix `_rs`, don't collide with the existing `_rs_<CName>` inline-shim *prefix*); (6) collapse the C `#[no_mangle] extern "system"` symbol into a thin trampoline over the `_rs` core; (7) new structures (locks, rundown, refs) implement `Drop` → `Drop` **is** the `finally`.

**Trampoline-template rule (load-bearing):** every out-param write in the trampoline MUST be `core::ptr::write_volatile`. A naive `*out = v` reintroduces the LLVM `captures(none)`/`ud2` miscompile the whole kernel-Rust surface is defended against. Keep the `extern "system"` ABI on the exported symbol (never `extern "C"`). Prefer `Result<(T, NTSTATUS), NTSTATUS>` over hardcoding `STATUS_SUCCESS`. Drop guards at the *exact original release point* — don't let a guard drift to function end.

**Frozen pilot / template:** `ob::ObReferenceProcessHandleTable` — a self-contained, survey-flagged leak rebuilt as `RundownGuard: Drop` + an `_rs` core + a thin trampoline, validated with smoke ×3 + `scan_ud2` + `verify_abi`, **frozen as the canonical template**. Copy its shape.

**Bulk ordering:** `ob` first (most self-contained; pilot lives here); `ke` last/heaviest (~55K LOC); algorithm sibling rlibs (`slist`/`avltable`/`splaytree`) are **already idiomatic** — leave them alone. The stage operates function-by-function (never file-by-file) with paired C+Rust context.

## c2rust behavior-anchor path (`translate --c2rust`)

`--c2rust` anchors each function's translation on its **c2rust transpilation** — a compiler-exact, behavior-preserving Rust body — injected into the prompt as a `## Behavior Anchor`. This stops the mechanical LLM pass from silently *stubbing* hard functions or turning function-like macros into phantom externs (an anchor-less pass emitted only 7 of 13 functions in a driver file, dropping 4 exports, invisible until link). The anchor preserves the **algorithm layer** (deref chains, IRQL checks, exact `DbgPrint` format strings, `wrapping_add`, GF/crypto constants); the LLM still introduces shell/constant bugs *around* it, so the Claude review tier stays **mandatory**.

Pieces: `tools/rustconv/rustconv/c2rust_anchor.py` (`build_anchor_map`, `build_preamble`); the `## Behavior Anchor` rule in `templates/function_prompt_mechanical.md`; the `--c2rust` flag. Best-effort — no binary / c2rust panic → empty map, proceeds anchor-less. When c2rust hits an un-lowerable AST construct, `--c2rust` **hard-fails** rather than degrading (fix the C side first).

**Installing c2rust (the LLVM footgun):** `cargo install c2rust` needs **`llvm-config` on `PATH`** (not just `LLVM_LIB_DIR`) — `build.rs` calls it for `--version`/`--libs`. `libclang-*-dev` alone is insufficient:
```bash
sudo apt-get install -y llvm-14-dev              # provides /usr/bin/llvm-config-14
ln -sf /usr/bin/llvm-config-14 ~/.local/bin/llvm-config
cargo install c2rust --version 0.22.1
```
**The transpile invocation** (two dead ends: pointing at `compile_commands.json` treats it as a dir → flag-less → `ntdef.h not found`; a one-entry db dir → `expected exactly one compiler job`). ✅ **Direct-source form:** `c2rust transpile --output-dir DIR SOURCE.c -- <FLAGS>` where `<FLAGS>` = `--target`/`-D`/`-I`/`-isystem`/`-include` from that file's `compile_commands.json` entry, **dropping `-o`/`-c`/`-M*`**. Output at `DIR/src/<stem>.rs`; c2rust may warn/panic but **still writes the `.rs`** — check for the output *file*, not the exit code. Note: **i386's cmake has `CMAKE_EXPORT_COMPILE_COMMANDS` OFF** (no db); already-translated files are `REMOVE_ITEM`'d so absent from the db — anchors only work for **still-C** targets (which is exactly where they're needed).

**Preamble (file-scope data) comes from c2rust, not a second LLM call.** Take ONLY `static*` and aggregate `const` (contains `[`/`{`); EXCLUDE scalar `const NAME: _ENUM = 5;` (c2rust floods header enum-variants that collide with bindgen). The path prepends these exact tables and **dedups** — dropping any LLM-emitted `static`/`const` of the same name (the LLM copy is fabrication-prone). Verify the agent actually spliced the big tables (a silent AES-S-box drop is invisible until `cargo check`).

## Shrinking giant C files: the `#include`-back split

rustconv's giant-file problem is a *batch* problem, not a fidelity one (`translate` already chunks function-by-function), so physical splitting is only to keep the *output* `.rs` sane. When you split, use **`#include`-back, never real TU-splitting.** Keep `<base>.c` as the **single** TU: it retains the prologue + first function group and appends `#include "<base>.split2.c" …`; fragments hold the rest **verbatim**. The preprocessor reassembles the identical TU → **zero CMake change, zero linkage/static/global hazard.** Safety guards: (1) per-file byte-exact reconstruction assertion (`base[:cut] + fragments == original`); (2) confirm the source list is explicit `list(APPEND SOURCE …)`, **not** `file(GLOB)` (a glob double-compiles fragments); (3) `nm --defined-only` identical pre/post; (4) smoke gate all arches. **Splitting invalidates per-file sidecar analysis** (Pass-0/2/3 `*.json`) — wipe all sidecar globs and regenerate (deterministic, no API cost).

## LLM mechanical transforms as a "remainder closer"

When a bulk mechanical C→C (or Rust→Rust) normalization is deterministic for *most* sites but a minority need a local judgement call, build the LLM stage as the **remainder closer** with the deterministic tool's output shapes as the contract — not a from-scratch rewriter. (This closed a 64-site win32k SEH-lowering ledger the deterministic `seh_rewrite.py` correctly refused, ×3 green.) The shape: (1) unit = enclosing function, splice back by char offset highest-first; (2) contract = the deterministic tool's exact output shapes so LLM output is byte-consistent; (3) `validate()` before any splice (brace/paren balance, required-symbol present, no stub markers, **token-superset**); (4) single cached API pass, reruns/`--apply` are $0; (5) build + smoke is the real gate — "0 units held back" is *yellow*, not green. **The bug only the build caught:** an LLM stage that emits calls into a shim must own the same include/declaration plumbing the deterministic stage does (a missing `#include` injection failed files the mechanical pass never touched).

## tree-sitter for C structural transforms: differential oracle, never `has_error`

tree-sitter's C grammar is **lenient, not a preprocessor** — real files parse "clean" yet emit `ERROR`/missing nodes because unresolved macros (`NTAPI`, SAL, `#if` arms) become ERROR subtrees, so `root_node.has_error` is **effectively always true**. Use a **differential oracle:** parse original + transformed, require `errs(new) <= errs(orig)` (no *new* errors) **plus** a domain invariant (e.g. `switch` `case` labels unchanged). The real gate stays build + `nm` + smoke. tree-sitter IS right for plain-C structural transforms (a `switch` in a probe-lowered fn is plain C — `switch_statement`→`case_statement`); the "can't parse macros" lore is specific to `_SEH2_TRY/_EXCEPT`. tree-sitter offsets are **byte** offsets — splice in `bytes` or a non-ASCII char desyncs everything after. Conservative per-unit proof beats blanket extraction (extract a switch arm only if it ends in `return`, no `break`/`goto`/`_SEH2_`, idents ⊆ params ∪ file-scope ∪ arm-local).

## Resume cache and the `--no-resume` model-upgrade footgun

`translate` persists each function's Rust to `tools/rustconv/.responses/<subsystem>__<stem>/<func>.rs` the instant it returns, with `manifest.json` recording a sha256 of the C body. Next run, every function whose **current C-body hash matches** is reused for **$0** (`CACHED ($0)`); only new/changed functions hit the API. A killed run costs at most the single in-flight function. **Footgun — the cache keys on `(subsystem, stem, fn-body-hash)`, NOT the model.** Re-running `translate --fn X -m opus` to *upgrade* a function first translated by a cheaper model returns the **cached worse output for $0.00** — no Opus call happens, silently. You **must** pass `--no-resume` to force a fresh call with the stronger model (applies to re-translating review-flagged stubs, and the c2rust-anchor path re-translating an already-cached file).

## DeepSeek mechanical-mode failure modes (detector/auto-fix backlog)

DeepSeek (`--mode mechanical`) is the cheap gruntwork tier but markedly weaker than Claude. Recurring failure modes found grinding driver crates — each a candidate auto-fix/detector: (1) **leaves raw C `goto`/labels** in complex fns → wrap body in a `'cleanup: { ... }` block, forward `goto Exit`→`break 'cleanup`; detector: grep live `goto |^\s*\w+:\s*$`. (2) **`expr as usize < size_of::<T>()` without parens** → rustc mis-parses `<` as generics; `cargo fix --broken-code --allow-dirty` auto-applies the paren suggestion (run *before* grinding). (3) **adds `use ...::*` globs** → E0659 cascades (+700 phantom errors in one turn); rail: NEVER glob, use explicit `use ntkrnl_sys::ddk::{Name};`. (4) **stubs hard functions to no-ops** (`{ let _ = a; }`) — flag in review. (5) **wrong module path for driver-private structs** (`ntkrnl_sys::ddk::FDE` when it's `crate::types`) — bulk-sed. (6) **strips *correct* `as i32` casts** on C-shaped shims → E0308 flood; feed callee signatures into the prompt. (7) **drops bit-masking helper calls** (`SRB_STATUS(x)==S` → `x==S`) — compiler can't catch; detector: call-superset (every `NAME(` in input reappears in output). (8) **emits C ternary `?:` and stream-of-consciousness `//` comments** — reject output containing `?...:`. **General:** the grinder's self-reported error counts are routinely exaggerated — trust your own `cargo check`. Re-running repair on already-regressed code does NOT self-heal a prior semantic drop — run deterministic passes first, repair on a **clean** baseline.
