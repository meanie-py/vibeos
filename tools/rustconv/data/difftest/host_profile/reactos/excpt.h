/* Host stand-in for vcruntime's <excpt.h>, which drags in MSVC runtime headers that
   collide with glibc. winnt.h needs only the disposition enum from it. */
#pragma once
typedef enum _EXCEPTION_DISPOSITION {
    ExceptionContinueExecution,
    ExceptionContinueSearch,
    ExceptionNestedException,
    ExceptionCollidedUnwind
} EXCEPTION_DISPOSITION;
