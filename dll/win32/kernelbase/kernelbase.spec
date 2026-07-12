; kernelbase.dll — minimal, Vista-scoped export surface (vibeos).
;
; This is NOT the full 1746-export wine kernelbase (dll/win32/kernelbase/wine,
; still stubbed out — "ReactOS isn't ready"). It is a deliberately tiny, real
; kernelbase.dll that owns the modern *synchronization* surface so the
; api-ms-win-core-synch-l1-2-0 api-set can resolve for foreign binaries (FEX-Emu
; guest host, ucrtbase/libc++) — see docs/engineering/fex-bringup-plan.md rung 2.
;
; Scope = exactly the symbols fexprobe imports from that api-set. Almost all
; forward: the condition-variable / InitOnce wrappers live (ungated) in
; kernel32_vista.dll; the address-wake primitives live in ntdll_vista
; (RtlWakeAddress*); legacy Sleep stays in kernel32. Only WaitOnAddress needs
; local code (a DWORD-ms -> LARGE_INTEGER shim over ntdll_vista!RtlWaitOnAddress)
; — dllmain.c. Grow this surface as more foreign-binary imports demand it; the
; real (below-kernel32) kernelbase layering is future work.
;
; NB: the modern Rtl synch prims are -version=0x600+, so they are STRIPPED from
; the 0x502-baseline base ntdll.dll and only export from ntdll_vista.dll — hence
; every ntdll forward below targets ntdll_vista, not ntdll.

@ stdcall Sleep(long) kernel32.Sleep
@ stdcall InitOnceExecuteOnce(ptr ptr ptr ptr) kernel32_vista.InitOnceExecuteOnce
@ stdcall SleepConditionVariableSRW(ptr ptr long long) kernel32_vista.SleepConditionVariableSRW
@ stdcall WakeConditionVariable(ptr) kernel32_vista.WakeConditionVariable
@ stdcall WakeAllConditionVariable(ptr) kernel32_vista.WakeAllConditionVariable
@ stdcall WakeByAddressSingle(ptr) ntdll_vista.RtlWakeAddressSingle
@ stdcall WakeByAddressAll(ptr) ntdll_vista.RtlWakeAddressAll
@ stdcall WaitOnAddress(ptr ptr long long)
