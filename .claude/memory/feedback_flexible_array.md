---
name: bindgen-flexible-array-bug
description: Bindgen flexible arrays [T; 1] must use pointer arithmetic, not indexing — causes silent corruption/hang
type: feedback
---

Bindgen models C flexible array members (`Type field[]` or `Type field[1]`) as `field: [T; 1]` in Rust. Indexing `field[i]` for `i >= 1` is UB in Rust because the array length is 1. On amd64 this silently corrupts/hangs; it doesn't crash with a bounds check.

**Why:** This was the root cause of the devaction.rs PnP hang. `DEVICE_RELATIONS.Objects` is a flexible array of `PDEVICE_OBJECT`. The Rust code indexed it as `(*DeviceRelations).Objects[i as usize]` which is UB for i >= 1. Codex found this by adding serial tracing — enumeration processed rel[0] but died before rel[1]. Two independent LLM translations (Sonnet + Codex) made the same mistake.

**How to apply:**
- NEVER index bindgen flexible arrays directly (e.g., `array[i]`)
- Use pointer arithmetic: `core::ptr::addr_of_mut!((*ptr).field[0]).add(i)` to get slot, `*slot` to read
- Or use `.as_ptr().add(i)` pattern (already used correctly in plugplay.rs)
- Common flexible array structs in the kernel: DEVICE_RELATIONS, CM_RESOURCE_LIST, IO_RESOURCE_REQUIREMENTS_LIST, KEY_VALUE_FULL_INFORMATION
- When reviewing Rust kernel translations, grep for `Objects[` or similar array indexing on bindgen types and check if the underlying C struct has a flexible array member
