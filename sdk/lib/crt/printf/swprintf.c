/*
 * COPYRIGHT:       GNU GPL, see COPYING in the top level directory
 * PROJECT:         ReactOS crt library
 * FILE:            lib/sdk/crt/printf/swprintf.c
 * PURPOSE:         Implementation of swprintf
 * PROGRAMMER:      Timo Kreuzer
 */

/* upstream-sync 2026-09-20: upstream removed the tree-wide
 * -D_CRT_NON_CONFORMING_SWPRINTFS and fixed every call site, so <stdio.h> now always
 * declares the C99 3-arg swprintf. This TU IS the count-less engine behind msvcrt/ntdll's
 * export of that name, so it alone still needs the old declaration.
 *
 * Defined HERE rather than via set_source_files_properties: libcntpr is replayed by the
 * cmake-off cargo build from a SINGLE recorded flag set, and a per-source define breaks
 * that invariant outright (tools/check_committed_drift.py: "2 distinct flag sets across
 * sources"). A define in the source costs the build system nothing. */
#define _CRT_NON_CONFORMING_SWPRINTFS

#define _sxprintf swprintf
#define USE_COUNT 0
#define _UNICODE

#include "_sxprintf.c"
