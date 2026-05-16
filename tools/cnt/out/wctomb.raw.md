```toml
[symbols.wctomb]
c_signature = "int wctomb(char *mbchar, wchar_t wchar)"
family = "wstring"
slice_pattern = "single_wchar"
core_fn = "NEW:wstring::wchar_to_mb_single"
return_collapse = "i32_passthrough"
inout_effects = "mbchar[0] written with single-byte conversion of wchar; if mbchar==NULL, no write occurs"
preserved_misfeature = "mbchar==NULL returns 0 (state-independent encoding marker), not error. RtlUnicodeToMultiByteN converts exactly 1 wchar_t to up to 1 byte; return value is Size (0 or 1), or -1 on conversion failure."
fuzz_domain = ["mbchar==NULL (expect 0)", "ASCII wchar (expect 1)", "non-ASCII wchar requiring multi-byte (expect Size or -1)", "invalid/surrogate wchar (expect -1)"]
```

## Rationale

**Five questions answered from source:**

1. **Exact return value** (lines 11-22):
   - If `mbchar == NULL`, returns `0` (lines 14-15) — standard state-independent encoding marker for `wctomb`.
   - Otherwise calls `RtlUnicodeToMultiByteN(mbchar, 1, &Size, &wchar, sizeof(WCHAR))` (lines 17-21).
   - On failure (`!NT_SUCCESS(Status)`), returns `-1` (lines 22).
   - On success, returns `(int)Size` — the number of bytes written (0 or 1, since max output is 1 byte).

2. **Which parameters written** (lines 17-21):
   - `mbchar[0]` is written with the multibyte encoding of `wchar` if the conversion succeeds and produces output.
   - The API `RtlUnicodeToMultiByteN` with `MaxBytesInMultiByteString = 1` writes at most 1 byte.
   - If `mbchar == NULL`, no write occurs (early return line 15).

3. **NULL behavior** (lines 14-15):
   - Explicit guard: `if (mbchar == NULL) return 0;`
   - This is **not** a crash — it's the standard behavior for `wctomb(NULL, *)` to query state-dependence of the encoding (returns 0 for state-independent like UTF-8/ASCII).

4. **Deviation from textbook** (lines 17-21):
   - The **key deviation**: `RtlUnicodeToMultiByteN` is called with `MaxBytesInMultiByteString = 1`, meaning the output buffer can hold **only 1 byte**.
   - Standard `wctomb` would allow up to `MB_CUR_MAX` bytes (often 4-6 for UTF-8). Here, multi-byte characters (e.g., non-ASCII UTF-8) **cannot** be fully encoded — the function will either fail (return -1) or produce truncated output (Size = 0 or 1).
   - This is a **buffer-size constraint misfeature**: the function can only encode single-byte characters (ASCII range in most SBCS/UTF-8 contexts).

5. **Fuzz domain** (equivalence classes):
   - `mbchar == NULL` → must return `0` (line 15).
   - ASCII `wchar` (U+0000–U+007F) → should succeed with Size = 1, return `1`.
   - Non-ASCII `wchar` requiring multi-byte UTF-8 (e.g., U+00A0–U+FFFF) → likely **fails** due to 1-byte limit, return `-1` or Size = 0.
   - Invalid/surrogate `wchar` (U+D800–U+DFFF) → conversion failure, return `-1`.
   - Covers: NULL-query, success, truncation/failure boundary, invalid input.

**Proposed core signature** (one sentence):
`wstring::wchar_to_mb_single(wchar: u16) -> Result<u8, ConversionError>` — converts a single `wchar_t` to a single-byte multibyte character (ASCII/SBCS range), returning the byte or error if multi-byte encoding required or invalid.

**Slice pattern rationale**:
Not a standard pattern — `wchar` is a scalar `wchar_t`, not a slice. `mbchar` is a single-byte output pointer (not length-prefixed). Labeled `single_wchar` to indicate scalar input + 1-byte output semantics. The shim would:
- Check `mbchar.is_null()` → return `0`.
- Call `wchar_to_mb_single(wchar)`.
- On `Ok(byte)`: write `*mbchar = byte`, return `1`.
- On `Err(_)`: return `-1`.