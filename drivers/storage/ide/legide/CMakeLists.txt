
add_library(legide MODULE legide.c legide.rc)
set_module_type(legide kernelmodedriver)

# Required for NT5.x pciidex.sys
add_custom_command(
    TARGET legide POST_BUILD
    COMMAND native-pefixup --section:.rsrc,!D $<TARGET_FILE:legide>
    VERBATIM)

add_importlibs(legide pciidex ntoskrnl hal)
add_cd_file(TARGET legide DESTINATION reactos/system32/drivers NO_CAB FOR all)
# legide_reg.inf is part of upstream's new "pciide is the one registered IDE driver"
# scheme, which arrives with ENABLE_UNIATA and their rewritten ATA stack. We kept
# uniata as the registered driver (see uniata/CMakeLists.txt), so registering this
# one too would put two bus drivers on the same devices.
# add_registry_inf(legide_reg.inf)
