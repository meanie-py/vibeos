/* Win32 stand-ins for building dll/win32/schannel/schannel_mbedtls.c on the host.
 *
 * The real precomp.h pulls in the whole PSDK. The backend needs a small, bounded
 * slice of it — measured, not guessed: 22 CALG_*, 8 SP_PROT_*, three SEC_E_*, one
 * SECPKG_*, HeapAlloc/HeapFree/GetProcessHeap, and the certificate-store call in
 * schan_imp_get_session_peer_certificate. Everything here is that slice.
 *
 * The schannel-side structs and the schan_imp_* prototypes are NOT duplicated here.
 * build.rs slices them out of the real dll/win32/schannel/schannel_priv.h and drops
 * them in OUT_DIR, so a change to that contract reaches this harness instead of
 * silently drifting away from it.
 */
#ifndef SCHAN_HARNESS_PRECOMP_H
#define SCHAN_HARNESS_PRECOMP_H

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

typedef int             BOOL;
typedef unsigned char   BYTE;
typedef unsigned short  WORD;
typedef unsigned long   DWORD;
typedef unsigned long   ULONG;
typedef long            LONG;
typedef size_t          SIZE_T;
typedef void           *PVOID;
typedef void           *HANDLE;
typedef const char     *LPCSTR;
typedef wchar_t         WCHAR;

#ifndef TRUE
#define TRUE  1
#define FALSE 0
#endif

#define WINAPI
#define DECLSPEC_HIDDEN
#define NO_ERROR 0L
#define HEAP_ZERO_MEMORY 0x00000008

typedef LONG SECURITY_STATUS;

#define SEC_E_OK                    ((SECURITY_STATUS)0x00000000L)
#define SEC_E_INTERNAL_ERROR        ((SECURITY_STATUS)0x80090304L)
#define SEC_E_UNSUPPORTED_FUNCTION  ((SECURITY_STATUS)0x80090302L)
#define SEC_E_ILLEGAL_MESSAGE       ((SECURITY_STATUS)0x80090326L)
#define SEC_I_CONTINUE_NEEDED       ((SECURITY_STATUS)0x00090312L)
#define SEC_E_BUFFER_TOO_SMALL      ((SECURITY_STATUS)0x80090321L)

#define SECPKG_CRED_INBOUND   0x00000001
#define SECPKG_CRED_OUTBOUND  0x00000002

/* wincrypt.h algorithm identifiers — generated from the real PSDK header by
 * build.rs, never restated here. See the note in build.rs for why. */
#include "wincrypt_algs.h"

/* schannel.h protocol bits. */
#define SP_PROT_SSL3_SERVER   0x00000010
#define SP_PROT_SSL3_CLIENT   0x00000020
#define SP_PROT_TLS1_0_SERVER 0x00000040
#define SP_PROT_TLS1_0_CLIENT 0x00000080
#define SP_PROT_TLS1_1_SERVER 0x00000100
#define SP_PROT_TLS1_1_CLIENT 0x00000200
#define SP_PROT_TLS1_2_SERVER 0x00000400
#define SP_PROT_TLS1_2_CLIENT 0x00000800

typedef struct _SecPkgContext_ConnectionInfo
{
    DWORD dwProtocol;
    ULONG aiCipher;
    DWORD dwCipherStrength;
    ULONG aiHash;
    DWORD dwHashStrength;
    ULONG aiExch;
    DWORD dwExchStrength;
} SecPkgContext_ConnectionInfo;

/* Certificate store: the backend only ever adds one DER blob and asks for the
 * context back, so that is all the harness models. */
typedef void *HCERTSTORE;
typedef const struct _CERT_CONTEXT *PCCERT_CONTEXT;
#define X509_ASN_ENCODING               0x00000001
#define CERT_STORE_ADD_REPLACE_EXISTING 3

BOOL WINAPI CertAddEncodedCertificateToStore(HCERTSTORE, DWORD, const BYTE *, DWORD,
                                             DWORD, PCCERT_CONTEXT *);
BOOL  WINAPI CertFreeCertificateContext(PCCERT_CONTEXT);
DWORD WINAPI GetLastError(void);

HANDLE WINAPI GetProcessHeap(void);
PVOID  WINAPI HeapAlloc(HANDLE, DWORD, SIZE_T);
BOOL   WINAPI HeapFree(HANDLE, DWORD, PVOID);

#include <limits.h>
#include <wine/debug.h>

/* Referenced by the sliced contract but never dereferenced by the backend, so an
 * incomplete type is the honest model. */
typedef struct _SecBufferDesc SecBufferDesc;
struct schan_context;

/* The backend contract, sliced out of the real schannel_priv.h by build.rs. */
#include "schan_backend_contract.h"

#endif /* SCHAN_HARNESS_PRECOMP_H */
