/*
 * Stubs for secur32 functions that advapi32 delay-imports on x86/amd64
 * but cannot delay-import on ARM64 (llvm-dlltool limitation).
 * These are LSA logon functions — not needed during boot.
 */

#ifdef _M_ARM64

#include <windef.h>
#include <winbase.h>

typedef LONG NTSTATUS;
#define STATUS_NOT_IMPLEMENTED ((NTSTATUS)0xC0000002L)

NTSTATUS __stdcall LsaDeregisterLogonProcess(HANDLE LsaHandle)
{
    (void)LsaHandle;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS __stdcall LsaFreeReturnBuffer(PVOID Buffer)
{
    (void)Buffer;
    return 0; /* STATUS_SUCCESS */
}

NTSTATUS __stdcall LsaLogonUser(HANDLE LsaHandle, void *OriginName,
    DWORD LogonType, ULONG AuthenticationPackage,
    PVOID AuthenticationInformation, ULONG AuthenticationInformationLength,
    void *LocalGroups, void *SourceContext,
    PVOID *ProfileBuffer, ULONG *ProfileBufferLength,
    void *LogonId, HANDLE *Token, void *Quotas, NTSTATUS *SubStatus)
{
    (void)LsaHandle; (void)OriginName; (void)LogonType; (void)AuthenticationPackage;
    (void)AuthenticationInformation; (void)AuthenticationInformationLength;
    (void)LocalGroups; (void)SourceContext;
    if (ProfileBuffer) *ProfileBuffer = NULL;
    if (ProfileBufferLength) *ProfileBufferLength = 0;
    if (Token) *Token = NULL;
    if (SubStatus) *SubStatus = STATUS_NOT_IMPLEMENTED;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS __stdcall LsaLookupAuthenticationPackage(HANDLE LsaHandle,
    void *PackageName, ULONG *AuthenticationPackage)
{
    (void)LsaHandle; (void)PackageName;
    if (AuthenticationPackage) *AuthenticationPackage = 0;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS __stdcall LsaRegisterLogonProcess(void *LogonProcessName,
    HANDLE *LsaHandle, void *SecurityMode)
{
    (void)LogonProcessName;
    if (LsaHandle) *LsaHandle = NULL;
    if (SecurityMode) *(ULONG *)SecurityMode = 0;
    return STATUS_NOT_IMPLEMENTED;
}

#endif /* _M_ARM64 */
