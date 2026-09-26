
add_subdirectory(packages)

# Common hives

add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/txtsetup.sif DESTINATION reactos NO_CAB FOR bootcd regtest)

add_custom_target(converted_caroots_inf DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/caroots.inf)
utf16le_convert(${CMAKE_CURRENT_SOURCE_DIR}/caroots.inf ${CMAKE_CURRENT_BINARY_DIR}/caroots.inf)
add_cd_file(TARGET converted_caroots_inf FILE ${CMAKE_CURRENT_BINARY_DIR}/caroots.inf DESTINATION reactos NO_CAB FOR bootcd regtest)

set(_hivesys_infs
    hivecls.inf
    hivedef.inf
    hivesft.inf
    hivesys.inf
    hivebcd.inf)

if(ARCH STREQUAL "arm64")
    list(APPEND _hivesys_infs hivesys_arm64.inf)
endif()

add_registry_inf(${_hivesys_infs})


# Regtest
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/bootcdregtest/regtest.cmd DESTINATION reactos/bin FOR all)

# autorun.inf
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/autorun-bootcd.inf DESTINATION root NO_CAB NOT_IN_HYBRIDCD NAME_ON_CD autorun.inf FOR bootcd)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/autorun-livecd.inf DESTINATION root NO_CAB NOT_IN_HYBRIDCD NAME_ON_CD autorun.inf FOR livecd)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/autorun-hybridcd.inf DESTINATION root NO_CAB NOT_IN_HYBRIDCD NAME_ON_CD autorun.inf FOR hybridcd)

# icon.ico
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/icon.ico DESTINATION root NO_CAB NOT_IN_HYBRIDCD FOR all hybridcd)

# readme.txt
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/readme.txt DESTINATION root NO_CAB NOT_IN_HYBRIDCD FOR all hybridcd)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/readme.txt DESTINATION reactos FOR all)

