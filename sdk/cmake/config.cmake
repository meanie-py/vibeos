
set(SARCH "" CACHE STRING "Sub-architecture (unused; legacy SARCH values pc/pc98/xbox/omap3-* removed).")

if(ARCH STREQUAL "i386")
    set(OARCH "pentium" CACHE STRING
    "Generate instructions for this CPU type. Specify one of:
     pentium, pentiumpro")
elseif(ARCH STREQUAL "amd64")
    set(OARCH "athlon64" CACHE STRING
    "Generate instructions for this CPU type. Specify one of:
     k8 opteron athlon64 athlon-fx")
elseif(ARCH STREQUAL "arm64")
    # This should not be bumped unless REALLY needed, because (as of 2021)
    # there are still new designs using the original A53 cores w/ armv8.0.
    set(OARCH "armv8-a" CACHE STRING
    "Generate instructions for this CPU type. Specify one of:
     armv8-a armv8.1-a armv8.2-a armv8.3-a armv8.4-a armv8.5-a armv8.6-a")
endif()

set(TUNE "generic" CACHE STRING
"Which CPU ReactOS should be optimized for.")

set(OPTIMIZE "4" CACHE STRING
"What level of optimization to use.
 0 = Off
 1 = Optimize for size (-Os) with some additional options
 2 = Optimize for size (-Os)
 3 = Optimize debugging experience (-Og)
 4 = Optimize (-O1)
 5 = Optimize even more (-O2)
 6 = Optimize yet more (-O3)
 7 = Disregard strict standards compliance (-Ofast)")

set(LTCG FALSE CACHE BOOL
"Whether to build with link-time code generation")

set(GDB FALSE CACHE BOOL
"Whether to use by default KDGDB.DLL instead of KDCOM.DLL for debugging with GDB.
Mainly used for cloud-based ReactOS development using Gitpod and Docker.
If you don't use GDB, don't enable this.")

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(DBG FALSE CACHE BOOL
"Whether to compile for debugging.")
else()
    set(DBG TRUE CACHE BOOL
"Whether to compile for debugging.")
endif()

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(KDBG FALSE CACHE BOOL "Whether to compile in the integrated kernel debugger.")
else()
    set(KDBG TRUE CACHE BOOL "Whether to compile in the integrated kernel debugger.")
endif()
set(_WINKD_ FALSE CACHE BOOL "Whether to compile with the KD protocol.")

cmake_dependent_option(ISAPNP_ENABLE "Whether to enable the ISA PnP support." ON
                       "ARCH STREQUAL i386" OFF)

set(GENERATE_DEPENDENCY_GRAPH FALSE CACHE BOOL
"Whether to create a GraphML dependency graph of DLLs.")

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    option(STACK_PROTECTOR "Whether to enable the GCC stack checker while compiling" OFF)
endif()

set(USE_DUMMY_PSEH FALSE CACHE BOOL
"Whether to disable PSEH support.")

set(DLL_EXPORT_VERSION "0x502" CACHE STRING
"The NT version the user mode DLLs target.")

set(REACTOS_TEST_KERNEL FALSE CACHE BOOL
"Whether to compile the in-kernel test runner (test builds only).")
