# vibeos wow64.dll — the Win10+-model WOW64 host layer (thunk dispatcher).
# Phase A skeleton (docs/engineering/wow64-plan.md): exactly the two exports
# libwow64fex.dll (the FEX CPU layer) imports; the thunk table grows in
# Phase B+. arm64-only for now (x86-on-arm64); Phase E reuses it on amd64.
@ stdcall -arch=arm64 Wow64SystemServiceEx(long ptr)
@ stdcall -arch=arm64 Wow64ProcessPendingCrossProcessItems()
# Phase C: the 64-bit ntdll loader's thread handoff for WoW64 processes
# (ldrinit.rs LdrpInitializeProcess resolves this by name; never returns).
@ stdcall -arch=arm64 Wow64LdrpInitialize(ptr)
