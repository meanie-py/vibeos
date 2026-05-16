```toml
[symbols._ultow]
c_signature = "wchar_t * _ultow(unsigned long value, wchar_t *string, int radix)"
family = "radix"
slice_pattern = "NEW:format_to_opt_buf"
core_fn = "NEW:format_u32_radix"
return_collapse = "ptr_or_null_passthrough"
inout_effects = "if string!=NULL: writes NUL-terminated radix representation (lowercase a-z for digits>9); if string==NULL: no write"
preserved_misfeature = "if string==NULL, returns NULL without crash (lines 264-266: guard `if (string != NULL)` before memcpy, unconditional `return string`); no radix validation (accepts any int radix, will divide-by-zero if radix<2)"
fuzz_domain = ["value=0", "value=MAX_U32", "radix=2,10,16,36", "string=NULL", "string=valid", "value requires multi-digit in radix (e.g., value=255,radix=16 → 'ff')"]
```

## Rationale

**1. Exact return value:**
Lines 242-266 show `_ultow` returns `string` unconditionally (line 266), even if `string` is NULL. The function formats `value` as an unsigned long in the given `radix` using lowercase letters 'a'-'z' for digits 10-35 (lines 254-258). The loop (lines 251-260) produces at least one digit (terminates on `value != 0L` after each iteration decrements `value`), so `value=0` produces "0".

**2. Which parameters are written:**
Only `string` is written, and only if non-NULL (line 264 guard). Lines 264-265 `memcpy` the NUL-terminated result from the stack buffer `pos` to `string`, copying `(&buffer[32] - pos + 1) * sizeof(WCHAR)` bytes (the +1 includes the NUL terminator placed at line 250).

**3. NULL / zero-length / edge behavior:**
Lines 264-266: the `if (string != NULL)` guard prevents dereferencing NULL, but line 266 returns `string` regardless, so `_ultow(42, NULL, 10)` returns NULL (no crash). There is **no validation of `radix`**; if `radix < 2`, line 253 `value % radix` and line 254 `value / radix` will divide-by-zero. If `radix == 1`, infinite loop. This contrasts with `_ui64tow_s` (lines 124-127) which validates `2 <= radix <= 36`.

**4. Deliberate deviation from textbook:**
- **NULL-safety**: Textbook `ultow` typically requires non-NULL `string`. ReactOS allows NULL and returns NULL without crash (lines 264-266).
- **No radix validation**: Textbook implementations validate `2 <= radix <= 36`. ReactOS accepts any `radix` value without checking (no validation code present), leading to undefined behavior (divide-by-zero or infinite loop) for invalid radix. This is consistent with the unsafe `_ui64tow` (lines 75-94) but contrasts with the safe `_ui64tow_s` (lines 124-127).

**5. Minimal differential test inputs:**
- `value=0, radix=10, string=valid` → "0" (line 251 loop executes once)
- `value=0xFFFFFFFF, radix=16, string=valid` → "ffffffff" (max u32, lowercase hex)
- `value=15, radix=16, string=valid` → "f" (single-digit in radix, tests line 257-258 branch)
- `value=10, radix=10, string=valid` → "10" (multi-digit in radix 10)
- `value=42, radix=10, string=NULL` → returns NULL, no crash (tests lines 264-266 NULL guard)
- `value=35, radix=36, string=valid` → "z" (max valid digit, tests line 258)
- `value=36, radix=2, string=valid` → "100100" (exercises many iterations, base-2)

**Proposed new core:** `NEW:format_u32_radix` with signature `fn format_u32_radix(value: u32, radix: u32) -> ArrayVec<u16, 33>` (returns wide-char digits in a fixed-size buffer, caller slices as needed). The shim pattern `NEW:format_to_opt_buf` describes: if `string.is_null()` return NULL, else call core, memcpy result to `string`, return `string`.