```toml
[symbols.strnlen]
c_signature  = "size_t strnlen(const char *str, size_t count)"
family       = "string"
slice_pattern = "nul_in_maxlen"
core_fn      = "string::len_of_bounded"
return_collapse = "usize"
inout_effects = "none (read-only)"
preserved_misfeature = "NULL returns 0 instead of undefined behavior: `if(str == 0) return 0;` (line 7). Textbook strnlen has UB on NULL."
fuzz_domain  = ["NULL", "empty string", "string shorter than count", "string exactly count length", "string longer than count (no NUL within count)", "count==0"]
```

## Rationale

**1. Exact return value:**
- NULL → 0 (line 7: `if(str == 0) return 0;`)
- Non-NULL → min(strlen(str), count): loop at line 9 increments `s` while `*s && count`, then returns `s - str` (line 11)
- count==0 → 0 (loop never enters)
- String longer than count → count (loop exits when count reaches 0)

**2. Which parameters are written:**
- None. Read-only traversal. No output parameters.

**3. NULL behavior:**
- **Does NOT crash**: explicit guard `if(str == 0) return 0;` (line 7)
- Standard strnlen has UB on NULL; this is a defensive extension

**4. Deliberate deviation:**
- Line 7: `if(str == 0) return 0;` — NULL-safe behavior vs textbook UB
- Otherwise faithful: stops at NUL or count exhaustion, whichever comes first

**5. Minimal test inputs:**
- NULL ptr (exercises line 7)
- Empty string `""` with count>0 (immediate NUL)
- String shorter than count (NUL before count exhausted)
- String exactly count chars long (NUL at position count)
- String longer than count (count exhausted, no NUL seen)
- count==0 (loop never enters, returns 0)

**Core requirement:**
Needs `NEW:string::len_of_bounded(s: &[u8], max_len: usize) -> usize` — returns min(position of first 0 byte, max_len), or equivalently: `s.iter().take(max_len).position(|&b| b == 0).unwrap_or(max_len)`.