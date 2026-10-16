/*
 * SetupAPI / CfgMgr32 device property (DEVPROPKEY) API
 *
 * SetupDiGetDevicePropertyW, SetupDiSetDevicePropertyW,
 * SetupDiGetDevicePropertyKeys, CM_Get_DevNode_PropertyW,
 * CM_Get_DevNode_Property_ExW, CM_Get_DevNode_Property_Keys(_Ex).
 *
 * Design
 * ------
 * Windows defines the DEVPKEY_Device_* keys with fmtid
 * {a45c254e-df1c-4efd-8020-67d146a850e0} and pid = SPDRP_xxx + 2.  Those keys
 * are served from the legacy registry-property store (SetupDiGetDeviceRegistry-
 * PropertyW or CM_Get_DevNode_Registry_Property_ExW, depending on the entry
 * point) and the REG_* value is re-typed as a DEVPROP_TYPE_*.  A few more keys
 * (InstanceId, NAME, the driver keys under fmtid {a8b865dd-...}) are synthesised.
 * Everything else is read from the device's "Properties" registry subkey using
 * Wine's layout:  Enum\<instance>\Properties\{fmtid}\<pid as %04X>, default
 * value, registry type = 0xffff0000 | DEVPROPTYPE.
 *
 * Both the SetupDi path and the CM path share one engine (dp_get / dp_list_keys)
 * and one key table (DpDeviceMap / DpDriverMap).
 *
 * Divergences from Windows (all honest, none faked):
 *  - The "Properties" subkey is a private store (Wine's design).  Keys that the
 *    table maps are served from the legacy store first and only fall back to
 *    "Properties" when the legacy value is absent, so a value written with
 *    SetupDiSetDevicePropertyW for such a key is visible but does not change
 *    the legacy value (Windows would).
 *  - The "Properties" fallback is only available for the local machine; a CM_
 *    call with a non-NULL hMachine serves only the table-mapped/synthesised
 *    keys and reports CR_NO_SUCH_VALUE for the rest.
 *  - SetupDiSetDevicePropertyW is not routed to the PnP manager: it only
 *    writes the registry store above.
 *  - Device-container, bus-reported and DEVPKEY_Device_ContainerId style keys
 *    are not synthesised (no data source here); they report ERROR_NOT_FOUND.
 */

#include "setupapi_private.h"

/* --------------------------------------------------------------------- */
/* Keys.  Local copies of the GUIDs so no devpkey.h/INITGUID dance is needed. */

static const GUID DpFmtidDevice   = {0xa45c254e, 0xdf1c, 0x4efd, {0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0}};
static const GUID DpFmtidInstance = {0x78c34fc8, 0x104a, 0x4aca, {0x9e, 0xa4, 0x52, 0x4d, 0x52, 0x99, 0x6e, 0x57}};
static const GUID DpFmtidDriver   = {0xa8b865dd, 0x2e3d, 0x4094, {0xad, 0x97, 0xe5, 0x93, 0xa7, 0x0c, 0x75, 0xd6}};
static const GUID DpFmtidName     = {0xb725f130, 0x47ef, 0x101a, {0xa5, 0xf1, 0x02, 0x60, 0x8c, 0x9e, 0xeb, 0xac}};

#define DP_PID_INSTANCEID 256
#define DP_PID_NAME       10

enum DpKind
{
    DPK_STRING,     /* REG_SZ            -> DEVPROP_TYPE_STRING */
    DPK_MULTISZ,    /* REG_MULTI_SZ      -> DEVPROP_TYPE_STRING_LIST */
    DPK_UINT32,     /* REG_DWORD         -> DEVPROP_TYPE_UINT32 */
    DPK_BOOL,       /* REG_DWORD         -> DEVPROP_TYPE_BOOLEAN (1 byte, -1 / 0) */
    DPK_GUID,       /* "{guid}" or 16 raw bytes -> DEVPROP_TYPE_GUID */
    DPK_BINARY,     /* REG_BINARY        -> DEVPROP_TYPE_BINARY */
    DPK_SD,         /* REG_BINARY        -> DEVPROP_TYPE_SECURITY_DESCRIPTOR */
    DPK_SDS,        /* SDDL string       -> DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING */
    DPK_FILETIME    /* 8 bytes binary    -> DEVPROP_TYPE_FILETIME */
};

struct DpDeviceMapEntry
{
    ULONG pid;      /* DEVPKEY_Device_xxx pid; always spdrp + 2 (verified against devpkey.h) */
    ULONG spdrp;    /* SPDRP_xxx */
    enum DpKind kind;
};

