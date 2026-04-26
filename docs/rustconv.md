# rustconv — C-to-Rust Translation Pipeline

API-driven pipeline for translating ReactOS ntoskrnl C files to Rust. Located at `tools/rustconv/`.

## Quick Start

```bash
cd vibeos
source .venv/bin/activate

# Analyze a C file
python -m rustconv scan ntoskrnl/ex/uuid.c

# Translate a single file (Sonnet)
python -m rustconv submit ntoskrnl/ex/uuid.c -o out.rs

# Translate and write directly to crate
python -m rustconv submit ntoskrnl/ex/uuid.c --assemble

# Batch-translate a subsystem
python -m rustconv batch ntoskrnl/ob/ -m sonnet

# Check all 3 targets compile
python -m rustconv check

# Run verification tools
python -m rustconv.verify_constants
python -m rustconv.verify_abi
python -m rustconv.verify_ffi
```

**Note:** Must run from `tools/rustconv/` directory or have it in PYTHONPATH. The `.venv` needs `anthropic` and/or `google-generativeai` packages for API calls.

## CLI Reference

### Translation Pipeline

#### `scan` — Parse C file
```bash
python -m rustconv scan ntoskrnl/ex/uuid.c
```
Uses tree-sitter to parse C source. Prints structural analysis: functions, globals, SEH blocks, architecture ifdefs.

#### `resolve` — Resolve dependencies
```bash
python -m rustconv resolve ntoskrnl/ex/uuid.c
```
Matches C identifiers against the ntkrnl-sys catalog (~9,700 symbols per arch). Outputs an import manifest showing which types/functions are available in bindgen and which are gaps.

#### `prompt` — Generate translation prompt
```bash
python -m rustconv prompt ntoskrnl/ex/uuid.c
python -m rustconv prompt ntoskrnl/ex/uuid.c --estimate  # token count only
```
Assembles the full LLM prompt: system prompt + C source + resolved imports + exports. Prints to stdout for inspection.

#### `submit` — Single-file translation
```bash
python -m rustconv submit ntoskrnl/ex/uuid.c -o out.rs        # write to file
python -m rustconv submit ntoskrnl/ex/uuid.c --assemble        # write to crate
python -m rustconv submit ntoskrnl/ex/uuid.c -m haiku          # use Haiku
python -m rustconv submit ntoskrnl/ex/uuid.c -m gemini-flash   # use Gemini
```
Sends the file to an LLM for translation. With `--assemble`, writes the output directly into the `ntoskrnl-rust/src/{subsystem}/` crate.

#### `batch` — Batch-translate a directory
```bash
python -m rustconv batch ntoskrnl/ob/ -m sonnet
python -m rustconv batch ntoskrnl/ob/ --exclude obhandle.c obref.c
python -m rustconv batch ntoskrnl/ob/ -s ob  # explicit subsystem name
```
Translates all C files in a directory. Supports exclusion lists. Uses batch API for cost efficiency.

#### `translate` — Function-by-function translation
```bash
python -m rustconv translate ntoskrnl/ex/uuid.c -m sonnet
python -m rustconv translate ntoskrnl/ex/uuid.c --fn MyFunction  # single function
python -m rustconv translate ntoskrnl/ex/uuid.c --prompt         # show prompt only
python -m rustconv translate ntoskrnl/ex/uuid.c --dry-run
```
Improved pipeline that translates one function at a time with dependency context. Uses call graph analysis to determine translation order.

#### `estimate` — Token count and cost
```bash
python -m rustconv estimate ntoskrnl/ex/uuid.c      # single file
python -m rustconv estimate ntoskrnl/ob/             # whole subsystem
```

### Post-Processing

#### `pp` — Run postprocessor
```bash
python -m rustconv pp se/token.rs           # single file
python -m rustconv pp se/                    # whole subsystem
python -m rustconv pp se/token.rs --dry-run  # preview changes
```
Applies safe mechanical fixes to LLM-generated Rust: type name underscore-prefixing, import fixes, enum variant name prefixing. Context-free transforms only.

