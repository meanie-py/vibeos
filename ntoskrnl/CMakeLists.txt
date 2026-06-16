
PROJECT(NTOS)

if (CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Enable this again. CORE-17637
    add_compile_options(-Wunused-result)
endif()

include(ntos.cmake)

set(NTOSKRNL_SOURCE ${SOURCE})
set(NTOSKRNL_ASM_SOURCE ${ASM_SOURCE})

set(NTKRNLMP_SOURCE ${SOURCE})
set(NTKRNLMP_ASM_SOURCE ${ASM_SOURCE})

spec2def(ntoskrnl.exe ntoskrnl.spec ADD_IMPORTLIB)

# Link GCC stack protector lib
if (STACK_PROTECTOR)
    target_link_libraries(libntoskrnl INTERFACE gcc_ssp_nt)
endif()

add_asm_files(ntoskrnl_asm ${NTOSKRNL_ASM_SOURCE})

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Clang optimises strcmp calls to memcmp.
    target_link_libraries(libntoskrnl INTERFACE memcmp)
endif()

if((CMAKE_C_COMPILER_ID STREQUAL "Clang") OR (CMAKE_C_COMPILER_ID STREQUAL "GNU"))
    # Enable xsave feature (x86/AMD64 only)
    if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
        add_compile_options(-mxsave)
    endif()
endif()

# UART traces for ARM64 boot debugging (disabled — too verbose for normal operation)
# if(ARCH STREQUAL "arm64")
#     add_definitions(-DARM64_UART_TRACE_ENABLE)
# endif()

# guid.c — replaced by Rust (ntoskrnl-rust/src/guid.rs)
#list(APPEND PCH_SKIP_SOURCE
#    guid.c)

list(APPEND PCH_SKIP_SOURCE
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/ke_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/variadic_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/lto_workaround.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm_lock_outlines.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/mm_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/io_rust_shims.c)
# kdbg_rust_shims.c is only built on i386/amd64 (KDBG is disabled on ARM64).
# Add to PCH_SKIP_SOURCE conditionally to mirror the SOURCE-list gating in
# ntos.cmake.
if(KDBG AND NOT ARCH STREQUAL "arm64")
    list(APPEND PCH_SKIP_SOURCE
        ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/kdbg_rust_shims.c)
endif()

# Rust kernel code — single umbrella staticlib (pool + ob/obhandle + future modules)
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(RUST_KERNEL_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kernel-target")
if(ARCH STREQUAL "i386")
    set(RUST_TARGET "i686-pc-windows-gnu")
    # i386: disable SSE — CR4.OSFXSR not set until Phase 1 but pool init runs in Phase 0
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(RUST_TARGET "x86_64-pc-windows-gnu")
    set(RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RUST_EXTRA_ENV "")
endif()

set(RUST_KERNEL_LIB "${CMAKE_CURRENT_BINARY_DIR}/libkernel_rust.a")
add_custom_command(
    OUTPUT ${RUST_KERNEL_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_KERNEL_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p kernel-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_KERNEL_TARGET_DIR}/${RUST_TARGET}/release/libkernel_rust.a
        ${RUST_KERNEL_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/kernel-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/kernel-rust/Cargo.toml            ${RUST_WORKSPACE_DIR}/ke-macros/src/lib.rs
            ${RUST_WORKSPACE_DIR}/ke-macros/Cargo.toml
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/shim_exports.rs
            ${RUST_WORKSPACE_DIR}/cmboot-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/cmboot-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/inline/mod.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/inline/se.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/inline/common.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/inline/ke.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/inline/mm.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/inline/ex.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/inline/ob.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ob/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ob/obhandle.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/access.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/accesschk.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/acl.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/audit.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/client.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/debug.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/objtype.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/priv_.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/sd.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/semgr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/sid.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/sqos.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/srm.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/subject.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/token.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/tokenadj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/tokencls.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/tokenlif.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/apphelp.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/debug.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/job.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/kill.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/info_class.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/process.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/psmgr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/psnotify.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/query.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/quota.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/security.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/state.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/thread.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/win32.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/config/cmdata.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/close.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/complete.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/connect.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/create.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/listen.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/port.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/reply.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/send.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/dbgk/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/dbgk/dbgkobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/dbgk/dbgkutil.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/events.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/poshtdwn.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/povolume.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/power.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/bootanim.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/inbv.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/inbvport.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/guidobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/smbios.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/wmi.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/wmidrv.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vdm/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vdm/vdmexec.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vdm/vdmmain.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vf/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vf/driver.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/rtl/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/rtl/libsupp.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/rtl/misc.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ARM3/kdbg.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ARM3/wslist.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/i386/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/i386/page.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/i386/procsup.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/i386/init.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/amd64/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/amd64/init.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/amd64/procsup.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/arm64/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/arm64/page.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/arm/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/arm/init.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/atom.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/callback.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/dbgctrl.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/efi.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/event.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/evtpair.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/exintrin.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/fmutex.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/handle.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/harderr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/hdlsterm.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/init.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/interlocked.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/keyedevt.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/locale.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/lookas.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/mutant.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/profile.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/pushlock.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/resource.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/rundown.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/sem.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/shutdown.rs
            # ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/sysinfo.rs  # kept in C
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/time.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/timer.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/uuid.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/win32k.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/work.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/xipdisp.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ex/zone.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/apc.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/balmgr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/bug.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/clock.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/config.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/devqueue.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/dpc.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/eventobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/except.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/freeze.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/gate.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/gmutex.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/ipi.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/krnlinit.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/mutex.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/processor.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/procobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/profobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/queue.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/semphobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/spinlock.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/thrdobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/thrdschd.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/time.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/timerobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/wait.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/abios.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/context.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/cpu.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/exp.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/freeze.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/irqobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/kiinit.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/ldt.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/mproc.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/mtrr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/patpge.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/thrdini.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/traphdlr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/usercall.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/i386/v86vdm.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/context.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/cpu.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/except.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/freeze.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/interrupt.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/ipi.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/irql.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/kiinit.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/krnlinit.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/mproc.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/spinlock.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/stubs.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/thrdini.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/traphandler.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/usercall.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/amd64/xstate.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/arm64/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/arm64/kiinit.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/arm64/stubs.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/arm64/thrdini.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/arm64/trap.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ke/arm64/usercall.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/lib.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/base.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/mm.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ke.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ex.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ps.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ob.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/rtl.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/interlocked.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/constants.rs
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust kernel code (${RUST_TARGET})"
)
add_custom_target(rust_kernel DEPENDS ${RUST_KERNEL_LIB})

# ntoskrnl.exe: link driven by cargo (graduated from cmake add_executable).
# ntoskrnl-cargo/build.rs (reactos-link) self-hosts the kernel-rust staticlib +
# the cmake C-floor static libs (ntoskrnl_vista/cportlib/csq/pseh/arbiter/cmlib/
# lsalib/rtl/rossym/setjmp/wdmguid/poguid/ioevent/hal/kdcom/bootvid, + cnt-rust
# replacing libcntpr) + self-assembled asm + spec2def + the .res, replacing
# add_executable + set_module_type(kernel) + target_link_libraries + add_pch.
# KIND kernelexe → native-pefixup --kernel + i386 forwarder fix. Proven ×3
# boot-equivalent via ntoskrnl-cargo/boot-swap-smoke.sh. The spec2def(...
# ADD_IMPORTLIB) at the top of this file STAYS — the tree imports libntoskrnl.
# EXTRA_DEPENDS == the OLD ntoskrnl dependency closure (target_link_libraries +
# add_importlibs + add_dependencies), so a cold build can't race a stale .a /
# generated header the cargo link reuses by path. (rust_kernel kept as a dep:
# the cargo build links its OWN kernel-rust, but the cmake target's build is the
# proven-sufficient source of any generated prerequisites; revisit to drop it.)
add_cargo_module(ntoskrnl ntoskrnl-cargo ntoskrnl.exe
    KIND kernelexe
    BODY_CRATE kernel-rust
    EXTRA_DEPENDS
        ntoskrnl_vista cportlib csq ${PSEH_LIB} arbiter cmlib ntlsalib rtl
        ${ROSSYM_LIB} libcntpr setjmp wdmguid poguid ioevent
        hal kdcom bootvid
        # The cargo link (reactos-link) consumes kdcom/bootvid's *import libs*
        # (CMakeFiles/lib{kdcom,bootvid}.dir/lib*.a) by path. The DLL targets
        # above do NOT build their own import libs (only consumers do), and
        # nothing else in the kernel's closure pulls them — so a from-scratch
        # `ninja -t clean` build leaves them missing and the cargo link panics
        # ("missing static lib … run ninja first"). Depend on the implib custom
        # targets explicitly so a cold build always produces them first.
        libkdcom_implib_target libbootvid_implib_target
        psdk asm rust_kernel)

add_subdirectory(ntkrnlmp)

add_asm_files(ntdllsys_asm ntdll.S)
add_library(ntdllsys ${ntdllsys_asm})
set_target_properties(ntdllsys PROPERTIES LINKER_LANGUAGE "C")
add_dependencies(ntdllsys asm)
