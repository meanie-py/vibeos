```toml
[symbols.toupper_nt_mb]
c_signature = "int __cdecl toupper_nt_mb(_In_ int _C)"
family = "ctype"
slice_pattern = "int_passthrough"
core_fn = "NEW:ctype::ansi_to_upper_mb"
return_collapse = "i32_passthrough"
inout_effects = "none (pure function)"
preserved_misfeature = "Returns entire multibyte sequence packed into int: 2-byte result as (chr[1] | (chr[0] << 8)), 1-byte as chr[0], fallback to input unchanged if conversion fails (mbSize == 0). This is NOT standard toupper which operates on single chars; it's a Windows NT multibyte extension that round-trips through Unicode (ANSI→Unicode→upcase→ANSI) and packs the result bytes into a single int return value."
fuzz_domain = ["ascii_lowercase ('a'-'z')", "ascii_uppercase ('A'-'Z')", "ascii_non_alpha (digits, punct)", "high_bit_set_single_byte (0x80-0xFF)", "multibyte_lead_bytes (DBCS sequences where applicable)", "zero (0x00)", "negative_input (-1, other negatives)"]
```

## Rationale

**Five questions answered:**

1. **Exact return value:** The function interprets `_C` as an ANSI multibyte character sequence (via `ptr = (PUCHAR)&_C`), converts to Unicode via `RtlAnsiCharToUnicodeChar`, upcases via `RtlUpcaseUnicodeChar`, converts back to multibyte via `RtlUnicodeToMultiByteN`. If the result is 2 bytes, returns `(UCHAR)chrUpper[1] + ((UCHAR)chrUpper[0] << 8)` (little-endian packed). If 1 byte, returns `(UCHAR)chrUpper[0]`. If conversion produces 0 bytes (`mbSize == 0`), returns `(WCHAR)_C` unchanged (lines 24-29).

2. **Which parameters written:** None. Pure function with no side effects. The intermediate `chrUpper` buffer is local stack allocation.

3. **NULL/zero-length behavior:** Input `_C == 0` is treated as valid ANSI character (null byte). The function does not check for NULL pointers since `_C` is passed by value. Zero input will be processed through the conversion pipeline; if the round-trip succeeds with 1 byte, returns `(UCHAR)chrUpper[0]`, otherwise falls back to `(WCHAR)_C` which is 0.

4. **Deviation from textbook libc:** Standard `toupper(int c)` is defined for single-byte characters (0-255) or EOF, operates on the character classification of the current locale, and returns the uppercase mapping or the input unchanged if not a lowercase letter. This function is fundamentally different: it's a **Windows NT multibyte extension** that interprets the int as potentially containing multibyte lead/trail bytes in its low bytes, round-trips through Unicode conversion for case mapping, and packs the multibyte result back into the int. The 2-byte packing `(chr[1] | (chr[0] << 8))` and fallback to `(WCHAR)_C` are non-standard. This is NOT a locale-aware ASCII `toupper`; it's an ANSI codepage-aware multibyte transformer using NT kernel functions.

5. **Minimal fuzz inputs:** Must cover ASCII single-byte (lowercase, uppercase, non-alpha), high-bit-set single-byte (extended ASCII), multibyte sequences if the active ANSI codepage supports DBCS (e.g., Shift-JIS, GBK), zero, and negative values (to test sign-extension behavior when casting to `PUCHAR`). The fallback path (`mbSize == 0`) is critical and may trigger on invalid multibyte sequences or unsupported characters.

**Core function needed:** `NEW:ctype::ansi_to_upper_mb` with signature approximately `fn ansi_to_upper_mb(c: i32) -> i32`. This core must replicate the NT conversion pipeline: treat the i32 as ANSI byte(s), convert to Unicode, upcase, convert back to ANSI multibyte, and pack the result as per the C code. The core will need access to codepage conversion tables or delegate to platform APIs (though `cnt-rust` is no_std, so this may require a codepage abstraction layer). The shim is trivial: pass `_C` through and return the core result directly.