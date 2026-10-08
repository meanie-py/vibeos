/* modrun probe: does msv1_0 check passwords? LogonUserW goes through lsass to the msv1_0
 * authentication package (msv1_0-rust). A no-op `goto done` there once let every failed
 * check fall through into session and token creation: an unknown user name crashed lsass
 * (found by scan_noop_goto + this probe, 2026-10-07).
 *
 * A passwordless account accepts ANY password (msv1_0 sam.c: "No password check!"), and the
 * livecd's Administrator has none -- so the probe makes its own user with a password. */
#include <windows.h>
#include <lm.h>
#include <stdio.h>

#define USER L"modrunprobe"
#define PASSWORD L"Correct-Horse-7"

static void try_logon(const char *what, const wchar_t *user, const wchar_t *password, DWORD type)
{
    HANDLE token = NULL;
    BOOL ok = LogonUserW(user, L".", password, type, LOGON32_PROVIDER_DEFAULT, &token);
    printf("logon %s type=%lu ok=%d err=%lu\n", what, type, ok, ok ? 0 : GetLastError());
    if (token)
        CloseHandle(token);
}

int main(void)
{
    USER_INFO_1 ui = { 0 };
    DWORD bad = 0;
    NET_API_STATUS st;

    NetUserDel(NULL, USER); /* a leftover from an aborted run */
    ui.usri1_name = USER;
    ui.usri1_password = PASSWORD;
    ui.usri1_priv = USER_PRIV_USER;
    ui.usri1_flags = UF_SCRIPT | UF_DONT_EXPIRE_PASSWD;
    st = NetUserAdd(NULL, 1, (LPBYTE)&ui, &bad);
    printf("useradd status=%lu\n", st);
    if (st == NERR_Success)
    {
        try_logon("wrong-password", USER, L"definitely-not-it", LOGON32_LOGON_INTERACTIVE);
        try_logon("wrong-password", USER, L"definitely-not-it", LOGON32_LOGON_BATCH);
        try_logon("empty-password", USER, L"", LOGON32_LOGON_INTERACTIVE);
        try_logon("right-password", USER, PASSWORD, LOGON32_LOGON_INTERACTIVE);
        printf("userdel status=%lu\n", NetUserDel(NULL, USER));
    }
    try_logon("unknown-user", L"NoSuchUser9", L"whatever", LOGON32_LOGON_INTERACTIVE);
    printf("probe done\n");
    fflush(stdout);
    return 0;
}
