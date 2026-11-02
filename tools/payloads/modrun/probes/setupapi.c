/* modrun probe: setupapi's logic, shipping Rust against the pre-flip C.
 *
 * The same probe is staged twice. Named setupapi_c.exe it loads setupapi_c.dll (the C oracle
 * c_oracle.cmake builds) from its own directory; under any other name it loads the system
 * setupapi.dll (setupapi-rust). Every export is bound by name, and the probe does NOT import
 * setupapi itself, so the oracle run never touches the shipping DLL. It prints VALUES only --
 * counts, strings, flags, registry data read back, the file-queue notification stream with
 * paths reduced to file names -- never a pointer or a handle, so the two runs diff byte for
 * byte.
 *
 * Covered: the INF parser (fields of every kind, quoting, continuation, string and dirid
 * substitution, line lookup, version info, decorated sections), the string table (with extra
 * data and duplication), AddReg/DelReg through SetupInstallFromInfSection into HKCU, the file
 * queue (copy/rename/delete, scan, commit with a callback), class and device enumeration with
 * registry and devprop properties, and a class driver list built from a crafted INF whose
 * models live ONLY under NT<arch>.10.0 -- the VibeOS fallback in SetupDiBuildDriverInfoList
 * (hand-ported in setupapi-rust) that no boot exercises. Writes only under %TEMP%\mrsa and
 * HKCU\Software\ModrunSetupapi, and removes both. */
#include <windows.h>
#include <setupapi.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

typedef PVOID HSTRING_TABLE_;
typedef struct { GUID fmtid; ULONG pid; } PKEY_;

static HMODULE dll;
static BOOL is_oracle;
static FARPROC bind_fn(const char *name)
{
    FARPROC f = GetProcAddress(dll, name);
    if (!f)
    {
        printf("missing export %s\n", name);
        fflush(stdout);
        ExitProcess(2);
    }
    return f;
}
#define FN(type, name, args) typedef type(WINAPI *name##_t) args; static name##_t p##name;
#define BIND(name) p##name = (name##_t)bind_fn(#name)

FN(HSTRING_TABLE_, pSetupStringTableInitialize, (void))
FN(HSTRING_TABLE_, pSetupStringTableInitializeEx, (DWORD, DWORD))
FN(DWORD, pSetupStringTableAddString, (HSTRING_TABLE_, LPWSTR, DWORD))
FN(DWORD, pSetupStringTableAddStringEx, (HSTRING_TABLE_, LPWSTR, DWORD, LPVOID, DWORD))
FN(DWORD, pSetupStringTableLookUpString, (HSTRING_TABLE_, LPWSTR, DWORD))
FN(BOOL, pSetupStringTableSetExtraData, (HSTRING_TABLE_, DWORD, LPVOID, DWORD))
FN(BOOL, pSetupStringTableGetExtraData, (HSTRING_TABLE_, DWORD, LPVOID, DWORD))
FN(HSTRING_TABLE_, pSetupStringTableDuplicate, (HSTRING_TABLE_))
FN(LPWSTR, pSetupStringTableStringFromId, (HSTRING_TABLE_, DWORD))
FN(VOID, pSetupStringTableDestroy, (HSTRING_TABLE_))
FN(BOOL, pSetupConcatenatePaths, (LPWSTR, LPCWSTR, DWORD, LPDWORD))
FN(PWSTR, pSetupGetFileTitle, (PCWSTR))
FN(DWORD, pSetupGuidFromString, (PCWSTR, LPGUID))
FN(DWORD, pSetupStringFromGuid, (LPGUID, PWSTR, DWORD))
FN(BOOL, pSetupIsGuidNull, (LPGUID))
FN(HINF, SetupOpenInfFileW, (PCWSTR, PCWSTR, DWORD, PUINT))
FN(VOID, SetupCloseInfFile, (HINF))
FN(LONG, SetupGetLineCountW, (HINF, PCWSTR))
FN(BOOL, SetupFindFirstLineW, (HINF, PCWSTR, PCWSTR, PINFCONTEXT))
FN(BOOL, SetupFindNextLine, (PINFCONTEXT, PINFCONTEXT))
FN(BOOL, SetupFindNextMatchLineW, (PINFCONTEXT, PCWSTR, PINFCONTEXT))
FN(BOOL, SetupGetLineByIndexW, (HINF, PCWSTR, DWORD, PINFCONTEXT))
FN(BOOL, SetupGetLineTextW, (PINFCONTEXT, HINF, PCWSTR, PCWSTR, PWSTR, DWORD, PDWORD))
FN(BOOL, SetupGetStringFieldW, (PINFCONTEXT, DWORD, PWSTR, DWORD, PDWORD))
FN(BOOL, SetupGetIntField, (PINFCONTEXT, DWORD, PINT))
FN(BOOL, SetupGetMultiSzFieldW, (PINFCONTEXT, DWORD, PWSTR, DWORD, LPDWORD))
FN(BOOL, SetupGetBinaryField, (PINFCONTEXT, DWORD, PBYTE, DWORD, LPDWORD))
FN(DWORD, SetupGetFieldCount, (PINFCONTEXT))
FN(BOOL, SetupGetInfInformationW, (LPCVOID, DWORD, PSP_INF_INFORMATION, DWORD, PDWORD))
FN(BOOL, SetupQueryInfVersionInformationW, (PSP_INF_INFORMATION, UINT, PCWSTR, PWSTR, DWORD, PDWORD))
FN(BOOL, SetupDiGetActualSectionToInstallW, (HINF, PCWSTR, PWSTR, DWORD, PDWORD, PWSTR *))
FN(BOOL, SetupGetInfFileListW, (PCWSTR, DWORD, PWSTR, DWORD, PDWORD))
FN(BOOL, SetupInstallFromInfSectionW, (HWND, HINF, PCWSTR, UINT, HKEY, PCWSTR, UINT, PSP_FILE_CALLBACK_W, PVOID, HDEVINFO, PSP_DEVINFO_DATA))
FN(HSPFILEQ, SetupOpenFileQueue, (void))
FN(BOOL, SetupCloseFileQueue, (HSPFILEQ))
FN(BOOL, SetupQueueCopyW, (HSPFILEQ, PCWSTR, PCWSTR, PCWSTR, PCWSTR, PCWSTR, PCWSTR, PCWSTR, DWORD))
FN(BOOL, SetupQueueDeleteW, (HSPFILEQ, PCWSTR, PCWSTR))
FN(BOOL, SetupQueueRenameW, (HSPFILEQ, PCWSTR, PCWSTR, PCWSTR, PCWSTR))
FN(BOOL, SetupScanFileQueueW, (HSPFILEQ, DWORD, HWND, PSP_FILE_CALLBACK_W, PVOID, PDWORD))
FN(BOOL, SetupCommitFileQueueW, (HWND, HSPFILEQ, PSP_FILE_CALLBACK_W, PVOID))
FN(BOOL, SetupDiClassGuidsFromNameW, (PCWSTR, LPGUID, DWORD, PDWORD))
FN(BOOL, SetupDiClassNameFromGuidW, (const GUID *, PWSTR, DWORD, PDWORD))
FN(BOOL, SetupDiGetClassDescriptionW, (const GUID *, PWSTR, DWORD, PDWORD))
FN(BOOL, SetupDiBuildClassInfoList, (DWORD, LPGUID, DWORD, PDWORD))
FN(HDEVINFO, SetupDiGetClassDevsW, (const GUID *, PCWSTR, HWND, DWORD))
FN(HDEVINFO, SetupDiCreateDeviceInfoList, (const GUID *, HWND))
FN(BOOL, SetupDiDestroyDeviceInfoList, (HDEVINFO))
FN(BOOL, SetupDiEnumDeviceInfo, (HDEVINFO, DWORD, PSP_DEVINFO_DATA))
FN(BOOL, SetupDiGetDeviceInstanceIdW, (HDEVINFO, PSP_DEVINFO_DATA, PWSTR, DWORD, PDWORD))
FN(BOOL, SetupDiGetDeviceRegistryPropertyW, (HDEVINFO, PSP_DEVINFO_DATA, DWORD, PDWORD, PBYTE, DWORD, PDWORD))
FN(BOOL, SetupDiGetDeviceInstallParamsW, (HDEVINFO, PSP_DEVINFO_DATA, PSP_DEVINSTALL_PARAMS_W))
FN(BOOL, SetupDiSetDeviceInstallParamsW, (HDEVINFO, PSP_DEVINFO_DATA, PSP_DEVINSTALL_PARAMS_W))
FN(BOOL, SetupDiBuildDriverInfoList, (HDEVINFO, PSP_DEVINFO_DATA, DWORD))
FN(BOOL, SetupDiEnumDriverInfoW, (HDEVINFO, PSP_DEVINFO_DATA, DWORD, DWORD, PSP_DRVINFO_DATA_W))
FN(BOOL, SetupDiGetDriverInfoDetailW, (HDEVINFO, PSP_DEVINFO_DATA, PSP_DRVINFO_DATA_W, PSP_DRVINFO_DETAIL_DATA_W, DWORD, PDWORD))
FN(BOOL, SetupDiDestroyDriverInfoList, (HDEVINFO, PSP_DEVINFO_DATA, DWORD))
FN(BOOL, SetupDiGetDevicePropertyKeys, (HDEVINFO, PSP_DEVINFO_DATA, PKEY_ *, DWORD, PDWORD, DWORD))
FN(BOOL, SetupDiGetDevicePropertyW, (HDEVINFO, PSP_DEVINFO_DATA, const PKEY_ *, ULONG *, PBYTE, DWORD, PDWORD, DWORD))

