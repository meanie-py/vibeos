// Generated C reference workers from CResourceList; never linked into the driver.
#include "resource_rust_bridge.h"
#define NDEBUG
#include <debug.h>
#define PC_ASSERT(exp) \
  (VOID)((!(exp)) ? \
    RtlAssert((PVOID) #exp, (PVOID)__FILE__, __LINE__, NULL ), FALSE : TRUE)

#define PC_ASSERT_IRQL_EQUAL(x) PC_ASSERT(KeGetCurrentIrql()==(x))


ULONG RustBridge_CResourceList_NumberOfEntries(RustBridge_CResourceList_Context *context)
{
   PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    return (*context->m_NumberOfEntries);
}

ULONG RustBridge_CResourceList_NumberOfEntriesOfType(RustBridge_CResourceList_Context *context, CM_RESOURCE_TYPE Type)
{
    ULONG Index, Count = 0;
    PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;

    PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    /* Is there a resource list? */
    if (!(*context->m_UntranslatedResourceList))
    {
        /* No resource list provided */
        return 0;
    }

    for (Index = 0; Index < (*context->m_NumberOfEntries); Index ++ )
    {

        /* Get descriptor */
        PartialDescriptor = &(*context->m_UntranslatedResourceList)->List[0].PartialResourceList.PartialDescriptors[Index];
        if (PartialDescriptor->Type == Type)
        {
            /* Yay! Finally found one that matches! */
            Count++;
        }
    }

    DPRINT("Found %d type %d\n", Count, Type);
    return Count;
}

PCM_PARTIAL_RESOURCE_DESCRIPTOR RustBridge_CResourceList_FindTranslatedEntry(RustBridge_CResourceList_Context *context, CM_RESOURCE_TYPE Type, ULONG Index)
{
    ULONG DescIndex, Count = 0;
    PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;

    PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    /* Is there a resource list? */
    if (!(*context->m_TranslatedResourceList))
    {
        /* No resource list */
        return NULL;
    }

    for (DescIndex = 0; DescIndex < (*context->m_NumberOfEntries); DescIndex ++ )
    {
        /* Get descriptor */
        PartialDescriptor = &(*context->m_TranslatedResourceList)->List[0].PartialResourceList.PartialDescriptors[DescIndex];

        if (PartialDescriptor->Type == Type)
        {
            /* Found type, is it the requested index? */
            if (Index == Count)
            {
                /* Found */
                return PartialDescriptor;
            }

            /* Need to continue search */
            Count++;
        }
    }

    /* No such descriptor */
    return NULL;
}

PCM_PARTIAL_RESOURCE_DESCRIPTOR RustBridge_CResourceList_FindUntranslatedEntry(RustBridge_CResourceList_Context *context, CM_RESOURCE_TYPE Type, ULONG Index)
{
    ULONG DescIndex, Count = 0;
    PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;

    PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    /* Is there a resource list? */
    if (!(*context->m_UntranslatedResourceList))
    {
        /* Empty resource list */
        return NULL;
    }

    /* Search descriptors */
    for (DescIndex = 0; DescIndex < (*context->m_NumberOfEntries); DescIndex ++ )
    {
        /* Get descriptor */
        PartialDescriptor = &(*context->m_UntranslatedResourceList)->List[0].PartialResourceList.PartialDescriptors[DescIndex];

        if (PartialDescriptor->Type == Type)
        {
            /* Found type, is it the requested index? */
            if (Index == Count)
            {
                /* Found */
                return PartialDescriptor;
            }

            /* Need to continue search */
            Count++;
        }
    }

    /* No such descriptor */
    return NULL;
}

NTSTATUS RustBridge_CResourceList_AddEntry(RustBridge_CResourceList_Context *context, PCM_PARTIAL_RESOURCE_DESCRIPTOR Translated, PCM_PARTIAL_RESOURCE_DESCRIPTOR Untranslated)
{
    PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;

    /* Sanity check */
    PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    /* Is there still room for another entry */
    if ((*context->m_NumberOfEntries) >= (*context->m_MaxEntries))
    {
        /* No more space */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Get free descriptor */
    PartialDescriptor = &(*context->m_UntranslatedResourceList)->List[0].PartialResourceList.PartialDescriptors[(*context->m_NumberOfEntries)];

    /* Copy descriptor */
    RtlCopyMemory(PartialDescriptor, Untranslated, sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR));

    /* Get free descriptor */
    PartialDescriptor = &(*context->m_TranslatedResourceList)->List[0].PartialResourceList.PartialDescriptors[(*context->m_NumberOfEntries)];

    /* Copy descriptor */
    RtlCopyMemory(PartialDescriptor, Translated, sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR));

    /* Add entry count */
    (*context->m_NumberOfEntries)++;
    (*context->m_UntranslatedResourceList)->List[0].PartialResourceList.Count++;
    (*context->m_TranslatedResourceList)->List[0].PartialResourceList.Count++;

    /* Done */
    return STATUS_SUCCESS;
}

NTSTATUS RustBridge_CResourceList_AddEntryFromParent(RustBridge_CResourceList_Context *context, IResourceList* Parent, CM_RESOURCE_TYPE Type, ULONG Index)
{
    PCM_PARTIAL_RESOURCE_DESCRIPTOR Translated, Untranslated;

    PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    /* Get entries from parent */
    Translated = RustBridge_CResourceList_call_IResourceList_FindTranslatedEntry(Parent, Type, Index);
    Untranslated = RustBridge_CResourceList_call_IResourceList_FindUntranslatedEntry(Parent, Type, Index);

    /* Are both found? */
    if (Translated && Untranslated)
    {
        /* Add entry from parent */
        return RustBridge_CResourceList_call_CResourceList_AddEntry(context->object, Translated, Untranslated);
    }

    /* Entry not found */
    return STATUS_INVALID_PARAMETER;
}

PCM_RESOURCE_LIST RustBridge_CResourceList_TranslatedList(RustBridge_CResourceList_Context *context)
{
    PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    return (*context->m_TranslatedResourceList);
}

PCM_RESOURCE_LIST RustBridge_CResourceList_UntranslatedList(RustBridge_CResourceList_Context *context)
{
    PC_ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);

    return (*context->m_UntranslatedResourceList);
}
