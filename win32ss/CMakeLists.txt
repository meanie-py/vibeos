
set(USE_DIBLIB FALSE)

# Give WIN32 subsystem its own project.
PROJECT(WIN32SS)

add_subdirectory(drivers)

if(USE_DIBLIB)
    add_subdirectory(gdi/diblib)
endif()

add_subdirectory(gdi/gdi32)
add_subdirectory(gdi/gdi32_vista)
add_subdirectory(printing)
add_subdirectory(reactx)
add_subdirectory(user/imm32)
add_subdirectory(user/user32)
add_subdirectory(user/user32_vista)
add_subdirectory(user/winsrv)
add_subdirectory(win32u)

spec2def(win32k.sys win32k.spec ADD_IMPORTLIB)

# Link GCC stack protector lib
if (STACK_PROTECTOR)
    target_link_libraries(libwin32k INTERFACE gcc_ssp_win32k)
endif()

include_directories(
    .
    include
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include
    ${REACTOS_SOURCE_DIR}/sdk/lib/3rdparty/freetype/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

add_definitions(
    -DLANGPACK
    -D_WIN32K_)

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/gdi/dib)

list(APPEND GENDIB_FILES
    ${CMAKE_CURRENT_BINARY_DIR}/gdi/dib/dib8gen.c
    ${CMAKE_CURRENT_BINARY_DIR}/gdi/dib/dib16gen.c
    ${CMAKE_CURRENT_BINARY_DIR}/gdi/dib/dib32gen.c)

add_custom_command(
    OUTPUT ${GENDIB_FILES}
    COMMAND native-gendib ${CMAKE_CURRENT_BINARY_DIR}/gdi/dib
    DEPENDS native-gendib)

