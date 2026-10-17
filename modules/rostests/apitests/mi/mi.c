/*
 * PROJECT:     ReactOS API tests
 * LICENSE:     MIT (https://spdx.org/licenses/MIT)
 * PURPOSE:     mi.dll, the Management Infrastructure client: what the CIM .NET layer
 *              (and through it Docker) calls. Every expectation was measured against
 *              Windows 11's mi.dll first: this binary runs there unchanged.
 */

#include <apitest.h>
#include <windows.h>
#include <mi.h>

static MI_Application app;
static MI_Session session;

static const MI_Instance *next_result(MI_Operation *op, MI_Boolean *more, MI_Result *result)
{
    const MI_Instance *inst = NULL;
    const MI_Char *msg = NULL;
    const MI_Instance *details = NULL;
    MI_Result r = MI_Operation_GetInstance(op, &inst, more, result, &msg, &details);
    ok(r == MI_RESULT_OK, "GetInstance returned %d\n", r);
    return inst;
}

/* Run a synchronous operation to its end: the instance count and the final result. */
static int drain(MI_Operation *op, MI_Result *final, const MI_Char *want_class)
{
    MI_Boolean more = MI_TRUE;
    MI_Result result = MI_RESULT_OK;
    int n = 0;
    while (more)
    {
        const MI_Instance *inst = next_result(op, &more, &result);
        if (inst)
        {
            const MI_Char *name = NULL;
            n++;
            ok(MI_Instance_GetClassName(inst, &name) == MI_RESULT_OK, "GetClassName failed\n");
            if (want_class && name)
                ok(!_wcsicmp(name, want_class), "class %ls, want %ls\n", name, want_class);
        }
        if (more)
            ok(result == MI_RESULT_OK, "a non-final result carries %d\n", result);
    }
    *final = result;
    MI_Operation_Close(op);
    return n;
}

static void test_tables(void)
{
    ok(mi_clientFT_V1 != NULL, "no mi_clientFT_V1\n");
    if (!mi_clientFT_V1) return;
    ok(app.ft == mi_clientFT_V1->applicationFT, "the application's table is not the client table's\n");
    ok(mi_clientFT_V1->sessionFT != NULL, "no sessionFT\n");
    ok(mi_clientFT_V1->operationFT != NULL, "no operationFT\n");
}

static void test_query_os(void)
{
    MI_Operation op = MI_OPERATION_NULL;
    MI_Boolean more = MI_TRUE;
    MI_Result result;
    const MI_Instance *inst;
    MI_Value v;
    MI_Type t;
    MI_Uint32 flags, count, index;
    const MI_Char *ns = NULL;

    MI_Session_QueryInstances(&session, 0, NULL, L"root/cimv2", L"WQL",
                              L"SELECT * FROM Win32_OperatingSystem", NULL, &op);
    inst = next_result(&op, &more, &result);
    ok(inst != NULL, "no Win32_OperatingSystem instance\n");
    if (inst)
    {
        ok(MI_Instance_GetElementCount(inst, &count) == MI_RESULT_OK && count > 10, "only %u elements\n", count);
        ok(MI_Instance_GetElement(inst, L"Caption", &v, &t, &flags, &index) == MI_RESULT_OK, "no Caption\n");
        ok(t == MI_STRING, "Caption type %d\n", t);
        ok(!(flags & MI_FLAG_NULL) && v.string && v.string[0], "Caption is empty\n");
        ok(MI_Instance_GetElement(inst, L"caption", &v, &t, NULL, NULL) == MI_RESULT_OK,
           "element names do not compare case-insensitively\n");
        ok(MI_Instance_GetElement(inst, L"NoSuchProperty", &v, &t, NULL, NULL) == MI_RESULT_NO_SUCH_PROPERTY,
           "a missing element was found\n");
        ok(MI_Instance_GetElement(inst, L"LastBootUpTime", &v, &t, &flags, NULL) == MI_RESULT_OK, "no LastBootUpTime\n");
        ok(t == MI_DATETIME, "LastBootUpTime type %d\n", t);
        if (t == MI_DATETIME && !(flags & MI_FLAG_NULL))
            ok(v.datetime.isTimestamp && v.datetime.u.timestamp.year >= 1980, "boot year %u\n", v.datetime.u.timestamp.year);
        ok(MI_Instance_GetElement(inst, L"NumberOfProcesses", &v, &t, NULL, NULL) == MI_RESULT_OK, "no NumberOfProcesses\n");
        ok(t == MI_UINT32, "NumberOfProcesses type %d\n", t);
        ok(MI_Instance_GetElement(inst, L"TotalVisibleMemorySize", &v, &t, NULL, NULL) == MI_RESULT_OK, "no TotalVisibleMemorySize\n");
        ok(t == MI_UINT64, "TotalVisibleMemorySize type %d\n", t);
        if (t == MI_UINT64) ok(v.uint64 > 0, "no memory\n");
        ok(MI_Instance_GetNameSpace(inst, &ns) == MI_RESULT_OK && ns && !_wcsicmp(ns, L"root/cimv2"), "namespace %ls\n", ns);
        ok(inst->classDecl != NULL && inst->classDecl->name && !_wcsicmp(inst->classDecl->name, L"Win32_OperatingSystem"),
           "classDecl does not name the class\n");
    }
    while (more)
        next_result(&op, &more, &result);
    ok(result == MI_RESULT_OK, "final result %d\n", result);
    ok(MI_Operation_Close(&op) == MI_RESULT_OK, "Close failed\n");
}

