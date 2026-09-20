/* Stand-in for sdk/include/wine/config.h. schannel_mbedtls.c includes "config.h"
 * before anything else; on the real build that is Wine's generated one. The harness
 * compiles the ReactOS arm of the file, so almost nothing here is read — but the
 * include has to resolve. */
#ifndef HARNESS_CONFIG_H
#define HARNESS_CONFIG_H
#define SONAME_LIBMBEDTLS "mbedtls"
#endif
