/* What genuine sdk/lib/rtl/nls.c needs beyond the refstub <rtl.h>: the code-page table
 * types (ndk/rtltypes.h, layouts identical on every target) and RtlCopyMemory. */
#ifndef RTL_REFSTUB_NLS_HOST_H
#define RTL_REFSTUB_NLS_HOST_H
#include <string.h>
#define MAXIMUM_LEADBYTES 12
typedef struct _CPTABLEINFO {
    USHORT CodePage;
    USHORT MaximumCharacterSize;
    USHORT DefaultChar;
    USHORT UniDefaultChar;
    USHORT TransDefaultChar;
    USHORT TransUniDefaultChar;
    USHORT DBCSCodePage;
    UCHAR LeadByte[MAXIMUM_LEADBYTES];
    PUSHORT MultiByteTable;
    void *WideCharTable;
    PUSHORT DBCSRanges;
    PUSHORT DBCSOffsets;
} CPTABLEINFO, *PCPTABLEINFO;
typedef struct _NLSTABLEINFO {
    CPTABLEINFO OemTableInfo;
    CPTABLEINFO AnsiTableInfo;
    PUSHORT UpperCaseTable;
    PUSHORT LowerCaseTable;
} NLSTABLEINFO, *PNLSTABLEINFO;
typedef struct _NLS_FILE_HEADER {
    USHORT HeaderSize;
    USHORT CodePage;
    USHORT MaximumCharacterSize;
    USHORT DefaultChar;
    USHORT UniDefaultChar;
    USHORT TransDefaultChar;
    USHORT TransUniDefaultChar;
    UCHAR LeadByte[MAXIMUM_LEADBYTES];
} NLS_FILE_HEADER, *PNLS_FILE_HEADER;
#ifndef RtlCopyMemory
#define RtlCopyMemory(d, s, n) memcpy((d), (s), (n))
#endif
#endif
