```toml
[symbols.mbstowcs]
c_signature = "size_t mbstowcs(wchar_t *wcstr, const char *mbstr, size_t count)"
family = "string"
slice_pattern = "nul_in"
core_fn = "NEW:mbstowcs_nt"
return_collapse = "usize"
inout_effects = "wcstr written with converted wide chars (NOT NUL-terminated by this function); length determined by mbstr NUL. count is max wchar_t elements (destination capacity), NOT bytes."
preserved_misfeature = "When wcstr==NULL, returns the required wchar_t count (Size/sizeof(wchar_t)) WITHOUT converting — a query mode (lines 48-53). RtlMultiByteToUnicodeN does NOT NUL-terminate wcstr (the Rtl function writes only converted chars). count is interpreted as wchar_t capacity, but passed as bytes to Rtl (count*sizeof(WCHAR), line 56). On NT API failure, returns (size_t)-1 (lines 60-61), not a well-defined error like textbook (size_t)-1 with errno."
fuzz_domain = ["wcstr==NULL (query mode)", "wcstr!=NULL with count==0", "empty mbstr (strlen==0)", "mbstr with count < required wchars", "mbstr with count >= required wchars", "non-ASCII mbstr requiring multi-byte conversion", "NT API failure simulation (Status!=SUCCESS)"]
```

## Rationale

**Exact return value** (lines 48-53, 60-62):
- If `wcstr == NULL`: calls `RtlMultiByteToUnicodeSize` to compute required wide-char count, returns `Size / sizeof(wchar_t)` — a query mode that computes without converting.
- If `wcstr != NULL`: calls `RtlMultiByteToUnicodeN`, which writes converted wide chars. On success, returns `Size / sizeof(wchar_t)` (the number of wchar_t written). On `!NT_SUCCESS(Status)`, returns `(size_t)-1`.

**Which parameters written** (lines 55-58):
- `wcstr` is written with converted wide characters up to `count` wchar_t elements (passed as `count * sizeof(WCHAR)` bytes to the Rtl function). **Crucially**, `RtlMultiByteToUnicodeN` does NOT NUL-terminate the output — it writes only the converted characters. The caller must handle termination if needed.
- `mbstr` is read as a NUL-terminated string (via `strlen`, line 45).

**NULL / zero-length behavior** (lines 47-53):
- `wcstr == NULL`: guard present (line 47), enters query mode — calls `RtlMultiByteToUnicodeSize` to compute required size without writing, returns the wchar_t count.
- `mbstr == NULL`: not guarded in `mbstowcs` itself (assumes valid pointer), but `strlen(NULL)` would crash per prior manifest entry (`strlen` with NULL is UB in this codebase despite its own NULL→0 behavior — here `strlen` is called directly on `mbstr` line 45).
- `count == 0` with `wcstr != NULL`: `RtlMultiByteToUnicodeN` would be called with 0-byte destination, writes nothing, returns 0.

**Deliberate deviation** (lines 47-53, 56, 60-61):
- **Query mode when `wcstr==NULL`**: textbook `mbstowcs` typically does not define NULL behavior; here it computes required size without converting (lines 48-53).
- **`count` is wchar_t capacity, not bytes**: passed as `count * sizeof(WCHAR)` to Rtl (line 56), but the return value is in wchar_t units (`Size / sizeof(wchar_t)`). This is correct for the function signature, but the multiplication is explicit in the implementation.
- **No NUL termination by `RtlMultiByteToUnicodeN`**: the NT function writes only converted characters, not a terminating NUL. The caller must ensure termination if a NUL-terminated wide string is required.
- **Error return**: returns `(size_t)-1` on NT failure (line 61), not a portable error code with `errno` set.

**Minimal fuzz domain** (questions 1-4 coverage):
1. `wcstr==NULL` → query mode (line 47 guard)
2. `wcstr!=NULL, count==0` → 0-byte write, return 0
3. `strlen(mbstr)==0` → empty input, should return 0 wchar_t
4. `count < required` → partial conversion (Rtl truncates to `count*sizeof(WCHAR)` bytes)
5. `count >= required` → full conversion
6. Non-ASCII multi-byte input → exercises `RtlMultiByteToUnicodeN` conversion logic
7. Simulated NT failure → returns `(size_t)-1`

**NEW core signature**: `core_fn = "NEW:mbstowcs_nt"` should have signature:
```
fn mbstowcs_nt(mbstr_slice: &[u8]) -> Result<(usize, impl FnOnce(&mut [u16])), ()>
```
Returns `Ok((required_wchar_count, converter_closure))` where the closure writes converted wide chars into the provided slice (no NUL termination), or `Err(())` on NT API failure. The shim calls this, and if `wcstr==NULL` returns the count; else invokes the closure on the `wcstr` slice and returns the written count.