/* Black-box probe (i686 / WOW64): header after RtlInterlockedPushListSList (fastcall) vs ...Ex (stdcall). */
#include <windows.h>
#include <stdio.h>
typedef PSLIST_ENTRY (__fastcall *FastFn)(PSLIST_HEADER, PSLIST_ENTRY, PSLIST_ENTRY, ULONG);
typedef PSLIST_ENTRY (WINAPI *ExFn)(PSLIST_HEADER, PSLIST_ENTRY, PSLIST_ENTRY, ULONG);
int main(void)
{
    HMODULE nt = GetModuleHandleW(L"ntdll.dll");
    FastFn f = (FastFn)GetProcAddress(nt, "RtlInterlockedPushListSList");
    ExFn x = (ExFn)GetProcAddress(nt, "RtlInterlockedPushListSListEx");
    static SLIST_HEADER h __attribute__((aligned(8)));
    static SLIST_ENTRY e[8] __attribute__((aligned(8)));
    for (int which = 0; which < 3; which++) {
        InitializeSListHead(&h);
        for (int k = 0; k < 3; k++) {
            e[k].Next = NULL;
            if (which == 0) f(&h, &e[k], &e[k], 1);
            else if (which == 1) x(&h, &e[k], &e[k], 1);
            else InterlockedPushEntrySList(&h, &e[k]);
            printf("which=%d push%d header=%016llx\n", which, k, (unsigned long long)h.Alignment);
        }
        InterlockedPopEntrySList(&h);
        printf("which=%d pop header=%016llx\n", which, (unsigned long long)h.Alignment);
    }
    return 0;
}
