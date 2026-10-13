# WppRecorder.sys: the in-flight recorder behind WPP tracing (the WDK's "Inflight Trace
# Recorder"), imported by 128 of 478 vendor driver packages on the measuring host. With no
# trace consumer and no crash-dump triage reader here, recording to nowhere is correct.
# Built for 64-bit only: every importing package is x64, and the exports' i386 argument
# sizes are not confirmed (stdcall decoration needs them exact).
# Not exported (no importer in the census): imp_WppRecorderDumpLiveDriverData,
# imp_WppRecorderGetCounterHandle, imp_WppRecorderLinkCounters, imp_WppRecorderLogDumpLiveData.
@ stdcall DllInitialize(ptr)
@ stdcall DllUnload()
@ stdcall WppAutoLogStart(ptr ptr ptr)
@ stdcall WppAutoLogStop(ptr ptr)
@ cdecl WppAutoLogTrace(ptr long long ptr long)
@ stdcall imp_WppRecorderConfigure(ptr ptr)
@ stdcall imp_WppRecorderGetTriageInfo(ptr ptr)
@ stdcall imp_WppRecorderIsDefaultLogAvailable(ptr)
@ stdcall imp_WppRecorderLogCreate(ptr ptr ptr)
@ stdcall imp_WppRecorderLogDelete(ptr ptr)
@ stdcall imp_WppRecorderLogGetDefault(ptr)
@ stdcall imp_WppRecorderLogSetIdentifier(ptr ptr ptr)
@ stdcall imp_WppRecorderReplay(ptr int64 long long)
