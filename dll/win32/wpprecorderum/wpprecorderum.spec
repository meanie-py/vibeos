# wpprecorderum.dll: the user-mode WPP in-flight recorder, wpprecorder.sys's counterpart
# for UMDF drivers and services (9 packages on the measuring host import it, all x64, all
# for WppAutoLogStart/Stop/Trace only). 64-bit only, as wpprecorder.sys is.
# Not exported (no importer in the census): imp_WppRecorderLogDumpLiveData.
@ stdcall WppAutoLogGetDefaultHandle()
@ stdcall WppAutoLogStart(ptr ptr)
@ stdcall WppAutoLogStop(ptr)
@ cdecl WppAutoLogTrace(ptr long long ptr long)
