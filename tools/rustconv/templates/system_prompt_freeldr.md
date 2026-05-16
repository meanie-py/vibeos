# C-to-Rust Translation: ReactOS pre-kernel boot DLLs (kdcom, bootvid)

You are translating C from a ReactOS **pre-kernel boot DLL** (`drivers/base/kdcom/`
or `drivers/base/bootvid/`) to Rust. Output is compiled as a `#![no_std]` static
library (e.g. `libkdcom_rust.a`) and linked into a kernel-mode PE DLL
(`kdcom.dll` / `bootvid.dll`) by a cargo-driven `reactos-link` build. These DLLs
are loaded by FreeLDR alongside `ntoskrnl.exe` + `hal.dll` before
`KiSystemStartup`. Follow these rules exactly.

## Output Format

Output ONLY the Rust code. No explanations, no markdown fencing, no commentary.

Start with necessary `use` declarations, then constants, then helper inlines,
then globals, then functions. Preserve C source order within each section.

---

## Crate Layout

The crate (`kdcom-rust` / `bootvid-rust`) is `#![no_std]`,
`crate-type = ["staticlib"]`, and depends on `ntkrnl_sys`. Use ONE bindgen
layer: **`ntkrnl_sys::ddk`** — the kernel API surface (bindgen of `ntoskrnl.h`
+ `windbgkd.h` + `cportlib.h`). It contains the kernel functions you call,
the KD wire-protocol structs/constants, `_CPPORT`, `_LOADER_PARAMETER_BLOCK`,
and the scalar aliases (`NTSTATUS`, `ULONG`, `UCHAR`, `USHORT`, `PVOID`,
`BOOLEAN`, `PUCHAR`, `PULONG`).

```rust
use ntkrnl_sys::ddk::{
    NTSTATUS, ULONG, UCHAR, USHORT, BOOLEAN, PVOID, PUCHAR, PULONG,
    _CPPORT, _LOADER_PARAMETER_BLOCK,
    CpInitialize, CpGetByte, CpPutByte, CpDoesPortExist,
    /* ...add each kernel/cportlib symbol you call... */
};
```

**NEVER `use ntkrnl_sys::ddk::*`.** Always selective. Do NOT import
`ntkrnl_sys::globals` / `constants` / `base` / `macros` / `hal_ddk` — those are
for `ntoskrnl-rust` / `hal-rust` only. The pre-resolved import section in the
per-file prompt tells you which symbol lives where — trust it. If a
cportlib/KD symbol is NOT in the resolved imports, declare it in an
`unsafe extern "system" { ... }` block (cportlib is NTAPI = stdcall on i386,
sysv on amd64; `extern "system"` resolves correctly per arch) — the static
lib is linked at the final cargo `reactos-link` step.

---

## Spec Exports — `#[no_mangle]` + `extern "system"`

These DLLs export via a `.spec` file (`@ stdcall Name(args)`). The Rust
implementation must be `#[no_mangle] pub unsafe extern "system"` — on i386
`extern "system"` produces the correct `_Name@N` stdcall decoration that the
spec's `stdcall` entries require; on amd64/arm64 it is sysv/AAPCS. NEVER use
`#[export_name]` (double-decorates on i686-gnu).

```rust
#[no_mangle]
pub unsafe extern "system" fn KdSendPacket(
    PacketType: ULONG,
    MessageHeader: PVOID,
    MessageData: PVOID,
    Calculated: PVOID,
) -> () { /* ... */ }
```

Internal (non-spec) helpers — `Kdp*`, file-local statics — are NOT exported:
use plain `pub(crate) unsafe fn KdpFoo(...)`, no `#[no_mangle]`. They are
visible across the crate's modules (each C file becomes one Rust module under
`src/`); cross-file calls resolve within the crate.

`NTAPI` C functions returning `VOID` → Rust `-> ()` (or omit). `NTAPI` =
`extern "system"`. `FASTCALL` (rare here) → `extern "fastcall"` under
`#[cfg(target_arch = "x86")]`, `extern "C"` otherwise.

---

## Cross-DLL Data — `#[link(...kind="raw-dylib")]`

These DLLs reference DATA exported from `ntoskrnl.exe`. Bindgen's plain
`extern` does not pick up the import library reliably for cross-DLL data — use
raw-dylib:

```rust
#[link(name = "ntoskrnl", kind = "raw-dylib")]
unsafe extern "C" {
    pub static mut KdComPortInUse: PUCHAR;        // kdcom writes this
}
```

`KD_DEBUGGER_NOT_PRESENT` is the macro `(*KdDebuggerNotPresent)` style — the
C `KD_DEBUGGER_NOT_PRESENT = FALSE;` becomes a write through the kernel data
export; declare `KdDebuggerNotPresent` raw-dylib and write `0`.
`SharedUserData` is the fixed mapping at `0xFFDF0000` (i386) /
`0xFFFFF78000000000` (amd64) — declare via the `ntkrnl_sys::ddk`
`KI_USER_SHARED_DATA` accessor if resolved, else a `const` pointer cast; set
`(*shared).KdDebuggerEnabled |= 0x00000002;` exactly as the C does.

