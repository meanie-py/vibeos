/* regsd — print the DACL (as SDDL) of the registry keys kmtest CmSecurity checks, so the
 * guest's hive-root security diffs against winref (Windows 11). Written when the version
 * flip sent CmSecurity down its Win8 branch, which Windows 11 itself fails (2026-10-19).
 *   regsd.exe        DACLs as SDDL
 *   regsd.exe -hex   owner, group and DACL as SDDL, then the self-relative bytes (for
 *                    sdk/tools/mkhive/registry.c's hive root blobs)
 */
#include <windows.h>
#include <winternl.h>
#include <sddl.h>
#include <stdio.h>

NTSTATUS NTAPI NtOpenKey(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES);
NTSTATUS NTAPI NtQuerySecurityObject(HANDLE, SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, ULONG, PULONG);

static const wchar_t *keys[] = {
    L"\\REGISTRY", L"\\REGISTRY\\MACHINE", L"\\REGISTRY\\MACHINE\\HARDWARE",
    L"\\REGISTRY\\MACHINE\\SAM", L"\\REGISTRY\\MACHINE\\SECURITY", L"\\REGISTRY\\MACHINE\\SOFTWARE",
    L"\\REGISTRY\\MACHINE\\SYSTEM", L"\\REGISTRY\\USER", L"\\REGISTRY\\USER\\.DEFAULT",
    L"\\REGISTRY\\USER\\S-1-5-18", L"\\REGISTRY\\USER\\S-1-5-19", L"\\REGISTRY\\USER\\S-1-5-20",
};

int wmain(int argc, wchar_t **argv)
{
    int hex = argc > 1 && !lstrcmpW(argv[1], L"-hex");
    SECURITY_INFORMATION si = hex ? OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION
                                  : DACL_SECURITY_INFORMATION;
    for (unsigned i = 0; i < sizeof(keys) / sizeof(keys[0]); i++) {
        UNICODE_STRING name;
        OBJECT_ATTRIBUTES oa;
        HANDLE h;
        BYTE sd[4096];
        ULONG n = 0;
        LPWSTR sddl = NULL;
        RtlInitUnicodeString(&name, keys[i]);
        InitializeObjectAttributes(&oa, &name, OBJ_CASE_INSENSITIVE, NULL, NULL);
        NTSTATUS st = NtOpenKey(&h, READ_CONTROL, &oa);
        if (st < 0) {
            printf("%ls: open 0x%08lx\n", keys[i], (ULONG)st);
            continue;
        }
        st = NtQuerySecurityObject(h, si, sd, sizeof(sd), &n);
        if (st >= 0 && ConvertSecurityDescriptorToStringSecurityDescriptorW(sd, SDDL_REVISION_1,
                                                                            si, &sddl, NULL)) {
            printf("%ls: %ls\n", keys[i], sddl);
            if (hex) {
                printf("%ls: hex", keys[i]);
                for (ULONG b = 0; b < n; b++)
                    printf(" %02x", sd[b]);
                printf("\n");
            }
        }
        else
            printf("%ls: query 0x%08lx err=%lu\n", keys[i], (ULONG)st, GetLastError());
        LocalFree(sddl);
        NtClose(h);
    }
    printf("regsd done\n");
    return 0;
}
