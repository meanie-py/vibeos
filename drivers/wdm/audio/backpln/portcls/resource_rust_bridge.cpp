/*
 * COPYRIGHT:       See COPYING in the top level directory
 * PROJECT:         ReactOS
 * FILE:            drivers/wdm/audio/backpln/portcls/resource.cpp
 * PURPOSE:         Port Class driver / ResourceList implementation
 * PROGRAMMER:      Andrew Greenwood
 *                  Johannes Anderwald
 * HISTORY:
 *                  27 Jan 07   Created
 */

#include "private.hpp"
#include "resource_rust_bridge.h"

#define NDEBUG
#include <debug.h>

class CResourceList : public CUnknownImpl<IResourceList>
{
public:
    STDMETHODIMP QueryInterface( REFIID InterfaceId, PVOID* Interface);

    IMP_IResourceList;

    CResourceList(IUnknown * OuterUnknown) :
        m_OuterUnknown(OuterUnknown),
        m_PoolType(NonPagedPool),
        m_TranslatedResourceList(0),
        m_UntranslatedResourceList(0),
        m_NumberOfEntries(0),
        m_MaxEntries(0)
    {
    }
    virtual ~CResourceList();

public:
    PUNKNOWN m_OuterUnknown;
    POOL_TYPE m_PoolType;
    PCM_RESOURCE_LIST m_TranslatedResourceList;
    PCM_RESOURCE_LIST m_UntranslatedResourceList;
    ULONG m_NumberOfEntries;
    ULONG m_MaxEntries;
};

CResourceList::~CResourceList()
{
    if (m_TranslatedResourceList)
    {
        /* Free resource list */
        FreeItem(m_TranslatedResourceList, TAG_PORTCLASS);
    }

    if (m_UntranslatedResourceList)
    {
        /* Free resource list */
        FreeItem(m_UntranslatedResourceList, TAG_PORTCLASS);
    }
}

NTSTATUS
NTAPI
CResourceList::QueryInterface(
    IN  REFIID refiid,
    OUT PVOID* Output)
{
    UNICODE_STRING GuidString;

    if (IsEqualGUIDAligned(refiid, IID_IResourceList) ||
        IsEqualGUIDAligned(refiid, IID_IUnknown))
    {
        *Output = PVOID(PRESOURCELIST(this));
        PUNKNOWN(*Output)->AddRef();
        return STATUS_SUCCESS;
    }

    if (RtlStringFromGUID(refiid, &GuidString) == STATUS_SUCCESS)
    {
        DPRINT1("IResourceList_QueryInterface no interface!!! iface %S\n", GuidString.Buffer);
        RtlFreeUnicodeString(&GuidString);
    }

    return STATUS_UNSUCCESSFUL;
}

ULONG
NTAPI
CResourceList::NumberOfEntries()
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_NumberOfEntries(&context);
}

ULONG
NTAPI
CResourceList::NumberOfEntriesOfType(
    IN  CM_RESOURCE_TYPE Type)
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_NumberOfEntriesOfType(&context, Type);
}

PCM_PARTIAL_RESOURCE_DESCRIPTOR
NTAPI
CResourceList::FindTranslatedEntry(
    IN  CM_RESOURCE_TYPE Type,
    IN  ULONG Index)
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_FindTranslatedEntry(&context, Type, Index);
}

PCM_PARTIAL_RESOURCE_DESCRIPTOR
NTAPI
CResourceList::FindUntranslatedEntry(
    IN  CM_RESOURCE_TYPE Type,
    IN  ULONG Index)
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_FindUntranslatedEntry(&context, Type, Index);
}

NTSTATUS
NTAPI
CResourceList::AddEntry(
    IN  PCM_PARTIAL_RESOURCE_DESCRIPTOR Translated,
    IN  PCM_PARTIAL_RESOURCE_DESCRIPTOR Untranslated)
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_AddEntry(&context, Translated, Untranslated);
}

