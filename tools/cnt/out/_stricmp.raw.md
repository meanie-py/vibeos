```toml
[symbols._stricmp]
c_signature  = "int __cdecl _stricmp(char const* _String1, char const* _String2)"
family       = "string"
slice_pattern = "nul_in"
core_fn      = "NEW:string::cmp_case_insensitive"
return_collapse = "i32_passthrough"
inout_effects = "none (both params read-only)"
preserved_misfeature = "ASCII-only case folding: only [A-Z]→[a-z] (lines 24-27). NOT Unicode-aware, NOT locale-sensitive. Signed-char difference after fold (line 29): returns (chr1 - chr2) as signed char arithmetic, matching strcmp family convention but potentially surprising on platforms where char is unsigned by default."
fuzz_domain  = ["both_empty", "equal_same_case", "equal_diff_case_ascii", "diff_before_fold", "diff_after_fold", "prefix_relation", "non_ascii_high_bit", "embedded_nul"]
```

## Rationale

**1. Exact return value:** Returns `(chr1 - chr2)` as signed char difference after ASCII case-folding both characters (lines 24-29). Returns 0 if strings are equal after folding (line 35). The function terminates when either string's NUL is encountered (lines 30-33): if characters match and `chr1 == 0`, both have terminated together; if they differ before termination, the difference is returned immediately.

**2. Parameters written:** Neither parameter is modified — both are `const` and only dereferenced for reading (lines 16-17, 20-21).

**3. NULL behavior:** No NULL guards present. Passing NULL will dereference and crash (line 20 `*p1++`).

**4. Deliberate deviation from textbook:** 
   - **ASCII-only folding:** Lines 24-27 implement manual `[A-Z] → [a-z]` mapping, NOT `tolower()`. This ignores locale and handles ONLY ASCII uppercase. Non-ASCII characters (e.g., `À`, `Ñ`, `Ø`) remain unchanged, making `_stricmp("Ñ", "ñ")` non-equal.
   - **Signed-char arithmetic:** Line 29 `return chr1 - chr2` uses plain `char` (which is signed in ReactOS `_TCHAR` context), matching `strcmp`'s signed behavior. On platforms where `char` is unsigned, this would behave differently, but ReactOS treats it as signed.

**5. Input equivalence classes (fuzz_domain):**
   - `both_empty`: `"", ""` → 0
   - `equal_same_case`: `"abc", "abc"` → 0
   - `equal_diff_case_ascii`: `"AbC", "aBc"` → 0 (exercises lines 24-27)
   - `diff_before_fold`: `"a", "A"` → 0 (fold makes equal), but `"a", "B"` → negative (diff after fold, line 29)
   - `diff_after_fold`: `"a", "b"` → negative (already lowercase, line 29)
   - `prefix_relation`: `"abc", "ab"` → positive (longer string has more content after shorter's NUL)
   - `non_ascii_high_bit`: `"à", "À"` → non-zero (NO fold for non-ASCII, exercises that lines 24-25 conditions fail)
   - `embedded_nul`: not applicable (NUL terminates, line 30)

**New core needed:** `string::cmp_case_insensitive(a: &[u8], b: &[u8]) -> i32` — byte-wise comparison with ASCII `[A-Z]→[a-z]` fold only, returning signed-char difference `(a_byte as i8 - b_byte as i8) as i32` of first mismatch after folding, or 0 if equal length and all match after fold. Must NOT use Unicode/locale folding.