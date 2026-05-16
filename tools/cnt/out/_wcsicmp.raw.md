```toml
[symbols._wcsicmp]
c_signature = "int __cdecl _wcsicmp(wchar_t const* _String1, wchar_t const* _String2)"
family = "wstring"
slice_pattern = "nul_in"
core_fn = "NEW:wstring::cmp_case_insensitive"
return_collapse = "i32_passthrough"
inout_effects = "none (read-only comparison)"
preserved_misfeature = "ASCII-only case folding: only [A-Z]→[a-z], no Unicode/locale awareness (lines 26-29). Returns raw wchar_t difference (line 32), NOT normalized to {-1,0,+1}"
fuzz_domain = ["equal_same_case", "equal_diff_case_ascii", "diff_before_nul", "diff_at_nul", "ascii_upper_lower_boundary", "non_ascii_no_fold", "empty_strings"]
```

## Rationale

**1. Return value:** The function returns the difference `chr1 - chr2` of the first differing wide characters after case folding (line 32), or `0` if strings are equal (line 37). This is the raw `wchar_t` arithmetic difference, not clamped to {-1, 0, +1} like some libc implementations. If characters differ even after case normalization, the pre-normalized difference is returned.

**2. Parameters written:** None. Both `_String1` and `_String2` are `const` and only read (lines 19-20, dereferenced at lines 24-25).

**3. NULL behavior:** No NULL guards exist. NULL input will crash on dereference at lines 24-25 (`*p1++`, `*p2++`).

**4. Deviation from textbook:** 
   - **ASCII-only case folding** (lines 26-29): Only `[A-Z]` maps to `[a-z]` via addition of `('a' - 'A')`. Non-ASCII characters (e.g., `À`, `Ü`, Cyrillic) are compared raw without case normalization. This is NOT locale-aware, NOT Unicode-aware. Textbook `wcsicmp` would typically use `towlower()` or locale facilities.
   - **Raw difference return** (line 32): Returns the full `wchar_t` subtraction, not the typical three-way {-1, 0, +1} of POSIX `wcscasecmp`.

**5. Test equivalence classes:**
   - `equal_same_case`: `L"abc"` vs `L"abc"` → 0
   - `equal_diff_case_ascii`: `L"AbC"` vs `L"aBc"` → 0 (fold matches)
   - `diff_before_nul`: `L"abc"` vs `L"abd"` → negative (differs at index 2)
   - `diff_at_nul`: `L"ab"` vs `L"abc"` → negative (`'\0' - 'c'`)
   - `ascii_upper_lower_boundary`: `L"Z"` vs `L"["` (fold `Z`→`z`, then `'z' - '['` ≠ 0), and `L"@"` vs `L"a"` (no fold on `@`)
   - `non_ascii_no_fold`: `L"Ü"` vs `L"ü"` → non-zero (raw difference, no fold)
   - `empty_strings`: `L""` vs `L""` → 0 (both NUL at index 0, line 34 breaks)

**New core signature:** `fn cmp_case_insensitive(s1: &[u16], s2: &[u16]) -> i32` — iterate both slices, fold each `u16` if in `[A-Z]` by adding 32, return signed difference of first mismatch or 0 if equal. Slices are NUL-terminated content (shim uses `nul_in` pattern).