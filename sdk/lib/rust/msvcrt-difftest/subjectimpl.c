/*
 * subjectimpl.c — the msvcrt internals that the CRATE UNDER TEST imports.
 *
 * `msvcrt-crt-rust` is the code that links into msvcrt.dll. For the `_s`
 * ("secure") family its contract is not just the destination buffer — it is
 * *errno* plus the *invalid-parameter handler*:
 *
 *     #define MSVCRT_INVALID_PMT(x,err) (*_errno() = (err), \
 *                                        _invalid_parameter(NULL,NULL,NULL,0,0))
 *
 * Inside the real DLL those two resolve to msvcrt's own C. In this harness they
 * must be supplied here.
 *
 * WHY NOT REUSE refimpl.c's? Because refimpl's are deliberately renamed into the
 * private `msvcrt_ref_*` namespace (refstub/msvcrt.h) — that namespace exists
 * precisely so the REFERENCE can never bind to the CODE UNDER TEST. Reusing them
 * would defeat it, AND it would give the subject and the reference a SHARED
 * errno — so a test could not tell which side set it.
 *
 * Separate storage is the point: `ref_strcpy_s` writes msvcrt_ref_errno_value,
 * the Rust `strcpy_s` writes subj_errno_value, and classify asserts BOTH agree.
 * An errno contract you cannot observe independently is an errno contract you are
 * not testing.
 *
 * Compiled WITHOUT the rename prelude and WITHOUT refstub/msvcrt.h, so these land
 * under their plain names — exactly what msvcrt-crt-rust's `extern "C"` block
 * declares.
 */

#include <stddef.h>
#include <stdint.h>

/* ------------------------------------------------------------------ *
 * errno — the SUBJECT's, independent of the reference's.
 * ------------------------------------------------------------------ */
static __thread int subj_errno_value;

int *_errno(void)
{
    return &subj_errno_value;
}

/* Harness API (un-namespaced on purpose — these are read by classify.rs). */
int  subj_errno_get(void)      { return subj_errno_value; }
void subj_errno_set(int value) { subj_errno_value = value; }

/* ------------------------------------------------------------------ *
 * The invalid-parameter handler.
 *
 * Real msvcrt dispatches to a user-installed handler; the DEFAULT is "none
 * installed -> just return", and the errno store (the part that IS a value
 * contract) has already happened inside MSVCRT_INVALID_PMT before we get here.
 * So a no-op is the faithful default — but we also COUNT the calls, because
 * "did the invalid-parameter path fire at all?" is itself part of the `_s`
 * contract and classify asserts on it.
 * ------------------------------------------------------------------ */
static __thread int subj_invalid_pmt_calls;

void _invalid_parameter(const void *expr, const void *func, const void *file,
                        unsigned int line, uintptr_t arg)
{
    (void)expr; (void)func; (void)file; (void)line; (void)arg;
    subj_invalid_pmt_calls++;
}

int  subj_invalid_pmt_count(void) { return subj_invalid_pmt_calls; }
void subj_invalid_pmt_reset(void) { subj_invalid_pmt_calls = 0; }
