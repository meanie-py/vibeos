# vibeos wow64cpu.dll — native amd64 WOW64 CPU layer (Phase E). The heaven's-
# gate mode switch is x86-64-specific, so this module only exists on amd64.
if(ARCH STREQUAL "amd64")

spec2def(wow64cpu.dll wow64cpu.spec ADD_IMPORTLIB)

list(APPEND ASM_SOURCE wow64cpu.S)
add_asm_files(wow64cpu_asm ${ASM_SOURCE})

add_library(wow64cpu MODULE
    wow64cpu.c
    ${wow64cpu_asm}
    wow64cpu.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wow64cpu.def)

set_module_type(wow64cpu win32dll ENTRYPOINT DllMain 12)
# ntdll for the syscall surface; wow64 for Wow64SystemServiceEx. A WoW64
# process must not pull 64-bit kernel32 through the CPU layer (Win10 parity).
add_importlibs(wow64cpu ntdll wow64)
add_dependencies(wow64cpu psdk)
add_cd_file(TARGET wow64cpu DESTINATION reactos/system32 FOR all)

# E3/E4: stage the i386 SysWOW64 set onto the amd64 livecd — the amd64 mirror
# of qemu/mkramdisk.sh's VIBEOS_FEX staging on arm64. Files come from SIBLING
# build trees (build-i386 / build-fex-hello / tools/doom) via the canonical
# path, so this is inherently cross-build: an amd64 configure only stages when
# the i386 artifacts already exist (build i386 + build-hello-compute.sh +
# doom/build.sh i386 first; re-run the amd64 configure after an i386 rebuild
# to re-snapshot). Opportunistic like the arm64 probes — the winlogon hooks
# are silent no-ops when a guest is absent — but the skip itself is loud.
#
# The DLL set is the FULL transitive import closure of the stock guests (roots:
# testgui/doom/gui32), computed by wow64-closure.py exactly as mkramdisk does it
# (all-or-nothing: a partial set half-binds a module and crashes at first call,
# far from the missing DLL — the tool hard-fails instead). --no-strip: the amd64
# livecd flow has its own strip step, and the hashed boot/stripped names would go
# stale. The full closure is staged so E4b (testgui/doom) needs no re-staging.
#
# GUEST EXES: all four rungs — hello32 (0x600D), gui32 (win32k GDI, 0x600e),
# testgui + doom (E4b: window creation + kernel->user callbacks).
set(_wow64_guests
    ${REACTOS_SOURCE_DIR}/build-fex-hello/hello32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/gui32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/win32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/file32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/async32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/marshal32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/fault32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/thread32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/selfmod32.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/testgui/testgui.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/winmine/winmine.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/calc/calc.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/notepad/notepad.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/solitaire/sol.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/charmap/charmap.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mspaint/mspaint.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/regedit/regedit.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/winver/winver.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/taskmgr/taskmgr.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/wordpad/wordpad.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/clipbrd/clipbrd.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/osk/osk.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/magnify/magnify.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/spider/spider.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mscutils/eventvwr/eventvwr.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mscutils/devmgmt/devmgmt.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/msconfig/msconfig.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/sndrec32/sndrec32.exe
    ${REACTOS_SOURCE_DIR}/build-i386/base/applications/kbswitch/kbswitch.exe
    ${REACTOS_SOURCE_DIR}/tools/doom/doom-i386.exe)
# OPTIONAL guests are kept OUT of _wow64_guests on purpose: a single missing
# entry there sets _wow64_guest_ok FALSE and skips the whole syswow64 staging,
# so listing a build-on-demand binary would silently disable every WoW64 probe
# for anyone who had not built it. Listed here instead, and only when present.
set(_wow64_optional_guests)
if(EXISTS ${REACTOS_SOURCE_DIR}/tools/stress/sqlite/sqlite32.exe)
    list(APPEND _wow64_optional_guests ${REACTOS_SOURCE_DIR}/tools/stress/sqlite/sqlite32.exe)
endif()
set(_wow64_guest_ok TRUE)
foreach(_f ${_wow64_guests})
    if(NOT EXISTS ${_f})
        set(_wow64_guest_ok FALSE)
        message(STATUS "WOW64 E4: ${_f} missing — syswow64 NOT staged on livecd (WoW64 probes will no-op)")
    endif()
