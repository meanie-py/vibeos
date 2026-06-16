use ntdll_sys::nt::*;
use core::ffi::c_void;

// winnt.h: #define Int32x32To64(a,b) ((LONGLONG)(a)*(LONGLONG)(b))
#[inline]
fn Int32x32To64(a: INT, b: INT) -> LONGLONG {
    (a as LONGLONG) * (b as LONGLONG)
}

// RtlEnlargedUnsignedDivide(dividend, divisor, remainder) = { *remainder = dividend %
// divisor; dividend / divisor }. It's an ntdll export only on i386 (`-arch=win32` in
// ntdll.spec); on amd64/arm64 the C gets it FORCEINLINE'd. Inline it natively so the
// translation is cross-arch and faithful (identical result on every target).
#[inline]
unsafe fn rtl_enlarged_unsigned_divide(dividend: u64, divisor: u32, remainder: *mut u32) -> u32 {
    *remainder = (dividend % divisor as u64) as u32;
    (dividend / divisor as u64) as u32
}

#[no_mangle]
pub unsafe extern "system" fn MulDiv(
    mut nNumber: INT,
    mut nNumerator: INT,
    mut nDenominator: INT,
) -> INT {
    let mut Result: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };

    let Negative: LONG = (nNumber ^ nNumerator ^ nDenominator) as LONG;

    // C does `*= -1`; on INT_MIN that's signed overflow (UB in C, wraps in practice).
    // wrapping_neg() reproduces the wrap on all build profiles (avoids a debug panic).
    if nNumber < 0 {
        nNumber = nNumber.wrapping_neg();
    }
    if nNumerator < 0 {
        nNumerator = nNumerator.wrapping_neg();
    }
    if nDenominator < 0 {
        nDenominator = nDenominator.wrapping_neg();
    }

    Result.QuadPart = Int32x32To64(nNumber, nNumerator) + (nDenominator / 2) as LONGLONG;

    if nDenominator as LONG > Result.__bindgen_anon_1.HighPart {
        // C: RtlEnlargedUnsignedDivide(*(PULARGE_INTEGER)&Result, nDenominator, &Result.HighPart)
        // — dividend passed BY VALUE (copy of QuadPart) before HighPart is overwritten.
        let dividend = Result.QuadPart as u64;
        let rem_ptr = &mut Result.__bindgen_anon_1.HighPart as *mut LONG as *mut u32;
        let quotient = rtl_enlarged_unsigned_divide(dividend, nDenominator as ULONG, rem_ptr);
        Result.__bindgen_anon_1.LowPart = quotient;

        if (Result.__bindgen_anon_1.LowPart as LONG) >= 0 {
            return if Negative >= 0 {
                Result.__bindgen_anon_1.LowPart as LONG
            } else {
                -(Result.__bindgen_anon_1.LowPart as LONG)
            } as INT;
        }
    }

    -1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/wine/muldiv.c
//  * PURPOSE:
//  * PROGRAMMER:      Casper S. Hornstrup
//  *                  Gunnar Andre Dalsnes
//  * UPDATE HISTORY:
//  *                  Created 06/12/2002
//  */
//
// #include <k32.h>
//
// /***********************************************************************
//  *           MulDiv   (KERNEL32.@)
//  * RETURNS
//  *      Result of multiplication and division
//  *      -1: Overflow occurred or Divisor was 0
//  * FIXME! move to correct file
//  *
//  * @implemented
//  */
// INT
// WINAPI
// MulDiv(INT nNumber,
//        INT nNumerator,
//        INT nDenominator)
// {
//     LARGE_INTEGER Result;
//     LONG Negative;
//
//     /* Find out if this will be a negative result */
//     Negative = nNumber ^ nNumerator ^ nDenominator;
//
//     /* Turn all the parameters into absolute values */
//     if (nNumber < 0) nNumber *= -1;
//     if (nNumerator < 0) nNumerator *= -1;
//     if (nDenominator < 0) nDenominator *= -1;
//
//     /* Calculate the result */
//     Result.QuadPart = Int32x32To64(nNumber, nNumerator) + (nDenominator / 2);
//
//     /* Now check for overflow */
//     if (nDenominator > Result.HighPart)
//     {
//         /* Divide the product to get the quotient and remainder */
//         Result.LowPart = RtlEnlargedUnsignedDivide(*(PULARGE_INTEGER)&Result,
//                                                    (ULONG)nDenominator,
//                                                    (PULONG)&Result.HighPart);
//
//         /* Do the sign changes */
//         if ((LONG)Result.LowPart >= 0)
//         {
//             return (Negative >= 0) ? (LONG)Result.LowPart : -(LONG)Result.LowPart;
//         }
//     }
//
//     /* Return overflow */
//     return - 1;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
