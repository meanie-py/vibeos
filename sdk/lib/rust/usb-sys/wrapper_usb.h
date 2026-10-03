/* Wrapper header for bindgen - focused USB DDK substrate (usb-sys).
 *
 * Mirrors ntkrnl-sys/generate_bindings.sh's proven include environment (the
 * <ntoskrnl.h> master umbrella sets up ntstatus/ndk/wdm in the right order),
 * then appends the USB DDK headers. The bindgen allowlist (USB, USBD and URB
 * type prefixes) keeps the emitted surface to just the types usbd.sys needs:
 * URB plus the USB descriptors (usb.h/usb100.h via usbdi.h) plus the USBD info
 * structs (usbdlib.h), which are absent from ntkrnl-sys (that crate is built
 * from ntoskrnl.h only and pulls in no USB headers). */
#define _NTOSKRNL_
#define _NTSYSTEM_
#define _USE_DUMMY_PSEH 1
#define __REACTOS__
/* Skip intrin.h - compiler intrinsics not available in libclang */
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
#undef FORCEINLINE
#define FORCEINLINE
#include <ntoskrnl.h>
/* USB DDK surface (usbdi.h pulls usb.h + usb100.h + usbioctl.h) */
#include <usbdi.h>
#include <usbdlib.h>
