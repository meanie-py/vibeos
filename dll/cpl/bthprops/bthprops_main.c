/*
 * bthprops.cpl radio-control exports.
 *
 * vibeos has no Bluetooth radio, so no HANDLE can ever come from
 * BluetoothFindFirstRadio. Windows defines the radio argument as: NULL means
 * "all local radios", a non-NULL value is one radio. With no radios present
 * the NULL form has nothing to act on, so it fails; a non-NULL value cannot
 * name a real radio, so it is reported as an invalid handle.
 * Error codes follow Wine's bluetoothapis enumeration path (no radio ->
 * ERROR_NO_MORE_ITEMS from the find; bad handle -> ERROR_INVALID_HANDLE).
 */

#include <stdarg.h>

#include "windef.h"
#include "winbase.h"
#include "winerror.h"

#include "wine/debug.h"

WINE_DEFAULT_DEBUG_CHANNEL(bthprops);

static BOOL bthprops_no_radio(HANDLE radio)
{
    SetLastError(radio ? ERROR_INVALID_HANDLE : ERROR_NO_MORE_ITEMS);
    return FALSE;
}

/***********************************************************************
 *           BluetoothEnableDiscovery (bthprops.@)
 */
BOOL WINAPI BluetoothEnableDiscovery(HANDLE radio, BOOL enabled)
{
    TRACE("(%p, %d)\n", radio, enabled);
    return bthprops_no_radio(radio);
}

/***********************************************************************
 *           BluetoothEnableIncomingConnections (bthprops.@)
 */
BOOL WINAPI BluetoothEnableIncomingConnections(HANDLE radio, BOOL enabled)
{
    TRACE("(%p, %d)\n", radio, enabled);
    return bthprops_no_radio(radio);
}

/***********************************************************************
 *           BluetoothIsConnectable (bthprops.@)
 */
BOOL WINAPI BluetoothIsConnectable(HANDLE radio)
{
    TRACE("(%p)\n", radio);
    return bthprops_no_radio(radio);
}
