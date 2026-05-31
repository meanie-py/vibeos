# winsrv subsystem rules (winsrv.dll — USER + Console server)

This target is the **winsrv.dll CSR server DLL** (the consrv + usersrv +
concfg static libs aggregated into one DLL, loaded by csrss alongside
basesrv). NOT a standalone process. Override the kernel-oriented
guidance above where it conflicts:

1. **FFI substrate is `winsrv_sys` — ONE crate, ONE glob.** Always:
   ```
   use winsrv_sys::winsrv::*;
   ```
   `winsrv_sys::winsrv::*` is a SUPERSET of ntdll_sys: it covers the
   user-mode Native API + SM/CSR/BASE surface (`Nt*`/`Zw*`/`Rtl*`/
   `Ldr*`/`Dbg*`/`Csr*`/`Sm*`/`Base*`), the Win32 surface winsrv
   consumes (`Reg*`/`Get*`/`Set*`/registry/console/nls), AND the
   winsrv-internal types (`CONSOLE_PROCESS_DATA`, `TERMINAL_VTBL`,
   `FRONTEND_VTBL`, `SHUTDOWN_SETTINGS`, `CONSOLE_STATE_INFO`,
   `TT_FONT_ENTRY`, `CONSOLE_*`, `COORD`, `INPUT_RECORD`, `HWND`,
   `HDC`, `LOGFONTW`, `WNDCLASSEXW`, `MSG`, `WNDPROC`, …).
   Do not emit `ntkrnl_sys::ddk` / `::base` / `::constants` paths.
   Do not emit raw `ntdll_sys::*` paths.
   **NEVER add `use winsrv_sys::nt::*;` as a second glob.** It
   re-exports hand-written `NtCurrentTeb`/`NtCurrentProcess`/
   `STATUS_*` etc. — pull those with a *targeted*
   `use winsrv_sys::nt::{NtCurrentTeb, STATUS_SUCCESS, …};` import,
   never a glob. A second glob makes HKEY/HANDLE/LPWSTR/MAX_PATH
   E0659-ambiguous against the winsrv-side definitions.

2. **Entry point is `DllMain` and the spec callbacks**:
   `ConServerDllInitialization`, `UserServerDllInitialization`,
   `_UserSoundSentry`. NOT `_main`. winsrv.spec exports them as
   `stdcall`. Translate them as
   `#[no_mangle] pub unsafe extern "system" fn …`.

3. **`NtCurrentPeb`, `NtCurrentTeb`, `NtCurrentProcess`,
   `NtCurrentThread`, `ZwCurrentProcess`, `ZwCurrentThread` are
   provided by `winsrv_sys::nt`** (re-exported from ntdll-sys's
   hand-written inlines). FORCEINLINE macros in C — NOT ntdll exports.
   - DO NOT redeclare them in an `extern "system" { … }` block.
   - Just call them; they come in via `use winsrv_sys::nt::*;`.

4. **Never invent a local `struct PEB`/`struct TEB`** with hardcoded
   `_padding: [u8; 0x…]` offsets. The real layouts are
   `winsrv_sys::nt::_PEB` / `_TEB`.

5. **`STATUS_*` codes ARE available** as named consts from
   `winsrv_sys::nt` (re-exported from ntdll-sys's hand-written
   `constants.rs`: `STATUS_SUCCESS`, `STATUS_UNSUCCESSFUL`,
   `STATUS_NO_MEMORY`, `STATUS_INVALID_PARAMETER`,
   `STATUS_BUFFER_TOO_SMALL`, `STATUS_OBJECT_NAME_NOT_FOUND`, … 129 of
   them). USE THE NAMED CONST. Only inline a `0x… as i32` literal with
   a trailing `// STATUS_NAME` comment if a specific code is genuinely
   absent.