/* pid = SPDRP + 2 for every row; the SPDRP column is the authority. */
static const struct DpDeviceMapEntry DpDeviceMap[] =
{
    {  2, SPDRP_DEVICEDESC,                 DPK_STRING },
    {  3, SPDRP_HARDWAREID,                 DPK_MULTISZ },
    {  4, SPDRP_COMPATIBLEIDS,              DPK_MULTISZ },
    {  6, SPDRP_SERVICE,                    DPK_STRING },
    {  9, SPDRP_CLASS,                      DPK_STRING },
    { 10, SPDRP_CLASSGUID,                  DPK_GUID },
    { 11, SPDRP_DRIVER,                     DPK_STRING },
    { 12, SPDRP_CONFIGFLAGS,                DPK_UINT32 },
    { 13, SPDRP_MFG,                        DPK_STRING },
    { 14, SPDRP_FRIENDLYNAME,               DPK_STRING },
    { 15, SPDRP_LOCATION_INFORMATION,       DPK_STRING },
    { 16, SPDRP_PHYSICAL_DEVICE_OBJECT_NAME, DPK_STRING },
    { 17, SPDRP_CAPABILITIES,               DPK_UINT32 },
    { 18, SPDRP_UI_NUMBER,                  DPK_UINT32 },
    { 19, SPDRP_UPPERFILTERS,               DPK_MULTISZ },
    { 20, SPDRP_LOWERFILTERS,               DPK_MULTISZ },
    { 21, SPDRP_BUSTYPEGUID,                DPK_GUID },
    { 22, SPDRP_LEGACYBUSTYPE,              DPK_UINT32 },
    { 23, SPDRP_BUSNUMBER,                  DPK_UINT32 },
    { 24, SPDRP_ENUMERATOR_NAME,            DPK_STRING },
    { 25, SPDRP_SECURITY,                   DPK_SD },
    { 26, SPDRP_SECURITY_SDS,               DPK_SDS },
    { 27, SPDRP_DEVTYPE,                    DPK_UINT32 },
    { 28, SPDRP_EXCLUSIVE,                  DPK_BOOL },
    { 29, SPDRP_CHARACTERISTICS,            DPK_UINT32 },
    { 30, SPDRP_ADDRESS,                    DPK_UINT32 },
    { 31, SPDRP_UI_NUMBER_DESC_FORMAT,      DPK_STRING },
    { 32, SPDRP_DEVICE_POWER_DATA,          DPK_BINARY },
    { 33, SPDRP_REMOVAL_POLICY,             DPK_UINT32 },
    { 34, SPDRP_REMOVAL_POLICY_HW_DEFAULT,  DPK_UINT32 },
    { 35, SPDRP_REMOVAL_POLICY_OVERRIDE,    DPK_UINT32 },
    { 36, SPDRP_INSTALL_STATE,              DPK_UINT32 },
};

struct DpDriverMapEntry
{
    ULONG pid;              /* DEVPKEY_Device_Driver* pid under DpFmtidDriver */
    PCWSTR valueName;       /* value in the device's driver (software) key */
    enum DpKind kind;
};

static const struct DpDriverMapEntry DpDriverMap[] =
{
    { 2, L"DriverDateData",   DPK_FILETIME },   /* DEVPKEY_Device_DriverDate */
    { 3, L"DriverVersion",    DPK_STRING },     /* DEVPKEY_Device_DriverVersion */
    { 4, L"DriverDesc",       DPK_STRING },     /* DEVPKEY_Device_DriverDesc */
    { 5, L"InfPath",          DPK_STRING },     /* DEVPKEY_Device_DriverInfPath */
    { 6, L"InfSection",       DPK_STRING },     /* DEVPKEY_Device_DriverInfSection */
    { 7, L"InfSectionExt",    DPK_STRING },     /* DEVPKEY_Device_DriverInfSectionExt */
    { 8, L"MatchingDeviceId", DPK_STRING },     /* DEVPKEY_Device_MatchingDeviceId */
    { 9, L"ProviderName",     DPK_STRING },     /* DEVPKEY_Device_DriverProvider */
};

#define ARRAY_COUNT(a) (sizeof(a) / sizeof((a)[0]))

/* --------------------------------------------------------------------- */
/* Prototypes for exports that setupapi.h / cfgmgr32.h do not declare.     */

BOOL WINAPI SetupDiGetDevicePropertyKeys(HDEVINFO, PSP_DEVINFO_DATA, DEVPROPKEY *, DWORD, DWORD *, DWORD);
BOOL WINAPI SetupDiSetDevicePropertyW(HDEVINFO, PSP_DEVINFO_DATA, const DEVPROPKEY *, DEVPROPTYPE,
                                      const BYTE *, DWORD, DWORD);
CONFIGRET WINAPI CM_Get_DevNode_PropertyW(DEVINST, const DEVPROPKEY *, DEVPROPTYPE *, PBYTE, PULONG, ULONG);
CONFIGRET WINAPI CM_Get_DevNode_Property_ExW(DEVINST, const DEVPROPKEY *, DEVPROPTYPE *, PBYTE, PULONG,
                                             ULONG, HMACHINE);
CONFIGRET WINAPI CM_Get_DevNode_Property_Keys(DEVINST, DEVPROPKEY *, PULONG, ULONG);
CONFIGRET WINAPI CM_Get_DevNode_Property_Keys_Ex(DEVINST, DEVPROPKEY *, PULONG, ULONG, HMACHINE);

/* --------------------------------------------------------------------- */
/* Device context shared by the SetupDi and CM entry points.              */

typedef struct _DPCTX
{
    BOOL setupdi;               /* TRUE: SetupDi entry point, FALSE: CM entry point */
    HDEVINFO hDevInfo;          /* SetupDi path */
    PSP_DEVINFO_DATA pData;     /* SetupDi path */
    struct DeviceInfo *devInfo; /* SetupDi path */
    struct DeviceInfoSet *set;  /* SetupDi path */
    DEVINST devInst;
    HMACHINE hMachine;
    PCWSTR instanceId;          /* lazily resolved on the CM path */
    WCHAR idBuf[MAX_DEVICE_ID_LEN];
} DPCTX;

static DWORD dp_cr_to_error(CONFIGRET cr)
{
    switch (cr)
    {
        case CR_SUCCESS:            return ERROR_SUCCESS;
        case CR_BUFFER_SMALL:       return ERROR_INSUFFICIENT_BUFFER;
        case CR_NO_SUCH_VALUE:
        case CR_INVALID_PROPERTY:   return ERROR_NOT_FOUND;
        case CR_INVALID_DEVNODE:    return ERROR_NO_SUCH_DEVINST;
        case CR_OUT_OF_MEMORY:      return ERROR_NOT_ENOUGH_MEMORY;
        case CR_INVALID_POINTER:
        case CR_INVALID_FLAG:
        case CR_INVALID_DATA:       return ERROR_INVALID_PARAMETER;
        case CR_ACCESS_DENIED:      return ERROR_ACCESS_DENIED;
        default:                    return ERROR_GEN_FAILURE;
    }
}

