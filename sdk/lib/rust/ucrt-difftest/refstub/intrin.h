/* The UCRT headers pull <intrin.h> for MSVC intrinsics that have no SysV
   equivalent and whose real header #errors on a non-Windows target. Nothing
   the reference TUs actually CALL comes from here; they need the include to
   resolve, not its contents. */
#pragma once
