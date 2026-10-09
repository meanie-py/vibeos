/*
 * PROJECT:     ReactOS kernel-mode tests
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     ExFastMutex with the concurrent Count/Contention checks settled before they assert
 *
 * The same source as ExFastMutex.c, compiled with KMT_EXFASTMUTEX_BOUNDED: every
 * assertion is identical, but a thread that "should block" is given up to ~5 s to
 * reach the mutex's contended path before Count/Owner/Contention are checked, instead
 * of 50 ms. ExFastMutex keeps the original timing and is expected to flake those
 * checks intermittently under load; this one is expected to pass them. See the comment
 * above SettleMutex in ExFastMutex.c.
 */

#define KMT_EXFASTMUTEX_BOUNDED
#include "ExFastMutex.c"
