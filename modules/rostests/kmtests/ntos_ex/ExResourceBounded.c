/*
 * PROJECT:     ReactOS kernel-mode tests
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     ExResource with the waiter-count checks settled before they assert
 *
 * The same source as ExResource.c, compiled with KMT_EXRESOURCE_BOUNDED: every
 * assertion is identical, but a thread that "should block" is given up to ~5 s to
 * reach the resource's wait before its waiter count is checked, instead of 10 ms.
 * ExResource keeps the original timing and is expected to fail those three checks
 * intermittently under load; this one is expected to pass them. See the comment
 * above WaitForWaiters in ExResource.c.
 */

#define KMT_EXRESOURCE_BOUNDED
#include "ExResource.c"
