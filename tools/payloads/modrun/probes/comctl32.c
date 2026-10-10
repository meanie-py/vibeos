/* modrun probe: comctl32's non-window logic, shipping Rust against the pre-flip C.
 *
 * The same probe is staged twice. Named comctl32_c.exe it loads comctl32_c.dll (the C oracle
 * c_oracle.cmake builds) from its own directory; under any other name it loads the system
 * comctl32.dll (comctl32-rust). It prints VALUES only -- indices, counts, offsets into its
 * own buffers, callback traces -- never a pointer or a handle, so the two runs diff byte for
 * byte. Exports are resolved by ordinal: most of these are -noname in comctl32.spec.
 *
 * Covered: DPA (insert/set/delete/grow/clone, sort, sorted + linear search with the insert
 * flags, merge with every DPAM_ mode, enum/destroy callbacks), DSA (insert/set/get/delete,
 * callbacks), the comctl32 string exports, and ImageList bookkeeping (add/remove/replace/
 * set-count/copy/merge/duplicate, sizes and image rects). Smoke never reaches most of it:
 * explorer only sorts and searches. */
#include <windows.h>
#include <commctrl.h>
#include <stdio.h>
#include <string.h>

typedef struct _DPA *HDPA_;
typedef struct _DSA *HDSA_;
typedef INT(CALLBACK *CMP)(LPVOID, LPVOID, LPARAM);
typedef PVOID(CALLBACK *MERGE)(UINT, PVOID, PVOID, LPARAM);
typedef INT(CALLBACK *ENUM)(LPVOID, LPVOID);

static HMODULE dll;
static FARPROC ord(WORD n)
{
    FARPROC f = GetProcAddress(dll, MAKEINTRESOURCEA(n));
    if (!f)
    {
        printf("missing ordinal %u\n", n);
        fflush(stdout);
        ExitProcess(2);
    }
    return f;
}
#define FN(type, name, n, args) typedef type(WINAPI *name##_t) args; static name##_t p##name;
#define BIND(name, n) p##name = (name##_t)ord(n)

