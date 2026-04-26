/*
 * Stubs for ole32/oleaut32/wininet functions that rpcrt4 delay-imports on
 * x86/amd64 but cannot delay-import on ARM64 (llvm-dlltool limitation).
 * These provide COM/OLE marshaling, OLE automation, and HTTP transport
 * for RPC — not needed during LPC-based boot.
 */

#ifdef _M_ARM64

#include <windef.h>
#include <winbase.h>

/* ole32 stubs — COM marshaling for DCOM transport */
LONG __stdcall CoGetClassObject(void *rclsid, DWORD dwClsContext, void *pServerInfo,
                                void *riid, void **ppv)
{
    if (ppv) *ppv = NULL;
    return (LONG)0x80004001L; /* E_NOTIMPL */
}

LONG __stdcall CoGetMarshalSizeMax(ULONG *pulSize, void *riid, void *pUnk,
                                   DWORD dwDestContext, void *pvDestContext,
                                   DWORD mshlflags)
{
    (void)riid; (void)pUnk; (void)dwDestContext; (void)pvDestContext; (void)mshlflags;
    if (pulSize) *pulSize = 0;
    return (LONG)0x80004001L;
}

LONG __stdcall CoGetPSClsid(void *riid, void *pClsid)
{
    (void)riid; (void)pClsid;
    return (LONG)0x80004001L;
}

LONG __stdcall CoMarshalInterface(void *pStm, void *riid, void *pUnk,
                                  DWORD dwDestContext, void *pvDestContext,
                                  DWORD mshlflags)
{
    (void)pStm; (void)riid; (void)pUnk; (void)dwDestContext; (void)pvDestContext; (void)mshlflags;
    return (LONG)0x80004001L;
}

void * __stdcall CoTaskMemAlloc(SIZE_T cb)
{
    return HeapAlloc(GetProcessHeap(), 0, cb);
}

void __stdcall CoTaskMemFree(void *pv)
{
    if (pv) HeapFree(GetProcessHeap(), 0, pv);
}

LONG __stdcall CoUnmarshalInterface(void *pStm, void *riid, void **ppv)
{
    (void)pStm; (void)riid;
    if (ppv) *ppv = NULL;
    return (LONG)0x80004001L;
}

/* oleaut32 stubs — OLE automation marshaling (BSTR, SAFEARRAY, VARIANT) */
ULONG __stdcall BSTR_UserSize(ULONG *pFlags, ULONG StartingSize, void *pBstr)
{
    (void)pFlags; (void)pBstr;
    return StartingSize;
}

unsigned char * __stdcall BSTR_UserMarshal(ULONG *pFlags, unsigned char *pBuffer, void *pBstr)
{
    (void)pFlags; (void)pBstr;
    return pBuffer;
}

unsigned char * __stdcall BSTR_UserUnmarshal(ULONG *pFlags, unsigned char *pBuffer, void *pBstr)
{
    (void)pFlags; (void)pBstr;
    return pBuffer;
}

void __stdcall BSTR_UserFree(ULONG *pFlags, void *pBstr)
{
    (void)pFlags; (void)pBstr;
}

ULONG __stdcall LPSAFEARRAY_UserSize(ULONG *pFlags, ULONG StartingSize, void *ppSA)
{
    (void)pFlags; (void)ppSA;
    return StartingSize;
}

unsigned char * __stdcall LPSAFEARRAY_UserMarshal(ULONG *pFlags, unsigned char *pBuffer, void *ppSA)
{
    (void)pFlags; (void)ppSA;
    return pBuffer;
}

unsigned char * __stdcall LPSAFEARRAY_UserUnmarshal(ULONG *pFlags, unsigned char *pBuffer, void *ppSA)
{
    (void)pFlags; (void)ppSA;
    return pBuffer;
}

void __stdcall LPSAFEARRAY_UserFree(ULONG *pFlags, void *ppSA)
{
    (void)pFlags; (void)ppSA;
}

