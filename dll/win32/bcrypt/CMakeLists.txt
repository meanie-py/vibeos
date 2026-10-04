
include_directories(
    ${REACTOS_SOURCE_DIR}/vendor/mbedtls/include
    ${REACTOS_SOURCE_DIR}/vendor/mbedtls/include/mbedtls)

# Wine-Staging 10.6's Mbed TLS backend was written against Mbed TLS 2.x and reads struct fields
# that 3.x moved behind MBEDTLS_PRIVATE; our vendored tree is 3.6.7. The 3.x-only
# differences (removed MD2/MD4/RC4/CFB8, mbedtls_pk_sign's sig_size) are #if'd in that source.
add_definitions(-DMBEDTLS_ALLOW_PRIVATE_ACCESS)

spec2def(bcrypt.dll bcrypt.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    bcrypt_main.c
    gnutls.c
    mbedtls.c
    commoncrypto.c
    version.rc
    ${CMAKE_CURRENT_BINARY_DIR}/bcrypt_stubs.c)

add_library(bcrypt MODULE
    ${SOURCE}
    version.rc
    ${CMAKE_CURRENT_BINARY_DIR}/bcrypt.def)
add_pch(bcrypt precomp.h SOURCE)

set_module_type(bcrypt win32dll)
target_link_libraries(bcrypt wine)
add_importlibs(bcrypt mbedtls advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET bcrypt DESTINATION reactos/system32 FOR all)
set_wine_module(bcrypt)