static CONFIGRET dp_error_to_cr(DWORD err)
{
    switch (err)
    {
        case ERROR_SUCCESS:             return CR_SUCCESS;
        case ERROR_INSUFFICIENT_BUFFER: return CR_BUFFER_SMALL;
        case ERROR_NOT_FOUND:           return CR_NO_SUCH_VALUE;
        case ERROR_NO_SUCH_DEVINST:     return CR_INVALID_DEVNODE;
        case ERROR_NOT_ENOUGH_MEMORY:   return CR_OUT_OF_MEMORY;
        case ERROR_ACCESS_DENIED:       return CR_ACCESS_DENIED;
        default:                        return CR_FAILURE;
    }
}

/* Resolve the device instance id (Enum-relative path). */
static DWORD dp_get_instance_id(DPCTX *c, PCWSTR *id)
{
    if (!c->instanceId)
    {
        CONFIGRET cr = CM_Get_Device_ID_ExW(c->devInst, c->idBuf, MAX_DEVICE_ID_LEN, 0, c->hMachine);
        if (cr != CR_SUCCESS)
            return dp_cr_to_error(cr);
        c->instanceId = c->idBuf;
    }
    *id = c->instanceId;
    return ERROR_SUCCESS;
}

/* Map "this value/property is simply not there" errors to ERROR_NOT_FOUND. */
static DWORD dp_normalize_missing(DWORD err)
{
    switch (err)
    {
        case ERROR_FILE_NOT_FOUND:
        case ERROR_PATH_NOT_FOUND:
        case ERROR_INVALID_DATA:
        case ERROR_INVALID_REG_PROPERTY:
            return ERROR_NOT_FOUND;
        default:
            return err;
    }
}

/* Read a legacy registry property (SPDRP_xxx) with caller-provided buffer.
 * On ERROR_SUCCESS / ERROR_INSUFFICIENT_BUFFER *size is the required size. */
static DWORD dp_read_reg(DPCTX *c, ULONG spdrp, ULONG *regType, BYTE *buf, ULONG *size)
{
    ULONG type = REG_NONE;

    if (c->setupdi)
    {
        DWORD req = 0, err;

        if (SetupDiGetDeviceRegistryPropertyW(c->hDevInfo, c->pData, spdrp, &type, buf, *size, &req))
        {
            *regType = type;
            *size = req;
            return ERROR_SUCCESS;
        }
        err = GetLastError();
        if (err == ERROR_INSUFFICIENT_BUFFER)
        {
            *regType = type;
            *size = req;
        }
        return dp_normalize_missing(err);
    }
    else
    {
        /* CM_Get_DevNode_Registry_Property_ExW rejects a zero length: callers always pass >= 1. */
        ULONG len = *size;
        CONFIGRET cr = CM_Get_DevNode_Registry_Property_ExW(c->devInst, spdrp + (CM_DRP_DEVICEDESC - SPDRP_DEVICEDESC),
                                                            &type, buf, &len, 0, c->hMachine);
        if (cr == CR_SUCCESS || cr == CR_BUFFER_SMALL)
        {
            *regType = type;
            *size = len;
        }
        return dp_cr_to_error(cr);
    }
}

/* Read a legacy property into a heap block (4 zero bytes of padding past *len
 * so that string conversions can safely terminate). */
static DWORD dp_read_reg_alloc(DPCTX *c, ULONG spdrp, ULONG *regType, BYTE **data, ULONG *len)
{
    ULONG cap = 128, i;
    BYTE *p;

    for (i = 0; i < 4; i++)
    {
        ULONG size = cap;
        DWORD err;

        p = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, cap + 4);
        if (!p)
            return ERROR_NOT_ENOUGH_MEMORY;
        err = dp_read_reg(c, spdrp, regType, p, &size);
        if (err == ERROR_SUCCESS)
        {
            *data = p;
            *len = size;
            return ERROR_SUCCESS;
        }
        HeapFree(GetProcessHeap(), 0, p);
        if (err != ERROR_INSUFFICIENT_BUFFER || size <= cap)
            return err == ERROR_INSUFFICIENT_BUFFER ? ERROR_GEN_FAILURE : err;
        cap = size;
    }
    return ERROR_GEN_FAILURE;
}

/* --------------------------------------------------------------------- */
/* Output helpers                                                          */

static DWORD dp_emit(DEVPROPTYPE type, const void *data, DWORD len, DEVPROPTYPE *outType,
                     BYTE *buf, DWORD bufSize, DWORD *req)
{
    *outType = type;
    if (req)
        *req = len;
    if (len > bufSize)
        return ERROR_INSUFFICIENT_BUFFER;
    if (len)
        memcpy(buf, data, len);
    return ERROR_SUCCESS;
}

static BOOL dp_parse_guid(const WCHAR *s, GUID *guid)
{
    WCHAR tmp[37];

    if (lstrlenW(s) < 38 || s[0] != '{' || s[37] != '}')
        return FALSE;
    memcpy(tmp, s + 1, 36 * sizeof(WCHAR));
    tmp[36] = 0;
    return UuidFromStringW(tmp, guid) == RPC_S_OK;
}

static void dp_guid_to_string(const GUID *guid, WCHAR *out)
{
    sprintfW(out, L"{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
             guid->Data1, guid->Data2, guid->Data3,
             guid->Data4[0], guid->Data4[1], guid->Data4[2], guid->Data4[3],
             guid->Data4[4], guid->Data4[5], guid->Data4[6], guid->Data4[7]);
}

