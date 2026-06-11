# evtlib-rust — Integration Notes

## CMakeLists.txt cohab-flip plan

Target file: `sdk/lib/evtlib/CMakeLists.txt`

Current content:
```cmake
## FIXME: Make the library cross-compiling aware (like cmlib or inflib)
add_library(evtlib evtlib.c)
add_dependencies(evtlib xdk)
```

Proposed replacement (mirror rossym/cmlib `if(CMAKE_CROSSCOMPILING)` pattern):

```cmake
## EventLog file library — C source replaced by Rust when cross-compiling.
## evtlib_rust_stub.c is a zero-symbol TU that satisfies CMake's "at least
## one source file" requirement while the real code lives in evtlib-rust.

if(CMAKE_CROSSCOMPILING)
    # Rust cohabitation: remove C source, link Rust staticlib
    add_library(evtlib evtlib_rust_stub.c)
    add_dependencies(evtlib xdk evtlib_rust)
    target_link_libraries(evtlib PRIVATE ${CMAKE_BINARY_DIR}/sdk/lib/rust/evtlib-rust/libevtlib_rust.a)
else()
    add_library(evtlib evtlib.c)
    add_dependencies(evtlib xdk)
endif()
```

Stub TU (`sdk/lib/evtlib/evtlib_rust_stub.c`):
```c
/* evtlib_rust_stub.c — empty TU; Rust staticlib provides all symbols. */
```

The CMake integration follows the same pattern as rossym-rust and the swenum/usbd cohab stubs already in the tree (see `drivers/ksfilter/swenum/rust_cohab_stub.c`). The Rust crate builds as part of the workspace via `sdk/lib/rust/Cargo.toml`.

The Cargo workspace entry to add to `sdk/lib/rust/Cargo.toml`:
```toml
"evtlib-rust",
```

## Fixes applied (file:fn → what)

### CRITICAL real-logic fixes