list(APPEND SOURCE
    safe_user_io.c
    # gdi/dib/alphablend.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib_alphablend.rs) — cohabitation [Phase 2b dib wave 2b: DIB_XXBPP_AlphaBlend]
    # gdi/dib/dib1bpp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib1bpp.rs) — cohabitation [Phase 2b dib wave 1]
    # gdi/dib/dib4bpp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib4bpp.rs) — cohabitation [Phase 2b dib wave 1]
    # gdi/dib/dib8bpp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib8bpp.rs) — cohabitation [Phase 2b dib wave 1]
    # gdi/dib/dib16bpp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib16bpp.rs) — cohabitation [Phase 2b dib wave 2]
    # gdi/dib/dib24bpp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib24bpp.rs) — cohabitation [Phase 2b dib wave 2]
    # gdi/dib/dib32bpp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib32bpp.rs) — cohabitation [Phase 2b dib wave 2]
    # gdi/dib/floodfill.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/floodfill.rs) — cohabitation [Phase 2b dib wave 2b: DIB_XXBPP_FloodFillSolid]
    # gdi/dib/stretchblt.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib_stretchblt.rs) — cohabitation [Phase 2b dib wave 2b: DIB_XXBPP_StretchBlt]
    # gdi/eng/alphablend.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/alphablend.rs) — cohabitation [Phase 2a wave 2]
    # gdi/eng/bitblt.c     # replaced by Rust (cohabitation, bisect iter 8)
    # gdi/eng/engbrush.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/engbrush.rs) — bisect ON
    # gdi/eng/engevent.c    # replaced by Rust (sdk/lib/rust/win32k-rust/src/engevent.rs) — cohabitation
    # gdi/eng/clip.c        # replaced by Rust (sdk/lib/rust/win32k-rust/src/clip.rs) — cohabitation
    # gdi/eng/debug.c       # replaced by Rust (sdk/lib/rust/win32k-rust/src/debug.rs) — cohabitation
    # gdi/eng/device.c     # replaced by Rust (sdk/lib/rust/win32k-rust/src/device.rs) — 3 boot-critical fns resurrected 2026-05-28 (HasVga/UpdateGraphicsDeviceList/RegisterGraphicsDevice); 7 non-boot stubs remain as STATUS_SUCCESS no-ops; INTEGRATED + FAILING (i386 smoke hangs pre-`shell ready`) per AGENTS.md "broken translation as debug signal" rule
    # gdi/eng/driverobj.c   # replaced by Rust (sdk/lib/rust/win32k-rust/src/driverobj.rs) — cohabitation
    # gdi/eng/error.c       # replaced by Rust (sdk/lib/rust/win32k-rust/src/error.rs) — cohabitation
    # gdi/eng/float.c       # replaced by Rust (sdk/lib/rust/win32k-rust/src/float.rs) — cohabitation (EngSave still C, scanner gap)
    # gdi/eng/gradient.c   # replaced by Rust (cohabitation, bisect iter 12)
    # gdi/eng/lineto.c   # replaced by Rust (lineto.rs) — cohabitation [wave 2 sweep]
    gdi/eng/ldevobj.c      # ldevobj.rs breaks i386 NtUserInitialize — TBD which fn
    # gdi/eng/mapping.c     # replaced by Rust (cohabitation, bisect iter 7)
    # gdi/eng/math.c        # replaced by Rust (sdk/lib/rust/win32k-rust/src/math.rs) — cohabitation
    # gdi/eng/mdevobj.c     # replaced by Rust (sdk/lib/rust/win32k-rust/src/mdevobj.rs) — cohabitation
    # gdi/eng/mem.c     # replaced by Rust (cohabitation, bisect iter 7)
    # gdi/eng/engmisc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/engmisc.rs) — bisect ON
    # gdi/eng/mouse.c  # replaced by Rust (mouse.rs) — cohabitation [wave 2 sweep]
    # gdi/eng/multidisp.c   # replaced by Rust (sdk/lib/rust/win32k-rust/src/multidisp.rs) — cohabitation
    # gdi/eng/pandisp.c     # replaced by Rust (cohabitation, bisect iter 7)
    # gdi/eng/paint.c       # replaced by Rust (sdk/lib/rust/win32k-rust/src/paint.rs) — cohabitation
    # gdi/eng/pathobj.c     # replaced by Rust (cohabitation, bisect iter 7)
    # gdi/eng/pdevobj.c  # replaced by Rust (pdevobj.rs) — cohabitation [wave 2 sweep, per-arch raw-dylib]
    # gdi/eng/perfcnt.c     # replaced by Rust (sdk/lib/rust/win32k-rust/src/perfcnt.rs) — cohabitation
    # gdi/eng/rlecomp.c     # replaced by Rust (cohabitation, bisect iter 7)
    # gdi/eng/semaphor.c    # replaced by Rust (sdk/lib/rust/win32k-rust/src/semaphor.rs) — cohabitation (EngCreate/Acquire still C)
    # gdi/eng/sort.c        # replaced by Rust (sdk/lib/rust/win32k-rust/src/sort.rs) — cohabitation
    # gdi/eng/string.c      # replaced by Rust (sdk/lib/rust/win32k-rust/src/string.rs) — cohabitation
    # gdi/eng/stretchblt.c  # replaced by Rust (stretchblt.rs) — cohabitation [wave 2 sweep]
    # gdi/eng/surface.c     # replaced by Rust (surface.rs) — cohabitation [wave 2 sweep]
    # gdi/eng/transblt.c     # replaced by Rust (cohabitation, bisect iter 7)
    # gdi/eng/engwindow.c     # replaced by Rust (cohabitation, bisect iter 7)
    # gdi/eng/xlateobj.c  # replaced by Rust (xlateobj.rs) — cohabitation [wave 2 sweep]
    user/ntuser/main.c
    user/ntuser/misc/file.c
    user/ntuser/misc/rtlstr.c
    user/ntuser/misc/copy.c
    user/ntuser/misc/registry.c
    reactx/ntddraw/ddraw.c
    reactx/ntddraw/dd.c
    reactx/ntddraw/ddsurf.c
    reactx/ntddraw/d3d.c
    reactx/ntddraw/d3dkmt.c
    reactx/ntddraw/dvp.c
    reactx/ntddraw/mocomp.c
    reactx/ntddraw/eng.c
    reactx/ntddraw/dxeng.c
    user/ntuser/accelerator.c
    user/ntuser/callback.c
    user/ntuser/callproc.c
    user/ntuser/caret.c
    user/ntuser/class.c
    user/ntuser/clipboard.c
    user/ntuser/console.c
    user/ntuser/csr.c
    user/ntuser/cursoricon.c
    user/ntuser/dde.c
    user/ntuser/defwnd.c
    user/ntuser/desktop.c
    user/ntuser/display.c
    user/ntuser/draw.c
    user/ntuser/event.c
    user/ntuser/focus.c
    user/ntuser/ghost.c
    user/ntuser/guicheck.c
    user/ntuser/hook.c
    user/ntuser/hotkey.c
    user/ntuser/input.c
    user/ntuser/ime.c
    user/ntuser/keyboard.c
    user/ntuser/kbdlayout.c
    user/ntuser/layered.c
    user/ntuser/menu.c
    user/ntuser/message.c
    user/ntuser/metric.c
    user/ntuser/misc.c
    user/ntuser/monitor.c
    user/ntuser/mouse.c
    user/ntuser/msgqueue.c
    user/ntuser/nonclient.c
    user/ntuser/ntstubs.c
    user/ntuser/ntuser.c
    user/ntuser/painting.c
    user/ntuser/power.c
    user/ntuser/prop.c
    user/ntuser/scrollbar.c
    user/ntuser/scrollex.c
    user/ntuser/security.c
    user/ntuser/session.c
    user/ntuser/shutdown.c
    user/ntuser/simplecall.c
    user/ntuser/sysparams.c
    user/ntuser/timer.c
    user/ntuser/useratom.c
    user/ntuser/usrheap.c
    user/ntuser/vis.c
    user/ntuser/windc.c
    user/ntuser/window.c
    user/ntuser/winpos.c
    user/ntuser/winsta.c
    # user/ntuser/object.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/object.rs) — cohabitation [Phase 1b]
    user/rtl/image.c
    user/rtl/text.c
    # gdi/ntgdi/arc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/arc.rs) — cohabitation [Phase 2c geometry wave 2]
    # gdi/ntgdi/bezier.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/bezier.rs) — cohabitation [Phase 2c geometry wave 1]
    gdi/ntgdi/bitblt.c
    gdi/ntgdi/bitmaps.c
    gdi/ntgdi/cliprgn.c
    gdi/ntgdi/coord.c
    gdi/ntgdi/d3dkmt.c
    # gdi/ntgdi/dcattr.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dcattr.rs) — cohabitation [Phase 2c geometry wave 1]
    gdi/ntgdi/dclife.c
    gdi/ntgdi/dcobjs.c
    gdi/ntgdi/dcstate.c
    gdi/ntgdi/dcutil.c
    gdi/ntgdi/device.c
    gdi/ntgdi/dibobj.c
    # gdi/ntgdi/drawing.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/drawing.rs) — cohabitation [Phase 2c geometry wave 3: drawing primitives]
    # gdi/ntgdi/fillshap.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/fillshap.rs) — cohabitation [Phase 2c geometry wave 3: drawing primitives]
    gdi/ntgdi/font.c
    gdi/ntgdi/freetype.c
    gdi/ntgdi/gdibatch.c
    gdi/ntgdi/gdidbg.c
    # gdi/ntgdi/gdiobj.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/gdiobj.rs, +split2) — cohabitation [Phase 1a]
    gdi/ntgdi/gdipool.c
    gdi/ntgdi/icm.c
    gdi/ntgdi/init.c
    # gdi/ntgdi/line.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/line.rs) — cohabitation [Phase 2c geometry wave 3: drawing primitives]
    gdi/ntgdi/metafile.c
    gdi/ntgdi/palette.c
    gdi/ntgdi/path.c
    gdi/ntgdi/pen.c
    # gdi/ntgdi/polyfill.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/polyfill.rs) — cohabitation [Phase 2c geometry wave 1]
    gdi/ntgdi/print.c
    # gdi/ntgdi/rect.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/rect.rs) — cohabitation [Phase 2c geometry wave 1]
    # gdi/ntgdi/region.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/region.rs) — cohabitation [Phase 2c geometry wave 4: region algebra; region.c + split2/3/4 = one TU, 68 fns]
    # gdi/ntgdi/stockobj.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/stockobj.rs) — cohabitation [Phase 2c geometry wave 2]
    gdi/ntgdi/text.c
    # gdi/ntgdi/utils.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/utils.rs) — cohabitation [Phase 2c geometry wave 1]
    gdi/ntgdi/wingl.c
    # gdi/ntgdi/xformobj.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/xformobj.rs) — cohabitation [Phase 2c geometry wave 2]
    # gdi/eng/stubs.c       # replaced by Rust (stubs.rs) — cohabitation [wave 2 sweep, per-arch SharedUserData]
    # gdi/eng/umpdstubs.c   # replaced by Rust (umpdstubs.rs) — cohabitation [wave 2 sweep]
    pch.h)

