# User-mode subsystem rules (SMSS / CSRSS / BASESRV)

These targets are **native user-mode processes** (smss.exe, csrss.exe,
csrsrv.dll, basesrv.dll), NOT kernel-mode. Override the kernel-oriented
guidance above where it conflicts:

1. **FFI substrate is `ntdll_sys::nt`, never `ntkrnl_sys`.** Import
   everything with a single `use ntdll_sys::nt::*;`. Do not emit
   `ntkrnl_sys::ddk` / `::base` / `::constants` / `::globals` paths.

2. **`NtCurrentPeb`, `NtCurrentTeb`, `NtCurrentProcess`,
   `NtCurrentThread`, `ZwCurrentProcess`, `ZwCurrentThread` are
   provided by `ntdll_sys::nt`** (hand-written inlines). They are
   FORCEINLINE macros in C — they are NOT ntdll exports.
   - DO NOT redeclare them in an `extern "system" { … }` /
     `extern "C" { … }` block. That fails to link.
   - Just call them; they come in via `use ntdll_sys::nt::*;`.

3. **Never invent a local `struct PEB`/`struct TEB`** with a hardcoded
   `_padding: [u8; 0x…]` offset. The real layouts are
   `ntdll_sys::nt::_PEB` / `_TEB`. `NtCurrentPeb()` returns
   `PPEB` (`*mut _PEB`); access fields directly, e.g.
   `(*NtCurrentPeb()).SessionId`. `NtCurrentTeb()` returns `*mut _TEB`.

4. The native entrypoint is `_main` (C: `_cdecl _main`). Translate it
   as `#[no_mangle] pub unsafe extern "C" fn _main(...)` exactly —
   the native CRT stub links to that symbol.

5. **`STATUS_*` codes ARE available** as named consts from
   `ntdll_sys::nt` (hand-written `constants.rs`: `STATUS_SUCCESS`,
   `STATUS_UNSUCCESSFUL`, `STATUS_NO_MEMORY`,
   `STATUS_INSUFFICIENT_RESOURCES`, `STATUS_INVALID_PARAMETER`,
   `STATUS_BUFFER_TOO_SMALL`, `STATUS_OBJECT_NAME_NOT_FOUND`, … 129
   of them). USE THE NAMED CONST (`return STATUS_NO_MEMORY;`). Only
   if a specific code is genuinely absent, inline the `0x… as i32`
   literal with a trailing `// STATUS_NAME` comment.

6. **The SM/CSR/BASE protocol structs are bindgen anonymous-union/
   struct nests — you MUST use the real bindgen field path, never
   the flat C member name.** The C source writes flat names because
   C anonymous members are transparent; bindgen is not. For
   `CSR_API_MESSAGE` / `_CSR_API_MESSAGE` the chain is:
   - `msg.Header`                 → `msg.Header` (a `PORT_MESSAGE`)
   - `msg.ConnectionInfo`         → `msg.__bindgen_anon_1.ConnectionInfo`
   - `msg.CsrCaptureData`         → `msg.__bindgen_anon_1.__bindgen_anon_1.CsrCaptureData`
   - `msg.ApiNumber`              → `msg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber`
   - `msg.Status`                 → `msg.__bindgen_anon_1.__bindgen_anon_1.Status`
   - `msg.Data.<Api>`             → `msg.__bindgen_anon_1.__bindgen_anon_1.Data.<Api>`
   For `PORT_MESSAGE` / `_PORT_MESSAGE`:
   - `pm.ClientId`                → `pm.__bindgen_anon_1.ClientId`
   - `pm.u1.s1.DataLength` / `TotalLength` stay as-is, but they are
     **`i16` (CSHORT)** — cast with `as i16`, never `as u16`.
   For `LARGE_INTEGER` / `_LARGE_INTEGER` (a union): use
   `li.QuadPart` **directly** (it is a top-level union member). Do
   NOT write `li.__bindgen_anon_1.QuadPart` — `__bindgen_anon_1` is
   the `{LowPart,HighPart}` struct and has no `QuadPart`.
   When in doubt about an SM_*_MSG / BASESRV_API_MSG nest, follow the
   same rule: descend through `__bindgen_anon_N` to reach the named
   leaf field.

7. **`RtlZeroMemory`/`RtlFillMemory` are C macros, not exports.**
   Use `core::ptr::write_bytes(p as *mut u8, 0, n as usize)` for
   `RtlZeroMemory(p, n)`. `RtlMoveMemory`/`RtlCopyMemory` ARE real
   ntdll exports — those may be called directly.

8. **File-scope C globals stay in the file.** A `HANDLE
   CsrPortHeap;` (no `extern`) at C file scope is a *definition* —
   emit `pub static mut CsrPortHeap: HANDLE = core::ptr::null_mut();`
   in the module. Only an `extern HANDLE CsrPortHeap;` is an import.
   Never reference a file-local global as if it came from
   `ntdll_sys::nt`.

9. **Wide-string literals**: C `L"\\Foo"` is `*const u16`. Emit a
   `static` `[u16; N]` (UTF-16 code units, NUL-terminated) and pass
   `.as_ptr()`, or build the `UNICODE_STRING` explicitly. Never pass
   a Rust byte string `b"…"` where `*const u16` is expected.