#### `probe-migrate` — Migrate ProbeForRead/Write to Rust probe API
```bash
python -m rustconv probe-migrate se/sqos.rs           # single file
python -m rustconv probe-migrate se/ -m haiku          # batch with Haiku
python -m rustconv probe-migrate se/sqos.rs --scan-only  # show probe sites
python -m rustconv probe-migrate se/sqos.rs --dry-run    # preview
```
Migrates bare `ProbeForRead`/`ProbeForWrite` calls to the Rust-native probe API (`ntkrnl-sys/src/probe.rs`) which returns `Result<T, NTSTATUS>`.

### Verification Tools

#### `verify_constants` — Check constant values
```bash
python -m rustconv.verify_constants                    # check all files
python -m rustconv.verify_constants --fix              # auto-fix mismatches
python -m rustconv.verify_constants path/to/file.rs    # single file
```
Cross-checks Rust `const` values against C `#define` values from kernel headers (4,633 constants). Catches pool tag endianness errors, wrong STATUS codes, wrong flag values. Skips arch-dependent constants.

#### `verify_abi` — Check calling conventions
```bash
python -m rustconv.verify_abi                          # check all files
python -m rustconv.verify_abi path/to/file.rs          # single file
```
Checks hand-written `extern "C"` blocks against ntoskrnl.spec (1,567 exports) and bindgen_i386.rs (2,921 functions). Flags functions that should be `extern "system"` (stdcall) or `extern "fastcall"`.

#### `verify_ffi` — Comprehensive FFI check
```bash
python -m rustconv.verify_ffi                          # check all files
python -m rustconv.verify_ffi path/to/file.rs          # single file
```
Combined checker: ABI mismatches + parameter count mismatches + C macros declared as extern fn + flexible array direct indexing.

### Analysis Tools

#### `check` — Cargo check all targets
```bash
python -m rustconv check
```
Runs `cargo check` for all 3 targets (`aarch64-pc-windows-gnullvm`, `x86_64-pc-windows-gnu`, `i686-pc-windows-gnu`). Reports structured errors.

#### `shims` — Generate C shim files
```bash
python -m rustconv shims ob                  # generate for ob subsystem
python -m rustconv shims ob -o shims.c       # write to file
python -m rustconv shims ob --write          # write to default location
python -m rustconv shims ob --check          # check existing shims are up to date
```
Generates C shim wrappers for FORCEINLINE/macro functions from TOML manifests in `data/shims/`.

#### `seh-classify` — Classify SEH blocks
```bash
python -m rustconv seh-classify ntoskrnl/ob/          # classify directory
python -m rustconv seh-classify ntoskrnl/ob/ -v       # verbose per-function
python -m rustconv seh-classify ntoskrnl/ob/ --json   # JSON output
```
Classifies `_SEH2_TRY`/`_SEH2_EXCEPT` blocks by pattern (probe-guard, resource-cleanup, etc.) for SEH idiomation planning.

#### `objdiff` — Compare C vs Rust objects
```bash
python -m rustconv objdiff                             # compare all
python -m rustconv objdiff ex/                         # single subsystem
python -m rustconv objdiff ex/atom                     # single file
python -m rustconv objdiff --baseline HEAD~1           # compare against commit
python -m rustconv objdiff --critical-only             # only show breaking diffs
python -m rustconv objdiff --deep                      # include section-level detail
```
Compares exported symbols, calling conventions, and parameter counts between C `.obj` files and Rust `.a` archive members.

#### `extern-audit` — Audit extern declarations
```bash
python -m rustconv extern-audit                        # audit all
python -m rustconv extern-audit --json                 # JSON output
python -m rustconv extern-audit --fix                  # auto-fix critical issues
python -m rustconv extern-audit --dry-run              # preview fixes
```
Checks extern block declarations in Rust files against `#[no_mangle]` definitions for convention mismatches.