6. **CSR_API_MESSAGE / PORT_MESSAGE / SM_*_MSG / BASESRV_API_MSG /
   CONSOLE_API_MESSAGE are bindgen anonymous-union/struct nests.**
   Use the real bindgen field path, never the flat C member name.
   For `CSR_API_MESSAGE` / `_CSR_API_MESSAGE`:
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
   For `LARGE_INTEGER` / `_LARGE_INTEGER` (a union): use `li.QuadPart`
   **directly**. Do NOT write `li.__bindgen_anon_1.QuadPart`.
   For `CONSOLE_PROCESS_DATA`/`CONSOLE_INFO`/the input-record union,
   descend through `__bindgen_anon_N` to reach the named leaf field.

7. **`ConsoleGetPerProcessData(P)` is a macro**, not an extern:
   `((PCONSOLE_PROCESS_DATA)((P)->ServerData[CONSRV_SERVERDLL_INDEX]))`.
   Inline it; do NOT declare an extern. Same for `CsrGetClientThread`
   (FORCEINLINE in csrsrv.h) and the `CON_API_*` macros from
   `consrv/api.h:20-64` that auto-fetch the per-process data — expand
   each call site.

8. **C macros are NOT extern functions — never `extern "system" { fn
   <macro>; }`.** That fails to link.
   - `RtlZeroMemory(p, n)` → `core::ptr::write_bytes(p as *mut u8,
     0, n as usize)`. `RtlFillMemory(p,n,v)` similarly.
   - `CONTAINING_RECORD(addr, Type, field)` →
     `((addr as usize) - core::mem::offset_of!(Type, field)) as *mut Type`
     (cast back to the pointer type the call site expects).
   - `InitializeListHead`/`IsListEmpty`/`InsertHeadList`/`InsertTailList`/
     `RemoveEntryList` come in via `winsrv_sys::nt::*` (hand-written
     inlines in ntdll-sys). Call them; do NOT redeclare.
   - `RtlMoveMemory`/`RtlCopyMemory` ARE real ntdll exports — call
     directly. When unsure whether a `Foo` is a macro: if it is
     ALL-CAPS or wraps a struct/offset computation, treat it as a
     macro and inline it; do not declare an extern.

9. **File-scope C globals stay in the file.** A `HANDLE
   ConSrvDllInstance;` (no `extern`) at C file scope is a *definition* —
   emit `#[no_mangle] pub static mut ConSrvDllInstance: HANDLE =
   core::ptr::null_mut();`. Only an `extern HANDLE ConSrvDllInstance;`
   is an import (`extern "C" { pub static mut ConSrvDllInstance:
   HANDLE; }`). Never reference a file-local global as if it came from
   `winsrv_sys::winsrv`.

10. **Wide-string literals**: C `L"\\Foo"` is `*const u16`. Emit a
    `static` `[u16; N]` (UTF-16 code units, NUL-terminated) and pass
    `.as_ptr()`, or build the `UNICODE_STRING` explicitly. Never pass
    a Rust byte string `b"…"` where `*const u16` is expected.

11. **Per-binary cohabitation** mirrors the smss-csrss pattern. The
    Cargo feature gate is `winsrv_dll_only` (reserved in
    winsrv-rust/Cargo.toml). `#[no_mangle]` on DllMain and the spec
    initializers MUST be `#[cfg_attr(feature = "winsrv_dll_only",
    no_mangle)]` so the symbol only collides with the cohabited C when
    the lean PE-flip feature is enabled.

12. **TERMINAL_VTBL / FRONTEND_VTBL pattern** — use the `#[repr(C)]`
    fn-pointer struct directly (the bindgen layout). Do NOT introduce
    a Rust trait or `dyn` until idiomation Stage 3; the cohab needs
    bit-exact layout parity with the C struct that other TUs reference.

13. **arm64-specific code paths** — keep `#[cfg(target_arch =
    "aarch64")]` shape. `arm64_delay_stubs.c` provides
    delay-load thunks for shell32/ole32 that are skipped on arm64
    (consrv.cmake sets `SERIALTERM_COMPILE` only on arm64).
