# cryptlib-rust — Assembly Notes

## CMakeLists Cohab Flip Plan

Target: `sdk/lib/cryptlib/CMakeLists.txt`

Current cmake:
```cmake
list(APPEND SOURCE aes.c des.c md4.c md5.c mvAesAlg.c rc4.c sha1.c util.c)
add_library(cryptlib ${SOURCE})
add_dependencies(cryptlib xdk)
```

Cohab flip plan (per-file, independently revertible):

1. Copy `cryptlib-rust/` into `sdk/lib/rust/cryptlib-rust/` (or existing workspace).
2. Add `cryptlib-rust` to `sdk/lib/rust/Cargo.toml` workspace members.
3. In CMakeLists.txt:
   ```cmake
   # Disable C sources (replaced by Rust)
   # REMOVE_ITEM SOURCE aes.c des.c md4.c md5.c mvAesAlg.c rc4.c sha1.c util.c
   list(REMOVE_ITEM SOURCE aes.c des.c md4.c md5.c mvAesAlg.c rc4.c sha1.c util.c)
   
   add_library(cryptlib ${SOURCE})
   add_dependencies(cryptlib xdk cryptlib_rust_build)
   
   # Link Rust archive
   target_link_libraries(cryptlib PRIVATE
       ${CMAKE_BINARY_DIR}/sdk/lib/rust/rust-kernel-target/<target>/release/libcryptlib_rust.a
   )
   ```
4. Add Rust build rule (mirrors rossym-rust, cmlib-rust patterns in existing cmake).
5. Build and check with `ninja cryptlib` (all 3 arches).
6. Run `nm libcryptlib_rust.a` to verify all public symbols present.

Key cmake note: cryptlib is a static lib linked by `ksecdd.sys` (at minimum). The cohab flip keeps the cmake target name `cryptlib` unchanged — callers see no difference.

---

## Fixes Applied (from triplet-review-findings.md)

### CRITICAL Fixes
1. **des.rs `cookey` src/dst swap** — `copy_nonoverlapping(keyout, dough.as_ptr(), 32)` was src/dst
   swapped in the LLM translation. Fixed to: `copy_nonoverlapping(dough.as_ptr(), keyout, 32)`.
   Source: `des_ecb_encrypt` / `des3_ecb_encrypt` (the cookey scheduling step).

2. **sha1.rs A_SHAUpdate infinite-loop** — `BufferContentSize`, `Buffer`, `BufferSize` were missing
   `let mut`. Without mutation, the while loop `BufferContentSize + BufferSize >= 64` never makes
   progress. Fixed: `let mut BufferContentSize`, `mut Buffer` and `mut BufferSize` as fn params.

### HIGH Fixes
3. **aes.rs `rk` not mut** — `let rk: *mut u32;` changed to `let mut rk: *mut u32;` in
   `aes_ecb_encrypt`. The pointer is rebased with `rk = rk.offset(8)` in the key schedule loop.

4. **byteReverse no-op** — All our targets (x86, amd64, arm64) are little-endian. The C source
   guards byteReverse with `#if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__`. Implemented as an empty
   `#[inline(always)] unsafe fn` in `util.rs`; local copies in `md4.rs` and `md5.rs` to avoid
   cross-module call overhead.

5. **ROR/byte macros-as-extern removed** — The assembled des.rs and aes.rs had `extern "C" { fn
   ROR; fn byte; }` declarations (C macros were mistranslated as extern fns). Replaced with local
   `#[inline(always)] unsafe fn` matching the actual tomcrypt macro semantics.

6. **memcpy/memset extern blocks removed** — sha1.rs and md5.rs had `extern "C"/"system" { fn
   memcpy; fn memset; fn MD5Transform; }` blocks. All replaced with
   `core::ptr::copy_nonoverlapping` and `core::ptr::write_bytes`.

7. **stray `use ntkrnl_sys::*` imports removed** — sha1.rs had leftover ntkrnl_sys glob imports.
   Not available without the dep; not needed (all types self-contained in crate::types).

