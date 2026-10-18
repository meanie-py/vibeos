
#idl files support
if(ARCH STREQUAL "i386")
    set(IDL_FLAGS -m32 --win32 -b i386-x-y -nostdinc)
elseif(ARCH STREQUAL "amd64")
    set(IDL_FLAGS -m64 --win64 -b amd64-x-y -nostdinc)
else()
    set(IDL_FLAGS -b ${ARCH}-x-y -nostdinc)
endif()

function(add_typelib)
    get_includes(INCLUDES)
    get_defines(DEFINES)
    get_includes(LIBS)
    string(REPLACE "-I" "-L" LIBS "${LIBS}")

    foreach(FILE ${ARGN})
        if(${FILE} STREQUAL "std_ole_v1.idl")
            set(IDL_FLAGS ${IDL_FLAGS} --oldtlb)
        endif()
        get_filename_component(NAME ${FILE} NAME_WE)
        add_custom_command(
            OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${NAME}.tlb
            COMMAND native-widl ${INCLUDES} ${LIBS} ${DEFINES} ${IDL_FLAGS} -t -o ${CMAKE_CURRENT_BINARY_DIR}/${NAME}.tlb ${CMAKE_CURRENT_SOURCE_DIR}/${FILE}
            DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${FILE} native-widl)
        list(APPEND OBJECTS ${CMAKE_CURRENT_BINARY_DIR}/${NAME}.tlb)
    endforeach()
endfunction()

function(add_idl_headers TARGET)
    get_includes(INCLUDES)
    get_defines(DEFINES)
    foreach(FILE ${ARGN})
        get_filename_component(NAME ${FILE} NAME_WLE)
        set(HEADER ${CMAKE_CURRENT_BINARY_DIR}/${NAME}.h)
        add_custom_command(
            OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${NAME}.h
            COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} -Oicf -h -o ${CMAKE_CURRENT_BINARY_DIR}/${NAME}.h ${FILE}
            DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${FILE} native-widl
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
        list(APPEND HEADERS ${HEADER})
    endforeach()
    add_custom_target(${TARGET} DEPENDS ${HEADERS})
endfunction()

function(add_rpcproxy_files)
    get_includes(INCLUDES)
    get_defines(DEFINES)

    foreach(FILE ${ARGN})
        get_filename_component(NAME ${FILE} NAME_WLE)
        # Most proxy idl's have names like <proxyname>_<original>.idl
        # We use this to create a dependency from the proxy to the original idl
        string(REPLACE "_" ";" SPLIT_FILE ${FILE})
        list(LENGTH SPLIT_FILE len)
        unset(EXTRA_DEP)
        if(len STREQUAL "2")
            list(GET SPLIT_FILE 1 SPLIT_FILE)
            if(EXISTS "${REACTOS_SOURCE_DIR}/sdk/include/psdk/${SPLIT_FILE}")
                set(EXTRA_DEP ${REACTOS_SOURCE_DIR}/sdk/include/psdk/${SPLIT_FILE})
            endif()
        endif()
        list(APPEND IDLS ${FILE})
        list(APPEND IDL_DEPS ${CMAKE_CURRENT_SOURCE_DIR}/${FILE} ${EXTRA_DEP})
        add_custom_command(
            OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${NAME}_p.c ${CMAKE_CURRENT_BINARY_DIR}/${NAME}_p.h
            # We generate the two files in two passes because WIDL accepts only one custom file name as output
            COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} -Oicf -p -o ${CMAKE_CURRENT_BINARY_DIR}/${NAME}_p.c -H ${NAME}_p.h ${FILE}
            COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} -Oicf -h -o ${CMAKE_CURRENT_BINARY_DIR}/${NAME}_p.h ${FILE}
            DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${FILE} ${EXTRA_DEP} native-widl
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
    endforeach()

    # Extra pass to generate dlldata
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/proxy.dlldata.c
        COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} -Oicf --dlldata-only -o ${CMAKE_CURRENT_BINARY_DIR}/proxy.dlldata.c ${IDLS}
        DEPENDS ${IDL_DEPS} native-widl
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
endfunction()

