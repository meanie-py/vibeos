/*
 * XInputGetAudioDeviceIds (xinput1_4 only; Wine leaves it a spec stub).
 *
 * The HID backend in ../xinput1_3/main.c never sees a headset, so a connected pad
 * reports none, which is the documented answer for a pad without one: ERROR_SUCCESS
 * with both IDs empty. A user index past XUSER_MAX_COUNT is ERROR_BAD_ARGUMENTS and a
 * disconnected slot ERROR_DEVICE_NOT_CONNECTED, as XInputGetCapabilities reports them.
 */

#include <stdarg.h>

#include "windef.h"
#include "winbase.h"
#include "winerror.h"
#include "xinput.h"

static void no_device(WCHAR *id, UINT *count)
{
    if (!count) return;
    if (id && *count) id[0] = 0;
    *count = 0;
}

DWORD WINAPI XInputGetAudioDeviceIds(DWORD index, WCHAR *render_id, UINT *render_count,
                                     WCHAR *capture_id, UINT *capture_count)
{
    XINPUT_CAPABILITIES caps;
    DWORD ret;

    if (index >= XUSER_MAX_COUNT) return ERROR_BAD_ARGUMENTS;
    if ((ret = XInputGetCapabilities(index, 0, &caps))) return ret;

    no_device(render_id, render_count);
    no_device(capture_id, capture_count);
    return ERROR_SUCCESS;
}
