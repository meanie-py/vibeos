/* Black-box probe: what MD5Final/MD4Final leave in the bit counter for unusual counters. */
#include <windows.h>
#include <stdio.h>
#include <string.h>
int main(void)
{
    HMODULE nt = GetModuleHandleW(L"ntdll.dll");
    void (WINAPI *Md5I)(void *) = (void *)GetProcAddress(nt, "MD5Init");
    void (WINAPI *Md5F)(void *) = (void *)GetProcAddress(nt, "MD5Final");
    void (WINAPI *Md4I)(void *) = (void *)GetProcAddress(nt, "MD4Init");
    void (WINAPI *Md4F)(void *) = (void *)GetProcAddress(nt, "MD4Final");
    unsigned hi[] = { 0, 1, 6, 0x10, 0x12345678, 0x6dd0c18e, 0xFFFFFFFF, 0x1FFFFFFF, 0x20000000 };
    unsigned lo[] = { 0x18, 0x1C0, 0x1F8, 0xFFFFFFF8, 0x36b7e800 - 0x200 };
    for (unsigned a = 0; a < sizeof hi / 4; a++)
        for (unsigned b = 0; b < sizeof lo / 4; b++) {
            unsigned c[26], d[26];
            Md5I(c); c[0] = lo[b]; c[1] = hi[a]; Md5F(c);
            Md4I(d); d[4] = lo[b]; d[5] = hi[a]; Md4F(d);
            printf("hi=%08x lo=%08x md5 i=%08x,%08x dig=%08x md4 i=%08x,%08x dig=%08x\n", hi[a], lo[b],
                   c[0], c[1], c[22], d[4], d[5], d[22]);
        }
    return 0;
}
