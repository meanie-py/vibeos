/* Wrapper header for bindgen - focused HID parser DDK substrate (hid-sys).
 *
 * Mirrors ntkrnl-sys/generate_bindings.sh's include environment (ntoskrnl.h
 * master umbrella), then appends the public HID parser DDK headers. The bindgen
 * allowlist (HIDP, HID, HIDD type prefixes plus HID usage consts) keeps the
 * emitted surface to the HID parser API the hidparser lib needs - HIDP_CAPS,
 * HIDP_VALUE_CAPS, HIDP_BUTTON_CAPS, HIDP_DATA, HIDP_PREPARSED_DATA,
 * HIDP_DEVICE_DESC, HIDP_REPORT_IDS, HIDP_STATUS_x, etc. - which are absent
 * from ntkrnl-sys (built from ntoskrnl.h only, no HID headers). */
#define _NTOSKRNL_
#define _NTSYSTEM_
#define _USE_DUMMY_PSEH 1
#define __REACTOS__
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
#undef FORCEINLINE
#define FORCEINLINE
#include <ntoskrnl.h>
#include <hidusage.h>
#include <hidpi.h>
#include <hidpddi.h>
