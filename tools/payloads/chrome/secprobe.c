/* secprobe: what Chrome's base::PlatformSharedMemoryRegion asks of a section handle. Its
 * browser CHECKed on a "Section" handle's duplicate (a DuplicateHandle to FILE_MAP_READ, the
 * read-only copy of a writable region), on four threads. This walks the same steps on a
 * pagefile-backed section and prints each answer; Windows' are in secprobe.win11.txt. */
#include <windows.h>
#include <winternl.h>
#include <stdio.h>

typedef NTSTATUS (NTAPI *NtQueryObject_t)(HANDLE, ULONG, PVOID, ULONG, PULONG);
typedef NTSTATUS (NTAPI *NtQuerySection_t)(HANDLE, ULONG, PVOID, SIZE_T, PSIZE_T);
typedef struct { ULONG Attributes; ACCESS_MASK GrantedAccess; ULONG HandleCount, PointerCount; ULONG pad[10]; } OBI;
typedef struct { PVOID Base; ULONG Attributes; LARGE_INTEGER Size; } SBI;

static NtQueryObject_t qo; static NtQuerySection_t qs;

static void show(const char *what, HANDLE h)
{
    OBI obi; ULONG len = 0; BYTE tbuf[512]; SBI sbi; SIZE_T sl = 0; NTSTATUS st;
    memset(&obi, 0, sizeof(obi));
    st = qo(h, 0 /* ObjectBasicInformation */, &obi, sizeof(obi), &len);
    printf("[SECPROBE] %-22s basic st=0x%08lx granted=0x%08lx len=%lu\n", what, (unsigned long)st, obi.GrantedAccess, len);
    st = qo(h, 2 /* ObjectTypeInformation */, tbuf, sizeof(tbuf), &len);
    if (st >= 0) {
        UNICODE_STRING *n = (UNICODE_STRING *)tbuf;
        printf("[SECPROBE] %-22s type st=0x%08lx name=%.*ls\n", what, (unsigned long)st, n->Length / 2, n->Buffer);
    } else printf("[SECPROBE] %-22s type st=0x%08lx\n", what, (unsigned long)st);
    memset(&sbi, 0, sizeof(sbi));
    st = qs(h, 0 /* SectionBasicInformation */, &sbi, sizeof(sbi), &sl);
    printf("[SECPROBE] %-22s section st=0x%08lx attrs=0x%08lx size=%lld\n", what, (unsigned long)st, sbi.Attributes, sbi.Size.QuadPart);
}

int main(void)
{
    HANDLE p = GetCurrentProcess(), sec, ro = NULL, same = NULL; void *v;
    HMODULE nt = GetModuleHandleW(L"ntdll.dll");
    qo = (NtQueryObject_t)GetProcAddress(nt, "NtQueryObject");
    qs = (NtQuerySection_t)GetProcAddress(nt, "NtQuerySection");
    sec = CreateFileMappingW(INVALID_HANDLE_VALUE, NULL, PAGE_READWRITE, 0, 65536, NULL);
    printf("[SECPROBE] CreateFileMapping %s (%lu)\n", sec ? "ok" : "FAILED", sec ? 0 : GetLastError());
    if (!sec) return 1;
    show("writable", sec);
    if (DuplicateHandle(p, sec, p, &ro, FILE_MAP_READ | SECTION_QUERY, FALSE, 0)) show("dup read|query", ro);
    else printf("[SECPROBE] dup read|query FAILED (%lu)\n", GetLastError());
    if (DuplicateHandle(p, sec, p, &same, 0, FALSE, DUPLICATE_SAME_ACCESS)) show("dup same access", same);
    else printf("[SECPROBE] dup same access FAILED (%lu)\n", GetLastError());
    v = ro ? MapViewOfFile(ro, FILE_MAP_READ, 0, 0, 0) : NULL;
    printf("[SECPROBE] map read-only dup %s (%lu)\n", v ? "ok" : "FAILED", v ? 0 : GetLastError());
    v = ro ? MapViewOfFile(ro, FILE_MAP_WRITE, 0, 0, 0) : NULL;
    printf("[SECPROBE] map it for write %s (%lu)\n", v ? "ok (WRONG)" : "refused", v ? 0 : GetLastError());
    /* Chrome's own shape (base CreateFileMappingWithReducedPermissions): an EMPTY DACL, so
     * the handles are the only access there is, then the read-only check: asking a read-only
     * handle's duplicate for FILE_MAP_WRITE must fail, or the region is not read-only. */
    {
        SECURITY_DESCRIPTOR sd; ACL acl; SECURITY_ATTRIBUTES sa = { sizeof(sa), &sd, FALSE };
        HANDLE rw, r = NULL, w = NULL; SBI sbi; NTSTATUS st;
        InitializeSecurityDescriptor(&sd, SECURITY_DESCRIPTOR_REVISION);
        InitializeAcl(&acl, sizeof(acl), ACL_REVISION);
        SetSecurityDescriptorDacl(&sd, TRUE, &acl, FALSE);
        rw = CreateFileMappingW(INVALID_HANDLE_VALUE, &sa, PAGE_READWRITE, 0, 65536, NULL);
        printf("[SECPROBE] empty-DACL section %s (%lu)\n", rw ? "ok" : "FAILED", rw ? 0 : GetLastError());
        if (rw) {
            show("empty-DACL writable", rw);
            st = qs(rw, 0, &sbi, sizeof(sbi), NULL);
            printf("[SECPROBE] query, NULL ResultLength st=0x%08lx\n", (unsigned long)st);
            if (DuplicateHandle(p, rw, p, &r, FILE_MAP_READ | SECTION_QUERY, FALSE, 0)) show("empty-DACL read-only", r);
            else printf("[SECPROBE] empty-DACL read-only dup FAILED (%lu)\n", GetLastError());
            printf("[SECPROBE] upgrade writable to WRITE %s (%lu)\n",
                   DuplicateHandle(p, rw, p, &w, FILE_MAP_WRITE, FALSE, 0) ? "ok" : "refused", GetLastError());
            w = NULL;
            printf("[SECPROBE] upgrade read-only to WRITE %s (%lu)\n",
                   r && DuplicateHandle(p, r, p, &w, FILE_MAP_WRITE, FALSE, 0) ? "ok (NOT read-only)" : "refused", GetLastError());
        }
    }
    printf("[SECPROBE] done\n");
    return 0;
}
