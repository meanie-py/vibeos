add_library(arm64vid MODULE arm64vid.c)
set_module_type(arm64vid kernelmodedriver)
add_importlibs(arm64vid ntoskrnl hal)
add_cd_file(TARGET arm64vid DESTINATION reactos/system32/drivers FOR all)
