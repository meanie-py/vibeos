
add_library(arm64input MODULE arm64input.c)
set_module_type(arm64input kernelmodedriver)
add_importlibs(arm64input ntoskrnl hal)