static void test_enumerate(void)
{
    MI_Operation op = MI_OPERATION_NULL;
    MI_Result result;
    int n;

    MI_Session_EnumerateInstances(&session, 0, NULL, L"root/cimv2", L"Win32_Process", MI_FALSE, NULL, &op);
    n = drain(&op, &result, L"Win32_Process");
    ok(n > 2, "%d processes\n", n);
    ok(result == MI_RESULT_OK, "final result %d\n", result);

    /* Docker asks this one; whatever the count, it must complete cleanly. */
    MI_Session_QueryInstances(&session, 0, NULL, L"root/cimv2", L"WQL",
                              L"SELECT * FROM Win32_OptionalFeature", NULL, &op);
    n = drain(&op, &result, L"Win32_OptionalFeature");
    ok(result == MI_RESULT_OK, "Win32_OptionalFeature final result %d\n", result);
    trace("%d optional features\n", n);
}

static void test_errors(void)
{
    MI_Operation op = MI_OPERATION_NULL;
    MI_Result result;

    MI_Session_QueryInstances(&session, 0, NULL, L"root/nosuchnamespace", L"WQL",
                              L"SELECT * FROM Win32_Process", NULL, &op);
    ok(drain(&op, &result, NULL) == 0, "results from a missing namespace\n");
    ok(result == MI_RESULT_INVALID_NAMESPACE, "missing namespace: %d\n", result);

    MI_Session_EnumerateInstances(&session, 0, NULL, L"root/cimv2", L"Win32_NoSuchClass", MI_FALSE, NULL, &op);
    ok(drain(&op, &result, NULL) == 0, "results from a missing class\n");
    ok(result == MI_RESULT_INVALID_CLASS, "missing class: %d\n", result);

    MI_Session_QueryInstances(&session, 0, NULL, L"root/cimv2", L"WQL", L"SELEKT nonsense", NULL, &op);
    drain(&op, &result, NULL);
    ok(result == MI_RESULT_INVALID_QUERY, "bad WQL: %d\n", result);
}

static void test_get_instance(void)
{
    MI_Instance *keys = NULL;
    MI_Operation op = MI_OPERATION_NULL;
    MI_Boolean more;
    MI_Result result;
    const MI_Instance *inst;
    MI_Value v;
    MI_Type t;
    WCHAR pid[16];

    wsprintfW(pid, L"%lu", GetCurrentProcessId());
    ok(MI_Application_NewInstance(&app, L"Win32_Process", NULL, &keys) == MI_RESULT_OK, "NewInstance failed\n");
    if (!keys) return;
    v.string = pid;
    ok(MI_Instance_AddElement(keys, L"Handle", &v, MI_STRING, MI_FLAG_KEY) == MI_RESULT_OK, "AddElement failed\n");
    MI_Session_GetInstance(&session, 0, NULL, L"root/cimv2", keys, NULL, &op);
    inst = next_result(&op, &more, &result);
    ok(inst != NULL, "own process not found (%d)\n", result);
    if (inst)
    {
        ok(MI_Instance_GetElement(inst, L"ProcessId", &v, &t, NULL, NULL) == MI_RESULT_OK, "no ProcessId\n");
        ok(t == MI_UINT32 && v.uint32 == GetCurrentProcessId(), "ProcessId %u type %d\n", v.uint32, t);
    }
    while (more)
        next_result(&op, &more, &result);
    ok(result == MI_RESULT_OK, "final result %d\n", result);
    MI_Operation_Close(&op);
    MI_Instance_Delete(keys);
}

