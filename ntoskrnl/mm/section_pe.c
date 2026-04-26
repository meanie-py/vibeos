/*
 * section_pe.c — PE format section parser
 *
 * Extracted from ntoskrnl/mm/section.c. PeFmtCreateSection is a 600-line PE
 * parser that was not translated to Rust. It remains in C and is called from
 * the Rust ExeFmtpCreateImageSection via extern "system".
 */

/* INCLUDES *****************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>
#include <reactos/exeformat.h>

#include "ARM3/miarm.h"

/* GLOBALS *******************************************************************/

static ULONG SectionCharacteristicsToProtect[16] =
{
    PAGE_NOACCESS,          /* 0 = NONE */
    PAGE_NOACCESS,          /* 1 = SHARED */
    PAGE_EXECUTE,           /* 2 = EXECUTABLE */
    PAGE_EXECUTE,           /* 3 = EXECUTABLE, SHARED */
    PAGE_READONLY,          /* 4 = READABLE */
    PAGE_READONLY,          /* 5 = READABLE, SHARED */
    PAGE_EXECUTE_READ,      /* 6 = READABLE, EXECUTABLE */
    PAGE_EXECUTE_READ,      /* 7 = READABLE, EXECUTABLE, SHARED */
    PAGE_READWRITE,         /* 8 = WRITABLE */
    PAGE_READWRITE,         /* 9 = WRITABLE, SHARED */
    PAGE_EXECUTE_READWRITE, /* 10 = WRITABLE, EXECUTABLE */
    PAGE_EXECUTE_READWRITE, /* 11 = WRITABLE, EXECUTABLE, SHARED */
    PAGE_READWRITE,         /* 12 = WRITABLE, READABLE */
    PAGE_READWRITE,         /* 13 = WRITABLE, READABLE, SHARED */
    PAGE_EXECUTE_READWRITE, /* 14 = WRITABLE, READABLE, EXECUTABLE */
    PAGE_EXECUTE_READWRITE, /* 15 = WRITABLE, READABLE, EXECUTABLE, SHARED */
};

/* FUNCTIONS *****************************************************************/

/*
 References:
  [1] Microsoft Corporation, "Microsoft Portable Executable and Common Object
      File Format Specification", revision 6.0 (February 1999)
*/
NTSTATUS NTAPI PeFmtCreateSection(IN CONST VOID * FileHeader,
                                  IN SIZE_T FileHeaderSize,
                                  IN PVOID File,
                                  OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
                                  OUT PULONG Flags,
                                  IN PEXEFMT_CB_READ_FILE ReadFileCb,
                                  IN PEXEFMT_CB_ALLOCATE_SEGMENTS AllocateSegmentsCb)
{
    NTSTATUS nStatus;
    ULONG cbFileHeaderOffsetSize = 0;
    ULONG cbSectionHeadersOffset = 0;
    ULONG cbSectionHeadersSize;
    ULONG cbSectionHeadersOffsetSize = 0;
    ULONG cbOptHeaderSize;
    ULONG cbHeadersSize = 0;
    ULONG nSectionAlignment;
    ULONG nFileAlignment;
    ULONG_PTR ImageBase = 0;
    const IMAGE_DOS_HEADER * pidhDosHeader;
    const IMAGE_NT_HEADERS32 * pinhNtHeader;
    const IMAGE_OPTIONAL_HEADER32 * piohOptHeader;
    const IMAGE_SECTION_HEADER * pishSectionHeaders;
    PMM_SECTION_SEGMENT pssSegments;
    LARGE_INTEGER lnOffset;
    PVOID pBuffer;
    SIZE_T nPrevVirtualEndOfSegment = 0;
    ULONG nFileSizeOfHeaders = 0;
    ULONG i;
    ULONG AlignedLength;

    ASSERT(FileHeader);
    ASSERT(FileHeaderSize > 0);
    ASSERT(File);
    ASSERT(ImageSectionObject);
    ASSERT(ReadFileCb);
    ASSERT(AllocateSegmentsCb);

    ASSERT(Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize));

    ASSERT(((UINT_PTR)FileHeader % TYPE_ALIGNMENT(IMAGE_DOS_HEADER)) == 0);

