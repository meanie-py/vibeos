
add_subdirectory(atl)
add_subdirectory(cmlib)
add_subdirectory(inflib)

if(CMAKE_CROSSCOMPILING)

add_subdirectory(3rdparty)
add_subdirectory(cardlib)
# Wine-derived and OURS to translate, so deliberately NOT under 3rdparty/:
# that directory is a registered vendor CONTAINER where every child must be a
# vendored tree. See vendor.yaml `ours:` -> "Wine-derived support libraries".
add_subdirectory(libwine)
add_subdirectory(strmbase)
# upstream's CRT restructure ("[CRT] Remove obsolete CRT files") deleted our
# live sdk/lib/crt/ (281 files - libcntpr, every driver, and our Rust-backed
# msvcrt link against it) and replaced it with crtheap/llvm-compat/
# ansi_sync_hacks. Declined: our crt below stays authoritative. llvm-compat and
# ansi_sync_hacks DID land on disk in this merge but are deliberately NOT wired
# up here (building them beside our crt is a duplicate-symbol hazard) - don't
# "fix" this by adding them back.
#
# TWO exceptions, both wired up because a target here actually links them:
#   * crtheap - six heap shims (malloc/free/calloc/realloc/_msize/_recalloc).
#     Upstream's new shcore.dll links it, and the auto-merged shlwapi does too.
#   * vcstartup - UNRELATED to the restructure; upstream split it out of
#     vcruntime and dll/win32/ucrtbase links it. See below.
add_subdirectory(apisets)
add_subdirectory(comsupp)
add_subdirectory(conutils)
add_subdirectory(cportlib)
add_subdirectory(crt)
add_subdirectory(cryptlib)
add_subdirectory(cpprt)
add_subdirectory(crtheap)

add_subdirectory(gcc-compat)

add_subdirectory(delayimp)
add_subdirectory(dmilib)
#add_subdirectory(dnslib) Nothing links to this lib.
add_subdirectory(drivers)
add_subdirectory(dxguid)
add_subdirectory(epsapi)
add_subdirectory(evtlib)
# fast486 (x86 software CPU) removed: its only consumers were the HAL x86bios
# INT10 emulation and NTVDM, both dropped in the pre-Vista prune.
add_subdirectory(fslib)

if(STACK_PROTECTOR)
    add_subdirectory(gcc_ssp)
endif()

add_subdirectory(ioevent)
add_subdirectory(lsalib)
add_subdirectory(nt)
add_subdirectory(pathcch)
add_subdirectory(poguid)
add_subdirectory(pseh)

if(KDBG)
    add_subdirectory(rossym)
endif()

add_subdirectory(rtl)
add_subdirectory(scrnsave)
add_subdirectory(skiplist)
add_subdirectory(dmoguids)
add_subdirectory(mfuuid)
add_subdirectory(strmiids)
add_subdirectory(smlib)
add_subdirectory(tdilib)
add_subdirectory(tzlib)
add_subdirectory(udmihelp)
add_subdirectory(uuid)
add_subdirectory(vcruntime)
add_subdirectory(vcstartup)
add_subdirectory(${REACTOS_SOURCE_DIR}/vendor/vkd3d vkd3d)
add_subdirectory(wine2ros)
add_subdirectory(wdmguid)

else()

add_subdirectory(${REACTOS_SOURCE_DIR}/vendor/zlib 3rdparty/zlib)

endif()
