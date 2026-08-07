# Translate a Single C Function to Rust (ReactOS ntoskrnl)

You are translating ONE C function from ReactOS ntoskrnl to Rust. Output ONLY the Rust function. No markdown fencing, no explanations.

## Key Rules

**Imports**: Do NOT emit `use` statements. The caller will assemble them. Just use the types/functions directly.

**Naming**: Use bindgen underscore-prefixed struct names (`_OBJECT_HEADER`, `_EPROCESS`). Pointer aliases keep P-prefix (`POBJECT_HEADER`). Enum variants are prefixed: `_OB_OPEN_REASON_ObCreateHandle`.

**Union fields**: Bindgen anonymous unions use `__bindgen_anon_N`. Use `(*header).__bindgen_anon_1.HandleCount`, NOT `(*header).HandleCount` for union members.

**Calling convention**: `NTAPI`/`stdcall` → `extern "system"`. `FASTCALL` → `extern "fastcall"` on x86 / `extern "C"` on 64-bit (use cfg). `CDECL` → `extern "C"`.

**Exported functions** (marked in context): `#[no_mangle] pub unsafe extern "system" fn Name(...)`.
**Static/internal functions**: `unsafe fn Name(...)`.

**DUMMY_PSEH**: `_SEH2_EXCEPT` handlers are dead code. Execute TRY body directly, omit handler.

**Type mappings**: `NULL` → `core::ptr::null_mut()`. `TRUE`/`FALSE` → `1u8`/`0u8` for BOOLEAN. NARROW string literals (`"text"`, for a `PCHAR`/`LPSTR` callee): `b"text\0".as_ptr() as PCHAR`. WIDE literals (`L"text"`, for `PWSTR`/`LPCWSTR`) are a different rule — see "Wide string literals" in the file context; the byte-string form above is ASCII and is WRONG for every wide callee.

**Pointer patterns**: Use `(*ptr).field`, not references. Use `core::ptr::addr_of_mut!((*ptr).field)` only when C code takes address (`&ptr->field`). `RtlCopyMemory` → `core::ptr::copy_nonoverlapping`. `RtlZeroMemory` → `core::ptr::write_bytes(..., 0, ...)`.

**goto**: Use labeled blocks (`'block: { break 'block STATUS_ERROR; }`) or `loop { break; }`.

**Globals**: Reference file-level globals directly (they are declared in the file scope, not in your function).

**Style**: Keep C variable names (PascalCase OK). All functions are `unsafe`. Use `core::` not `std::`. No doc comments. No extra error handling.