#define DIE(ARGS_) { DPRINT ARGS_; goto l_Return; }

    pBuffer = NULL;
    pidhDosHeader = FileHeader;

    /* DOS HEADER */
    nStatus = STATUS_ROS_EXEFMT_UNKNOWN_FORMAT;

    /* image too small to be an MZ executable */
    if(FileHeaderSize < sizeof(IMAGE_DOS_HEADER))
        DIE(("Too small to be an MZ executable, size is %lu\n", FileHeaderSize));

    /* no MZ signature */
    if(pidhDosHeader->e_magic != IMAGE_DOS_SIGNATURE)
        DIE(("No MZ signature found, e_magic is %hX\n", pidhDosHeader->e_magic));

    /* NT HEADER */
    nStatus = STATUS_INVALID_IMAGE_PROTECT;

    /* not a Windows executable */
    if(pidhDosHeader->e_lfanew <= 0)
        DIE(("Not a Windows executable, e_lfanew is %d\n", pidhDosHeader->e_lfanew));

    if(!Intsafe_AddULong32(&cbFileHeaderOffsetSize, pidhDosHeader->e_lfanew, RTL_SIZEOF_THROUGH_FIELD(IMAGE_NT_HEADERS32, FileHeader)))
        DIE(("The DOS stub is too large, e_lfanew is %X\n", pidhDosHeader->e_lfanew));

    if(FileHeaderSize < cbFileHeaderOffsetSize)
        pinhNtHeader = NULL;
    else
    {
        /*
         * we already know that Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize),
         * and FileHeaderSize >= cbFileHeaderOffsetSize, so this holds true too
         */
        ASSERT(Intsafe_CanOffsetPointer(FileHeader, pidhDosHeader->e_lfanew));
        pinhNtHeader = (PVOID)((UINT_PTR)FileHeader + pidhDosHeader->e_lfanew);
    }

    /*
     * the buffer doesn't contain the NT file header, or the alignment is wrong: we
     * need to read the header from the file
     */
    if(FileHeaderSize < cbFileHeaderOffsetSize ||
            (UINT_PTR)pinhNtHeader % TYPE_ALIGNMENT(IMAGE_NT_HEADERS32) != 0)
    {
        ULONG cbNtHeaderSize;
        ULONG cbReadSize;
        PVOID pData;

l_ReadHeaderFromFile:
        cbNtHeaderSize = 0;
        lnOffset.QuadPart = pidhDosHeader->e_lfanew;

        /* read the header from the file */
        nStatus = ReadFileCb(File, &lnOffset, sizeof(IMAGE_NT_HEADERS64), &pData, &pBuffer, &cbReadSize);

        if(!NT_SUCCESS(nStatus))
        {
            NTSTATUS ReturnedStatus = nStatus;

            /* If it attempted to read past the end of the file, it means e_lfanew is invalid */
            if (ReturnedStatus == STATUS_END_OF_FILE) nStatus = STATUS_INVALID_IMAGE_PROTECT;

            DIE(("ReadFile failed, status %08X\n", ReturnedStatus));
        }

        ASSERT(pData);
        ASSERT(pBuffer);
        ASSERT(cbReadSize > 0);

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        /* the buffer doesn't contain the file header */
        if(cbReadSize < RTL_SIZEOF_THROUGH_FIELD(IMAGE_NT_HEADERS32, FileHeader))
            DIE(("The file doesn't contain the PE file header\n"));

        pinhNtHeader = pData;

        /* object still not aligned: copy it to the beginning of the buffer */
        if((UINT_PTR)pinhNtHeader % TYPE_ALIGNMENT(IMAGE_NT_HEADERS32) != 0)
        {
            ASSERT((UINT_PTR)pBuffer % TYPE_ALIGNMENT(IMAGE_NT_HEADERS32) == 0);
            RtlMoveMemory(pBuffer, pData, cbReadSize);
            pinhNtHeader = pBuffer;
        }

        /* invalid NT header */
        nStatus = STATUS_INVALID_IMAGE_PROTECT;

        if(pinhNtHeader->Signature != IMAGE_NT_SIGNATURE)
            DIE(("The file isn't a PE executable, Signature is %X\n", pinhNtHeader->Signature));

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        if(!Intsafe_AddULong32(&cbNtHeaderSize, pinhNtHeader->FileHeader.SizeOfOptionalHeader, FIELD_OFFSET(IMAGE_NT_HEADERS32, OptionalHeader)))
            DIE(("The full NT header is too large\n"));

        /* the buffer doesn't contain the whole NT header */
        if(cbReadSize < cbNtHeaderSize)
            DIE(("The file doesn't contain the full NT header\n"));
    }
    else
    {
        ULONG cbOptHeaderOffsetSize = 0;

        nStatus = STATUS_INVALID_IMAGE_PROTECT;

        /* don't trust an invalid NT header */
        if(pinhNtHeader->Signature != IMAGE_NT_SIGNATURE)
            DIE(("The file isn't a PE executable, Signature is %X\n", pinhNtHeader->Signature));

        if(!Intsafe_AddULong32(&cbOptHeaderOffsetSize, pidhDosHeader->e_lfanew, FIELD_OFFSET(IMAGE_NT_HEADERS32, OptionalHeader)))
            DIE(("The DOS stub is too large, e_lfanew is %X\n", pidhDosHeader->e_lfanew));

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        if(!Intsafe_AddULong32(&cbOptHeaderOffsetSize, cbOptHeaderOffsetSize, pinhNtHeader->FileHeader.SizeOfOptionalHeader))
            DIE(("The NT header is too large, SizeOfOptionalHeader is %X\n", pinhNtHeader->FileHeader.SizeOfOptionalHeader));

        /* the buffer doesn't contain the whole NT header: read it from the file */
        if(cbOptHeaderOffsetSize > FileHeaderSize)
            goto l_ReadHeaderFromFile;
    }

    /* read information from the NT header */
    piohOptHeader = &pinhNtHeader->OptionalHeader;
    cbOptHeaderSize = pinhNtHeader->FileHeader.SizeOfOptionalHeader;

    nStatus = STATUS_INVALID_IMAGE_FORMAT;

    if(!RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, Magic))
        DIE(("The optional header doesn't contain the Magic field, SizeOfOptionalHeader is %X\n", cbOptHeaderSize));

    /* ASSUME: RtlZeroMemory(ImageSectionObject, sizeof(*ImageSectionObject)); */

    switch(piohOptHeader->Magic)
    {
        case IMAGE_NT_OPTIONAL_HDR64_MAGIC:
#ifndef _WIN64
            nStatus = STATUS_INVALID_IMAGE_WIN_64;
            DIE(("Win64 optional header, unsupported\n"));
#else
            // Fall through.
#endif
        case IMAGE_NT_OPTIONAL_HDR32_MAGIC:
            break;
        default:
            DIE(("Unrecognized optional header, Magic is %X\n", piohOptHeader->Magic));
    }

    if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SectionAlignment) &&
            RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, FileAlignment))
    {
        /* See [1], section 3.4.2 */
        if(piohOptHeader->SectionAlignment < PAGE_SIZE)
        {
            if(piohOptHeader->FileAlignment != piohOptHeader->SectionAlignment)
                DIE(("Sections aren't page-aligned and the file alignment isn't the same\n"));
        }
        else if(piohOptHeader->SectionAlignment < piohOptHeader->FileAlignment)
            DIE(("The section alignment is smaller than the file alignment\n"));

        nSectionAlignment = piohOptHeader->SectionAlignment;
        nFileAlignment = piohOptHeader->FileAlignment;

        if(!IsPowerOf2(nSectionAlignment) || !IsPowerOf2(nFileAlignment))
            DIE(("The section alignment (%u) and file alignment (%u) aren't both powers of 2\n", nSectionAlignment, nFileAlignment));
    }
    else
    {
        nSectionAlignment = PAGE_SIZE;
        nFileAlignment = PAGE_SIZE;
    }

    ASSERT(IsPowerOf2(nSectionAlignment));
    ASSERT(IsPowerOf2(nFileAlignment));

    switch(piohOptHeader->Magic)
    {
    /* PE32 */
    case IMAGE_NT_OPTIONAL_HDR32_MAGIC:
    {
        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, ImageBase))
            ImageBase = piohOptHeader->ImageBase;

        if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfImage))
            ImageSectionObject->ImageInformation.ImageFileSize = piohOptHeader->SizeOfImage;

        if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfStackReserve))
            ImageSectionObject->ImageInformation.MaximumStackSize = piohOptHeader->SizeOfStackReserve;

        if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfStackCommit))
            ImageSectionObject->ImageInformation.CommittedStackSize = piohOptHeader->SizeOfStackCommit;

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, Subsystem))
        {
            ImageSectionObject->ImageInformation.SubSystemType = piohOptHeader->Subsystem;

            if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, MinorSubsystemVersion) &&
                    RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, MajorSubsystemVersion))
            {
                ImageSectionObject->ImageInformation.SubSystemMinorVersion = piohOptHeader->MinorSubsystemVersion;
                ImageSectionObject->ImageInformation.SubSystemMajorVersion = piohOptHeader->MajorSubsystemVersion;
            }
        }

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            ImageSectionObject->ImageInformation.TransferAddress = (PVOID) (ImageBase +
                    piohOptHeader->AddressOfEntryPoint);
        }

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfCode))
            ImageSectionObject->ImageInformation.ImageContainsCode = piohOptHeader->SizeOfCode != 0;
        else
            ImageSectionObject->ImageInformation.ImageContainsCode = TRUE;

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            if (piohOptHeader->AddressOfEntryPoint == 0)
            {
                ImageSectionObject->ImageInformation.ImageContainsCode = FALSE;
            }
        }

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, LoaderFlags))
            ImageSectionObject->ImageInformation.LoaderFlags = piohOptHeader->LoaderFlags;

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, DllCharacteristics))
        {
            ImageSectionObject->ImageInformation.DllCharacteristics = piohOptHeader->DllCharacteristics;

            /*
             * Since we don't really implement SxS yet and LD doesn't supoprt /ALLOWISOLATION:NO, hard-code
             * this flag here, which will prevent the loader and other code from doing any .manifest or SxS
             * magic to any binary.
             *
             * This will break applications that depend on SxS when running with real Windows Kernel32/SxS/etc
             * but honestly that's not tested. It will also break them when running no ReactOS once we implement
             * the SxS support -- at which point, duh, this should be removed.
             *
             * But right now, any app depending on SxS is already broken anyway, so this flag only helps.
             */
            ImageSectionObject->ImageInformation.DllCharacteristics |= IMAGE_DLLCHARACTERISTICS_NO_ISOLATION;
        }

        break;
    }
