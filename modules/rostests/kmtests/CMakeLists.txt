
include_directories(include)

#
# Native SEH for the standalone kernel-mode test drivers (amd64/arm64)
#
# USE_DUMMY_PSEH is the project-wide C compile mode, so every _SEH2_TRY/_SEH2_EXCEPT
# compiles to a DEAD handler. A test driver that deliberately faults and expects to
# catch it therefore BUGCHECKs instead. IoReadWrite_drv is the textbook case: fast I/O
# hands the driver the raw user buffer, TestCommonWrite() derefs it with no NULL guard,
# and its _SEH2_EXCEPT is supposed to turn the AV into STATUS_ACCESS_VIOLATION — which
# is precisely what the test asserts.
#
# kmtest_drv and kmtest.exe opted into real clang SEH (below); the 17 STANDALONE helper
# drivers never did, so their handlers were silently dead. That is the single root cause
# of the IoReadWrite / NtCreateSection / MmMapLockedPagesSpecifyCache crasher quarantines
# — the kernel was innocent in all three.
#
# pseh2.h's _USE_NATIVE_SEH branch outranks the global _USE_DUMMY_PSEH define and maps
# _SEH2_TRY straight to __try. That is necessary but NOT sufficient: clang's default
# calls-only contract only covers a fault in a CALLEE of the __try body (the return
# address lands inside the scope table), which holds for free because clang refuses to
# inline into a __try region. A fault by a bare INSTRUCTION in the body was still
# uncovered — clang emitted an empty scope table and -O2 deleted the handler — until
# -fasync-exceptions was added below (2026-07-25).
#
# Applied to ALL standalone drivers, not just the three that fault today: the invariant
# is "kmtest drivers get real SEH wherever clang lowers it", which keeps the next
# SEH-using test from silently inheriting a dead handler.
#
# i386 gets this too as of 2026-07-25: its SEH lowering is MSVC-env-only, and the msvc
# retarget is now on for all three arches. See the i386 note below.
#
set(KMT_NATIVE_SEH_SHIMS ${CMAKE_CURRENT_SOURCE_DIR}/kmt_native_seh_shims.c)

#
# INDEPENDENT-ORACLE build: compile the fault-and-catch kmtest drivers with the
# SAME clang retargeted to the -windows-msvc environment instead of the
# project-default -windows-gnu (llvm-mingw). The point is a conformance oracle
# built by a *different* toolchain environment than the OS-under-test, so a
# blind spot shared by our clang-mingw toolchain can't hide a bug from the tests.
# ON by default (this is the standard conformance build). Opt out with
# -DKMT_MSVC_ENV=OFF for an apples-to-apples same-toolchain build.
#
# Proven on amd64/arm64 (2026-07-17): a clean drop-in relink — the retargeted
# objects add ZERO new undefined refs, keep the same .pdata/.xdata SEH model
# (table-based in both envs), and boot+sweep produce results identical to the
# mingw build (only IoDeviceObject wobbles ±1, a pre-existing flaky assertion).
# The real payoff is i386, where -windows-msvc unlocks the frame-based fs:[0] SEH
# the gnu target refuses to lower (L__ehtable$ errors) — see the i386 branch.
#
option(KMT_MSVC_ENV "Build kmtest fault-catch drivers with the clang -windows-msvc env (independent conformance oracle)" ON)