static WCHAR tmp[MAX_PATH], inf_path[MAX_PATH], drv_path[MAX_PATH];

/* Wide string -> stdout, replacing the run's temp root so paths compare. */
static void pw(const WCHAR *s)
{
    size_t n = wcslen(tmp);
    if (!s)
    {
        printf("(null)");
        return;
    }
    while (*s)
    {
        if (n && !_wcsnicmp(s, tmp, n))
        {
            printf("<TMP>");
            s += n;
            continue;
        }
        putchar(*s < 0x80 ? (char)*s : '?');
        s++;
    }
}
static void pmsz(const WCHAR *s)
{
    printf("[");
    for (; s && *s; s += wcslen(s) + 1)
    {
        pw(s);
        printf("|");
    }
    printf("]");
}
static void pguid(const GUID *g)
{
    printf("{%08lX-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}", g->Data1, g->Data2, g->Data3,
           g->Data4[0], g->Data4[1], g->Data4[2], g->Data4[3], g->Data4[4], g->Data4[5],
           g->Data4[6], g->Data4[7]);
}
#define FAIL(what) printf("%s fail %lu\n", what, GetLastError())

static void write_file(const WCHAR *path, const char *text)
{
    HANDLE h = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
    DWORD done;
    WriteFile(h, text, (DWORD)strlen(text), &done, NULL);
    CloseHandle(h);
}
static void cat_file(const WCHAR *path)
{
    char buf[256];
    DWORD got = 0;
    HANDLE h = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        printf("<missing>");
        return;
    }
    ReadFile(h, buf, sizeof(buf) - 1, &got, NULL);
    buf[got] = 0;
    CloseHandle(h);
    printf("\"%s\"", buf);
}
static void join(WCHAR *out, const WCHAR *a, const WCHAR *b)
{
    lstrcpyW(out, a);
    lstrcatW(out, L"\\");
    lstrcatW(out, b);
}

