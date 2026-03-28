# rustconv — C-to-Rust Translation Pipeline

## Overview
`tools/rustconv/` — API-driven pipeline translating ntoskrnl C files to Rust via LLM.
393 C files (~283K lines). Sonnet batch API at ~$8.70 for full kernel.

## CLI
```bash
python -m rustconv scan|resolve|prompt|estimate|submit|batch|check
```
Model shortcuts: `sonnet`, `haiku`, `opus`, `gemini-pro`, `gemini-flash`

## Pipeline
scanner.py (tree-sitter) → catalog.py (ntkrnl-sys index) → resolver.py (deps) →
prompt_builder.py (system+file prompt) → batch_runner.py (API) → postprocess.py →
assembler.py (write crate) → checker.py (cargo check ×3)

## Post-Processor (postprocess.py) — SAFE transforms only
- Type name underscore-prefixing (OBJECT_HEADER → _OBJECT_HEADER)
- Import fixes (ddk::* warning, pool type dedup)
- Enum variant prefixing (ObCreateHandle → _OB_OPEN_REASON_ObCreateHandle)
- **NOT** union field access — fundamentally unsafe with context-free regex

## System Prompt Key Patterns (templates/system_prompt.md)
Teaches the LLM to generate correct code directly. Key sections:
- **Imports**: NEVER use `ddk::*` (conflicts with base::*). Selective imports only.
- **Union field access**: `__bindgen_anon_N` mapping per struct (OBJECT_HEADER, HANDLE_TABLE_ENTRY, etc.)
- **EPROCESS/ETHREAD**: Cast `PsGetCurrentProcess()` to `*mut _EPROCESS` for field access
- **Calling conventions**: `extern "system"` for callbacks (maps to stdcall on i386)
- **Function pointers**: Wrap in `Some()` for Option<unsafe extern fn> fields
- **Raw pointers**: `core::ptr::addr_of_mut!()` for struct field pointers
- **LONG_PTR fields**: InterlockedIncrementSizeT/DecrementSizeT
- **ObAttributes**: Portable ULONG_PTR casting (u32 on i386, u64 on 64-bit)

## Lessons Learned (obhandle.c translation)

### Errors by category (initial Sonnet output → 521 errors)
1. Missing ntkrnl-sys declarations → added constants.rs, macros.rs, interlocked.rs
2. `ddk::*` glob conflicts with `base::*` → selective ddk imports in system prompt
3. Union field access wrong → system prompt teaches `__bindgen_anon_N` per struct
4. EPROCESS opaque type (ps::PEPROCESS) vs full struct (ddk::_EPROCESS) → system prompt
5. Calling convention: `extern "C"` → `extern "system"` for callbacks → system prompt
6. i386 type width: `u64` hardcoded where ULONG_PTR needed → system prompt
7. PsInitialSystemProcess: PVOID needs casting for PEPROCESS comparison → system prompt

### Context-free regex is dangerous for field access
Same field name exists on multiple structs with different layouts:
- HandleCount: _OBJECT_HEADER (in union), _HANDLE_TABLE (direct), _OBJECT_HANDLE_COUNT_ENTRY (direct)
- GrantedAccess: _HANDLE_TABLE_ENTRY (in union), _OBJECT_HANDLE_INFORMATION (direct)
- ObjectCreateInfo: _OBJECT_HEADER (in union), _OB_TEMP_BUFFER (direct)
Regex can't distinguish struct types → over-applies transforms → MORE errors.
Solution: system prompt teaches LLM correct accessors, post-processor stays safe.

### Scorecard
- **Round 1** (before prompt updates): 521 errors → 0 on ARM64/amd64, 26 on i386
- **Round 2** (after prompt+postprocessor updates): 142 errors → 0 on ALL 3 arches
- Round 2 needed: ~30 fixup-script fixes (imports, casts, mutability), callback cfg wrappers, ObAttrType alias
- Remaining issues the prompt still doesn't fully prevent:
  - `extern "C"` vs `extern "stdcall"` callbacks on i386 (needs cfg wrapper pattern)
  - `base::ULONG_PTR` (usize) vs `ddk::ULONG_PTR` (c_ulonglong/c_ulong) type mismatch
  - Missing ddk imports for Ob*/Se*/Ps* helper functions from other files
  - PsGetCurrentProcess() → needs cast to *mut _EPROCESS for field access

## ntkrnl-sys Additions for rustconv
- `src/constants.rs`: OBJ_*, OB_*, STATUS_* constants used by ob/ subsystem
- `src/macros.rs`: PsProcessType, ObpSymbolicLinkObjectType, KdDebuggerEnabled
- `src/interlocked.rs`: Added InterlockedIncrementSizeT, InterlockedDecrementSizeT

## Next Steps
1. Batch-translate remaining ob/ subsystem (11 files) → iterate on prompt
2. Scale to full ntoskrnl (393 files) in subsystem batches
3. Build integration (link Rust crate into ntoskrnl, replace C files)