function(add_rpc_files __type)
    get_includes(INCLUDES)
    get_defines(DEFINES)
    set(__additional_flags -Oif)
    # Is it a client or server module?
    if(__type STREQUAL "server")
        set(__server_client -s)
        set(__suffix _s)
    elseif(__type STREQUAL "client")
        set(__server_client -c)
        set(__suffix _c)
    else()
        message(FATAL_ERROR "Please pass either server or client as argument to add_rpc_files")
    endif()
    foreach(FILE ${ARGN})
        get_filename_component(__name ${FILE} NAME_WLE)
        set(__name ${__name}${__suffix})
        add_custom_command(
            OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${__name}.c ${CMAKE_CURRENT_BINARY_DIR}/${__name}.h
            # We generate the two files in two passes because WIDL accepts only one custom file name as output
            COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} -Oicf ${__additional_flags} ${__server_client} -o ${CMAKE_CURRENT_BINARY_DIR}/${__name}.c -H ${__name}.h ${FILE}
            COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} -Oicf ${__additional_flags} -h -o ${CMAKE_CURRENT_BINARY_DIR}/${__name}.h ${FILE}
            DEPENDS ${FILE} native-widl
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
    endforeach()
endfunction()

# SEH-guarded wrappers for a client stub set: add_rpc_guards(<idl stem> [prelude.h ...])
# generates <stem>_guard.{c,h} from <stem>_c.h, and returns the .c in <stem>_GUARD_SOURCE
# for the caller's source list. Under DUMMY_PSEH a C RpcTryExcept/__TRY is a no-op on
# every arch; <fn>_guarded runs the stub under ntdll's RtlSehTryExcept instead.
# tools/gen_rpc_guards.py explains the contract.
function(add_rpc_guards __stem)
    set(__bin ${CMAKE_CURRENT_BINARY_DIR})
    set(__preludes)
    foreach(__inc ${ARGN})
        list(APPEND __preludes --prelude ${__inc})
    endforeach()
    add_custom_command(
        OUTPUT ${__bin}/${__stem}_guard.c ${__bin}/${__stem}_guard.h
        COMMAND ${CMAKE_COMMAND} -E env python3 ${REACTOS_SOURCE_DIR}/tools/gen_rpc_guards.py
            --header ${__bin}/${__stem}_c.h ${__preludes}
            --out-c ${__bin}/${__stem}_guard.c --out-h ${__bin}/${__stem}_guard.h
        DEPENDS ${__bin}/${__stem}_c.h ${REACTOS_SOURCE_DIR}/tools/gen_rpc_guards.py)
    set(${__stem}_GUARD_SOURCE ${__bin}/${__stem}_guard.c ${__bin}/${__stem}_guard.h PARENT_SCOPE)
endfunction()

function(add_rpc_file __type __idl_file)
    cmake_parse_arguments(__args "" "ACF;PREFIX_ALL;PREFIX_CLIENT;PREFIX_SERVER" "" ${ARGN})
    get_includes(INCLUDES)
    get_defines(DEFINES)
    set(__additional_flags -Oif)
    # Is it a client or server module?
    if(__type STREQUAL "server")
        set(__server_client -s)
        set(__suffix _s)
    elseif(__type STREQUAL "client")
        set(__server_client -c)
        set(__suffix _c)
    else()
        message(FATAL_ERROR "Please pass either server or client as argument to add_rpc_files")
    endif()
    if(__args_ACF)
        set(__additional_flags ${__additional_flags} --acf=${__args_ACF})
    endif()
    if (__args_PREFIX_ALL)
        set(__additional_flags ${__additional_flags} --prefix-all=${__args_PREFIX_ALL})
    else()
        if (__args_PREFIX_CLIENT)
            set(__additional_flags ${__additional_flags} --prefix-client=${__args_PREFIX_CLIENT})
        endif()
        if (__args_PREFIX_SERVER)
            set(__additional_flags ${__additional_flags} --prefix-server=${__args_PREFIX_SERVER})
        endif()
    endif()
    get_filename_component(__name ${__idl_file} NAME_WE)
    set(__name ${__name}${__suffix})
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${__name}.c ${CMAKE_CURRENT_BINARY_DIR}/${__name}.h
        # We generate the two files in two passes because WIDL accepts only one custom file name as output
        COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} ${__additional_flags} ${__server_client} -o ${CMAKE_CURRENT_BINARY_DIR}/${__name}.c -H ${__name}.h ${__idl_file}
        COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} ${__additional_flags} -h -o ${CMAKE_CURRENT_BINARY_DIR}/${__name}.h ${__idl_file}
        DEPENDS ${__idl_file} native-widl
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
endfunction()