static const char INF[] =
    "[Version]\r\n"
    "Signature=\"$Windows NT$\"\r\n"
    "Class=ModrunTest\r\n"
    "ClassGUID={6E3C8A10-7A3B-4C5E-9F3A-2B1C0D9E8F70}\r\n"
    "Provider=%Prov%\r\n"
    "DriverVer=10/31/2026,1.2.3.4\r\n"
    "\r\n"
    "[Strings]\r\n"
    "Prov=\"VibeOS Modrun\"\r\n"
    "Quoted=\"a \"\"quoted\"\" word\"\r\n"
    "Sub=%10%\\system32\r\n"
    "\r\n"
    "[Parse]\r\n"
    "key1 = value1\r\n"
    "key2 = \"spaced value\" , second,  third  ; a comment\r\n"
    "key3 = 0x10, -5, 077, 12abc, 4294967295\r\n"
    "multi = a,b,,c\r\n"
    "bin = 01,02,ff,7F,0\r\n"
    "cont = first, \\\r\n"
    "       second\r\n"
    "nokey\r\n"
    "\"quoted key\" = qv\r\n"
    "key1 = dup\r\n"
    "subst = %Sub%, %Prov%, %Quoted%, %%, %Missing%\r\n"
    "dirid = %11%\\drivers, %12%, %17%\\x.inf, %24%\r\n"
    "; full-line comment\r\n"
    "  KEY1  =  case  \r\n"
    "\r\n"
    "[Deco]\r\nbase=1\r\n"
    "[Deco.NT]\r\nnt=1\r\n"
    "[Deco.NTx86]\r\nx86=1\r\n"
    "[Deco.NTamd64]\r\namd64=1\r\n"
    "[Deco.NTarm64]\r\narm64=1\r\n"
    "[Plain]\r\nonly=1\r\n"
    "[Empty]\r\n"
    "\r\n"
    "[AddRegSec]\r\nAddReg=Reg1\r\n"
    "[Reg1]\r\n"
    "HKCU,Software\\ModrunSetupapi,Str,0,\"hello\"\r\n"
    "HKCU,Software\\ModrunSetupapi,Dw,0x10001,0x12345678\r\n"
    "HKCU,Software\\ModrunSetupapi,Multi,0x10000,\"a\",\"b\",\"c\"\r\n"
    "HKCU,Software\\ModrunSetupapi,Multi,0x10008,\"d\"\r\n"
    "HKCU,Software\\ModrunSetupapi,Exp,0x20000,\"%%SystemRoot%%\\x\"\r\n"
    "HKCU,Software\\ModrunSetupapi,Bin,1,de,ad,be,ef\r\n"
    "HKCU,Software\\ModrunSetupapi,Keep,0x2,\"new\"\r\n"
    "HKCU,Software\\ModrunSetupapi,Prov,0,%Prov%\r\n"
    "HKCU,Software\\ModrunSetupapi\\Sub,,,\"default\"\r\n"
    "[MultiBase]\r\nAddReg=RegBase\r\n"
    "[RegBase]\r\nHKCU,Software\\ModrunSetupapi,Steps,0x10000,\"a\",\"b\",\"c\"\r\n"
    "[MultiApp]\r\nAddReg=RegApp\r\n"
    "[RegApp]\r\nHKCU,Software\\ModrunSetupapi,Steps,0x10008,\"d\"\r\n"
    "[MultiApp2]\r\nAddReg=RegApp2\r\n"
    "[RegApp2]\r\nHKCU,Software\\ModrunSetupapi,Steps,0x10008,\"b\",\"e\"\r\n"
    "[MultiDel]\r\nDelReg=RegDel\r\n"
    "[RegDel]\r\nHKCU,Software\\ModrunSetupapi,Steps,0x18002,\"B\"\r\n"
    "[DelRegSec]\r\nDelReg=Del1\r\n"
    "[Del1]\r\n"
    "HKCU,Software\\ModrunSetupapi,Str\r\n"
    "HKCU,Software\\ModrunSetupapi\\Sub\r\n";

/* Models only under NT<arch>.10.0, with the undecorated section present and empty: what
 * current vendor INFs ship, and what SetupDiBuildDriverInfoList's VibeOS fallback is for. */
static const char DRV[] =
    "[Version]\r\n"
    "Signature=\"$Windows NT$\"\r\n"
    "Class=ModrunTest\r\n"
    "ClassGUID={6E3C8A10-7A3B-4C5E-9F3A-2B1C0D9E8F70}\r\n"
    "Provider=%Prov%\r\n"
    "DriverVer=10/31/2026,1.2.3.4\r\n"
    "[Manufacturer]\r\n"
    "%Mfg%=Models,NTx86.10.0,NTamd64.10.0,NTarm64.10.0\r\n"
    "Plain=PlainModels\r\n"
    "[Models]\r\n"
    "; empty on purpose: this INF does not support the reported version\r\n"
    "[Models.NTx86.10.0]\r\n%Dev%=Inst,ROOT\\MODRUN_SA,*MODRUNCOMPAT\r\n"
    "[Models.NTamd64.10.0]\r\n%Dev%=Inst,ROOT\\MODRUN_SA,*MODRUNCOMPAT\r\n"
    "[Models.NTarm64.10.0]\r\n%Dev%=Inst,ROOT\\MODRUN_SA,*MODRUNCOMPAT\r\n"
    "[PlainModels]\r\n"
    "\"Plain Device\"=PlainInst,ROOT\\PLAIN_SA\r\n"
    "\"Second Plain\"=PlainInst2,ROOT\\PLAIN_SB,COMPAT_B1,COMPAT_B2\r\n"
    "[Inst]\r\n[PlainInst]\r\n[PlainInst2]\r\n"
    "[Strings]\r\n"
    "Prov=\"VibeOS Modrun\"\r\n"
    "Mfg=\"Modrun Mfg\"\r\n"
    "Dev=\"Modrun Device\"\r\n";