1. **`evtlib.rs:ElfpInitExistingFile` — EVENTLOGEOF_SIZE_FIXED wrong value**
   - LLM: `sizeof(EVENTLOGEOF) - sizeof(u32) = 40 - 4 = 36`
   - Correct: `5 * sizeof(u32) = 20` (= FIELD_OFFSET(EVENTLOGEOF, BeginRecord))
   - Fix: `const EVENTLOGEOF_SIZE_FIXED: usize = 5 * core::mem::size_of::<u32>()`
   - Impact: With 36, the EOF-record scan reads 36 bytes and compares against the
     first 36 bytes of EOFRecord. The C header's `C_ASSERT(EVENTLOGEOF_SIZE_FIXED
     == FIELD_OFFSET(EVENTLOGEOF, BeginRecord))` proves 20 is correct. With 36,
     no existing .evt file ever opened (always returned CORRUPT).

2. **`evtlib.rs:ElfWriteRecord` — RtlCopyMemory UB (overread from 1-byte stack)**
   - LLM: `RtlCopyMemory(&raw mut RecBuf, &(0u8), sizeof::<EVENTLOGRECORD>())`
     copies 48 bytes from a 1-byte stack `u8` — undefined behavior.
   - Fix: `core::ptr::write_bytes(&raw mut RecBuf as *mut u8, 0, size_of::<EVENTLOGRECORD>())`
     (or `let mut RecBuf: EVENTLOGRECORD = core::mem::zeroed()` + `write_bytes`)

3. **`evtlib.rs` — STATUS_EVENTLOG_FILE_CORRUPT wrong constant (×2 locations)**
   - LLM (InitExistingFile): `0xC0000188` = STATUS_LOG_FILE_FULL — WRONG
   - LLM (WriteRecord): `0xC0000189` = STATUS_LOG_FILE_CHANGED/TOO_LATE — WRONG
   - Correct: `0xC000018E` (STATUS_EVENTLOG_FILE_CORRUPT, from ntstatus.h)
   - Fix: single `const STATUS_EVENTLOG_FILE_CORRUPT: i32 = 0xC000018Eu32 as i32`
     used everywhere.

### ABI / shell fixes

4. **`evtlib.rs:ElfCreateFile` — `Status >= 0i32 as i64` type-suspicious cast**
   - LLM: `!(Status >= 0i32 as i64)` — `0i32 as i64` = 0i64, then compared to
     NTSTATUS (i32), which promotes. Works numerically but is confusing.
   - Fix: plain `Status < 0` (NTSTATUS is i32; no cast needed).

5. **Wrong pool TAGs**
   - LLM: `TAG_ELF = 0x464C4545` ('ELEF'), `TAG_ELF_BUF = 0`
   - Correct (from evtlib.h): `TAG_ELF = ' flE' = 0x456c6620`,
     `TAG_ELF_BUF = 'BflE' = 0x456c6642`
   - Fix: two `const` definitions with correct values.

6. **Private helper ABI — bogus `extern {}` blocks removed**
   - LLM output scattered `extern "system"` / `extern "C"` blocks declaring
     `ElfpDeleteOffsetInformation`, `ElfpAddOffsetInformation`, etc. as if they
     were external symbols. Within a single crate these must be plain `unsafe fn`
     called by path.
   - Fix: all private helpers (`ReadLogBuffer`, `WriteLogBuffer`,
     `ElfpOffsetByNumber`, `ElfpAddOffsetInformation`, `ElfpDeleteOffsetInformation`,
     `ElfpInitNewFile`, `ElfpInitExistingFile`) are plain `unsafe fn` with no
     `extern` wrapper. All call sites updated to direct calls.

7. **`extern "system"` vs `extern "C"` on public API**
   - All public `Elf*` functions are NTAPI in the C header → `extern "system"`.
   - `PRINT_HEADER` is plain `VOID` (no NTAPI) → `extern "C"`.
   - LLM mixed `extern "C"` and `extern "system"` inconsistently; fixed throughout.

8. **`NextOffset` 64-bit computation (i386 truncation fix)**
   - LLM: `(FileOffset.QuadPart as usize).wrapping_add(BufSize)` truncates to 32
     bits on i386 (usize = u32).
   - Fix: compute in i64 throughout:
     `(*NextOffset).QuadPart = (FileOffset.QuadPart as i64).wrapping_add(BufSize as i64)`

9. **EVENTLOGEOF field naming — reconciled**
   - LLM preamble used `One/Two/Three/Four` for fields 2–5.
   - LLM `ElfWriteRecord` initializer used `Ones/Twos/Threes/Fours`.
   - C header uses `Ones/Twos/Threes/Fours` (from `_EVENTLOGEOF`).
   - Fix: struct definition and all use sites use `Ones/Twos/Threes/Fours`.

10. **`ElfpInitExistingFile` — `&raw const FileOffset` for `ReadLogBuffer`**
    - `ReadLogBuffer` takes `*const LARGE_INTEGER` for `ByteOffset` (read-only).
    - LLM passed `&raw mut FileOffset` in the scan loop; fixed to `&raw const`.

11. **Duplicate `const` definitions in `ElfpInitExistingFile` and `ElfWriteRecord`**
    - LLM redefined `ELF_LOGFILE_HEADER_DIRTY`, `LOGFILE_SIGNATURE`, etc. as
      local consts inside each function. Removed duplicates; all constants are
      file-scope.

12. **`ElfBackupFile` loop restructure**
    - LLM used a `current_block: u64` Goto-simulation with `match`. Rewritten as
      a clean `'outer` labeled loop that writes the EOF record and flushes inline
      when all records are consumed (mirrors the C source structure).

13. **`ElfCloseFile` — TAG_ELF missing in Free calls**
    - LLM passed `0u32` instead of `TAG_ELF` in the two `Free` calls.
    - Fix: pass `TAG_ELF` in both.