function(generate_idl_iids)
    foreach(IDL_FILE ${ARGN})
        get_includes(INCLUDES)
        get_defines(DEFINES)
        get_filename_component(NAME ${IDL_FILE} NAME_WE)
        add_custom_command(
            OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${NAME}_i.c
            COMMAND native-widl ${INCLUDES} ${DEFINES} ${IDL_FLAGS} -Os -u -o ${CMAKE_CURRENT_BINARY_DIR}/${NAME}_i.c ${IDL_FILE}
            DEPENDS ${IDL_FILE} native-widl
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
    endforeach()
endfunction()

function(add_iid_library TARGET)
    foreach(IDL_FILE ${ARGN})
        get_filename_component(NAME ${IDL_FILE} NAME_WE)
        generate_idl_iids(${IDL_FILE})
        list(APPEND IID_SOURCES ${NAME}_i.c)
    endforeach()
    add_library(${TARGET} ${IID_SOURCES})
    add_dependencies(${TARGET} psdk)
    set_target_properties(${TARGET} PROPERTIES EXCLUDE_FROM_ALL TRUE)
endfunction()

# Generates Wine .rgs registry files for DllRegisterServer. Automatically adds dependency and generated res files to target.
# The registry/typelib resource objects alone: <idl><suffix>.o for each IDL, under the custom
# target <TARGET><suffix>, attached to nothing. add_idl_reg_scripts attaches them to a cmake
# target; a cargo cell (add_cargo_module) links ${CMAKE_CURRENT_BINARY_DIR}/<idl><suffix>.o
# itself and lists <TARGET><suffix> in its EXTRA_DEPENDS.
function(add_idl_reg_objects TARGET TYPE)
    get_includes(INCLUDES)
    get_defines(DEFINES)
    get_includes(LIBS)
    string(REPLACE "-I" "-L" LIBS "${LIBS}")

    if(TYPE STREQUAL "winrt")
        set(IDL_REGFLAGS -r -winrt)
        set(__suffix _w)
    elseif(TYPE STREQUAL "registry")
        set(IDL_REGFLAGS -r)
        set(__suffix _r)
    elseif(TYPE STREQUAL "regtypelib")
        set(IDL_REGFLAGS -t)
        set(__suffix _t)
    else()
        message(FATAL_ERROR "Please pass either winrt, registry or regtypelib as argument to add_idl_reg_scripts")
    endif()
    set(IDL_REGTARGETNAME ${TARGET}${__suffix})
    foreach(IDL_FILE ${ARGN})
        get_filename_component(NAME ${IDL_FILE} NAME_WE)
        add_custom_command(
            OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.res
            COMMAND native-widl ${INCLUDES} ${DEFINES} ${LIBS} ${IDL_FLAGS} -Oicf ${IDL_REGFLAGS} -o ${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.res ${IDL_FILE}
            DEPENDS ${IDL_FILE} native-widl
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
        set_source_files_properties(${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.res PROPERTIES
            GENERATED TRUE EXTERNAL_OBJECT TRUE)

        # convert the .res file into a .o file for GCC/Clang using windres
        add_custom_command(
            OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.o
            COMMAND ${CMAKE_RC_COMPILER} -O coff -i ${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.res -o ${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.o
            DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.res
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
        set_source_files_properties(${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.o PROPERTIES
            GENERATED TRUE EXTERNAL_OBJECT TRUE)
        list(APPEND RCFILES ${CMAKE_CURRENT_BINARY_DIR}/${NAME}${__suffix}.o)
    endforeach()
        
    add_custom_target(${IDL_REGTARGETNAME} DEPENDS ${RCFILES})
    add_dependencies(${IDL_REGTARGETNAME} stdole2)
    set(IDL_REG_OBJECTS ${RCFILES} PARENT_SCOPE)
    set(IDL_REG_TARGET ${IDL_REGTARGETNAME} PARENT_SCOPE)
endfunction()

function(add_idl_reg_scripts TARGET TYPE)
    add_idl_reg_objects(${TARGET} ${TYPE} ${ARGN})
    add_dependencies(${TARGET} ${IDL_REG_TARGET})
    target_sources(${TARGET} PRIVATE ${IDL_REG_OBJECTS})
endfunction()