FN(HDPA_, DPA_Create, 328, (INT))
FN(BOOL, DPA_Destroy, 329, (HDPA_))
FN(BOOL, DPA_Grow, 330, (HDPA_, INT))
FN(HDPA_, DPA_Clone, 331, (HDPA_, HDPA_))
FN(LPVOID, DPA_GetPtr, 332, (HDPA_, INT_PTR))
FN(INT, DPA_GetPtrIndex, 333, (HDPA_, LPCVOID))
FN(INT, DPA_InsertPtr, 334, (HDPA_, INT, LPVOID))
FN(BOOL, DPA_SetPtr, 335, (HDPA_, INT, LPVOID))
FN(LPVOID, DPA_DeletePtr, 336, (HDPA_, INT))
FN(BOOL, DPA_DeleteAllPtrs, 337, (HDPA_))
FN(BOOL, DPA_Sort, 338, (HDPA_, CMP, LPARAM))
FN(INT, DPA_Search, 339, (HDPA_, LPVOID, INT, CMP, LPARAM, UINT))
FN(BOOL, DPA_Merge, 11, (HDPA_, HDPA_, DWORD, CMP, MERGE, LPARAM))
FN(void, DPA_EnumCallback, 385, (HDPA_, ENUM, LPVOID))
FN(void, DPA_DestroyCallback, 386, (HDPA_, ENUM, LPVOID))
FN(HDSA_, DSA_Create, 320, (INT, INT))
FN(BOOL, DSA_Destroy, 321, (HDSA_))
FN(BOOL, DSA_GetItem, 322, (HDSA_, INT, LPVOID))
FN(LPVOID, DSA_GetItemPtr, 323, (HDSA_, INT))
FN(INT, DSA_InsertItem, 324, (HDSA_, INT, LPVOID))
FN(BOOL, DSA_SetItem, 325, (HDSA_, INT, LPVOID))
FN(BOOL, DSA_DeleteItem, 326, (HDSA_, INT))
FN(BOOL, DSA_DeleteAllItems, 327, (HDSA_))
FN(void, DSA_EnumCallback, 387, (HDSA_, ENUM, LPVOID))
FN(void, DSA_DestroyCallback, 388, (HDSA_, ENUM, LPVOID))
FN(INT, Str_GetPtrA, 233, (LPCSTR, LPSTR, INT))
FN(INT, Str_GetPtrW, 235, (LPCWSTR, LPWSTR, INT))
FN(LPSTR, StrChrA, 350, (LPCSTR, WORD))
FN(LPSTR, StrRChrA, 351, (LPCSTR, LPCSTR, WORD))
FN(INT, StrCmpNA, 352, (LPCSTR, LPCSTR, INT))
FN(INT, StrCmpNIA, 353, (LPCSTR, LPCSTR, INT))
FN(LPSTR, StrStrA, 354, (LPCSTR, LPCSTR))
FN(LPSTR, StrStrIA, 355, (LPCSTR, LPCSTR))
FN(INT, StrCSpnA, 356, (LPCSTR, LPCSTR))
FN(INT, StrToIntA, 357, (LPCSTR))
FN(LPWSTR, StrChrW, 358, (LPCWSTR, WCHAR))
FN(LPWSTR, StrRChrW, 359, (LPCWSTR, LPCWSTR, WORD))
FN(INT, StrCmpNW, 360, (LPCWSTR, LPCWSTR, INT))
FN(INT, StrCmpNIW, 361, (LPCWSTR, LPCWSTR, INT))
FN(LPWSTR, StrStrW, 362, (LPCWSTR, LPCWSTR))
FN(LPWSTR, StrStrIW, 363, (LPCWSTR, LPCWSTR))
FN(INT, StrCSpnW, 364, (LPCWSTR, LPCWSTR))
FN(INT, StrToIntW, 365, (LPCWSTR))
FN(LPSTR, StrChrIA, 366, (LPCSTR, WORD))
FN(LPWSTR, StrChrIW, 367, (LPCWSTR, WCHAR))
FN(LPSTR, StrRChrIA, 368, (LPCSTR, LPCSTR, WORD))
FN(LPWSTR, StrRChrIW, 369, (LPCWSTR, LPCWSTR, WORD))
FN(LPSTR, StrRStrIA, 372, (LPCSTR, LPCSTR, LPCSTR))
FN(LPWSTR, StrRStrIW, 373, (LPCWSTR, LPCWSTR, LPCWSTR))
FN(INT, StrCSpnIA, 374, (LPCSTR, LPCSTR))
FN(INT, StrCSpnIW, 375, (LPCWSTR, LPCWSTR))
FN(BOOL, IntlStrEqWorkerA, 376, (BOOL, LPCSTR, LPCSTR, INT))
FN(BOOL, IntlStrEqWorkerW, 377, (BOOL, LPCWSTR, LPCWSTR, INT))

typedef HIMAGELIST(WINAPI *IL_Create_t)(INT, INT, UINT, INT, INT);
typedef INT(WINAPI *IL_Add_t)(HIMAGELIST, HBITMAP, HBITMAP);
typedef INT(WINAPI *IL_AddMasked_t)(HIMAGELIST, HBITMAP, COLORREF);
typedef INT(WINAPI *IL_Count_t)(HIMAGELIST);
typedef BOOL(WINAPI *IL_Remove_t)(HIMAGELIST, INT);
typedef BOOL(WINAPI *IL_Replace_t)(HIMAGELIST, INT, HBITMAP, HBITMAP);
typedef BOOL(WINAPI *IL_SetCount_t)(HIMAGELIST, UINT);
typedef BOOL(WINAPI *IL_GetIconSize_t)(HIMAGELIST, INT *, INT *);
typedef BOOL(WINAPI *IL_SetIconSize_t)(HIMAGELIST, INT, INT);
typedef BOOL(WINAPI *IL_GetImageInfo_t)(HIMAGELIST, INT, IMAGEINFO *);
typedef BOOL(WINAPI *IL_Copy_t)(HIMAGELIST, INT, HIMAGELIST, INT, UINT);
typedef HIMAGELIST(WINAPI *IL_Merge_t)(HIMAGELIST, INT, HIMAGELIST, INT, INT, INT);
typedef HIMAGELIST(WINAPI *IL_Duplicate_t)(HIMAGELIST);
typedef BOOL(WINAPI *IL_Destroy_t)(HIMAGELIST);

