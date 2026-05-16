```toml
[symbols._ultoa]
c_signature = "char * _ultoa(unsigned long value, char *string, int radix)"
family = "radix"
slice_pattern = "radix_to_string"
core_fn = "NEW:radix::format_u64"
return_collapse = "ret_dst"
inout_effects = "writes NUL-terminated base-[radix] representation of value to string (lowercase a-z for digits 10-35); returns string unchanged"
preserved_misfeature = "NO radix validation (radix<2 or radix>36 causes modulo-by-invalid UB or incorrect output); NO NULL-check on string (immediate crash); unconditional write assumes string has sufficient capacity (max 33 bytes for binary u32, no overflow detection)"
fuzz_domain = ["value=0", "value=ULONG_MAX", "radix=2 (max digits)", "radix=10 (common)", "radix=16 (hex)", "radix=36 (max valid)", "radix=1 (invalid)", "radix=37 (invalid)", "value with digits requiring a-z (value>=10 in radix<=36)"]
```

## Rationale

**1. Exact return value as a function of inputs:**
Lines 277-293 show the conversion loop: `pos = &buffer[32]; *pos = '\0';` initializes at buffer end, then `do { digit = value % radix; value /= radix; ... } while (value != 0L);` builds the string backwards by prepending digits. Digits 0-9 use `'0' + digit`, digits 10-35 use `'a' + digit - 10` (lowercase). Line 295 `memcpy(string, pos, &buffer[32] - pos + 1);` copies the result including NUL terminator to `string`. Line 297 `return string;` returns the destination pointer unchanged (the standard `_ultoa` return convention).

**2. Which parameters are written:**
`string` is written with the NUL-terminated base-[radix] representation of `value`. The function writes from 2 bytes (value=0 → "0\0") to 33 bytes (ULONG_MAX in binary on 32-bit: 32 digits + NUL). No endptr or padding; just the minimal digit string plus NUL.

**3. NULL / zero-length / not-found behavior:**
**NO NULL-check on `string`** (line 295 `memcpy(string, ...)` proceeds unconditionally). NULL `string` → immediate crash. When `value == 0`, the loop executes once (lines 286-292: `digit = 0`, `*--pos = '0'`, `value = 0`, condition `value != 0L` false), producing "0\0" (2 bytes). **NO radix validation**: unlike the `_s` variants (e.g., `_ltoa_s` lines 211-214 check `radix >= 2 && radix <= 36`), `_ultoa` has **no guard**. `radix < 2` causes modulo-by-zero/one UB; `radix > 36` produces incorrect digit characters (line 291 `'a' + digit - 10` overflows lowercase alphabet).

**4. Deliberate deviation from textbook libc:**
The **missing radix validation** is the critical misfeature. Textbook `ultoa` (and ReactOS's own `_ultoa_s` at line 252-254) mandates `2 <= radix <= 36`. Here, lines 277-297 proceed with **no MSVCRT_CHECK_PMT** or radix range test. This matches ReactOS's non-`_s` variants (`_ltoa`, `_itoa`, `_i64toa` also omit validation) but contradicts the `_s` functions in the same file and standard library safety expectations. Additionally, **no buffer overflow protection**: the function assumes `string` has ≥33 bytes available (max for `ULONG_MAX` in base-2), but never checks `string` capacity—unlike `_ltoa_s` (lines 230-246) which enforces `size` and returns `ERANGE` on overflow.

**5. Minimal input equivalence-classes:**
- **value boundaries**: `value=0` (single-digit, loop minimum), `ULONG_MAX` (max digits: 32 in binary, 10 in decimal, 8 in hex).
- **radix coverage**: `radix=2` (max buffer usage), `radix=10` (most common, decimal), `radix=16` (hex, tests lowercase a-f), `radix=36` (max valid, tests 'z'), `radix=1` (invalid, UB path), `radix=37` (invalid, tests overflow of lowercase alphabet), `radix=8` (octal, intermediate).
- **digit-character mapping**: `value` requiring `a-z` (e.g., `value=255` in `radix=16` → "ff") vs. pure `0-9` (e.g., `value=123` in `radix=10`).
- **NULL dst**: `string=NULL` with any `value`/`radix` (crash path).

**Core signature needed:**
`NEW:radix::format_u64(value: u64, radix: u32) -> Result<ArrayVec<u8, 65>, RadixError>` (or return `String` analogue in no_std). Returns the digit buffer; shim memcpys to C `string`. The shim must **NOT validate radix** (preserve UB) and **NOT check `string` NULL** (preserve crash). Return collapse: `ret_dst` (shim always returns `string` as-is per line 297).