/* Black-box probe of how MD5/MD4/A_SHA Update advance a context's counters (garbage start values). */
#include <windows.h>
#include <stdio.h>
#include <string.h>
int main(void)
{
    HMODULE nt = GetModuleHandleW(L"ntdll.dll");
    void (WINAPI *Md5U)(void *, const void *, unsigned) = (void *)GetProcAddress(nt, "MD5Update");
    void (WINAPI *Md4U)(void *, const void *, unsigned) = (void *)GetProcAddress(nt, "MD4Update");
    void (WINAPI *ShaU)(void *, const void *, unsigned) = (void *)GetProcAddress(nt, "A_SHAUpdate");
    static unsigned char data[0x2000];
    unsigned lo[] = { 0, 7, 0x1F, 0xFFFFFFF8u, 0xFFFFFFFFu, 0xFFFFFE07u, 0x80000001u };
    unsigned lens[] = { 0, 1, 8, 63, 64, 0x1FFF };
    for (unsigned a = 0; a < sizeof lo / 4; a++)
        for (unsigned l = 0; l < sizeof lens / 4; l++) {
            unsigned c5[26] = {0}, c4[26] = {0}, cs[23] = {0};
            c5[0] = lo[a]; c5[1] = 0x10;
            c4[4] = lo[a]; c4[5] = 0x10;
            cs[16 + 5 + 1] = lo[a]; cs[16 + 5] = 0x10;
            Md5U(c5, data, lens[l]); Md4U(c4, data, lens[l]); ShaU(cs, data, lens[l]);
            printf("lo=%08x len=%04x md5=%08x,%08x md4=%08x,%08x sha=%08x,%08x\n", lo[a], lens[l],
                   c5[0], c5[1], c4[4], c4[5], cs[21], cs[22]);
        }
    return 0;
}