NTSTATUS
NTAPI
CResourceList::AddEntryFromParent(
    IN  IResourceList* Parent,
    IN  CM_RESOURCE_TYPE Type,
    IN  ULONG Index)
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_AddEntryFromParent(&context, Parent, Type, Index);
}

PCM_RESOURCE_LIST
NTAPI
CResourceList::TranslatedList()
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_TranslatedList(&context);
}

PCM_RESOURCE_LIST
NTAPI
CResourceList::UntranslatedList()
{
    RustBridge_CResourceList_Context context = {this, &this->m_MaxEntries, &this->m_NumberOfEntries, &this->m_TranslatedResourceList, &this->m_UntranslatedResourceList};
    return RustBridge_CResourceList_UntranslatedList(&context);
}

PORTCLASSAPI
NTSTATUS
NTAPI
PcNewResourceList(
    OUT PRESOURCELIST* OutResourceList,
    IN  PUNKNOWN OuterUnknown OPTIONAL,
    IN  POOL_TYPE PoolType,
    IN  PCM_RESOURCE_LIST TranslatedResourceList,
    IN  PCM_RESOURCE_LIST UntranslatedResourceList)
{
    PCM_RESOURCE_LIST NewUntranslatedResources, NewTranslatedResources;
    ULONG ResourceSize, ResourceCount;
    CResourceList* NewList;
    NTSTATUS Status;

    if (!TranslatedResourceList)
    {
        /* If the untranslated resource list is also not provided, it becomes an empty resource list */
        if (UntranslatedResourceList)
        {
            /* Invalid parameter mix */
            return STATUS_INVALID_PARAMETER;
        }
    }
    else
    {
        /* If the translated resource list is also not provided, it becomes an empty resource list */
        if (!UntranslatedResourceList)
        {
            /* Invalid parameter mix */
            return STATUS_INVALID_PARAMETER;
        }
    }

    /* Allocate resource list */
    NewList = new(PoolType, TAG_PORTCLASS)CResourceList(OuterUnknown);
    if (!NewList)
        return STATUS_INSUFFICIENT_RESOURCES;

    /* Query resource list */
    Status = NewList->QueryInterface(IID_IResourceList, (PVOID*)OutResourceList);
    if (!NT_SUCCESS(Status))
    {
        /* Ouch, FIX ME */
        delete NewList;
        return STATUS_INVALID_PARAMETER;
    }

    /* Is there a resource list */
    if (!TranslatedResourceList)
    {
        /* Empty resource list */
        return STATUS_SUCCESS;
    }

    /* Sanity check */
    ASSERT(UntranslatedResourceList->List[0].PartialResourceList.Count == TranslatedResourceList->List[0].PartialResourceList.Count);

    /* Get resource count */
    ResourceCount = UntranslatedResourceList->List[0].PartialResourceList.Count;
#ifdef _MSC_VER
    ResourceSize = FIELD_OFFSET(CM_RESOURCE_LIST, List[0].PartialResourceList.PartialDescriptors[ResourceCount]);
#else
    ResourceSize = sizeof(CM_RESOURCE_LIST) - sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR) + (ResourceCount) * sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
