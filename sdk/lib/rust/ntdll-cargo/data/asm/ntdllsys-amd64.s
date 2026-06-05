.intel_syntax noprefix


.cfi_sections .debug_frame


.altmacro
.macro .PROC name







    .globl \name
    \name:



    .seh_proc \name
.endm



.macro .ENDP







    .seh_endproc
.endm



.macro PUBLIC symbol
    .global \symbol
.endm


.macro public symbol
    .global \symbol
.endm



.macro GLOBAL_LABEL label
    \label:
.endm


.macro ASSUME p1 p2 p3 p4 p5 p6 p7 p8
.endm


.macro .endcode16
.endm


.macro ALIGN x
    .align x
.endm

.macro align x
    .align x
.endm
.macro ljmp segment, offset
    .byte 0xEA
    .long offset
    .word segment
.endm

.macro ljmp16 segment, offset
    .byte 0xEA
    .word offset
    .word segment
.endm

.macro data32 opcode:vararg
    .byte 0x66
    opcode
.endm

.macro retf
    .byte 0xCB
.endm


.macro EXTERN name:vararg
.endm


.macro END
.endm

.macro end
.endm


.macro .MODEL model
.endm

.macro .code
    .text
.endm

.macro .const
    .section .rdata
.endm





.macro FPO cdwLocals, cdwParams, cbProlog, cbRegs, fUseBP, cbFrame
    .if (cbFrame == 1)
        .cfi_signal_frame
    .endif
.endm


.macro .allocstack size
    .seh_stackalloc \size
.endm

.macro .pushframe param

    .ifc \param, code
    .seh_pushframe @code
    .else
    .seh_pushframe
    .endif
.endm

.macro .pushreg reg
    .seh_pushreg \reg
.endm

.macro .savereg reg, offset
    .seh_savereg \reg, \offset
.endm

.macro .savexmm128 reg, offset
    .seh_savexmm \reg, \offset
.endm

.macro .setframe reg, offset
    .seh_setframe \reg, \offset
.endm

.macro .endprolog
    .seh_endprologue
.endm



.macro .ALLOCSTACK size
    .seh_stackalloc \size
.endm

.macro .PUSHFRAME param
    .ifc \param, code
    .seh_pushframe @code
    .else
    .seh_pushframe
    .endif
.endm

.macro .PUSHREG reg
    .seh_pushreg \reg
.endm

.macro .SAVEREG reg, offset
    .seh_savereg \reg, \offset
.endm

.macro .SAVEXMM128 reg, offset
    .seh_savexmm \reg, \offset
.endm

.macro .SETFRAME reg, offset
    .seh_setframe \reg, \offset
.endm

.macro .ENDPROLOG
    .seh_endprologue
.endm


.macro .seh_code
    .text
.endm


.macro absolute address
    __absolute__address__ = \address
.endm

.macro resb name, size
    \name = __absolute__address__
    __absolute__address__ = __absolute__address__ + \size
.endm

.macro UNIMPLEMENTED2 file, line, func
    jmp 4f
1: .ascii "Unimplemented %s (%s:%d)", "\r", "\n", "\0"
2: .asciz "\func"
3: .asciz "\file"
4:
    sub rsp, 0x20
    lea rcx, 1b[rip]
    lea rdx, 2b[rip]
    lea r8, 3b[rip]
    mov r9, \line
    call DbgPrint
    add rsp, 0x20
.endm

SyscallId = 0
.macro STUBCODE_U, Name, SyscallId, ArgCount
    .ENDPROLOG
    mov eax, SyscallId
    mov r10, rcx
    syscall
    ret
.endm
.macro STUBCODE_K, Name, SyscallId, ArgCount
    .ENDPROLOG
    EXTERN Nt\Name:PROC
    lea rax, Nt\Name\()[rip]
    mov r10, ArgCount * 8
    jmp KiZwSystemService
.endm
.macro MAKE_LABEL, Name, StackBytes
    PUBLIC \Name
    \Name\():
.endm
.macro START_PROC, Name, StackBytes
    PUBLIC \Name
    .PROC \Name
.endm



.macro STUB_U, Name, ArgCount
    MAKE_LABEL Zw\Name, %ArgCount * 4
    START_PROC Nt\Name, %ArgCount * 4
    STUBCODE_U Name, SyscallId, %ArgCount
    .ENDP
    SyscallId = SyscallId + 1
.endm

.macro STUB_K, Name, ArgCount
    START_PROC Zw\Name, %ArgCount * 4
    STUBCODE_K Name, SyscallId, %ArgCount
    .ENDP
    SyscallId = SyscallId + 1