#ifdef _WIN64
    /* PE64 */
    case IMAGE_NT_OPTIONAL_HDR64_MAGIC:
    {
        const IMAGE_OPTIONAL_HEADER64 * pioh64OptHeader;

        pioh64OptHeader = (const IMAGE_OPTIONAL_HEADER64 *)piohOptHeader;

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, ImageBase))
        {
            ImageBase = pioh64OptHeader->ImageBase;
            if(pioh64OptHeader->ImageBase > MAXULONG_PTR)
                DIE(("ImageBase exceeds the address space\n"));
        }

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfImage))
        {
            if(pioh64OptHeader->SizeOfImage > MAXULONG_PTR)
                DIE(("SizeOfImage exceeds the address space\n"));

            ImageSectionObject->ImageInformation.ImageFileSize = pioh64OptHeader->SizeOfImage;
        }

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfStackReserve))
        {
            if(pioh64OptHeader->SizeOfStackReserve > MAXULONG_PTR)
                DIE(("SizeOfStackReserve exceeds the address space\n"));

            ImageSectionObject->ImageInformation.MaximumStackSize = (ULONG_PTR) pioh64OptHeader->SizeOfStackReserve;
        }

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfStackCommit))
        {
            if(pioh64OptHeader->SizeOfStackCommit > MAXULONG_PTR)
                DIE(("SizeOfStackCommit exceeds the address space\n"));

            ImageSectionObject->ImageInformation.CommittedStackSize = (ULONG_PTR) pioh64OptHeader->SizeOfStackCommit;
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, Subsystem))
        {
            ImageSectionObject->ImageInformation.SubSystemType = pioh64OptHeader->Subsystem;

            if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, MinorSubsystemVersion) &&
                    RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, MajorSubsystemVersion))
            {
                ImageSectionObject->ImageInformation.SubSystemMinorVersion = pioh64OptHeader->MinorSubsystemVersion;
                ImageSectionObject->ImageInformation.SubSystemMajorVersion = pioh64OptHeader->MajorSubsystemVersion;
            }
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            ImageSectionObject->ImageInformation.TransferAddress = (PVOID) (ImageBase +
                    pioh64OptHeader->AddressOfEntryPoint);
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfCode))
            ImageSectionObject->ImageInformation.ImageContainsCode = pioh64OptHeader->SizeOfCode != 0;
        else
            ImageSectionObject->ImageInformation.ImageContainsCode = TRUE;

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            if (pioh64OptHeader->AddressOfEntryPoint == 0)
            {
                ImageSectionObject->ImageInformation.ImageContainsCode = FALSE;
            }
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, LoaderFlags))
            ImageSectionObject->ImageInformation.LoaderFlags = pioh64OptHeader->LoaderFlags;

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, DllCharacteristics))
            ImageSectionObject->ImageInformation.DllCharacteristics = pioh64OptHeader->DllCharacteristics;

        break;
    }