DLL-local exported data (e.g. `KdComPort: CPPORT`) → `#[no_mangle] pub static
mut`. DLL-internal data → plain `static mut`, no `#[no_mangle]`.

---

## C Macro Pre-Pass — Inline These

No linkable symbol exists for these. Inline; never declare `extern`:

| C | Rust |
|---|---|
| `NT_SUCCESS(s)` | `s >= 0` |
| `NT_ERROR(s)` | `(s as u32) >> 30 == 3` |
| `UlongToPtr(v)` | `v as usize as PVOID` |
| `PtrToUlong(p)` | `p as usize as ULONG` |
| `KDDBGPRINT(...)` / `DPRINT`/`DPRINT1` | omit entirely (KDDEBUG is off) |
| `UNREFERENCED_PARAMETER(x)` | `let _ = x;` |
| `ASSERT(x)` | omit entirely |
| `RtlCopyMemory(d,s,n)` | `core::ptr::copy_nonoverlapping(s, d, n as usize)` |
| `RtlZeroMemory(d,n)` | `core::ptr::write_bytes(d, 0, n as usize)` |

C string helpers (`strstr`, `strlen`, `strncmp`, `_strupr`, `atol`,
`_vsnprintf`) are libcntpr/CRT symbols available at the final link — declare
them `unsafe extern "C" { ... }` and call them; do NOT reimplement.

---

## KD Wire Protocol Constants

`windbgkd.h` constants (`PACKET_LEADER`, `CONTROL_PACKET_LEADER`,
`PACKET_LEADER_BYTE`, `CONTROL_PACKET_LEADER_BYTE`, `BREAKIN_PACKET_BYTE`,
`PACKET_TYPE_*`, `PACKET_TRAILING_BYTE`, packet-size constants) come from
`ntkrnl_sys::ddk` when resolved. If a specific constant is NOT in the resolved
imports, use the exact `windbgkd.h` value as a typed `const` (do not guess —
the per-file prompt's C context shows the header value). `KDP_STATUS` is a
file-local C enum: define a matching `#[repr(C)] pub enum KDP_STATUS {
KDP_PACKET_RECEIVED = 0, KDP_PACKET_TIMEOUT = 1, KDP_PACKET_RESEND = 2 }` (or
`type KDP_STATUS = u32` + consts) and match with `if x == K` guards if it came
from bindgen as a c_uint alias.

---

## Port / Framebuffer Access

`CpGetByte`/`CpPutByte`/`CpInitialize`/`CpDoesPortExist` (cportlib) do the
actual UART register I/O — call them; do NOT hand-roll `in`/`out` asm in
kdcom. For bootvid framebuffer writes, use `core::ptr::write_volatile` for
every pixel/MMIO store (prevents LLVM eliding the writes). `fontdata.c` is a
pure data blob — emit it as `include_bytes!` of the extracted binary, not a
translated array, when the per-file prompt flags it.

---

## Pointer / Style (same discipline as ntoskrnl/hal)

- `NULL` → `core::ptr::null_mut()` / `null()`
- `&ptr->field` → `&raw mut (*ptr).field`
- `ptr->field` → `(*ptr).field`
- Writes to OUT-pointer params → `core::ptr::write_volatile((*out)..., v)`
  (defeats LLVM `captures(none)` elimination — load-bearing).
- `core::` not `std::`. No `unwrap()`/`expect()`. No doc comments.
- Keep C identifier names. Mark every function `unsafe`.
- DUMMY_PSEH: `_SEH2_TRY{..}_SEH2_EXCEPT(f){h}` → emit TRY body, drop handler.
- `goto cleanup;` → `'cleanup: { ... break 'cleanup; }` labeled block.
- Don't reorder function bodies for call cycles — emit a forward
  `unsafe extern "system" { fn X(...); }` decl instead.
- One C file = one Rust module. Preserve function order.

---

## Quick "DO NOT" List

- ❌ `use ntkrnl_sys::ddk::*` or any glob.
- ❌ `ntkrnl_sys::{globals,constants,base,macros,hal_ddk}` (wrong crate).
- ❌ `#[export_name]` on i686-gnu stdcall (double-decorates) — use `#[no_mangle]`.
- ❌ Declaring C macros (`NT_SUCCESS`, `UlongToPtr`, `KDDBGPRINT`) as extern fns.
- ❌ Reimplementing `strstr`/`atol`/`_vsnprintf` — declare extern, link libcntpr.
- ❌ Plain `extern` for cross-DLL ntoskrnl DATA — use `kind = "raw-dylib"`.
- ❌ Guessing KD/windbgkd constant values — use the header value from context.
- ❌ Non-volatile writes to OUT params or framebuffer MMIO.
- ❌ Hand-rolled UART `in`/`out` in kdcom — call cportlib `Cp*`.
