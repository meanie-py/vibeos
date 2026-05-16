```toml
[symbols.mbtowc]
c_signature = "int mbtowc(wchar_t *wchar, const char *mbchar, size_t count)"
family = "string"
slice_pattern = "ptr_len"
core_fn = "NEW:mb_to_wc"
return_collapse = "i32_passthrough"
inout_effects = "*wchar written with decoded wchar_t on success (ret>0); unchanged on error (ret==-1)"
preserved_misfeature = "NULL checks short-circuit to 0 for BOTH wchar and mbchar (lines 18-19, 21-22), not the textbook reset-state behavior for wchar==NULL. Space-mapping guard (line 28-29): returns -1 if RtlAnsiCharToUnicodeChar produces L' ' but input mbarr[0]!=' ', treating this as invalid conversion. Return is signed difference of PUCHAR pointers (line 32), typically 1 for single-byte, 2 for lead-byte sequences up to MB_CUR_MAX==2."
fuzz_domain = ["mbchar==NULL", "wchar==NULL", "count==0", "valid ASCII char count>=1", "valid lead-byte sequence count==2", "invalid sequence triggering space-guard", "count<needed (truncation)"]
```

## Rationale

**Function contract from source:**
`mbtowc` converts a multibyte character sequence starting at `mbchar` (up to `count` bytes) into a single wide character written to `*wchar`, returning the number of consumed bytes or -1 on error.

**Five questions answered:**

1. **Return value (lines 18-19, 21-22, 28-32):**
   - If `mbchar==NULL` OR `wchar==NULL`: return `0` (both short-circuit, line 18-22).
   - If `RtlAnsiCharToUnicodeChar` produces `L' '` but `mbarr[0]!=' '`: return `-1` (line 28-29, the invalid-conversion sentinel).
   - Otherwise: return `(int)(mbs - mbarr)` (line 32), the number of bytes consumed (1 for ASCII, up to 2 for lead-byte per `MB_CUR_MAX==2`).

2. **Which parameters written (line 31):**
   - `*wchar` is assigned `wc` (the decoded wide character) on success (when return is positive).
   - On error (return -1), `*wchar` is NOT written (guard exits early line 29).

3. **NULL behavior (lines 18-22):**
   - `if (mbchar == NULL) return 0;` (line 18-19).
   - `if (wchar == NULL) return 0;` (line 21-22).
   - Neither dereferences NULL; both return 0. Textbook `mbtowc(NULL, ...)` resets shift-state and returns nonzero if stateful, but this implementation ignores that and returns 0 for both NULL cases.

4. **Deviation from textbook (lines 18-22, 28-29, 10):**
   - **NULL wchar:** textbook behavior is state-reset query (return nonzero if encoding is stateful); here it's a no-op returning 0 (line 21-22).
   - **Space-mapping guard (line 28-29):** `if (wc == L' ' && mbarr[0] != ' ') return -1;` treats a space result from a non-space input as invalid, likely detecting `RtlAnsiCharToUnicodeChar`'s fallback behavior for malformed sequences.
   - **MB_CUR_MAX==2 (line 10):** hard-coded for DBCS (double-byte character sets), not UTF-8's up-to-4.
   - **memcpy guard (line 24):** copies `min(count, sizeof mbarr)` (i.e., min(count,2)), allowing partial input for lead-byte detection.

5. **Fuzz domain (covering all branches):**
   - `mbchar==NULL` (line 18-19 exit).
   - `wchar==NULL` (line 21-22 exit).
   - `count==0` (memcpy copies 0, `mbarr` stays zero-filled, tests RtlAnsiCharToUnicodeChar on null byte).
   - Valid ASCII (single-byte, `mbarr[0]<0x80`, expect `mbs-mbarr==1`).
   - Valid lead-byte sequence (e.g., shift-JIS or codepage DBCS, `count>=2`, expect `mbs-mbarr==2`).
   - Invalid sequence triggering space guard (malformed byte producing `wc==L' '` with `mbarr[0]!=' '`, line 28-29 → -1).
   - `count<needed` (e.g., lead-byte present but `count==1`, tests truncation behavior of `RtlAnsiCharToUnicodeChar` with partial input).

**Core needed:**
`NEW:mb_to_wc` signature: `fn mb_to_wc(mb_slice: &[u8]) -> Result<(char, usize), MbError>` — wraps `RtlAnsiCharToUnicodeChar` logic (decode up to 2 bytes, detect space-fallback, return wide char + consumed count or error). The shim reconstructs `&mbchar[..min(count,2)]`, calls core, writes `*wchar` on Ok, collapses `Ok((wc, n))` → `n as i32`, `Err(_)` → -1.