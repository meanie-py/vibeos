/*
 * perfctl.c — the C half of the perf harness's NULL HYPOTHESIS.
 *
 * tests/perf.rs compares each flipped Rust primitive against the C it replaced
 * and fails the flip past a fixed slowdown multiple. That verdict is only worth
 * something if the harness can RESOLVE the multiple it tests for, and at the
 * small sizes it benches it cannot.
 *
 * At n=16 a call to one of these primitives costs ~13 cycles END TO END, so the
 * reading is dominated by per-call front-end effects — where the body landed in
 * the I-cache, how its loop straddles a fetch window, whether the predictor has
 * the exit branch. Those move with edits that have nothing to do with the body.
 * Measured directly: taking the shipping `strcat` and DELETING two null checks
 * from it — strictly less work — made it read 1.65x SLOWER at n=16 and 1.82x at
 * n=64, while at n=4096 the same pair read 1.03x. A fixed 2.0x bar cannot
 * survive that, and did not: the gate rejected a lane for `strlen/16` on one run
 * and `strcat/16` on the next, with no code change in between.
 *
 * So measure the band instead of guessing it. This file is compiled TWICE, into
 * `*_a` and `*_b` symbol sets (build.rs, -DPERF_CTL_TWIN). The two copies are
 * the same source through the same compiler with the same flags, so they are
 * identical code at different addresses, and benching one against the other
 * MUST read 1.00x. Whatever it actually reads is this harness's error at that
 * size, in this binary, under this load, and no case may be failed for
 * exceeding the threshold by less than that.
 *
 * TWO COPIES OF ONE LANGUAGE, not one of each. The first cut of this control
 * benched the C body against its Rust twin, which reads 1.28-1.43x at n=4096
 * and above — but at those sizes per-call effects are noise-level, so that
 * number was not harness error at all. It was rustc and cc genuinely generating
 * different code for the same loop, which is the very thing this harness exists
 * to measure. Folding it into the error bar would have inflated the bar to
 * ~2.8x at every large size and quietly retired the gate. Same language, same
 * compiler, same flags on both sides is the only comparison whose expected
 * value is exactly 1.00x.
 *
 * The two copies must not be merged back together, or the control becomes
 * vacuous — it would read a perfect 1.00x forever while measuring nothing.
 * Separate TUs put them out of reach of gcc's -fipa-icf (which folds identical
 * functions WITHIN a TU at -O2), and no ICF is enabled at link. perf.rs asserts
 * the addresses actually differ rather than trusting that.
 *
 * -fno-builtin IS LOAD-BEARING (build.rs). Without it the compiler recognizes
 * these loops and substitutes glibc's vectorized routine. Both copies would get
 * the same treatment so the control would still read 1.00x, but it would be
 * measuring the front-end behaviour of an AVX2 library routine rather than of
 * the byte loops the suite actually benches, and the error bar it produced
 * would not describe those cases. It is the counterpart of `#![no_builtins]` on
 * the Rust side, which is additionally what stops LLVM folding a byte loop into
 * a call to the very function being defined.
 *
 * Nothing here is under test and nothing here is an oracle. Keep the three
 * shapes matched to the three the suite benches — a copy (strcpy/strcat), a
 * scan (strlen/strchr) and a word fill (memset) — because the per-call floor
 * differs between them.
 */

#include <stddef.h>

#ifndef PERF_CTL_TWIN
#error "build.rs must compile this twice, with -DPERF_CTL_TWIN=a and =b"
#endif

#define CAT_(a, b) a##b
#define CAT(a, b) CAT_(a, b)
#define SYM(base) CAT(base, PERF_CTL_TWIN)

/* copy shape — `while ((*d++ = *s++));`, the strcpy/strcat inner loop */
char *SYM(perf_ctl_copy_c_)(char *d, const char *s)
{
    char *r = d;
    while ((*d++ = *s++))
        ;
    return r;
}

/* scan shape — `while (*p) p++;`, the strlen/strchr inner loop */
size_t SYM(perf_ctl_scan_c_)(const char *s)
{
    const char *p = s;
    while (*p)
        p++;
    return (size_t)(p - s);
}

/* fill shape — the 8-bytes-at-a-time store loop, memset's inner loop */
void *SYM(perf_ctl_fill_c_)(void *dst, int val, size_t count)
{
    unsigned char *p = (unsigned char *)dst;
    unsigned char b = (unsigned char)val;
    unsigned long long w = 0x0101010101010101ULL * (unsigned long long)b;
    size_t i = 0;
    while (i + 8 <= count) {
        __builtin_memcpy(p + i, &w, 8); /* an unaligned 8-byte store, not a call */
        i += 8;
    }
    while (i < count) {
        p[i] = b;
        i++;
    }
    return dst;
}