static void strings(void)
{
    HSTRING_TABLE_ t = ppSetupStringTableInitialize(), d;
    HSTRING_TABLE_ x = ppSetupStringTableInitializeEx(sizeof(DWORD), 0);
    WCHAR alpha[] = L"Alpha", beta[] = L"beta", up[] = L"ALPHA", none[] = L"none";
    DWORD a, b, c, data = 0x1234, got = 0;

    a = ppSetupStringTableAddString(t, alpha, 0);
    b = ppSetupStringTableAddString(t, beta, 0);
    c = ppSetupStringTableAddString(t, up, 0);
    printf("st add a=%d b=%d case-insens-same=%d\n", a != (DWORD)-1, b != a, c == a);
    printf("st lookup ALPHA=%d none=%ld\n", ppSetupStringTableLookUpString(t, up, 0) == a,
           (long)ppSetupStringTableLookUpString(t, none, 0));
    printf("st lookup case-sens ALPHA=%ld\n",
           (long)(ppSetupStringTableLookUpString(t, up, 1 /* ST_CASE_SENSITIVE_COMPARE */) == a));
    c = ppSetupStringTableAddString(t, up, 1);
    printf("st add case-sens new=%d\n", c != a);
    printf("st fromid b=");
    pw(ppSetupStringTableStringFromId(t, b));
    printf("\n");
    d = ppSetupStringTableDuplicate(t);
    printf("st dup lookup beta=%d\n", ppSetupStringTableLookUpString(d, beta, 0) == b);
    ppSetupStringTableDestroy(d);

    a = ppSetupStringTableAddStringEx(x, alpha, 0, &data, sizeof(data));
    printf("st ex get=%d", ppSetupStringTableGetExtraData(x, a, &got, sizeof(got)));
    printf(" val=%lx\n", got);
    data = 0xbeef;
    printf("st ex set=%d", ppSetupStringTableSetExtraData(x, a, &data, sizeof(data)));
    got = 0;
    ppSetupStringTableGetExtraData(x, a, &got, sizeof(got));
    printf(" val=%lx toolarge=%d\n", got,
           ppSetupStringTableSetExtraData(x, a, &data, sizeof(data) + 8));
    ppSetupStringTableDestroy(x);
    ppSetupStringTableDestroy(t);
}

static void misc(void)
{
    WCHAR buf[64], small[8];
    DWORD need = 0;
    GUID g = {0}, g2;
    BOOL r;

    lstrcpyW(buf, L"C:\\a");
    r = ppSetupConcatenatePaths(buf, L"b\\c", 64, &need);
    printf("concat %d need=%lu ", r, need);
    pw(buf);
    lstrcpyW(buf, L"C:\\a\\");
    r = ppSetupConcatenatePaths(buf, L"\\b", 64, &need);
    printf(" | %d need=%lu ", r, need);
    pw(buf);
    lstrcpyW(small, L"C:\\a");
    r = ppSetupConcatenatePaths(small, L"bcdefgh", 8, &need);
    printf(" | small %d need=%lu\n", r, need);
    printf("title ");
    pw(ppSetupGetFileTitle(L"C:\\x\\y.inf"));
    printf(" ");
    pw(ppSetupGetFileTitle(L"noslash"));
    printf("\n");
    printf("guidnull %d", ppSetupIsGuidNull(&g));
    printf(" parse %lu", ppSetupGuidFromString(L"{6E3C8A10-7A3B-4C5E-9F3A-2B1C0D9E8F70}", &g));
    printf(" bad %lu", ppSetupGuidFromString(L"6E3C8A10", &g2));
    printf(" fmt %lu ", ppSetupStringFromGuid(&g, buf, 64));
    pw(buf);
    printf(" short %lu notnull %d\n", ppSetupStringFromGuid(&g, small, 8), ppSetupIsGuidNull(&g));
}

static void field_dump(INFCONTEXT *ctx)
{
    DWORD n = pSetupGetFieldCount(ctx), i, need;
    WCHAR s[512];
    INT v;
    printf(" fields=%lu", n);
    for (i = 0; i <= n; i++)
    {
        need = 0;
        if (pSetupGetStringFieldW(ctx, i, s, 512, &need))
        {
            printf(" [%lu]=", i);
            pw(s);
            printf("(%lu)", need);
        }
        else
            printf(" [%lu]!%lu", i, GetLastError());
        if (i && pSetupGetIntField(ctx, i, &v))
            printf("#%d", v);
    }
}

static void parser(void)
{
    UINT err = 0;
    HINF hinf = pSetupOpenInfFileW(inf_path, NULL, INF_STYLE_WIN4, &err);
    INFCONTEXT ctx, next;
    WCHAR s[512], *ext = NULL;
    BYTE bin[16];
    DWORD need, i;
    BOOL r;

    if (hinf == INVALID_HANDLE_VALUE)
    {
        printf("open fail %lu line %u\n", GetLastError(), err);
        return;
    }
    printf("lines Parse=%ld Strings=%ld Missing=%ld Empty=%ld\n", pSetupGetLineCountW(hinf, L"Parse"),
           pSetupGetLineCountW(hinf, L"Strings"), pSetupGetLineCountW(hinf, L"Missing"),
           pSetupGetLineCountW(hinf, L"Empty"));
    for (r = pSetupFindFirstLineW(hinf, L"Parse", NULL, &ctx), i = 0; r;
         r = pSetupFindNextLine(&ctx, &ctx), i++)
    {
        printf("parse %lu", i);
        field_dump(&ctx);
        need = 0;
        if (pSetupGetLineTextW(&ctx, NULL, NULL, NULL, s, 512, &need))
        {
            printf(" text=");
            pw(s);
            printf("(%lu)", need);
        }
        printf("\n");
    }
    if (pSetupFindFirstLineW(hinf, L"Parse", L"KEY1", &ctx))
    {
        printf("find key1:");
        field_dump(&ctx);
        while (pSetupFindNextMatchLineW(&ctx, L"key1", &next))
        {
            printf(" | next:");
            field_dump(&next);
            ctx = next;
        }
        printf("\n");
    }
    printf("find missing key %d err=%lu\n", pSetupFindFirstLineW(hinf, L"Parse", L"nope", &ctx),
           GetLastError());
    if (pSetupGetLineByIndexW(hinf, L"Parse", 3, &ctx) &&
        pSetupGetMultiSzFieldW(&ctx, 1, s, 512, &need))
    {
        printf("multisz need=%lu ", need);
        pmsz(s);
        printf("\n");
    }
    if (pSetupGetLineByIndexW(hinf, L"Parse", 4, &ctx))
    {
        need = 0;
        r = pSetupGetBinaryField(&ctx, 1, bin, sizeof(bin), &need);
        printf("binary %d need=%lu", r, need);
        for (i = 0; r && i < need; i++)
            printf(" %02x", bin[i]);
        r = pSetupGetBinaryField(&ctx, 1, bin, 2, &need);
        printf(" small %d err=%lu need=%lu\n", r, GetLastError(), need);
    }
    printf("byindex 99 %d err=%lu\n", pSetupGetLineByIndexW(hinf, L"Parse", 99, &ctx), GetLastError());
    if (pSetupFindFirstLineW(hinf, L"Parse", L"key2", &ctx))
    {
        r = pSetupGetStringFieldW(&ctx, 1, s, 3, &need);
        printf("short buffer %d err=%lu need=%lu\n", r, GetLastError(), need);
    }

    for (i = 0; i < 4; i++)
    {
        static const WCHAR *names[] = {L"Deco", L"Plain", L"Empty", L"Missing"};
        need = 0;
        ext = NULL;
        r = pSetupDiGetActualSectionToInstallW(hinf, names[i], s, 512, &need, &ext);
        printf("actual ");
        pw(names[i]);
        printf(" %d ", r);
        if (r)
        {
            pw(s);
            printf(" need=%lu ext=", need);
            pw(ext);
        }
        printf("\n");
    }
    pSetupCloseInfFile(hinf);

    {
        BYTE info[4096];
        PSP_INF_INFORMATION pi = (PSP_INF_INFORMATION)info;
        static const WCHAR *keys[] = {L"Provider", L"DriverVer", L"Class", L"Signature", L"Nope"};
        need = 0;
        r = pSetupGetInfInformationW(inf_path, INFINFO_INF_NAME_IS_ABSOLUTE, pi, sizeof(info), &need);
        printf("infinfo %d style=%lu count=%lu\n", r, r ? pi->InfStyle : 0, r ? pi->InfCount : 0);
        for (i = 0; r && i < 5; i++)
        {
            need = 0;
            printf("  ver ");
            pw(keys[i]);
            if (pSetupQueryInfVersionInformationW(pi, 0, keys[i], s, 512, &need))
            {
                printf(" = ");
                pw(s);
                printf(" (%lu)\n", need);
            }
            else
                printf(" ! %lu\n", GetLastError());
        }
    }
    need = 0;
    r = pSetupGetInfFileListW(tmp, INF_STYLE_WIN4, s, 512, &need);
    printf("inflist %d need=%lu ", r, need);
    if (r)
        pmsz(s);
    printf("\n");
}

