/* gdiobj.split2.c — #include fragment of gdiobj.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original gdiobj.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL NTAPI
GDI_CleanupForProcess(struct _EPROCESS *Process)
{
    PENTRY pentry;
    ULONG ulIndex;
    DWORD dwProcessId;
    PPROCESSINFO ppi;

    DPRINT("CleanupForProcess prochandle %p Pid %p\n",
           Process, Process->UniqueProcessId);

    ASSERT(Process == PsGetCurrentProcess());

    /* Get the current process Id */
    dwProcessId = PtrToUlong(PsGetCurrentProcessId());

    /* Loop all handles in the handle table */
    for (ulIndex = RESERVE_ENTRIES_COUNT; ulIndex < gulFirstUnused; ulIndex++)
    {
        pentry = &gpentHmgr[ulIndex];

        /* Check if the object is owned by the process */
        if (pentry->ObjectOwner.ulObj == dwProcessId)
        {
            ASSERT(pentry->einfo.pobj->cExclusiveLock == 0);

            /* Reference the object and delete it */
            InterlockedIncrement((LONG*)&gpaulRefCount[ulIndex]);
            GDIOBJ_vDeleteObject(pentry->einfo.pobj);
        }
    }

#if DBG
    DbgGdiHTIntegrityCheck();
#endif

    ppi = PsGetCurrentProcessWin32Process();
    DPRINT("Completed cleanup for process %p\n", Process->UniqueProcessId);
    if (ppi->GDIHandleCount != 0)
    {
        DPRINT1("Leaking %d handles!\n", ppi->GDIHandleCount);
        ASSERT(FALSE);
    }

    /* Loop all handles in the handle table */
    for (ulIndex = RESERVE_ENTRIES_COUNT; ulIndex < gulFirstUnused; ulIndex++)
    {
        pentry = &gpentHmgr[ulIndex];

        /* Check if the object is owned by the process */
        if (pentry->ObjectOwner.ulObj == dwProcessId)
        {
            DPRINT1("Leaking object. Index=%lx, type=0x%x, refcount=%lx\n",
                    ulIndex, pentry->Objt, gpaulRefCount[ulIndex]);
            DBG_DUMP_EVENT_LIST(&pentry->einfo.pobj->slhLog);
            //DBG_CLEANUP_EVENT_LIST(&pentry->einfo.pobj->slhLog);
            ASSERT(FALSE);
        }
    }

    return TRUE;
}

/// HACK!
PGDI_POOL
GetBrushAttrPool(VOID)
{
    PPROCESSINFO ppi;

    ppi = PsGetCurrentProcessWin32Process();
    NT_ASSERT(ppi != NULL);

    return ppi->pPoolBrushAttr;
}

/* EOF */