/* ---- DPA ------------------------------------------------------------------------------- */

static unsigned rng = 12345;
static int next(void)
{
    rng = rng * 1103515245u + 12345u;
    return (int)((rng >> 16) % 1000);
}

static INT CALLBACK cmp_int(LPVOID a, LPVOID b, LPARAM dir)
{
    INT x = (INT)(INT_PTR)a, y = (INT)(INT_PTR)b;
    return dir * ((x > y) - (x < y));
}

static void dump(const char *what, HDPA_ dpa)
{
    INT i = 0;
    printf("%s:", what);
    for (;;)
    {
        LPVOID p = pDPA_GetPtr(dpa, i);
        if (!p && i >= 64)
            break;
        if (!p)
        {
            /* a NULL slot inside the array reads like the end; probe one further */
            if (!pDPA_GetPtr(dpa, i + 1) && !pDPA_GetPtr(dpa, i + 2))
                break;
            printf(" -");
        }
        else
            printf(" %d", (INT)(INT_PTR)p);
        i++;
    }
    printf(" (n=%d)\n", i);
}

static int merge_calls;
static PVOID CALLBACK merge_cb(UINT msg, PVOID a, PVOID b, LPARAM l)
{
    (void)l;
    merge_calls++;
    printf("  merge msg=%u a=%d b=%d\n", msg, (INT)(INT_PTR)a, (INT)(INT_PTR)b);
    if (msg == 1 /* DPAMM_MERGE */)
        return (PVOID)(INT_PTR)((INT)(INT_PTR)a + 1000);
    if (msg == 3 /* DPAMM_INSERT */)
        return (PVOID)(INT_PTR)((INT)(INT_PTR)a + 2000);
    return NULL;
}

static int enum_stop_at;
static INT CALLBACK enum_cb(LPVOID p, LPVOID ctx)
{
    INT *seen = ctx;
    printf(" %d", (INT)(INT_PTR)p);
    return ++*seen != enum_stop_at;
}

static HDPA_ make_dpa(const int *v, int n)
{
    HDPA_ d = pDPA_Create(2);
    for (int i = 0; i < n; i++)
        pDPA_InsertPtr(d, 0x7fffffff, (LPVOID)(INT_PTR)v[i]);
    return d;
}