#endif // _WIN64
    }

    /* [1], section 3.4.2 */
    if((ULONG_PTR)ImageBase % 0x10000)
        DIE(("ImageBase is not aligned on a 64KB boundary"));

    ImageSectionObject->ImageInformation.ImageCharacteristics = pinhNtHeader->FileHeader.Characteristics;
    ImageSectionObject->ImageInformation.Machine = pinhNtHeader->FileHeader.Machine;
    ImageSectionObject->ImageInformation.GpValue = 0;
    ImageSectionObject->ImageInformation.ZeroBits = 0;
    ImageSectionObject->BasedAddress = (PVOID)ImageBase;

    /* SECTION HEADERS */
    nStatus = STATUS_INVALID_IMAGE_FORMAT;

    /* see [1], section 3.3 */
    if(pinhNtHeader->FileHeader.NumberOfSections > 96)
        DIE(("Too many sections, NumberOfSections is %u\n", pinhNtHeader->FileHeader.NumberOfSections));

    /*
     * the additional segment is for the file's headers. They need to be present for
     * the benefit of the dynamic loader (to locate exports, defaults for thread
     * parameters, resources, etc.)
     */
    ImageSectionObject->NrSegments = pinhNtHeader->FileHeader.NumberOfSections + 1;

    /* file offset for the section headers */
    if(!Intsafe_AddULong32(&cbSectionHeadersOffset, pidhDosHeader->e_lfanew, FIELD_OFFSET(IMAGE_NT_HEADERS32, OptionalHeader)))
        DIE(("Offset overflow\n"));

    if(!Intsafe_AddULong32(&cbSectionHeadersOffset, cbSectionHeadersOffset, pinhNtHeader->FileHeader.SizeOfOptionalHeader))
        DIE(("Offset overflow\n"));

    /* size of the section headers */
    ASSERT(Intsafe_CanMulULong32(pinhNtHeader->FileHeader.NumberOfSections, sizeof(IMAGE_SECTION_HEADER)));
    cbSectionHeadersSize = pinhNtHeader->FileHeader.NumberOfSections * sizeof(IMAGE_SECTION_HEADER);

    if(!Intsafe_AddULong32(&cbSectionHeadersOffsetSize, cbSectionHeadersOffset, cbSectionHeadersSize))
        DIE(("Section headers too large\n"));

    /* size of the executable's headers */
    if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfHeaders))
    {
//        if(!IsAligned(piohOptHeader->SizeOfHeaders, nFileAlignment))
//            DIE(("SizeOfHeaders is not aligned\n"));

        if(cbSectionHeadersSize > piohOptHeader->SizeOfHeaders)
            DIE(("The section headers overflow SizeOfHeaders\n"));

        cbHeadersSize = piohOptHeader->SizeOfHeaders;
    }
    else if(!AlignUp(&cbHeadersSize, cbSectionHeadersOffsetSize, nFileAlignment))
        DIE(("Overflow aligning the size of headers\n"));

    if(pBuffer)
    {
        ExFreePool(pBuffer);
        pBuffer = NULL;
    }
    /* WARNING: pinhNtHeader IS NO LONGER USABLE */
    /* WARNING: piohOptHeader IS NO LONGER USABLE */
    /* WARNING: pioh64OptHeader IS NO LONGER USABLE */

    if(FileHeaderSize < cbSectionHeadersOffsetSize)
        pishSectionHeaders = NULL;
    else
    {
        /*
         * we already know that Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize),
         * and FileHeaderSize >= cbSectionHeadersOffsetSize, so this holds true too
         */
        ASSERT(Intsafe_CanOffsetPointer(FileHeader, cbSectionHeadersOffset));
        pishSectionHeaders = (PVOID)((UINT_PTR)FileHeader + cbSectionHeadersOffset);
    }

    /*
     * the buffer doesn't contain the section headers, or the alignment is wrong:
     * read the headers from the file
     */
    if(FileHeaderSize < cbSectionHeadersOffsetSize ||
            (UINT_PTR)pishSectionHeaders % TYPE_ALIGNMENT(IMAGE_SECTION_HEADER) != 0)
    {
        PVOID pData;
        ULONG cbReadSize;

        lnOffset.QuadPart = cbSectionHeadersOffset;

        /* read the header from the file */
        nStatus = ReadFileCb(File, &lnOffset, cbSectionHeadersSize, &pData, &pBuffer, &cbReadSize);

        if(!NT_SUCCESS(nStatus))
            DIE(("ReadFile failed with status %08X\n", nStatus));

        ASSERT(pData);
        ASSERT(pBuffer);
        ASSERT(cbReadSize > 0);

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        /* the buffer doesn't contain all the section headers */
        if(cbReadSize < cbSectionHeadersSize)
            DIE(("The file doesn't contain all of the section headers\n"));

        pishSectionHeaders = pData;

        /* object still not aligned: copy it to the beginning of the buffer */
        if((UINT_PTR)pishSectionHeaders % TYPE_ALIGNMENT(IMAGE_SECTION_HEADER) != 0)
        {
            ASSERT((UINT_PTR)pBuffer % TYPE_ALIGNMENT(IMAGE_SECTION_HEADER) == 0);
            RtlMoveMemory(pBuffer, pData, cbReadSize);
            pishSectionHeaders = pBuffer;
        }
    }

    /* SEGMENTS */
    /* allocate the segments */
    nStatus = STATUS_INSUFFICIENT_RESOURCES;
    ImageSectionObject->Segments = AllocateSegmentsCb(ImageSectionObject->NrSegments);

    if(ImageSectionObject->Segments == NULL)
        DIE(("AllocateSegments failed\n"));

    /* initialize the headers segment */
    pssSegments = ImageSectionObject->Segments;