.endm
.code



    STUB_U AcceptConnectPort, 6
    STUB_U AccessCheck, 8
    STUB_U AccessCheckAndAuditAlarm, 11
    STUB_U AccessCheckByType, 11
    STUB_U AccessCheckByTypeAndAuditAlarm, 16
    STUB_U AccessCheckByTypeResultList, 11
    STUB_U AccessCheckByTypeResultListAndAuditAlarm, 16
    STUB_U AccessCheckByTypeResultListAndAuditAlarmByHandle, 17
    STUB_U AddAtom, 3
    STUB_U AddBootEntry, 2
    STUB_U AddDriverEntry, 2
    STUB_U AdjustGroupsToken, 6
    STUB_U AdjustPrivilegesToken, 6
    STUB_U AlertResumeThread, 2
    STUB_U AlertThread, 1
    STUB_U AllocateLocallyUniqueId, 1
    STUB_U AllocateUserPhysicalPages, 3
    STUB_U AllocateUuids, 4
    STUB_U AllocateVirtualMemory, 6
    STUB_U ApphelpCacheControl, 2
    STUB_U AreMappedFilesTheSame, 2
    STUB_U AssignProcessToJobObject, 2
    STUB_U CallbackReturn, 3
    STUB_U CancelDeviceWakeupRequest, 1
    STUB_U CancelIoFile, 2
    STUB_U CancelTimer, 2
    STUB_U ClearEvent, 1
    STUB_U Close, 1
    STUB_U CloseObjectAuditAlarm, 3
    STUB_U CompactKeys, 2
    STUB_U CompareTokens, 3
    STUB_U CompleteConnectPort, 1
    STUB_U CompressKey, 1
    STUB_U ConnectPort, 8
    STUB_U Continue, 2
    STUB_U CreateDebugObject, 4
    STUB_U CreateDirectoryObject, 3
    STUB_U CreateEvent, 5
    STUB_U CreateEventPair, 3
    STUB_U CreateFile, 11
    STUB_U CreateIoCompletion, 4
    STUB_U CreateJobObject, 3
    STUB_U CreateJobSet, 3
    STUB_U CreateKey, 7
    STUB_U CreateMailslotFile, 8
    STUB_U CreateMutant, 4
    STUB_U CreateNamedPipeFile, 14
    STUB_U CreatePagingFile, 4
    STUB_U CreatePort, 5
    STUB_U CreateProcess, 8
    STUB_U CreateProcessEx, 9
    STUB_U CreateProfile, 9
    STUB_U CreateSection, 7
    STUB_U CreateSemaphore, 5
    STUB_U CreateSymbolicLinkObject, 4
    STUB_U CreateThread, 8
    STUB_U CreateTimer, 4
    STUB_U CreateToken, 13
    STUB_U CreateWaitablePort, 5
    STUB_U DebugActiveProcess, 2
    STUB_U DebugContinue, 3
    STUB_U DelayExecution, 2
    STUB_U DeleteAtom, 1
    STUB_U DeleteBootEntry, 1
    STUB_U DeleteDriverEntry, 1
    STUB_U DeleteFile, 1
    STUB_U DeleteKey, 1
    STUB_U DeleteObjectAuditAlarm, 3
    STUB_U DeleteValueKey, 2
    STUB_U DeviceIoControlFile, 10
    STUB_U DisplayString, 1
    STUB_U DuplicateObject, 7
    STUB_U DuplicateToken, 6
    STUB_U EnumerateBootEntries, 2
    STUB_U EnumerateDriverEntries, 2
    STUB_U EnumerateKey, 6
    STUB_U EnumerateSystemEnvironmentValuesEx, 3
    STUB_U EnumerateValueKey, 6
    STUB_U ExtendSection, 2
    STUB_U FilterToken, 6
    STUB_U FindAtom, 3
    STUB_U FlushBuffersFile, 2
    STUB_U FlushInstructionCache, 3
    STUB_U FlushKey, 1
    STUB_U FlushVirtualMemory, 4
    STUB_U FlushWriteBuffer, 0
    STUB_U FreeUserPhysicalPages, 3
    STUB_U FreeVirtualMemory, 4
    STUB_U FsControlFile, 10
    STUB_U GetContextThread, 2
    STUB_U GetDevicePowerState, 2
    STUB_U GetPlugPlayEvent, 4
    STUB_U GetWriteWatch, 7
    STUB_U ImpersonateAnonymousToken, 1
    STUB_U ImpersonateClientOfPort, 2
    STUB_U ImpersonateThread, 3
    STUB_U InitializeRegistry, 1
    STUB_U InitiatePowerAction, 4
    STUB_U IsProcessInJob, 2
    STUB_U IsSystemResumeAutomatic, 0
    STUB_U ListenPort, 2
    STUB_U LoadDriver, 1
    STUB_U LoadKey, 2
    STUB_U LoadKey2, 3
    STUB_U LoadKeyEx, 4
    STUB_U LockFile, 10
    STUB_U LockProductActivationKeys, 2
    STUB_U LockRegistryKey, 1
    STUB_U LockVirtualMemory, 4
    STUB_U MakePermanentObject, 1
    STUB_U MakeTemporaryObject, 1
    STUB_U MapUserPhysicalPages, 3
    STUB_U MapUserPhysicalPagesScatter, 3
    STUB_U MapViewOfSection, 10
    STUB_U ModifyBootEntry, 1
    STUB_U ModifyDriverEntry, 1
    STUB_U NotifyChangeDirectoryFile, 9
    STUB_U NotifyChangeKey, 10
    STUB_U NotifyChangeMultipleKeys, 12
    STUB_U OpenDirectoryObject, 3
    STUB_U OpenEvent, 3
    STUB_U OpenEventPair, 3
    STUB_U OpenFile, 6
    STUB_U OpenIoCompletion, 3
    STUB_U OpenJobObject, 3
    STUB_U OpenKey, 3
    STUB_U OpenMutant, 3
    STUB_U OpenObjectAuditAlarm, 12
    STUB_U OpenProcess, 4
    STUB_U OpenProcessToken, 3
    STUB_U OpenProcessTokenEx, 4
    STUB_U OpenSection, 3
    STUB_U OpenSemaphore, 3
    STUB_U OpenSymbolicLinkObject, 3
    STUB_U OpenThread, 4
    STUB_U OpenThreadToken, 4
    STUB_U OpenThreadTokenEx, 5
    STUB_U OpenTimer, 3
    STUB_U PlugPlayControl, 3
    STUB_U PowerInformation, 5
    STUB_U PrivilegeCheck, 3
    STUB_U PrivilegeObjectAuditAlarm, 6
    STUB_U PrivilegedServiceAuditAlarm, 5
    STUB_U ProtectVirtualMemory, 5
    STUB_U PulseEvent, 2
    STUB_U QueryAttributesFile, 2
    STUB_U QueryBootEntryOrder, 2
    STUB_U QueryBootOptions, 2
    STUB_U QueryDebugFilterState, 2
    STUB_U QueryDefaultLocale, 2
    STUB_U QueryDefaultUILanguage, 1
    STUB_U QueryDirectoryFile, 11
    STUB_U QueryDirectoryObject, 7
    STUB_U QueryDriverEntryOrder, 2
    STUB_U QueryEaFile, 9
    STUB_U QueryEvent, 5
    STUB_U QueryFullAttributesFile, 2
    STUB_U QueryInformationAtom, 5
    STUB_U QueryInformationFile, 5
    STUB_U QueryInformationJobObject, 5
    STUB_U QueryInformationPort, 5
    STUB_U QueryInformationProcess, 5
    STUB_U QueryInformationThread, 5
    STUB_U QueryInformationToken, 5
    STUB_U QueryInstallUILanguage, 1
    STUB_U QueryIntervalProfile, 2
    STUB_U QueryIoCompletion, 5
    STUB_U QueryKey, 5
    STUB_U QueryMultipleValueKey, 6
    STUB_U QueryMutant, 5
    STUB_U QueryObject, 5
    STUB_U QueryOpenSubKeys, 2
    STUB_U QueryOpenSubKeysEx, 4
    STUB_U QueryPerformanceCounter, 2
    STUB_U QueryQuotaInformationFile, 9
    STUB_U QuerySection, 5
    STUB_U QuerySecurityObject, 5
    STUB_U QuerySemaphore, 5
    STUB_U QuerySymbolicLinkObject, 3
    STUB_U QuerySystemEnvironmentValue, 4
    STUB_U QuerySystemEnvironmentValueEx, 5
    STUB_U QuerySystemInformation, 4
    STUB_U QuerySystemTime, 1
    STUB_U QueryTimer, 5
    STUB_U QueryTimerResolution, 3
    STUB_U QueryValueKey, 6
    STUB_U QueryVirtualMemory, 6
    STUB_U QueryVolumeInformationFile, 5
    STUB_U QueueApcThread, 5
    STUB_U RaiseException, 3
    STUB_U RaiseHardError, 6
    STUB_U ReadFile, 9
    STUB_U ReadFileScatter, 9
    STUB_U ReadRequestData, 6
    STUB_U ReadVirtualMemory, 5
    STUB_U RegisterThreadTerminatePort, 1
    STUB_U ReleaseMutant, 2
    STUB_U ReleaseSemaphore, 3
    STUB_U RemoveIoCompletion, 5
    STUB_U RemoveProcessDebug, 2
    STUB_U RenameKey, 2
    STUB_U ReplaceKey, 3
    STUB_U ReplyPort, 2
    STUB_U ReplyWaitReceivePort, 4
    STUB_U ReplyWaitReceivePortEx, 5
    STUB_U ReplyWaitReplyPort, 2
    STUB_U RequestDeviceWakeup, 1
    STUB_U RequestPort, 2
    STUB_U RequestWaitReplyPort, 3
    STUB_U RequestWakeupLatency, 1
    STUB_U ResetEvent, 2
    STUB_U ResetWriteWatch, 3
    STUB_U RestoreKey, 3
    STUB_U ResumeProcess, 1
    STUB_U ResumeThread, 2
    STUB_U SaveKey, 2
    STUB_U SaveKeyEx, 3
    STUB_U SaveMergedKeys, 3
    STUB_U SecureConnectPort, 9
    STUB_U SetBootEntryOrder, 2
    STUB_U SetBootOptions, 2
    STUB_U SetContextThread, 2
    STUB_U SetDebugFilterState, 3
    STUB_U SetDefaultHardErrorPort, 1
    STUB_U SetDefaultLocale, 2
    STUB_U SetDefaultUILanguage, 1
    STUB_U SetDriverEntryOrder, 2
    STUB_U SetEaFile, 4
    STUB_U SetEvent, 2
    STUB_U SetEventBoostPriority, 1
    STUB_U SetHighEventPair, 1
    STUB_U SetHighWaitLowEventPair, 1
    STUB_U SetInformationDebugObject, 5
    STUB_U SetInformationFile, 5
    STUB_U SetInformationJobObject, 4
    STUB_U SetInformationKey, 4
    STUB_U SetInformationObject, 4
    STUB_U SetInformationProcess, 4
    STUB_U SetInformationThread, 4
    STUB_U SetInformationToken, 4
    STUB_U SetIntervalProfile, 2
    STUB_U SetIoCompletion, 5
    STUB_U SetLdtEntries, 6
    STUB_U SetLowEventPair, 1
    STUB_U SetLowWaitHighEventPair, 1
    STUB_U SetQuotaInformationFile, 4
    STUB_U SetSecurityObject, 3
    STUB_U SetSystemEnvironmentValue, 2
    STUB_U SetSystemEnvironmentValueEx, 5
    STUB_U SetSystemInformation, 3
    STUB_U SetSystemPowerState, 3
    STUB_U SetSystemTime, 2
    STUB_U SetThreadExecutionState, 2
    STUB_U SetTimer, 7
    STUB_U SetTimerResolution, 3
    STUB_U SetUuidSeed, 1
    STUB_U SetValueKey, 6
    STUB_U SetVolumeInformationFile, 5
    STUB_U ShutdownSystem, 1
    STUB_U SignalAndWaitForSingleObject, 4
    STUB_U StartProfile, 1
    STUB_U StopProfile, 1
    STUB_U SuspendProcess, 1
    STUB_U SuspendThread, 2
    STUB_U SystemDebugControl, 6
    STUB_U TerminateJobObject, 2
    STUB_U TerminateProcess, 2
    STUB_U TerminateThread, 2
    STUB_U TestAlert, 0
    STUB_U TraceEvent, 4
    STUB_U TranslateFilePath, 4
    STUB_U UnloadDriver, 1
    STUB_U UnloadKey, 1
    STUB_U UnloadKey2, 2
    STUB_U UnloadKeyEx, 2
    STUB_U UnlockFile, 5
    STUB_U UnlockVirtualMemory, 4
    STUB_U UnmapViewOfSection, 2
    STUB_U VdmControl, 2
    STUB_U WaitForDebugEvent, 4
    STUB_U WaitForMultipleObjects, 5
    STUB_U WaitForSingleObject, 3
    STUB_U WaitHighEventPair, 1
    STUB_U WaitLowEventPair, 1
    STUB_U WriteFile, 9
    STUB_U WriteFileGather, 9
    STUB_U WriteRequestData, 6
    STUB_U WriteVirtualMemory, 5
    STUB_U YieldExecution, 0
    STUB_U CreateKeyedEvent, 4
    STUB_U OpenKeyedEvent, 3
    STUB_U ReleaseKeyedEvent, 4
    STUB_U WaitForKeyedEvent, 4
    STUB_U QueryPortInformationProcess, 0
    STUB_U GetCurrentProcessorNumber, 0
    STUB_U WaitForMultipleObjects32, 5

END
