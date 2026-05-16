```toml
[symbols.strrchr]
c_signature = "char *strrchr(const char *s, int c)"
family = "string"
slice_pattern = "nul_in"
core_fn = "NEW:find_last_char"
return_collapse = "ptr_or_null_from_opt"
inout_effects = "none (read-only search)"
preserved_misfeature = "c is truncated to char via `_TCHAR cc = c` (line tcsrchr.h:5), matching only the low byte. The NUL terminator IS a valid search target: if c==0, returns pointer to the NUL (lines 12-13), NOT NULL. Scans entire string even after finding matches, keeping last occurrence."
fuzz_domain = ["c in range, found once", "c in range, found multiple times (verify LAST returned)", "c in range, not found (ret NULL)", "c==0 (ret ptr to NUL)", "c with high bits set (verify truncation to char)", "empty string with c==0 (ret ptr to empty NUL)", "empty string with c!=0 (ret NULL)"]
```

## Rationale

**1. Exact return value:**
- Lines 7-11: Loop scans the entire string; whenever `*s == cc`, updates `sp` to current position (keeps overwriting, so last match wins).
- Line 12-13: Special case — if `cc == 0` (search char is NUL), returns pointer to the terminating NUL itself (`sp = s` after loop exits at NUL).
- Line 15: Returns `sp` (cast back to non-const); `sp` is NULL only if no match found AND `c != 0`.

**2. Which parameters are written:**
None. This is a read-only search function.

**3. NULL/zero-length/not-found behavior:**
- NULL input: Not guarded — would dereference and crash (line 7 `while(*s)`).
- Empty string (`*s == 0` immediately): If `c == 0`, returns pointer to that NUL (line 12-13). Otherwise `sp` remains NULL (line 6 init), returned at line 15.
- Not found: Returns NULL (line 15, `sp` never updated from line 6 init).

**4. Deliberate deviations:**
- **Truncation of `c`**: Line 5 `_TCHAR cc = c;` — the `int` parameter is truncated to `char` (signed on ReactOS). High bits ignored. This matches standard `strrchr`, but must be explicit.
- **NUL is searchable**: Lines 12-13 explicitly handle `cc == 0` by returning a pointer to the NUL terminator, NOT NULL. This is standard behavior but crucial to pin.
- **Last occurrence**: The loop (lines 7-11) scans the entire string even after finding a match, continuously overwriting `sp`. This ensures the *last* occurrence is returned, as required by `strrchr` semantics.

**5. Minimal test equivalence classes:**
Must cover: (a) character found once, (b) character found multiple times (verify last, not first), (c) character not found, (d) `c==0` (NUL search), (e) `c` with high bits set to verify truncation, (f) empty string with both `c==0` and `c!=0`.

**Proposed core signature:**
`fn find_last_char(s: &[u8], needle: u8) -> Option<usize>` — returns byte offset of the last occurrence of `needle` in `s`, or `None`. Slice is content-only (NUL excluded). Shim adds NUL-check manually: if `c == 0`, return `ptr.add(len)` (pointer to NUL); else call core and offset pointer or return NULL.