static struct
{
    HANDLE done;
    LONG instances;
    MI_Result result;
    MI_Boolean manual;
    LONG acks;
} async_state;

static void MI_CALL on_instance(MI_Operation *op, void *context, const MI_Instance *inst, MI_Boolean more,
                                MI_Result result, const MI_Char *error, const MI_Instance *details,
                                MI_Result (MI_CALL *ack)(MI_Operation *))
{
    ok(context == &async_state, "context %p\n", context);
    if (inst)
        InterlockedIncrement(&async_state.instances);
    if (async_state.manual)
    {
        ok(ack != NULL || !more, "no acknowledgement callback\n");
        if (ack)
        {
            InterlockedIncrement(&async_state.acks);
            ack(op);
        }
    }
    if (!more)
    {
        async_state.result = result;
        SetEvent(async_state.done);
    }
}

static void test_async(MI_Boolean manual)
{
    MI_OperationCallbacks cb = MI_OPERATIONCALLBACKS_NULL;
    MI_Operation op = MI_OPERATION_NULL;

    async_state.done = CreateEventW(NULL, TRUE, FALSE, NULL);
    async_state.instances = 0;
    async_state.acks = 0;
    async_state.result = MI_RESULT_FAILED;
    async_state.manual = manual;
    cb.callbackContext = &async_state;
    cb.instanceResult = on_instance;
    MI_Session_QueryInstances(&session, manual ? MI_OPERATIONFLAGS_MANUAL_ACK_RESULTS : 0, NULL, L"root/cimv2",
                              L"WQL", L"SELECT * FROM Win32_Process", &cb, &op);
    ok(WaitForSingleObject(async_state.done, 60000) == WAIT_OBJECT_0, "no final callback\n");
    ok(async_state.result == MI_RESULT_OK, "async final result %d\n", async_state.result);
    ok(async_state.instances > 2, "async: %ld processes\n", async_state.instances);
    if (manual)
        ok(async_state.acks >= async_state.instances, "%ld acks for %ld instances\n", async_state.acks, async_state.instances);
    MI_Operation_Close(&op);
    CloseHandle(async_state.done);
}

static void test_instance_api(void)
{
    MI_Instance *inst = NULL, *copy = NULL;
    MI_Value v, got;
    MI_Type t;
    MI_Uint32 flags, count;
    const MI_Char *name;
    MI_Uint32 nums[3] = { 1, 2, 3 };
    MI_Char *strs[2] = { L"one", L"two" };

    ok(MI_Application_NewInstance(&app, L"Test_Class", NULL, &inst) == MI_RESULT_OK && inst, "NewInstance failed\n");
    if (!inst) return;
    v.string = L"hello";
    ok(MI_Instance_AddElement(inst, L"Name", &v, MI_STRING, MI_FLAG_KEY) == MI_RESULT_OK, "add string\n");
    ok(MI_Instance_AddElement(inst, L"name", &v, MI_STRING, 0) == MI_RESULT_ALREADY_EXISTS, "added a duplicate\n");
    v.uint32 = 42;
    ok(MI_Instance_AddElement(inst, L"Count", &v, MI_UINT32, 0) == MI_RESULT_OK, "add uint32\n");
    v.uint32a.data = nums;
    v.uint32a.size = 3;
    ok(MI_Instance_AddElement(inst, L"Nums", &v, MI_UINT32A, 0) == MI_RESULT_OK, "add uint32 array\n");
    v.stringa.data = strs;
    v.stringa.size = 2;
    ok(MI_Instance_AddElement(inst, L"Strs", &v, MI_STRINGA, 0) == MI_RESULT_OK, "add string array\n");
    ok(MI_Instance_AddElement(inst, L"Empty", NULL, MI_SINT64, MI_FLAG_NULL) == MI_RESULT_OK, "add null\n");
    nums[0] = 99; /* the instance owns a copy */

    ok(MI_Instance_GetElementCount(inst, &count) == MI_RESULT_OK && count == 5, "count %u\n", count);
    ok(MI_Instance_GetElement(inst, L"NUMS", &got, &t, &flags, NULL) == MI_RESULT_OK, "get array\n");
    ok(t == MI_UINT32A && got.uint32a.size == 3 && got.uint32a.data[0] == 1 && got.uint32a.data[2] == 3, "array copy\n");
    ok(MI_Instance_GetElementAt(inst, 3, &name, &got, &t, &flags) == MI_RESULT_OK, "get at\n");
    ok(!wcscmp(name, L"Strs") && t == MI_STRINGA && got.stringa.size == 2 && !wcscmp(got.stringa.data[1], L"two"),
       "string array copy\n");
    ok(MI_Instance_GetElement(inst, L"Empty", &got, &t, &flags, NULL) == MI_RESULT_OK && (flags & MI_FLAG_NULL),
       "null element flags %#x\n", flags);
    v.string = L"x";
    ok(MI_Instance_SetElement(inst, L"Count", &v, MI_STRING, 0) == MI_RESULT_TYPE_MISMATCH, "set with another type\n");
    v.uint32 = 7;
    ok(MI_Instance_SetElement(inst, L"Count", &v, MI_UINT32, 0) == MI_RESULT_OK, "set\n");
    ok(MI_Instance_GetElement(inst, L"Count", &got, &t, NULL, NULL) == MI_RESULT_OK && got.uint32 == 7, "set value\n");
    ok(MI_Instance_ClearElement(inst, L"Count") == MI_RESULT_OK, "clear\n");
    ok(MI_Instance_GetElement(inst, L"Count", &got, &t, &flags, NULL) == MI_RESULT_OK && (flags & MI_FLAG_NULL)
       && t == MI_UINT32, "cleared element flags %#x type %d\n", flags, t);

    ok(MI_Instance_Clone(inst, &copy) == MI_RESULT_OK && copy, "clone\n");
    MI_Instance_Delete(inst);
    if (copy)
    {
        ok(MI_Instance_GetClassName(copy, &name) == MI_RESULT_OK && !wcscmp(name, L"Test_Class"), "clone class\n");
        ok(MI_Instance_GetElement(copy, L"Name", &got, &t, &flags, NULL) == MI_RESULT_OK && !wcscmp(got.string, L"hello")
           && (flags & MI_FLAG_KEY), "clone kept the key: flags %#x\n", flags);
        MI_Instance_Delete(copy);
    }
}

