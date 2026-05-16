```toml
[symbols._wcsnicmp]
c_signature = "int __cdecl _wcsnicmp(wchar_t const* _String1, wchar_t const* _String2, size_t _MaxCount)"
family = "wstring"
slice_pattern = "ptr_len"
core_fn = "NEW:wstring::nicmp"
return_collapse = "i32_passthrough"
inout_effects = "none (read-only comparison)"
preserved_misfeature = "case-folding applies ONLY to ASCII 'A'-'Z' → 'a'-'z' (lines 28-31), NOT full Unicode case-folding; returns raw wchar_t difference (line 33) as int (potential truncation on platforms where wchar_t > 16-bit, though Windows wchar_t is 16-bit); compares at most _MaxCount characters OR until first NUL in either string (lines 22-38)"
fuzz_domain = ["both_empty", "n_zero", "equal_within_n", "equal_at_n_boundary", "diff_ascii_case_only", "diff_non_ascii", "diff_beyond_n", "nul_before_n_s1", "nul_before_n_s2", "nul_at_n", "non_ascii_uppercase", "wchar_diff_overflow"]
```

## Rationale

**1. Exact return value:** Returns `chr1 - chr2` (line 33) as `int` where `chr1` and `chr2` are `wchar_t`. If strings match for the first `_MaxCount` characters OR both reach NUL within that span, returns 0 (line 40). The comparison is **case-insensitive** but only for ASCII A-Z (lines 28-31). The function post-decrements `remaining` (line 22) starting from `_MaxCount`, so it compares at most `_MaxCount` characters.

**2. Parameters written:** None. Both `_String1` and `_String2` are `const` pointers, used read-only.

**3. NULL/zero-length behavior:** 
   - `_MaxCount == 0`: The `while` condition `remaining-- != 0` (line 22) immediately fails (0 post-decrements to SIZE_MAX but the comparison against 0 happens first), so function returns 0 immediately without dereferencing either pointer.
   - NULL pointers: **Not guarded**. If either pointer is NULL and `_MaxCount > 0`, undefined behavior (dereference at line 23/24).

**4. Deliberate deviation from textbook:**
   - **ASCII-only case-folding**: Lines 28-31 apply case normalization exclusively to ASCII uppercase 'A'-'Z' (convert to 'a'-'z'). Unicode uppercase characters (e.g., Cyrillic, Greek) are compared raw, NOT case-folded. This is a Windows CRT idiom (the `_i` in `_wcsnicmp` means "case-insensitive" but traditionally only for ASCII in many implementations).
   - **Raw wchar_t difference**: Returns `chr1 - chr2` directly as `int` (line 33). On platforms where `wchar_t` is wider than `int` (e.g., 32-bit), this could truncate, but Windows `wchar_t` is 16-bit (`unsigned short`), so practically safe within `int` range.
   - **Boundary behavior**: Comparison stops when **either** (a) `_MaxCount` characters have been compared, OR (b) a NUL is encountered in **either** string (line 36: `if (chr1 == 0)` after confirming equality). If strings differ before NUL/n, returns immediately (line 33).

**5. Minimal test inputs** (fuzz_domain):
   - `both_empty`: `"", "", n>0` → 0
   - `n_zero`: `"any", "any", 0` → 0 (no dereference)
   - `equal_within_n`: `"hello", "hello", 3` → 0
   - `equal_at_n_boundary`: `"abc", "abc", 3` → 0 (NUL at position 3, but already matched 3 chars)
   - `diff_ascii_case_only`: `"ABC", "abc", 3` → 0 (case-fold ASCII)
   - `diff_non_ascii`: `"Ω", "ω", 1` → non-zero (Greek omega, no case-fold)
   - `diff_beyond_n`: `"abcX", "abcY", 3` → 0 (differ at pos 3, but n=3 stops before)
   - `nul_before_n_s1`: `"ab\0c", "abcd", 4` → 0 (s1 NUL at pos 2, stops)
   - `nul_before_n_s2`: `"abcd", "ab\0c", 4` → 0 (s2 NUL at pos 2, stops)
   - `nul_at_n`: `"ab", "ab", 3` → 0 (NUL at pos 2, n=3)
   - `non_ascii_uppercase`: `"Ä", "ä", 1` → non-zero (Latin A-umlaut, raw compare)
   - `wchar_diff_overflow`: Pair of wide chars whose difference exceeds `INT_MAX` (academic on Windows where `wchar_t` is 16-bit, but tests return-value truncation logic if ported).

**Proposed new core signature (one-line):**  
`wstring::nicmp(s1: &[u16], s2: &[u16]) -> i32` — compares up to `min(s1.len(), s2.len())` wide chars with ASCII-only case-folding ('A'-'Z' → 'a'-'z'), returning signed difference of first mismatch or 0 if equal/both-NUL.