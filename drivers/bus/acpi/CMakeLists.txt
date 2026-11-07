# ACPICA itself is vendored at vendor/acpica; the acpica library is still
# defined here, beside the ReactOS driver glue that is its only consumer.
set(ACPICA_DIR ${REACTOS_SOURCE_DIR}/vendor/acpica)

include_directories(
    include
    ${ACPICA_DIR}/include)

if(ARCH STREQUAL "amd64")
    add_definitions(-DWIN64)
endif()

list(APPEND ACPICA_SOURCE
    ${ACPICA_DIR}/dispatcher/dsargs.c
    ${ACPICA_DIR}/dispatcher/dscontrol.c
    ${ACPICA_DIR}/dispatcher/dsdebug.c
    ${ACPICA_DIR}/dispatcher/dsfield.c
    ${ACPICA_DIR}/dispatcher/dsinit.c
    ${ACPICA_DIR}/dispatcher/dsmethod.c
    ${ACPICA_DIR}/dispatcher/dsmthdat.c
    ${ACPICA_DIR}/dispatcher/dsobject.c
    ${ACPICA_DIR}/dispatcher/dsopcode.c
    ${ACPICA_DIR}/dispatcher/dspkginit.c
    ${ACPICA_DIR}/dispatcher/dsutils.c
    ${ACPICA_DIR}/dispatcher/dswexec.c
    ${ACPICA_DIR}/dispatcher/dswload.c
    ${ACPICA_DIR}/dispatcher/dswload2.c
    ${ACPICA_DIR}/dispatcher/dswscope.c
    ${ACPICA_DIR}/dispatcher/dswstate.c
    ${ACPICA_DIR}/events/evevent.c
    ${ACPICA_DIR}/events/evglock.c
    ${ACPICA_DIR}/events/evgpe.c
    ${ACPICA_DIR}/events/evgpeblk.c
    ${ACPICA_DIR}/events/evgpeinit.c
    ${ACPICA_DIR}/events/evgpeutil.c
    ${ACPICA_DIR}/events/evhandler.c
    ${ACPICA_DIR}/events/evmisc.c
    ${ACPICA_DIR}/events/evregion.c
    ${ACPICA_DIR}/events/evrgnini.c
    ${ACPICA_DIR}/events/evsci.c
    ${ACPICA_DIR}/events/evxface.c
    ${ACPICA_DIR}/events/evxfevnt.c
    ${ACPICA_DIR}/events/evxfgpe.c
    ${ACPICA_DIR}/events/evxfregn.c
    ${ACPICA_DIR}/executer/exconcat.c
    ${ACPICA_DIR}/executer/exconfig.c
    ${ACPICA_DIR}/executer/exconvrt.c
    ${ACPICA_DIR}/executer/excreate.c
    ${ACPICA_DIR}/executer/exdebug.c
    # ${ACPICA_DIR}/executer/exdump.c
    ${ACPICA_DIR}/executer/exfield.c
    ${ACPICA_DIR}/executer/exfldio.c
    ${ACPICA_DIR}/executer/exmisc.c
    ${ACPICA_DIR}/executer/exmutex.c
    ${ACPICA_DIR}/executer/exnames.c
    ${ACPICA_DIR}/executer/exoparg1.c
    ${ACPICA_DIR}/executer/exoparg2.c
    ${ACPICA_DIR}/executer/exoparg3.c
    ${ACPICA_DIR}/executer/exoparg6.c
    ${ACPICA_DIR}/executer/exprep.c
    ${ACPICA_DIR}/executer/exregion.c
    ${ACPICA_DIR}/executer/exresnte.c
    ${ACPICA_DIR}/executer/exresolv.c
    ${ACPICA_DIR}/executer/exresop.c
    ${ACPICA_DIR}/executer/exserial.c
    ${ACPICA_DIR}/executer/exstore.c
    ${ACPICA_DIR}/executer/exstoren.c
    ${ACPICA_DIR}/executer/exstorob.c
    ${ACPICA_DIR}/executer/exsystem.c
    ${ACPICA_DIR}/executer/extrace.c
    ${ACPICA_DIR}/executer/exutils.c
    ${ACPICA_DIR}/hardware/hwacpi.c
    ${ACPICA_DIR}/hardware/hwesleep.c
    ${ACPICA_DIR}/hardware/hwgpe.c
    ${ACPICA_DIR}/hardware/hwpci.c
    ${ACPICA_DIR}/hardware/hwregs.c
    ${ACPICA_DIR}/hardware/hwsleep.c
    ${ACPICA_DIR}/hardware/hwtimer.c
    ${ACPICA_DIR}/hardware/hwvalid.c
    ${ACPICA_DIR}/hardware/hwxface.c
    ${ACPICA_DIR}/hardware/hwxfsleep.c
    ${ACPICA_DIR}/namespace/nsaccess.c
    ${ACPICA_DIR}/namespace/nsalloc.c
    ${ACPICA_DIR}/namespace/nsarguments.c
    ${ACPICA_DIR}/namespace/nsconvert.c
    # ${ACPICA_DIR}/namespace/nsdump.c
    # ${ACPICA_DIR}/namespace/nsdumpdv.c
    ${ACPICA_DIR}/namespace/nseval.c
    ${ACPICA_DIR}/namespace/nsinit.c
    ${ACPICA_DIR}/namespace/nsload.c
    ${ACPICA_DIR}/namespace/nsnames.c
    ${ACPICA_DIR}/namespace/nsobject.c
    ${ACPICA_DIR}/namespace/nsparse.c
    ${ACPICA_DIR}/namespace/nspredef.c
    ${ACPICA_DIR}/namespace/nsprepkg.c
    ${ACPICA_DIR}/namespace/nsrepair.c
    ${ACPICA_DIR}/namespace/nsrepair2.c
    ${ACPICA_DIR}/namespace/nssearch.c
    ${ACPICA_DIR}/namespace/nsutils.c
    ${ACPICA_DIR}/namespace/nswalk.c
    ${ACPICA_DIR}/namespace/nsxfeval.c
    ${ACPICA_DIR}/namespace/nsxfname.c
    ${ACPICA_DIR}/namespace/nsxfobj.c
    ${ACPICA_DIR}/parser/psargs.c
    ${ACPICA_DIR}/parser/psloop.c
    ${ACPICA_DIR}/parser/psobject.c
    ${ACPICA_DIR}/parser/psopcode.c
    ${ACPICA_DIR}/parser/psopinfo.c
    ${ACPICA_DIR}/parser/psparse.c
    ${ACPICA_DIR}/parser/psscope.c
    ${ACPICA_DIR}/parser/pstree.c
    ${ACPICA_DIR}/parser/psutils.c
    ${ACPICA_DIR}/parser/pswalk.c
    ${ACPICA_DIR}/parser/psxface.c
    ${ACPICA_DIR}/resources/rsaddr.c
    ${ACPICA_DIR}/resources/rscalc.c
    ${ACPICA_DIR}/resources/rscreate.c
    # ${ACPICA_DIR}/resources/rsdump.c
    # ${ACPICA_DIR}/resources/rsdumpinfo.c
    ${ACPICA_DIR}/resources/rsinfo.c
    ${ACPICA_DIR}/resources/rsio.c
    ${ACPICA_DIR}/resources/rsirq.c
    ${ACPICA_DIR}/resources/rslist.c
    ${ACPICA_DIR}/resources/rsmemory.c
    ${ACPICA_DIR}/resources/rsmisc.c
    ${ACPICA_DIR}/resources/rsserial.c
    ${ACPICA_DIR}/resources/rsutils.c
    ${ACPICA_DIR}/resources/rsxface.c
    ${ACPICA_DIR}/tables/tbdata.c
    ${ACPICA_DIR}/tables/tbfadt.c
    ${ACPICA_DIR}/tables/tbfind.c
    ${ACPICA_DIR}/tables/tbinstal.c
    ${ACPICA_DIR}/tables/tbprint.c
    ${ACPICA_DIR}/tables/tbutils.c
    ${ACPICA_DIR}/tables/tbxface.c
    ${ACPICA_DIR}/tables/tbxfload.c
    ${ACPICA_DIR}/tables/tbxfroot.c
    ${ACPICA_DIR}/utilities/utaddress.c
    ${ACPICA_DIR}/utilities/utalloc.c
    ${ACPICA_DIR}/utilities/utascii.c
    ${ACPICA_DIR}/utilities/utbuffer.c
    ${ACPICA_DIR}/utilities/utcache.c
    ${ACPICA_DIR}/utilities/utcksum.c
    # ${ACPICA_DIR}/utilities/utclib.c
    ${ACPICA_DIR}/utilities/utcopy.c
    # ${ACPICA_DIR}/utilities/utdebug.c
    ${ACPICA_DIR}/utilities/utdecode.c
    ${ACPICA_DIR}/utilities/utdelete.c
    ${ACPICA_DIR}/utilities/uterror.c
    ${ACPICA_DIR}/utilities/uteval.c
    # ${ACPICA_DIR}/utilities/utfileio.c
    ${ACPICA_DIR}/utilities/uthex.c
    ${ACPICA_DIR}/utilities/utids.c
    ${ACPICA_DIR}/utilities/utinit.c
    ${ACPICA_DIR}/utilities/utlock.c
    ${ACPICA_DIR}/utilities/utmath.c
    ${ACPICA_DIR}/utilities/utmisc.c
    ${ACPICA_DIR}/utilities/utmutex.c
    ${ACPICA_DIR}/utilities/utnonansi.c
    ${ACPICA_DIR}/utilities/utobject.c
    ${ACPICA_DIR}/utilities/utosi.c
    ${ACPICA_DIR}/utilities/utownerid.c
    ${ACPICA_DIR}/utilities/utpredef.c
    # ${ACPICA_DIR}/utilities/utprint.c
    # ${ACPICA_DIR}/utilities/utresdecode.c
    ${ACPICA_DIR}/utilities/utresrc.c
    ${ACPICA_DIR}/utilities/utstate.c
    ${ACPICA_DIR}/utilities/utstring.c
    ${ACPICA_DIR}/utilities/utstrtoul64.c
    ${ACPICA_DIR}/utilities/utstrsuppt.c
    # ${ACPICA_DIR}/utilities/utuuid.c
    ${ACPICA_DIR}/utilities/uttrack.c
    ${ACPICA_DIR}/utilities/utxface.c
    ${ACPICA_DIR}/utilities/utxferror.c
    ${ACPICA_DIR}/utilities/utxfinit.c
    ${ACPICA_DIR}/utilities/utxfmutex.c)

