//! Object manager extern declarations.

use crate::base::*;

// ObfReferenceObject / ObfDereferenceObject use fastcall convention.
// On i386, fastcall is distinct (param in ecx). On x86_64/arm64, it's the same as "C".

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ObfReferenceObject(Object: PVOID);
    fn ObfDereferenceObject(Object: PVOID);
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn ObfReferenceObject(Object: PVOID);
    fn ObfDereferenceObject(Object: PVOID);
}

/// ObReferenceObject — C macro forwarding to ObfReferenceObject.
#[inline]
pub unsafe fn ObReferenceObject(object: PVOID) {
    ObfReferenceObject(object);
}

/// ObDereferenceObject — C macro forwarding to ObfDereferenceObject.
#[inline]
pub unsafe fn ObDereferenceObject(object: PVOID) {
    ObfDereferenceObject(object);
}
