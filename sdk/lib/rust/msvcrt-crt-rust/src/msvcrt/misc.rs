use crate::prelude::*;
// NOT `use msvcrt_sys::*;` as well. prelude.rs already re-exports the whole island
// (`pub use msvcrt_sys::*`), so a second glob imports every name TWICE from two different
// roots and every use of an overlapping one is E0659 "ambiguous" — this file arrived with
// the second glob and ten `WEOF`/`FALSE` uses failed exactly that way. The prelude also
// re-types a few island constants to the shape their uses need, and an explicit item beats
// a glob, so going through the prelude keeps those choices. One door.
use core::ffi::c_void;

// misc.c's wine-debug diagnostics. `macro_rules!`, never a `fn` — an out-of-line no-op is
// an optimization barrier that has uncloaked latent ABI mismatches into `ud2`.

// ---- file-scope preamble (translated) ----
static mut output_format: core::ffi::c_uint = 0;

#[no_mangle]
pub unsafe extern "C" fn _beep(freq: u32, duration: u32) {
    // TRACE(":Freq %d, Duration %d\n",freq,duration);
    Beep(freq, duration);
}

#[no_mangle]
pub unsafe extern "C" fn srand(seed: core::ffi::c_uint) {
    let data: *mut thread_data_t = msvcrt_get_thread_data();
    (*data).random_seed = seed;
}

#[no_mangle]
pub unsafe extern "C" fn rand() -> core::ffi::c_int {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    // `wrapping_*`, not `*`/`+`. The C is `data->random_seed * 214013 + 2531011` on an
    // `unsigned int`, and wrapping IS the algorithm — this is MSVC's LCG, and truncating to
    // 32 bits at every step is what makes the sequence the one msvcrt produces. A plain `*`
    // is right in release (where Rust wraps too) and panics in debug, which is not a
    // cosmetic difference: it makes the function ungradeable, because the difftest harness
    // builds debug. Same class as mbcs.rs's underflow.
    (*data).random_seed = (*data).random_seed.wrapping_mul(214013).wrapping_add(2531011);
    (((*data).random_seed >> 16) & RAND_MAX) as core::ffi::c_int
}

