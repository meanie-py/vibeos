
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
if(STACK_PROTECTOR)
    target_link_libraries(libwin32k INTERFACE gcc_ssp_win32k)
endif()

include_directories(
    include
    ${REACTOS_SOURCE_DIR}/vendor/freetype/include ## HACK for ntgdi/freetype.c and related
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
    # safe_user_io.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/safe_user_io.rs) — cohabitation [SafeProbeForRead/Write, validate-only]
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
    # user/ntuser/main.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/main.rs) — cohabitation [ntuser frontier]
    user/ntuser/main_rust_shims.c  # napi.h syscall dispatch tables (Win32kSSDT/SSPT/NumberOfSysCalls) for main.rs
    # user/ntuser/misc/file.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/file.rs) — cohabitation [ntuser wave 10]
    # user/ntuser/misc/rtlstr.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/rtlstr.rs) — cohabitation [ntuser wave 8]
    # user/ntuser/misc/copy.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/copy.rs) — cohabitation [ntuser wave 8]
    # user/ntuser/misc/registry.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/registry.rs) — cohabitation [ntuser wave 11]
    # reactx/ntddraw/ddraw.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_ddraw.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/dd.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_dd.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/ddsurf.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_ddsurf.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/d3d.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_d3d.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/d3dkmt.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_d3dkmt.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/dvp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_dvp.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/mocomp.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_mocomp.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/eng.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_eng.rs) — cohabitation [DirectDraw flip]
    # reactx/ntddraw/dxeng.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntddraw_dxeng.rs) — cohabitation [DirectDraw flip: all 42 DxEng* bodies + gaEngFuncs table now Rust]
    # user/ntuser/accelerator.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/accelerator.rs) — cohabitation [ntuser wave 4]
    # user/ntuser/callback.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/callback.rs) — cohabitation [ntuser wave 28]
    # user/ntuser/callproc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/callproc.rs) — cohabitation [ntuser wave 7]
    # user/ntuser/caret.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/caret.rs) — cohabitation [ntuser wave 3]
    # user/ntuser/class.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/class.rs) — cohabitation [ntuser wave 12 spine]
    # user/ntuser/clipboard.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/clipboard.rs) — cohabitation [ntuser wave 15 spine]
    # user/ntuser/console.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/console.rs) — cohabitation [ntuser wave 7]
    # user/ntuser/csr.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/csr.rs) — cohabitation [ntuser wave 9]
    # user/ntuser/cursoricon.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/cursoricon.rs) — cohabitation [ntuser wave 35]
    # user/ntuser/dde.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/dde.rs) — cohabitation [ntuser wave 21]
    # user/ntuser/defwnd.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/defwnd.rs) — cohabitation [ntuser frontier]
    # user/ntuser/desktop.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/desktop.rs) — cohabitation [ntuser wave 13 spine]
    # user/ntuser/display.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/display.rs) — cohabitation [ntuser wave 31]
    # user/ntuser/draw.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/draw.rs) — cohabitation [ntuser wave 32]
    # user/ntuser/event.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/event.rs) — cohabitation [ntuser wave 10]
    # user/ntuser/focus.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/focus.rs) — cohabitation [ntuser wave 25]
    # user/ntuser/ghost.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ghost.rs) — cohabitation [ntuser wave 5]
    # user/ntuser/guicheck.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/guicheck.rs) — cohabitation [ntuser wave 2]
    # user/ntuser/hook.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/hook.rs) — cohabitation [ntuser wave 33]
    # user/ntuser/hotkey.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/hotkey.rs) — cohabitation [ntuser wave 18]
    # user/ntuser/input.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/input.rs) — cohabitation [ntuser wave 30]
    # user/ntuser/job.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/job.rs) — upstream b272427baa3, never built as C here
    # user/ntuser/ime.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ime.rs) — cohabitation [ntuser wave 36]
    # user/ntuser/keyboard.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/keyboard.rs) — cohabitation [ntuser wave 38]
    # user/ntuser/kbdlayout.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/kbdlayout.rs) — cohabitation [ntuser frontier]
    # user/ntuser/layered.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/layered.rs) — cohabitation [ntuser wave 16]
    # user/ntuser/menu.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/menu.rs) — cohabitation [ntuser frontier]
    # user/ntuser/message.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/message.rs) — cohabitation [ntuser frontier]
    # user/ntuser/metric.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/metric.rs) — cohabitation [ntuser wave 2]
    # user/ntuser/misc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/misc.rs) — cohabitation [ntuser wave 26]
    # user/ntuser/monitor.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/monitor.rs) — cohabitation [ntuser wave 20]
    # user/ntuser/mouse.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntmouse.rs) — cohabitation [ntuser wave 29]
    # user/ntuser/msgqueue.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/msgqueue.rs) — cohabitation [ntuser wave 37]
    # user/ntuser/nonclient.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/nonclient.rs) — cohabitation [ntuser wave 27]
    # user/ntuser/ntstubs.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntstubs.rs) — cohabitation [ntuser wave 22]
    # user/ntuser/ntuser.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/ntuser.rs) — cohabitation [ntuser wave 7]
    # user/ntuser/painting.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/painting.rs) — cohabitation [ntuser frontier]
    # user/ntuser/power.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/power.rs) — cohabitation [ntuser wave 6]
    # user/ntuser/prop.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/prop.rs) — cohabitation [ntuser wave 2]
    # user/ntuser/scrollbar.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/scrollbar.rs) — cohabitation [ntuser wave 34]
    # user/ntuser/scrollex.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/scrollex.rs) — cohabitation [ntuser wave 19]
    # user/ntuser/security.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/security.rs) — cohabitation [ntuser wave 11]
    # user/ntuser/session.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/session.rs) — cohabitation [ntuser wave 1]
    # user/ntuser/shutdown.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/shutdown.rs) — cohabitation [ntuser wave 10]
    # user/ntuser/simplecall.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/simplecall.rs) — cohabitation [ntuser wave 24]
    # user/ntuser/sysparams.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/sysparams.rs) — cohabitation [ntuser frontier]
    # user/ntuser/timer.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/timer.rs) — cohabitation [ntuser wave 17]
    # user/ntuser/useratom.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/useratom.rs) — cohabitation [ntuser wave 1]
    # user/ntuser/usrheap.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/usrheap.rs) — cohabitation [ntuser wave 6]
    # user/ntuser/vis.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/vis.rs) — cohabitation [ntuser wave 5]
    # user/ntuser/windc.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/windc.rs) — cohabitation [ntuser wave 23]
    # user/ntuser/window.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/window.rs) — cohabitation [ntuser frontier]
    # user/ntuser/winpos.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/winpos.rs) — cohabitation [ntuser frontier]
    # user/ntuser/winsta.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/winsta.rs) — cohabitation [ntuser wave 14 spine]
    # user/ntuser/object.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/object.rs) — cohabitation [Phase 1b]
    # user/rtl/image.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/image.rs) — cohabitation [RtlGetExpWinVer]
    # user/rtl/text.c  # replaced by Rust (sdk/lib/rust/win32k-rust/src/rtl_text.rs) — cohabitation [DrawTextExWorker/UserExtTextOutW + ellipsify/wordbreak]
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

# ── C residue static lib ─────────────────────────────────────────────
# The still-C remainder of win32k (DirectDraw, gendib tables, FORCEINLINE
# rust shims, safe_user_io, user/rtl, i386 floatobj + dib asm), compiled by
# cmake into a static archive the cargo link reuses (the same tier as
# libhal.a / libcmlib.a / liblibcntpr.a). This was the force-linked object
# list of the old `add_library(win32k MODULE …)`; win32k-cargo/build.rs
# places this archive FIRST in the link's --start-group so C keeps the
# cohab first-match precedence over libwin32k_kernel_rust.a, exactly as the
# objects-before-rust-archive cmake link ordered it.
add_library(win32k_residue STATIC ${SOURCE} ${win32k_asm})
set_target_properties(win32k_residue PROPERTIES C_STANDARD 23)
add_dependencies(win32k_residue bugcodes xdk asm)
add_pch(win32k_residue pch.h SOURCE)

# ── win32k.sys: cargo-driven link ──────────────────────────────────────
# The Rust body (win32k-kernel-rust → libwin32k_kernel_rust.a) is built by
# the reactos-link engine with the user hardfloat target + i386 -sse,-sse2
# (.rust_user_target() — the same proven config the old rust_win32k
# custom_command used), NOT the softfloat kernel json. Replaces
# add_library(win32k MODULE) + set_module_type(kernelmodedriver) +
# target_link_libraries + add_importlibs + add_pch + add_cd_file (and the
# rust_win32k custom_command — the engine builds the body itself). KIND
# kernelmodedriver → native-pefixup --kernelmodedriver + i386 fwd fix.
# spec2def(win32k.sys win32k.spec ADD_IMPORTLIB) above STAYS — the libwin32k
# import lib consumers (framebuf, …) are independent of this target.
# EXTRA_DEPENDS = every cmake target whose .a the cargo link reuses
# (win32k_residue/pseh/dxguid/libcntpr/chkstk/rust_cnt/memcmp) + the
# generated-header providers (psdk/asm/bugcodes/xdk).
add_cargo_module(win32k win32k-cargo win32k.sys
    KIND kernelmodedriver
    BODY_CRATES win32k-kernel-rust win32k-rust
    EXTRA_DEPENDS
        win32k_residue ${PSEH_LIB} dxguid libcntpr chkstk rust_cnt memcmp
        psdk asm bugcodes xdk)
# gdi/ntgdi/brush.cpp (upstream's BRUSH C++ class) declined: already covered by
# Rust (sdk/lib/rust/win32k-rust/src/brush.rs + brushbody.rs).

if(ARCH STREQUAL "amd64")
    set_source_files_properties(sys-stubs.S PROPERTIES OBJECT_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/w32ksvc64.h)
else()
    set_source_files_properties(sys-stubs.S PROPERTIES OBJECT_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/w32ksvc32.h)
endif()
add_asm_files(win32ksys_asm sys-stubs.S)
add_library(win32ksys ${win32ksys_asm})
set_target_properties(win32ksys PROPERTIES LINKER_LANGUAGE "C")
add_dependencies(win32ksys asm)