8. **MD5Update `byteReverse` not called** — The C md5.c calls `byteReverse` before each
   `MD5Transform` on little-endian platforms where the guard macro is active. We keep the calls
   in MD5Update to match C structure; on LE they inline to nothing.

---

## ABI Decisions

| Module | API | ABI | Notes |
|--------|-----|-----|-------|
| md4.rs | MD4Init/Update/Final | `extern "system"` | NTAPI = stdcall on i386 |
| md5.rs | MD5Init/Update/Final | `extern "system"` | NTAPI = stdcall on i386 |
| sha1.rs | A_SHAInit/Update/Final | `extern "system"` | NTAPI = stdcall on i386 |
| aes.rs | aes_ecb_{setup,encrypt,decrypt} | `extern "C"` | tomcrypt, plain cdecl |
| des.rs | des_ecb_{encrypt,decrypt}, des3_* | `extern "C"` | tomcrypt, plain cdecl |
| rc4.rs | rc4_init, rc4_crypt | `extern "C"` | plain cdecl |
| mvaesalg.rs | rijndaelKeySched/Encrypt128/Decrypt128 | `extern "C"` | Marvell ref impl, cdecl |
| util.rs | byteReverse | `extern "C"` | LE no-op |

---

## Preamble Sources

All tables sourced from c2rust (NOT fabricated):
- **des.rs**: SP1-SP8 ([u32;64] × 8), des_ip/des_fp ([[u64;256];8] × 2), bytebit ([u16;64]),
  bigbyte ([u32;24]), pc1 ([u8;56]), pc2 ([u8;48]), totrot ([u8;16]).
- **aes.rs**: TE0-TE3 ([u32;256] × 4), Te4_0-Te4_3 ([u32;256] × 4), TD0-TD3 ([u32;256] × 4),
  Td4 ([u32;256]), Tks0-Tks3 ([u32;256] × 4), rcon ([u32;10]).
- **mvaesalg.rs**: mask ([u8;256]), Logtable ([u8;256]), Alogtable ([u8;256]),
  S ([u8;256]), Si ([u8;256]), rcon ([u32;30]).

---

## Unresolved Items

1. **MD4Update byteReverse alignment**: The C md4.c calls `byteReverse(ctx->in, 16)` before
   MD4Transform. On LE this is a no-op (confirmed by conditional compilation guard). The Rust
   keeps the call structure matching C; the LE no-op is correct but technically this means
   `in_0` is treated as u32 array by MD4Transform (via pointer cast) without any byte swapping —
   identical to the LE C build behavior.

2. **MD4 hand-written**: The md4.rs was hand-written from the C source (no .responses/ file for
   MD4 functions). The MD4Transform implementation uses the standard round constants
   (0x5a827999, 0x6ed9eba1) and MD4 round structure as documented. Should be difftest-verified
   against a known MD4 vector before shipping.

3. **DES des_ip/des_fp table sizes**: The c2rust output for des.rs includes `des_ip` and `des_fp`
   as `[[u64;256];8]` (8×256 = 2048 u64 per table). This is from the DES implementation that
   uses 64-bit subkey scheduling — verify these match the actual C array declarations in tomcrypt.h
   if the DES cohab is exercised.

4. **No cargo check run**: Per REFERENCE.md rules, no `cargo build` was run during assembly.
   First compile will surface any remaining type errors. Expected surfaces:
   - `write_volatile` / raw ptr alignment warnings (lint, not errors)
   - Potential `unused variable` warnings on the `let _ = (a, b, c, d, e);` wipe in SHA1Transform
   - Potential `dead_code` warnings on local helper fns in mvaesalg.rs

5. **mvaesalg LLM body garbage was stripped**: The c2rust preamble pass for mvAesAlg.c produced
   KeyAddition body fragments as the leading "preamble" output. These were stripped and replaced
   with the actual c2rust-derived KeyAddition/ShiftRow/Substitution/MixColumn helper functions.
   The LLM bodies (rijndaelKeySched/Encrypt128/Decrypt128) were kept as-is.