# Welcome.exe optional custom configuration (only for HybridCD)
if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/welcome_config/)
    # Copy the main configuration file
    add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/welcome_config/welcome.ini DESTINATION bootcd/reactos NO_CAB FOR hybridcd)

    # Convert the translation files (name format: xx-YY.ini) into UTF-16
    file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/welcome_config)
    file(GLOB I18N_FILES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR}/welcome_config/ ${CMAKE_CURRENT_SOURCE_DIR}/welcome_config/*-*.ini)
    foreach(_file ${I18N_FILES})
        set(_converted_file ${CMAKE_CURRENT_BINARY_DIR}/welcome_config/${_file})
        set(_source_file ${CMAKE_CURRENT_SOURCE_DIR}/welcome_config/${_file})
        utf16le_convert(${_source_file} ${_converted_file})
        add_cd_file(TARGET converted_welcome_i18n_files FILE ${_converted_file} DESTINATION bootcd/reactos/welcome NO_CAB NAME_ON_CD ${_file} FOR hybridcd)
        list(APPEND _converted_welcome_i18n_files ${_converted_file})
    endforeach(_file)
    add_custom_target(converted_welcome_i18n_files DEPENDS ${_converted_welcome_i18n_files})
endif()

# freeldr.ini
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/bootcd.ini DESTINATION root NO_CAB NOT_IN_HYBRIDCD NAME_ON_CD freeldr.ini FOR bootcd regtest)
if(VIBEOS_UP_DEFAULT_BOOT)
    # UP OPT-OUT (`vibeos-cli build --up-kernel`). The shipped livecd.ini defaults
    # to LiveCD_SMP (ntkrnlmp.exe + halmp.dll) since 2026-09-12; this knob is the
    # only way to get media that boots the UP pair instead. Nothing automated
    # should set it -- gate, smoke and conformance all test MP, because UP is a
    # dead branch and a green UP run tells us nothing we are going to ship.
    # TimeOut=0 means the default entry is the one automated boots get.
    file(READ ${CMAKE_CURRENT_SOURCE_DIR}/livecd.ini _livecd_ini)
    string(REPLACE "DefaultOS=LiveCD_SMP" "DefaultOS=LiveCD_Debug" _livecd_ini "${_livecd_ini}")
    file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/livecd-up.ini "${_livecd_ini}")
    add_cd_file(FILE ${CMAKE_CURRENT_BINARY_DIR}/livecd-up.ini DESTINATION root NOT_IN_HYBRIDCD NAME_ON_CD freeldr.ini FOR livecd)
else()
    add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/livecd.ini DESTINATION root NOT_IN_HYBRIDCD NAME_ON_CD freeldr.ini FOR livecd)
endif()
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/hybridcd.ini DESTINATION root NAME_ON_CD freeldr.ini FOR hybridcd)

# Unattend
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/bootcdregtest/unattend.inf DESTINATION reactos NO_CAB FOR regtest)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/bootcd/unattend.inf DESTINATION reactos NO_CAB FOR bootcd)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/livecd/unattend.inf DESTINATION reactos NO_CAB FOR livecd)

# LiveCD shortcuts
macro(add_livecd_shortcut name app dest)
    add_link(NAME ${name} CMD_LINE_ARGS ${app} ICON ${app} PATH livecd_start.cmd GUID "{450D8FBA-AD25-11D0-98A8-0800361B1103}" MINIMIZE)
    list(APPEND LIVECD_SHORTCUTS "${CMAKE_CURRENT_BINARY_DIR}/${name}.lnk")
    add_cd_file(FILE "${CMAKE_CURRENT_BINARY_DIR}/${name}.lnk"
                TARGET livecd_links
                DESTINATION ${dest}
                FOR livecd)
endmacro(add_livecd_shortcut)

## NOTE: What would be nice is to create this list using /media/inf/shortcuts.inf
## and taking their default english translation!
add_cd_file(FILE "${CMAKE_CURRENT_SOURCE_DIR}/livecd_start.cmd" DESTINATION "Profiles/Default User/My Documents" FOR livecd)
add_livecd_shortcut("Read Me" readme.txt "Profiles/All Users/Desktop")
add_livecd_shortcut("Command Prompt" cmd.exe "Profiles/All Users/Desktop")
add_livecd_shortcut("Device Manager" devmgmt.exe "Profiles/All Users/Start Menu/Programs/Administrative Tools")
add_livecd_shortcut("Event Viewer" eventvwr.exe "Profiles/All Users/Start Menu/Programs/Administrative Tools")
add_livecd_shortcut("Service Manager" servman.exe "Profiles/All Users/Start Menu/Programs/Administrative Tools")
add_livecd_shortcut("System Configuration" msconfig.exe "Profiles/All Users/Start Menu/Programs/Administrative Tools")
add_livecd_shortcut("Accessibility Utility Manager" utilman.exe "Profiles/All Users/Start Menu/Programs/Accessories/Accessibility")
add_livecd_shortcut("Magnify" magnify.exe "Profiles/All Users/Start Menu/Programs/Accessories/Accessibility")
add_livecd_shortcut("On-Screen Keyboard" osk.exe "Profiles/All Users/Start Menu/Programs/Accessories/Accessibility")
add_livecd_shortcut("Remote Desktop Connection" mstsc.exe "Profiles/All Users/Start Menu/Programs/Accessories/Communications")
add_livecd_shortcut("Multimedia Player" mplay32.exe "Profiles/All Users/Start Menu/Programs/Accessories/Entertainment")
add_livecd_shortcut("Sound Recorder" sndrec32.exe "Profiles/All Users/Start Menu/Programs/Accessories/Entertainment")
add_livecd_shortcut("Volume Control" sndvol32.exe "Profiles/All Users/Start Menu/Programs/Accessories/Entertainment")
add_livecd_shortcut("Character Map" charmap.exe "Profiles/All Users/Start Menu/Programs/Accessories/System Tools")
add_livecd_shortcut("Clipboard Viewer" clipbrd.exe "Profiles/All Users/Start Menu/Programs/Accessories/System Tools")
add_livecd_shortcut("Keyboard Layout Switcher" kbswitch.exe "Profiles/All Users/Start Menu/Programs/Accessories/System Tools")
add_livecd_shortcut("ReactX Diagnostic" dxdiag.exe "Profiles/All Users/Start Menu/Programs/Accessories/System Tools")
add_livecd_shortcut("Registry Editor" regedit.exe "Profiles/All Users/Start Menu/Programs/Accessories/System Tools")
add_livecd_shortcut("Task Manager" taskmgr.exe "Profiles/All Users/Start Menu/Programs/Accessories/System Tools")
add_livecd_shortcut("Calculator" calc.exe "Profiles/All Users/Start Menu/Programs/Accessories")
add_livecd_shortcut("Command Prompt" cmd.exe "Profiles/All Users/Start Menu/Programs/Accessories")
add_livecd_shortcut("Paint" mspaint.exe "Profiles/All Users/Start Menu/Programs/Accessories")
add_livecd_shortcut("Notepad" notepad.exe "Profiles/All Users/Start Menu/Programs/Accessories")
add_livecd_shortcut("WordPad" wordpad.exe "Profiles/All Users/Start Menu/Programs/Accessories")
add_livecd_shortcut("Solitaire" sol.exe "Profiles/All Users/Start Menu/Programs/Games")
add_livecd_shortcut("Spider Solitaire" spider.exe "Profiles/All Users/Start Menu/Programs/Games")
add_livecd_shortcut("WineMine" winmine.exe "Profiles/All Users/Start Menu/Programs/Games")
add_livecd_shortcut("ReactOS Explorer" explorer.exe "Profiles/All Users/Start Menu/Programs")
add_livecd_shortcut("Command Prompt" cmd.exe "Profiles/Default User/Application Data/Microsoft/Internet Explorer/Quick Launch")
add_livecd_shortcut("ReactOS Explorer" explorer.exe "Profiles/Default User/Application Data/Microsoft/Internet Explorer/Quick Launch")

add_custom_target(livecd_links DEPENDS ${LIVECD_SHORTCUTS})

# The NATIVE half of the winetest A/B (i386 only).
#
# winlogon runs the same i386 user32_winetest.exe twice: once as a WoW64 guest
# on amd64 media, and once here on bare i386. Subtracting the two is the only
# thing that makes a red Wine assertion attributable -- a failure present in
# both is our user32, a failure present only under WoW64 is the 32<->64 seam.
# Without the control every finding reads equally as either.
#
# Staged by `if(EXISTS)` on the build output rather than through
# add_rostests_file, deliberately: that macro adds the file `FOR all`, which
# makes the livecd target DEPEND on every other rostests target -- and two of
# them (rosautotest, interop_unittest) do not build, so the i386 livecd cannot
# be produced with ENABLE_ROSTESTS at all. This way the opt-in is still just
# "run an i386 build --rostests once", and the media that carries the control
# is an ordinary one.
# Not while ENABLE_ROSTESTS is on: then that path is also the user32_winetest
# target's OUTPUT, and naming it here (as a staged file OR as a configure
# dependency) makes ninja refuse the whole manifest ("multiple rules generate
# ..."), so the exe could never be rebuilt. The plain build that follows turns
# rostests back off and stages it.
if(ARCH STREQUAL "i386" AND NOT ENABLE_ROSTESTS)
    set(_native_winetest
        ${REACTOS_SOURCE_DIR}/build-i386/modules/rostests/winetests/user32/user32_winetest.exe)
    # Its CREATION must re-run configure, or a tree that builds it after the
    # last configure keeps the cached "absent" answer and stages nothing --
    # which reads as "the control did not run" rather than "was never staged".
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${_native_winetest})
    if(EXISTS ${_native_winetest})
        message(STATUS "winetest A/B: staging the native i386 user32_winetest onto livecd")
        add_cd_file(FILE ${_native_winetest} DESTINATION reactos/bin FOR livecd)
    endif()
endif()