#endif

    /* Allocate translated resource list */
    NewTranslatedResources = (PCM_RESOURCE_LIST)AllocateItem(PoolType, ResourceSize, TAG_PORTCLASS);
    if (!NewTranslatedResources)
    {
        /* No memory */
        delete NewList;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Allocate untranslated resource list */
    NewUntranslatedResources = (PCM_RESOURCE_LIST)AllocateItem(PoolType, ResourceSize, TAG_PORTCLASS);
    if (!NewUntranslatedResources)
    {
        /* No memory */
        delete NewList;
        FreeItem(NewTranslatedResources, TAG_PORTCLASS);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Copy resource lists */
    RtlCopyMemory(NewTranslatedResources, TranslatedResourceList, ResourceSize);
    RtlCopyMemory(NewUntranslatedResources, UntranslatedResourceList, ResourceSize);

    /* Init resource list */
    NewList->m_TranslatedResourceList= NewTranslatedResources;
    NewList->m_UntranslatedResourceList = NewUntranslatedResources;
    NewList->m_NumberOfEntries = ResourceCount;
    NewList->m_MaxEntries = ResourceCount;
    NewList->m_PoolType = PoolType;

    /* Done */
    return STATUS_SUCCESS;
}

PORTCLASSAPI
NTSTATUS
NTAPI
PcNewResourceSublist(
    OUT PRESOURCELIST* OutResourceList,
    IN  PUNKNOWN OuterUnknown OPTIONAL,
    IN  POOL_TYPE PoolType,
    IN  PRESOURCELIST ParentList,
    IN  ULONG MaximumEntries)
{
    CResourceList* NewList;
    ULONG ResourceSize;

    if (!OutResourceList || !ParentList || !MaximumEntries)
        return STATUS_INVALID_PARAMETER;

    /* Allocate new list */
    NewList = new(PoolType, TAG_PORTCLASS) CResourceList(OuterUnknown);
    if (!NewList)
        return STATUS_INSUFFICIENT_RESOURCES;

    /* Get resource size */
#ifdef _MSC_VER
    ResourceSize = FIELD_OFFSET(CM_RESOURCE_LIST, List[0].PartialResourceList.PartialDescriptors[MaximumEntries]);
#else
    ResourceSize = sizeof(CM_RESOURCE_LIST) - sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR) + (MaximumEntries) * sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
#endif

    /* Allocate resource list */
    NewList->m_TranslatedResourceList = (PCM_RESOURCE_LIST)AllocateItem(PoolType, ResourceSize, TAG_PORTCLASS);
    if (!NewList->m_TranslatedResourceList)
    {
        /* No memory */
        delete NewList;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Allocate resource list */
    NewList->m_UntranslatedResourceList = (PCM_RESOURCE_LIST)AllocateItem(PoolType, ResourceSize, TAG_PORTCLASS);
    if (!NewList->m_UntranslatedResourceList)
    {
        /* No memory */
        delete NewList;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Copy resource lists */
    RtlCopyMemory(NewList->m_TranslatedResourceList, ParentList->TranslatedList(), sizeof(CM_RESOURCE_LIST));
    RtlCopyMemory(NewList->m_UntranslatedResourceList, ParentList->UntranslatedList(), sizeof(CM_RESOURCE_LIST));

    /* Resource list is empty */
    NewList->m_UntranslatedResourceList->List[0].PartialResourceList.Count = 0;
    NewList->m_TranslatedResourceList->List[0].PartialResourceList.Count = 0;

    /* Store members */
    NewList->m_OuterUnknown = OuterUnknown;
    NewList->m_PoolType = PoolType;
    NewList->AddRef();
    NewList->m_NumberOfEntries = 0;
    NewList->m_MaxEntries = MaximumEntries;

    /* Store result */
    *OutResourceList = (IResourceList*)NewList;

    /* Done */
    return STATUS_SUCCESS;
}

extern "C" NTSTATUS RustBridge_CResourceList_call_CResourceList_AddEntry(void *object, PCM_PARTIAL_RESOURCE_DESCRIPTOR Translated, PCM_PARTIAL_RESOURCE_DESCRIPTOR Untranslated) { return static_cast<CResourceList *>(object)->AddEntry(Translated, Untranslated); }
extern "C" PCM_PARTIAL_RESOURCE_DESCRIPTOR RustBridge_CResourceList_call_IResourceList_FindTranslatedEntry(IResourceList *object, CM_RESOURCE_TYPE Type, ULONG Index) { return static_cast<IResourceList *>(object)->FindTranslatedEntry(Type, Index); }
extern "C" PCM_PARTIAL_RESOURCE_DESCRIPTOR RustBridge_CResourceList_call_IResourceList_FindUntranslatedEntry(IResourceList *object, CM_RESOURCE_TYPE Type, ULONG Index) { return static_cast<IResourceList *>(object)->FindUntranslatedEntry(Type, Index); }