static void dpa_tests(void)
{
    HDPA_ dpa = pDPA_Create(4), clone;
    INT i, r;

    for (i = 0; i < 40; i++)
    {
        int v = next() + 1, at = (i % 3 == 0) ? 0 : (i % 3 == 1) ? 0x7fffffff : i / 2;
        r = pDPA_InsertPtr(dpa, at, (LPVOID)(INT_PTR)v);
        printf("insert %d at %d -> %d\n", v, at, r);
    }
    printf("insert at -1 -> %d\n", pDPA_InsertPtr(dpa, -1, (LPVOID)7));
    dump("built", dpa);

    printf("getptr -1:%d 40:%d 41:%d 1000:%d\n", pDPA_GetPtr(dpa, -1) != NULL,
           pDPA_GetPtr(dpa, 40) != NULL, pDPA_GetPtr(dpa, 41) != NULL, pDPA_GetPtr(dpa, 1000) != NULL);
    printf("index of 7: %d, of 99999: %d\n", pDPA_GetPtrIndex(dpa, (LPVOID)7),
           pDPA_GetPtrIndex(dpa, (LPVOID)99999));

    printf("sort asc %d\n", pDPA_Sort(dpa, cmp_int, 1));
    dump("asc", dpa);
    for (i = -1; i <= 1001; i += 77)
    {
        INT a = pDPA_Search(dpa, (LPVOID)(INT_PTR)i, 0, cmp_int, 1, DPAS_SORTED);
        INT b = pDPA_Search(dpa, (LPVOID)(INT_PTR)i, 0, cmp_int, 1, DPAS_SORTED | DPAS_INSERTBEFORE);
        INT c = pDPA_Search(dpa, (LPVOID)(INT_PTR)i, 0, cmp_int, 1, DPAS_SORTED | DPAS_INSERTAFTER);
        INT d = pDPA_Search(dpa, (LPVOID)(INT_PTR)i, 5, cmp_int, 1, 0);
        printf("search %d: exact=%d before=%d after=%d linear@5=%d\n", i, a, b, c, d);
    }
    /* every present value, sorted and linear, including duplicates */
    for (i = 0; i < 41; i += 4)
    {
        LPVOID v = pDPA_GetPtr(dpa, i);
        printf("find [%d]: sorted=%d linear=%d\n", i,
               pDPA_Search(dpa, v, 0, cmp_int, 1, DPAS_SORTED),
               pDPA_Search(dpa, v, 0, cmp_int, 1, 0));
    }
    printf("sort desc %d\n", pDPA_Sort(dpa, cmp_int, -1));
    dump("desc", dpa);

    printf("setptr 45 -> %d\n", pDPA_SetPtr(dpa, 45, (LPVOID)4242));
    dump("set45", dpa);
    printf("setptr -1 -> %d\n", pDPA_SetPtr(dpa, -1, (LPVOID)1));
    for (i = 0; i < 6; i++)
    {
        INT at = (i * 7) % 30;
        printf("delete %d -> %d\n", at, (INT)(INT_PTR)pDPA_DeletePtr(dpa, at));
    }
    printf("delete 500 -> %d, delete -1 -> %d\n", (INT)(INT_PTR)pDPA_DeletePtr(dpa, 500),
           (INT)(INT_PTR)pDPA_DeletePtr(dpa, -1));
    dump("deleted", dpa);
    printf("grow %d\n", pDPA_Grow(dpa, 100));

    clone = pDPA_Clone(dpa, NULL);
    dump("clone", clone);
    {
        HDPA_ into = pDPA_Create(1);
        pDPA_InsertPtr(into, 0, (LPVOID)5);
        printf("clone-into same=%d\n", pDPA_Clone(dpa, into) == into);
        dump("clone-into", into);
        pDPA_Destroy(into);
    }

    enum_stop_at = 5;
    {
        INT seen = 0;
        printf("enum(stop@5):");
        pDPA_EnumCallback(dpa, enum_cb, &seen);
        printf(" -> %d\n", seen);
    }
    printf("deleteall %d\n", pDPA_DeleteAllPtrs(dpa));
    dump("empty", dpa);
    printf("search empty: %d %d\n",
           pDPA_Search(dpa, (LPVOID)3, 0, cmp_int, 1, DPAS_SORTED | DPAS_INSERTAFTER),
           pDPA_Search(dpa, (LPVOID)3, 0, cmp_int, 1, 0));
    pDPA_Destroy(dpa);
    enum_stop_at = -1;
    {
        INT seen = 0;
        printf("destroycb:");
        pDPA_DestroyCallback(clone, enum_cb, &seen);
        printf(" -> %d\n", seen);
    }

    /* DPA_Merge: both inputs sorted; every mode, and a callback trace of each step */
    {
        static const int a[] = { 1, 3, 5, 7, 9, 11, 20 }, b[] = { 2, 3, 4, 9, 10, 20, 30 };
        static const DWORD modes[] = { DPAM_UNION, DPAM_INTERSECT, DPAM_NORMAL,
                                       DPAM_SORTED | DPAM_UNION, DPAM_SORTED | DPAM_INTERSECT,
                                       DPAM_SORTED };
        for (i = 0; i < (INT)(sizeof(modes) / sizeof(modes[0])); i++)
        {
            HDPA_ x = make_dpa(a, 7), y = make_dpa(b, 7);
            BOOL ok;
            if (!(modes[i] & DPAM_SORTED))
            {
                pDPA_Sort(x, cmp_int, 1);
                pDPA_Sort(y, cmp_int, 1);
            }
            merge_calls = 0;
            printf("merge mode=%#lx\n", modes[i]);
            ok = pDPA_Merge(x, y, modes[i], cmp_int, merge_cb, 1);
            printf("  -> %d calls=%d\n", ok, merge_calls);
            dump("  merged", x);
            pDPA_Destroy(x);
            pDPA_Destroy(y);
        }
    }
}