# i386 frame-based native SEH via -windows-msvc.
# Unlike amd64/arm64 (table-based SEH lowers in BOTH clang envs), i386's frame-based
# fs:[0] __try/__except lowers ONLY under -windows-msvc — the gnu target errors
# "L__ehtable$... can not be undefined". So on i386, native SEH is inseparable from
# the msvc-env retarget. _except_handler3 resolves from libpseh.a (i386/seh.s, a real
# implementation — NOT the dummy.c stub in the same archive; pseh precedes ntoskrnl on
# the link line, and ntoskrnl also exports one).
#
# VALIDATED 2026-07-25 and therefore NO LONGER AN OPTION. It was `option(... OFF)`
# pending "validated end-to-end (link+boot+catch)"; all three now hold — it links with
# one shim (_InterlockedAddLargeStatistic, see kmt_native_seh_shims.c), boots, and
# catches: IoReadWrite went CRASH -> OK 16192 pass (matching arm64's baseline exactly)
# and RtlExceptionKM CRASH -> OK 33/0.
#
# Deliberately NOT left as an option(): option() defers to any value already in
# CMakeCache.txt, so every EXISTING build-i386 in the fleet would keep OFF while the
# checked-in conformance baseline assumes ON — i.e. a false regression that depends on
# how old your build dir is. i386 now follows the same rule as amd64/arm64: native SEH
# is unconditional, and KMT_MSVC_ENV is the single opt-out for the whole msvc retarget.

# ASYNCHRONOUS (hardware-fault) SEH coverage — the msvc-env retarget and
# -fasync-exceptions, always together, because neither is any use without the other.
#
# Clang's DEFAULT contract is CALLS-ONLY: it assumes no instruction can fault, so a
# __try whose body contains a faulting *instruction* rather than a *call* gets an
# EMPTY scope table — and at -O2 the __except block is then deleted outright as
# unreachable. Measured on our pinned clang 19.1.7 with
# `__try { *(volatile int*)0 = 1; } __except(1){ return 2; }`:
#
#   {x86_64,aarch64,i686}-pc-windows-msvc, no flag  -> handler DELETED; the function
#                                                      is literally store/xor/ret
#   ... same + -fasync-exceptions                   -> real handler: __C_specific_handler
#                                                      + .seh_handlerdata scope table
#                                                      (x64/arm64), or __except_handler3
#                                                      + fs:[0] frame chain (i686)
#   any *-windows-gnu, either way                   -> flag is a NO-OP (unused-argument
#                                                      warning); in-body faults stay dead
#
# A fault in a CALLEE is covered with or without the flag — that is why the existing
# drivers work at all, since clang refuses to inline into a __try region and the fault
# therefore lands at a return address inside the scope table. Tests that fault DIRECTLY
# in the body — RtlException/RtlExceptionKM do `(void)*(volatile CHAR *)NULL` — get no
# handler without this, which is a dead __except, an unhandled AV, and BUGCHECK 0x1E.
#
# The flag does nothing outside -windows-msvc, so it is applied ONLY where we retarget.
function(kmt_msvc_seh_env _target)
    if(ARCH STREQUAL "amd64")
        target_compile_options(${_target} PRIVATE --target=x86_64-pc-windows-msvc -fasync-exceptions)
    elseif(ARCH STREQUAL "arm64")
        target_compile_options(${_target} PRIVATE --target=aarch64-pc-windows-msvc -fasync-exceptions)
    elseif(ARCH STREQUAL "i386")
        target_compile_options(${_target} PRIVATE --target=i686-pc-windows-msvc -fasync-exceptions)
    endif()
endfunction()

function(kmt_enable_native_seh _target)
    if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
        target_compile_definitions(${_target} PRIVATE _USE_NATIVE_SEH)
        # Out-of-line homes for FORCEINLINE SDK helpers clang won't inline into a
        # __try region. See kmt_native_seh_shims.c.
        target_sources(${_target} PRIVATE ${KMT_NATIVE_SEH_SHIMS})
        # Retarget to the -windows-msvc env (see the KMT_MSVC_ENV option above).
        if(KMT_MSVC_ENV)
            kmt_msvc_seh_env(${_target})
        endif()
    elseif(ARCH STREQUAL "i386" AND KMT_MSVC_ENV)
        # i386 native SEH is inseparable from the msvc-env retarget (see the note above).
        target_compile_definitions(${_target} PRIVATE _USE_NATIVE_SEH)
        target_sources(${_target} PRIVATE ${KMT_NATIVE_SEH_SHIMS})
        kmt_msvc_seh_env(${_target})
    endif()
endfunction()

