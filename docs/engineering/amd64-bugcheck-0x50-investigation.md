# amd64 BUGCHECK 0x50 (0xFFD03000) — Investigation Notes

**Status: localized, not fixed.**

## Symptom

amd64 boot fails immediately with:
```
*** BUGCHECK: 0x00000050 (0x00000000ffd03000, 0x0000000000000002, 0x000000000000df80, 0x0000000000000002)
```

Only ~107 bytes of serial output — the BUGCHECK line and nothing else. No DbgPrint
from earlier boot stages fires. The kernel halts at `xHalHaltSystem`.

i386 boots 7/7 PASS. arm64 boots 5/5 PASS. amd64 alone is broken.

## Decoding the BUGCHECK

BUGCHECK 0x50 = `PAGE_FAULT_IN_NONPAGED_AREA`. Params (from the call site in
`ntoskrnl-rust/src/ARM3/pagfault.rs:1489`):

- Param1: `0xFFD03000` — bad VA (the address being accessed)
- Param2: `0x2` — `FaultCode` (write fault)
- Param3: `0xdf80` — `TrapInformation` (probably truncated trap-frame pointer)
- Param4: `0x2` — sentinel marker: "PXE/PPE/PDE invalid for this VA"

The faulting address `0xFFD03000` is **NOT a canonical amd64 kernel VA**. It is the
**lower 32 bits of `0xFFFFFFFFFFD03000`**, which IS the HAL heap address on amd64
after 3 page allocations (`MM_HAL_VA_START + MM_HAL_HEAP_OFFSET + 3*PAGE_SIZE` =
`0xFFFFFFFFFFC00000 + 0x100000 + 0x3000`).

**This is a 64-bit→32-bit truncation bug** in HAL VA handling on amd64.

## Bisect

Last known good amd64 boot: `ef416833` (Mon May 4 — "ke/mm: per-arch kernel stack
constants — fixes amd64 taskbar BUGCHECK 0x7F"). Confirmed boots to "shell
ready".

First bad commit found: somewhere between `173c27a3` (reboot.c translation, GOOD) and
`b549442c` (halinit.c translation, BAD). Specifically:

- `173c27a3` reboot.c — GOOD (boots to shell ready)
- `b811836f` ntkrnl-sys raw-dylib HAL infra — **first BAD**
- `b549442c` halinit.c — BAD (same as b811836f)

The b811836f commit message claims smoke passed at the time. So either:

1. The commit was always-broken on amd64 but the user verified only i386.
2. Build environment drift made it broken since.
3. Something else broke and the symptom manifests at this commit.

## Two distinct failure modes

The git history shows **TWO bugs** at different commits:

| Commit | Mode |
|--------|------|
| b811836f → 975766d9- | freeldr fails to load ntoskrnl.dll (`PeLdrLoadImage failed`) |
| 975766d9 → HEAD | freeldr loads OK, kernel crashes with `BUGCHECK 0x50 0xFFD03000` |

`975766d9 freeldr: don't mutate OriginalFirstThunk during by-name import bind` is a
freeldr-side fix for the raw-dylib issue from b811836f. It successfully unblocks PE
loading but **reveals the next layer of breakage** — the HAL VA truncation
bug. The HAL VA truncation likely existed before b811836f but was masked by the
freeldr error.

## Most suspicious commits between 173c27a3 (good) and HEAD (BUGCHECK 0x50)

- `b811836f` — adds raw-dylib HAL imports (41 kernel-data symbols). These are
  imported via the PE IAT. If any IAT entry is being read as 32-bit on amd64
  somewhere, that's the bug.
- `3df10a09` — translates `generic/memory.c` (HAL VA window mapper). Defines
  `HalpHeapStart`, `MM_HAL_VA_START`, `address_to_pte`. The HAL VA-related
  computations all live here.

`MM_HAL_VA_START` is correctly defined as `0xFFFFFFFFFFC00000` for amd64. The
binary dump shows `HalpHeapStart` static is correctly initialized to
`0xFFFFFFFFFFD00000` (verified via `objdump -s -j .data`).

## Next steps (when revisiting)

1. **Direct gdb on amd64 boot** with breakpoint at `HalpMapPhysicalMemory64Vista`
   to see the actual register state at first call.
2. **Cherry-pick freeldr fix onto `b811836f^`** to test whether kernel-side bug
   exists pre-raw-dylib. If yes, the bug is in memory.rs (3df10a09); if not, in
   raw-dylib infra.
3. **Check 64-bit reads of HalpHeapStart in the disassembly** of
   HalpMapPhysicalMemory64Vista and its callers (halacpi.c, etc.). Look for any
   `movl` (32-bit) where `movq` is expected.
4. **Audit raw-dylib externs in `hal_bindgen_amd64.rs`** for type-width bugs —
   bindgen might generate `static mut X: u32` for a `ULONG` that's effectively
   `void *` on the C side, causing IAT entry truncation.

## i386 status

i386 boots 7/7 PASS at every commit tested above. The HAL VA on i386 is natively
32-bit (`0xFFC00000`) so the truncation bug — if it's a 32-bit cast — wouldn't
manifest there.

## Working snapshot

Confirmed last-good-amd64 commit: `ef416833`.
Reproducer: `qemu/start.sh amd64` → BUGCHECK in <5s at HEAD.