if(USE_DIBLIB)
    add_definitions(-D_USE_DIBLIB_)
    list(APPEND SOURCE
        gdi/dib/dib_new.c
        gdi/eng/bitblt_new.c)
else()
    list(APPEND SOURCE
        # gdi/dib/dib.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib.rs) — cohabitation [Phase 2b dib wave 2b: table + DIB_DoRop + Dummy_*]
        # gdi/eng/copybits.c    # replaced by Rust (sdk/lib/rust/win32k-rust/src/copybits.rs) — cohabitation
        ${GENDIB_FILES})
endif()

if(ARCH STREQUAL "i386")
list(APPEND ASM_SOURCE
    gdi/dib/i386/dib24bpp_hline.s
    gdi/dib/i386/dib32bpp_hline.s
    gdi/dib/i386/dib32bpp_colorfill.s
    gdi/eng/i386/floatobj.S)
list(APPEND SOURCE gdi/eng/i386/floatobj.c)
else()
list(APPEND SOURCE
    # gdi/dib/dib24bppc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib24bppc.rs) — cohabitation [Phase 2b dib wave 1, non-x86]
    # gdi/dib/dib32bppc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dib32bppc.rs) — cohabitation [Phase 2b dib wave 1, non-x86]
    )
endif()

if(KDBG)
    add_definitions(-DKDBG)
    list(APPEND SOURCE gdi/ntgdi/gdikdbgext.c)
