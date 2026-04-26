---
name: cgu1-not-drop-in
description: codegen-units=1 eliminates captures(none) bug but can't be done without reverting i386 workarounds
type: feedback
---

Setting `codegen-units = 1` in Cargo.toml eliminates all cross-CGU `captures(none)` LLVM miscompiles (verified via LLVM IR). However, it CANNOT be done as a drop-in change — i386 crashes with BUGCHECK 0x1E because the existing workarounds (ObCreateObjectAddress_shim returning ULONG_PTR, IopFinishCreatedDeviceObject #[inline(never)] helper split, RTL string shims) were designed for CGU=16 and interfere with the CGU=1 optimization landscape.

**Why:** amd64 passes with CGU=1, but i386 gets ud2 at a different address. The workarounds change the code structure in ways that CGU=1's optimizer handles differently.

**How to apply:** CGU=1 must be a coordinated change:
1. Set `codegen-units = 1`
2. Revert ObCreateObjectAddress_shim → back to ObCreateObject_shim returning PVOID
3. Revert IopFinishCreatedDeviceObject → merge back into IoCreateDevice
4. Revert IopFinishIoInitSystem → merge back into IoInitSystem
5. Revert RTL string shims → use direct imports
6. Remove the ARM64 `#[inline(never)]` on PspCreateThread
7. Test all 3 arches
This is a future task, not a quick toggle.
