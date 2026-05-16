```toml
[symbols._ltoa]
c_signature = "char* _ltoa(long value, char *string, int radix)"
family = "radix"
slice_pattern = "none"
core_fn = "NEW:radix::format_long"
return_collapse = "ret_dst"
inout_effects = "Writes the ASCII representation of `value` in base `radix` (2-36) to `string`, NUL-terminated. Always writes at least 2 bytes (digit + NUL). Negative values only get '-' prefix when radix==10; otherwise treated as unsigned. Digits >=10 use lowercase 'a'-'z'. Writes unbounded: worst-case 33 bytes for radix=2 (sign + 31 bits + NUL, but really 32 bits unsigned). Returns `string` pointer unchanged."
preserved_misfeature = "No bounds checking: buffer overflow if `string` lacks space (up to 33 bytes needed). Negative values only signed-extend when radix==10 (lines 146-148); for any other radix, `val = value` casts negative `long` to `unsigned long`, producing large positive value (line 151). No validation that radix ∈ [2,36] — invalid radix causes divide-by-zero or garbage output."
fuzz_domain = ["radix=10 value=-2147483648 (min negative, edge of negation)", "radix=10 value=0 (single digit '0')", "radix=10 value=2147483647 (max positive)", "radix=2 value=-1 (becomes 0xFFFFFFFF, all '1' digits, longest output)", "radix=36 value=46655 (exactly 'zzz', tests digit>10 boundary)", "radix=16 value=-256 (tests unsigned cast for non-decimal)", "radix=1 (invalid, will infinite-loop or crash)", "radix=37 (invalid, 'a'+27 exceeds lowercase)"]
```

## Rationale

**Q1: Exact return value?**
Lines 169-170: `memcpy(string, pos, &buffer[32] - pos + 1); return string;` — always returns the input `string` pointer unchanged, regardless of value/radix.

**Q2: Which parameters written, what exactly?**
Lines 146-168 build the ASCII representation in a 33-byte stack buffer (backwards from index 32), then line 169 copies `[pos, &buffer[32]+1)` to `string`, including the NUL terminator (line 154: `*pos = '\0'`). The algorithm ensures at least one digit is written (do-while loop at lines 156-166 guarantees `val % radix` runs at least once even if `val==0` initially after the sign check). For `radix==10` and `value < 0`, a '-' prefix is prepended (line 167). The length varies from 2 bytes (e.g., "0\0") up to 33 bytes worst-case (32 digits + NUL for radix=2 with all bits set, though the buffer accommodates 32 chars + NUL for a 32-bit `long`).

**Q3: NULL/zero-length/not-found behavior?**
No NULL checks on `string` or `radix` validation. If `string==NULL`, line 169 `memcpy` will crash. If `radix==0` or `radix==1`, line 157 `val % radix` causes divide-by-zero or infinite loop. `value==0` works correctly: the do-while writes "0\0".

**Q4: Deviation from textbook?**
Lines 146-151: Negative handling is **radix-conditional**. For `radix==10`, negative `value` is negated to `unsigned long` and a '-' is prepended (lines 146-148, 167). For **any other radix**, `negative=0` and `val=value` (line 151), meaning the negative `long` is **reinterpreted as `unsigned long`** (C's implicit cast), producing a large positive value. This matches historical MSVCRT behavior (comment line 1: "taken from wine ntdll and msvcrt") but deviates from textbook libc, which typically either restricts signed conversion to radix 10 or defines the behavior explicitly. Additionally, **no bounds checking** exists — the caller must ensure `string` has ≥33 bytes, but the function provides no mechanism to verify this (unlike `_ltoa_s` at lines 176-234). Finally, **no radix validation** — values outside [2,36] cause UB (divide-by-zero for ≤1, out-of-range ASCII for ≥37 at line 163 `'a' + digit - 10`).

**Q5: Differential test must cover:**
- **radix=10, value<0**: exercises signed path (line 146-148), ensures '-' prefix and correct negation.
- **radix≠10, value<0**: exercises unsigned cast (line 151), ensures large positive output (e.g., `-1` → "ffffffff" for radix=16).
- **value=0**: single digit "0\0" output.
- **Digit boundaries**: `digit<10` (line 160) vs `digit>=10` (line 162, 'a'-'z' encoding).
- **Edge lengths**: radix=2 with all bits set (longest output), radix=36 (largest valid radix).
- **Invalid radix** (though UB): radix=1 or 37 to detect lack of validation.

**New core needed:**
`radix::format_long(value: i32, radix: u32) -> Result<ArrayVec<u8, 33>, RadixError>` — formats `value` as ASCII in base `radix` (2-36), returning the complete NUL-terminated byte sequence. Handles radix validation, negative-only-in-decimal logic, and digit encoding ('0'-'9', 'a'-'z'). Returns `RadixError::InvalidRadix` if radix ∉ [2,36]. The shim unsafely copies the result bytes to `string`, then returns `string` (no length bounds enforcement, matching the C misfeature).