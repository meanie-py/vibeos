# vibeos wow64.dll — the Win10+-model WOW64 host layer (thunk dispatcher).
# Phase A skeleton (docs/engineering/wow64-plan.md): exactly the two exports
# the CPU layer (libwow64fex.dll on arm64, wow64cpu.dll on amd64) imports;
# the thunk table grows in Phase B+. arm64 + amd64 hosts (Phase E).
@ stdcall -arch=arm64,x86_64 Wow64SystemServiceEx(long ptr)
@ stdcall -arch=arm64,x86_64 Wow64ProcessPendingCrossProcessItems()
# Phase C: the 64-bit ntdll loader's thread handoff for WoW64 processes
# (ldrinit.rs LdrpInitializeProcess resolves this by name; never returns).
@ stdcall -arch=arm64,x86_64 Wow64LdrpInitialize(ptr)