/* Convert a raw registry value to the DEVPROP representation for 'kind' and
 * emit it.  'data' has 4 bytes of zero padding past 'len'.  Returns
 * ERROR_INVALID_DATA if the stored value cannot be represented as that kind. */
static DWORD dp_convert(enum DpKind kind, ULONG regType, BYTE *data, ULONG len,
                        DEVPROPTYPE *outType, BYTE *buf, DWORD bufSize, DWORD *req)
{
    switch (kind)
    {
        case DPK_STRING:
        case DPK_SDS:
        {
            WCHAR *s = (WCHAR *)data;
            ULONG n = len / sizeof(WCHAR);

            if (regType != REG_SZ && regType != REG_EXPAND_SZ)
                return ERROR_INVALID_DATA;
            if (!n || s[n - 1])
                n++;                                /* add the NUL the store lacked */
            return dp_emit(kind == DPK_STRING ? DEVPROP_TYPE_STRING : DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING,
                           s, n * sizeof(WCHAR), outType, buf, bufSize, req);
        }
        case DPK_MULTISZ:
        {
            WCHAR *s = (WCHAR *)data;
            ULONG n = len / sizeof(WCHAR);

            if (regType != REG_MULTI_SZ)
                return ERROR_INVALID_DATA;
            if (!n)
                n = 2;                              /* empty list = two NULs */
            else
            {
                if (s[n - 1])
                    n++;                            /* terminate the last string */
                if (n < 2 || s[n - 2])
                    n++;                            /* list terminator */
            }
            return dp_emit(DEVPROP_TYPE_STRING_LIST, s, n * sizeof(WCHAR), outType, buf, bufSize, req);
        }
        case DPK_UINT32:
        case DPK_BOOL:
        {
            DWORD v;

            if (!(regType == REG_DWORD || regType == REG_BINARY) || len < sizeof(DWORD))
                return ERROR_INVALID_DATA;
            memcpy(&v, data, sizeof(v));
            if (kind == DPK_UINT32)
                return dp_emit(DEVPROP_TYPE_UINT32, &v, sizeof(v), outType, buf, bufSize, req);
            else
            {
                DEVPROP_BOOLEAN b = v ? DEVPROP_TRUE : DEVPROP_FALSE;
                return dp_emit(DEVPROP_TYPE_BOOLEAN, &b, sizeof(b), outType, buf, bufSize, req);
            }
        }
        case DPK_GUID:
        {
            GUID g;

            if ((regType == REG_SZ || regType == REG_EXPAND_SZ) && dp_parse_guid((WCHAR *)data, &g))
                ;
            else if (regType == REG_BINARY && len == sizeof(GUID))
                memcpy(&g, data, sizeof(g));
            else
                return ERROR_INVALID_DATA;
            return dp_emit(DEVPROP_TYPE_GUID, &g, sizeof(g), outType, buf, bufSize, req);
        }
        case DPK_BINARY:
            return dp_emit(DEVPROP_TYPE_BINARY, data, len, outType, buf, bufSize, req);
        case DPK_SD:
            return dp_emit(DEVPROP_TYPE_SECURITY_DESCRIPTOR, data, len, outType, buf, bufSize, req);
        case DPK_FILETIME:
            if (regType != REG_BINARY || len != sizeof(FILETIME))
                return ERROR_INVALID_DATA;
            return dp_emit(DEVPROP_TYPE_FILETIME, data, len, outType, buf, bufSize, req);
    }
    return ERROR_INVALID_DATA;
}

/* --------------------------------------------------------------------- */
/* Sources                                                                 */

static DWORD dp_get_mapped(DPCTX *c, const struct DpDeviceMapEntry *m, DEVPROPTYPE *type,
                           BYTE *buf, DWORD bufSize, DWORD *req)
{
    ULONG regType = REG_NONE, len = 0;
    BYTE *data = NULL;
    DWORD err;

    err = dp_read_reg_alloc(c, m->spdrp, &regType, &data, &len);
    if (err != ERROR_SUCCESS)
        return err;
    err = dp_convert(m->kind, regType, data, len, type, buf, bufSize, req);
    HeapFree(GetProcessHeap(), 0, data);
    return err == ERROR_INVALID_DATA ? ERROR_NOT_FOUND : err;
}

static HKEY dp_open_driver_key(DPCTX *c)
{
    HKEY key = NULL;

    if (c->setupdi)
    {
        key = SETUPDI_OpenDrvKey(c->set->HKLM, c->devInfo, KEY_QUERY_VALUE);
        return key == INVALID_HANDLE_VALUE ? NULL : key;
    }
    if (CM_Open_DevNode_Key_Ex(c->devInst, KEY_QUERY_VALUE, 0, RegDisposition_OpenExisting,
                               &key, CM_REGISTRY_SOFTWARE, c->hMachine) != CR_SUCCESS)
        return NULL;
    return key;
}

static DWORD dp_get_driver(DPCTX *c, const struct DpDriverMapEntry *m, DEVPROPTYPE *type,
                           BYTE *buf, DWORD bufSize, DWORD *req)
{
    HKEY key;
    DWORD regType = REG_NONE, size = 0, err;
    BYTE *data;

    /* No driver key (driver not installed) or no such value: not found. */
    key = dp_open_driver_key(c);
    if (!key)
        return ERROR_NOT_FOUND;
    err = RegQueryValueExW(key, m->valueName, NULL, &regType, NULL, &size);
    if (err != ERROR_SUCCESS)
    {
        RegCloseKey(key);
        return dp_normalize_missing(err);
    }
    data = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size + 4);
    if (!data)
    {
        RegCloseKey(key);
        return ERROR_NOT_ENOUGH_MEMORY;
    }
    err = RegQueryValueExW(key, m->valueName, NULL, &regType, data, &size);
    RegCloseKey(key);
    if (err == ERROR_SUCCESS)
    {
        err = dp_convert(m->kind, regType, data, size, type, buf, bufSize, req);
        if (err == ERROR_INVALID_DATA)
            err = ERROR_NOT_FOUND;
    }
    else
        err = dp_normalize_missing(err);
    HeapFree(GetProcessHeap(), 0, data);
    return err;
}

