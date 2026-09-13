#pragma once
#include "ntddk.h"
#ifdef __cplusplus
using REFIID = const GUID &;
struct IUnknown { virtual NTSTATUS QueryInterface(REFIID, PVOID*)=0; virtual ULONG AddRef()=0; virtual ULONG Release()=0; };
#define IMP_IResourceList \
    ULONG NumberOfEntries(); \
    ULONG NumberOfEntriesOfType(CM_RESOURCE_TYPE); \
    PCM_PARTIAL_RESOURCE_DESCRIPTOR FindTranslatedEntry(CM_RESOURCE_TYPE, ULONG); \
    PCM_PARTIAL_RESOURCE_DESCRIPTOR FindUntranslatedEntry(CM_RESOURCE_TYPE, ULONG); \
    NTSTATUS AddEntry(PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR); \
    NTSTATUS AddEntryFromParent(struct IResourceList*, CM_RESOURCE_TYPE, ULONG); \
    PCM_RESOURCE_LIST TranslatedList(); \
    PCM_RESOURCE_LIST UntranslatedList()
struct IResourceList : IUnknown {
    virtual ULONG NumberOfEntries()=0;
    virtual ULONG NumberOfEntriesOfType(CM_RESOURCE_TYPE)=0;
    virtual PCM_PARTIAL_RESOURCE_DESCRIPTOR FindTranslatedEntry(CM_RESOURCE_TYPE, ULONG)=0;
    virtual PCM_PARTIAL_RESOURCE_DESCRIPTOR FindUntranslatedEntry(CM_RESOURCE_TYPE, ULONG)=0;
    virtual NTSTATUS AddEntry(PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR)=0;
    virtual NTSTATUS AddEntryFromParent(IResourceList*, CM_RESOURCE_TYPE, ULONG)=0;
    virtual PCM_RESOURCE_LIST TranslatedList()=0;
    virtual PCM_RESOURCE_LIST UntranslatedList()=0;
};
#else
typedef struct IUnknown IUnknown;
typedef struct IResourceList IResourceList;
#endif
typedef IUnknown *PUNKNOWN;
typedef IResourceList *PRESOURCELIST;
