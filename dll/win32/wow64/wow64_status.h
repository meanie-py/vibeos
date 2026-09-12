/*
 * PROJECT:     vibeos WoW64
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     The status sentinels the syscall dispatchers agree on
 * COPYRIGHT:   vibeos WoW64 lane
 *
 * These three values used to be #define'd separately in each of the four
 * dispatcher translation units, and one copy drifted: generated
 * wow64_automarshal.c spelled WOW64_MANUAL_UNIMPL 0xC0BBBBBB while the three
 * hand-written files spelled it 0x7FFFFFFF. Because Wow64ManualDispatch tails
 * unconditionally into Wow64AutoMarshalDispatch, the value that actually came
 * back was the generated one -- so both callers' `Status == WOW64_MANUAL_UNIMPL`
 * branches were dead, and an unhandled syscall returned a bare sentinel to the
 * guest instead of printing the line that NAMES it. A missing marshaller is the
 * single most common thing this lane has to diagnose, and the diagnostic for it
 * had been silently off. One header, one definition each, so it cannot recur.
 */

#ifndef _WOW64_STATUS_H_
#define _WOW64_STATUS_H_

/* What a guest sees when no thunk could serve its syscall. Both sentinels below
 * are converted to this before returning, so neither ever reaches guest code. */
#define WOW64_STATUS_INVALID_SYSTEM_SERVICE ((NTSTATUS)0xC000001CL)

/* "A hand-written marshaller exists for this name but declined." Internal to
 * Wow64ManualDispatch and its callers. Deliberately a SUCCESS-range value: a
 * marshaller that forgets to convert it fails visibly rather than looking like
 * a plausible error code. */
#define WOW64_MANUAL_UNIMPL ((NTSTATUS)0x7FFFFFFF)

/* "No thunk anywhere -- not hand-written, not generated." Returned by the tail
 * of Wow64AutoMarshalDispatch, which is where every unhandled name ends up. */
#define WOW64_AUTOMARSHAL_UNIMPL ((NTSTATUS)0xC0BBBBBBL)

/* Either sentinel means the same thing to a caller: nothing served this name. */
#define WOW64_IS_UNIMPL(st) \
    ((st) == WOW64_MANUAL_UNIMPL || (st) == WOW64_AUTOMARSHAL_UNIMPL)

#endif /* _WOW64_STATUS_H_ */
