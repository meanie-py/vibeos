```toml
[symbols._itoa_s]
c_signature = "int _itoa_s(int value, char *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:radix::format_i32"
return_collapse = "i32_passthrough"
inout_effects = "str[0..len] written with NUL-terminated decimal/radix string on success; str[0] = '\\0' on EINVAL; str[0..size] partially filled backwards (no sign) + str[0] = '\\0' on ERANGE"
preserved_misfeature = "Delegates to _ltoa_s (line 144-147), which truncates buffer on ERANGE by copying backwards WITHOUT the sign, then setting str[0]='\\0' (lines 234-247). This produces a malformed non-NUL-terminated partial string before the terminal NUL at str[0]. Parameter validation uses MSVCRT_CHECK_PMT; under _LIBCNT_ errno is NOT set (lines 219, 225)."
fuzz_domain = ["radix=10 negative", "radix=10 positive", "radix=2", "radix=16", "radix=36", "size exactly sufficient", "size insufficient (ERANGE)", "size=0 (EINVAL)", "str=NULL (EINVAL)", "radix=1 (EINVAL)", "radix=37 (EINVAL)", "INT_MIN (most negative)", "zero"]
```

## Rationale

**1. Exact return value:** `_itoa_s` immediately delegates to `_ltoa_s(value, str, size, radix)` (lines 144-147), widening `int` to `long`. The return contract is therefore identical to `_ltoa_s`:
   - Returns `0` on success (line 250)
   - Returns `EINVAL` if `str == NULL`, `size == 0`, `radix < 2`, or `radix > 36` (lines 216-225)
   - Returns `ERANGE` if the formatted string length (including NUL) exceeds `size` (line 246)

**2. Which parameters are written:**
   - On success: `str[0..len-1]` receives the NUL-terminated radix string (line 249: `memcpy(str, pos, len)` where `len = buffer + 33 - pos` includes the NUL)
   - On `EINVAL`: If `str && size`, `str[0] = '\0'` (line 221)
   - On `ERANGE`: The buffer-too-small path (lines 232-246) copies `size` characters backwards from the temporary buffer (starting at `buffer[31]` going down), **excluding the negative sign** if present (lines 236-238), then unconditionally sets `str[0] = '\0'` (line 244). This produces a malformed partial result: bytes `[1..size)` contain the *least significant* digits in reverse chronological order, then `str[0]` is overwritten with NUL. The string is not validly NUL-terminated except at position 0.

**3. NULL/zero-length behavior:**
   - `str == NULL` or `size == 0` → `EINVAL` via `MSVCRT_CHECK_PMT` guards (lines 216-217)
   - If both fail, `str[0] = '\0'` is still attempted if `str && size` (line 221), preventing a crash

**4. Preserved misfeature (ERANGE path):**
   Lines 232-246 in `_ltoa_s` implement a bizarre truncation strategy on buffer overflow. Instead of truncating the *most significant* digits (which would preserve the sign and least-significant digits), it:
   - Skips the sign if negative (lines 236-238: `p++; size--;`)
   - Copies `size` characters *backwards* from `buffer[31]` downward (the END of the string, excluding NUL) into `str[1..size]` (lines 242-243)
   - Overwrites `str[0] = '\0'` (line 244)
   
   The result is that `str[0]` is NUL, and `str[1..size]` contains the *least significant* `size-1` digits in **left-to-right order** (because the loop copies right-to-left from the buffer). This is not a valid C string in any conventional sense—it's a NUL followed by digits with no terminator. This appears to be an attempt to satisfy a non-NULL return while signaling error, but it's not documented behavior in any Windows CRT spec. The comment "Don't copy the negative sign if present" confirms intent, but the outcome is nonsensical.

**5. Differential test domains:**
   - **Radix coverage:** 2, 10, 16, 36 (boundary values; 10 for signed behavior)
   - **Sign:** Negative (radix 10 only, lines 227-230), zero, positive, `INT_MIN` (most negative 32-bit)
   - **Buffer size:** Exactly sufficient (`size == strlen(result)+1`), one too small (ERANGE), zero (EINVAL)
   - **Validation:** `str=NULL`, `radix=1`, `radix=37` (all EINVAL paths)
   - **Edge case:** `radix=16` with negative (treated as unsigned per line 231: `val = value` when not radix-10-negative)

**New core signature (one sentence):**
`radix::format_i32(value: i32, radix: u32) -> Result<ArrayVec<u8, 33>, RadixError>` — formats a signed 32-bit integer in the given radix (2–36) into a stack buffer, returning the NUL-terminated byte sequence or an error if radix is invalid. The shim must then handle the ERANGE truncation misfeature by mimicking the backwards-copy-without-sign behavior.