endif()

add_asm_files(win32k_asm ${ASM_SOURCE})

# ── Rust win32k code — single umbrella staticlib (cohabitation) ──────
# Analog of ntoskrnl's libkernel_rust.a. Each translated win32ss/*.c is
# commented out in the SOURCE list above and its #[no_mangle] symbol is
# provided here instead; --allow-multiple-definition + lld first-match
# covers the swap-in window. Own rust-target dir (not shared with the
# ntoskrnl build) so the two cargo invocations don't fight.
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(RUST_WIN32K_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-win32k-target")
if(ARCH STREQUAL "i386")
    set(RUST_WIN32K_RUST_TARGET "i686-pc-windows-gnu")
    set(RUST_WIN32K_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(RUST_WIN32K_RUST_TARGET "x86_64-pc-windows-gnu")
    set(RUST_WIN32K_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(RUST_WIN32K_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RUST_WIN32K_EXTRA_ENV "")
endif()
set(RUST_WIN32K_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwin32k_kernel_rust.a")
add_custom_command(
    OUTPUT ${RUST_WIN32K_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_WIN32K_TARGET_DIR} ${RUST_WIN32K_EXTRA_ENV}
        cargo build --release --target ${RUST_WIN32K_RUST_TARGET}
            -p win32k-kernel-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_WIN32K_TARGET_DIR}/${RUST_WIN32K_RUST_TARGET}/release/libwin32k_kernel_rust.a
        ${RUST_WIN32K_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/win32k-kernel-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/win32k-kernel-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/sort.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/gdiobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/object.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/math.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/error.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/multidisp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/perfcnt.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/driverobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/string.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/paint.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/float.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/engevent.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/mdevobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/semaphor.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/alphablend.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/bitblt.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/clip.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/copybits.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/debug.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/device.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/engbrush.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/engmisc.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/engwindow.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/gradient.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/ldevobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/lineto.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/mapping.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/mem.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/mouse.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/pandisp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/pathobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/pdevobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/rlecomp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/stretchblt.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/stubs.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/surface.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/transblt.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/umpdstubs.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/xlateobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib_helpers.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib1bpp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib4bpp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib8bpp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib16bpp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib24bpp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib32bpp.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib24bppc.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib32bppc.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib_alphablend.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib_stretchblt.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/floodfill.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dib.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dcattr.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/rect.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/bezier.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/utils.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/polyfill.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/brushbody.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/arc.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/stockobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/xformobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/drawing.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/line.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/fillshap.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/region.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust win32k code (${RUST_WIN32K_RUST_TARGET})"
)
add_custom_target(rust_win32k DEPENDS ${RUST_WIN32K_LIB})

add_library(win32k MODULE
    ${SOURCE}
    gdi/ntgdi/brush.cpp
    ${win32k_asm}
    win32k.rc
    ${CMAKE_CURRENT_BINARY_DIR}/win32k.def)

set_module_type(win32k kernelmodedriver)
set_target_properties(win32k PROPERTIES C_STANDARD 23)
# Rust compiler_builtins / libclang_rt both provide __chkstk, and the
# cohabitation swap-in window has C+Rust defining the same export.
add_dependencies(win32k rust_win32k)
target_link_options(win32k PRIVATE "LINKER:--allow-multiple-definition")
target_link_libraries(win32k ${PSEH_LIB} dxguid libcntpr ${RUST_WIN32K_LIB})

if(USE_DIBLIB)
    target_link_libraries(win32k diblib)
endif()

add_importlibs(win32k ntoskrnl hal ftfd)
add_pch(win32k pch.h SOURCE)
add_cd_file(TARGET win32k DESTINATION reactos/system32 FOR all)

if(ARCH STREQUAL "amd64")
    set_source_files_properties(sys-stubs.S PROPERTIES OBJECT_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/w32ksvc64.h)
else()
    set_source_files_properties(sys-stubs.S PROPERTIES OBJECT_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/w32ksvc32.h)
endif()
add_asm_files(win32ksys_asm sys-stubs.S)
add_library(win32ksys ${win32ksys_asm})
set_target_properties(win32ksys PROPERTIES LINKER_LANGUAGE "C")
add_dependencies(win32ksys asm)
