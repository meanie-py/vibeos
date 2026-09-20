# -DHAVE_HARDCLOCK suppresses every hardclock definition in upstream's timing.c
# (each is guarded on `!defined(HAVE_HARDCLOCK)`); we supply the symbol from
# sdk/lib/mbedtls_reactos/timing_hardclock.c so timing.c stays pristine.
add_definitions(-DHAVE_HARDCLOCK)


include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs
                    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/zlib)

spec2def(mbedtls.dll mbedtls.spec ADD_IMPORTLIB)

# The ReactOS-authored adapter lives OUTSIDE this vendored tree: mbedtls is an
# Apache-2.0 upstream drop and that file is LGPL-2.1-or-later ReactOS code, so keeping
# it here made the tree mixed-licence and made an upstream re-vendor a merge instead of
# a copy. See vendor.yaml's mbedtls entry.
list(APPEND SOURCE
    ${REACTOS_SOURCE_DIR}/sdk/lib/mbedtls_reactos/strsafe_wrapper.c
    ${REACTOS_SOURCE_DIR}/sdk/lib/mbedtls_reactos/timing_hardclock.c
    aes.c
    aesni.c
    arc4.c
    #aria.c
    asn1parse.c
    asn1write.c
    #base64.c
    bignum.c
    blowfish.c
    camellia.c
    ccm.c
    #certs.c
    #chacha20.c
    #chachapoly.c
    cipher.c
    cipher_wrap.c
    #cmac.c
    ctr_drbg.c
    #debug.c
    des.c
    dhm.c
    ecdh.c
    ecdsa.c
    ecp.c
    ecp_curves.c
    entropy.c
    entropy_poll.c
    #error.c
    gcm.c
    #havege.c
    #hkdf.c
    hmac_drbg.c
    #md2.c
    #md4.c
    md5.c
    md.c
    md_wrap.c
    #memory_buffer_alloc.c
    #net_sockets.c
    #nist_kw.c
    oid.c
    padlock.c
    #pem.c
    pk.c
    #pkcs11.c
    pkcs12.c
    pkcs5.c
    pkparse.c
    pk_wrap.c
    #pkwrite.c
    platform.c
    platform_util.c
    #poly1305.c
    ripemd160.c
    rsa.c
    rsa_internal.c
    sha1.c
    sha256.c
    sha512.c
    #ssl_cache.c
    ssl_ciphersuites.c
    ssl_cli.c
    #ssl_cookie.c
    #ssl_srv.c
    #ssl_ticket.c
    ssl_tls.c
    #threading.c
    timing.c
    #version.c
    #version_features.c
    x509.c
    #x509_create.c
    #x509_crl.c
    x509_crt.c
    #x509_csr.c
    #x509write_crt.c
    #x509write_csr.c
    #xtea.c
    )

add_library(mbedtls MODULE
    ${SOURCE}
    mbedtls.rc
    ${CMAKE_CURRENT_BINARY_DIR}/mbedtls.def)

set_module_type(mbedtls win32dll)
add_importlibs(mbedtls advapi32 msvcrt kernel32 ntdll)

# to use `_vsnprintf_s` looks like we have to define MINGW_HAS_SECURE_API
# and explicitly export it from msvcrt for it to work... what on earth?!
add_definitions(-DMINGW_HAS_SECURE_API -DCRTDLL)

add_cd_file(TARGET mbedtls DESTINATION reactos/system32 FOR all)

if(NOT MSVC)
    target_compile_options(mbedtls PRIVATE -Wno-pointer-sign -Wno-unused-function)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(mbedtls PRIVATE -Wno-tautological-constant-compare)
endif()