/* Open Enum\<instance> under the right HKLM; local machine only for the CM path. */
static DWORD dp_open_enum_key(DPCTX *c, REGSAM sam, HKEY *key)
{
    WCHAR path[sizeof(REGSTR_PATH_SYSTEMENUM) / sizeof(WCHAR) + MAX_DEVICE_ID_LEN + 2];
    PCWSTR id;
    HKEY hklm;
    DWORD err;

    if (c->setupdi)
        hklm = c->set->HKLM;
    else if (c->hMachine == NULL)
        hklm = HKEY_LOCAL_MACHINE;
    else
        return ERROR_NOT_FOUND;                 /* see header: no remote registry here */

    err = dp_get_instance_id(c, &id);
    if (err)
        return err;
    lstrcpyW(path, REGSTR_PATH_SYSTEMENUM);
    lstrcatW(path, L"\\");
    lstrcatW(path, id);
    err = RegOpenKeyExW(hklm, path, 0, sam, key);
    return err == ERROR_FILE_NOT_FOUND ? ERROR_NO_SUCH_DEVINST : err;
}

/* Wine's "Properties" subkey store. */
static DWORD dp_get_registry(DPCTX *c, const DEVPROPKEY *key, DEVPROPTYPE *type,
                             BYTE *buf, DWORD bufSize, DWORD *req)
{
    WCHAR path[11 + 38 + 5 + 1];
    HKEY dev, prop;
    DWORD size = 0, valType = 0, err;

    err = dp_open_enum_key(c, KEY_QUERY_VALUE, &dev);
    if (err == ERROR_SUCCESS)
    {
        lstrcpyW(path, L"Properties\\");
        dp_guid_to_string(&key->fmtid, path + 11);
        sprintfW(path + 49, L"\\%04X", key->pid);
        err = RegOpenKeyExW(dev, path, 0, KEY_QUERY_VALUE, &prop);
        RegCloseKey(dev);
        if (err == ERROR_SUCCESS)
        {
            size = bufSize;
            err = RegQueryValueExW(prop, NULL, NULL, &valType, buf, &size);
            RegCloseKey(prop);
        }
    }

    switch (err)
    {
        case ERROR_SUCCESS:
        case ERROR_MORE_DATA:
            *type = valType & 0xffff;
            if (req)
                *req = size;
            return (err == ERROR_MORE_DATA || (!buf && size)) ? ERROR_INSUFFICIENT_BUFFER : ERROR_SUCCESS;
        case ERROR_FILE_NOT_FOUND:
        case ERROR_PATH_NOT_FOUND:
            err = ERROR_NOT_FOUND;
            /* fall through */
        default:
            *type = DEVPROP_TYPE_EMPTY;
            if (req)
                *req = 0;
            return err;
    }
}

static const struct DpDeviceMapEntry *dp_find_device_map(ULONG pid)
{
    ULONG i;

    for (i = 0; i < ARRAY_COUNT(DpDeviceMap); i++)
        if (DpDeviceMap[i].pid == pid)
            return &DpDeviceMap[i];
    return NULL;
}

static const struct DpDriverMapEntry *dp_find_driver_map(ULONG pid)
{
    ULONG i;

    for (i = 0; i < ARRAY_COUNT(DpDriverMap); i++)
        if (DpDriverMap[i].pid == pid)
            return &DpDriverMap[i];
    return NULL;
}

/* The engine.  Arguments are already validated.  Returns a Win32 error. */
static DWORD dp_get(DPCTX *c, const DEVPROPKEY *key, DEVPROPTYPE *type, BYTE *buf, DWORD bufSize, DWORD *req)
{
    DWORD err = ERROR_NOT_FOUND;

    if (IsEqualGUID(&key->fmtid, &DpFmtidInstance) && key->pid == DP_PID_INSTANCEID)
    {
        PCWSTR id;

        err = dp_get_instance_id(c, &id);
        if (err)
            return err;
        return dp_emit(DEVPROP_TYPE_STRING, id, (lstrlenW(id) + 1) * sizeof(WCHAR), type, buf, bufSize, req);
    }
    if (IsEqualGUID(&key->fmtid, &DpFmtidName) && key->pid == DP_PID_NAME)
    {
        /* DEVPKEY_NAME: friendly name, else the device description. */
        err = dp_get_mapped(c, dp_find_device_map(12), type, buf, bufSize, req);
        if (err == ERROR_NOT_FOUND)
            err = dp_get_mapped(c, dp_find_device_map(2), type, buf, bufSize, req);
    }
    else if (IsEqualGUID(&key->fmtid, &DpFmtidDevice))
    {
        const struct DpDeviceMapEntry *m = dp_find_device_map(key->pid);

        if (m)
            err = dp_get_mapped(c, m, type, buf, bufSize, req);
    }
    else if (IsEqualGUID(&key->fmtid, &DpFmtidDriver))
    {
        const struct DpDriverMapEntry *m = dp_find_driver_map(key->pid);

        if (m)
            err = dp_get_driver(c, m, type, buf, bufSize, req);
    }

    if (err != ERROR_NOT_FOUND)
        return err;
    return dp_get_registry(c, key, type, buf, bufSize, req);
}

/* --------------------------------------------------------------------- */
/* Key enumeration                                                         */

typedef struct _DPKEYS
{
    DEVPROPKEY *k;
    DWORD n, cap;
    DWORD err;
} DPKEYS;

