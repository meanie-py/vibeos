
/* INCLUDES ******************************************************************/

/* ARM64 stack probing functions for ReactOS CRT */

#include <kxarm64.h>

/* CODE **********************************************************************/
    TEXTAREA

/*
 * __chkstk - Stack probe function
 * On ARM64, this is typically a no-op or simple implementation
 * as the stack probing model is different from x86
 */
    LEAF_ENTRY __chkstk
    ret
    LEAF_END __chkstk

/*
 * __alloca_probe - Alloca stack probe
 */
    LEAF_ENTRY __alloca_probe
    ret
    LEAF_END __alloca_probe

    END
/* EOF */
