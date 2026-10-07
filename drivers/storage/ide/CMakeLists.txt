
add_subdirectory(atapi)
if(ARCH STREQUAL "i386" AND NOT ENABLE_UNIATA)
    add_subdirectory(legide)
endif()
add_subdirectory(pciide)
add_subdirectory(pciidex)
add_subdirectory(uniata)

# Opt-in boot on upstream's new ATA stack (./vibeos-cli build --ata-stack). uniata stays the
# default storage driver; this re-points the CriticalDeviceDatabase and registers the three
# services upstream's 7d33f7503bb switched to. The drivers themselves are built either way.
option(VIBEOS_ATA_STACK "Boot storage on pciide/pciidex/atapi instead of uniata" OFF)
if(VIBEOS_ATA_STACK)
    # Fail loud rather than boot a half-built stack: atapi.sys and pciidex.sys still build
    # from the PRE-rewrite translations (atapi-legacy-rust is a SCSIPORT miniport, not the
    # port driver this overlay maps the channels to) until atapi-rust/pciidex-rust compile.
    message(FATAL_ERROR
        "VIBEOS_ATA_STACK: the new ATA stack's Rust (atapi-rust, pciidex-rust, legide-rust) "
        "is translated but not wired into the build yet -- see "
        "docs/engineering/active-lanes/upstream-sync-merge-decisions.md, 'The new ATA stack'.")
    add_registry_inf(ata_stack.inf)
    add_registry_inf(pciide/pciide_reg.inf atapi/atapi_reg.inf)
    if(ARCH STREQUAL "i386")
        add_registry_inf(legide/legide_reg.inf)
    endif()
endif()