/* ---- DSA ------------------------------------------------------------------------------- */

typedef struct
{
    INT id;
    char tag[6];
} ITEM;

static INT CALLBACK dsa_cb(LPVOID p, LPVOID ctx)
{
    ITEM *it = p;
    INT *seen = ctx;
    printf(" %d:%s", it->id, it->tag);
    return ++*seen != enum_stop_at;
}

static void dsa_dump(const char *what, HDSA_ dsa)
{
    INT i;
    printf("%s:", what);
    for (i = 0;; i++)
    {
        ITEM *it = pDSA_GetItemPtr(dsa, i);
        if (!it)
            break;
        printf(" %d:%s", it->id, it->tag);
    }
    printf(" (n=%d)\n", i);
}

static void dsa_tests(void)
{
    HDSA_ dsa = pDSA_Create(sizeof(ITEM), 3);
    ITEM it, out;
    INT i;

    for (i = 0; i < 12; i++)
    {
        INT at = (i % 4 == 0) ? 0 : (i % 4 == 1) ? 0x7fffffff : (i % 4 == 2) ? i / 2 : 1000;
        memset(&it, 0, sizeof it);
        it.id = i * 10;
        sprintf(it.tag, "t%d", i);
        printf("dsa insert %d at %d -> %d\n", it.id, at, pDSA_InsertItem(dsa, at, &it));
    }
    printf("dsa insert at -1 -> %d\n", pDSA_InsertItem(dsa, -1, &it));
    dsa_dump("dsa", dsa);
    memset(&out, 0, sizeof out);
    printf("getitem 3 -> %d id=%d, getitem 99 -> %d, getitem -1 -> %d\n",
           pDSA_GetItem(dsa, 3, &out), out.id, pDSA_GetItem(dsa, 99, &out),
           pDSA_GetItem(dsa, -1, &out));
    it.id = 777;
    strcpy(it.tag, "set");
    printf("setitem 2 -> %d, setitem 20 -> %d, setitem -1 -> %d\n", pDSA_SetItem(dsa, 2, &it),
           pDSA_SetItem(dsa, 20, &it), pDSA_SetItem(dsa, -1, &it));
    dsa_dump("dsa-set", dsa);
    printf("deleteitem 0 -> %d, 5 -> %d, 100 -> %d, -1 -> %d\n", pDSA_DeleteItem(dsa, 0),
           pDSA_DeleteItem(dsa, 5), pDSA_DeleteItem(dsa, 100), pDSA_DeleteItem(dsa, -1));
    dsa_dump("dsa-del", dsa);
    enum_stop_at = 4;
    {
        INT seen = 0;
        printf("dsa enum(stop@4):");
        pDSA_EnumCallback(dsa, dsa_cb, &seen);
        printf(" -> %d\n", seen);
    }
    printf("dsa deleteall %d\n", pDSA_DeleteAllItems(dsa));
    dsa_dump("dsa-empty", dsa);
    pDSA_InsertItem(dsa, 0, &it);
    enum_stop_at = -1;
    {
        INT seen = 0;
        printf("dsa destroycb:");
        pDSA_DestroyCallback(dsa, dsa_cb, &seen);
        printf(" -> %d\n", seen);
    }
}

/* ---- strings --------------------------------------------------------------------------- */

static int offA(LPCSTR base, LPCSTR p) { return p ? (int)(p - base) : -1; }
static int offW(LPCWSTR base, LPCWSTR p) { return p ? (int)(p - base) : -1; }

