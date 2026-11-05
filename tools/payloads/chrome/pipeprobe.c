/* pipeprobe: Chrome's crashpad dies at its first CreateNamedPipe with ERROR_PRIVILEGE_NOT_HELD.
 * This makes the same call (crashpad_client_win.cc CreateNamedPipeInstance: a self-relative
 * descriptor whose only part is a SACL holding one mandatory-label ACE, untrusted level, mask 0)
 * and variants of it, through kernel32 and straight through ntdll, so the failing layer and
 * its NTSTATUS are measured rather than guessed. Windows 10/11 accept every case here. */
#include <windows.h>
#include <winternl.h>
#include <stdio.h>

typedef NTSTATUS (NTAPI *NtCreateNamedPipeFile_t)(PHANDLE, ULONG, POBJECT_ATTRIBUTES, PIO_STATUS_BLOCK,
    ULONG, ULONG, ULONG, ULONG, ULONG, ULONG, ULONG, ULONG, ULONG, PLARGE_INTEGER);

/* Built by hand, the way crashpad builds its own (no sddl parse in the way). */
static void build_label_sd(BYTE *buf, DWORD level_rid, DWORD mask)
{
    SECURITY_DESCRIPTOR_RELATIVE *sd = (void *)buf;
    ACL *acl = (void *)(buf + sizeof(*sd));
    BYTE *ace = (BYTE *)(acl + 1);
    SID *sid;
    memset(buf, 0, 64);
    sd->Revision = 1;
    sd->Control = SE_SELF_RELATIVE | SE_SACL_PRESENT;
    sd->Sacl = sizeof(*sd);
    acl->AclRevision = ACL_REVISION;
    acl->AclSize = sizeof(ACL) + 8 + 12;
    acl->AceCount = 1;
    ace[0] = 0x11;                 /* SYSTEM_MANDATORY_LABEL_ACE_TYPE */
    ace[1] = 0;
    *(WORD *)(ace + 2) = 8 + 12;   /* header + mask + SID(1 subauth) */
    *(DWORD *)(ace + 4) = mask;
    sid = (SID *)(ace + 8);
    sid->Revision = 1;
    sid->SubAuthorityCount = 1;
    sid->IdentifierAuthority.Value[5] = 16; /* SECURITY_MANDATORY_LABEL_AUTHORITY */
    sid->SubAuthority[0] = level_rid;
}

static void try_k32(const char *what, SECURITY_DESCRIPTOR *sd, DWORD open_mode)
{
    SECURITY_ATTRIBUTES sa = { sizeof(sa), sd, TRUE };
    wchar_t name[64];
    HANDLE h;
    static int n;
    _snwprintf(name, 64, L"\\\\.\\pipe\\pipeprobe-%d-%lu", ++n, GetCurrentProcessId());
    h = CreateNamedPipeW(name, open_mode, PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                         2, 512, 512, 0, sd ? &sa : NULL);
    printf("[PIPEPROBE] k32 %-34s -> %s (err %lu)\n", what, h == INVALID_HANDLE_VALUE ? "FAIL" : "ok",
           h == INVALID_HANDLE_VALUE ? GetLastError() : 0);
    if (h != INVALID_HANDLE_VALUE) CloseHandle(h);
}

static void try_nt(const char *what, SECURITY_DESCRIPTOR *sd)
{
    NtCreateNamedPipeFile_t f = (void *)GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtCreateNamedPipeFile");
    UNICODE_STRING us;
    OBJECT_ATTRIBUTES oa;
    IO_STATUS_BLOCK iosb;
    LARGE_INTEGER to = { .QuadPart = -500000 };
    HANDLE h = NULL;
    wchar_t name[64];
    static int n;
    NTSTATUS st;
    _snwprintf(name, 64, L"\\??\\pipe\\pipeprobe-nt-%d-%lu", ++n, GetCurrentProcessId());
    us.Buffer = name; us.Length = (USHORT)(wcslen(name) * 2); us.MaximumLength = us.Length + 2;
    InitializeObjectAttributes(&oa, &us, OBJ_CASE_INSENSITIVE, NULL, sd);
    st = f(&h, GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE, &oa, &iosb, FILE_SHARE_READ | FILE_SHARE_WRITE,
           FILE_CREATE, FILE_SYNCHRONOUS_IO_NONALERT, 1, 1, 0, 2, 512, 512, &to);
    printf("[PIPEPROBE] nt  %-34s -> 0x%08lx\n", what, (unsigned long)st);
    if (h) CloseHandle(h);
}

int main(void)
{
    BYTE untrusted[64], low[64], low_nw[64];
    DWORD v = GetVersion();
    build_label_sd(untrusted, 0x0000, 0);
    build_label_sd(low, 0x1000, 0);
    build_label_sd(low_nw, 0x1000, 1 /* SYSTEM_MANDATORY_LABEL_NO_WRITE_UP */);
    printf("[PIPEPROBE] GetVersion %lu.%lu build %lu\n", v & 0xff, (v >> 8) & 0xff, v >> 16);
    try_k32("no security attributes", NULL, PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE);
    try_k32("label untrusted mask 0 (crashpad)", (void *)untrusted, PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE);
    try_k32("label low mask 0", (void *)low, PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE);
    try_k32("label low NO_WRITE_UP", (void *)low_nw, PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE);
    try_k32("label untrusted, not first", (void *)untrusted, PIPE_ACCESS_DUPLEX);
    try_nt("no descriptor", NULL);
    try_nt("label untrusted mask 0 (crashpad)", (void *)untrusted);
    printf("[PIPEPROBE] done\n");
    return 0;
}
