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
 * set-count/copy/merge/duplicate, sizes and image rects), and the treeview and listview
 * driven by message from a hidden parent: navigation (every TVGN_/LVNI_ code, including
 * `TVGN_CARET | TVSI_NOSINGLEEXPAND`, the flag combination a translation once turned into a
 * Rust or-pattern), expand/collapse, sorting with callbacks, find, selection state and the
 * WM_NOTIFY stream each operation sends. Smoke never reaches most of it: explorer only
 * sorts and searches, and never navigates a tree. */
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

/* ---- window controls: a hidden parent records the WM_NOTIFY stream ---- */

static HWND tree, list;
static int notify_quiet;

static void tv_text(HTREEITEM h, char *out, int cch)
{
    WCHAR w[64];
    TVITEMW it = {0};
    if (!h)
    {
        lstrcpynA(out, "(null)", cch);
        return;
    }
    it.mask = TVIF_TEXT | TVIF_HANDLE;
    it.hItem = h;
    it.pszText = w;
    it.cchTextMax = 64;
    w[0] = 0;
    if (!SendMessageW(tree, TVM_GETITEMW, 0, (LPARAM)&it))
    {
        lstrcpynA(out, "(bad)", cch);
        return;
    }
    WideCharToMultiByte(CP_ACP, 0, w, -1, out, cch, NULL, NULL);
}

