#ifndef _KMIXER_PCH_
#define _KMIXER_PCH_

#include <portcls.h>
/* lrintf() comes straight from C99 <math.h>. This used to be <float_cast.h>, a
 * PRIVATE header of the vendored libsamplerate tree -- which 0.2.0 deleted, because
 * upstream now relies on C99 math.h too. Do not reach back into that tree: its
 * float_cast.h also pulled in libsamplerate's config.h, whose kernel-mode shim
 * #defines printf to DbgPrint and exit() to __debugbreak() in whoever includes it. */
#include <math.h>

typedef struct
{
    KSDEVICE_HEADER KsDeviceHeader;


}KMIXER_DEVICE_EXT, *PKMIXER_DEVICE_EXT;

typedef struct
{
    KSPIN_LOCK Lock;


}SUM_NODE_CONTEXT, *PSUM_NODE_CONTEXT;


NTSTATUS
NTAPI
KMixAllocateDeviceHeader(
    IN PKMIXER_DEVICE_EXT DeviceExtension);

NTSTATUS
CreatePin(
    IN PIRP Irp);

#ifndef _M_IX86
#define KeSaveFloatingPointState(x) ((void)(x), STATUS_SUCCESS)
#define KeRestoreFloatingPointState(x) ((void)0)
#endif

#endif /* _KMIXER_PCH_ */
