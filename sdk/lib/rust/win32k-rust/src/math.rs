use win32k_sys::win32k::*;

#[no_mangle]
pub unsafe extern "system" fn EngMulDiv(
    iMultiplicand: INT,
    iMultiplier: INT,
    iDivisor: INT,
) -> INT {

    let mut iMultiplicand = iMultiplicand;
    let mut iDivisor = iDivisor;

    if iDivisor == 0 {
        return if (iMultiplicand ^ iMultiplier) < 0 { i32::MIN } else { i32::MAX };
    }

    if iDivisor < 0 {
        iMultiplicand = -iMultiplicand;
        iDivisor = -iDivisor;
    }

    // Int32x32To64(a,b) is a macro in <xdk/ntbasedef.h>:774 — inline its
    // sign-extending widened multiply rather than declaring it extern.
    let mut i64Multiplied: INT64 = (iMultiplicand as INT64) * (iMultiplier as INT64);

    if i64Multiplied >= 0 {
        i64Multiplied += (iDivisor / 2) as INT64;
    } else {
        i64Multiplied -= (iDivisor / 2) as INT64;
    }

    let i64Result: INT64 = i64Multiplied / iDivisor as INT64;

    if i64Result > i32::MAX as INT64 {
        return i32::MAX;
    }

    if i64Result < i32::MIN as INT64 {
        return i32::MIN;
    }

    i64Result as INT
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Math functions for i387.
//    Copyright (C) 1995, 1996, 1997 Free Software Foundation, Inc.
//    This file is part of the GNU C Library.
//    Contributed by John C. Bowman <bowman@ipp-garching.mpg.de>, 1995.
//
//    The GNU C Library is free software; you can redistribute it and/or
//    modify it under the terms of the GNU Lesser General Public
//    License as published by the Free Software Foundation; either
//    version 2.1 of the License, or (at your option) any later version.
//
//    The GNU C Library is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//    Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public
//    License along with the GNU C Library; if not, write to the Free
//    Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
//    Boston, MA 02110-1301, USA.
// */
//
// #include <win32k.h>
//
// INT
// APIENTRY
// EngMulDiv(
//     _In_ INT iMultiplicand,
//     _In_ INT iMultiplier,
//     _In_ INT iDivisor)
// {
//     INT64 i64Multiplied, i64Result;
//
//     /* Check for divide by zero */
//     if (iDivisor == 0)
//     {
//         /* Quick sign check and return "infinite" */
//         return ((iMultiplicand ^ iMultiplier) < 0) ? i32::MIN : i32::MAX;
//     }
//
//     /* We want to deal with a positive divisor to simplify the logic. */
//     if (iDivisor < 0)
//     {
//         iMultiplicand = -iMultiplicand;
//         iDivisor = -iDivisor;
//     }
//
//     /* Do the multiplication */
//     i64Multiplied = Int32x32To64(iMultiplicand, iMultiplier);
//
//     /* If the result is positive, we add to round, else we subtract to round. */
//     if (i64Multiplied >= 0)
//     {
//         i64Multiplied += (iDivisor / 2);
//     }
//     else
//     {
//         i64Multiplied -= (iDivisor / 2);
//     }
//
//     /* Now do the divide */
//     i64Result = i64Multiplied / iDivisor;
//
//     /* Check for positive overflow */
//     if (i64Result > i32::MAX)
//     {
//         return i32::MAX;
//     }
//
//     /* Check for negative overflow. */
//     if (i64Result < i32::MIN)
//     {
//         return i32::MIN;
//     }
//
//     return (INT)i64Result;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