14. **`RtlZeroMemory` signature**
    - ntkrnl-sys declares Length as `usize`; used consistently.

## Self-review results (grep pass)

- **`extern {` blocks for sibling fns**: NONE — all private helpers are plain `unsafe fn`.
- **Undefined consts used**: NONE — every constant referenced is defined at file scope.
- **`extern "system"` vs `"C"` mismatches**: NONE — all NTAPI public functions are
  `extern "system"`; `PRINT_HEADER` (non-NTAPI) is `extern "C"`.
- **Missing `#[no_mangle]`**: NONE — all public Elf* functions have `#[no_mangle]`.
- **STATUS_EVENTLOG_FILE_CORRUPT**: single definition `0xC000018Eu32 as i32`; used in
  9 return sites. Grep confirms no 0xC0000188 / 0xC0000189 remain in this constant's role.
- **EVENTLOGEOF_SIZE_FIXED**: `5 * size_of::<u32>()` = 20 at compile time.
- **TAG_ELF / TAG_ELF_BUF**: 0x456c6620 / 0x456c6642 — correct.
- **RecBuf zero-init in ElfWriteRecord**: `core::mem::zeroed()` + `write_bytes`, no UB.
- **NextOffset computation**: all i64 arithmetic, cast to i64 at end; no usize truncation.

## Unresolved items (for the central grinder)

1. **`UNICODE_STRING` redeclared locally** — the crate declares its own `UNICODE_STRING`
   to avoid pulling in all of `ntkrnl-sys`. The grinder should verify the layout matches
   ntkrnl-sys's definition and consolidate if a shared path exists (e.g., via
   `ntkrnl_sys::base::UNICODE_STRING`). If ntkrnl-sys exports it, replace the local def
   with `use ntkrnl_sys::base::UNICODE_STRING` and remove the local one.

2. **`EVTLOGFILE.ReadOnly` type** — declared as `u8` (BOOLEAN). Some call sites in
   ntkrnl-sys or eventlog.dll may pass it as a different width. Verify against the
   callers in `base/services/eventlog/`.

3. **`ElfBackupFile` — missing `Allocate`/`Free` in BackupLogFile** — the C source does
   not copy Allocate/Free into BackupLogFile (unlike FileSetSize/FileWrite/FileFlush).
   So ReadLogBuffer can call `(*LogFile).Allocate` (the source log) for buffer alloc,
   but the backup path in fact calls `(*LogFile).Allocate` directly — this matches C.
   Verified consistent with the C source; no fix needed.

4. **`PRINT_HEADER` symbol** — the C source gates it with `#if DBG`. The Rust version
   gates the body with `#[cfg(debug_assertions)]` but always exports the symbol. If the
   cmake target needs a true conditional export, use a linker script or `#[cfg(debug_assertions)]`
   on the whole fn. For now, the empty function body in release is harmless.

5. **`pack(4)` on structs** — used for EVENTLOGHEADER, EVENTLOGRECORD, EVENTLOGEOF to
   match `#include <pshpack4.h>`. Verify with `size_of` assertions once the crate
   compiles. Expected sizes: EVENTLOGHEADER=48, EVENTLOGRECORD=56, EVENTLOGEOF=40.

6. **`RtlInitEmptyUnicodeString` / `RtlCopyUnicodeString` ABI** — declared `extern "C"`.
   These are RTL inlines on some targets; confirm they resolve from ntoskrnl's export
   table in the kernel build (they should — ntoskrnl exports both).

7. **Workspace Cargo.toml** — add `"evtlib-rust"` to `sdk/lib/rust/Cargo.toml` members.

8. **ntkrnl-sys dependency** — the crate lists `ntkrnl-sys` as a dep but the main module
   doesn't `use` it directly (all kernel types are redeclared locally from the C header).
   The dependency is kept for consistency with the cohab pattern and in case future
   modules need it. The grinder can remove it if it causes link friction.