#
# subdirectories containing special-purpose drivers
#
add_subdirectory(example)
add_subdirectory(fltmgr)
add_subdirectory(hidparse)
add_subdirectory(kernel32)
add_subdirectory(ntos_cc)
add_subdirectory(ntos_io)
add_subdirectory(ntos_mm)
add_subdirectory(ntos_po)
add_subdirectory(tcpip)

list(APPEND COMMON_SOURCE
    example/GuardedMemory.c
    rtl/RtlAvlTree.c
    rtl/RtlException.c
    ../apitests/rtl/RtlIntSafe.c
    rtl/RtlMemory.c
    rtl/RtlRegistry.c
    rtl/RtlSplayTree.c
    rtl/RtlStack.c
    rtl/RtlStrSafe.c
    rtl/RtlUnicodeString.c)

if(ARCH STREQUAL "i386")

elseif(ARCH STREQUAL "amd64")
    add_asm_files(kmtest_common_asm ../apitests/rtl/amd64/RtlCaptureContext-asm.s)
    list(APPEND COMMON_SOURCE
        ../apitests/rtl/amd64/RtlCaptureContext.c
        ${kmtest_common_asm}
    )
endif()

if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    # Native-SEH support (see the _USE_NATIVE_SEH opt-ins below): out-of-line
    # homes for FORCEINLINE SDK helpers clang won't inline into __try regions.
    list(APPEND COMMON_SOURCE kmt_native_seh_shims.c)
endif()

#
# kmtest_drv.sys driver
#
list(APPEND KMTEST_DRV_SOURCE
    kmtest_drv/guid.c
    kmtest_drv/kmtest_drv.c
    kmtest_drv/structs.c
    kmtest_drv/testlist.c
    kmtest_drv/vm_detect.c

    example/Example.c
    example/KernelType.c
    hal/HalSystemInfo.c
    npfs/NpfsConnect.c
    npfs/NpfsCreate.c
    npfs/NpfsFileInfo.c
    npfs/NpfsHelpers.c
    npfs/NpfsReadWrite.c
    npfs/NpfsVolumeInfo.c
    novp_fsrtl/FsRtlRemoveDotsFromPath.c
    ntos_cm/CmSecurity.c
    ntos_ex/ExCallback.c
    ntos_ex/ExDoubleList.c
    ntos_ex/ExFastMutex.c
    ntos_ex/ExHardError.c
    ntos_ex/ExInterlocked.c
    ntos_ex/ExPool2.c
    ntos_ex/ExPools.c
    ntos_ex/ExPushLock.c
    ntos_ex/ExResource.c
    ntos_ex/ExResourceBounded.c
    ntos_ex/ExSequencedList.c
    ntos_ex/ExSingleList.c
    rtl/RtlUtf8KM.c
    ntos_ex/ExTimer.c
    ntos_ex/ExUuid.c
    ntos_fsrtl/FsRtlDissect.c
    ntos_fsrtl/FsRtlExpression.c
    ntos_fsrtl/FsRtlLegal.c
    ntos_fsrtl/FsRtlMcb.c
    ntos_fsrtl/FsRtlTunnel.c
    ntos_io/IoCreateFile.c
    ntos_io/IoDeviceInterface.c
    ntos_io/IoDeviceProperty.c
    ntos_io/IoEvent.c
    ntos_io/IoFilesystem.c
    ntos_io/IoInterrupt.c
    ntos_io/IoIrp.c
    ntos_io/IoMdl.c
    ntos_io/IoVolume.c
    ntos_kd/KdSystemDebugControl.c
    ntos_ke/KeApc.c
    ntos_ke/KeDevQueue.c
    ntos_ke/KeDpc.c
    ntos_ke/KeEvent.c
    ntos_ke/KeFloatPointState.c
    ntos_ke/KeGuardedMutex.c
    ntos_ke/KeIrql.c
    ntos_ke/KeMutex.c
    ntos_ke/KeProcessor.c
    ntos_ke/KeSpinLock.c
    ntos_ke/KeTimer.c
    ntos_mm/MmMapIoSpaceEx.c
    ntos_mm/MmMdl.c
    ntos_mm/MmReservedMapping.c
    ntos_mm/MmSection.c
    ntos_mm/ZwAllocateVirtualMemory.c
    ntos_mm/ZwCreateSection.c
    ntos_mm/ZwMapViewOfSection.c
    ntos_ob/ObHandle.c
    ntos_ob/ObQuery.c
    ntos_ob/ObReference.c
    ntos_ob/ObSecurity.c
    ntos_ob/ObSymbolicLink.c
    ntos_ob/ObType.cpp
    ntos_ob/ObTypes.cpp
    ntos_ob/ObWait.c
    ntos_ps/PsNotify.c
    ntos_ps/PsQuota.c
    ntos_se/SeHelpers.c
    ntos_se/SeInheritance.c
    ntos_se/SeLogonSession.c
    ntos_se/SeQueryInfoToken.c
    ntos_se/SeTokenFiltering.c
    rtl/RtlIsValidOemCharacter.c
    rtl/RtlRangeList.c
    ${COMMON_SOURCE}

    kmtest_drv/kmtest_drv.rc)