static void dpk_add(DPKEYS *l, const GUID *fmtid, DWORD pid)
{
    DWORD i;

    if (l->err)
        return;
    for (i = 0; i < l->n; i++)
        if (l->k[i].pid == pid && IsEqualGUID(&l->k[i].fmtid, fmtid))
            return;
    if (l->n == l->cap)
    {
        DWORD cap = l->cap ? l->cap * 2 : 64;
        DEVPROPKEY *p = l->k ? HeapReAlloc(GetProcessHeap(), 0, l->k, cap * sizeof(*p))
                             : HeapAlloc(GetProcessHeap(), 0, cap * sizeof(*p));
        if (!p)
        {
            l->err = ERROR_NOT_ENOUGH_MEMORY;
            return;
        }
        l->k = p;
        l->cap = cap;
    }
    l->k[l->n].fmtid = *fmtid;
    l->k[l->n].pid = pid;
    l->n++;
}

static BOOL dp_probe(DPCTX *c, const GUID *fmtid, DWORD pid)
{
    DEVPROPKEY key;
    DEVPROPTYPE type;
    BYTE tmp[8];
    DWORD err;

    key.fmtid = *fmtid;
    key.pid = pid;
    err = dp_get(c, &key, &type, tmp, 0, NULL);     /* size 0: existence check only */
    return err == ERROR_SUCCESS || err == ERROR_INSUFFICIENT_BUFFER;
}

/* Keys present in the "Properties" registry store. */
static void dp_list_registry_keys(DPCTX *c, DPKEYS *l)
{
    HKEY dev, props;
    DWORD i;

    if (dp_open_enum_key(c, KEY_READ, &dev))
        return;
    if (RegOpenKeyExW(dev, L"Properties", 0, KEY_ENUMERATE_SUB_KEYS, &props))
    {
        RegCloseKey(dev);
        return;
    }
    RegCloseKey(dev);

    for (i = 0; !l->err; i++)
    {
        WCHAR name[39];
        DWORD len = ARRAY_COUNT(name), j;
        HKEY fmt;
        GUID guid;

        if (RegEnumKeyExW(props, i, name, &len, NULL, NULL, NULL, NULL))
            break;
        if (!dp_parse_guid(name, &guid))
            continue;
        if (RegOpenKeyExW(props, name, 0, KEY_ENUMERATE_SUB_KEYS, &fmt))
            continue;
        for (j = 0; !l->err; j++)
        {
            WCHAR pidName[6];
            DWORD plen = ARRAY_COUNT(pidName);

            if (RegEnumKeyExW(fmt, j, pidName, &plen, NULL, NULL, NULL, NULL))
                break;
            dpk_add(l, &guid, strtoulW(pidName, NULL, 16));
        }
        RegCloseKey(fmt);
    }
    RegCloseKey(props);
}

/* Build the list of keys we can serve for the device.  Caller frees l->k. */
static DWORD dp_list_keys(DPCTX *c, DPKEYS *l)
{
    ULONG i;
    PCWSTR id;
    DWORD err;

    memset(l, 0, sizeof(*l));

    /* A bad devnode must fail, not yield an empty list. */
    err = dp_get_instance_id(c, &id);
    if (err)
        return err;
    dpk_add(l, &DpFmtidInstance, DP_PID_INSTANCEID);

    for (i = 0; i < ARRAY_COUNT(DpDeviceMap); i++)
        if (dp_probe(c, &DpFmtidDevice, DpDeviceMap[i].pid))
            dpk_add(l, &DpFmtidDevice, DpDeviceMap[i].pid);
    for (i = 0; i < ARRAY_COUNT(DpDriverMap); i++)
        if (dp_probe(c, &DpFmtidDriver, DpDriverMap[i].pid))
            dpk_add(l, &DpFmtidDriver, DpDriverMap[i].pid);
    dp_list_registry_keys(c, l);

    if (l->err)
    {
        HeapFree(GetProcessHeap(), 0, l->k);
        l->k = NULL;
    }
    return l->err;
}

static DWORD dp_get_keys(DPCTX *c, DEVPROPKEY *out, DWORD outLen, DWORD *required)
{
    DPKEYS l;
    DWORD err = dp_list_keys(c, &l);

    if (err)
        return err;
    if (required)
        *required = l.n;
    if (l.n > outLen)
        err = ERROR_INSUFFICIENT_BUFFER;
    else if (l.n)
        memcpy(out, l.k, l.n * sizeof(*out));
    HeapFree(GetProcessHeap(), 0, l.k);
    return err;
}

/* --------------------------------------------------------------------- */
/* SetupDi entry points                                                    */

