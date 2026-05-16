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

5. NTSTATUS values that are `((NTSTATUS)0xC…)` C cast-macros (e.g.
   `STATUS_IO_PRIVILEGE_FAILED`) are not in the bindgen surface by
   design — inline the literal with a trailing `// STATUS_NAME`
   comment (the value is the ground truth).
