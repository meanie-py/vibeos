```toml
[symbols.wcstombs]
c_signature = "size_t wcstombs(char *mbstr, const wchar_t *wcstr, size_t count)"
family = "wstring"
slice_pattern = "nul_in + dst_optional"
core_fn = "NEW:wstring::to_multibyte"
return_collapse = "size_or_neg1_on_err"
inout_effects = "If mbstr non-NULL: writes up to count bytes (multibyte encoding of wcstr), NO NUL termination guaranteed. If mbstr NULL: no mutation, returns required size."
preserved_misfeature = "Lines 39-44: NULL dst is query mode (returns required size without terminating NUL). Lines 46-53: writes up to count bytes with NO automatic NUL termination—caller must ensure buffer sized for content+NUL or manually terminate. Conversion calls RtlUnicodeToMultiByteN which may fail (Status check line 51), returning -1 cast to size_t (SIZE_MAX on most platforms)."
fuzz_domain = ["mbstr=NULL (query mode)", "mbstr valid, count exact for content (no NUL room)", "count=0 with valid mbstr", "wcstr empty L\"\"", "wcstr with multibyte-expanding chars", "conversion failure (invalid wchar sequence)", "count larger than needed"]
```

## Rationale

**Signature & behavior (lines 28-54):** `wcstombs` converts a NUL-terminated wide string (`wcstr`) to multibyte encoding, writing up to `count` bytes into `mbstr`. Returns the number of bytes written (excluding any NUL), or `(size_t)-1` on conversion error.

**NULL dst query mode (lines 39-44):** If `mbstr == NULL`, the function becomes a size-query: it computes the required output size via `RtlUnicodeToMultiByteSize` and returns that byte count **without writing anything**. This is a standard libc feature but must be explicitly tested.

**NO automatic NUL termination (lines 46-53):** When `mbstr` is non-NULL, `RtlUnicodeToMultiByteN` is called with `count` as the destination buffer size. The function writes **up to `count` bytes** and returns the actual bytes written in `Size`. Critically, there is **no code appending a NUL terminator**—if `count` equals the exact multibyte length of `wcstr`, the output will **not** be NUL-terminated. This is textbook `wcstombs` behavior (caller must allocate `count` large enough for content + NUL and manually terminate if needed), but the lack of termination logic is the load-bearing detail.

**Error handling (lines 50-52):** If `RtlUnicodeToMultiByteN` fails (e.g., invalid wide character sequence), the function returns `-1` cast to `size_t`, which is `SIZE_MAX` on typical platforms.

**Input length (line 33):** The source length is computed as `wcslen(wcstr) * sizeof(WCHAR)` bytes—conversion stops at the NUL terminator of `wcstr`, but the NUL itself is **not** included in the conversion (line 33 uses `Length * sizeof(WCHAR)` where `Length` is the character count excluding NUL).

**Preserved misfeature:** The dual-mode NULL-dst query and the no-auto-terminate on exact-fit buffer are both standard, but the `-1` error cast to `size_t` (resulting in `SIZE_MAX`) and the exact boundary behavior (no NUL if `count` equals content length) are the critical behavioral pins.

**Core function needed:** `NEW:wstring::to_multibyte` with signature `fn to_multibyte(dst: Option<&mut [u8]>, src: &[u16]) -> Result<usize, ()>`. When `dst` is `None`, returns required size; when `Some`, writes up to slice length and returns bytes written. The shim must handle the `NULL` pointer → `None` mapping, reconstruct `wcstr` via NUL-scan, call the core, and collapse `Err(())` → `(size_t)-1`.

**Fuzz domain:** Must cover NULL-dst query mode, exact-fit buffer (no room for NUL), zero-count, empty source, multibyte-expanding characters, conversion failure paths, and over-sized buffers to exercise all branches and the no-terminate edge case.