static void test_options(void)
{
    MI_OperationOptions oo = MI_OPERATIONOPTIONS_NULL, oo2 = MI_OPERATIONOPTIONS_NULL;
    MI_Uint32 n = 0, count = 0;
    const MI_Char *s = NULL;

    ok(MI_Application_NewOperationOptions(&app, MI_FALSE, &oo) == MI_RESULT_OK, "NewOperationOptions\n");
    ok(MI_OperationOptions_SetNumber(&oo, L"Number", 5, 0) == MI_RESULT_OK, "SetNumber\n");
    ok(oo.ft->SetString(&oo, L"Text", L"abc", 0) == MI_RESULT_OK, "SetString\n");
    ok(MI_OperationOptions_GetNumber(&oo, L"Number", &n, NULL, NULL) == MI_RESULT_OK && n == 5, "GetNumber %u\n", n);
    ok(oo.ft->GetString(&oo, L"Text", &s, NULL, NULL) == MI_RESULT_OK && s && !wcscmp(s, L"abc"), "GetString\n");
    ok(MI_OperationOptions_Clone(&oo, &oo2) == MI_RESULT_OK, "Clone\n");
    ok(oo2.ft->GetOptionCount(&oo2, &count) == MI_RESULT_OK && count >= 2, "cloned count %u\n", count);
    MI_OperationOptions_Delete(&oo2);
    MI_OperationOptions_Delete(&oo);
}

START_TEST(mi)
{
    MI_Instance *err = NULL;
    MI_Result r;

    r = MI_Application_Initialize(0, L"mi_apitest", &err, &app);
    ok(r == MI_RESULT_OK, "MI_Application_Initialize: %d\n", r);
    if (r != MI_RESULT_OK) return;
    test_tables();
    r = MI_Application_NewSession(&app, NULL, NULL, NULL, NULL, &err, &session);
    ok(r == MI_RESULT_OK, "NewSession: %d\n", r);
    if (r == MI_RESULT_OK)
    {
        test_query_os();
        test_enumerate();
        test_errors();
        test_get_instance();
        test_async(MI_FALSE);
        test_async(MI_TRUE);
        MI_Session_Close(&session, NULL, NULL);
    }
    test_instance_api();
    test_options();
    ok(MI_Application_Close(&app) == MI_RESULT_OK, "Close\n");
}