endforeach()
if(_wow64_guest_ok)
    # The closure runs at CONFIGURE time, so ninja has no idea it exists: editing
    # wow64-closure.py or the INF it derives dynamic edges from would leave the
    # previously-staged module list on the livecd and the next boot would test the
    # OLD closure while looking like it tested the new one. Re-configure on either.
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
                 ${REACTOS_SOURCE_DIR}/tools/fex/wow64-closure.py
                 ${REACTOS_SOURCE_DIR}/media/inf/nettcpip.inf)
    execute_process(
        COMMAND python3 ${REACTOS_SOURCE_DIR}/tools/fex/wow64-closure.py
                --build-dir ${REACTOS_SOURCE_DIR}/build-i386
                --include-prestaged --no-strip
                # Every guest we STAGE is also a closure root. It was not always so:
                # the roots were the stock applets, and a ladder guest's imports were
                # covered only because some applet happened to import the same DLLs.
                # marshal32 linking ws2_32 was covered by that coincidence; the next
                # guest to link something new would have half-bound instead, which is
                # the crash-at-first-call this closure exists to prevent. Adding them
                # is currently a no-op (measured: same 64 modules), which is the point
                # — it costs nothing and stops depending on luck.
                ${_wow64_guests}
                ${_wow64_optional_guests}
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/testgui/testgui.exe
                ${REACTOS_SOURCE_DIR}/tools/doom/doom-i386.exe
                ${REACTOS_SOURCE_DIR}/build-fex-hello/gui32.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/calc/calc.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/notepad/notepad.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/solitaire/sol.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/charmap/charmap.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mspaint/mspaint.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/regedit/regedit.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/winver/winver.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/taskmgr/taskmgr.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/wordpad/wordpad.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/clipbrd/clipbrd.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/osk/osk.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/magnify/magnify.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/spider/spider.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mscutils/eventvwr/eventvwr.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mscutils/devmgmt/devmgmt.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/msconfig/msconfig.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/sndrec32/sndrec32.exe
                ${REACTOS_SOURCE_DIR}/build-i386/base/applications/kbswitch/kbswitch.exe
        OUTPUT_VARIABLE _wow64_closure
        ERROR_VARIABLE _wow64_closure_err
        RESULT_VARIABLE _wow64_closure_rc)
    if(NOT _wow64_closure_rc EQUAL 0)
        message(FATAL_ERROR "WOW64 E4: SysWOW64 import closure INCOMPLETE — refusing to stage a partial set (a half-bound module crashes at first call). Build the missing i386 module first.\n${_wow64_closure_err}")
    endif()
    string(STRIP "${_wow64_closure}" _wow64_closure)
    string(REPLACE "\n" ";" _wow64_lines "${_wow64_closure}")
    list(LENGTH _wow64_lines _wow64_n)
    message(STATUS "WOW64 E4: staging ${_wow64_n} i386 SysWOW64 modules + hello32/gui32 onto livecd")
    foreach(_line ${_wow64_lines})
        string(REPLACE "\t" ";" _kv "${_line}")
        list(GET _kv 0 _name)
        list(GET _kv 1 _path)
        add_cd_file(FILE ${_path} DESTINATION reactos/syswow64 NAME_ON_CD ${_name} FOR livecd)
    endforeach()
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/hello32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/gui32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/win32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/file32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/async32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/marshal32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/fault32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/thread32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-fex-hello/selfmod32.exe DESTINATION reactos/syswow64 FOR livecd)
    if(EXISTS ${REACTOS_SOURCE_DIR}/tools/stress/sqlite/sqlite32.exe)
        add_cd_file(FILE ${REACTOS_SOURCE_DIR}/tools/stress/sqlite/sqlite32.exe DESTINATION reactos/syswow64 FOR livecd)
    endif()
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/testgui/testgui.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/winmine/winmine.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/calc/calc.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/notepad/notepad.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/solitaire/sol.exe DESTINATION reactos/syswow64 FOR livecd)
    # E8 robustness ladder: new-surface stock applets — charmap (font-enum
    # arrays + EnumFontFamilies callback), mspaint (heavy GDI regions/DIB +
    # comdlg32 common dialogs), regedit (treeview/listview + registry-enum
    # arrays), winver (shell32 ShellAbout dialog).
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/charmap/charmap.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mspaint/mspaint.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/regedit/regedit.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/winver/winver.exe DESTINATION reactos/syswow64 FOR livecd)
    # E9 robustness ladder: taskmgr (listview + NtQuerySystemInformation process
    # array), wordpad (richedit/riched20 — a new control library), clipbrd
    # (clipboard format marshalling).
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/taskmgr/taskmgr.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/wordpad/wordpad.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/clipbrd/clipbrd.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/osk/osk.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/magnify/magnify.exe DESTINATION reactos/syswow64 FOR livecd)
    # E11 robustness ladder, chosen for surface no earlier guest reached:
    #   spider    cardlib DIB blitting — the direct disproof rung for E10's
    #             NtGdiCreateDIBSection marshaller (guest gdi32's 4-byte &bmBits)
    #   eventvwr  listview + the event-log RPC client (first 32-bit guest to do
    #             real RPC/LPC to a 64-bit service)
    #   devmgmt   setupapi device enumeration + treeview
    #   msconfig  comctl32 PROPERTY SHEETS / tab control (a new control family)
    #   sndrec32  winmm/mmsystem (a new DLL family in the import closure)
    #   kbswitch  shell tray notification icon (Shell_NotifyIcon IPC)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/games/spider/spider.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mscutils/eventvwr/eventvwr.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/mscutils/devmgmt/devmgmt.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/msconfig/msconfig.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/sndrec32/sndrec32.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/build-i386/base/applications/kbswitch/kbswitch.exe DESTINATION reactos/syswow64 FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/tools/doom/doom-i386.exe DESTINATION reactos/syswow64 NAME_ON_CD doom.exe FOR livecd)
    add_cd_file(FILE ${REACTOS_SOURCE_DIR}/tools/doom/doom1.wad DESTINATION reactos/syswow64 FOR livecd)
endif()

endif()