if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
    add_asm_files(KMTEST_DRV_ASM
        kmtest_drv/vm_detect.S)
    list(APPEND KMTEST_DRV_SOURCE
        ${KMTEST_DRV_ASM}
        hal/HalPortIo.c)
endif()

add_library(kmtest_drv MODULE ${KMTEST_DRV_SOURCE})
set_module_type(kmtest_drv kernelmodedriver)
# Real clang SEH lowers on amd64/arm64 only (base/system/sehtest, dummy-pseh-verdict.md
# §C); pseh2.h's _USE_NATIVE_SEH branch outranks the global _USE_DUMMY_PSEH define, so
# SEH-dependent tests get real __try/__except semantics instead of dead handlers.
# i386 stays dummy-PSEH (its SEH lowering is MSVC-env-only) — expect a crash/fail
# bucket there, baselined per-arch.
if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    target_compile_definitions(kmtest_drv PRIVATE _USE_NATIVE_SEH)
    # kmtest_drv hosts RtlExceptionKM, which faults DIRECTLY in the __try body
    # ((void)*(volatile CHAR *)NULL). Without the msvc env + -fasync-exceptions that
    # __except is compiled away entirely and the AV escapes as BUGCHECK 0x1E — which
    # is exactly the quarantine we carried for it. The 17 standalone helper drivers
    # already got this via kmt_enable_native_seh; this target never did.
    if(KMT_MSVC_ENV)
        kmt_msvc_seh_env(kmtest_drv)
    endif()
elseif(ARCH STREQUAL "i386" AND KMT_MSVC_ENV)
    target_compile_definitions(kmtest_drv PRIVATE _USE_NATIVE_SEH)
    target_sources(kmtest_drv PRIVATE ${KMT_NATIVE_SEH_SHIMS})
    kmt_msvc_seh_env(kmtest_drv)
endif()
target_link_libraries(kmtest_drv kmtest_printf chkstk memcmp ntoskrnl_vista ${PSEH_LIB})
add_importlibs(kmtest_drv ntoskrnl hal)
add_dependencies(kmtest_drv bugcodes xdk)
target_compile_definitions(kmtest_drv PRIVATE KMT_KERNEL_MODE NTDDI_VERSION=NTDDI_WS03SP1)
#add_pch(kmtest_drv include/kmt_test.h)
add_rostests_file(TARGET kmtest_drv)

add_library(kmtest_printf
    kmtest_drv/printf_stubs.c
    ${REACTOS_SOURCE_DIR}/sdk/lib/crt/string/strnlen.c
    ${REACTOS_SOURCE_DIR}/sdk/lib/crt/string/wcsnlen.c
    ${REACTOS_SOURCE_DIR}/sdk/lib/crt/printf/streamout.c)
target_compile_definitions(kmtest_printf PRIVATE _LIBCNT_ _USER32_WSPRINTF wctomb=KmtWcToMb)
target_include_directories(kmtest_printf PRIVATE ${REACTOS_SOURCE_DIR}/sdk/lib/crt/include)

