```toml
[symbols._i64toa]
c_signature = "char* _i64toa(__int64 value, char *string, int radix)"
family = "radix"
slice_pattern = "dst_unchecked"
core_fn = "NEW:i64_to_radix"
return_collapse = "ret_dst"
inout_effects = "string written with NUL-terminated ASCII digits of `value` in base `radix` (2-36), lowercase a-f for 10-15; negative sign only if radix==10 && value<0; caller must ensure string has ≥65 bytes (no bounds check)"
preserved_misfeature = "NO validation of radix (caller responsible for 2-36), NO bounds check on output buffer (caller must ensure ≥65 bytes available), NO error return (always returns input pointer). Unlike _i64toa_s, this is an unchecked API that silently corrupts memory if buffer too small or radix invalid."
fuzz_domain = ["value={0, 1, -1, INT64_MIN, INT64_MAX}", "radix={2, 10, 16, 36, 1, 37}", "buffer_adequate={true, false}"]
```

## Rationale

**1. Exact return value:** Always returns the input `string` pointer unchanged (line 62). The conversion result is written *into* the buffer, but the function returns the original pointer, not the position where digits start.

**2. Parameters written:** Only `string` is written. The function builds the representation backwards in a 65-byte stack `buffer[65]` (lines 18, 20-21), then uses `memcpy(string, pos, &buffer[64] - pos + 1)` (line 61) to copy the result including NUL terminator into the caller's buffer. For negative values when `radix == 10`, a `-` prefix is prepended (lines 13-16, 58-60). Digits 0-9 use ASCII '0'-'9', digits 10-35 use lowercase 'a'-'z' (lines 51-55).

**3. NULL/bounds behavior:** NO NULL check on `string`. NO validation that `radix` is in range 2-36. NO bounds check that the output buffer is large enough. The code always writes up to 65 characters (max: 64 binary digits + NUL for UINT64_MAX, or '-' + 63 digits + NUL for INT64_MIN in base 2). If the caller passes a buffer smaller than required or an invalid radix, memory corruption occurs silently.

**4. Deliberate deviation from textbook:** The textbook `i64toa` often validates radix and returns NULL on error. **This implementation performs NO validation** (contrasting sharply with `_i64toa_s` in the same file, lines 67-85, which validates `str != NULL`, `size > 0`, and `radix` in 2-36, returning `EINVAL` on failure). The unchecked variant is a raw, fast conversion that trades safety for performance—caller must guarantee valid radix and adequate buffer space. Additionally, negative representation is **only** triggered when `radix == 10` (lines 13-16); for all other radixes, the value is reinterpreted as unsigned (line 17), producing a large positive hex/octal/etc. representation for negative inputs.

**5. Minimal test cases:**
   - `value=0, radix=10` → "0"
   - `value=1, radix=2` → "1"
   - `value=-1, radix=10` → "-1" (signed)
   - `value=-1, radix=16` → "ffffffffffffffff" (unsigned reinterpret, 16 f's)
   - `value=INT64_MIN, radix=10` → "-9223372036854775808" (most digits + sign)
   - `value=INT64_MAX, radix=2` → 63 '1's (longest output)
   - `radix=36` → exercises 'a'-'z' digit encoding
   - `radix=1` or `radix=37` → undefined/corruption (misfeature: no guard)
   - Buffer undersized (e.g., 10 bytes for INT64_MIN base-10) → corruption (misfeature: no check)

**Proposed core signature:** `fn i64_to_radix(value: i64, radix: u32) -> Result<ArrayVec<u8, 65>, RadixError>` returning the NUL-terminated ASCII representation in a fixed-size buffer, with error on invalid radix. The shim would `.unwrap_unchecked()` to preserve the no-check contract and `memcpy` the result into the C buffer.