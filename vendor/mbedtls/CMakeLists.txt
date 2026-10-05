# mbedTLS 3.6.7 (LTS).
#
# EVERY .c UPSTREAM SHIPS IS LISTED, deliberately. 2.16's list was hand-curated with
# 32 entries commented out, which meant every bump was a three-way merge between
# upstream's file set, our comments and whatever the config actually enabled. A source
# whose feature is off compiles to an empty translation unit, so listing all of them
# costs nothing and makes the next bump a copy. What is built is decided in ONE place:
# vendor/mbedtls/include/mbedtls/mbedtls_config.h.

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include
                    ${CMAKE_CURRENT_SOURCE_DIR})

spec2def(mbedtls.dll mbedtls.spec ADD_IMPORTLIB)

# The ReactOS-authored adapter lives OUTSIDE this vendored tree: mbedtls is an
# Apache-2.0 upstream drop and that file is LGPL-2.1-or-later ReactOS code, so keeping
# it here made the tree mixed-licence and made an upstream re-vendor a merge instead of
# a copy. See vendor.yaml's mbedtls entry.
#
# timing_hardclock.c USED to be listed here too, supplying mbedtls_timing_hardclock
# behind -DHAVE_HARDCLOCK. 3.x deleted hardclock from timing.c outright, so both the
# adapter and the define are gone.
list(APPEND SOURCE
    ${REACTOS_SOURCE_DIR}/sdk/lib/mbedtls_reactos/strsafe_wrapper.c
    ${REACTOS_SOURCE_DIR}/sdk/lib/mbedtls_reactos/entropy_hardware_poll.c
    aes.c
    aesce.c
    aesni.c
    aria.c
    asn1parse.c
    asn1write.c
    base64.c
    bignum.c
    bignum_core.c
    bignum_mod.c
    bignum_mod_raw.c
    block_cipher.c
    camellia.c
    ccm.c
    chacha20.c
    chachapoly.c
    cipher.c
    cipher_wrap.c
    cmac.c
    constant_time.c
    ctr_drbg.c
    debug.c
    des.c
    dhm.c
    dsa.c
    ecdh.c
    ecdsa.c
    ecjpake.c
    ecp.c
    ecp_curves.c
    ecp_curves_new.c
    entropy.c
    entropy_poll.c
    error.c
    gcm.c
    hkdf.c
    hmac_drbg.c
    lmots.c
    lms.c
    md.c
    md5.c
    memory_buffer_alloc.c
    mps_reader.c
    mps_trace.c
    net_sockets.c
    nist_kw.c
    oid.c
    padlock.c
    pem.c
    pk.c
    pk_ecc.c
    pk_wrap.c
    pkcs12.c
    pkcs5.c
    pkcs7.c
    pkparse.c
    pkwrite.c
    platform.c
    platform_util.c
    poly1305.c
    psa_crypto.c
    psa_crypto_aead.c
    psa_crypto_cipher.c
    psa_crypto_client.c
    psa_crypto_driver_wrappers_no_static.c
    psa_crypto_ecp.c
    psa_crypto_ffdh.c
    psa_crypto_hash.c
    psa_crypto_mac.c
    psa_crypto_pake.c
    psa_crypto_random.c
    psa_crypto_rsa.c
    psa_crypto_se.c
    psa_crypto_slot_management.c
    psa_crypto_storage.c
    psa_its_file.c
    psa_util.c
    ripemd160.c
    rsa.c
    rsa_alt_helpers.c
    sha1.c
    sha256.c
    sha3.c
    sha512.c
    ssl_cache.c
    ssl_ciphersuites.c
    ssl_client.c
    ssl_cookie.c
    ssl_debug_helpers_generated.c
    ssl_msg.c
    ssl_ticket.c
    ssl_tls.c
    ssl_tls12_client.c
    ssl_tls12_server.c
    ssl_tls13_client.c
    ssl_tls13_generic.c
    ssl_tls13_keys.c
    ssl_tls13_server.c
    threading.c
    timing.c
    version.c
    version_features.c
    x509.c
    x509_create.c
    x509_crl.c
    x509_crt.c
    x509_csr.c
    x509write.c
    x509write_crt.c
    x509write_csr.c
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

# 3.x rewrote aesni.c from inline asm to AES-NI/PCLMUL intrinsics, which the compiler
# refuses to emit unless the ISA is enabled for that translation unit -- and our x86
# baseline is -march=pentium, deliberately, because ReactOS runs on old hardware.
# Enabling them for this ONE file is safe and is what upstream does: every entry point
# in it is gated at runtime behind mbedtls_aesni_has_support(), so a CPU without the
# instructions never reaches them. On arm64 the whole file preprocesses away
# (MBEDTLS_ARCH_IS_X86/X64), so there is nothing to flag.
if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
    set_source_files_properties(aesni.c PROPERTIES COMPILE_OPTIONS "-msse2;-maes;-mpclmul")
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(mbedtls PRIVATE -Wno-tautological-constant-compare)
endif()