static LRESULT CALLBACK parent_proc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp)
{
    if (msg == WM_NOTIFY && !notify_quiet)
    {
        NMHDR *hdr = (NMHDR *)lp;
        char a[32], b[32];
        if (hdr->hwndFrom == tree)
        {
            NMTREEVIEWW *nm = (NMTREEVIEWW *)lp;
            switch (hdr->code)
            {
            case TVN_SELCHANGINGW: case TVN_SELCHANGEDW:
            case TVN_ITEMEXPANDINGW: case TVN_ITEMEXPANDEDW:
                tv_text(nm->itemOld.hItem, a, sizeof(a));
                tv_text(nm->itemNew.hItem, b, sizeof(b));
                printf("  tvn %d action=%u old=%s new=%s newstate=0x%x\n", (int)hdr->code,
                       nm->action, a, b, nm->itemNew.state & 0xffff);
                break;
            case TVN_DELETEITEMW:
                printf("  tvn delete lparam=%ld\n", (long)nm->itemOld.lParam);
                break;
            }
        }
        else if (hdr->hwndFrom == list)
        {
            NMLISTVIEW *nm = (NMLISTVIEW *)lp;
            switch (hdr->code)
            {
            case LVN_ITEMCHANGING: case LVN_ITEMCHANGED:
                printf("  lvn %d item=%d sub=%d new=0x%x old=0x%x changed=0x%x\n",
                       (int)hdr->code, nm->iItem, nm->iSubItem, nm->uNewState, nm->uOldState,
                       nm->uChanged);
                break;
            case LVN_INSERTITEM: case LVN_DELETEITEM: case LVN_DELETEALLITEMS:
                printf("  lvn %d item=%d lparam=%ld\n", (int)hdr->code, nm->iItem,
                       (long)nm->lParam);
                break;
            }
        }
        fflush(stdout);
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

static HTREEITEM tv_insert(HTREEITEM parent, HTREEITEM after, const WCHAR *text, LPARAM lp)
{
    TVINSERTSTRUCTW is = {0};
    is.hParent = parent;
    is.hInsertAfter = after;
    is.item.mask = TVIF_TEXT | TVIF_PARAM;
    is.item.pszText = (LPWSTR)text;
    is.item.lParam = lp;
    return (HTREEITEM)SendMessageW(tree, TVM_INSERTITEMW, 0, (LPARAM)&is);
}

static void tv_walk(const char *what)
{
    /* Depth-first over every item, children whether or not expanded. */
    HTREEITEM stack[32], h = (HTREEITEM)SendMessageW(tree, TVM_GETNEXTITEM, TVGN_ROOT, 0);
    int depth = 0;
    char s[32];
    printf("%s count=%ld:", what, (long)SendMessageW(tree, TVM_GETCOUNT, 0, 0));
    while (h)
    {
        HTREEITEM child = (HTREEITEM)SendMessageW(tree, TVM_GETNEXTITEM, TVGN_CHILD, (LPARAM)h);
        tv_text(h, s, sizeof(s));
        printf(" %d:%s/0x%x", depth, s,
               (unsigned)SendMessageW(tree, TVM_GETITEMSTATE, (WPARAM)h, 0xffff));
        if (child && depth < 31)
        {
            stack[depth++] = h;
            h = child;
            continue;
        }
        while (h && !(child = (HTREEITEM)SendMessageW(tree, TVM_GETNEXTITEM, TVGN_NEXT, (LPARAM)h)))
            h = depth ? stack[--depth] : NULL;
        h = h ? child : NULL;
    }
    printf("\n");
}

static void tv_nav(const char *what, UINT code, HTREEITEM from)
{
    char s[32];
    tv_text((HTREEITEM)SendMessageW(tree, TVM_GETNEXTITEM, code, (LPARAM)from), s, sizeof(s));
    printf("tv next %s = %s\n", what, s);
}

static int CALLBACK tv_sort_cb(LPARAM a, LPARAM b, LPARAM dir)
{
    return dir ? (int)(b - a) : (int)(a - b);
}

static void tree_tests(HWND parent)
{
    HTREEITEM a, b, c, a1, a2, a3, b1;
    char s[32];
    TVSORTCB sort;

    tree = CreateWindowExW(0, WC_TREEVIEWW, L"", WS_CHILD | TVS_HASBUTTONS | TVS_LINESATROOT,
                           0, 0, 200, 300, parent, NULL, NULL, NULL);
    printf("tv created %d\n", tree != NULL);
    if (!tree)
        return;
    a = tv_insert(TVI_ROOT, TVI_LAST, L"alpha", 10);
    c = tv_insert(TVI_ROOT, TVI_LAST, L"charlie", 30);
    b = tv_insert(TVI_ROOT, TVI_SORT, L"bravo", 20);
    a2 = tv_insert(a, TVI_LAST, L"a2", 12);
    a1 = tv_insert(a, TVI_FIRST, L"a1", 11);
    a3 = tv_insert(a, a2, L"a3", 13);
    b1 = tv_insert(b, TVI_LAST, L"b1", 21);
    tv_walk("tv built");

    tv_nav("root", TVGN_ROOT, NULL);
    tv_nav("alpha.next", TVGN_NEXT, a);
    tv_nav("charlie.prev", TVGN_PREVIOUS, c);
    tv_nav("a3.parent", TVGN_PARENT, a3);
    tv_nav("alpha.child", TVGN_CHILD, a);
    tv_nav("charlie.next", TVGN_NEXT, c);
    tv_nav("firstvisible", TVGN_FIRSTVISIBLE, NULL);
    tv_nav("alpha.nextvisible", TVGN_NEXTVISIBLE, a);
    tv_nav("caret", TVGN_CARET, NULL);
    tv_nav("lastvisible", TVGN_LASTVISIBLE, NULL);

    printf("tv expand alpha %ld\n", (long)SendMessageW(tree, TVM_EXPAND, TVE_EXPAND, (LPARAM)a));
    tv_nav("alpha.nextvisible", TVGN_NEXTVISIBLE, a);
    tv_nav("bravo.prevvisible", TVGN_PREVIOUSVISIBLE, b);
    printf("tv toggle bravo %ld\n", (long)SendMessageW(tree, TVM_EXPAND, TVE_TOGGLE, (LPARAM)b));
    printf("tv collapse alpha %ld\n",
           (long)SendMessageW(tree, TVM_EXPAND, TVE_COLLAPSE, (LPARAM)a));
    tv_walk("tv expanded");

    printf("tv select a1 %ld\n", (long)SendMessageW(tree, TVM_SELECTITEM, TVGN_CARET, (LPARAM)a1));
    tv_nav("caret", TVGN_CARET, NULL);
    printf("tv select b1 nosingle %ld\n",
           (long)SendMessageW(tree, TVM_SELECTITEM, TVGN_CARET | TVSI_NOSINGLEEXPAND, (LPARAM)b1));
    tv_nav("caret", TVGN_CARET, NULL);
    printf("tv drophilite charlie %ld\n",
           (long)SendMessageW(tree, TVM_SELECTITEM, TVGN_DROPHILITE, (LPARAM)c));
    tv_nav("drophilite", TVGN_DROPHILITE, NULL);
    printf("tv select null %ld\n", (long)SendMessageW(tree, TVM_SELECTITEM, TVGN_CARET, 0));
    tv_nav("caret", TVGN_CARET, NULL);
    tv_walk("tv selected");

    sort.hParent = a;
    sort.lpfnCompare = tv_sort_cb;
    sort.lParam = 1;
    printf("tv sortcb alpha desc %ld\n", (long)SendMessageW(tree, TVM_SORTCHILDRENCB, 0, (LPARAM)&sort));
    tv_walk("tv sorted desc");
    printf("tv sort alpha %ld\n", (long)SendMessageW(tree, TVM_SORTCHILDREN, 0, (LPARAM)a));
    tv_walk("tv sorted text");

    {
        TVITEMW it = {0};
        it.mask = TVIF_TEXT | TVIF_STATE | TVIF_HANDLE;
        it.hItem = a2;
        it.pszText = (LPWSTR)L"a2-renamed";
        it.state = TVIS_BOLD;
        it.stateMask = TVIS_BOLD;
        printf("tv setitem %ld\n", (long)SendMessageW(tree, TVM_SETITEMW, 0, (LPARAM)&it));
        it.mask = TVIF_PARAM | TVIF_HANDLE;
        SendMessageW(tree, TVM_GETITEMW, 0, (LPARAM)&it);
        tv_text(a2, s, sizeof(s));
        printf("tv getitem %s lparam=%ld\n", s, (long)it.lParam);
    }
    printf("tv indent set %ld\n", (long)SendMessageW(tree, TVM_SETINDENT, 25, 0));
    printf("tv indent %ld\n", (long)SendMessageW(tree, TVM_GETINDENT, 0, 0));
    printf("tv itemheight old %ld\n", (long)SendMessageW(tree, TVM_SETITEMHEIGHT, 21, 0));
    printf("tv itemheight %ld\n", (long)SendMessageW(tree, TVM_GETITEMHEIGHT, 0, 0));

    printf("tv delete alpha %ld\n", (long)SendMessageW(tree, TVM_DELETEITEM, 0, (LPARAM)a));
    tv_walk("tv deleted");
    printf("tv deleteall %ld\n", (long)SendMessageW(tree, TVM_DELETEITEM, 0, (LPARAM)TVI_ROOT));
    tv_walk("tv empty");
    notify_quiet = 1;
    DestroyWindow(tree);
    notify_quiet = 0;
}

static int lv_insert(int at, const WCHAR *text, LPARAM lp)
{
    LVITEMW it = {0};
    it.mask = LVIF_TEXT | LVIF_PARAM;
    it.iItem = at;
    it.pszText = (LPWSTR)text;
    it.lParam = lp;
    return (int)SendMessageW(list, LVM_INSERTITEMW, 0, (LPARAM)&it);
}

static void lv_dump(const char *what)
{
    int i, n = (int)SendMessageW(list, LVM_GETITEMCOUNT, 0, 0);
    printf("%s count=%d sel=%ld:", what, n, (long)SendMessageW(list, LVM_GETSELECTEDCOUNT, 0, 0));
    for (i = 0; i < n; i++)
    {
        WCHAR w[32], w2[32];
        char s[32], s2[32];
        LVITEMW it = {0};
        it.iSubItem = 0;
        it.pszText = w;
        it.cchTextMax = 32;
        w[0] = 0;
        SendMessageW(list, LVM_GETITEMTEXTW, i, (LPARAM)&it);
        it.iSubItem = 1;
        it.pszText = w2;
        w2[0] = 0;
        SendMessageW(list, LVM_GETITEMTEXTW, i, (LPARAM)&it);
        it.mask = LVIF_PARAM;
        it.iItem = i;
        it.iSubItem = 0;
        SendMessageW(list, LVM_GETITEMW, 0, (LPARAM)&it);
        WideCharToMultiByte(CP_ACP, 0, w, -1, s, sizeof(s), NULL, NULL);
        WideCharToMultiByte(CP_ACP, 0, w2, -1, s2, sizeof(s2), NULL, NULL);
        printf(" %s|%s/%ld/0x%x", s, s2, (long)it.lParam,
               (unsigned)SendMessageW(list, LVM_GETITEMSTATE, i, LVIS_SELECTED | LVIS_FOCUSED));
    }
    printf("\n");
}

static int CALLBACK lv_sort_param(LPARAM a, LPARAM b, LPARAM dir)
{
    return dir ? (int)(b - a) : (int)(a - b);
}

static int CALLBACK lv_sort_index(LPARAM a, LPARAM b, LPARAM l)
{
    /* SORTITEMSEX passes indices: order by the item's text, reversed. */
    WCHAR wa[32], wb[32];
    LVITEMW it = {0};
    it.pszText = wa;
    it.cchTextMax = 32;
    SendMessageW(list, LVM_GETITEMTEXTW, a, (LPARAM)&it);
    it.pszText = wb;
    SendMessageW(list, LVM_GETITEMTEXTW, b, (LPARAM)&it);
    return lstrcmpW(wb, wa);
}

static void lv_find(const char *what, UINT flags, const WCHAR *text, LPARAM lp, int start)
{
    LVFINDINFOW fi = {0};
    fi.flags = flags;
    fi.psz = text;
    fi.lParam = lp;
    printf("lv find %s = %ld\n", what, (long)SendMessageW(list, LVM_FINDITEMW, start, (LPARAM)&fi));
}

static void lv_select(int i, UINT state, UINT mask)
{
    LVITEMW it = {0};
    it.state = state;
    it.stateMask = mask;
    printf("lv setstate %d 0x%x/0x%x %ld\n", i, state, mask,
           (long)SendMessageW(list, LVM_SETITEMSTATE, i, (LPARAM)&it));
}

static void list_tests(HWND parent)
{
    static const WCHAR *names[] = { L"pear", L"apple", L"fig", L"banana", L"cherry", L"apricot" };
    static const WCHAR *sizes[] = { L"3", L"10", L"7", L"1", L"5", L"2" };
    LVCOLUMNW col = {0};
    int i;

    list = CreateWindowExW(0, WC_LISTVIEWW, L"", WS_CHILD | LVS_REPORT | LVS_SHOWSELALWAYS,
                           0, 0, 300, 200, parent, NULL, NULL, NULL);
    printf("lv created %d\n", list != NULL);
    if (!list)
        return;
    col.mask = LVCF_TEXT | LVCF_WIDTH;
    col.cx = 100;
    col.pszText = (LPWSTR)L"Name";
    printf("lv col0 %ld\n", (long)SendMessageW(list, LVM_INSERTCOLUMNW, 0, (LPARAM)&col));
    col.pszText = (LPWSTR)L"Size";
    col.cx = 60;
    printf("lv col1 %ld\n", (long)SendMessageW(list, LVM_INSERTCOLUMNW, 1, (LPARAM)&col));
    printf("lv header items %ld\n",
           (long)SendMessageW((HWND)SendMessageW(list, LVM_GETHEADER, 0, 0), HDM_GETITEMCOUNT, 0, 0));
    for (i = 0; i < 6; i++)
    {
        LVITEMW it = {0};
        int at = lv_insert(i % 2 ? 0 : 0x7fff, names[i], (i + 1) * 100);
        it.iSubItem = 1;
        it.pszText = (LPWSTR)sizes[i];
        printf("lv insert %d at %d subitem %ld\n", i, at,
               (long)SendMessageW(list, LVM_SETITEMTEXTW, at, (LPARAM)&it));
    }
    lv_dump("lv built");

    lv_find("string fig", LVFI_STRING, L"fig", 0, -1);
    lv_find("string FIG", LVFI_STRING, L"FIG", 0, -1);
    lv_find("partial ap", LVFI_PARTIAL | LVFI_STRING, L"ap", 0, -1);
    lv_find("partial ap from 1", LVFI_PARTIAL | LVFI_STRING, L"ap", 0, 1);
    lv_find("partial ap wrap from 5", LVFI_PARTIAL | LVFI_STRING | LVFI_WRAP, L"ap", 0, 5);
    lv_find("partial ap nowrap from 5", LVFI_PARTIAL | LVFI_STRING, L"ap", 0, 5);
    lv_find("param 400", LVFI_PARAM, NULL, 400, -1);
    lv_find("param 999", LVFI_PARAM, NULL, 999, -1);

    lv_select(1, LVIS_SELECTED | LVIS_FOCUSED, LVIS_SELECTED | LVIS_FOCUSED);
    lv_select(3, LVIS_SELECTED, LVIS_SELECTED);
    lv_dump("lv selected");
    printf("lv next selected from -1 %ld\n", (long)SendMessageW(list, LVM_GETNEXTITEM, -1, LVNI_SELECTED));
    printf("lv next selected from 1 %ld\n", (long)SendMessageW(list, LVM_GETNEXTITEM, 1, LVNI_SELECTED));
    printf("lv next focused %ld\n", (long)SendMessageW(list, LVM_GETNEXTITEM, -1, LVNI_FOCUSED));
    printf("lv next all from 2 %ld\n", (long)SendMessageW(list, LVM_GETNEXTITEM, 2, LVNI_ALL));
    printf("lv next below 2 %ld\n", (long)SendMessageW(list, LVM_GETNEXTITEM, 2, LVNI_BELOW));
    printf("lv next above 2 %ld\n", (long)SendMessageW(list, LVM_GETNEXTITEM, 2, LVNI_ABOVE));
    printf("lv next selected|below 0 %ld\n",
           (long)SendMessageW(list, LVM_GETNEXTITEM, 0, LVNI_SELECTED | LVNI_BELOW));
    lv_select(-1, 0, LVIS_SELECTED);
    lv_dump("lv deselected all");

    printf("lv sort param desc %ld\n", (long)SendMessageW(list, LVM_SORTITEMS, 1, (LPARAM)lv_sort_param));
    lv_dump("lv sorted param");
    printf("lv sortex text desc %ld\n", (long)SendMessageW(list, LVM_SORTITEMSEX, 0, (LPARAM)lv_sort_index));
    lv_dump("lv sorted text");

    printf("lv colwidth %ld\n", (long)SendMessageW(list, LVM_GETCOLUMNWIDTH, 1, 0));
    printf("lv setcolwidth %ld\n", (long)SendMessageW(list, LVM_SETCOLUMNWIDTH, 1, 75));
    printf("lv colwidth %ld\n", (long)SendMessageW(list, LVM_GETCOLUMNWIDTH, 1, 0));
    printf("lv exstyle old 0x%lx\n", (long)SendMessageW(list, LVM_SETEXTENDEDLISTVIEWSTYLE,
           LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES, LVS_EX_FULLROWSELECT));
    printf("lv exstyle 0x%lx\n", (long)SendMessageW(list, LVM_GETEXTENDEDLISTVIEWSTYLE, 0, 0));

    printf("lv delete 2 %ld\n", (long)SendMessageW(list, LVM_DELETEITEM, 2, 0));
    printf("lv delete 99 %ld\n", (long)SendMessageW(list, LVM_DELETEITEM, 99, 0));
    lv_dump("lv deleted");
    printf("lv deletecol 0 %ld\n", (long)SendMessageW(list, LVM_DELETECOLUMN, 0, 0));
    lv_dump("lv col deleted");
    printf("lv deleteall %ld\n", (long)SendMessageW(list, LVM_DELETEALLITEMS, 0, 0));
    lv_dump("lv empty");
    notify_quiet = 1;
    DestroyWindow(list);
    notify_quiet = 0;
}

/* The taskbar's geometry, as explorer builds it: the task switcher's toolbar (taskswnd.cpp),
 * then browseui's band site rebar (CBandSite.cpp) holding it as a RBBS_VARIABLEHEIGHT band
 * (taskband.cpp GetBandInfo), then FitToRebar's RB_SIZETORECT/RBSTR_CHANGERECT (traywnd.cpp).
 * comctl32-rust shipped a taskbar 8 px tall -- the window's borders and no rebar -- from this
 * path, and nothing else in the boot measures it. */
static void rebar_tests(HWND parent)
{
    static const int heights[] = { 0, 8, 30, 60 };
    HWND tb, rb;
    TBBUTTON btn = {0};
    REBARBANDINFOW rbi;
    RECT rc;
    DWORD size;
    int i, btn_cy;

    tb = CreateWindowExW(0, TOOLBARCLASSNAMEW, NULL,
                         WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | TBSTYLE_TOOLTIPS |
                             TBSTYLE_WRAPABLE | TBSTYLE_LIST | TBSTYLE_TRANSPARENT | CCS_TOP |
                             CCS_NORESIZE | CCS_NODIVIDER,
                         0, 0, 0, 0, parent, NULL, NULL, NULL);
    printf("tb created %d\n", tb != NULL);
    if (!tb)
        return;
    SendMessageW(tb, TB_BUTTONSTRUCTSIZE, sizeof(TBBUTTON), 0);
    btn.iBitmap = I_IMAGENONE;
    btn.idCommand = 1;
    btn.fsState = TBSTATE_ENABLED | TBSTATE_ELLIPSES;
    btn.fsStyle = BTNS_CHECK | BTNS_NOPREFIX | BTNS_SHOWTEXT;
    btn.iString = (INT_PTR)L"Task";
    printf("tb add %ld\n", (long)SendMessageW(tb, TB_ADDBUTTONSW, 1, (LPARAM)&btn));
    size = (DWORD)SendMessageW(tb, TB_GETBUTTONSIZE, 0, 0);
    btn_cy = HIWORD(size);
    printf("tb buttonsize %u,%u\n", LOWORD(size), HIWORD(size));
    SendMessageW(tb, TB_GETITEMRECT, 0, (LPARAM)&rc);
    printf("tb itemrect %ld,%ld,%ld,%ld\n", rc.left, rc.top, rc.right, rc.bottom);
    SetWindowPos(tb, NULL, 0, 0, 200, btn_cy, SWP_NOZORDER | SWP_NOACTIVATE);

    rb = CreateWindowExW(WS_EX_TOOLWINDOW, REBARCLASSNAMEW, NULL,
                         WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | RBS_VARHEIGHT |
                             RBS_AUTOSIZE | RBS_BANDBORDERS | CCS_NODIVIDER | CCS_NOPARENTALIGN,
                         0, 0, 0, 0, parent, NULL, NULL, NULL);
    printf("rb created %d\n", rb != NULL);
    if (!rb)
    {
        DestroyWindow(tb);
        return;
    }
    /* tbsite.cpp then drops RBS_BANDBORDERS */
    SetWindowLongW(rb, GWL_STYLE, GetWindowLongW(rb, GWL_STYLE) & ~RBS_BANDBORDERS);

    memset(&rbi, 0, sizeof(rbi));
    rbi.cbSize = sizeof(rbi);
    rbi.fMask = RBBIM_ID | RBBIM_CHILD | RBBIM_CHILDSIZE | RBBIM_IDEALSIZE | RBBIM_SIZE |
                RBBIM_STYLE;
    rbi.wID = 1;
    rbi.hwndChild = tb;
    rbi.fStyle = RBBS_VARIABLEHEIGHT | RBBS_NOGRIPPER;
    rbi.cxMinChild = 0;
    rbi.cyMinChild = btn_cy;
    rbi.cyMaxChild = (UINT)-1;
    rbi.cyIntegral = btn_cy;
    rbi.cx = rbi.cxIdeal = 200;
    rbi.cyChild = btn_cy;
    printf("rb insert %ld\n", (long)SendMessageW(rb, RB_INSERTBANDW, (WPARAM)-1, (LPARAM)&rbi));
    printf("rb bands %ld rows %ld barheight %ld\n", (long)SendMessageW(rb, RB_GETBANDCOUNT, 0, 0),
           (long)SendMessageW(rb, RB_GETROWCOUNT, 0, 0),
           (long)SendMessageW(rb, RB_GETBARHEIGHT, 0, 0));
    memset(&rbi, 0, sizeof(rbi));
    rbi.cbSize = sizeof(rbi);
    rbi.fMask = RBBIM_CHILDSIZE | RBBIM_STYLE | RBBIM_SIZE | RBBIM_IDEALSIZE;
    SendMessageW(rb, RB_GETBANDINFOW, 0, (LPARAM)&rbi);
    printf("rb band style=0x%x cx=%u cxIdeal=%u minchild=%u,%u child=%u max=%d integral=%u\n",
           rbi.fStyle, rbi.cx, rbi.cxIdeal, rbi.cxMinChild, rbi.cyMinChild, rbi.cyChild,
           (int)rbi.cyMaxChild, rbi.cyIntegral);
    SendMessageW(rb, RB_GETRECT, 0, (LPARAM)&rc);
    printf("rb bandrect %ld,%ld,%ld,%ld\n", rc.left, rc.top, rc.right, rc.bottom);

    for (i = 0; i < (int)(sizeof(heights) / sizeof(heights[0])); i++)
    {
        RECT r = { 0, 0, 400, heights[i] };
        LRESULT ok = SendMessageW(rb, RB_SIZETORECT, RBSTR_CHANGERECT, (LPARAM)&r);
        GetClientRect(rb, &rc);
        printf("rb sizetorect %d -> %ld rect=%ld,%ld,%ld,%ld client=%ldx%ld barheight %ld rows %ld\n",
               heights[i], (long)ok, r.left, r.top, r.right, r.bottom, rc.right, rc.bottom,
               (long)SendMessageW(rb, RB_GETBARHEIGHT, 0, 0),
               (long)SendMessageW(rb, RB_GETROWCOUNT, 0, 0));
    }
    DestroyWindow(rb); /* takes the toolbar with it: SetParent made it the band's child */
}

static void control_tests(void)
{
    typedef BOOL(WINAPI * ICCEX_t)(const INITCOMMONCONTROLSEX *);
    ICCEX_t iccex = (ICCEX_t)GetProcAddress(dll, "InitCommonControlsEx");
    INITCOMMONCONTROLSEX icc = { sizeof(icc), ICC_TREEVIEW_CLASSES | ICC_LISTVIEW_CLASSES |
                                                ICC_BAR_CLASSES | ICC_COOL_CLASSES };
    WNDCLASSW wc = {0};
    HWND parent;

    printf("iccex %d\n", iccex && iccex(&icc));
    wc.lpfnWndProc = parent_proc;
    wc.hInstance = GetModuleHandleW(NULL);
    wc.lpszClassName = L"modrun_comctl32_parent";
    RegisterClassW(&wc);
    parent = CreateWindowExW(0, wc.lpszClassName, L"", WS_OVERLAPPEDWINDOW, 0, 0, 400, 400,
                             NULL, NULL, wc.hInstance, NULL);
    printf("parent created %d\n", parent != NULL);
    if (!parent)
        return;
    tree_tests(parent);
    fflush(stdout);
    list_tests(parent);
    fflush(stdout);
    rebar_tests(parent);
    fflush(stdout);
    DestroyWindow(parent);
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
    fflush(stdout);
    control_tests();
    printf("probe done\n");
    fflush(stdout);
    return 0;
}