//  ASSERT(IsAligned(cbHeadersSize, nFileAlignment));

    if(!AlignUp(&nFileSizeOfHeaders, cbHeadersSize, nFileAlignment))
        DIE(("Cannot align the size of the section headers\n"));

    nPrevVirtualEndOfSegment = ALIGN_UP_BY(cbHeadersSize, nSectionAlignment);
    if (nPrevVirtualEndOfSegment < cbHeadersSize)
        DIE(("Cannot align the size of the section headers\n"));

    pssSegments[0].Image.FileOffset = 0;
    pssSegments[0].Protection = PAGE_READONLY;
    pssSegments[0].Length.QuadPart = nPrevVirtualEndOfSegment;
    pssSegments[0].RawLength.QuadPart = nFileSizeOfHeaders;
    pssSegments[0].Image.VirtualAddress = 0;
    pssSegments[0].Image.Characteristics = 0;
    pssSegments[0].WriteCopy = TRUE;

    /* skip the headers segment */
    ++ pssSegments;

    nStatus = STATUS_INVALID_IMAGE_FORMAT;

    ASSERT(ImageSectionObject->RefCount > 0);

    /* convert the executable sections into segments. See also [1], section 4 */
    for(i = 0; i < ImageSectionObject->NrSegments - 1; ++ i)
    {
        ULONG nCharacteristics;

        /* validate the alignment */
        if(!IsAligned(pishSectionHeaders[i].VirtualAddress, nSectionAlignment))
            DIE(("Image.VirtualAddress[%u] is not aligned\n", i));

        /* sections must be contiguous, ordered by base address and non-overlapping */
        if(pishSectionHeaders[i].VirtualAddress != nPrevVirtualEndOfSegment)
            DIE(("Memory gap between section %u and the previous\n", i));

        /* ignore explicit BSS sections */
        if(pishSectionHeaders[i].PointerToRawData != 0 && pishSectionHeaders[i].SizeOfRawData != 0)
        {
            /* validate the alignment */
#if 0
            /* Yes, this should be a multiple of FileAlignment, but there's
             * stuff out there that isn't. We can cope with that
             */
            if(!IsAligned(pishSectionHeaders[i].SizeOfRawData, nFileAlignment))
                DIE(("SizeOfRawData[%u] is not aligned\n", i));
#endif

//            if(!IsAligned(pishSectionHeaders[i].PointerToRawData, nFileAlignment))
//                DIE(("PointerToRawData[%u] is not aligned\n", i));

            if(!Intsafe_CanAddULong32(pishSectionHeaders[i].PointerToRawData, pishSectionHeaders[i].SizeOfRawData))
                DIE(("SizeOfRawData[%u] too large\n", i));

            /* conversion */
            pssSegments[i].Image.FileOffset = pishSectionHeaders[i].PointerToRawData;
            pssSegments[i].RawLength.QuadPart = pishSectionHeaders[i].SizeOfRawData;
        }
        else
        {
            /* FIXME: Should reset PointerToRawData to 0 in the image mapping */
            ASSERT(pssSegments[i].Image.FileOffset == 0);
            ASSERT(pssSegments[i].RawLength.QuadPart == 0);
        }

        ASSERT(Intsafe_CanAddLong64(pssSegments[i].Image.FileOffset, pssSegments[i].RawLength.QuadPart));

        nCharacteristics = pishSectionHeaders[i].Characteristics;

        /* no explicit protection */
        if((nCharacteristics & (IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE)) == 0)
        {
            if(nCharacteristics & IMAGE_SCN_CNT_CODE)
                nCharacteristics |= IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ;

            if(nCharacteristics & IMAGE_SCN_CNT_INITIALIZED_DATA)
                nCharacteristics |= IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;

            if(nCharacteristics & IMAGE_SCN_CNT_UNINITIALIZED_DATA)
                nCharacteristics |= IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
        }

        /* see table above */
        pssSegments[i].Protection = SectionCharacteristicsToProtect[nCharacteristics >> 28];
        pssSegments[i].WriteCopy = !(nCharacteristics & IMAGE_SCN_MEM_SHARED);

        if(pishSectionHeaders[i].Misc.VirtualSize == 0)
            pssSegments[i].Length.QuadPart = pishSectionHeaders[i].SizeOfRawData;
        else
            pssSegments[i].Length.QuadPart = pishSectionHeaders[i].Misc.VirtualSize;

        AlignedLength = ALIGN_UP_BY(pssSegments[i].Length.LowPart, nSectionAlignment);
        if(AlignedLength < pssSegments[i].Length.LowPart)
            DIE(("Cannot align the virtual size of section %u\n", i));

        pssSegments[i].Length.LowPart = AlignedLength;

        if(pssSegments[i].Length.QuadPart == 0)
            DIE(("Virtual size of section %u is null\n", i));

        pssSegments[i].Image.VirtualAddress = pishSectionHeaders[i].VirtualAddress;
        pssSegments[i].Image.Characteristics = pishSectionHeaders[i].Characteristics;

        /* ensure the memory image is no larger than 4GB */
        nPrevVirtualEndOfSegment = (ULONG_PTR)(pssSegments[i].Image.VirtualAddress + pssSegments[i].Length.QuadPart);
        if (nPrevVirtualEndOfSegment < pssSegments[i].Image.VirtualAddress)
            DIE(("The image is too large\n"));
    }

    if(nSectionAlignment >= PAGE_SIZE)
        *Flags |= EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED;

    /* Success */
    nStatus = STATUS_SUCCESS;// STATUS_ROS_EXEFMT_LOADED_FORMAT | EXEFMT_LOADED_PE32;

l_Return:
    if(pBuffer)
        ExFreePool(pBuffer);

    return nStatus;
}
