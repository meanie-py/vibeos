# irp.rs Known Bugs (found 2026-03-19)

## Bug 1: KTHREAD offset in KeInitializeApc (CRITICAL)
Lines 608, 1315, 1333: `Tcb.__bindgen_anon_1` gives ApcState (offset 72), not KTHREAD start.
Fix: `&mut (*Thread).Tcb as *mut _ as *mut _KTHREAD` (already applied in source)

## Bug 2: Wrong PacketType constant
Line 624: `PacketType = IoPagingPriorityHigh` (2) should be `IopCompletionPacketIrp` (0).
Fix: `PacketType = 0` (already applied in source)

## Status
Fixes applied but IopCompleteRequest still hangs — likely more bugs in the 300-line
APC completion callback. The function needs a careful line-by-line audit or a C shim.
IofCompleteRequestInner and IofCallDriverInner also need review.
