```toml
[symbols.memchr]
c_signature  = "void* __cdecl memchr(const void *s, int c, size_t n)"
family       = "mem"
slice_pattern = "ptr_len"
core_fn      = "mem::chr"
return_collapse = "ptr_or_null_from_opt"
inout_effects = "none (read-only scan)"
preserved_misfeature = "none (textbook behavior: scan n bytes for first match of c cast to unsigned char, return pointer or NULL)"
fuzz_domain  = ["n=0 (immediate return NULL)", "c present at offset 0", "c present at offset n-1", "c not present", "c=0 (NUL search)", "c with high bit set (cast to unsigned char matters)"]
```

## Rationale

**1. Exact return value:**
- Lines 10-14: Returns pointer to first byte matching `c` within `n` bytes, or `0` (NULL) if not found or `n==0`
- Line 9: Guard `if (n)` means `n==0` immediately returns `0` without dereferencing `s`
- Line 12: Match test `*p++ == c` — the `int c` is implicitly cast to `char` for comparison with dereferenced byte
- Line 13: Returns `(void *)(p-1)` — pointer to the matching byte (since `p` was post-incremented)

**2. Parameters written:**
None. Pure read-only search (line 11: `const char *p`).

**3. NULL/zero-length behavior:**
- Line 9: `if (n)` guard means `n==0` returns NULL without touching `s` — **does NOT crash on `memchr(NULL, c, 0)`**
- If `n > 0`, `s` is dereferenced (line 12), so `memchr(NULL, 'x', 1)` would crash (textbook undefined behavior)

**4. Deviation from textbook:**
None. This is textbook `memchr`:
- Scans exactly `n` bytes (not NUL-terminated)
- The `int c` parameter is converted to `unsigned char` via implicit cast in comparison (standard behavior)
- Returns pointer to match or NULL
- The loop structure (lines 11-14) matches standard: post-increment `*p++`, decrement `--n`, test `!= 0`

**5. Differential test must cover:**
- `n=0` → NULL (line 9 branch)
- `c` at first position (offset 0) → immediate match (line 12-13, first iteration)
- `c` at last position (offset `n-1`) → match on final iteration (line 14 boundary)
- `c` not present → exhaust loop, return NULL (line 14 → line 16)
- `c=0` (NUL byte search) → must find embedded NULs
- `c` with values 128-255 (high bit set) → tests unsigned char conversion (the cast matters for correct comparison with byte values > 127)

**Core mapping:**
`mem::chr` should be a byte-slice search returning `Option<usize>` (offset of first match). The shim reconstructs the slice via `ptr_len` (`slice::from_raw_parts(s as *const u8, n)`), calls `mem::chr(slice, c as u8)`, and collapses `Some(offset)` → `s.add(offset)` or `None` → `null_mut()` via `ptr_or_null_from_opt`.