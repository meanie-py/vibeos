# inflib-rust crate — Integration Notes

## 1. Kernel/Host File Split

`sdk/lib/inflib/CMakeLists.txt` builds two targets:

### Cross-compiled target (`inflib`, lines 1–25):
```
infcore.c infget.c infput.c infrosgen.c infrosget.c infrosput.c
```
These 6 files are compiled for the PE-COFF target (aarch64/x86_64/i686-windows) and linked
into ntoskrnl, SetupAPI, etc. They live in this crate.

### Host-only target (`inflibhost`, guarded by `NOT CMAKE_CROSSCOMPILING`):
```
infhostgen.c infhostget.c infhostput.c infhostrtl.c
```
These use Linux libc (malloc/free, wchar_t, FILE*) and are excluded from the crate entirely.

## 2. CMakeLists Cohabitation Flip Plan

The pattern mirrors `rossym-rust` / `cmlib-rust`.

### Step A — Add `inflib_rust_stub.c`

Create `sdk/lib/inflib/inflib_rust_stub.c`:
```c
/* Rust cohabitation stub — pull in the inflib-rust staticlib. */
/* Intentionally empty; the CMake glue below links the .a directly. */
```

### Step B — Edit `sdk/lib/inflib/CMakeLists.txt`

Add at the top of the cross-compiled block (inside the existing `if(CMAKE_CROSSCOMPILING)` or
unconditionally if there isn't one):
```cmake
if(CMAKE_CROSSCOMPILING)
    # Rust cohabitation: 6 C files replaced by inflib-rust staticlib.
    set(INFLIB_RUST_LIB "${CMAKE_SOURCE_DIR}/sdk/lib/rust/target/${RUST_TARGET}/release/libinflib_rust.a")
    add_library(inflib STATIC inflib_rust_stub.c)
    target_link_libraries(inflib PRIVATE ${INFLIB_RUST_LIB} ntkrnl_sys)
    # Removed from SOURCE list (now Rust):
    # infcore.c infget.c infput.c infrosgen.c infrosget.c infrosput.c
else()
    # Host (Linux native): unchanged inflibhost target, keep C files.
    add_library(inflibhost STATIC
        infhostgen.c infhostget.c infhostput.c infhostrtl.c)
endif()
```

Note: the exact cmake integration pattern should follow what was used for rossym-rust and
cmlib-rust (see those CMakeLists.txt for the real template).  The crate name in Cargo.toml
is `inflib-rust`; the staticlib output is `libinflib_rust.a`.

## 3. Fixes Applied

### types.rs
- Added `pub type LANGID = u16` (LANGID is only in `ntkrnl_sys::ddk` bindgen layer, not base).
- All INF_STATUS_* constants now carry correct NTSTATUS values (see §B below).
- Changed `use ntkrnl_sys::globals::*` (nonexistent) → `use ntkrnl_sys::constants::*`.

### infcore.rs
- **B1 isspaceW:** `iswctype(*p, _SPACE=0)` placeholder in `line_start_state` /
  `leading_spaces_state` → local `fn isspaceW(c: u16) -> bool` matching iswspace semantics
  (0x09/0x0A/0x0B/0x0C/0x0D/0x20). The `_SPACE=0` constant is gone entirely.
- **B3 offset_of:** `offset_of!(INFCACHEFIELD, Data[wcslen+1])` (invalid Rust) →
  `offset_of!(_INFCACHEFIELD, Data) + (wcslen(Data) + 1) * size_of::<u16>()`.
- **B6 null_mut double-pointer:** `null_mut::<PINFCACHE>()` in `InfpGetSectionForContext` →
  `null_mut::<_INFCACHE>()` (PINFCACHE = `*mut _INFCACHE`; the original was a `**_INFCACHE`).
- All sibling calls via direct Rust path (no `extern {}` for siblings).
- `#[no_mangle]` on all `pub unsafe extern "C" fn Infp*` functions.

### infget.rs
- **B2 NTSTATUS constants:**
  - `INF_STATUS_INVALID_PARAMETER` = `0xC000000Du32 as i32` (was 0x57 / STATUS_INVALID_PARAMETER_1)
  - `INF_STATUS_NOT_FOUND` = `0xC0000225u32 as i32` (was 0xE0000202)
  - `INF_STATUS_NO_MEMORY` = `0xC0000017u32 as i32` (was 1)
- **B4 STATUS_SUCCESS undefined:** replaced with `INF_STATUS_SUCCESS` (= 0) throughout.
- **B5 lang-id type error:** `((0<<10)|((*Inf).LanguageId & 0x3ff))` (i32|u16 mismatch) →
  `let primary_only: u16 = (*Inf).LanguageId & 0x3ff;`.
- **mut missing:** added `mut CacheLine` / `mut Section` in `InfpFindNextMatchLine`.
- Duplicate const definitions removed; all from `crate::types`.

### infput.rs
- **B7 OUTPUTBUFFER union:** `(*OutBuf).__bindgen_anon_1.Status` → `(*OutBuf).Status`.
  The C struct has no union; `__bindgen_anon_1` was bogus assembler output.
- `INFSTATUS` signed i32 throughout; `INF_STATUS_NO_MEMORY = 0xC0000017u32 as i32`.
- `DPRINT` extern block removed (no-op in NDEBUG kernel build).
- `extern "C"` `InfpFindSection`/`InfpAddSection` etc. replaced by direct Rust imports.

### infrosgen.rs
- **Single InfpHeap definition:** assembled version had both a local `static mut InfpHeap`
  and an `extern "system" { static mut InfpHeap }` — duplicate. Fixed: define once as
  `#[no_mangle] pub static mut InfpHeap: *mut c_void = core::ptr::null_mut()`.
- `extern "system"` → `extern "C"` throughout (no stdcall in kernel-mode inflib).
- `use ntkrnl_sys::constants::*` for GENERIC_READ / SYNCHRONIZE / FILE_SHARE_READ.
- Local struct definitions: `FILE_STANDARD_INFORMATION` / `OBJECT_ATTRIBUTES` /
  `IO_STATUS_BLOCK` / `LARGE_INTEGER` (avoids ntkrnl_sys::ddk union complexity).

### infrosget.rs
- **B8 BOOLEAN returns:** `InfGetBinaryField` / `InfGetData` now return `BOOLEAN`
  (was `-> i32` / `as i32`). Pattern: `(0 <= InfpFn(...)) as BOOLEAN`.
- No `extern {}` for siblings; all via `use crate::infget::...`.

### infrosput.rs
- **B9 BOOLEAN return:** `InfFindOrAddSection` returns `BOOLEAN` (was `as i32`).
- **A1 public API names:** `InfHostAddLine` / `InfHostAddField` (wrong) →
  `InfAddLine` / `InfAddField` (correct, from `sdk/lib/inflib/infros.h` lines 73–74).
- Sibling `InfpAddLineWithKey` extern block removed; now direct Rust import from `infput`.
- `use ntkrnl_sys::constants::*` for GENERIC_WRITE / SYNCHRONIZE.

## 4. Self-Review Findings (grep pass)

Checks performed:

| Check | Result |
|-------|--------|
| `extern {}` for sibling fns | None found |
| `extern "system"` usage | None (all `extern "C"`) |
| `STATUS_SUCCESS` undefined | `ntkrnl_sys::base::*` provides it (infrosput InfWriteFile NTSTATUS return) — OK |
| `INF_STATUS_NO_MEMORY = 1` residue | None (only in comment/doc strings) |
| `iswctype` / `_SPACE=0` residue | None (only in comment strings) |
| `offset_of!(Data[...])` residue | None (only in comment strings) |
| `null_mut::<PINFCACHE>()` residue | None (only in comment strings) |
| Missing `#[no_mangle]` on Infp* fns | None — all covered |
| `as i32` on BOOLEAN-return fns | None |
| Duplicate `InfpHeap` static | None — defined once in infrosgen, `extern "C"` in others |

## 5. Unresolved Items

### 5a. `INFSTATUS as NTSTATUS` casts (InfOpenFile, InfOpenBufferedFile, InfWriteFile)
The original C functions return `NTSTATUS` but internally use `INFSTATUS` (which maps to
NTSTATUS in the kernel build). The casts `inf_status as NTSTATUS` are correct at runtime but
Rust will emit a narrowing-cast lint if `INFSTATUS = i32` and `NTSTATUS = i32` (they're the
same width — no actual problem, just noise). Consider `type NTSTATUS = i32` alignment.

### 5b. `wcslen` / `wcscpy` / `_wcsicmp` unresolved
These are declared in `extern "C"` blocks referencing the CRT wide-string functions. In the
kernel build these resolve from `ntoskrnl` (which re-exports `wcslen`/`wcscpy`) or from
`ntdll`. Confirm these export correctly on all 3 architectures during the cohab link step.

### 5c. `InfpBuildFileBuffer` status return
The function always returns `INF_STATUS_SUCCESS` at the end (C original does same). The actual
error is encoded into `OutBuf.Status`. This is faithful to the C but looks wrong — the caller
checks the output `Buffer == NULL` to detect failure. No fix needed; preserved as-is.

### 5d. FILE_STANDARD_INFORMATION layout
The local layout uses `EndOfFile_lo: u32` for the low 32 bits of `EndOfFile`. This is
correct for files < 4GB (the normal inflib use case). For full correctness the `i64` QuadPart
version should be used. Low risk for INF files.

### 5e. Host build excluded
`infhostgen / infhostget / infhostput / infhostrtl` are C-only and excluded from this crate.
The `inflibhost` cmake target is unaffected. If a host-Rust version is ever wanted, a
separate `cfg(feature = "host")` gate with `std` imports would be needed.
