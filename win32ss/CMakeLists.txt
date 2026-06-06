
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
    # gdi/eng/ldevobj.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ldevobj.rs) — cohabitation [Phase 2c: LDEVOBJ driver-load layer]
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
    # user/ntuser/misc/file.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/file.rs) — cohabitation [ntuser wave 10]
    # user/ntuser/misc/rtlstr.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/rtlstr.rs) — cohabitation [ntuser wave 8]
    # user/ntuser/misc/copy.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/copy.rs) — cohabitation [ntuser wave 8]
    # user/ntuser/misc/registry.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/registry.rs) — cohabitation [ntuser wave 11]
    reactx/ntddraw/ddraw.c
    reactx/ntddraw/dd.c
    reactx/ntddraw/ddsurf.c
    reactx/ntddraw/d3d.c
    reactx/ntddraw/d3dkmt.c
    reactx/ntddraw/dvp.c
    reactx/ntddraw/mocomp.c
    reactx/ntddraw/eng.c
    reactx/ntddraw/dxeng.c
    # user/ntuser/accelerator.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/accelerator.rs) — cohabitation [ntuser wave 4]
    # user/ntuser/callback.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/callback.rs) — cohabitation [ntuser wave 28]
    # user/ntuser/callproc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/callproc.rs) — cohabitation [ntuser wave 7]
    # user/ntuser/caret.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/caret.rs) — cohabitation [ntuser wave 3]
    # user/ntuser/class.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/class.rs) — cohabitation [ntuser wave 12 spine]
    # user/ntuser/clipboard.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/clipboard.rs) — cohabitation [ntuser wave 15 spine]
    # user/ntuser/console.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/console.rs) — cohabitation [ntuser wave 7]
    # user/ntuser/csr.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/csr.rs) — cohabitation [ntuser wave 9]
    user/ntuser/cursoricon.c
    # user/ntuser/dde.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dde.rs) — cohabitation [ntuser wave 21]
    user/ntuser/defwnd.c
    # user/ntuser/desktop.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/desktop.rs) — cohabitation [ntuser wave 13 spine]
    # user/ntuser/display.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/display.rs) — cohabitation [ntuser wave 31]
    # user/ntuser/draw.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/draw.rs) — cohabitation [ntuser wave 32]
    # user/ntuser/event.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/event.rs) — cohabitation [ntuser wave 10]
    # user/ntuser/focus.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/focus.rs) — cohabitation [ntuser wave 25]
    # user/ntuser/ghost.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ghost.rs) — cohabitation [ntuser wave 5]
    # user/ntuser/guicheck.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/guicheck.rs) — cohabitation [ntuser wave 2]
    user/ntuser/hook.c
    # user/ntuser/hotkey.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/hotkey.rs) — cohabitation [ntuser wave 18]
    # user/ntuser/input.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/input.rs) — cohabitation [ntuser wave 30]
    user/ntuser/ime.c
    user/ntuser/keyboard.c
    user/ntuser/kbdlayout.c
    # user/ntuser/layered.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/layered.rs) — cohabitation [ntuser wave 16]
    user/ntuser/menu.c
    user/ntuser/message.c
    # user/ntuser/metric.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/metric.rs) — cohabitation [ntuser wave 2]
    # user/ntuser/misc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/misc.rs) — cohabitation [ntuser wave 26]
    # user/ntuser/monitor.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/monitor.rs) — cohabitation [ntuser wave 20]
    # user/ntuser/mouse.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntmouse.rs) — cohabitation [ntuser wave 29]
    user/ntuser/msgqueue.c
    # user/ntuser/nonclient.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/nonclient.rs) — cohabitation [ntuser wave 27]
    # user/ntuser/ntstubs.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntstubs.rs) — cohabitation [ntuser wave 22]
    # user/ntuser/ntuser.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntuser.rs) — cohabitation [ntuser wave 7]
    user/ntuser/painting.c
    # user/ntuser/power.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/power.rs) — cohabitation [ntuser wave 6]
    # user/ntuser/prop.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/prop.rs) — cohabitation [ntuser wave 2]
    user/ntuser/scrollbar.c
    # user/ntuser/scrollex.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/scrollex.rs) — cohabitation [ntuser wave 19]
    # user/ntuser/security.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/security.rs) — cohabitation [ntuser wave 11]
    # user/ntuser/session.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/session.rs) — cohabitation [ntuser wave 1]
    # user/ntuser/shutdown.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/shutdown.rs) — cohabitation [ntuser wave 10]
    # user/ntuser/simplecall.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/simplecall.rs) — cohabitation [ntuser wave 24]
    user/ntuser/sysparams.c
    # user/ntuser/timer.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/timer.rs) — cohabitation [ntuser wave 17]
    # user/ntuser/useratom.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/useratom.rs) — cohabitation [ntuser wave 1]
    # user/ntuser/usrheap.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/usrheap.rs) — cohabitation [ntuser wave 6]
    # user/ntuser/vis.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/vis.rs) — cohabitation [ntuser wave 5]
    # user/ntuser/windc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/windc.rs) — cohabitation [ntuser wave 23]
    user/ntuser/window.c
    user/ntuser/winpos.c
    # user/ntuser/winsta.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/winsta.rs) — cohabitation [ntuser wave 14 spine]
    # user/ntuser/object.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/object.rs) — cohabitation [Phase 1b]
    user/rtl/image.c
    user/rtl/text.c
    # gdi/ntgdi/arc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/arc.rs) — cohabitation [Phase 2c geometry wave 2]
    # gdi/ntgdi/bezier.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/bezier.rs) — cohabitation [Phase 2c geometry wave 1]
    # gdi/ntgdi/bitblt.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntbitblt.rs) — cohabitation [Phase 2c wave 20: ntgdi bitblt syscalls; bitblt.c + split2 = one TU]
    gdi/ntgdi/ntbitblt_rust_shims.c  # cdecl wrapper for FORCEINLINE DC_pmxWorldToDevice ntbitblt.rs calls
    # gdi/ntgdi/bitmaps.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/bitmaps.rs) — cohabitation [Phase 2c wave 8: bitmap/surface object mgmt]
    # gdi/ntgdi/cliprgn.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/cliprgn.rs) — cohabitation [Phase 2c wave 8: clip-region functions]
    # gdi/ntgdi/coord.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/coord.rs) — cohabitation [Phase 2c geometry wave 6: coordinate transforms; coord.c + split2 = one TU]
    # gdi/ntgdi/d3dkmt.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/d3dkmt.rs) — cohabitation [Phase 2c wave 22: D3DKMT dxgkrnl thunk layer]
    # gdi/ntgdi/dcattr.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dcattr.rs) — cohabitation [Phase 2c geometry wave 1]
    # gdi/ntgdi/dclife.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dclife.rs) — cohabitation [Phase 2c wave 5: DC lifecycle]
    # gdi/ntgdi/dcobjs.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dcobjs.rs) — cohabitation [Phase 2c wave 5: DC lifecycle]
    # gdi/ntgdi/dcstate.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dcstate.rs) — cohabitation [Phase 2c wave 5: DC lifecycle]
    # gdi/ntgdi/dcutil.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dcutil.rs) — cohabitation [Phase 2c wave 5: DC lifecycle]
    # gdi/ntgdi/device.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/gdidev.rs) — cohabitation [Phase 2c wave 17: primary-surface tier]
    # gdi/ntgdi/dibobj.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dibobj.rs) — cohabitation [Phase 2c wave 9: DIB sections; dibobj.c + split2 = one TU]
    # gdi/ntgdi/drawing.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/drawing.rs) — cohabitation [Phase 2c geometry wave 3: drawing primitives]
    # gdi/ntgdi/fillshap.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/fillshap.rs) — cohabitation [Phase 2c geometry wave 3: drawing primitives]
    # gdi/ntgdi/font.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/font.rs) — cohabitation [Phase 2c wave 19: font tier]
    gdi/ntgdi/font_rust_shims.c  # cdecl wrappers for FORCEINLINE TEXTOBJ_LockText/UnlockText font.rs calls
    # gdi/ntgdi/freetype.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/freetype.rs) — cohabitation [Phase 2c wave 21: FreeType font engine; uses ft-sys FFI crate]
    # gdi/ntgdi/gdibatch.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/gdibatch.rs) — cohabitation [Phase 2c wave 24]
    # gdi/ntgdi/gdidbg.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/gdidbg.rs) — cohabitation [Phase 2c wave 25]
    # gdi/ntgdi/gdiobj.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/gdiobj.rs, +split2) — cohabitation [Phase 1a]
    # gdi/ntgdi/gdipool.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/gdipool.rs) — cohabitation [Phase 2c wave 12: GDI pool allocator]
    # gdi/ntgdi/icm.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/icm.rs) — cohabitation [Phase 2c wave 11: ICM color mgmt]
    # gdi/ntgdi/init.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/init.rs) — cohabitation [Phase 2c wave 15: GDI process/thread init callouts]
    # gdi/ntgdi/line.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/line.rs) — cohabitation [Phase 2c geometry wave 3: drawing primitives]
    # gdi/ntgdi/metafile.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/metafile.rs) — cohabitation [Phase 2c wave 13: server metafile object tier]
    # gdi/ntgdi/palette.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/palette.rs) — cohabitation [Phase 2c geometry wave 7: PALETTE object tier]
    # gdi/ntgdi/path.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/path.rs) — cohabitation [Phase 2c wave 18: path tier; path.c + split2/3 = one TU]
    gdi/ntgdi/path_rust_shims.c  # cdecl wrappers for FORCEINLINE/macro/arch-divergent eng helpers path.rs calls
    gdi/ntgdi/freetype_rust_shims.c  # ntstrsafe header-inline wrappers for freetype.rs calls
    # gdi/ntgdi/pen.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/pen.rs) — cohabitation [Phase 2c geometry wave 6: PEN object tier]
    # gdi/ntgdi/polyfill.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/polyfill.rs) — cohabitation [Phase 2c geometry wave 1]
    # gdi/ntgdi/print.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/print.rs) — cohabitation [Phase 2c wave 16: print/escape tier]
    # gdi/ntgdi/rect.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/rect.rs) — cohabitation [Phase 2c geometry wave 1]
    # gdi/ntgdi/region.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/region.rs) — cohabitation [Phase 2c geometry wave 4: region algebra; region.c + split2/3/4 = one TU, 68 fns]
    # gdi/ntgdi/stockobj.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/stockobj.rs) — cohabitation [Phase 2c geometry wave 2]
    # gdi/ntgdi/text.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/text.rs) — cohabitation [Phase 2c wave 10: text query tier]
    # gdi/ntgdi/utils.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/utils.rs) — cohabitation [Phase 2c geometry wave 1]
    # gdi/ntgdi/wingl.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/wingl.rs) — cohabitation [Phase 2c wave 14: WinGL/pixel-format tier]
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
    # list(APPEND SOURCE gdi/ntgdi/gdikdbgext.c)  # replaced by Rust (sdk/lib/rust/win32k-rust/src/gdikdbgext.rs) — cohabitation [Phase 2c wave 26]
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
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/session.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/useratom.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/guicheck.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/prop.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/metric.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/caret.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/accelerator.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/vis.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/ghost.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/power.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/usrheap.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/ntuser.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/callproc.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/console.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/copy.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/rtlstr.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/csr.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/file.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/shutdown.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/event.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/registry.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/security.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/class.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/desktop.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/winsta.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/clipboard.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/layered.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/timer.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/hotkey.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/scrollex.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/monitor.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dde.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/ntstubs.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/windc.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/simplecall.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/focus.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/misc.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/nonclient.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/callback.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/ntmouse.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/input.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/display.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/draw.rs
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
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dcstate.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dcobjs.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dclife.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dcutil.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/pen.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/coord.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/palette.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/cliprgn.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/bitmaps.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/dibobj.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/text.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/icm.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/gdipool.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/metafile.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/wingl.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/init.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/print.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/gdidev.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/path.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/font.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/ntbitblt.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/freetype.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/d3dkmt.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/brush.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/gdibatch.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/gdidbg.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/src/gdikdbgext.rs
            ${RUST_WORKSPACE_DIR}/win32k-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust win32k code (${RUST_WIN32K_RUST_TARGET})"
)
add_custom_target(rust_win32k DEPENDS ${RUST_WIN32K_LIB})

add_library(win32k MODULE
    ${SOURCE}
    # gdi/ntgdi/brush.cpp  # replaced by Rust (sdk/lib/rust/win32k-rust/src/brush.rs) — cohabitation [Phase 2c wave 23: BRUSH object]
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