static void dump_value(HKEY k, const WCHAR *name)
{
    BYTE data[512];
    DWORD type = 0, size = sizeof(data), i;
    LONG rc = RegQueryValueExW(k, name, NULL, &type, data, &size);
    printf("  reg ");
    pw(name && *name ? name : L"(default)");
    if (rc)
    {
        printf(" ! %ld\n", rc);
        return;
    }
    printf(" type=%lu size=%lu ", type, size);
    if (type == REG_SZ || type == REG_EXPAND_SZ)
        pw((WCHAR *)data);
    else if (type == REG_MULTI_SZ)
    {
        pmsz((WCHAR *)data);
        printf(" raw");
        for (i = 0; i < size; i += 2)
            printf(" %04x", *(WCHAR *)(data + i));
    }
    else
        for (i = 0; i < size; i++)
            printf("%02x", data[i]);
    printf("\n");
}

static void install(void)
{
    static const WCHAR *names[] = {L"Str", L"Dw", L"Multi", L"Exp", L"Bin", L"Keep", L"Prov"};
    HINF hinf = pSetupOpenInfFileW(inf_path, NULL, INF_STYLE_WIN4, NULL);
    HKEY k;
    DWORD i;
    BOOL r;

    if (hinf == INVALID_HANDLE_VALUE)
    {
        FAIL("install open");
        return;
    }

    RegDeleteTreeW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi");
    RegCreateKeyW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi", &k);
    RegSetValueExW(k, L"Keep", 0, REG_SZ, (const BYTE *)L"old", 8);
    RegCloseKey(k);

    {
        INFCONTEXT ctx;
        WCHAR ms[64];
        DWORD need = 0, j;
        for (j = 2; j <= 3; j++)
            if (pSetupGetLineByIndexW(hinf, L"Reg1", j, &ctx))
            {
                BOOL ok = pSetupGetMultiSzFieldW(&ctx, 5, ms, 64, &need);
                printf("reg1 line %lu multisz %d need=%lu raw", j, ok, need);
                for (i = 0; ok && i < need; i++)
                    printf(" %04x", ms[i]);
                printf("\n");
            }
    }
    r = pSetupInstallFromInfSectionW(NULL, hinf, L"AddRegSec", SPINST_REGISTRY, NULL, NULL, 0,
                                     NULL, NULL, NULL, NULL);
    printf("addreg %d err=%lu\n", r, r ? 0 : GetLastError());
    if (!RegOpenKeyW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi", &k))
    {
        for (i = 0; i < 7; i++)
            dump_value(k, names[i]);
        RegCloseKey(k);
    }
    if (!RegOpenKeyW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi\\Sub", &k))
    {
        dump_value(k, L"");
        RegCloseKey(k);
    }
    {
        static const WCHAR *steps[] = {L"MultiBase", L"MultiApp", L"MultiApp2", L"MultiDel"};
        for (i = 0; i < 4; i++)
        {
            /* The C's delete_multi_sz_value writes its result past its own heap block; on
             * i386 that killed the oracle run (0xc0000005). setupapi-rust fixes it. */
            if (is_oracle && i == 3)
            {
                printf("step 3 skipped: the C's DELSTRING overruns its heap block\n");
                continue;
            }
            r = pSetupInstallFromInfSectionW(NULL, hinf, steps[i], SPINST_REGISTRY, NULL, NULL, 0,
                                             NULL, NULL, NULL, NULL);
            printf("step %lu %d\n", i, r);
            if (!RegOpenKeyW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi", &k))
            {
                dump_value(k, L"Steps");
                RegCloseKey(k);
            }
        }
    }
    r = pSetupInstallFromInfSectionW(NULL, hinf, L"DelRegSec", SPINST_REGISTRY, NULL, NULL, 0,
                                     NULL, NULL, NULL, NULL);
    printf("delreg %d\n", r);
    if (!RegOpenKeyW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi", &k))
    {
        dump_value(k, L"Str");
        dump_value(k, L"Dw");
        RegCloseKey(k);
    }
    printf("  sub after delreg %ld\n",
           RegOpenKeyW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi\\Sub", &k));
    r = pSetupInstallFromInfSectionW(NULL, hinf, L"Missing", SPINST_REGISTRY, NULL, NULL, 0,
                                     NULL, NULL, NULL, NULL);
    printf("missing section %d err=%lu\n", r, GetLastError());
    RegDeleteTreeW(HKEY_CURRENT_USER, L"Software\\ModrunSetupapi");
    pSetupCloseInfFile(hinf);
}

static UINT CALLBACK queue_cb(PVOID ctx, UINT note, UINT_PTR p1, UINT_PTR p2)
{
    printf("  note %#x", note);
    switch (note)
    {
    case SPFILENOTIFY_STARTCOPY:
    case SPFILENOTIFY_STARTRENAME:
    case SPFILENOTIFY_STARTDELETE:
    case SPFILENOTIFY_ENDCOPY:
    case SPFILENOTIFY_ENDRENAME:
    case SPFILENOTIFY_ENDDELETE:
    case SPFILENOTIFY_QUEUESCAN:
    {
        if (note == SPFILENOTIFY_QUEUESCAN)
        {
            printf(" ");
            pw((const WCHAR *)p1);
            printf("\n");
            return NO_ERROR;
        }
        FILEPATHS_W *fp = (FILEPATHS_W *)p1;
        printf(" src=");
        pw(fp->Source);
        printf(" dst=");
        pw(fp->Target);
        printf(" err=%u\n", fp->Win32Error);
        return (note == SPFILENOTIFY_STARTCOPY || note == SPFILENOTIFY_STARTRENAME ||
                note == SPFILENOTIFY_STARTDELETE)
                   ? FILEOP_DOIT
                   : NO_ERROR;
    }
    case SPFILENOTIFY_COPYERROR:
    {
        FILEPATHS_W *fp = (FILEPATHS_W *)p1;
        printf(" copyerror src=");
        pw(fp->Source);
        printf(" dst=");
        pw(fp->Target);
        printf(" err=%u\n", fp->Win32Error);
        return FILEOP_SKIP;
    }
    case SPFILENOTIFY_STARTQUEUE:
    case SPFILENOTIFY_STARTSUBQUEUE:
        printf(" p2=%lu\n", (unsigned long)p2);
        return TRUE;
    default:
        printf("\n");
        return NO_ERROR;
    }
}

static void queue(void)
{
    WCHAR src[MAX_PATH], dst[MAX_PATH], p[MAX_PATH];
    static const WCHAR *after[] = {L"a2.txt", L"b.txt", L"old.txt", L"ren.txt", L"renamed.txt"};
    HSPFILEQ q = pSetupOpenFileQueue();
    DWORD result = 0, i;
    BOOL r;

    join(src, tmp, L"src");
    join(dst, tmp, L"dst");
    CreateDirectoryW(src, NULL);
    CreateDirectoryW(dst, NULL);
    join(p, src, L"a.txt");
    write_file(p, "alpha file");
    join(p, src, L"b.txt");
    write_file(p, "bravo");
    join(p, dst, L"old.txt");
    write_file(p, "to delete");
    join(p, dst, L"ren.txt");
    write_file(p, "to rename");

    printf("q copy %d", pSetupQueueCopyW(q, src, NULL, L"a.txt", NULL, NULL, dst, L"a2.txt", 0));
    printf(" copy %d", pSetupQueueCopyW(q, src, NULL, L"b.txt", NULL, NULL, dst, NULL, 0));
    printf(" del %d", pSetupQueueDeleteW(q, dst, L"old.txt"));
    printf(" ren %d\n", pSetupQueueRenameW(q, dst, L"ren.txt", dst, L"renamed.txt"));
    r = pSetupScanFileQueueW(q, SPQ_SCAN_USE_CALLBACK, NULL, queue_cb, NULL, &result);
    printf("scan %d result=%lu\n", r, result);
    r = pSetupCommitFileQueueW(NULL, q, queue_cb, NULL);
    printf("commit %d err=%lu\n", r, r ? 0 : GetLastError());
    pSetupCloseFileQueue(q);
    for (i = 0; i < 5; i++)
    {
        join(p, dst, after[i]);
        printf("  after ");
        pw(after[i]);
        printf(" ");
        cat_file(p);
        printf("\n");
        DeleteFileW(p);
    }
    join(p, src, L"a.txt");
    DeleteFileW(p);
    join(p, src, L"b.txt");
    DeleteFileW(p);
    RemoveDirectoryW(src);
    RemoveDirectoryW(dst);
}

static void classes(void)
{
    static const WCHAR *names[] = {L"DiskDrive", L"Net", L"Display", L"System", L"NoSuchClass"};
    GUID g[8];
    WCHAR s[256];
    DWORD n, i, need;

    for (i = 0; i < 5; i++)
    {
        n = 0;
        printf("class ");
        pw(names[i]);
        if (!pSetupDiClassGuidsFromNameW(names[i], g, 8, &n) || !n)
        {
            printf(" n=%lu err=%lu\n", n, GetLastError());
            continue;
        }
        printf(" n=%lu ", n);
        pguid(&g[0]);
        need = 0;
        if (pSetupDiClassNameFromGuidW(&g[0], s, 256, &need))
        {
            printf(" name=");
            pw(s);
            printf("(%lu)", need);
        }
        if (pSetupDiGetClassDescriptionW(&g[0], s, 256, &need))
        {
            printf(" desc=");
            pw(s);
        }
        printf("\n");
    }
    n = 0;
    pSetupDiBuildClassInfoList(0, NULL, 0, &n);
    printf("classinfolist need=%lu err=%lu\n", n, GetLastError());
}

static void devices(void)
{
    static const DWORD props[] = {SPDRP_HARDWAREID, SPDRP_COMPATIBLEIDS, SPDRP_CLASS,
                                  SPDRP_CLASSGUID, SPDRP_SERVICE, SPDRP_DRIVER, SPDRP_MFG,
                                  SPDRP_CONFIGFLAGS, SPDRP_DEVICEDESC};
    static const GUID devfmt = {0xa45c254e, 0xdf1c, 0x4efd, {0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0}};
    static const GUID drvfmt = {0xa8b865dd, 0x2e3d, 0x4094, {0xad, 0x97, 0xe5, 0x93, 0xa7, 0x0c, 0x75, 0xd6}};
    HDEVINFO set = pSetupDiGetClassDevsW(NULL, NULL, NULL, DIGCF_ALLCLASSES | DIGCF_PRESENT);
    SP_DEVINFO_DATA d = {sizeof(d)};
    BYTE buf[2048];
    WCHAR id[512];
    DWORD i, j, type, need;

    if (set == INVALID_HANDLE_VALUE)
    {
        FAIL("classdevs");
        return;
    }
    for (i = 0; pSetupDiEnumDeviceInfo(set, i, &d); i++)
    {
        if (i >= 40)
            continue;
        pSetupDiGetDeviceInstanceIdW(set, &d, id, 512, NULL);
        printf("dev %lu ", i);
        pw(id);
        printf("\n");
        for (j = 0; j < sizeof(props) / sizeof(props[0]); j++)
        {
            need = 0;
            if (!pSetupDiGetDeviceRegistryPropertyW(set, &d, props[j], &type, buf, sizeof(buf), &need))
            {
                printf("  p%lu ! %lu\n", props[j], GetLastError());
                continue;
            }
            printf("  p%lu t=%lu n=%lu ", props[j], type, need);
            if (type == REG_MULTI_SZ)
                pmsz((WCHAR *)buf);
            else if (type == REG_DWORD)
                printf("%lx", *(DWORD *)buf);
            else
                pw((WCHAR *)buf);
            printf("\n");
        }
        if (i < 8)
        {
            PKEY_ keys[64];
            ULONG ptype;
            DWORD nk = 0;
            BOOL r = pSetupDiGetDevicePropertyKeys(set, &d, keys, 64, &nk, 0);
            printf("  keys %d\n", r);
            for (j = 0; r && j < nk; j++)
            {
                /* Capabilities (pid 17) is written by the PnP manager while the boot settles:
                 * a C run repeated after the Rust run gains it exactly as the Rust run does, so
                 * its presence measures the clock, not the code. */
                if (keys[j].pid == 17 && !memcmp(&keys[j].fmtid, &devfmt, sizeof(GUID)))
                    continue;
                printf("   key ");
                pguid(&keys[j].fmtid);
                printf(",%lu", keys[j].pid);
                need = 0;
                ptype = 0;
                if (pSetupDiGetDevicePropertyW(set, &d, &keys[j], &ptype, buf, sizeof(buf), &need, 0))
                {
                    printf(" t=%#lx n=%lu ", ptype, need);
                    if (ptype == 0x12 /* STRING */)
                        pw((WCHAR *)buf);
                    else if (ptype == 0x2012 /* STRING_LIST */)
                        pmsz((WCHAR *)buf);
                    else if (ptype == 0x7 /* UINT32 */)
                        printf("%lx", *(DWORD *)buf);
                    else if (ptype == 0xd /* GUID */)
                        pguid((GUID *)buf);
                }
                else
                    printf(" ! %lu", GetLastError());
                printf("\n");
            }
            /* Two keys asked for directly, whether or not the enumeration listed them. */
            for (j = 0; j < 2; j++)
            {
                PKEY_ k;
                k.fmtid = j ? drvfmt : devfmt;
                k.pid = j ? 4 /* DriverDesc */ : 3 /* HardwareIds */;
                need = 0;
                ptype = 0;
                printf("   direct %lu %d t=%#lx n=%lu err=%lu\n", j,
                       pSetupDiGetDevicePropertyW(set, &d, &k, &ptype, buf, sizeof(buf), &need, 0),
                       ptype, need, GetLastError());
            }
        }
    }
    printf("devices %lu end err=%lu\n", i, GetLastError());
    pSetupDiDestroyDeviceInfoList(set);
}

static void drivers(void)
{
    HDEVINFO set = pSetupDiCreateDeviceInfoList(NULL, NULL);
    SP_DEVINSTALL_PARAMS_W ip = {sizeof(ip)};
    SP_DRVINFO_DATA_W di;
    BYTE buf[4096];
    PSP_DRVINFO_DETAIL_DATA_W dd = (PSP_DRVINFO_DETAIL_DATA_W)buf;
    SYSTEMTIME st;
    DWORD i, need;
    BOOL r;

    if (!pSetupDiGetDeviceInstallParamsW(set, NULL, &ip))
    {
        FAIL("getparams");
        return;
    }
    lstrcpyW(ip.DriverPath, drv_path);
    ip.Flags |= DI_ENUMSINGLEINF;
    printf("setparams %d\n", pSetupDiSetDeviceInstallParamsW(set, NULL, &ip));
    r = pSetupDiBuildDriverInfoList(set, NULL, SPDIT_CLASSDRIVER);
    printf("build %d err=%lu\n", r, r ? 0 : GetLastError());
    for (i = 0; ; i++)
    {
        di.cbSize = sizeof(di);
        if (!pSetupDiEnumDriverInfoW(set, NULL, SPDIT_CLASSDRIVER, i, &di))
        {
            printf("drivers %lu end err=%lu\n", i, GetLastError());
            break;
        }
        FileTimeToSystemTime(&di.DriverDate, &st);
        printf("drv %lu type=%lu desc=", i, di.DriverType);
        pw(di.Description);
        printf(" mfg=");
        pw(di.MfgName);
        printf(" prov=");
        pw(di.ProviderName);
        printf(" date=%04u-%02u-%02u ver=%llx\n", st.wYear, st.wMonth, st.wDay,
               (unsigned long long)di.DriverVersion);
        dd->cbSize = sizeof(*dd);
        need = 0;
        if (pSetupDiGetDriverInfoDetailW(set, NULL, &di, dd, sizeof(buf), &need))
        {
            printf("  detail need=%lu section=", need);
            pw(dd->SectionName);
            printf(" inf=");
            pw(dd->InfFileName);
            printf(" ids=");
            pmsz(dd->HardwareID);
            printf(" compat=%lu/%lu\n", dd->CompatIDsOffset, dd->CompatIDsLength);
        }
        else
            FAIL("  detail");
    }
    pSetupDiGetDeviceInstallParamsW(set, NULL, &ip);
    printf("flags %#lx flagsex %#lx\n", ip.Flags & (DI_DIDCLASS | DI_DIDCOMPAT),
           ip.FlagsEx & (DI_FLAGSEX_DIDINFOLIST | DI_FLAGSEX_DIDCOMPATINFO));
    printf("destroy %d\n", pSetupDiDestroyDriverInfoList(set, NULL, SPDIT_CLASSDRIVER));
    printf("compat without device %d err=%lu\n",
           pSetupDiBuildDriverInfoList(set, NULL, SPDIT_COMPATDRIVER), GetLastError());
    printf("bad type %d err=%lu\n", pSetupDiBuildDriverInfoList(set, NULL, 7), GetLastError());
    pSetupDiDestroyDeviceInfoList(set);
}

int main(void)
{
    WCHAR self[MAX_PATH], *slash;
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);

    /* setupapi_c.exe -> its sibling setupapi_c.dll (the C oracle); anything else -> the
     * system setupapi (what ships). */
    if (len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe"))
    {
        slash = wcsrchr(self, L'\\');
        lstrcpyW(slash + 1, L"setupapi_c.dll");
        dll = LoadLibraryW(self);
        is_oracle = TRUE;
    }
    else
        dll = LoadLibraryW(L"setupapi.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    GetModuleFileNameW(NULL, self, MAX_PATH);
    BIND(pSetupStringTableInitialize); BIND(pSetupStringTableInitializeEx);
    BIND(pSetupStringTableAddString); BIND(pSetupStringTableAddStringEx);
    BIND(pSetupStringTableLookUpString); BIND(pSetupStringTableSetExtraData);
    BIND(pSetupStringTableGetExtraData); BIND(pSetupStringTableDuplicate);
    BIND(pSetupStringTableStringFromId); BIND(pSetupStringTableDestroy);
    BIND(pSetupConcatenatePaths); BIND(pSetupGetFileTitle); BIND(pSetupGuidFromString);
    BIND(pSetupStringFromGuid); BIND(pSetupIsGuidNull);
    BIND(SetupOpenInfFileW); BIND(SetupCloseInfFile); BIND(SetupGetLineCountW);
    BIND(SetupFindFirstLineW); BIND(SetupFindNextLine); BIND(SetupFindNextMatchLineW);
    BIND(SetupGetLineByIndexW); BIND(SetupGetLineTextW); BIND(SetupGetStringFieldW);
    BIND(SetupGetIntField); BIND(SetupGetMultiSzFieldW); BIND(SetupGetBinaryField);
    BIND(SetupGetFieldCount); BIND(SetupGetInfInformationW);
    BIND(SetupQueryInfVersionInformationW); BIND(SetupDiGetActualSectionToInstallW);
    BIND(SetupGetInfFileListW); BIND(SetupInstallFromInfSectionW);
    BIND(SetupOpenFileQueue); BIND(SetupCloseFileQueue); BIND(SetupQueueCopyW);
    BIND(SetupQueueDeleteW); BIND(SetupQueueRenameW); BIND(SetupScanFileQueueW);
    BIND(SetupCommitFileQueueW);
    BIND(SetupDiClassGuidsFromNameW); BIND(SetupDiClassNameFromGuidW);
    BIND(SetupDiGetClassDescriptionW); BIND(SetupDiBuildClassInfoList);
    BIND(SetupDiGetClassDevsW); BIND(SetupDiCreateDeviceInfoList);
    BIND(SetupDiDestroyDeviceInfoList); BIND(SetupDiEnumDeviceInfo);
    BIND(SetupDiGetDeviceInstanceIdW); BIND(SetupDiGetDeviceRegistryPropertyW);
    BIND(SetupDiGetDeviceInstallParamsW); BIND(SetupDiSetDeviceInstallParamsW);
    BIND(SetupDiBuildDriverInfoList); BIND(SetupDiEnumDriverInfoW);
    BIND(SetupDiGetDriverInfoDetailW); BIND(SetupDiDestroyDriverInfoList);
    BIND(SetupDiGetDevicePropertyKeys); BIND(SetupDiGetDevicePropertyW);

    /* The first candidate that takes a write: C: is writable on the amd64 and i386 media but
     * not on arm64's, and the live CD's %TEMP% may not exist yet. Only the label is printed. */
    {
        static const char *labels[3] = {"c root", "temp", "self"};
        const char *picked = "none";
        WCHAR cands[3][MAX_PATH];
        int c;
        lstrcpyW(cands[0], L"C:\\");
        GetTempPathW(MAX_PATH, cands[1]);
        lstrcpyW(cands[2], self);
        *(wcsrchr(cands[2], L'\\') + 1) = 0;
        for (c = 0; c < 3 && !strcmp(picked, "none"); c++)
        {
            lstrcpyW(tmp, cands[c]);
            lstrcatW(tmp, L"mrsa");
            if (CreateDirectoryW(tmp, NULL) || GetLastError() == ERROR_ALREADY_EXISTS)
                picked = labels[c];
        }
        printf("work dir %s\n", picked);
    }
    join(inf_path, tmp, L"probe.inf");
    join(drv_path, tmp, L"drv.inf");
    write_file(inf_path, INF);
    write_file(drv_path, DRV);

    strings();
    fflush(stdout);
    misc();
    fflush(stdout);
    parser();
    fflush(stdout);
    install();
    fflush(stdout);
    queue();
    fflush(stdout);
    classes();
    fflush(stdout);
    drivers();
    fflush(stdout);
    devices();

    DeleteFileW(inf_path);
    DeleteFileW(drv_path);
    RemoveDirectoryW(tmp);
    printf("probe done\n");
    fflush(stdout);
    return 0;
}
