```toml
[symbols.memset]
c_signature  = "void* __cdecl memset(void* src, int val, size_t count)"
family       = "mem"
slice_pattern = "set_raw"
core_fn      = "mem::set"
return_collapse = "ret_dst"
inout_effects = "writes `count` bytes at `src`, each byte = `val as u8` (truncated); returns `src` unchanged"
preserved_misfeature = "none — textbook memset: writes val (as u8) to each of count bytes, returns dst"
fuzz_domain  = ["count=0 (no-op)", "count=1", "count>1 typical", "val in 0..=255", "val>255 (truncation)", "val<0 (negative int, cast to u8)"]
```

## Rationale

**1. Exact return value:** Always returns `src` unchanged (line 15), regardless of inputs.

**2. Parameters written:** `src` is written. Lines 11-14 loop `count` times, writing `val` (implicitly truncated to `char`, i.e., `u8`) to each successive byte starting at `char_src`. If `count==0`, the loop never executes (line 11 guard), no write occurs.

**3. NULL/zero-length behavior:** 
   - `count==0`: Loop condition `count>0` (line 11) is false, no deref, returns `src` immediately.
   - `src==NULL` with `count>0`: No explicit guard; would deref NULL at line 12 → crash. This is standard memset behavior.

**4. Deliberate deviation:** **None**. This is textbook memset: byte-wise fill, `val` implicitly truncated to `unsigned char` per C standard (promotion to `int` in signature, then stored as `char`). The loop is naive but functionally equivalent to optimized impls.

**5. Differential test inputs:**
   - **`count=0`**: Boundary — must return `src` with no write.
   - **`count=1` and `count>1`**: Exercise loop body once vs. multiple iterations.
   - **`val` in `0..=255`**: Normal byte range.
   - **`val>255` or `val<0`**: `int` truncation to `u8` — e.g., `val=256` → byte `0x00`, `val=-1` → byte `0xFF` (two's complement cast). Lines 12 assigns `val` to `char`; C truncates high bits.

**Core mapping:** `mem::set(dst: &mut [u8], val: u8)` writes `val` to all bytes. Shim must:
- Reconstruct `slice::from_raw_parts_mut(src as *mut u8, count)` (if `count>0`; if `count==0`, skip to avoid UB on potentially-invalid `src`).
- Cast `val as u8` (Rust truncates `i32` → `u8` identically to C `int` → `char`).
- Call `mem::set(&mut slice, val_u8)`.
- Return `src` (`ret_dst` pattern).