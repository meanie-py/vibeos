/* dosfsck.h  -  Interface with vfatlib.h */
/* ReactOS glue, not dosfstools: lives beside vfatlib and reaches the vendored
   headers as "dosfstools/x.h" through the vendor/ include root. The prefix is NOT
   optional: vfatlib has its own common.h, and a bare "common.h" resolves to it. */

/* Redefine off_t as a 'long long' */
// typedef __int64 off_t;
#define off_t __int64

#include "dosfstools/common.h"
#include "dosfstools/fsck.fat.h"
#include "dosfstools/io.h"
#include "dosfstools/boot.h"
#include "dosfstools/check.h"
#include "dosfstools/fat.h"
#include "dosfstools/file.h"
#include "dosfstools/lfn.h"
