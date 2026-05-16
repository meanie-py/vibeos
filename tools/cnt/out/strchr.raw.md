```toml
[symbols.strchr]
c_signature = "char *strchr(const char *s, int c)"
family = "string"
slice_pattern = "nul_in"
core_fn = "NEW:find_char_or_nul"
return_collapse = "ptr_or_null_from_opt"
inout_effects = "none (read-only scan)"
preserved_misfeature = "Returns pointer to the terminating NUL when c==0 (line 'if(cc == 0) return (_TCHAR *)s;'), not NULL. This is actually standard C behavior but worth pinning since it's a special case distinct from not-found."
fuzz_domain = ["char found mid-string", "char found at start", "char not found (return NULL)", "c==0 (must return ptr to NUL)", "c with high bits set (truncated to char, e.g. 0x161 → 0x61)", "empty string (s points to NUL immediately)", "multibyte char values to verify _TCHAR(int) truncation"]
```

## Rationale

**Line-by-line contract extraction:**

1. **Exact return** (lines 7-13): Scans `s` until `*s == 0`. If any `*s == cc`, returns pointer to that position. After the loop (i.e., at the NUL terminator), if `cc == 0`, returns pointer to the NUL itself. Otherwise returns `NULL`. This means:
   - Found → pointer to first occurrence
   - Not found and `c != 0` → `NULL`
   - `c == 0` (searching for NUL) → pointer to the terminating NUL

2. **Which parameters written**: None. This is a pure search—no mutation of `s` or any side effects.

3. **NULL/zero-length behavior**: No NULL guard. If `s == NULL`, will dereference and crash (UB). If `s` points to an immediate NUL (`*s == 0` on first iteration), the loop exits immediately and falls through to the `if(cc == 0)` check. If `c == 0`, returns pointer to that NUL; otherwise returns `NULL`. This is well-defined (empty string is valid).

4. **Deviation from textbook**: The `if(cc == 0) return (_TCHAR *)s;` is actually standard C99/POSIX behavior (strchr must return a pointer to the NUL when searching for `'\0'`), but it's a special case worth documenting because:
   - The loop condition `while(*s)` means the NUL is NOT examined inside the loop.
   - The post-loop check explicitly handles `c == 0` as a distinct case.
   - This is load-bearing: textbook descriptions often say "returns pointer to first occurrence" without emphasizing the NUL is considered an occurrence of `'\0'`.

   Also note: `_XINT c` is cast to `_TCHAR cc = c`. Since `_TCHAR` is `char` (signed), high bits of `c` are truncated. For example, `strchr(s, 0x161)` searches for `0x61` (`'a'`). This is standard but must be tested.

5. **Fuzz domain** (branch coverage):
   - **Found cases**: mid-string, at start (exercises `if(*s == cc)` before any increment).
   - **Not found**: confirms return `NULL`.
   - **`c == 0`**: exercises the post-loop special case.
   - **High-bit truncation**: e.g., `c = 0x161` → `cc = 0x61`, ensuring we test the cast.
   - **Empty string** (`s = ""`): loop never enters, falls to `if(cc == 0)` check immediately.
   - **Multibyte/signed-char edge**: Negative `char` values (e.g., `c = -1` → `cc = 0xFF`) to verify signed/unsigned comparison semantics.

**New core function needed:**

`NEW:find_char_or_nul` with signature `fn(haystack: &[u8], needle: u8) -> Option<usize>`. Scans the slice for the first occurrence of `needle`. Returns `Some(index)` if found within the slice, `None` if not found. The shim will:
- Reconstruct `&[u8]` from the NUL-terminated `s`.
- Call `find_char_or_nul(slice, c as u8)`.
- If `Some(i)`, return `s.add(i)`.
- If `None` AND `(c as u8) == 0`, return `s.add(slice.len())` (pointer to the NUL).
- Otherwise return `NULL`.

This cleanly separates the core logic (scan for byte) from the C ABI peculiarity (returning NUL pointer when searching for `'\0'`).