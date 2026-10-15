/*
 * BLE Generic Attribute Profile (GATT) APIs -- no-Bluetooth-stack versions
 *
 * vibeos has no Bluetooth stack, BthLEEnum or GATT service device interface,
 * so no HANDLE an application holds can name a BLE device or GATT service.
 * Each function therefore performs Windows' argument validation (order and
 * HRESULTs taken from Wine 11 dlls/bluetoothapis/gatt.c where Wine implements
 * the function, mirrored for the siblings Wine leaves as stubs) and then fails
 * the way the IOCTL to a non-BLE handle fails on Windows:
 *   - handle is not an open handle      -> HRESULT_FROM_WIN32(ERROR_INVALID_HANDLE)
 *   - handle is open but not a BLE dev  -> HRESULT_FROM_WIN32(ERROR_INVALID_FUNCTION)
 *     (the driver behind a file/pipe/etc handle rejects the unknown IOCTL with
 *      STATUS_INVALID_DEVICE_REQUEST, which maps to ERROR_INVALID_FUNCTION)
 * The "no device" outcome is final, so the ERROR_MORE_DATA sizing protocol and
 * ERROR_INVALID_USER_BUFFER paths are never reached (nothing to enumerate).
 * The Win32-code-as-HRESULT convention follows the MS docs, which list the
 * returns as ERROR_* values (HRESULT_FROM_WIN32 applied, as Wine does).
 *
 * Licensed LGPL 2.1+ (derived argument checks from Wine, Copyright 2025-2026
 * Vibhav Pant).
 */

#include <stdarg.h>
#include <stddef.h>

#include <windef.h>
#include <winbase.h>
#include <winerror.h>

#include "wine/debug.h"

WINE_DEFAULT_DEBUG_CHANNEL(bluetoothapis);

/* ---- Declarations normally from <bthledef.h>/<bluetoothleapis.h>, which sdk/include
 * lacks. Layouts match the Windows SDK (they are only passed through, never
 * dereferenced beyond sizeof, here). Move into sdk/include/psdk if wanted. ---- */
#ifndef __BTHLEDEFS_H
typedef struct _BTH_LE_UUID
{
    BOOLEAN IsShortUuid;
    union { USHORT ShortUuid; GUID LongUuid; } Value;
} BTH_LE_UUID, *PBTH_LE_UUID;

typedef struct _BTH_LE_GATT_SERVICE
{
    BTH_LE_UUID ServiceUuid;
    USHORT AttributeHandle;
} BTH_LE_GATT_SERVICE, *PBTH_LE_GATT_SERVICE;

typedef struct _BTH_LE_GATT_CHARACTERISTIC
{
    USHORT ServiceHandle;
    BTH_LE_UUID CharacteristicUuid;
    USHORT AttributeHandle;
    USHORT CharacteristicValueHandle;
    BOOLEAN IsBroadcastable, IsReadable, IsWritable, IsWritableWithoutResponse,
            IsSignedWritable, IsNotifiable, IsIndicatable, HasExtendedProperties;
} BTH_LE_GATT_CHARACTERISTIC, *PBTH_LE_GATT_CHARACTERISTIC;

typedef struct _BTH_LE_GATT_CHARACTERISTIC_VALUE
{
    ULONG DataSize;
    UCHAR Data[1];
} BTH_LE_GATT_CHARACTERISTIC_VALUE, *PBTH_LE_GATT_CHARACTERISTIC_VALUE;

typedef enum _BTH_LE_GATT_DESCRIPTOR_TYPE
{
    CharacteristicExtendedProperties, CharacteristicUserDescription,
    ClientCharacteristicConfiguration, ServerCharacteristicConfiguration,
    CharacteristicFormat, CharacteristicAggregateFormat, CustomDescriptor
} BTH_LE_GATT_DESCRIPTOR_TYPE;

typedef struct _BTH_LE_GATT_DESCRIPTOR
{
    USHORT ServiceHandle;
    USHORT CharacteristicHandle;
    BTH_LE_GATT_DESCRIPTOR_TYPE DescriptorType;
    BTH_LE_UUID DescriptorUuid;
    USHORT AttributeHandle;
} BTH_LE_GATT_DESCRIPTOR, *PBTH_LE_GATT_DESCRIPTOR;

/* Real definition is a large union-bearing struct; only the pointer is used here. */
typedef struct _BTH_LE_GATT_DESCRIPTOR_VALUE BTH_LE_GATT_DESCRIPTOR_VALUE, *PBTH_LE_GATT_DESCRIPTOR_VALUE;

typedef enum _BTH_LE_GATT_EVENT_TYPE { CharacteristicValueChangedEvent } BTH_LE_GATT_EVENT_TYPE;
typedef PVOID BLUETOOTH_GATT_EVENT_HANDLE;
typedef UINT64 BTH_LE_GATT_RELIABLE_WRITE_CONTEXT;
typedef VOID (CALLBACK *PFNBLUETOOTH_GATT_EVENT_CALLBACK)(BTH_LE_GATT_EVENT_TYPE, PVOID, PVOID);
#endif

/* Discriminate a closed/garbage handle from an open non-BLE one, like the IOCTL would. */
static HRESULT gatt_no_such_device(HANDLE device)
{
    DWORD flags;
    if (device == INVALID_HANDLE_VALUE || !GetHandleInformation(device, &flags))
        return HRESULT_FROM_WIN32(ERROR_INVALID_HANDLE);
    return HRESULT_FROM_WIN32(ERROR_INVALID_FUNCTION);
}

/*********************************************************************
 *  BluetoothGATTGetServices
 */
