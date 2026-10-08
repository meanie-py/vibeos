
#
# Define three Delay-Import binary test programs:
# - "nohook": delayload is used, but neither of the hook/failurehook is defined;
# - "globalhook": delayload is used, and the hooks are assigned at global scope;
# - "runtimehook": delayload is used, and the hooks are assigned at runtime.
#
list(APPEND DELAYIMP_TEST_NAME nohook globalhook runtimehook)
list(APPEND DELAYIMP_TEST_INDEX 0 1 2)
foreach(testname testid IN ZIP_LISTS DELAYIMP_TEST_NAME DELAYIMP_TEST_INDEX)
    add_executable(delayimp_${testname}_apitest delayimp.cpp)
    target_compile_definitions(delayimp_${testname}_apitest PRIVATE DELAYIMP_TEST=${testid})
    # The test recovers from the delay helper's VcppException in an _SEH2_EXCEPT filter
    # (it patches pfnCur and returns EXCEPTION_CONTINUE_EXECUTION). Under the default
    # _USE_DUMMY_PSEH that filter never runs, so give these binaries real SEH, the
    # kmtests' way: _USE_NATIVE_SEH plus the -windows-msvc retarget, without which
    # clang will not lower a frame-based i386 __try (see kmt_msvc_seh_env).
    target_compile_definitions(delayimp_${testname}_apitest PRIVATE _USE_NATIVE_SEH)
    if(ARCH STREQUAL "i386")
        target_compile_options(delayimp_${testname}_apitest PRIVATE --target=i686-pc-windows-msvc -fasync-exceptions)
    elseif(ARCH STREQUAL "amd64")
        target_compile_options(delayimp_${testname}_apitest PRIVATE --target=x86_64-pc-windows-msvc -fasync-exceptions)
    elseif(ARCH STREQUAL "arm64")
        target_compile_options(delayimp_${testname}_apitest PRIVATE --target=aarch64-pc-windows-msvc -fasync-exceptions)
    endif()
    set_module_type(delayimp_${testname}_apitest win32cui)
    target_link_libraries(delayimp_${testname}_apitest ${PSEH_LIB})
    add_importlibs(delayimp_${testname}_apitest msvcrt kernel32)
    add_delay_importlibs(delayimp_${testname}_apitest winmm version dbghelp shlwapi sfc_os imagehlp)
    add_rostests_file(TARGET delayimp_${testname}_apitest)
endforeach()

#add_executable(sdk_apitest testlist.c)
#set_module_type(sdk_apitest win32cui)
#target_link_libraries(sdk_apitest ${PSEH_LIB})
#add_importlibs(sdk_apitest msvcrt kernel32 ntdll)
#add_rostests_file(TARGET sdk_apitest)
