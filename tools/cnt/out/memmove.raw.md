```toml
[symbols.memmove]
c_signature = "void * __cdecl memmove(void *dest, const void *src, size_t count)"
family = "mem"
slice_pattern = "mem_raw"
core_fn = "raw_move"
return_collapse = "ret_dst"
inout_effects = "dest[0..count] ← src[0..count], byte-by-byte with overlap safety; no termination, no padding"
preserved_misfeature = "none — textbook overlap-safe copy via direction selection"
fuzz_domain = ["non_overlap_forward", "non_overlap_backward", "overlap_forward_shift", "overlap_backward_shift", "zero_count", "same_ptr"]
```

## Rationale

### 1. Exact return value (lines 8, 36)
`return dest;` — always returns the original `dest` pointer unchanged, regardless of count or overlap. No error conditions.

### 2. Which parameters written (lines 13-35)
`dest[0..count]` is written byte-by-byte with `src[0..count]` content. The direction depends on overlap:
- **Non-overlapping or forward-safe** (line 11: `(char_dest <= char_src) || (char_dest >= (char_src+count))`): forward copy, lines 14-20
- **Backward overlap** (else): backward copy from end, lines 24-35

No null-termination added. No padding. Exactly `count` bytes copied.

### 3. NULL/zero-length behavior
- **`count == 0`**: Lines 14/27 `while(count > 0)` — loop bodies never execute, returns `dest` immediately with no memory access. Safe for any pointer values including NULL.
- **NULL pointers with `count > 0`**: No guard. Will dereference and crash (lines 18/31 `*char_dest = *char_src`).

### 4. Deviation from textbook libc
**None.** This is textbook `memmove`: overlap-safe via direction selection. The comment on line 7 explicitly notes the duplication with `memcpy`, confirming ReactOS's `memcpy` is *also* overlap-safe (the actual deviation is in `memcpy`, not here).

The overlap detection condition (line 11) correctly identifies when forward-copy is safe:
- `char_dest <= char_src` → dest before src, forward is safe
- `char_dest >= (char_src+count)` → dest after src region, forward is safe
- Otherwise → backward copy to prevent clobbering

### 5. Minimal fuzz domain
1. **`non_overlap_forward`**: `dest < src` with gap — exercises line 11 first clause, forward loop lines 14-20
2. **`non_overlap_backward`**: `dest > src+count` — exercises line 11 second clause, forward loop
3. **`overlap_forward_shift`**: `src < dest < src+count` — exercises backward loop lines 24-35
4. **`overlap_backward_shift`**: `dest < src < dest+count` (though forward-copy safe, tests line 11 logic)
5. **`zero_count`**: `count == 0` — tests no-op path, no loop entry
6. **`same_ptr`**: `dest == src` — degenerate overlap, tests either branch works correctly

These cover both branch outcomes of line 11's compound condition, both loop implementations, and the zero-count boundary.