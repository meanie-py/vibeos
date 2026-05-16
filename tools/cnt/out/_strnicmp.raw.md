```toml
[symbols._strnicmp]
c_signature = "int _strnicmp(char const* _String1, char const* _String2, size_t _MaxCount)"
family = "string"
slice_pattern = "nul_in"
core_fn = "NEW:strnicmp"
return_collapse = "i32_passthrough"
inout_effects = "none (read-only comparison)"
preserved_misfeature = "ASCII-only case-folding: only [A-Z] → [a-z], no locale/Unicode (lines 24-27). Compares as *signed char* difference (line 30: `chr1 - chr2` where chr1/chr2 are `char`). Stops at first NUL in *either* string even if n remains (line 32-35: `chr1 == 0` breaks, implicitly chr2==0 also stops via chr1==chr2 path). Returns 0 if n chars compared without difference OR NUL reached (line 38)."
fuzz_domain = ["equal_within_n", "diff_case_insensitive_within_n", "diff_case_sensitive_within_n", "s1_shorter_than_n", "s2_shorter_than_n", "both_shorter_equal", "n_zero", "ascii_upper_lower_boundary", "non_ascii_no_fold", "signed_char_negative_diff"]
```

## Rationale

### 1. Exact return value
Lines 17-38: Returns `chr1 - chr2` (signed char difference, line 30) if case-insensitive comparison finds inequality within `_MaxCount` characters. Returns 0 if `_MaxCount` characters compared without difference (line 38), OR if a NUL terminator is encountered in either string before `_MaxCount` exhausted (lines 32-35: `chr1 == 0` breaks the loop, and the symmetric case where `chr2 == 0` also terminates via the `chr1 == chr2` branch at line 21 followed by the `chr1 == 0` check).

### 2. Parameters written
None. Both `_String1` and `_String2` are `const char*` and only read (lines 17-18: local pointers initialized; lines 22-23: dereferenced read-only).

### 3. NULL / zero-length / not-found behavior
- **NULL**: No explicit guard. Dereferencing NULL (line 22-23: `*p1++`, `*p2++`) will crash. This is standard undefined behavior, not a special case.
- **`_MaxCount == 0`**: Line 20: `while (remaining-- != 0)` will immediately fail (0 decrements to SIZE_MAX, but the post-decrement condition `!= 0` was already false before the decrement side-effect for the 0 case in the original value check semantics — actually, `remaining--` returns the old value, so `0-- != 0` is `false`). Loop body never executes, falls through to line 38 `return 0`. Thus `_strnicmp(s1, s2, 0) == 0` regardless of content.
- **Not-found/shorter strings**: Lines 32-35 handle NUL in either string: stops comparison early, returns 0.

### 4. Deliberate deviation from textbook libc
- **ASCII-only case folding** (lines 24-27): Only `[A-Z] → [a-z]` transformation. No locale awareness, no Unicode handling. Characters outside ASCII uppercase (e.g., `À`, `ñ`, any `char > 127`) are compared as-is, *without* case folding. Textbook `strnicmp` typically defers to `tolower()` which respects locale; this is a hard-coded ASCII-only fast path.
- **Signed char comparison** (line 30: `return chr1 - chr2`): `chr1` and `chr2` are `char` (line 18), which in ReactOS CRT is **signed**. Difference of two signed chars can overflow (e.g., `'\x7F' - '\x80'` = `255` in unsigned math, but as signed `char` arithmetic it's `127 - (-128) = 255` which overflows `int` range if interpreted as signed result — actually, the *promotion* to `int` happens before subtraction per C rules, so `(int)chr1 - (int)chr2` is the actual operation, yielding a valid `int`. However, the *sign* of the chars matters: comparing `'\xFF'` (−1 if signed) with `'\x01'` gives `−1 − 1 = −2`, whereas unsigned char comparison would give `255 - 1 = 254`. The key misfeature is that **signed char** semantics apply, not unsigned byte comparison like `memcmp`.

### 5. Minimal fuzz domain (input equivalence classes)
- `equal_within_n`: strings identical (case-insensitive) for at least `n` chars, no NUL before `n`.
- `diff_case_insensitive_within_n`: differ only in case within `n` chars (e.g., "aBc" vs "AbC", n=3 → 0).
- `diff_case_sensitive_within_n`: differ in a non-case way within `n` chars (e.g., "abc" vs "abd", n=3 → negative).
- `s1_shorter_than_n`: `_String1` has NUL before `_MaxCount` chars, `_String2` longer or equal at that point.
- `s2_shorter_than_n`: `_String2` NUL first.
- `both_shorter_equal`: both strings identical and terminate before `n`.
- `n_zero`: `_MaxCount == 0` (must return 0, lines 20+38).
- `ascii_upper_lower_boundary`: test `'A'-1`, `'Z'+1`, `'a'-1`, `'z'+1` (characters just outside the folding range, lines 24, 25) to confirm no folding.
- `non_ascii_no_fold`: e.g., `'\xC0'` (À) vs `'\xE0'` (à) — should compare as raw signed char difference, no folding.
- `signed_char_negative_diff`: e.g., `'\x01'` vs `'\xFF'` — if `char` is signed, `'\xFF'` is −1, so diff = `1 - (-1) = 2` (after int promotion). Contrast with hypothetical unsigned `1 - 255 = -254`. This pins the signed-char contract.

### New core needed
`NEW:strnicmp` with signature:
```
fn strnicmp(s1: &[u8], s2: &[u8]) -> i32
```
Behavior: Compare `s1` and `s2` byte-by-byte with ASCII-only case folding (`[A-Z] → [a-z]`), stopping at first NUL in either slice or end of shorter slice. Return the signed difference `(s1[i] as i8 as i32) - (s2[i] as i8 as i32)` of the first differing byte (after folding), or 0 if equal up to the stop point. The shim will pass `min(strlen(s1), strlen(s2), n)` as the slice length derived from the C pointers, ensuring NUL-stop semantics and `n` limit are both honored.