ULONG __stdcall VARIANT_UserSize(ULONG *pFlags, ULONG StartingSize, void *pVar)
{
    (void)pFlags; (void)pVar;
    return StartingSize;
}

unsigned char * __stdcall VARIANT_UserMarshal(ULONG *pFlags, unsigned char *pBuffer, void *pVar)
{
    (void)pFlags; (void)pVar;
    return pBuffer;
}

unsigned char * __stdcall VARIANT_UserUnmarshal(ULONG *pFlags, unsigned char *pBuffer, void *pVar)
{
    (void)pFlags; (void)pVar;
    return pBuffer;
}

void __stdcall VARIANT_UserFree(ULONG *pFlags, void *pVar)
{
    (void)pFlags; (void)pVar;
}

/* iphlpapi stubs — network adapter info for RPC transport */
DWORD __stdcall GetAdaptersInfo(void *AdapterInfo, ULONG *SizePointer)
{
    (void)AdapterInfo;
    if (SizePointer) *SizePointer = 0;
    return 111; /* ERROR_BUFFER_OVERFLOW — no adapters */
}

/* secur32 stubs — SSPI security for authenticated RPC */
typedef long SECURITY_STATUS;
#ifndef SEC_E_INTERNAL_ERROR
#define SEC_E_INTERNAL_ERROR ((SECURITY_STATUS)0x80090304L)
#endif
#ifndef SEC_E_UNSUPPORTED_FUNCTION
#define SEC_E_UNSUPPORTED_FUNCTION ((SECURITY_STATUS)0x80090302L)
#endif