HRESULT WINAPI BluetoothGATTGetServices(HANDLE le_device, USHORT count, BTH_LE_GATT_SERVICE *buf,
                                        USHORT *actual, ULONG flags)
{
    TRACE("(%p, %u, %p, %p, %#lx)\n", le_device, count, buf, actual, flags);

    if (!actual)
        return E_POINTER;
    if (!!buf != !!count)
        return E_INVALIDARG;
    return gatt_no_such_device(le_device);
}

/*********************************************************************
 *  BluetoothGATTGetCharacteristics
 */
HRESULT WINAPI BluetoothGATTGetCharacteristics(HANDLE device, BTH_LE_GATT_SERVICE *service, USHORT count,
                                               BTH_LE_GATT_CHARACTERISTIC *buf, USHORT *actual, ULONG flags)
{
    TRACE("(%p, %p, %u, %p, %p, %#lx)\n", device, service, count, buf, actual, flags);

    if (!actual)
        return E_POINTER;
    if (buf && !count)
        return E_INVALIDARG;
    return gatt_no_such_device(device);
}

/*********************************************************************
 *  BluetoothGATTGetDescriptors
 *
 * Wine leaves this a stub; checks mirror GetCharacteristics, plus the parent
 * characteristic is mandatory (the descriptors are enumerated relative to it).
 */
HRESULT WINAPI BluetoothGATTGetDescriptors(HANDLE device, BTH_LE_GATT_CHARACTERISTIC *characteristic,
                                           USHORT count, BTH_LE_GATT_DESCRIPTOR *buf, USHORT *actual,
                                           ULONG flags)
{
    TRACE("(%p, %p, %u, %p, %p, %#lx)\n", device, characteristic, count, buf, actual, flags);

    if (!actual)
        return E_POINTER;
    if (!characteristic || (buf && !count))
        return E_INVALIDARG;
    return gatt_no_such_device(device);
}

/*********************************************************************
 *  BluetoothGATTGetCharacteristicValue
 */
HRESULT WINAPI BluetoothGATTGetCharacteristicValue(HANDLE device, BTH_LE_GATT_CHARACTERISTIC *chrc, ULONG size,
                                                   BTH_LE_GATT_CHARACTERISTIC_VALUE *val, USHORT *actual,
                                                   ULONG flags)
{
    TRACE("(%p, %p, %lu, %p, %p, %#lx)\n", device, chrc, size, val, actual, flags);

    if (!device)
        return E_HANDLE;
    if (!chrc || !(size || val || actual) || (size && size < sizeof(*val)))
        return E_INVALIDARG;
    if (size && !val)
        return E_POINTER;
    return gatt_no_such_device(device);
}

/*********************************************************************
 *  BluetoothGATTSetCharacteristicValue
 *
 * Wine stub. Checks follow GetCharacteristicValue's style: handle, then the
 * mandatory characteristic and value pointers.
 */
HRESULT WINAPI BluetoothGATTSetCharacteristicValue(HANDLE device, BTH_LE_GATT_CHARACTERISTIC *chrc,
                                                   BTH_LE_GATT_CHARACTERISTIC_VALUE *val,
                                                   BTH_LE_GATT_RELIABLE_WRITE_CONTEXT reliable_write_ctx,
                                                   ULONG flags)
{
    TRACE("(%p, %p, %p, %s, %#lx)\n", device, chrc, val, wine_dbgstr_longlong(reliable_write_ctx), flags);

    if (!device)
        return E_HANDLE;
    if (!chrc || !val)
        return E_INVALIDARG;
    return gatt_no_such_device(device);
}

/*********************************************************************
 *  BluetoothGATTSetDescriptorValue
 */
HRESULT WINAPI BluetoothGATTSetDescriptorValue(HANDLE device, BTH_LE_GATT_DESCRIPTOR *descriptor,
                                               BTH_LE_GATT_DESCRIPTOR_VALUE *val, ULONG flags)
{
    TRACE("(%p, %p, %p, %#lx)\n", device, descriptor, val, flags);

    if (!device)
        return E_HANDLE;
    if (!descriptor || !val)
        return E_INVALIDARG;
    return gatt_no_such_device(device);
}

/*********************************************************************
 *  BluetoothGATTRegisterEvent
 *
 * Only CharacteristicValueChangedEvent exists (MS docs). On failure the out
 * handle is zeroed so a caller that ignores the HRESULT never unregisters junk.
 */
HRESULT WINAPI BluetoothGATTRegisterEvent(HANDLE service, BTH_LE_GATT_EVENT_TYPE type, PVOID event_param,
                                          PFNBLUETOOTH_GATT_EVENT_CALLBACK callback, PVOID context,
                                          BLUETOOTH_GATT_EVENT_HANDLE *event_handle, ULONG flags)
{
    TRACE("(%p, %d, %p, %p, %p, %p, %#lx)\n", service, type, event_param, callback, context,
          event_handle, flags);

    if (!event_handle)
        return E_POINTER;
    *event_handle = NULL;
    if (!service)
        return E_HANDLE;
    if (type != CharacteristicValueChangedEvent || !event_param || !callback)
        return E_INVALIDARG;
    return gatt_no_such_device(service);
}

/*********************************************************************
 *  BluetoothGATTUnregisterEvent
 *
 * No registration can ever have succeeded, so every event handle is invalid.
 */
HRESULT WINAPI BluetoothGATTUnregisterEvent(BLUETOOTH_GATT_EVENT_HANDLE event_handle, ULONG flags)
{
    TRACE("(%p, %#lx)\n", event_handle, flags);
    return E_INVALIDARG; /* == HRESULT_FROM_WIN32(ERROR_INVALID_PARAMETER) */
}