list(APPEND ACPICA_PCH_SKIP_SOURCE
    ${ACPICA_DIR}/utilities/utexcep.c
    ${ACPICA_DIR}/utilities/utglobal.c)

add_library(acpica
    ${ACPICA_SOURCE}
    ${ACPICA_PCH_SKIP_SOURCE})

add_pch(acpica ${ACPICA_DIR}/include/acpi.h "${ACPICA_PCH_SKIP_SOURCE}")
set_target_properties(acpica PROPERTIES COMPILE_DEFINITIONS "ACPI_USE_LOCAL_CACHE;ACPI_USE_SYSTEM_INTTYPES")
add_dependencies(acpica bugcodes xdk)

list(APPEND ACPI_SOURCE
    # busmgr/bus.c — replaced by Rust (sdk/lib/rust/acpi-rust/src/bus.rs, Phase 3a:
    #   the acpi_bus_* bus-manager API). Symbols from libacpi_rust.a; C kept on disk.
    # Phase 3b — the remaining ReactOS ACPI driver glue is now Rust. Each .c below is
    #   replaced by its acpi-rust module (#[no_mangle] symbols win at link); C kept on disk.
    # busmgr/button.c  -> button.rs
    # busmgr/power.c   -> busmgr_power.rs
    # busmgr/utils.c   -> utils.rs
    # busmgr/system.c  -> system.rs
    # osl.c — replaced by Rust (sdk/lib/rust/acpi-rust/, ACPICA FFI-to-blob
    #   Phase 2: the 45 AcpiOs* OSL callbacks). Symbols come from libacpi_rust.a
    #   (linked below); the C is kept on disk per the never-delete rule. Only the
    #   C-variadic AcpiOsPrintf stays C (acpi_printf_shim.c → Rust AcpiOsVprintf).
    acpi_printf_shim.c
    # acpienum.c   -> acpienum.rs   (Phase 3b)
    # eval.c       -> eval.rs       (Phase 3b)
    # interface.c  -> interface.rs  (Phase 3b)
    # pnp.c        -> pnp.rs        (Phase 3b)
    # power.c      -> power.rs      (Phase 3b)
    # buspdo.c — replaced by Rust (sdk/lib/rust/acpi-rust/src/buspdo.rs, Phase 3a:
    #   the Bus_PDO_* PnP/resource dispatch). Symbols from libacpi_rust.a; C kept on disk.
    # main.c       -> driver_main.rs (Phase 3b)
    )

