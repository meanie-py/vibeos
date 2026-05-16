```toml
[symbols.memcmp]
c_signature  = "int __cdecl memcmp(const void *s1, const void *s2, size_t n)"
family       = "mem"
slice_pattern = "ptr_len"
core_fn      = "mem::cmp"
return_collapse = "i32_passthrough"
inout_effects = "none (read-only comparison)"
preserved_misfeature = "unsigned char comparison: `return (*--p1 - *--p2)` casts bytes to unsigned char, then subtracts as signed int. This yields values outside [-1,0,1] range (e.g., 0xFF vs 0x00 → 255, not -1). Standard libc typically clamps to {-1,0,1}, but ReactOS returns raw unsigned-byte difference."
fuzz_domain  = ["n==0 (return 0)", "equal sequences", "differ at first byte", "differ at last byte", "high-byte difference (0xFF vs 0x00) → verify return == 255", "n < actual buffer lengths", "n == buffer lengths"]
```

## Rationale

### 1. Exact return value
- **Lines 10-14**: If `n != 0`, iterate comparing bytes as `unsigned char`. On first mismatch, `return (*--p1 - *--p2)` — the `unsigned char` values are promoted to `int`, subtracted, yielding the raw difference (range [-255, 255]).
- **Line 16**: If `n == 0` or all bytes match, `return 0`.

### 2. Parameters written
- **Read-only**: Both `s1` and `s2` are `const void *`; no writes occur.

### 3. NULL / zero-length behavior
- **Line 9**: Guard `if (n != 0)` prevents dereferencing when `n == 0` → immediately `return 0`. No NULL check; calling with NULL and n>0 would crash (undefined per standard C).

### 4. Deliberate deviation from textbook libc
- **Line 13**: `return (*--p1 - *--p2)` returns the **raw unsigned-byte difference as a signed int**. Textbook `memcmp` typically returns -1/0/1 (or implementation-defined negative/zero/positive). Here, differing bytes like `0xFF` vs `0x00` yield `255`, not `-1`. This is the load-bearing misfeature: the exact numeric difference, not a normalized sign indicator.

### 5. Minimal fuzz domain
- `n == 0` → pins the guard, returns 0 without dereference.
- Equal sequences → exercises full loop, returns 0.
- Differ at first/last byte → tests early-exit vs full iteration.
- **High-byte difference (e.g., `{0xFF}` vs `{0x00}`)** → critical to verify `return == 255` (unsigned arithmetic), distinguishing this implementation from clamping variants.
- `n` spanning partial/full buffers → boundary coverage.