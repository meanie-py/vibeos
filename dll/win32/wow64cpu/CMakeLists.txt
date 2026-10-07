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
# The NINE machine-verdicted ladder guests. Their staging must not depend on
# anything else: every entry below reports a real exit code (0x600D, gui32
# 0x600E), `smoke --ladder` grades them, and the gate now opts in on their
# presence (tests/qemu/cli/ladder.py).
set(_wow64_ladder
    ${REACTOS_SOURCE_DIR}/build-fex-hello/hello32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/gui32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/win32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/file32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/async32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/marshal32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/fault32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/thread32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/selfmod32.exe
    ${REACTOS_SOURCE_DIR}/build-fex-hello/reg32.exe
    # spawn32: PROMOTED out of the diagnostic extras 2026-09-19, the day it
    # first reported 0x600D. It measured guest CreateProcess while that gap was
    # open; now that the gap is closed it is a parity guest like the rest, and
    # belongs in the unconditional set so a REGRESSION can red it. Its old
    # reason for being opt-in -- it could BUGCHECK the machine -- died with the
    # dropped _SEH2_EXCEPT that caused it.
    ${REACTOS_SOURCE_DIR}/build-fex-hello/spawn32.exe)

# Best-effort robustness extras. Their pass criterion is "still running after
# 8-10s", which ladder.py explicitly does NOT check, so a missing one must not
# cost the nine guests their staging -- it did: ONE absent doom-i386.exe kept all
# nine off the livecd, and doom cannot even be built here (its shareware WAD URL
# 404s), so the ladder was unreachable rather than merely unstaged.
set(_wow64_extras
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
    ${REACTOS_SOURCE_DIR}/tools/doom/doom-i386.exe
    # sqlite32: a third-party STRESS guest (real SQLite, tools/stress/sqlite/build.sh).
    # Built on demand, so absent from a normal tree -- which is exactly what this
    # list is for. It IS machine-verdicted (0x600D), but it belongs here rather
    # than in _wow64_ladder because the ladder set must stage unconditionally and
    # this one legitimately does not exist most of the time; ladder.py grades it
    # from its OPTIONAL table instead.
    ${REACTOS_SOURCE_DIR}/tools/stress/sqlite/sqlite32.exe
    # lua32: the second third-party STRESS guest (real Lua 5.4,
    # tools/stress/lua/build.sh). Same deal as sqlite32 -- built on demand,
    # machine-verdicted (0x600D), graded from ladder.py's OPTIONAL table.
    ${REACTOS_SOURCE_DIR}/tools/stress/lua/lua32.exe
    # Wine's own conformance suites, i386, run under WoW64 by winlogon's
    # TryStartWow64Winetest. Present only after an i386 `build --rostests`, so
    # the EXISTS filter above is the opt-in: a normal tree stages nothing and a
    # rostests tree stages them automatically, with no third flag to forget.
    # Also a closure root, like every other guest -- a winetest links more of
    # the CRT than any applet here does.
    ${REACTOS_SOURCE_DIR}/build-i386/modules/rostests/winetests/user32/user32_winetest.exe
    )

set(_wow64_guest_ok TRUE)
foreach(_f ${_wow64_ladder})
    if(NOT EXISTS ${_f})
        set(_wow64_guest_ok FALSE)
        message(STATUS "WOW64 E4: ${_f} missing — syswow64 NOT staged on livecd (WoW64 probes will no-op)")
    endif()
endforeach()
# `vibeos-cli build --no-wow64-stage`: the caller is rebuilding build-i386
# concurrently (a gate), so the closure below would snapshot a half-cleaned tree
# -- measured: `UNRESOLVED import kernel32.dll` and a FATAL configure, because
# --clean deletes the cargo-built kernel32.dll. The gate restages afterwards.
option(VIBEOS_WOW64_NO_STAGE "Defer SysWOW64 staging (a concurrent i386 build is running)" OFF)
if(VIBEOS_WOW64_NO_STAGE)
    set(_wow64_guest_ok FALSE)
    message(STATUS "WOW64 E4: staging DEFERRED (VIBEOS_WOW64_NO_STAGE) — syswow64 NOT staged on this build")