static void str_tests(void)
{
    static const char *hay[] = { "", "a", "Hello, World", "abcABCabc", "aaaa", "x;y,z", "  -42xyz", "123" };
    static const char *needle[] = { "", "a", "ABC", "l", "o, w", "zz", ";,", "aa" };
    char buf[16];
    WCHAR wbuf[16];
    INT i, j, n;

    for (i = 0; i < 8; i++)
    {
        const char *h = hay[i];
        WCHAR wh[32], wn[32];
        MultiByteToWideChar(CP_ACP, 0, h, -1, wh, 32);
        printf("[%s] toint=%d/%d chr(l)=%d/%d chri(A)=%d/%d rchr(a)=%d/%d rchri(B)=%d/%d\n", h,
               pStrToIntA(h), pStrToIntW(wh), offA(h, pStrChrA(h, 'l')), offW(wh, pStrChrW(wh, 'l')),
               offA(h, pStrChrIA(h, 'A')), offW(wh, pStrChrIW(wh, 'A')),
               offA(h, pStrRChrA(h, NULL, 'a')), offW(wh, pStrRChrW(wh, NULL, 'a')),
               offA(h, pStrRChrIA(h, NULL, 'B')), offW(wh, pStrRChrIW(wh, NULL, 'b')));
        for (j = 0; j < 8; j++)
        {
            const char *nd = needle[j];
            MultiByteToWideChar(CP_ACP, 0, nd, -1, wn, 32);
            printf("  [%s] str=%d/%d stri=%d/%d rstri=%d/%d rstri(end)=%d/%d cspn=%d/%d cspni=%d/%d"
                   " cmpn3=%d/%d cmpni3=%d/%d eq=%d/%d eqi=%d/%d\n",
                   nd, offA(h, pStrStrA(h, nd)), offW(wh, pStrStrW(wh, wn)),
                   offA(h, pStrStrIA(h, nd)), offW(wh, pStrStrIW(wh, wn)),
                   offA(h, pStrRStrIA(h, NULL, nd)), offW(wh, pStrRStrIW(wh, NULL, wn)),
                   offA(h, pStrRStrIA(h, h + strlen(h) / 2, nd)),
                   offW(wh, pStrRStrIW(wh, wh + lstrlenW(wh) / 2, wn)),
                   pStrCSpnA(h, nd), pStrCSpnW(wh, wn), pStrCSpnIA(h, nd), pStrCSpnIW(wh, wn),
                   pStrCmpNA(h, nd, 3) < 0 ? -1 : pStrCmpNA(h, nd, 3) > 0,
                   pStrCmpNW(wh, wn, 3) < 0 ? -1 : pStrCmpNW(wh, wn, 3) > 0,
                   pStrCmpNIA(h, nd, 3) < 0 ? -1 : pStrCmpNIA(h, nd, 3) > 0,
                   pStrCmpNIW(wh, wn, 3) < 0 ? -1 : pStrCmpNIW(wh, wn, 3) > 0,
                   pIntlStrEqWorkerA(TRUE, h, nd, 2), pIntlStrEqWorkerW(TRUE, wh, wn, 2),
                   pIntlStrEqWorkerA(FALSE, h, nd, 2), pIntlStrEqWorkerW(FALSE, wh, wn, 2));
        }
    }
    printf("null: str=%d chr=%d rchr=%d rstri=%d cspn=%d/%d\n", offA("", pStrStrA(NULL, "a")),
           offA("", pStrChrA(NULL, 'a')), offA("", pStrRChrA(NULL, NULL, 'a')),
           offA("", pStrRStrIA(NULL, NULL, "a")), pStrCSpnA(NULL, "a"), pStrCSpnW(NULL, L"a"));

    for (n = 0; n <= 14; n += (n < 4 ? 1 : 5))
    {
        memset(buf, '#', sizeof buf);
        for (i = 0; i < 16; i++)
            wbuf[i] = '#';
        i = pStr_GetPtrA("comctl32", buf, n);
        j = pStr_GetPtrW(L"comctl32", wbuf, n);
        buf[15] = 0;
        wbuf[15] = 0;
        printf("getptr n=%d: A=%d [%.15s] W=%d [", n, i, buf, j);
        for (i = 0; i < 15; i++)
            putchar(wbuf[i] ? (char)wbuf[i] : '0');
        printf("]\n");
    }
    printf("getptr nodest: A=%d W=%d; nullsrc: A=%d W=%d\n", pStr_GetPtrA("abc", NULL, 5),
           pStr_GetPtrW(L"abc", NULL, 5), pStr_GetPtrA(NULL, buf, 5), pStr_GetPtrW(NULL, wbuf, 5));
}

/* ---- ImageList ------------------------------------------------------------------------- */

