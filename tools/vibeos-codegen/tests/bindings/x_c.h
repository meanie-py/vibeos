
/* client prototype */
UINT __cdecl remote_GetProperty(
    MSIHANDLE hinst,
    LPCWSTR property,
    LPWSTR *value,
    DWORD *size);

/* server prototype */
UINT __cdecl s_remote_GetProperty(
    MSIHANDLE hinst,
    LPCWSTR property,
    LPWSTR *value,
    DWORD *size);

void __cdecl ept_insert(
    handle_t h,
    unsigned32 num_ents,
    ept_entry_t entries[],
    boolean32 replace,
    error_status_t *status);

LANGID __cdecl remote_GetLanguage(
    MSIHANDLE hinst);

DWORD __cdecl PNP_GetVersion(void);
