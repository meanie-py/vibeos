# msvcrt rules (the C runtime itself)

This target is **msvcrt.dll — the CRT**, not a Win32 DLL that *calls* the CRT.
Override the kernel-oriented guidance above where it conflicts:

1. **FFI substrate is `msvcrt_sys`, flat — never `ntkrnl_sys` or `ntdll_sys`.**
   Import with a single `use msvcrt_sys::*;`. `msvcrt-crt-rust` does not depend
   on ntdll-sys at all, so an `ntdll_sys::nt::` path cannot resolve.

2. **Per-thread and per-locale state comes from `msvcrt_sys`. NEVER declare
   your own version of one of those structs.** The layouts are a bindgen
   snapshot guarded on both sides by two compilers, and they are *not* stable by
   inspection: `pcumap` sits at offset 208 on i386 but 336 on amd64/arm64, and
   `pctype`/`mb_cur_max` migrate to the front of `threadlocinfo` at
   `_MSVCR_VER >= 140`. A hand-written struct or a numeric offset here is the
   exact bug class `tools/scan_magic_offsets.py` exists to forbid.
   - `get_locinfo()` → `pthreadlocinfo`: `pctype`, `pclmap`, `pcumap`, `lconv`,
     `mb_cur_max`, `lc_handle[]`, `lc_codepage`.
   - `get_mbcinfo()` → `pthreadmbcinfo`: `mbcodepage`, `ismbcodepage`, `mblcid`,
     `mbulinfo[6]`, `mbctype[257]`, `mbcasemap[256]`.
   - `get_thread_data()` → `thread_data_t`: `random_seed`, the `strtok`/`wcstok`/
     `_mbstok` saved pointers, `errno`.
   - Note `mbctype` is indexed **`[c + 1]`**, not `[c]` — slot 0 is a header byte.
     Preserve the C's `+ 1` exactly; do not "fix" it.

3. **Locale correctness lives in the DATA, so read the real table.** msvcrt's
   `_toupper_l` is, for `c < 256`, exactly `locinfo->pcumap[c]`. A body that
   reads the same table inherits locale-correctness by construction. NEVER
   substitute a hardcoded ASCII fold (`c >= b'a' && c <= b'z'`) — that is right
   only in the "C" locale and silently wrong in CP1252, and it is precisely what
   the differential harness's locale pin cannot catch.

4. **Error discipline is the CRT's, not NT's.** There are no `STATUS_*` codes
   here. On a bad argument the C calls `_invalid_parameter(...)` (often via the
   `MSVCRT_CHECK_PMT` / `CHECK_PMT` macro) and then sets `*_errno() = EINVAL` and
   returns the documented failure value. Reproduce all three steps — the errno
   write and the return value are independently observable and both are graded.
   Never swallow the failure path.

5. **Never hand-write `memcpy` / `memset` / `memmove` / `memcmp`.** LLVM folds any
   byte-copy loop back into a call to the function itself → infinite
   self-recursion → silent hang. Import them; msvcrt exports them.

6. **A bindgen flexible array is `[T; 1]`. Indexing it past 0 with `[i]`
   bounds-check-PANICS.** This shipped live in this tree (`ws2help`, then 37
   times in `iphlpapi`). Always `.as_mut_ptr().add(i)` / `.as_ptr().add(i)`.

7. **`char` is signed here, and that is load-bearing.** Widening a `char` to
   compare against `WEOF` must reproduce the sign extension (`*p as i8 as i16 as
   u16`), because byte `0xFF` becoming `0xFFFF == WEOF` is behaviour the C has
   and callers depend on. Do not "clean it up" to an unsigned widen.

8. **i386 is pre-SSE (`-march=pentium`, `-sse,-sse2`).** `extern "C" fn -> f64` is
   fine (the C ABI returns `double` in x87 `ST(0)`). What hard-errors is a
   *Rust*-ABI `-> f64` and the SSE-pulling `f64`/`f32` std methods: use bit tests
   over the raw `u64`/`u32` pattern instead of `is_infinite()` / `is_nan()` /
   `f64::from_bits`.

9. **Exports are `cdecl` on every arch.** msvcrt's spec is `@ cdecl`, so a
   `#[no_mangle] pub unsafe extern "C" fn` is correct on i386 too — do NOT use
   `extern "system"`/stdcall for a CRT export (that would decorate it `_name@N`
   and the DLL would lose the symbol).

10. **Keep upstream Wine quirks; mark them.** This is a faithful transliteration
    of Wine's C. If a body has a bug that upstream later fixed, reproduce the
    behaviour as-is and add a `// BUG-KEPT:` comment saying so. Silently
    "correcting" it diverges msvcrt.dll from the reference the harness grades
    against.