#### `trace-catalog` — Generate debugcon trace catalog
```bash
python -m rustconv trace-catalog                       # default arch
python -m rustconv trace-catalog --arch i386           # specific arch
```
Scans Rust kernel source for inline asm `out 0xe9, al` trace points and generates JSON catalogs (`data/trace_catalog_{arch}.json`) for debugcon boot failure triage.

## Model Shortcuts

| Shortcut | Model ID |
|----------|----------|
| `sonnet` | `claude-sonnet-4-5-20250929` |
| `haiku` | `claude-haiku-4-5-20251001` |
| `opus` | `claude-opus-4-6-20250908` |
| `gemini-pro` | `gemini-2.5-pro` |
| `gemini-flash` | `gemini-2.5-flash` |

Or pass a full model ID directly: `-m claude-sonnet-4-5-20250929`

## Pipeline Architecture

```
C source → [scanner] tree-sitter parse
         → [resolver] match vs ntkrnl-sys catalog
         → [prompt_builder] system prompt + C + imports + exports
         → [batch_runner] LLM API (Anthropic / Gemini)
         → [postprocess] safe mechanical fixes
         → [assembler] write crate
         → [checker] cargo check ×3
```

### Multi-Stage Pipeline

```
Stage 1: C → Rust        (submit/batch)       — bulk transliteration
Stage 2: Probe migration (probe-migrate)      — ProbeForRead/Write → Result<T>
Stage 3: SEH idiomation  (planned)            — full SEH → Result/RAII/match
```

## Key Files

```
tools/rustconv/
├── rustconv/
│   ├── scanner.py          # tree-sitter C parser
│   ├── catalog.py          # ntkrnl-sys symbol index (~9,700 per arch)
│   ├── resolver.py         # identifier → import matching
│   ├── spec_parser.py      # ntoskrnl.spec parser (1,592 exports)
│   ├── prompt_builder.py   # system + per-file prompt assembly
│   ├── batch_runner.py     # Anthropic + Gemini API client
│   ├── postprocess.py      # safe mechanical fixes
│   ├── assembler.py        # write to ntoskrnl-rust/src/{subsys}/
│   ├── checker.py          # cargo check ×3 + structured errors
│   ├── callgraph.py        # call graph extraction for function-level translation
│   ├── function_prompt.py  # per-function prompt builder
│   ├── function_resolver.py # per-function identifier resolution
│   ├── stitcher.py         # assemble individual functions into .rs file
│   ├── probe_migrate.py    # ProbeForRead/Write → probe API migration
│   ├── seh_classifier.py   # SEH block pattern classification
│   ├── shim_generator.py   # C shim file generator from TOML
│   ├── objdiff.py          # C vs Rust object file comparison
│   ├── extern_audit.py     # extern declaration auditor
│   ├── trace_catalog.py    # debugcon trace catalog generator
│   ├── verify_constants.py # constant value cross-checker
│   ├── verify_abi.py       # calling convention checker
│   └── verify_ffi.py       # comprehensive FFI checker
├── templates/
│   ├── system_prompt.md    # translation rules (~5K tokens, prompt-cached)
│   ├── function_prompt.md  # per-function translation template
│   └── probe_migrate_prompt.md  # probe migration rules
└── data/
    ├── gap_functions.toml  # FORCEINLINE/macro functions not in bindgen
    ├── shims/              # TOML shim manifests per subsystem
    ├── trace_catalog_i386.json   # debugcon trace points
    └── trace_catalog_amd64.json
```

## Cost (Sonnet Batch API)

| Scope | Est. Cost |
|-------|-----------|
| Single file | ~$0.03 |
| Subsystem (11 files) | ~$0.30 |
| Full ntoskrnl (393 files) | ~$8.70 |
| 5 iterations | ~$43.50 |
