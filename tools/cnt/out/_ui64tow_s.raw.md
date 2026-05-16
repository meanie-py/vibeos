```toml
[symbols._ui64tow_s]
c_signature = "int _ui64tow_s(unsigned __int64 value, wchar_t *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:radix::format_u64_w"
return_collapse = "i32_passthrough"
inout_effects = "str[0..min(size,actual_len)] written with radix-converted digits + NUL; on error str[0]='\\0' if str && size"
preserved_misfeature = "Line 163: buffer size check `(size_t)(buffer-pos+65) > size` has POINTER SUBTRACTION BUG — should be `(buffer+65-pos)` or `65-(pos-buffer)`. As written, `buffer-pos` is NEGATIVE (buffer is at higher address), wraps to huge positive via size_t cast, almost always triggers false EINVAL. Also line 164: returns EINVAL instead of ERANGE for overflow (unlike _i64tow_s which returns ERANGE)."
fuzz_domain = ["value=0, radix=10", "value=UINT64_MAX, radix=2 (longest output, 64 chars)", "value=12345, radix=16", "size=1 (minimal, only NUL fits)", "size=2..10 (triggers pointer-math bug)", "size=65 (exact fit for max binary)", "radix=2 (min)", "radix=36 (max)", "str=NULL (caught by CHECK_PMT)", "size=0 (caught by CHECK_PMT)", "radix=1 or 37 (out of range)"]
```

## Rationale

**1. Exact return value:**
- Lines 150-156: Parameter validation via `MSVCRT_CHECK_PMT` — if any fail (`str==NULL`, `size==0`, `radix<2`, `radix>36`), returns `EINVAL` immediately (and sets `errno` when not `_LIBCNT_`).
- Lines 157-172: Formats `value` into a temporary 65-wchar_t buffer in reverse (digits 'a'-'z' for 10-35), stopping at buffer[64] initially then decrementing `pos`.
- Lines 174-176: **THE BUG** — The size check `(size_t)(buffer-pos+65) > size` performs pointer arithmetic incorrectly. Since `buffer` is declared at a lower address than `pos` (pos starts at `&buffer[64]`), `buffer - pos` is a *negative* `ptrdiff_t`. When cast to `size_t`, this becomes a huge positive value (e.g., `SIZE_MAX - small_offset`), causing the check to almost always evaluate true for any reasonable `size`, incorrectly returning `EINVAL`.
- Line 174: If the (buggy) check triggers, returns `EINVAL` (NOT `ERANGE` like the signed `_i64tow_s` does on line 148).
- Lines 177-178: If size is sufficient (in theory, though the bug prevents this path for most inputs), calls `memcpy(str, pos, buffer-pos+65)` and returns `0` (success).

The intended behavior is to check if `(buffer + 65 - pos) > size` (the actual string length including NUL), but the pointer subtraction order is reversed.

**2. Parameters written:**
- `str`: On success (lines 177-178), writes the NUL-terminated radix representation starting at `pos` (length `buffer+65-pos` including NUL). On the size-overflow error path (line 175), the function calls `MSVCRT_INVALID_PMT` but does NOT explicitly zero `str[0]` in the overflow branch (unlike `_i64tow_s` which does on line 148). On the validation-failure path (lines 150-156), `str[0] = '\0'` is written only if `str && size` (guarded by the two separate checks on line 150 and the result logic, though the code structure ensures at least one of the validation failures occurs before we confirm both).

**3. NULL/zero-length behavior:**
- `str==NULL` or `size==0`: Caught by `MSVCRT_CHECK_PMT` on lines 150-151, returns `EINVAL`. If both `str && size` are truthy but one of the checks failed, `str[0] = '\0'` is written (lines 152-153). If `str` is NULL, no write occurs (guarded by the `if (str && size)` on line 152).

**4. Deliberate deviations:**
- **Pointer arithmetic bug (line 174):** `(size_t)(buffer-pos+65)` should be `(buffer+65-pos)` or cast `pos-buffer` correctly. As written, `buffer-pos` is negative, wrapping to huge positive on size_t cast, making the overflow check nearly always true, incorrectly rejecting valid inputs with `EINVAL`.
- **Error code mismatch (line 175):** Returns `EINVAL` on size overflow, whereas the signed variant `_i64tow_s` returns `ERANGE` (line 148) and performs a partial backwards copy. This function does NOT copy any partial data on overflow (no equivalent of lines 138-149 in `_i64tow_s`).
- **Lowercase hex digits (lines 169-172):** Uses 'a'-'z' for digits 10-35, matching the unsigned `_ui64tow` (lines 116-120) and other variants in this file.

**5. Minimal fuzz domain:**
- `value=0, radix=10`: Minimal output "0" (2 wchars with NUL).
- `value=UINT64_MAX, radix=2`: Longest output (64 '1's + NUL = 65 wchars), exercises buffer boundary.
- `value=12345, radix=16`: Typical hex conversion ("3039").
- `size=1`: Only NUL fits, should trigger overflow error (but pointer bug may cause EINVAL instead).
- `size=2..10`: Intermediate sizes that expose the pointer-math bug (calculated length is reasonable, but `buffer-pos` wraps).
- `size=65`: Exact fit for worst-case (radix=2, UINT64_MAX), tests boundary condition and the bug.
- `radix=2` and `radix=36`: Min/max valid radix, exercises digit conversion branches.
- `str=NULL`, `size=0`, `radix=1 or 37`: Validation failures (EINVAL path).

**New core needed:** `radix::format_u64_w(value: u64, radix: u8) -> WideString` — converts unsigned 64-bit value to wide-char string in given radix (2-36), lowercase 'a'-'z' for digits 10-35, returns owned WideString. The shim will then check `result.len()+1 <= size`, copy to `str` if sufficient, else return the appropriate error (fixing the pointer bug by computing length correctly from the core's returned string).