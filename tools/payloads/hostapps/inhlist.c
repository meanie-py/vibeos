/* inhlist.c: CreateProcess with PROC_THREAD_ATTRIBUTE_HANDLE_LIST, to diff against winref.
 * The parent makes events A and B inheritable and C not, then starts itself as a child under
 * several attribute lists; the child reports which of the three handle values are open in
 * it. The child writes its report to a file the parent then prints: a child's stdout does not
 * reach the hostapps sweep's capture. Usage: inhlist.exe (parent); the child is
 * `inhlist.exe child <report file> <A> <B> <C>`.
 *
 * Known limit: "open" means the handle VALUE is valid in the child, not that it is the
 * parent's object. Under `list {A}` our child reads C as open, inherit_flag=0: its own startup
 * opened a handle in the slot that inheritance left free (hostapps 20261122-163746-nL4X9i).
 * Windows happens to allocate elsewhere. The handle is not inherited: C is not inheritable in
 * the parent, and an inherited handle keeps HANDLE_FLAG_INHERIT. To make this an identity
 * check, name the events and compare names in the child; that needs a fresh winref run.
 */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>

static void child(int argc, wchar_t **argv)
{
    const char *name[3] = { "A", "B", "C" };
    char lines[3][96];
    int n = 0;
    /* Check every handle BEFORE opening the report file: the file's handle takes the lowest
     * free slot, which can be a value under test that was not inherited. */
    for (int i = 0; i < 3 && i + 3 < argc; i++, n++) {
        HANDLE h = (HANDLE)(ULONG_PTR)wcstoul(argv[i + 3], NULL, 16);
        DWORD flags = 0;
        BOOL open = GetHandleInformation(h, &flags);
        /* An inherited event must be the same object: the parent signalled A and B. */
        DWORD wait = open ? WaitForSingleObject(h, 0) : WAIT_FAILED;
        _snprintf(lines[i], sizeof(lines[i]), "  child %s open=%d inherit_flag=%d signalled=%d\n",
                  name[i], open, open && (flags & HANDLE_FLAG_INHERIT) != 0, wait == WAIT_OBJECT_0);
    }
    FILE *out = _wfopen(argv[2], L"w");
    if (!out)
        return;
    for (int i = 0; i < n; i++)
        fputs(lines[i], out);
    fclose(out);
}

static void run(const char *tag, BOOL inherit, HANDLE *list, int n, HANDLE a, HANDLE b, HANDLE c)
{
    wchar_t self[MAX_PATH], report[MAX_PATH], cmd[3 * MAX_PATH];
    STARTUPINFOEXW si = { { sizeof(si) } };
    PROCESS_INFORMATION pi;
    SIZE_T size = 0;
    DWORD flags = 0;
    GetModuleFileNameW(NULL, self, MAX_PATH);
    GetTempPathW(MAX_PATH, report);
    lstrcatW(report, L"inhlist.out");
    DeleteFileW(report);
    _snwprintf(cmd, 3 * MAX_PATH, L"\"%ls\" child \"%ls\" %lx %lx %lx", self, report,
               (unsigned long)(ULONG_PTR)a, (unsigned long)(ULONG_PTR)b, (unsigned long)(ULONG_PTR)c);
    if (list) {
        InitializeProcThreadAttributeList(NULL, 1, 0, &size);
        si.lpAttributeList = HeapAlloc(GetProcessHeap(), 0, size);
        if (!InitializeProcThreadAttributeList(si.lpAttributeList, 1, 0, &size) ||
            !UpdateProcThreadAttribute(si.lpAttributeList, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                                       list, n * sizeof(HANDLE), NULL, NULL)) {
            printf("%s: attribute setup err=%lu\n", tag, GetLastError());
            return;
        }
        flags = EXTENDED_STARTUPINFO_PRESENT;
    }
    fflush(stdout);
    if (!CreateProcessW(NULL, cmd, NULL, NULL, inherit, flags, NULL, NULL, &si.StartupInfo, &pi)) {
        printf("%s: CreateProcess err=%lu\n", tag, GetLastError());
    } else {
        printf("%s:\n", tag);
        fflush(stdout);
        WaitForSingleObject(pi.hProcess, 30000);
        FILE *in = _wfopen(report, L"r");
        char line[128];
        if (!in)
            printf("  child wrote no report\n");
        while (in && fgets(line, sizeof(line), in))
            fputs(line, stdout);
        if (in)
            fclose(in);
        DeleteFileW(report);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
    }
    if (list) {
        DeleteProcThreadAttributeList(si.lpAttributeList);
        HeapFree(GetProcessHeap(), 0, si.lpAttributeList);
    }
}

int wmain(int argc, wchar_t **argv)
{
    if (argc > 1 && !wcscmp(argv[1], L"child")) {
        child(argc, argv);
        return 0;
    }
    SECURITY_ATTRIBUTES sa = { sizeof(sa), NULL, TRUE };
    HANDLE a = CreateEventW(&sa, TRUE, TRUE, NULL);
    HANDLE b = CreateEventW(&sa, TRUE, TRUE, NULL);
    HANDLE c = CreateEventW(NULL, TRUE, TRUE, NULL);
    HANDLE onlyA[1] = { a }, withC[2] = { a, c }, dup[2] = { a, a };
    run("inherit, no list", TRUE, NULL, 0, a, b, c);
    run("inherit, list {A}", TRUE, onlyA, 1, a, b, c);
    run("inherit, list {A,C(not inheritable)}", TRUE, withC, 2, a, b, c);
    run("inherit, list {A,A}", TRUE, dup, 2, a, b, c);
    run("no inherit, list {A}", FALSE, onlyA, 1, a, b, c);
    run("no inherit, no list", FALSE, NULL, 0, a, b, c);
    printf("inhlist done\n");
    return 0;
}
