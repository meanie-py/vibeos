```toml
[symbols.strncmp]
c_signature  = "int strncmp(const char * s1, const char * s2, size_t n)"
family       = "string"
slice_pattern = "nul_in"
core_fn      = "string::ncmp"
return_collapse = "i32_passthrough"
inout_effects = "none (read-only)"
preserved_misfeature = "Returns signed-char difference (*s1 - *s2) as int, NOT the standard -1/0/+1 normalization. _TCHAR is signed char. Early-return on n==0 before any dereference."
fuzz_domain  = ["n==0 (no deref)", "equal prefixes shorter than n", "equal strings exactly n long", "differ at position < n", "differ at position 0", "one string ends (NUL) before n", "both strings same through NUL within n", "negative vs positive char values (signed difference)"]
```

## Rationale

### 1. Exact return value (lines 13-19)
- **n==0**: Returns `0` immediately (line 11) — **no dereference**.
- **Match through NUL or n exhausted**: Returns `0` (line 19).
- **Mismatch at position i < n**: Returns `*s1 - *s2` **as signed char difference** (line 15), where both operands are `_TCHAR` (signed `char`). The subtraction produces an `int` via integer promotion, but the VALUES subtracted are signed-char interpretations. This can produce any int in roughly [-255, +255], NOT the standard -1/0/+1.

### 2. Parameters written
**None.** Both `s1` and `s2` are `const`; the function is read-only.

### 3. NULL / zero-length behavior (line 11)
```c
if(n == 0) return 0;
```
**Explicit guard**: If `n == 0`, function returns `0` **before any pointer dereference**. This makes `strncmp(NULL, NULL, 0)` safe (returns 0), and `strncmp(p1, p2, 0)` always returns 0 regardless of pointer validity.

### 4. Deviation from textbook libc (line 15)
```c
if(*s1 != *s2 ++) return *s1 - *-- s2;
```
Standard `strncmp` per POSIX/C11 returns an integer *"less than, equal to, or greater than zero"* — typically normalized to -1/0/+1. **ReactOS returns the raw signed-char difference.** Since `_TCHAR` is `char` (signed on most platforms), comparing e.g. `"\xFF"` vs `"\x01"` yields `0xFF - 0x01 = -2` (interpreting both as signed chars: -1 - 1 = -2), not -1.

This deviation is **load-bearing** for any caller depending on the magnitude or exact value of the difference.

### 5. Fuzz domain (minimal equivalence classes)
- **n==0**: exercises the early-return guard (no deref).
- **Equal prefixes shorter than n**: strings match for first k < n chars, then both hit NUL → return 0 (line 19 via break at line 17).
- **Equal strings exactly n long**: compare n chars, all match, loop exits via `--n != 0` becoming false → return 0.
- **Differ at position < n**: mismatch before NUL or n exhausted → return signed difference.
- **Differ at position 0**: immediate mismatch on first iteration.
- **One string ends (NUL) before n**: e.g. s1="ab\0", s2="abcd", n=4 → s1 hits NUL first, comparison stops, return 0 if s2 also at same content.
- **Both strings same through NUL within n**: e.g. "foo" vs "foo", n=10 → break on NUL (line 17), return 0.
- **Negative vs positive char values**: e.g. s1[0]=-1 (0xFF), s2[0]=1 → return -2 (signed difference), NOT -1. This tests the signed-char arithmetic misfeature.

**Core function needed**: `string::ncmp` with signature `fn ncmp(s1: &[u8], s2: &[u8]) -> i32` that performs **signed-char comparison** (reinterpret bytes as `i8` for subtraction) and returns the **raw difference**, NOT normalized. If this doesn't exist, specify `NEW:string::ncmp_signed_diff` — compare up to `min(s1.len(), s2.len())` bytes as signed, return first `(s1[i] as i8) - (s2[i] as i8)` nonzero difference or 0 if all equal.