SECURITY_STATUS __stdcall AcceptSecurityContext(void *phCredential, void *phContext,
    void *pInput, ULONG fContextReq, ULONG TargetDataRep,
    void *phNewContext, void *pOutput, ULONG *pfContextAttr, void *ptsExpiry)
{
    (void)phCredential; (void)phContext; (void)pInput; (void)fContextReq;
    (void)TargetDataRep; (void)phNewContext; (void)pOutput; (void)pfContextAttr; (void)ptsExpiry;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall AcquireCredentialsHandleA(void *pszPrincipal, void *pszPackage,
    ULONG fCredentialUse, void *pvLogonId, void *pAuthData,
    void *pGetKeyFn, void *pvGetKeyArgument, void *phCredential, void *ptsExpiry)
{
    (void)pszPrincipal; (void)pszPackage; (void)fCredentialUse; (void)pvLogonId;
    (void)pAuthData; (void)pGetKeyFn; (void)pvGetKeyArgument; (void)phCredential; (void)ptsExpiry;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall AcquireCredentialsHandleW(void *pszPrincipal, void *pszPackage,
    ULONG fCredentialUse, void *pvLogonId, void *pAuthData,
    void *pGetKeyFn, void *pvGetKeyArgument, void *phCredential, void *ptsExpiry)
{
    (void)pszPrincipal; (void)pszPackage; (void)fCredentialUse; (void)pvLogonId;
    (void)pAuthData; (void)pGetKeyFn; (void)pvGetKeyArgument; (void)phCredential; (void)ptsExpiry;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall CompleteAuthToken(void *phContext, void *pToken)
{
    (void)phContext; (void)pToken;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall DecryptMessage(void *phContext, void *pMessage,
    ULONG MessageSeqNo, ULONG *pfQOP)
{
    (void)phContext; (void)pMessage; (void)MessageSeqNo; (void)pfQOP;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall DeleteSecurityContext(void *phContext)
{
    (void)phContext;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall EncryptMessage(void *phContext, ULONG fQOP,
    void *pMessage, ULONG MessageSeqNo)
{
    (void)phContext; (void)fQOP; (void)pMessage; (void)MessageSeqNo;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall EnumerateSecurityPackagesA(ULONG *pcPackages, void **ppPackageInfo)
{
    (void)pcPackages; (void)ppPackageInfo;
    if (pcPackages) *pcPackages = 0;
    if (ppPackageInfo) *ppPackageInfo = NULL;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall EnumerateSecurityPackagesW(ULONG *pcPackages, void **ppPackageInfo)
{
    (void)pcPackages; (void)ppPackageInfo;
    if (pcPackages) *pcPackages = 0;
    if (ppPackageInfo) *ppPackageInfo = NULL;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall FreeContextBuffer(void *pvContextBuffer)
{
    (void)pvContextBuffer;
    return 0; /* SEC_E_OK */
}

SECURITY_STATUS __stdcall FreeCredentialsHandle(void *phCredential)
{
    (void)phCredential;
    return 0;
}

BOOL __stdcall GetUserNameExW(DWORD NameFormat, void *lpNameBuffer, ULONG *nSize)
{
    (void)NameFormat; (void)lpNameBuffer;
    if (nSize) *nSize = 0;
    SetLastError(1355); /* ERROR_NO_SUCH_DOMAIN */
    return FALSE;
}

SECURITY_STATUS __stdcall ImpersonateSecurityContext(void *phContext)
{
    (void)phContext;
    return SEC_E_UNSUPPORTED_FUNCTION;
}

SECURITY_STATUS __stdcall InitializeSecurityContextW(void *phCredential, void *phContext,
    void *pszTargetName, ULONG fContextReq, ULONG Reserved1, ULONG TargetDataRep,
    void *pInput, ULONG Reserved2, void *phNewContext, void *pOutput,
    ULONG *pfContextAttr, void *ptsExpiry)
{
    (void)phCredential; (void)phContext; (void)pszTargetName; (void)fContextReq;
    (void)Reserved1; (void)TargetDataRep; (void)pInput; (void)Reserved2;
    (void)phNewContext; (void)pOutput; (void)pfContextAttr; (void)ptsExpiry;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall MakeSignature(void *phContext, ULONG fQOP,
    void *pMessage, ULONG MessageSeqNo)
{
    (void)phContext; (void)fQOP; (void)pMessage; (void)MessageSeqNo;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall QueryContextAttributesA(void *phContext, ULONG ulAttribute, void *pBuffer)
{
    (void)phContext; (void)ulAttribute; (void)pBuffer;
    return SEC_E_UNSUPPORTED_FUNCTION;
}

SECURITY_STATUS __stdcall QuerySecurityPackageInfoW(void *pszPackageName, void **ppPackageInfo)
{
    (void)pszPackageName;
    if (ppPackageInfo) *ppPackageInfo = NULL;
    return SEC_E_INTERNAL_ERROR;
}

SECURITY_STATUS __stdcall RevertSecurityContext(void *phContext)
{
    (void)phContext;
    return SEC_E_UNSUPPORTED_FUNCTION;
}

SECURITY_STATUS __stdcall VerifySignature(void *phContext, void *pMessage,
    ULONG MessageSeqNo, ULONG *pfQOP)
{
    (void)phContext; (void)pMessage; (void)MessageSeqNo; (void)pfQOP;
    return SEC_E_INTERNAL_ERROR;
}

/* wininet stubs — HTTP transport for RPC */
void * __stdcall InternetOpenW(const void *lpszAgent, DWORD dwAccessType,
                               const void *lpszProxy, const void *lpszProxyBypass,
                               DWORD dwFlags)
{
    (void)lpszAgent; (void)dwAccessType; (void)lpszProxy; (void)lpszProxyBypass; (void)dwFlags;
    return NULL;
}

void * __stdcall InternetConnectW(void *hInternet, const void *lpszServerName,
                                  unsigned short nServerPort, const void *lpszUserName,
                                  const void *lpszPassword, DWORD dwService,
                                  DWORD dwFlags, ULONG_PTR dwContext)
{
    (void)hInternet; (void)lpszServerName; (void)nServerPort;
    (void)lpszUserName; (void)lpszPassword; (void)dwService;
    (void)dwFlags; (void)dwContext;
    return NULL;
}

BOOL __stdcall InternetCloseHandle(void *hInternet)
{
    (void)hInternet;
    return TRUE;
}

void * __stdcall HttpOpenRequestW(void *hConnect, const void *lpszVerb,
                                  const void *lpszObjectName, const void *lpszVersion,
                                  const void *lpszReferrer, const void **lplpszAcceptTypes,
                                  DWORD dwFlags, ULONG_PTR dwContext)
{
    (void)hConnect; (void)lpszVerb; (void)lpszObjectName; (void)lpszVersion;
    (void)lpszReferrer; (void)lplpszAcceptTypes; (void)dwFlags; (void)dwContext;
    return NULL;
}

BOOL __stdcall HttpSendRequestW(void *hRequest, const void *lpszHeaders,
                                DWORD dwHeadersLength, void *lpOptional,
                                DWORD dwOptionalLength)
{
    (void)hRequest; (void)lpszHeaders; (void)dwHeadersLength;
    (void)lpOptional; (void)dwOptionalLength;
    return FALSE;
}

BOOL __stdcall HttpSendRequestExW(void *hRequest, void *lpBuffersIn,
                                  void *lpBuffersOut, DWORD dwFlags,
                                  ULONG_PTR dwContext)
{
    (void)hRequest; (void)lpBuffersIn; (void)lpBuffersOut;
    (void)dwFlags; (void)dwContext;
    return FALSE;
}

BOOL __stdcall HttpAddRequestHeadersW(void *hRequest, const void *lpszHeaders,
                                      DWORD dwHeadersLength, DWORD dwModifiers)
{
    (void)hRequest; (void)lpszHeaders; (void)dwHeadersLength; (void)dwModifiers;
    return FALSE;
}

BOOL __stdcall HttpQueryInfoW(void *hRequest, DWORD dwInfoLevel, void *lpBuffer,
                              DWORD *lpdwBufferLength, DWORD *lpdwIndex)
{
    (void)hRequest; (void)dwInfoLevel; (void)lpBuffer;
    (void)lpdwBufferLength; (void)lpdwIndex;
    return FALSE;
}

BOOL __stdcall InternetQueryDataAvailable(void *hFile, DWORD *lpdwNumberOfBytesAvailable,
                                          DWORD dwFlags, ULONG_PTR dwContext)
{
    (void)hFile; (void)dwFlags; (void)dwContext;
    if (lpdwNumberOfBytesAvailable) *lpdwNumberOfBytesAvailable = 0;
    return FALSE;
}

BOOL __stdcall InternetReadFileExW(void *hFile, void *lpBuffersOut,
                                   DWORD dwFlags, ULONG_PTR dwContext)
{
    (void)hFile; (void)lpBuffersOut; (void)dwFlags; (void)dwContext;
    return FALSE;
}

BOOL __stdcall InternetWriteFile(void *hFile, const void *lpBuffer,
                                 DWORD dwNumberOfBytesToWrite,
                                 DWORD *lpdwNumberOfBytesWritten)
{
    (void)hFile; (void)lpBuffer; (void)dwNumberOfBytesToWrite;
    if (lpdwNumberOfBytesWritten) *lpdwNumberOfBytesWritten = 0;
    return FALSE;
}

BOOL __stdcall InternetSetCookieW(const void *lpszUrl, const void *lpszCookieName,
                                  const void *lpszCookieData)
{
    (void)lpszUrl; (void)lpszCookieName; (void)lpszCookieData;
    return FALSE;
}

void * __stdcall InternetSetStatusCallbackW(void *hInternet, void *lpfnInternetCallback)
{
    (void)hInternet; (void)lpfnInternetCallback;
    return NULL;
}

BOOL __stdcall InternetCreateUrlW(void *lpUrlComponents, DWORD dwFlags,
                                  void *lpszUrl, DWORD *lpdwUrlLength)
{
    (void)lpUrlComponents; (void)dwFlags; (void)lpszUrl;
    if (lpdwUrlLength) *lpdwUrlLength = 0;
    return FALSE;
}

#endif /* _M_ARM64 */