#[no_mangle]
pub unsafe extern "C" fn rand_s(pval: *mut core::ffi::c_uint) -> core::ffi::c_int {
    if pval.is_null()
        || RtlGenRandom(pval as *mut _, core::mem::size_of::<core::ffi::c_uint>() as _) == 0
    {
        *_errno() = EINVAL as core::ffi::c_int;
        return EINVAL as core::ffi::c_int;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _sleep(timeout: __msvcrt_ulong) {
    Sleep(if timeout != 0 { timeout } else { 1 });
}

#[no_mangle]
pub unsafe extern "C" fn _lsearch(
    match_: *const core::ffi::c_void,
    mut start: *mut core::ffi::c_void,
    array_size: *mut core::ffi::c_uint,
    elem_size: core::ffi::c_uint,
    cf: Option<unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int>,
) -> *mut core::ffi::c_void {
    let mut size = *array_size;
    if size != 0 {
        loop {
            if cf.unwrap_unchecked()(match_, start) == 0 {
                return start;
            }
            start = (start as *mut core::ffi::c_char).add(elem_size as usize) as *mut core::ffi::c_void;
            size -= 1;
            if size == 0 {
                break;
            }
        }
    }

    memcpy(start, match_, elem_size as usize);
    *array_size.offset(0) += 1;
    start
}

#[no_mangle]
pub unsafe extern "C" fn bsearch_s(
    key: *const core::ffi::c_void,
    base: *const core::ffi::c_void,
    nmemb: usize,
    size: usize,
    compare: Option<
        unsafe extern "C" fn(
            *mut core::ffi::c_void,
            *const core::ffi::c_void,
            *const core::ffi::c_void,
        ) -> core::ffi::c_int,
    >,
    ctx: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    let mut min: isize = 0;
    let mut max: isize = nmemb as isize - 1;

    if MSVCRT_CHECK_PMT!(size != 0) == 0 {
        return core::ptr::null_mut();
    }
    if MSVCRT_CHECK_PMT!(compare.is_some()) == 0 {
        return core::ptr::null_mut();
    }

    while min <= max {
        let cursor: isize = min + (max - min) / 2;
        let ret = compare.unwrap_unchecked()(
            ctx,
            key,
            (base as *const core::ffi::c_char).offset(cursor * size as isize)
                as *const core::ffi::c_void,
        );
        if ret == 0 {
            return (base as *mut core::ffi::c_char).offset(cursor * size as isize)
                as *mut core::ffi::c_void;
        }
        if ret < 0 {
            max = cursor - 1;
        } else {
            min = cursor + 1;
        }
    }
    core::ptr::null_mut()
}

unsafe extern "C" fn compare_wrapper(ctx: *mut core::ffi::c_void, e1: *const core::ffi::c_void, e2: *const core::ffi::c_void) -> core::ffi::c_int {
    let compare: Option<unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int> = core::mem::transmute(ctx);
    compare.unwrap_unchecked()(e1, e2)
}

#[no_mangle]
pub unsafe extern "C" fn bsearch(
    key: *const core::ffi::c_void,
    base: *const core::ffi::c_void,
    nmemb: usize,
    size: usize,
    compar: Option<
        unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int,
    >,
) -> *mut core::ffi::c_void {
    bsearch_s(
        key,
        base,
        nmemb,
        size,
        Some(compare_wrapper),
        core::mem::transmute::<_, *mut core::ffi::c_void>(compar),
    )
}

#[no_mangle]
pub unsafe extern "C" fn chkesp_fail() {
    ERR!("Stack pointer incorrect after last function call - Bad prototype/spec entry?\n");
    DebugBreak();
}

// misc.c's `__ASM_GLOBAL_FUNC(_chkesp, ...)` (commented in full below), i386 only. This
// was an empty `fn`, which silently dropped the check: MSVC debug code calls `_chkesp`
// with ZF from comparing esp before and after a call, and only the asm can read ZF.
#[cfg(all(target_arch = "x86", target_os = "windows"))]
core::arch::global_asm!(
    ".text",
    ".globl __chkesp",
    "__chkesp:",
    "jnz 1f",
    "ret",
    "1:",
    "pushl %ebp",
    "movl %esp, %ebp",
    "subl $12, %esp",
    "pushl %eax",
    "pushl %ecx",
    "pushl %edx",
    "call _chkesp_fail",
    "popl %edx",
    "popl %ecx",
    "popl %eax",
    "leave",
    "ret",
    options(att_syntax)
);
unsafe extern "C" fn swap(mut l: *mut core::ffi::c_char, mut r: *mut core::ffi::c_char, mut size: usize) {
    while size != 0 {
        size -= 1;
        let tmp = *l;
        *l = *r;
        l = l.add(1);
        *r = tmp;
        r = r.add(1);
    }
}
unsafe extern "C" fn small_sort(base: *mut core::ffi::c_void, nmemb: usize, size: usize,
        compar: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int>,
        context: *mut core::ffi::c_void) {
    let mut max: *mut core::ffi::c_char;
    let mut p: *mut core::ffi::c_char = base as *mut core::ffi::c_char;
    let cmp = compar.unwrap_unchecked();

    let mut e = nmemb;
    while e > 1 {
        max = base as *mut core::ffi::c_char;
        let mut i = 1;
        while i < e {
            p = (base as *mut core::ffi::c_char).add(i * size);
            if cmp(context, p as *const core::ffi::c_void, max as *const core::ffi::c_void) > 0 {
                max = p;
            }
            i += 1;
        }

        if p != max {
            swap(p, max, size);
        }
        e -= 1;
    }
}
unsafe extern "C" fn quick_sort(base: *mut core::ffi::c_void, nmemb: usize, size: usize,
        compar: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int>,
        context: *mut core::ffi::c_void) {
    let mut stack_lo: [usize; 8 * core::mem::size_of::<usize>()] = [0; 8 * core::mem::size_of::<usize>()];
    let mut stack_hi: [usize; 8 * core::mem::size_of::<usize>()] = [0; 8 * core::mem::size_of::<usize>()];
    let mut beg: usize;
    let mut end: usize;
    let mut lo: usize;
    let mut hi: usize;
    let mut med: usize;
    let mut stack_pos: core::ffi::c_int;

    let cmp = compar.unwrap_unchecked();

    stack_pos = 0;
    stack_lo[stack_pos as usize] = 0;
    stack_hi[stack_pos as usize] = nmemb - 1;

    macro_rules! X {
        ($i:expr) => {
            (base as *mut core::ffi::c_char).add(size * ($i)) as *mut core::ffi::c_void
        };
    }

    while stack_pos >= 0 {
        beg = stack_lo[stack_pos as usize];
        end = stack_hi[stack_pos as usize];
        stack_pos -= 1;

        if end - beg < 8 {
            small_sort(X!(beg), end - beg + 1, size, compar, context);
            continue;
        }

        lo = beg;
        hi = end;
        med = lo + (hi - lo + 1) / 2;
        if cmp(context, X!(lo), X!(med)) > 0 {
            swap(X!(lo) as *mut core::ffi::c_char, X!(med) as *mut core::ffi::c_char, size);
        }
        if cmp(context, X!(lo), X!(hi)) > 0 {
            swap(X!(lo) as *mut core::ffi::c_char, X!(hi) as *mut core::ffi::c_char, size);
        }
        if cmp(context, X!(med), X!(hi)) > 0 {
            swap(X!(med) as *mut core::ffi::c_char, X!(hi) as *mut core::ffi::c_char, size);
        }

        lo += 1;
        hi -= 1;
        while 1 != 0 {
            while lo <= hi {
                if lo != med && cmp(context, X!(lo), X!(med)) > 0 {
                    break;
                }
                lo += 1;
            }

            while med != hi {
                if cmp(context, X!(hi), X!(med)) <= 0 {
                    break;
                }
                hi -= 1;
            }

            if hi < lo {
                break;
            }

            swap(X!(lo) as *mut core::ffi::c_char, X!(hi) as *mut core::ffi::c_char, size);
            if hi == med {
                med = lo;
            }
            lo += 1;
            hi -= 1;
        }

        while hi > beg {
            if hi != med && cmp(context, X!(hi), X!(med)) != 0 {
                break;
            }
            hi -= 1;
        }

        if hi - beg >= end - lo {
            stack_pos += 1;
            stack_lo[stack_pos as usize] = beg;
            stack_hi[stack_pos as usize] = hi;
            stack_pos += 1;
            stack_lo[stack_pos as usize] = lo;
            stack_hi[stack_pos as usize] = end;
        } else {
            stack_pos += 1;
            stack_lo[stack_pos as usize] = lo;
            stack_hi[stack_pos as usize] = end;
            stack_pos += 1;
            stack_lo[stack_pos as usize] = beg;
            stack_hi[stack_pos as usize] = hi;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn qsort_s(base: *mut core::ffi::c_void, nmemb: usize, size: usize,
    compar: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int>,
    context: *mut core::ffi::c_void) {
    let total_size: usize = nmemb * size;

    if MSVCRT_CHECK_PMT!(!base.is_null() || (base.is_null() && nmemb == 0)) == 0 { return; }
    if MSVCRT_CHECK_PMT!(size > 0) == 0 { return; }
    if MSVCRT_CHECK_PMT!(compar.is_some()) == 0 { return; }
    if total_size / size != nmemb { return; }

    if nmemb < 2 { return; }

    quick_sort(base, nmemb, size, compar, context);
}
#[no_mangle]
pub unsafe extern "C" fn qsort(base: *mut core::ffi::c_void, nmemb: usize, size: usize,
    compar: Option<unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int>) {
    qsort_s(
        base,
        nmemb,
        size,
        Some(compare_wrapper),
        core::mem::transmute::<_, *mut core::ffi::c_void>(compar),
    );
}

#[no_mangle]
pub unsafe extern "C" fn _get_output_format() -> ::core::ffi::c_uint {
    output_format
}

#[no_mangle]
pub unsafe extern "C" fn _set_output_format(new_output_format: c_uint) -> c_uint {
    let ret = output_format;

    if MSVCRT_CHECK_PMT!(new_output_format == 0 || new_output_format == _TWO_DIGIT_EXPONENT) == 0
    {
        return ret;
    }

    output_format = new_output_format;
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _resetstkoflw() -> core::ffi::c_int {
    let mut stack_addr: core::ffi::c_int = 0;
    let mut oldprot: __msvcrt_ulong = 0;

    VirtualProtect(
        core::ptr::addr_of_mut!(stack_addr) as *mut core::ffi::c_void,
        1,
        PAGE_GUARD | PAGE_READWRITE,
        core::ptr::addr_of_mut!(oldprot),
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll misc functions
//  *
//  * Copyright 2000 Jon Griffiths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <stdlib.h>
// #include <sys/types.h>
//
// #include "msvcrt.h"
// #include "wine/debug.h"
// #include "ntsecapi.h"
// #include "windows.h"
// #include "wine/asm.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// static unsigned int output_format;
//
// #ifdef _MSC_VER
// #pragma function(_byteswap_ushort, _byteswap_ulong, _byteswap_uint64)
// #endif
//
// /*********************************************************************
//  *		_beep (MSVCRT.@)
//  */
// void CDECL _beep( unsigned int freq, unsigned int duration)
// {
//     TRACE(":Freq %d, Duration %d\n",freq,duration);
//     Beep(freq, duration);
// }
//
// /*********************************************************************
//  *		srand (MSVCRT.@)
//  */
// void CDECL srand( unsigned int seed )
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     data->random_seed = seed;
// }
//
// /*********************************************************************
//  *		rand (MSVCRT.@)
//  */
// int CDECL rand(void)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     /* this is the algorithm used by MSVC, according to
//      * http://en.wikipedia.org/wiki/List_of_pseudorandom_number_generators */
//     data->random_seed = data->random_seed * 214013 + 2531011;
//     return (data->random_seed >> 16) & RAND_MAX;
// }
//
// /*********************************************************************
//  *		rand_s (MSVCRT.@)
//  */
// int CDECL rand_s(unsigned int *pval)
// {
//     if (!pval || !RtlGenRandom(pval, sizeof(*pval)))
//     {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//     return 0;
// }
//
// /*********************************************************************
//  *		_sleep (MSVCRT.@)
//  */
// void CDECL _sleep(__msvcrt_ulong timeout)
// {
//   TRACE("_sleep for %ld milliseconds\n",timeout);
//   Sleep((timeout)?timeout:1);
// }
//
// /*********************************************************************
//  *		_lfind (MSVCRT.@)
//  */
// void*  _lfind(const void* match, const void* start,
//                    unsigned int* array_size, unsigned int elem_size,
//                    int (CDECL *cf)(const void*,const void*) )
// {
//   unsigned int size = *array_size;
//   if (size)
//     do
//     {
//       if (cf(match, start) == 0)
//         return (void *)start; /* found */
//       start = (const char *)start + elem_size;
//     } while (--size);
//   return NULL;
// }
//
// /*********************************************************************
//  *		_lfind_s (MSVCRT.@)
//  */
// void*  _lfind_s(const void* match, const void* start,
//                    unsigned int* array_size, unsigned int elem_size,
//                    int (CDECL *cf)(void*,const void*,const void*),
//                    void* context)
// {
//   unsigned int size;
//   if (!MSVCRT_CHECK_PMT(match != NULL)) return NULL;
//   if (!MSVCRT_CHECK_PMT(array_size != NULL)) return NULL;
//   if (!MSVCRT_CHECK_PMT(start != NULL || *array_size == 0)) return NULL;
//   if (!MSVCRT_CHECK_PMT(cf != NULL)) return NULL;
//   if (!MSVCRT_CHECK_PMT(elem_size != 0)) return NULL;
//
//   size = *array_size;
//   if (size)
//     do
//     {
//       if (cf(context, match, start) == 0)
//         return (void *)start; /* found */
//       start = (const char *)start + elem_size;
//     } while (--size);
//   return NULL;
// }
//
// /*********************************************************************
//  *		_lsearch (MSVCRT.@)
//  */
// void*  _lsearch(const void* match, void* start,
//                      unsigned int* array_size, unsigned int elem_size,
//                      int (CDECL *cf)(const void*,const void*) )
// {
//   unsigned int size = *array_size;
//   if (size)
//     do
//     {
//       if (cf(match, start) == 0)
//         return start; /* found */
//       start = (char*)start + elem_size;
//     } while (--size);
//
//   /* not found, add to end */
//   memcpy(start, match, elem_size);
//   array_size[0]++;
//   return start;
// }
//
// /*********************************************************************
//  *                  bsearch_s (msvcrt.@)
//  */
// void*  bsearch_s(const void *key, const void *base, size_t nmemb, size_t size,
//                              int (__cdecl *compare)(void *, const void *, const void *), void *ctx)
// {
//     ssize_t min = 0;
//     ssize_t max = nmemb - 1;
//
//     if (!MSVCRT_CHECK_PMT(size != 0)) return NULL;
//     if (!MSVCRT_CHECK_PMT(compare != NULL)) return NULL;
//
//     while (min <= max)
//     {
//         ssize_t cursor = min + (max - min) / 2;
//         int ret = compare(ctx, key,(const char *)base+(cursor*size));
//         if (!ret)
//             return (char*)base+(cursor*size);
//         if (ret < 0)
//             max = cursor - 1;
//         else
//             min = cursor + 1;
//     }
//     return NULL;
// }
//
// static int CDECL compare_wrapper(void *ctx, const void *e1, const void *e2)
// {
//     int (__cdecl *compare)(const void *, const void *) = ctx;
//     return compare(e1, e2);
// }
//
// /*********************************************************************
//  *                  bsearch (msvcrt.@)
//  */
// void*  bsearch(const void *key, const void *base, size_t nmemb,
//         size_t size, int (__cdecl *compar)(const void *, const void *))
// {
//     return bsearch_s(key, base, nmemb, size, compare_wrapper, compar);
// }
// /*********************************************************************
//  *		_chkesp (MSVCRT.@)
//  *
//  * Trap to a debugger if the value of the stack pointer has changed.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  Does not return.
//  *
//  * NOTES
//  *  This function is available for iX86 only.
//  *
//  *  When VC++ generates debug code, it stores the value of the stack pointer
//  *  before calling any external function, and checks the value following
//  *  the call. It then calls this function, which will trap if the values are
//  *  not the same. Usually this means that the prototype used to call
//  *  the function is incorrect.  It can also mean that the .spec entry has
//  *  the wrong calling convention or parameters.
//  */
// #ifdef __i386__
//
// # if defined(__GNUC__) || defined(__clang__)
//
// __ASM_GLOBAL_FUNC(_chkesp,
//                   "jnz 1f\n\t"
//                   "ret\n"
//                   "1:\tpushl %ebp\n\t"
//                   __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                   __ASM_CFI(".cfi_rel_offset %ebp,0\n\t")
//                   "movl %esp,%ebp\n\t"
//                   __ASM_CFI(".cfi_def_cfa_register %ebp\n\t")
//                   "subl $12,%esp\n\t"
//                   "pushl %eax\n\t"
//                   "pushl %ecx\n\t"
//                   "pushl %edx\n\t"
//                   "call " __ASM_NAME("chkesp_fail") "\n\t"
//                   "popl %edx\n\t"
//                   "popl %ecx\n\t"
//                   "popl %eax\n\t"
//                   "leave\n\t"
//                   __ASM_CFI(".cfi_def_cfa %esp,4\n\t")
//                   __ASM_CFI(".cfi_same_value %ebp\n\t")
//                   "ret")
//
// void CDECL chkesp_fail(void)
// {
//   ERR("Stack pointer incorrect after last function call - Bad prototype/spec entry?\n");
//   DebugBreak();
// }
//
// # else  /* __GNUC__ || __clang__ */
//
// /**********************************************************************/
//
// void CDECL _chkesp(void)
// {
// }
//
// # endif  /* __GNUC__ || __clang__ */
//
// #endif  /* __i386__ */
//
// static inline void swap(char *l, char *r, size_t size)
// {
//     char tmp;
//
//     while(size--) {
//         tmp = *l;
//         *l++ = *r;
//         *r++ = tmp;
//     }
// }
//
// static void small_sort(void *base, size_t nmemb, size_t size,
//         int (CDECL *compar)(void *, const void *, const void *), void *context)
// {
//     size_t e, i;
//     char *max, *p;
//
//     for(e=nmemb; e>1; e--) {
//         max = base;
//         for(i=1; i<e; i++) {
//             p = (char*)base + i*size;
//             if(cmp(context, p, max) > 0)
//                 max = p;
//         }
//
//         if(p != max)
//             swap(p, max, size);
//     }
// }
//
// static void quick_sort(void *base, size_t nmemb, size_t size,
//         int (CDECL *compar)(void *, const void *, const void *), void *context)
// {
//     size_t stack_lo[8*sizeof(size_t)], stack_hi[8*sizeof(size_t)];
//     size_t beg, end, lo, hi, med;
//     int stack_pos;
//
//     stack_pos = 0;
//     stack_lo[stack_pos] = 0;
//     stack_hi[stack_pos] = nmemb-1;
//
// #define X(i) ((char*)base+size*(i))
//     while(stack_pos >= 0) {
//         beg = stack_lo[stack_pos];
//         end = stack_hi[stack_pos--];
//
//         if(end-beg < 8) {
//             small_sort(X(beg), end-beg+1, size, compar, context);
//             continue;
//         }
//
//         lo = beg;
//         hi = end;
//         med = lo + (hi-lo+1)/2;
//         if(cmp(context, X(lo), X(med)) > 0)
//             swap(X(lo), X(med), size);
//         if(cmp(context, X(lo), X(hi)) > 0)
//             swap(X(lo), X(hi), size);
//         if(cmp(context, X(med), X(hi)) > 0)
//             swap(X(med), X(hi), size);
//
//         lo++;
//         hi--;
//         while(1) {
//             while(lo <= hi) {
//                 if(lo!=med && cmp(context, X(lo), X(med))>0)
//                     break;
//                 lo++;
//             }
//
//             while(med != hi) {
//                 if(cmp(context, X(hi), X(med)) <= 0)
//                     break;
//                 hi--;
//             }
//
//             if(hi < lo)
//                 break;
//
//             swap(X(lo), X(hi), size);
//             if(hi == med)
//                 med = lo;
//             lo++;
//             hi--;
//         }
//
//         while(hi > beg) {
//             if(hi!=med && cmp(context, X(hi), X(med))!=0)
//                 break;
//             hi--;
//         }
//
//         if(hi-beg >= end-lo) {
//             stack_lo[++stack_pos] = beg;
//             stack_hi[stack_pos] = hi;
//             stack_lo[++stack_pos] = lo;
//             stack_hi[stack_pos] = end;
//         }else {
//             stack_lo[++stack_pos] = lo;
//             stack_hi[stack_pos] = end;
//             stack_lo[++stack_pos] = beg;
//             stack_hi[stack_pos] = hi;
//         }
//     }
// #undef X
// }
//
// /*********************************************************************
//  * qsort_s (MSVCRT.@)
//  *
//  * This function is trying to sort data doing identical comparisons
//  * as native does. There are still cases where it behaves differently.
//  */
// void CDECL qsort_s(void *base, size_t nmemb, size_t size,
//     int (CDECL *compar)(void *, const void *, const void *), void *context)
// {
//     const size_t total_size = nmemb*size;
//
//     if (!MSVCRT_CHECK_PMT(base != NULL || (base == NULL && nmemb == 0))) return;
//     if (!MSVCRT_CHECK_PMT(size > 0)) return;
//     if (!MSVCRT_CHECK_PMT(compar != NULL)) return;
//     if (total_size / size != nmemb) return;
//
//     if (nmemb < 2) return;
//
//     quick_sort(base, nmemb, size, compar, context);
// }
//
// /*********************************************************************
//  * qsort (MSVCRT.@)
//  */
// void CDECL qsort(void *base, size_t nmemb, size_t size,
//         int (CDECL *compar)(const void*, const void*))
// {
//     qsort_s(base, nmemb, size, compare_wrapper, compar);
// }
//
// /*********************************************************************
//  * _get_output_format (MSVCRT.@)
//  */
// unsigned int CDECL _get_output_format(void)
// {
//    return output_format;
// }
//
// /*********************************************************************
//  * _set_output_format (MSVCRT.@)
//  */
// unsigned int CDECL _set_output_format(unsigned int new_output_format)
// {
//     unsigned int ret = output_format;
//
//     if(!MSVCRT_CHECK_PMT(new_output_format==0 || new_output_format==_TWO_DIGIT_EXPONENT))
//         return ret;
//
//     output_format = new_output_format;
//     return ret;
// }
//
// /*********************************************************************
//  * _resetstkoflw (MSVCRT.@)
//  */
// int CDECL _resetstkoflw(void)
// {
//     int stack_addr;
//     DWORD oldprot;
//
//     /* causes stack fault that updates NtCurrentTeb()->Tib.StackLimit */
//     return VirtualProtect(&stack_addr, 1, PAGE_GUARD|PAGE_READWRITE, &oldprot);
// }
//
//
//
//
//
//
//
//
//
//

// ---- hand-written: the two bodies --skip-fn dropped -------------------------------------
//
// The paid translation damaged the deterministic call envelope on both (the sealed
// `cf(...)` occurrence came back missing / non-executable), so rustconv REFUSED them rather
// than shipping a body whose one interesting line is the callback dispatch. Transcribed from
// dll/win32/msvcrt/misc.c instead. The `Option<fn>` is how Rust spells a nullable C function
// pointer; `unwrap_unchecked` is the faithful call — a null `cf` is UB in the C too, and
// `unwrap()` would introduce a panic the C does not have.

/// `misc.c` — `void* CDECL _lfind(const void*, const void*, unsigned int*, unsigned int,
/// int (CDECL*)(const void*,const void*))`. Linear search, NO insert (that is `_lsearch`),
/// and it does NOT validate its arguments (that is `_lfind_s`).
#[no_mangle]
pub unsafe extern "C" fn _lfind(
    match_: *const core::ffi::c_void,
    mut start: *const core::ffi::c_void,
    array_size: *mut core::ffi::c_uint,
    elem_size: core::ffi::c_uint,
    cf: Option<
        unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int,
    >,
) -> *mut core::ffi::c_void {
    let mut size = *array_size;
    if size != 0 {
        loop {
            if cf.unwrap_unchecked()(match_, start) == 0 {
                return start as *mut core::ffi::c_void; // found
            }
            start = (start as *const core::ffi::c_char).add(elem_size as usize)
                as *const core::ffi::c_void;
            size -= 1;
            if size == 0 {
                break;
            }
        }
    }
    core::ptr::null_mut()
}

/// `misc.c` — `_lfind_s`. Same search, but with the five `MSVCRT_CHECK_PMT` guards, and the
/// comparator takes `context` FIRST. Note the third guard is a disjunction: a NULL `start` is
/// legal exactly when the array is empty.
#[no_mangle]
pub unsafe extern "C" fn _lfind_s(
    match_: *const core::ffi::c_void,
    mut start: *const core::ffi::c_void,
    array_size: *mut core::ffi::c_uint,
    elem_size: core::ffi::c_uint,
    cf: Option<
        unsafe extern "C" fn(
            *mut core::ffi::c_void,
            *const core::ffi::c_void,
            *const core::ffi::c_void,
        ) -> core::ffi::c_int,
    >,
    context: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    if MSVCRT_CHECK_PMT!(!match_.is_null()) == 0 {
        return core::ptr::null_mut();
    }
    if MSVCRT_CHECK_PMT!(!array_size.is_null()) == 0 {
        return core::ptr::null_mut();
    }
    if MSVCRT_CHECK_PMT!(!start.is_null() || *array_size == 0) == 0 {
        return core::ptr::null_mut();
    }
    if MSVCRT_CHECK_PMT!(cf.is_some()) == 0 {
        return core::ptr::null_mut();
    }
    if MSVCRT_CHECK_PMT!(elem_size != 0) == 0 {
        return core::ptr::null_mut();
    }

    let mut size = *array_size;
    if size != 0 {
        loop {
            if cf.unwrap_unchecked()(context, match_, start) == 0 {
                return start as *mut core::ffi::c_void; // found
            }
            start = (start as *const core::ffi::c_char).add(elem_size as usize)
                as *const core::ffi::c_void;
            size -= 1;
            if size == 0 {
                break;
            }
        }
    }
    core::ptr::null_mut()
}

// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