static void il_dump(const char *what, HIMAGELIST il, IL_Count_t count, IL_GetIconSize_t size,
                    IL_GetImageInfo_t info)
{
    INT cx = -1, cy = -1, n = count(il), i;
    size(il, &cx, &cy);
    printf("%s: n=%d size=%dx%d rects:", what, n, cx, cy);
    for (i = 0; i < n && i < 12; i++)
    {
        IMAGEINFO ii;
        memset(&ii, 0, sizeof ii);
        if (info(il, i, &ii))
            printf(" (%ld,%ld,%ld,%ld)%c", ii.rcImage.left, ii.rcImage.top, ii.rcImage.right,
                   ii.rcImage.bottom, ii.hbmMask ? 'm' : '-');
        else
            printf(" x");
    }
    printf("\n");
}

static void il_tests(void)
{
    IL_Create_t create = (IL_Create_t)GetProcAddress(dll, "ImageList_Create");
    IL_Add_t add = (IL_Add_t)GetProcAddress(dll, "ImageList_Add");
    IL_AddMasked_t addm = (IL_AddMasked_t)GetProcAddress(dll, "ImageList_AddMasked");
    IL_Count_t count = (IL_Count_t)GetProcAddress(dll, "ImageList_GetImageCount");
    IL_Remove_t rem = (IL_Remove_t)GetProcAddress(dll, "ImageList_Remove");
    IL_Replace_t rep = (IL_Replace_t)GetProcAddress(dll, "ImageList_Replace");
    IL_SetCount_t setcount = (IL_SetCount_t)GetProcAddress(dll, "ImageList_SetImageCount");
    IL_GetIconSize_t gsize = (IL_GetIconSize_t)GetProcAddress(dll, "ImageList_GetIconSize");
    IL_SetIconSize_t ssize = (IL_SetIconSize_t)GetProcAddress(dll, "ImageList_SetIconSize");
    IL_GetImageInfo_t info = (IL_GetImageInfo_t)GetProcAddress(dll, "ImageList_GetImageInfo");
    IL_Copy_t copy = (IL_Copy_t)GetProcAddress(dll, "ImageList_Copy");
    IL_Merge_t merge = (IL_Merge_t)GetProcAddress(dll, "ImageList_Merge");
    IL_Duplicate_t dup = (IL_Duplicate_t)GetProcAddress(dll, "ImageList_Duplicate");
    IL_Destroy_t destroy = (IL_Destroy_t)GetProcAddress(dll, "ImageList_Destroy");
    static const UINT flags[] = { ILC_COLOR32 | ILC_MASK, ILC_COLOR24, ILC_COLOR4 | ILC_MASK };
    INT f, i;

    if (!create || !add || !addm || !count || !rem || !rep || !setcount || !gsize || !ssize ||
        !info || !copy || !merge || !dup || !destroy)
    {
        printf("imagelist exports missing\n");
        return;
    }
    for (f = 0; f < 3; f++)
    {
        HIMAGELIST il = create(16, 16, flags[f], 2, 3), d, m;
        HDC dc = GetDC(NULL);
        HBITMAP strip = CreateCompatibleBitmap(dc, 16 * 5, 16), one = CreateCompatibleBitmap(dc, 16, 16);
        HBITMAP odd = CreateCompatibleBitmap(dc, 40, 16), mask = CreateBitmap(16 * 5, 16, 1, 1, NULL);
        ReleaseDC(NULL, dc);
        printf("flags %#x created=%d\n", flags[f], il != NULL);
        printf(" add strip -> %d\n", add(il, strip, (flags[f] & ILC_MASK) ? mask : NULL));
        printf(" add one -> %d\n", add(il, one, NULL));
        printf(" add odd(40px) -> %d\n", add(il, odd, NULL));
        printf(" addmasked -> %d\n", addm(il, one, RGB(255, 0, 255)));
        il_dump(" after-add", il, count, gsize, info);
        printf(" remove 1 -> %d, remove 50 -> %d\n", rem(il, 1), rem(il, 50));
        printf(" replace 0 -> %d, replace 40 -> %d\n", rep(il, 0, one, NULL), rep(il, 40, one, NULL));
        il_dump(" after-rr", il, count, gsize, info);
        printf(" copy 0->2 -> %d, swap 1<->3 -> %d\n", copy(il, 2, il, 0, ILCF_MOVE),
               copy(il, 1, il, 3, ILCF_SWAP));
        printf(" setcount 13 -> %d\n", setcount(il, 13));
        il_dump(" after-setcount", il, count, gsize, info);
        printf(" setcount 3 -> %d\n", setcount(il, 3));
        il_dump(" after-shrink", il, count, gsize, info);
        d = dup(il);
        il_dump(" dup", d, count, gsize, info);
        m = merge(il, 0, d, 1, 4, 3);
        if (m)
            il_dump(" merge", m, count, gsize, info);
        else
            printf(" merge failed\n");
        printf(" seticonsize 24x8 -> %d\n", ssize(il, 24, 8));
        il_dump(" resized", il, count, gsize, info);
        for (i = 0; i < 4; i++)
            printf(" add after resize -> %d\n", add(il, one, NULL));
        il_dump(" readd", il, count, gsize, info);
        printf(" remove all -> %d\n", rem(il, -1));
        il_dump(" cleared", il, count, gsize, info);
        if (m)
            destroy(m);
        destroy(d);
        destroy(il);
        DeleteObject(strip);
        DeleteObject(one);
        DeleteObject(odd);
        DeleteObject(mask);
    }
}

