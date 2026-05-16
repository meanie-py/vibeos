```toml
[symbols._itoa]
c_signature = "char* _itoa(int value, char* string, int radix)"
family = "radix"
slice_pattern = "NEW:itoa_delegate"
core_fn = "NEW:format_i32_radix"
return_collapse = "ret_dst"
inout_effects = "string written with NUL-terminated decimal/radix representation (incl. optional '-' prefix); always returns input string pointer"
preserved_misfeature = "NO validation: radix outside [2,36] causes undefined behavior (modulo/array OOB in digit mapping); negative values with radix!=10 are cast to unsigned (lines 112-117 in _ltoa: 'if (value < 0 && radix == 10)' means radix 2-9,11-36 treat negative as large positive); NO bounds check on output string (caller must ensure >=33 bytes for worst-case); lowercase 'a'-'z' for digits 10-35 (line 128)"
fuzz_domain = ["value={INT_MIN, -1, 0, 1, INT_MAX}", "radix={2, 10, 16, 36, invalid=1,37}", "combinations: negative+radix!=10 (unsigned cast), value=0 (single '0'), INT_MIN+radix=10 (most digits)"]
```

## Rationale

**Delegation chain:** `_itoa` (line 179) is a thin wrapper calling `_ltoa(value, string, radix)` (line 181), which contains the actual implementation (lines 187-219). The core logic (lines 206-217) builds the string backward in a 33-byte stack buffer, then `memcpy`s to the output.

**Exact return:** Always returns the input `string` pointer (line 217), regardless of success — this is `ret_dst` collapse. No error signaling.

**Output contract:** Writes a NUL-terminated string representation of `value` in the given radix. For `radix==10` and `value < 0`, prepends '-' and converts the absolute value (lines 206-208). For all other radix or non-negative values, treats the bit pattern as unsigned (lines 209-211: `negative = 0; val = value` casts `long` to `unsigned long`). Digits 0-9 use '0'-'9', 10-35 use 'a'-'z' (lines 214-218).

**Load-bearing misfeatures (lines 206-211, 214-218):**
1. **Unsigned cast for radix≠10 or positive:** A negative `int` with radix 2-9 or 11-36 is reinterpreted as its two's-complement unsigned equivalent (e.g., `-1` in base 16 → `"ffffffff"` on 32-bit). This is NOT textbook itoa, which typically only supports base 10 for signed integers.
2. **No radix validation:** Values outside [2,36] cause modulo by invalid radix (line 213) and potential OOB in digit computation (lines 214-218: `digit - 10` indexes into 'a'+...).
3. **No buffer overflow protection:** The caller MUST provide ≥33 bytes (32 digits + NUL for a 32-bit value in base 2, plus optional '-'). The function writes unchecked (line 217).
4. **Lowercase hex:** 'a'-'z' (line 218), not 'A'-'F'.

**Slice pattern:** NEW because this is a pure-output function with no input slice to parse. The shim must allocate or assume the caller-provided buffer is valid, call the core formatter, and write the result. Signature proposed: `core_fn = "NEW:format_i32_radix"` with behavior `fn(value: i32, radix: u32) -> Result<ArrayVec<u8, 33>, InvalidRadix>` (the core checks radix [2,36]; shim ignores the error and invokes UB if out of range, to preserve the C contract).

**Inout:** Only `string` is written; `value` and `radix` are read-only. The exact content depends on the radix and sign (see preserved_misfeature #1).

**Fuzz domain:** Must cover:
- **Sign+radix interaction:** Negative values with `radix==10` (signed path, lines 206-208) vs. `radix!=10` (unsigned cast, lines 209-211).
- **Boundary values:** `INT_MIN` (maximum digit count in base 10, potential overflow in negation handled by cast to unsigned), `0` (single '0'), `INT_MAX`.
- **Radix edges:** 2 (longest output), 10 (signed/unsigned branch), 16 (common hex), 36 (max valid), and invalid (1, 37) to expose the undefined behavior.
- **Digit ranges:** Ensure digits <10 and ≥10 both tested (lines 214-218).