#
# kmtest.exe loader application
#

list(APPEND KMTEST_SOURCE
    kmtest/filter.c
    kmtest/fltsupport.c
    kmtest/kmtest.c
    kmtest/service.c
    kmtest/support.c
    kmtest/testlist.c

    example/Example_user.c

    fltmgr/fltmgr_load/fltmgr_user.c
    fltmgr/fltmgr_register/fltmgr_reg_user.c

    hidparse/HidP_user.c
    kernel32/FileAttributes_user.c
    kernel32/FindFile_user.c
    ntos_cc/CcCopyRead_user.c
    ntos_cc/CcCopyWrite_user.c
    ntos_cc/CcMapData_user.c
    ntos_cc/CcPinMappedData_user.c
    ntos_cc/CcPinRead_user.c
    ntos_cc/CcSetFileSizes_user.c
    ntos_io/IoCreateFile_user.c
    ntos_io/IoDeviceObject_user.c
    ntos_io/IoReadWrite_user.c
    ntos_kd/NtSystemDebugControl.c
    ntos_mm/MmMapLockedPagesSpecifyCache_user.c
    ntos_mm/NtCreateSection_user.c
    ntos_po/PoIrp_user.c
    tcpip/TcpIp_user.c
    ${COMMON_SOURCE}

    kmtest/kmtest.rc)

add_executable(kmtest ${KMTEST_SOURCE})
set_module_type(kmtest win32cui)
if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    # Same native-SEH opt-in as kmtest_drv above (user-mode dispatch proven by sehtest)
    target_compile_definitions(kmtest PRIVATE _USE_NATIVE_SEH)
    # RtlException is the USER-MODE twin of RtlExceptionKM and faults the same way,
    # in-body — so it needs the same async-exception coverage. It self-crashes
    # (NO-END exit=128) rather than bugchecking, but the cause is identical.
    if(KMT_MSVC_ENV)
        kmt_msvc_seh_env(kmtest)
    endif()
elseif(ARCH STREQUAL "i386" AND KMT_MSVC_ENV)
    target_compile_definitions(kmtest PRIVATE _USE_NATIVE_SEH)
    kmt_msvc_seh_env(kmtest)
endif()
target_link_libraries(kmtest ${PSEH_LIB})
add_importlibs(kmtest fltlib advapi32 ws2_32 msvcrt kernel32 ntdll)
target_compile_definitions(kmtest PRIVATE KMT_USER_MODE NTDDI_VERSION=NTDDI_WS03SP1)
# vibeos: tee results + BEGIN/END markers to kernel debug serial (host capture)
target_compile_definitions(kmtest PRIVATE VIBEOS_KMTEST_SERIAL)
#add_pch(kmtest include/kmt_test.h)
# RESTORED 2026-09-21 (upstream-sync). Upstream dropped this rename in the merge, so the
# binary became kmtest.exe. Our conformance harness — tools/payloads/kmtest/sweep.c, and
# the three COMMITTED kmtest-sweep-<arch>.exe built from it — launches
# %SystemRoot%\bin\kmtest_.exe by name, so every one of the 91 tests came back
# "launch FAILED err=2" (ERROR_FILE_NOT_FOUND) with the sweep itself reporting success.
set_target_properties(kmtest PROPERTIES OUTPUT_NAME "kmtest_")
add_rostests_file(TARGET kmtest)

#
# Group targets
#
add_custom_target(kmtest_drivers)
add_dependencies(kmtest_drivers
    kmtest_drv
    example_drv
    hidp_drv
    iocreatefile_drv
    iodeviceobject_drv
    iohelper_drv
    ioreadwrite_drv
    kernel32_drv
    mmmaplockedpagesspecifycache_drv
    ntcreatesection_drv
    poirp_drv
    tcpip_drv
    cccopyread_drv
    ccmapdata_drv)

add_custom_target(kmtest_all)
add_dependencies(kmtest_all kmtest_drivers kmtest)
