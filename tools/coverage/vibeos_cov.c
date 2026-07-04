/*
 * vibeos_cov.c — QEMU TCG plugin: record every translated-block start vaddr.
 *
 * Purpose: the boot coverage map (docs: coverage = which translated Rust
 * functions a smoke boot actually EXERCISES). TCG translates a TB exactly when
 * execution first reaches it, so the set of TB start vaddrs ≈ the set of
 * executed code addresses; symbolized against the kernel's function ranges it
 * yields the per-function exercised set and — more importantly — the
 * never-executed landmine registry (painting.rs sat dormant in exactly that
 * blind spot until Rust explorer's paint pattern first exercised it).
 *
 * Zero deps beyond the vendored qemu-plugin.h (no glib): open-addressing hash
 * set under a pthread mutex (tb_trans fires from vCPU threads), dumped sorted
 * at exit as one hex vaddr per line.
 *
 * Build:  gcc -shared -fPIC -O2 -o vibeos_cov.so vibeos_cov.c   (see build.sh)
 * Use:    qemu-system-* -accel tcg -plugin ./vibeos_cov.so,out=/tmp/cov.txt
 *         [,min=0xfffff80000000000][,max=0xffffffffffffffff]
 *
 * License: GPL-2.0-or-later (links against the QEMU plugin interface).
 */

#include <inttypes.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "qemu-plugin.h"

QEMU_PLUGIN_EXPORT int qemu_plugin_version = QEMU_PLUGIN_VERSION;

static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
static uint64_t *slots;      /* open addressing; 0 = empty (vaddr 0 never a TB) */
static size_t cap;           /* power of two */
static size_t used;
static char out_path[4096] = "vibeos_cov.txt";
static uint64_t filt_min;    /* inclusive; 0 = no filter */
static uint64_t filt_max = UINT64_MAX;

static void set_grow(void)
{
    size_t ncap = cap ? cap * 2 : (1u << 20);
    uint64_t *n = calloc(ncap, sizeof(*n));
    if (!n) {
        fprintf(stderr, "vibeos_cov: OOM growing to %zu slots\n", ncap);
        abort(); /* fail loud — a silently truncated coverage set is worse */
    }
    for (size_t i = 0; i < cap; i++) {
        uint64_t v = slots[i];
        if (v) {
            size_t j = (v * 0x9E3779B97F4A7C15ull) >> 1 & (ncap - 1);
            while (n[j]) {
                j = (j + 1) & (ncap - 1);
            }
            n[j] = v;
        }
    }
    free(slots);
    slots = n;
    cap = ncap;
}

static void set_add(uint64_t v)
{
    if (used * 10 >= cap * 7) {
        set_grow();
    }
    size_t j = (v * 0x9E3779B97F4A7C15ull) >> 1 & (cap - 1);
    while (slots[j]) {
        if (slots[j] == v) {
            return;
        }
        j = (j + 1) & (cap - 1);
    }
    slots[j] = v;
    used++;
}

static void tb_trans(qemu_plugin_id_t id, struct qemu_plugin_tb *tb)
{
    uint64_t va = qemu_plugin_tb_vaddr(tb);
    if (va < filt_min || va > filt_max) {
        return;
    }
    pthread_mutex_lock(&lock);
    set_add(va);
    pthread_mutex_unlock(&lock);
}

static int cmp_u64(const void *a, const void *b)
{
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return x < y ? -1 : x > y;
}

static void at_exit(qemu_plugin_id_t id, void *p)
{
    pthread_mutex_lock(&lock);
    uint64_t *all = malloc(used * sizeof(*all));
    size_t n = 0;
    if (all) {
        for (size_t i = 0; i < cap; i++) {
            if (slots[i]) {
                all[n++] = slots[i];
            }
        }
        qsort(all, n, sizeof(*all), cmp_u64);
    }
    FILE *f = fopen(out_path, "w");
    if (!f) {
        fprintf(stderr, "vibeos_cov: cannot write %s\n", out_path);
        pthread_mutex_unlock(&lock);
        free(all);
        return;
    }
    for (size_t i = 0; i < n; i++) {
        fprintf(f, "0x%" PRIx64 "\n", all[i]);
    }
    fclose(f);
    fprintf(stderr, "vibeos_cov: %zu unique TB vaddrs -> %s\n", n, out_path);
    pthread_mutex_unlock(&lock);
    free(all);
}

QEMU_PLUGIN_EXPORT int qemu_plugin_install(qemu_plugin_id_t id,
                                           const qemu_info_t *info,
                                           int argc, char **argv)
{
    for (int i = 0; i < argc; i++) {
        if (!strncmp(argv[i], "out=", 4)) {
            snprintf(out_path, sizeof(out_path), "%s", argv[i] + 4);
        } else if (!strncmp(argv[i], "min=", 4)) {
            filt_min = strtoull(argv[i] + 4, NULL, 0);
        } else if (!strncmp(argv[i], "max=", 4)) {
            filt_max = strtoull(argv[i] + 4, NULL, 0);
        } else {
            fprintf(stderr, "vibeos_cov: unknown arg %s\n", argv[i]);
            return -1; /* fail loud, don't half-configure */
        }
    }
    qemu_plugin_register_vcpu_tb_trans_cb(id, tb_trans);
    qemu_plugin_register_atexit_cb(id, at_exit, NULL);
    return 0;
}
