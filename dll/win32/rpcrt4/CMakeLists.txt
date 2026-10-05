
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

add_definitions(
    -D_RPCRT4_
    -DCOM_NO_WINDOWS_H
    -D_CRT_NON_CONFORMING_SWPRINTFS
    -DMSWMSG)

spec2def(rpcrt4.dll rpcrt4.spec ADD_IMPORTLIB)

add_rpc_files(client epm.idl)
add_idl_headers(ndr_types_header ndr_types.idl)

set(OLD_IDL_FLAGS ${IDL_FLAGS})
set(IDL_FLAGS ${IDL_FLAGS} -Oicf)
add_rpcproxy_files(ndr_types.idl)
set(IDL_FLAGS ${OLD_IDL_FLAGS})

list(APPEND SOURCE
    cproxy.c
    cpsf.c
    ndr_clientserver.c
    ndr_contexthandle.c
    ndr_es.c
    ndr_fullpointer.c
    ndr_marshall.c
    ndr_ole.c
    ros_extra.c
    rpc_assoc.c
    rpc_async.c
    rpc_binding.c
    rpc_message.c
    rpcrt4_main.c
    rpc_transport.c
    thunks.c
    unix_func.c
    ${CMAKE_CURRENT_BINARY_DIR}/epm_c.c)

# Native SEH for the TUs whose __TRY/__EXCEPT is the RPC error contract
# (dummy-pseh-verdict.md, "Consequence for RPC clients"). Under the global DUMMY_PSEH
# mode none of these handlers runs: a failed COM/RPC call's RpcRaiseException is
# resumed by the dispatcher, RaiseException returns, and RpcRaiseException ends in
# ExitProcess(1), so any client of a dead or refusing server died (Wine's ole32
# marshal test: a proxy called from the wrong apartment). Every try body here is
# call-shaped (the raise lands in a callee), the same contract as the advapi32
# EventLog shim. They skip the PCH, which would otherwise pin pseh2.h to DUMMY.
# i386 keeps DUMMY: clang i386 native SEH is unreliable.
set(RPCRT4_SEH_SOURCE
    cstub.c
    ndr_stubless.c
    rpc_epmap.c
    rpc_server.c)
if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    set_source_files_properties(${RPCRT4_SEH_SOURCE} PROPERTIES COMPILE_DEFINITIONS _USE_NATIVE_SEH)
endif()

list(APPEND PCH_SKIP_SOURCE
    ${RPCRT4_SEH_SOURCE}
    ndr_typelib.c
    thunks.c
    ${CMAKE_CURRENT_BINARY_DIR}/ndr_types_p.c
    ${CMAKE_CURRENT_BINARY_DIR}/proxy.dlldata.c
    ${CMAKE_CURRENT_BINARY_DIR}/rpcrt4_stubs.c)

add_library(rpcrt4 MODULE
    ${SOURCE}
    ${rpcrt4_asm}
    ${PCH_SKIP_SOURCE}
    rpcrt4.rc
    ${CMAKE_CURRENT_BINARY_DIR}/rpcrt4.def)

set_module_type(rpcrt4 win32dll)
target_link_libraries(rpcrt4 wine wine_dll_register uuid ${PSEH_LIB} oldnames)
add_delay_importlibs(rpcrt4 iphlpapi wininet secur32 user32 ole32 oleaut32)
add_importlibs(rpcrt4 advapi32 advapi32_vista kernel32_vista ws2_32 msvcrt kernel32 ntdll)
add_dependencies(rpcrt4 ndr_types_header)
add_pch(rpcrt4 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET rpcrt4 DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(rpcrt4) # CORE-5743: No ARRAY_SIZE and CONST_VTABLE
