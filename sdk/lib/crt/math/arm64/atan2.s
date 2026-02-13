/*
 * COPYRIGHT:         BSD - See COPYING.ARM in the top level directory
 * PROJECT:           ReactOS CRT library
 * PURPOSE:           Implementation of atan2
 * PROGRAMMER:        ReactOS ARM64 Team
 */

/* INCLUDES ******************************************************************/

#include <kxarm64.h>

/* CODE **********************************************************************/

    TEXTAREA

    LEAF_ENTRY atan2

    /* FIXME: Unimplemented - return 0.0 for now */
    fmov d0, xzr
    ret

    LEAF_END atan2

    END
/* EOF */