static BOOL dp_init_setupdi(DPCTX *c, HDEVINFO hDevInfo, PSP_DEVINFO_DATA data)
{
    struct DeviceInfoSet *set = (struct DeviceInfoSet *)hDevInfo;
    struct DeviceInfo *devInfo;

    if (!hDevInfo || hDevInfo == INVALID_HANDLE_VALUE || set->magic != SETUP_DEVICE_INFO_SET_MAGIC)
    {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    if (!data || data->cbSize != sizeof(SP_DEVINFO_DATA) || !data->Reserved)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    devInfo = (struct DeviceInfo *)data->Reserved;
    if (devInfo->set != set)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    memset(c, 0, sizeof(*c));
    c->setupdi = TRUE;
    c->hDevInfo = hDevInfo;
    c->pData = data;
    c->devInfo = devInfo;
    c->set = set;
    c->devInst = devInfo->dnDevInst;
    c->hMachine = set->hMachine;
    c->instanceId = devInfo->instanceId;
    return TRUE;
}

/***********************************************************************
 *		SetupDiGetDevicePropertyW (SETUPAPI.@)
 *
 * Error order follows Wine (whose tests are derived from Windows): device
 * validity first, then NULL key -> ERROR_INVALID_DATA, NULL type or
 * NULL buffer with nonzero size -> ERROR_INVALID_USER_BUFFER, Flags ->
 * ERROR_INVALID_FLAGS.
 */
BOOL WINAPI SetupDiGetDevicePropertyW(
    HDEVINFO DeviceInfoSet,
    PSP_DEVINFO_DATA DeviceInfoData,
    const DEVPROPKEY *PropertyKey,
    DEVPROPTYPE *PropertyType,
    PBYTE PropertyBuffer,
    DWORD PropertyBufferSize,
    PDWORD RequiredSize,
    DWORD Flags)
{
    DPCTX ctx;
    DWORD err;

    TRACE("%p %p %p %p %p %lu %p %lx\n", DeviceInfoSet, DeviceInfoData, PropertyKey, PropertyType,
          PropertyBuffer, PropertyBufferSize, RequiredSize, Flags);

    if (!dp_init_setupdi(&ctx, DeviceInfoSet, DeviceInfoData))
        return FALSE;
    if (!PropertyKey)
    {
        SetLastError(ERROR_INVALID_DATA);
        return FALSE;
    }
    if (!PropertyType || (!PropertyBuffer && PropertyBufferSize))
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE;
    }
    if (Flags)
    {
        SetLastError(ERROR_INVALID_FLAGS);
        return FALSE;
    }

    err = dp_get(&ctx, PropertyKey, PropertyType, PropertyBuffer, PropertyBufferSize, RequiredSize);
    SetLastError(err);
    return err == ERROR_SUCCESS;
}

/***********************************************************************
 *		SetupDiGetDevicePropertyKeys (SETUPAPI.@)
 *
 * Lists InstanceId, every table-mapped key whose legacy property exists, the
 * driver keys of an installed driver, and every key in the "Properties"
 * registry store.
 */
BOOL WINAPI SetupDiGetDevicePropertyKeys(
    HDEVINFO DeviceInfoSet,
    PSP_DEVINFO_DATA DeviceInfoData,
    DEVPROPKEY *PropertyKeyArray,
    DWORD PropertyKeyCount,
    PDWORD RequiredPropertyKeyCount,
    DWORD Flags)
{
    DPCTX ctx;
    DWORD err;

    TRACE("%p %p %p %lu %p %lx\n", DeviceInfoSet, DeviceInfoData, PropertyKeyArray, PropertyKeyCount,
          RequiredPropertyKeyCount, Flags);

    if (Flags)
    {
        SetLastError(ERROR_INVALID_FLAGS);
        return FALSE;
    }
    if (!PropertyKeyArray && PropertyKeyCount)
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE;
    }
    if (!dp_init_setupdi(&ctx, DeviceInfoSet, DeviceInfoData))
        return FALSE;

    err = dp_get_keys(&ctx, PropertyKeyArray, PropertyKeyCount, RequiredPropertyKeyCount);
    SetLastError(err);
    return err == ERROR_SUCCESS;
}

static BOOL dp_is_valid_type(DEVPROPTYPE t)
{
    DWORD type = t & DEVPROP_MASK_TYPE;
    DWORD mod = t & DEVPROP_MASK_TYPEMOD;

    if (type > MAX_DEVPROP_TYPE || mod > MAX_DEVPROP_TYPEMOD)
        return FALSE;
    if (mod == DEVPROP_TYPEMOD_ARRAY &&
        (type == DEVPROP_TYPE_EMPTY || type == DEVPROP_TYPE_NULL || type == DEVPROP_TYPE_STRING ||
         type == DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING))
        return FALSE;
    if (mod == DEVPROP_TYPEMOD_LIST &&
        !(type == DEVPROP_TYPE_STRING || type == DEVPROP_TYPE_SECURITY_DESCRIPTOR_STRING))
        return FALSE;
    return TRUE;
}

/***********************************************************************
 *		SetupDiSetDevicePropertyW (SETUPAPI.@)
 *
 * Writes the private "Properties" registry store (Wine's design); it does not
 * reach the PnP manager and does not change the legacy SPDRP values.
 * DEVPROP_TYPE_EMPTY deletes the property, DEVPROP_TYPE_NULL clears its value.
 */
BOOL WINAPI SetupDiSetDevicePropertyW(
    HDEVINFO DeviceInfoSet,
    PSP_DEVINFO_DATA DeviceInfoData,
    const DEVPROPKEY *PropertyKey,
    DEVPROPTYPE PropertyType,
    const BYTE *PropertyBuffer,
    DWORD PropertyBufferSize,
    DWORD Flags)
{
    DPCTX ctx;
    HKEY dev, props, prop;
    WCHAR path[38 + 5 + 1];
    DWORD err;
    BOOL empty;

    TRACE("%p %p %p %lx %p %lu %lx\n", DeviceInfoSet, DeviceInfoData, PropertyKey, PropertyType,
          PropertyBuffer, PropertyBufferSize, Flags);

    if (!dp_init_setupdi(&ctx, DeviceInfoSet, DeviceInfoData))
        return FALSE;
    if (PropertyBufferSize && !PropertyBuffer)
    {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE;
    }
    empty = (PropertyType == DEVPROP_TYPE_EMPTY || PropertyType == DEVPROP_TYPE_NULL);
    if (!PropertyKey || !dp_is_valid_type(PropertyType) ||
        (!(PropertyBuffer && PropertyBufferSize) && !empty) ||
        (PropertyBuffer && PropertyBufferSize && empty))
    {
        SetLastError(ERROR_INVALID_DATA);
        return FALSE;
    }
    if (Flags)
    {
        SetLastError(ERROR_INVALID_FLAGS);
        return FALSE;
    }

    err = dp_open_enum_key(&ctx, KEY_READ | KEY_WRITE, &dev);
    if (err == ERROR_SUCCESS)
    {
        err = RegCreateKeyExW(dev, L"Properties", 0, NULL, 0, KEY_READ | KEY_WRITE, NULL, &props, NULL);
        RegCloseKey(dev);
    }
    if (err)
    {
        SetLastError(err);
        return FALSE;
    }

    dp_guid_to_string(&PropertyKey->fmtid, path);
    sprintfW(path + 38, L"\\%04X", PropertyKey->pid);

    switch (PropertyType)
    {
        case DEVPROP_TYPE_EMPTY:
            err = RegDeleteKeyW(props, path);
            break;
        case DEVPROP_TYPE_NULL:
            err = RegOpenKeyExW(props, path, 0, KEY_SET_VALUE, &prop);
            if (err == ERROR_SUCCESS)
            {
                err = RegDeleteValueW(prop, NULL);
                RegCloseKey(prop);
            }
            break;
        default:
            err = RegCreateKeyExW(props, path, 0, NULL, 0, KEY_READ | KEY_WRITE, NULL, &prop, NULL);
            if (err == ERROR_SUCCESS)
            {
                err = RegSetValueExW(prop, NULL, 0, 0xffff0000 | (0xffff & PropertyType),
                                     PropertyBuffer, PropertyBufferSize);
                RegCloseKey(prop);
            }
            break;
    }
    RegCloseKey(props);

    if (err == ERROR_FILE_NOT_FOUND)
        err = ERROR_NOT_FOUND;
    SetLastError(err);
    return err == ERROR_SUCCESS;
}

