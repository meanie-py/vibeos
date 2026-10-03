/* Wrapper header for bindgen - hidparser INTERNAL types (parser.h).
 *
 * parser.h defines the parser's private structs (HID_PARSER_CONTEXT,
 * HID_COLLECTION, HID_REPORT, HID_REPORT_ITEM, GLOBAL_ITEM_STATE,
 * LOCAL_ITEM_STATE, USAGE_VALUE, SHORT_ITEM, LONG_ITEM, MAIN_ITEM_DATA,
 * ITEM_PREFIX) - not public DDK, so a focused bindgen into the crate. Public
 * HIDP_x types come from hid-sys; kernel types from ntkrnl-sys. Some structs are
 * pragma pack(1), so bindgen (not hand) keeps the layout exact. */
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
#include "parser.h"
