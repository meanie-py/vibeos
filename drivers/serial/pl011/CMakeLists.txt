
add_library(pl011 MODULE pl011.c)
set_module_type(pl011 kernelmodedriver)
add_importlibs(pl011 ntoskrnl hal)
add_cd_file(TARGET pl011 DESTINATION reactos/system32/drivers FOR all)
