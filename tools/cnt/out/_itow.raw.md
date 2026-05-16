```toml
[symbols._itow]
c_signature = "wchar_t* _itow(int value, wchar_t *string, int radix)"
family = "radix"
slice_pattern = "itow_passthrough"
core_fn = "NEW:wstring_format_i32"
return_collapse = "ret_dst"
inout_effects = "if string!=NULL: writes NUL-terminated radix representation (lowercase hex a-z for digits>9), negated if value<0 && radix==10 else reinterpreted as unsigned; ALWAYS returns the dst pointer unchanged (even if NULL)"
preserved_misfeature = "Lines 133-135: _itow is a trivial passthrough to _ltow, which exhibits three deviations: (1) NULL dst is accepted without crash—function computes the string but doesn't write it, still returns NULL (lines 170-172); (2) negative values are negated ONLY for radix==10, else reinterpreted as unsigned long (lines 162-166); (3) no radix validation—invalid radix causes undefined modulo/division behavior"
fuzz_domain = ["value={INT_MIN, -42, 0, 42, INT_MAX}", "radix={2, 10, 16, 36, 1_invalid, 37_invalid}", "string={NULL, valid_buf[33]}", "cross: negative_value × radix!=10 (unsigned reinterpret)", "cross: NULL × any_value (returns NULL without crash)"]
```

## Rationale

**Source lines 133-135** show `_itow` is a trivial delegation to `_ltow(int → long)`, so its behavioral contract is inherited entirely from `_ltow`.

**Exact return value (lines 170-172, question 1):** `_ltow` unconditionally returns the `string` parameter verbatim—even if NULL. The caller receives back exactly what they passed in (lines 133-135 confirm the passthrough).

**Output effects (lines 162-169 + 170-172, question 2):** The function builds a NUL-terminated wide-character string in a local 33-wchar_t buffer (line 156), then **only if `string != NULL`** does it `memcpy` the result including the NUL terminator (lines 170-172). If NULL, the computation still happens (consuming CPU cycles) but nothing is written. Digits ≥10 use lowercase letters 'a'..'z' (line 161).

**NULL/empty behavior (lines 170-172, question 3):** The explicit guard `if (string != NULL)` prevents the memcpy when dst is NULL—no crash, but the function still returns NULL. This is a load-bearing deviation: the caller can pass NULL to "dry-run" the conversion (compute the length implicitly) or just ignore the result.

**Sign handling misfeature (lines 162-166, question 4):** Negation (`val = -value`) occurs **only when `value < 0 && radix == 10`**. For any other radix, a negative `int` is cast to `unsigned long` (line 165: `val = value`), reinterpreting the two's-complement representation as a large positive unsigned integer. This is deliberate in the ReactOS source—distinct from textbook behavior where signed/unsigned interpretation might vary or error.

**Radix validation absence (question 4 continued):** Unlike the `_s` variants which validate `2 ≤ radix ≤ 36`, `_ltow` (and thus `_itow`) perform **no radix check**. Invalid radix (e.g., 1, 0, 37) leads to undefined modulo and division operations (lines 158-161), likely looping forever or producing garbage. This is the second key misfeature.

**Proposed new core (`NEW:wstring_format_i32`):** A signature like `fn wstring_format_i32(value: i32, radix: u32, negate_if_negative_base10: bool) -> WideString` that encodes the exact lines 162-169 logic: produce lowercase ASCII digits/letters into a heap-free or fixed buffer, handling the conditional negation. The shim would then either write to the user buffer or discard, returning the input pointer.

**Differential test fuzzing (question 5):**
- **INT_MIN**: edge case for negation overflow (though cast to `unsigned long` masks it).
- **Negative × radix≠10**: must confirm unsigned reinterpretation (e.g., -1 in base 16 → "ffffffff" on 32-bit).
- **radix ∈ {2, 10, 16, 36}**: exercise digits vs letters boundary.
- **radix invalid {1, 37}**: document undefined behavior (test harness may skip or expect timeout).
- **NULL dst × any value**: must return NULL without segfault, proving the guard works.
- **Valid buffer**: ensure NUL termination and correct digit order (lines 158-161 build the string backwards via `*--pos`).

The fuzz domain must include the NULL case to pin the "silent no-op" behavior, and the negative-non-decimal cross to pin the unsigned reinterpretation—both are the core deviations from a hypothetical "safe" libc.