/* --------------------------------------------------------------------- */
/* CfgMgr32 entry points                                                   */

static void dp_init_cm(DPCTX *c, DEVINST devInst, HMACHINE hMachine)
{
    memset(c, 0, sizeof(*c));
    c->devInst = devInst;
    c->hMachine = hMachine;
}

/***********************************************************************
 * CM_Get_DevNode_Property_ExW [SETUPAPI.@]
 */
CONFIGRET WINAPI CM_Get_DevNode_Property_ExW(
    DEVINST dnDevInst,
    const DEVPROPKEY *PropertyKey,
    DEVPROPTYPE *PropertyType,
    PBYTE PropertyBuffer,
    PULONG PropertyBufferSize,
    ULONG ulFlags,
    HMACHINE hMachine)
{
    DPCTX ctx;
    DWORD err, req = 0;

    TRACE("%lx %p %p %p %p %lx %p\n", dnDevInst, PropertyKey, PropertyType, PropertyBuffer,
          PropertyBufferSize, ulFlags, hMachine);

    if (!PropertyKey || !PropertyType || !PropertyBufferSize)
        return CR_INVALID_POINTER;
    if (!PropertyBuffer && *PropertyBufferSize)
        return CR_INVALID_POINTER;
    if (ulFlags)
        return CR_INVALID_FLAG;
    if (!dnDevInst)
        return CR_INVALID_DEVNODE;

    dp_init_cm(&ctx, dnDevInst, hMachine);
    err = dp_get(&ctx, PropertyKey, PropertyType, PropertyBuffer, *PropertyBufferSize, &req);
    if (err == ERROR_SUCCESS || err == ERROR_INSUFFICIENT_BUFFER)
        *PropertyBufferSize = req;
    return dp_error_to_cr(err);
}

/***********************************************************************
 * CM_Get_DevNode_PropertyW [SETUPAPI.@]
 */
CONFIGRET WINAPI CM_Get_DevNode_PropertyW(
    DEVINST dnDevInst,
    const DEVPROPKEY *PropertyKey,
    DEVPROPTYPE *PropertyType,
    PBYTE PropertyBuffer,
    PULONG PropertyBufferSize,
    ULONG ulFlags)
{
    return CM_Get_DevNode_Property_ExW(dnDevInst, PropertyKey, PropertyType, PropertyBuffer,
                                       PropertyBufferSize, ulFlags, NULL);
}

/***********************************************************************
 * CM_Get_DevNode_Property_Keys_Ex [SETUPAPI.@]
 */
CONFIGRET WINAPI CM_Get_DevNode_Property_Keys_Ex(
    DEVINST dnDevInst,
    DEVPROPKEY *PropertyKeyArray,
    PULONG PropertyKeyCount,
    ULONG ulFlags,
    HMACHINE hMachine)
{
    DPCTX ctx;
    DWORD err, required = 0;

    TRACE("%lx %p %p %lx %p\n", dnDevInst, PropertyKeyArray, PropertyKeyCount, ulFlags, hMachine);

    if (!PropertyKeyCount)
        return CR_INVALID_POINTER;
    if (!PropertyKeyArray && *PropertyKeyCount)
        return CR_INVALID_POINTER;
    if (ulFlags)
        return CR_INVALID_FLAG;
    if (!dnDevInst)
        return CR_INVALID_DEVNODE;

    dp_init_cm(&ctx, dnDevInst, hMachine);
    err = dp_get_keys(&ctx, PropertyKeyArray, *PropertyKeyCount, &required);
    if (err == ERROR_SUCCESS || err == ERROR_INSUFFICIENT_BUFFER)
        *PropertyKeyCount = required;
    return dp_error_to_cr(err);
}

/***********************************************************************
 * CM_Get_DevNode_Property_Keys [SETUPAPI.@]
 */
CONFIGRET WINAPI CM_Get_DevNode_Property_Keys(
    DEVINST dnDevInst,
    DEVPROPKEY *PropertyKeyArray,
    PULONG PropertyKeyCount,
    ULONG ulFlags)
{
    return CM_Get_DevNode_Property_Keys_Ex(dnDevInst, PropertyKeyArray, PropertyKeyCount, ulFlags, NULL);
}
