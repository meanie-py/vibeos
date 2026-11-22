/* tokprobe: the restricted token Chrome's sandbox builds for a locked-down child.
 * The sandboxed browser's GPU process launch failed 3 times with sandbox ResultCode 49,
 * SBOX_ERROR_CANNOT_CREATE_RESTRICTED_TOKEN (sandbox quest, step 3). This makes the same
 * calls, one line per fact: which step fails, and with what. On Windows every step succeeds. */
#include <windows.h>
#include <sddl.h>
#include <stdio.h>

#ifndef SANDBOX_INERT
#define SANDBOX_INERT 0x2
#endif
#ifndef WRITE_RESTRICTED
#define WRITE_RESTRICTED 0x8
#endif

static int fails;

static void report(const char *what, BOOL ok)
{
    DWORD e = ok ? 0 : GetLastError();
    printf("[TOKPROBE] %-58s %s (err=%lu)\n", what, ok ? "ok" : "FAILED", e);
    if (!ok)
        fails++;
}

static void *query(HANDLE tok, TOKEN_INFORMATION_CLASS c, const char *what)
{
    DWORD need = 0;
    GetTokenInformation(tok, c, NULL, 0, &need);
    void *buf = need ? HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, need) : NULL;
    BOOL ok = buf && GetTokenInformation(tok, c, buf, need, &need);
    report(what, ok);
    return ok ? buf : NULL;
}

int main(void)
{
    HANDLE tok = NULL;
    report("OpenProcessToken(ALL_ACCESS)", OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &tok));
    if (!tok) {
        printf("[TOKPROBE] done VERDICT=FAIL\n");
        return 1;
    }
    TOKEN_GROUPS *groups = query(tok, TokenGroups, "GetTokenInformation(TokenGroups)");
    TOKEN_USER *user = query(tok, TokenUser, "GetTokenInformation(TokenUser)");
    query(tok, TokenIntegrityLevel, "GetTokenInformation(TokenIntegrityLevel)");
    query(tok, TokenLogonSid, "GetTokenInformation(TokenLogonSid)");
    query(tok, TokenDefaultDacl, "GetTokenInformation(TokenDefaultDacl)");

    HANDLE r = NULL;
    report("CreateRestrictedToken(0, nothing)", CreateRestrictedToken(tok, 0, 0, NULL, 0, NULL, 0, NULL, &r));
    if (r) CloseHandle(r), r = NULL;
    report("CreateRestrictedToken(DISABLE_MAX_PRIVILEGE)",
           CreateRestrictedToken(tok, DISABLE_MAX_PRIVILEGE, 0, NULL, 0, NULL, 0, NULL, &r));
    if (r) CloseHandle(r), r = NULL;
    report("CreateRestrictedToken(SANDBOX_INERT)", CreateRestrictedToken(tok, SANDBOX_INERT, 0, NULL, 0, NULL, 0, NULL, &r));
    if (r) CloseHandle(r), r = NULL;

    /* Chrome's USER_LIMITED shape: every group deny-only, all privileges gone, restricting
     * SIDs = Everyone + Users + the user + the logon SID. */
    SID_AND_ATTRIBUTES deny[64];
    DWORD ndeny = 0;
    if (groups)
        for (DWORD i = 0; i < groups->GroupCount && ndeny < 64; i++)
            if (!(groups->Groups[i].Attributes & SE_GROUP_INTEGRITY)) {
                deny[ndeny].Sid = groups->Groups[i].Sid;
                deny[ndeny++].Attributes = 0;
            }
    PSID everyone = NULL, users = NULL;
    ConvertStringSidToSidA("S-1-1-0", &everyone);
    ConvertStringSidToSidA("S-1-5-32-545", &users);
    SID_AND_ATTRIBUTES restrict_[4];
    DWORD nr = 0;
    restrict_[nr].Sid = everyone, restrict_[nr++].Attributes = 0;
    restrict_[nr].Sid = users, restrict_[nr++].Attributes = 0;
    if (user)
        restrict_[nr].Sid = user->User.Sid, restrict_[nr++].Attributes = 0;
    printf("[TOKPROBE] shape: %lu deny-only groups, %lu restricting SIDs\n", ndeny, nr);
    report("CreateRestrictedToken(deny-only groups)",
           CreateRestrictedToken(tok, DISABLE_MAX_PRIVILEGE, ndeny, deny, 0, NULL, 0, NULL, &r));
    if (r) CloseHandle(r), r = NULL;
    report("CreateRestrictedToken(restricting SIDs)",
           CreateRestrictedToken(tok, DISABLE_MAX_PRIVILEGE, 0, NULL, 0, NULL, nr, restrict_, &r));
    if (r) CloseHandle(r), r = NULL;
    report("CreateRestrictedToken(WRITE_RESTRICTED + restricting SIDs)",
           CreateRestrictedToken(tok, DISABLE_MAX_PRIVILEGE | WRITE_RESTRICTED, 0, NULL, 0, NULL, nr, restrict_, &r));
    if (r) CloseHandle(r), r = NULL;
    BOOL ok = CreateRestrictedToken(tok, DISABLE_MAX_PRIVILEGE, ndeny, deny, 0, NULL, nr, restrict_, &r);
    report("CreateRestrictedToken(full USER_LIMITED shape)", ok);

    if (r) {
        /* Then the sandbox sets a default DACL and Low integrity on it. */
        TOKEN_DEFAULT_DACL *dacl = query(r, TokenDefaultDacl, "restricted: GetTokenInformation(TokenDefaultDacl)");
        if (dacl)
            report("restricted: SetTokenInformation(TokenDefaultDacl)",
                   SetTokenInformation(r, TokenDefaultDacl, dacl, sizeof(*dacl)));
        PSID low = NULL;
        ConvertStringSidToSidA("S-1-16-4096", &low);
        TOKEN_MANDATORY_LABEL tml = { { low, SE_GROUP_INTEGRITY } };
        report("restricted: SetTokenInformation(TokenIntegrityLevel=Low)",
               SetTokenInformation(r, TokenIntegrityLevel, &tml, sizeof tml + GetLengthSid(low)));
        HANDLE imp = NULL;
        report("restricted: DuplicateTokenEx(SecurityImpersonation)",
               DuplicateTokenEx(r, TOKEN_ALL_ACCESS, NULL, SecurityImpersonation, TokenImpersonation, &imp));
        if (imp)
            CloseHandle(imp);
        CloseHandle(r);
    }
    printf("[TOKPROBE] done VERDICT=%s\n", fails ? "FAIL" : "PASS");
    return fails ? 1 : 0;
}