int main(void)
{
    WCHAR self[MAX_PATH], *slash;
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);

    /* comctl32_c.exe -> its sibling comctl32_c.dll (the C oracle); anything else -> the
     * system comctl32 (what ships). */
    if (len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe"))
    {
        slash = wcsrchr(self, L'\\');
        lstrcpyW(slash + 1, L"comctl32_c.dll");
        dll = LoadLibraryW(self);
    }
    else
        dll = LoadLibraryW(L"comctl32.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }

    BIND(DPA_Create, 328); BIND(DPA_Destroy, 329); BIND(DPA_Grow, 330); BIND(DPA_Clone, 331);
    BIND(DPA_GetPtr, 332); BIND(DPA_GetPtrIndex, 333); BIND(DPA_InsertPtr, 334);
    BIND(DPA_SetPtr, 335); BIND(DPA_DeletePtr, 336); BIND(DPA_DeleteAllPtrs, 337);
    BIND(DPA_Sort, 338); BIND(DPA_Search, 339); BIND(DPA_Merge, 11);
    BIND(DPA_EnumCallback, 385); BIND(DPA_DestroyCallback, 386);
    BIND(DSA_Create, 320); BIND(DSA_Destroy, 321); BIND(DSA_GetItem, 322);
    BIND(DSA_GetItemPtr, 323); BIND(DSA_InsertItem, 324); BIND(DSA_SetItem, 325);
    BIND(DSA_DeleteItem, 326); BIND(DSA_DeleteAllItems, 327); BIND(DSA_EnumCallback, 387);
    BIND(DSA_DestroyCallback, 388);
    BIND(Str_GetPtrA, 233); BIND(Str_GetPtrW, 235); BIND(StrChrA, 350); BIND(StrRChrA, 351);
    BIND(StrCmpNA, 352); BIND(StrCmpNIA, 353); BIND(StrStrA, 354); BIND(StrStrIA, 355);
    BIND(StrCSpnA, 356); BIND(StrToIntA, 357); BIND(StrChrW, 358); BIND(StrRChrW, 359);
    BIND(StrCmpNW, 360); BIND(StrCmpNIW, 361); BIND(StrStrW, 362); BIND(StrStrIW, 363);
    BIND(StrCSpnW, 364); BIND(StrToIntW, 365); BIND(StrChrIA, 366); BIND(StrChrIW, 367);
    BIND(StrRChrIA, 368); BIND(StrRChrIW, 369); BIND(StrRStrIA, 372); BIND(StrRStrIW, 373);
    BIND(StrCSpnIA, 374); BIND(StrCSpnIW, 375); BIND(IntlStrEqWorkerA, 376);
    BIND(IntlStrEqWorkerW, 377);

    dpa_tests();
    fflush(stdout);
    dsa_tests();
    fflush(stdout);
    str_tests();
    fflush(stdout);
    il_tests();
    printf("probe done\n");
    fflush(stdout);
    return 0;
}