endif()
# Every guest is admitted by an `if(EXISTS)` evaluated at CONFIGURE time, so
# BUILDING one does not make the next build stage it -- the cached configure
# still says it was absent, and the guest then reads as "did not run" rather
# than "was never staged". Depending on the paths was the first fix, and it
# does not work for the case it was for: CMake silently DROPS a configure
# dependency on a file that does not exist (measured, cmake 3.28: the path is
# absent from build.ninja), so creation re-ran nothing. The merge queue's
# integration tree sat in exactly that state -- configured before its guests
# existed, staging none of them forever after.
#
# So depend on the file when it exists, and otherwise on its nearest EXISTING
# ancestor directory: creating an entry bumps that directory's mtime, which
# ninja sees. Outside `if(_wow64_guest_ok)` on purpose -- the refused case is
# the one that needs it.
foreach(_f ${_wow64_ladder} ${_wow64_extras})
    set(_dep ${_f})
    while(NOT EXISTS ${_dep})
        get_filename_component(_dep ${_dep} DIRECTORY)
    endwhile()
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${_dep})
endforeach()
set(_wow64_extras_present "")
foreach(_f ${_wow64_extras})
    if(EXISTS ${_f})
        list(APPEND _wow64_extras_present ${_f})
    else()
        message(STATUS "WOW64 E4: optional ${_f} missing — not staged; the nine ladder guests are unaffected")
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
                # Follow LoadLibrary targets named inside staged libraries too
                # (comctl32 -> msvfw32 for the Animate control). The livecd has
                # room; see string_referenced in the tool for the measurement.
                --library-literals
                # Every guest we STAGE is also a closure root. It was not always so:
                # the roots were the stock applets, and a ladder guest's imports were
                # covered only because some applet happened to import the same DLLs.
                # marshal32 linking ws2_32 was covered by that coincidence; the next
                # guest to link something new would have half-bound instead, which is
                # the crash-at-first-call this closure exists to prevent. Adding them
                # is currently a no-op (measured: same 64 modules), which is the point
                # — it costs nothing and stops depending on luck.
                ${_wow64_ladder}
                ${_wow64_extras_present}
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
    set(_wow64_has_comctl32 FALSE)
    foreach(_line ${_wow64_lines})
        string(REPLACE "\t" ";" _kv "${_line}")
        list(GET _kv 0 _name)
        list(GET _kv 1 _path)
        add_cd_file(FILE ${_path} DESTINATION reactos/syswow64 NAME_ON_CD ${_name} FOR livecd)
        if(_name STREQUAL "comctl32.dll")
            set(_wow64_has_comctl32 ${_path})
        endif()
    endforeach()
    # The x86 side-by-side assemblies. A 64-bit Windows carries both
    # architectures in winsxs; with only the amd64 ones, the guest's comctl32
    # failed CreateActCtxW for both its v5 and v6 contexts, because it looks for
    # x86_microsoft.windows.common-controls_* manifests.
    if(_wow64_has_comctl32)
        foreach(_v 5.82.2600.2982 6.0.2600.2982)
            set(_asm x86_microsoft.windows.common-controls_6595b64144ccf1df_${_v}_none_deadbeef)
            add_cd_file(FILE ${_wow64_has_comctl32} DESTINATION reactos/winsxs/${_asm}
                        NAME_ON_CD comctl32.dll FOR livecd)
            add_cd_file(FILE ${REACTOS_SOURCE_DIR}/dll/win32/comctl32/${_asm}.manifest
                        DESTINATION reactos/winsxs/manifests FOR livecd)
        endforeach()
    endif()
    # The nine graded guests: the existence gate above already proved every one
    # of them, so these are unconditional.
    foreach(_f ${_wow64_ladder})
        add_cd_file(FILE ${_f} DESTINATION reactos/syswow64 FOR livecd)
    endforeach()
    # The robustness extras, whichever survived the presence filter. Same set and
    # same rationale as before (E8 charmap/mspaint/regedit/winver, E9
    # taskmgr/wordpad/clipbrd, E11 spider/eventvwr/devmgmt/msconfig/sndrec32/
    # kbswitch, plus testgui and the stock four) -- only no longer all-or-nothing
    # with the guests.
    foreach(_f ${_wow64_extras_present})
        if("${_f}" MATCHES "doom-i386\\.exe$")
            # winlogon launches it by the name it expects, not the build's.
            add_cd_file(FILE ${_f} DESTINATION reactos/syswow64 NAME_ON_CD doom.exe FOR livecd)
        else()
            add_cd_file(FILE ${_f} DESTINATION reactos/syswow64 FOR livecd)
        endif()
    endforeach()
    # The IWAD is data, not a PE, so it is not a closure root -- it rides here
    # beside doom.exe instead, and only when doom itself made it on.
    if(EXISTS ${REACTOS_SOURCE_DIR}/tools/doom/doom1.wad)
        add_cd_file(FILE ${REACTOS_SOURCE_DIR}/tools/doom/doom1.wad DESTINATION reactos/syswow64 FOR livecd)
    endif()
endif()

# hello64.exe — the BTCpu64 M1 guest (docs/engineering/wow64-x64-guest-design.md,
# Fork B). On arm64 it runs under FEX; HERE it runs natively, and that is the
# point: the same binary passing on real x86-64 hardware semantics is the oracle
# that the guest is right, independently of the emulator. An x64 exe is not a
# WoW64 guest, so system32, and it needs no i386 closure — hence outside both
# the ladder set and _wow64_guest_ok. Same configure-time EXISTS caveat as above,
# with the same nearest-existing-ancestor dependency fix.
# The rest of the x86-64 ladder (M2) rides the same rule: the 32-bit ladder's
# sources rebuilt for x86-64 by build-hello-compute.sh, each staged if built.
foreach(_x64_guest hello64 gui64 win64 file64 async64 marshal64 fault64 thread64
                   selfmod64 reg64 spawn64 apc64)
    set(_x64_exe ${REACTOS_SOURCE_DIR}/build-fex-hello/${_x64_guest}.exe)
    set(_dep ${_x64_exe})
    while(NOT EXISTS ${_dep})
        get_filename_component(_dep ${_dep} DIRECTORY)
    endwhile()
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${_dep})
    if(EXISTS ${_x64_exe} AND NOT VIBEOS_WOW64_NO_STAGE)
        add_cd_file(FILE ${_x64_exe} DESTINATION reactos/system32 FOR livecd)
    else()
        message(STATUS "BTCpu64: ${_x64_exe} not staged — ${_x64_guest} will no-op")
    endif()
endforeach()
# spawn64's native helper for rung 13 (xinject): on amd64 a native amd64
# program like spawn64 itself, staged under the name spawn64 starts.
set(_xinject ${REACTOS_SOURCE_DIR}/build-fex-hello/xinject-amd64.exe)
if(EXISTS ${_xinject} AND NOT VIBEOS_WOW64_NO_STAGE)
    add_cd_file(FILE ${_xinject} DESTINATION reactos/system32 NAME_ON_CD xinject.exe FOR livecd)
else()
    message(STATUS "BTCpu64: ${_xinject} not staged — spawn64 rung 13 will fail")
endif()

endif()