# ============================================================================
# acpi-rust — OSL (osl.c) Rust cohabitation flip. Mirrors processr/pl011: a
# crate-type=staticlib → libacpi_rust.a whose #[no_mangle] AcpiOs* symbols take
# over from the (commented-out) osl.c. ACPICA (libacpica.a) stays C and resolves
# its OS-services callbacks from the Rust archive. acpi-rust deps ntkrnl-sys +
# acpica-sys; the custom_command mirrors the swenum/processr cohab.
# ============================================================================

add_driver_inf(acpi acpi.inf)

# acpi.sys is all Rust (acpi-rust) and cargo links it:
# sdk/lib/rust/acpi-cargo (reactos-link, generated by tools/cargo_cell_gen.py).
# The C stays on disk, uncompiled.
# The C still compiled, in an OBJECT library whose compile the cargo link replays.
add_library(acpi_shims OBJECT acpi_printf_shim.c)
target_link_libraries(acpi_shims PRIVATE wdmguid acpica memcmp libntoskrnl libhal)
add_cargo_module(acpi acpi-cargo acpi.sys
    KIND kernelmodedriver
    BODY_CRATE acpi-rust
    DESTINATION reactos/system32/drivers
    NO_CAB
    EXTRA_DEPENDS acpi_shims psdk bugcodes xdk)
