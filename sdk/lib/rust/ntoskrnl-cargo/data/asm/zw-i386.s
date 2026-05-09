.intel_syntax noprefix


.cfi_sections .debug_frame


.altmacro
.macro .PROC name







    .globl \name
    \name:

    .cfi_startproc
.endm



.macro .ENDP





    .cfi_endproc
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
    StackBytes = 4 * ArgCount
    FPO 0, 0, 0, 0, 0, 0
    mov eax, SyscallId
    mov edx, 0x7ffe0300
    call dword ptr [edx]
    ret StackBytes
.endm
.macro STUBCODE_K, Name, SyscallId, ArgCount
    StackBytes = 4 * \ArgCount
    FPO 0, 0, 0, 0, 0, 0
    mov eax, SyscallId
    lea edx, [esp + 4]
    pushfd
    push 8
    call _KiSystemService
    ret StackBytes
.endm
.macro MAKE_LABEL, Name, StackBytes
    PUBLIC _\Name\()@\StackBytes
    _\Name\()@\StackBytes\():
.endm
.macro START_PROC, Name, StackBytes
    PUBLIC _\Name\()@\StackBytes
    .PROC _\Name\()@\StackBytes
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
EXTERN _KiSystemService:PROC






.code



    STUB_K AcceptConnectPort, 6
    STUB_K AccessCheck, 8
    STUB_K AccessCheckAndAuditAlarm, 11
    STUB_K AccessCheckByType, 11
    STUB_K AccessCheckByTypeAndAuditAlarm, 16
    STUB_K AccessCheckByTypeResultList, 11
    STUB_K AccessCheckByTypeResultListAndAuditAlarm, 16
    STUB_K AccessCheckByTypeResultListAndAuditAlarmByHandle, 17
    STUB_K AddAtom, 3
    STUB_K AddBootEntry, 2
    STUB_K AddDriverEntry, 2
    STUB_K AdjustGroupsToken, 6
    STUB_K AdjustPrivilegesToken, 6
    STUB_K AlertResumeThread, 2
    STUB_K AlertThread, 1
    STUB_K AllocateLocallyUniqueId, 1
    STUB_K AllocateUserPhysicalPages, 3
    STUB_K AllocateUuids, 4
    STUB_K AllocateVirtualMemory, 6
    STUB_K ApphelpCacheControl, 2
    STUB_K AreMappedFilesTheSame, 2
    STUB_K AssignProcessToJobObject, 2
    STUB_K CallbackReturn, 3
    STUB_K CancelDeviceWakeupRequest, 1
    STUB_K CancelIoFile, 2
    STUB_K CancelTimer, 2
    STUB_K ClearEvent, 1
    STUB_K Close, 1
    STUB_K CloseObjectAuditAlarm, 3
    STUB_K CompactKeys, 2
    STUB_K CompareTokens, 3
    STUB_K CompleteConnectPort, 1
    STUB_K CompressKey, 1
    STUB_K ConnectPort, 8
    STUB_K Continue, 2
    STUB_K CreateDebugObject, 4
    STUB_K CreateDirectoryObject, 3
    STUB_K CreateEvent, 5
    STUB_K CreateEventPair, 3
    STUB_K CreateFile, 11
    STUB_K CreateIoCompletion, 4
    STUB_K CreateJobObject, 3
    STUB_K CreateJobSet, 3
    STUB_K CreateKey, 7
    STUB_K CreateMailslotFile, 8
    STUB_K CreateMutant, 4
    STUB_K CreateNamedPipeFile, 14
    STUB_K CreatePagingFile, 4
    STUB_K CreatePort, 5
    STUB_K CreateProcess, 8
    STUB_K CreateProcessEx, 9
    STUB_K CreateProfile, 9
    STUB_K CreateSection, 7
    STUB_K CreateSemaphore, 5
    STUB_K CreateSymbolicLinkObject, 4
    STUB_K CreateThread, 8
    STUB_K CreateTimer, 4
    STUB_K CreateToken, 13
    STUB_K CreateWaitablePort, 5
    STUB_K DebugActiveProcess, 2
    STUB_K DebugContinue, 3
    STUB_K DelayExecution, 2
    STUB_K DeleteAtom, 1
    STUB_K DeleteBootEntry, 1
    STUB_K DeleteDriverEntry, 1
    STUB_K DeleteFile, 1
    STUB_K DeleteKey, 1
    STUB_K DeleteObjectAuditAlarm, 3
    STUB_K DeleteValueKey, 2
    STUB_K DeviceIoControlFile, 10
    STUB_K DisplayString, 1
    STUB_K DuplicateObject, 7
    STUB_K DuplicateToken, 6
    STUB_K EnumerateBootEntries, 2
    STUB_K EnumerateDriverEntries, 2
    STUB_K EnumerateKey, 6
    STUB_K EnumerateSystemEnvironmentValuesEx, 3
    STUB_K EnumerateValueKey, 6
    STUB_K ExtendSection, 2
    STUB_K FilterToken, 6
    STUB_K FindAtom, 3
    STUB_K FlushBuffersFile, 2
    STUB_K FlushInstructionCache, 3
    STUB_K FlushKey, 1
    STUB_K FlushVirtualMemory, 4
    STUB_K FlushWriteBuffer, 0
    STUB_K FreeUserPhysicalPages, 3
    STUB_K FreeVirtualMemory, 4
    STUB_K FsControlFile, 10
    STUB_K GetContextThread, 2
    STUB_K GetDevicePowerState, 2
    STUB_K GetPlugPlayEvent, 4
    STUB_K GetWriteWatch, 7
    STUB_K ImpersonateAnonymousToken, 1
    STUB_K ImpersonateClientOfPort, 2
    STUB_K ImpersonateThread, 3
    STUB_K InitializeRegistry, 1
    STUB_K InitiatePowerAction, 4
    STUB_K IsProcessInJob, 2
    STUB_K IsSystemResumeAutomatic, 0
    STUB_K ListenPort, 2
    STUB_K LoadDriver, 1
    STUB_K LoadKey, 2
    STUB_K LoadKey2, 3
    STUB_K LoadKeyEx, 4
    STUB_K LockFile, 10
    STUB_K LockProductActivationKeys, 2
    STUB_K LockRegistryKey, 1
    STUB_K LockVirtualMemory, 4
    STUB_K MakePermanentObject, 1
    STUB_K MakeTemporaryObject, 1
    STUB_K MapUserPhysicalPages, 3
    STUB_K MapUserPhysicalPagesScatter, 3
    STUB_K MapViewOfSection, 10
    STUB_K ModifyBootEntry, 1
    STUB_K ModifyDriverEntry, 1
    STUB_K NotifyChangeDirectoryFile, 9
    STUB_K NotifyChangeKey, 10
    STUB_K NotifyChangeMultipleKeys, 12
    STUB_K OpenDirectoryObject, 3
    STUB_K OpenEvent, 3
    STUB_K OpenEventPair, 3
    STUB_K OpenFile, 6
    STUB_K OpenIoCompletion, 3
    STUB_K OpenJobObject, 3
    STUB_K OpenKey, 3
    STUB_K OpenMutant, 3
    STUB_K OpenObjectAuditAlarm, 12
    STUB_K OpenProcess, 4
    STUB_K OpenProcessToken, 3
    STUB_K OpenProcessTokenEx, 4
    STUB_K OpenSection, 3
    STUB_K OpenSemaphore, 3
    STUB_K OpenSymbolicLinkObject, 3
    STUB_K OpenThread, 4
    STUB_K OpenThreadToken, 4
    STUB_K OpenThreadTokenEx, 5
    STUB_K OpenTimer, 3
    STUB_K PlugPlayControl, 3
    STUB_K PowerInformation, 5
    STUB_K PrivilegeCheck, 3
    STUB_K PrivilegeObjectAuditAlarm, 6
    STUB_K PrivilegedServiceAuditAlarm, 5
    STUB_K ProtectVirtualMemory, 5
    STUB_K PulseEvent, 2
    STUB_K QueryAttributesFile, 2
    STUB_K QueryBootEntryOrder, 2
    STUB_K QueryBootOptions, 2
    STUB_K QueryDebugFilterState, 2
    STUB_K QueryDefaultLocale, 2
    STUB_K QueryDefaultUILanguage, 1
    STUB_K QueryDirectoryFile, 11
    STUB_K QueryDirectoryObject, 7
    STUB_K QueryDriverEntryOrder, 2
    STUB_K QueryEaFile, 9
    STUB_K QueryEvent, 5
    STUB_K QueryFullAttributesFile, 2
    STUB_K QueryInformationAtom, 5
    STUB_K QueryInformationFile, 5
    STUB_K QueryInformationJobObject, 5
    STUB_K QueryInformationPort, 5
    STUB_K QueryInformationProcess, 5
    STUB_K QueryInformationThread, 5
    STUB_K QueryInformationToken, 5
    STUB_K QueryInstallUILanguage, 1
    STUB_K QueryIntervalProfile, 2
    STUB_K QueryIoCompletion, 5
    STUB_K QueryKey, 5
    STUB_K QueryMultipleValueKey, 6
    STUB_K QueryMutant, 5
    STUB_K QueryObject, 5
    STUB_K QueryOpenSubKeys, 2
    STUB_K QueryOpenSubKeysEx, 4
    STUB_K QueryPerformanceCounter, 2
    STUB_K QueryQuotaInformationFile, 9
    STUB_K QuerySection, 5
    STUB_K QuerySecurityObject, 5
    STUB_K QuerySemaphore, 5
    STUB_K QuerySymbolicLinkObject, 3
    STUB_K QuerySystemEnvironmentValue, 4
    STUB_K QuerySystemEnvironmentValueEx, 5
    STUB_K QuerySystemInformation, 4
    STUB_K QuerySystemTime, 1
    STUB_K QueryTimer, 5
    STUB_K QueryTimerResolution, 3
    STUB_K QueryValueKey, 6
    STUB_K QueryVirtualMemory, 6
    STUB_K QueryVolumeInformationFile, 5
    STUB_K QueueApcThread, 5
    STUB_K RaiseException, 3
    STUB_K RaiseHardError, 6
    STUB_K ReadFile, 9
    STUB_K ReadFileScatter, 9
    STUB_K ReadRequestData, 6
    STUB_K ReadVirtualMemory, 5
    STUB_K RegisterThreadTerminatePort, 1
    STUB_K ReleaseMutant, 2
    STUB_K ReleaseSemaphore, 3
    STUB_K RemoveIoCompletion, 5
    STUB_K RemoveProcessDebug, 2
    STUB_K RenameKey, 2
    STUB_K ReplaceKey, 3
    STUB_K ReplyPort, 2
    STUB_K ReplyWaitReceivePort, 4
    STUB_K ReplyWaitReceivePortEx, 5
    STUB_K ReplyWaitReplyPort, 2
    STUB_K RequestDeviceWakeup, 1
    STUB_K RequestPort, 2
    STUB_K RequestWaitReplyPort, 3
    STUB_K RequestWakeupLatency, 1
    STUB_K ResetEvent, 2
    STUB_K ResetWriteWatch, 3
    STUB_K RestoreKey, 3
    STUB_K ResumeProcess, 1
    STUB_K ResumeThread, 2
    STUB_K SaveKey, 2
    STUB_K SaveKeyEx, 3
    STUB_K SaveMergedKeys, 3
    STUB_K SecureConnectPort, 9
    STUB_K SetBootEntryOrder, 2
    STUB_K SetBootOptions, 2
    STUB_K SetContextThread, 2
    STUB_K SetDebugFilterState, 3
    STUB_K SetDefaultHardErrorPort, 1
    STUB_K SetDefaultLocale, 2
    STUB_K SetDefaultUILanguage, 1
    STUB_K SetDriverEntryOrder, 2
    STUB_K SetEaFile, 4
    STUB_K SetEvent, 2
    STUB_K SetEventBoostPriority, 1
    STUB_K SetHighEventPair, 1
    STUB_K SetHighWaitLowEventPair, 1
    STUB_K SetInformationDebugObject, 5
    STUB_K SetInformationFile, 5
    STUB_K SetInformationJobObject, 4
    STUB_K SetInformationKey, 4
    STUB_K SetInformationObject, 4
    STUB_K SetInformationProcess, 4
    STUB_K SetInformationThread, 4
    STUB_K SetInformationToken, 4
    STUB_K SetIntervalProfile, 2
    STUB_K SetIoCompletion, 5
    STUB_K SetLdtEntries, 6
    STUB_K SetLowEventPair, 1
    STUB_K SetLowWaitHighEventPair, 1
    STUB_K SetQuotaInformationFile, 4
    STUB_K SetSecurityObject, 3
    STUB_K SetSystemEnvironmentValue, 2
    STUB_K SetSystemEnvironmentValueEx, 5
    STUB_K SetSystemInformation, 3
    STUB_K SetSystemPowerState, 3
    STUB_K SetSystemTime, 2
    STUB_K SetThreadExecutionState, 2
    STUB_K SetTimer, 7
    STUB_K SetTimerResolution, 3
    STUB_K SetUuidSeed, 1
    STUB_K SetValueKey, 6
    STUB_K SetVolumeInformationFile, 5
    STUB_K ShutdownSystem, 1
    STUB_K SignalAndWaitForSingleObject, 4
    STUB_K StartProfile, 1
    STUB_K StopProfile, 1
    STUB_K SuspendProcess, 1
    STUB_K SuspendThread, 2
    STUB_K SystemDebugControl, 6
    STUB_K TerminateJobObject, 2
    STUB_K TerminateProcess, 2
    STUB_K TerminateThread, 2
    STUB_K TestAlert, 0
    STUB_K TraceEvent, 4
    STUB_K TranslateFilePath, 4
    STUB_K UnloadDriver, 1
    STUB_K UnloadKey, 1
    STUB_K UnloadKey2, 2
    STUB_K UnloadKeyEx, 2
    STUB_K UnlockFile, 5
    STUB_K UnlockVirtualMemory, 4
    STUB_K UnmapViewOfSection, 2
    STUB_K VdmControl, 2
    STUB_K WaitForDebugEvent, 4
    STUB_K WaitForMultipleObjects, 5
    STUB_K WaitForSingleObject, 3
    STUB_K WaitHighEventPair, 1
    STUB_K WaitLowEventPair, 1
    STUB_K WriteFile, 9
    STUB_K WriteFileGather, 9
    STUB_K WriteRequestData, 6
    STUB_K WriteVirtualMemory, 5
    STUB_K YieldExecution, 0
    STUB_K CreateKeyedEvent, 4
    STUB_K OpenKeyedEvent, 3
    STUB_K ReleaseKeyedEvent, 4
    STUB_K WaitForKeyedEvent, 4
    STUB_K QueryPortInformationProcess, 0
    STUB_K GetCurrentProcessorNumber, 0
    STUB_K WaitForMultipleObjects32, 5



    END
