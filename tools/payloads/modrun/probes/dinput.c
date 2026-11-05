/* modrun probe: dinput (dll/directx/wine/dinput -> dinput-rust) against its pre-flip C.
 *
 * The system keyboard and mouse, driven through what needs no input: device enumeration,
 * capabilities, object enumeration and lookup, the data-format parser (a hand-built format,
 * plus malformed ones), properties, cooperative levels, acquire/unacquire and an idle state
 * read. Mouse values are read only in relative mode, where an idle device reads zero, so
 * neither run depends on where the cursor happens to be. Smoke never loads dinput. */
#include "dx_probe.h"
#define DIRECTINPUT_VERSION 0x0700
#include <dinput.h>

typedef HRESULT(WINAPI *CREATEEX)(HINSTANCE, DWORD, REFIID, void **, IUnknown *);

static int devs;
static BOOL CALLBACK dev_cb(const DIDEVICEINSTANCEA *d, void *ctx)
{
    (void)ctx;
    devs++;
    printf("  dev type=%lx name=%s product=%s kbd=%d mouse=%d\n", d->dwDevType, d->tszInstanceName,
           d->tszProductName, IsEqualGUID(&d->guidInstance, &GUID_SysKeyboard),
           IsEqualGUID(&d->guidInstance, &GUID_SysMouse));
    return DIENUM_CONTINUE;
}
static int objs;
static BOOL CALLBACK obj_cb(const DIDEVICEOBJECTINSTANCEA *o, void *ctx)
{
    (void)ctx;
    if (objs++ < 12 || o->dwOfs == DIK_SPACE)
        printf("  obj ofs=%lu type=%lx flags=%lx name=%s\n", o->dwOfs, o->dwType, o->dwFlags,
               o->tszName);
    return DIENUM_CONTINUE;
}

static DIOBJECTDATAFORMAT keys[256];
static DIOBJECTDATAFORMAT mouse_objs[7];

static void caps(IDirectInputDevice7A *d, const char *tag)
{
    DIDEVCAPS c;
    memset(&c, 0, sizeof(c));
    c.dwSize = sizeof(c);
    HR(tag, IDirectInputDevice7_GetCapabilities(d, &c));
    printf("  caps flags=%lx type=%lx axes=%lu buttons=%lu povs=%lu\n", c.dwFlags, c.dwDevType,
           c.dwAxes, c.dwButtons, c.dwPOVs);
    objs = 0;
    HR("  enum objects", IDirectInputDevice7_EnumObjects(d, obj_cb, NULL, DIDFT_ALL));
    printf("  objects %d\n", objs);
}

static void keyboard(IDirectInput7A *di, HWND hwnd)
{
    IDirectInputDevice7A *k;
    DIDATAFORMAT f = {sizeof(DIDATAFORMAT), sizeof(DIOBJECTDATAFORMAT), DIDF_RELAXIS, 256, 256, keys};
    DIDEVICEOBJECTINSTANCEA oi;
    DIPROPDWORD p;
    BYTE state[256];
    DIDEVICEOBJECTDATA data[8];
    DWORD i, n;
    HRESULT hr;

    hr = IDirectInput7_CreateDeviceEx(di, &GUID_SysKeyboard, &IID_IDirectInputDevice7A, (void **)&k, NULL);
    HR("keyboard", hr);
    if (FAILED(hr))
        return;
    caps(k, "kbd caps");
    for (i = 0; i < 256; i++)
    {
        keys[i].pguid = &GUID_Key;
        keys[i].dwOfs = i;
        keys[i].dwType = DIDFT_OPTIONAL | DIDFT_BUTTON | DIDFT_MAKEINSTANCE(i);
        keys[i].dwFlags = 0;
    }
    f.dwObjSize = 12;
    HR("kbd format bad objsize", IDirectInputDevice7_SetDataFormat(k, &f));
    f.dwObjSize = sizeof(DIOBJECTDATAFORMAT);
    HR("kbd format", IDirectInputDevice7_SetDataFormat(k, &f));
    memset(&oi, 0, sizeof(oi));
    oi.dwSize = sizeof(oi);
    HR("kbd object A", IDirectInputDevice7_GetObjectInfo(k, &oi, DIK_A, DIPH_BYOFFSET));
    printf("  A ofs=%lu type=%lx name=%s\n", oi.dwOfs, oi.dwType, oi.tszName);
    HR("kbd object bad", IDirectInputDevice7_GetObjectInfo(k, &oi, 300, DIPH_BYOFFSET));
    HR("kbd coop bad", IDirectInputDevice7_SetCooperativeLevel(k, hwnd, DISCL_EXCLUSIVE | DISCL_NONEXCLUSIVE));
    HR("kbd coop", IDirectInputDevice7_SetCooperativeLevel(k, hwnd, DISCL_NONEXCLUSIVE | DISCL_BACKGROUND));

    memset(&p, 0, sizeof(p));
    p.diph.dwSize = sizeof(p);
    p.diph.dwHeaderSize = sizeof(DIPROPHEADER);
    p.diph.dwHow = DIPH_DEVICE;
    HR("kbd get bufsize", IDirectInputDevice7_GetProperty(k, DIPROP_BUFFERSIZE, &p.diph));
    printf("  bufsize %lu\n", p.dwData);
    p.dwData = 16;
    HR("kbd set bufsize", IDirectInputDevice7_SetProperty(k, DIPROP_BUFFERSIZE, &p.diph));
    p.dwData = 0;
    IDirectInputDevice7_GetProperty(k, DIPROP_BUFFERSIZE, &p.diph);
    printf("  bufsize back %lu\n", p.dwData);
    p.diph.dwHeaderSize = 4;
    HR("kbd bad header", IDirectInputDevice7_GetProperty(k, DIPROP_BUFFERSIZE, &p.diph));

    HR("kbd state unacquired", IDirectInputDevice7_GetDeviceState(k, sizeof(state), state));
    hr = IDirectInputDevice7_Acquire(k);
    HR("kbd acquire", hr);
    HR("kbd acquire twice", IDirectInputDevice7_Acquire(k));
    if (SUCCEEDED(hr))
    {
        memset(state, 0xcc, sizeof(state));
        HR("kbd state", IDirectInputDevice7_GetDeviceState(k, sizeof(state), state));
        for (n = 0, i = 0; i < 256; i++)
            n += state[i] != 0;
        printf("  keys down %lu\n", n);
        HR("kbd state short", IDirectInputDevice7_GetDeviceState(k, 16, state));
        n = 8;
        HR("kbd data", IDirectInputDevice7_GetDeviceData(k, sizeof(DIDEVICEOBJECTDATA), data, &n, 0));
        printf("  events %lu\n", n);
        HR("kbd unacquire", IDirectInputDevice7_Unacquire(k));
    }
    HR("kbd unacquire twice", IDirectInputDevice7_Unacquire(k));
    printf("kbd release %lu\n", IDirectInputDevice7_Release(k));
}

static void mouse(IDirectInput7A *di, HWND hwnd)
{
    IDirectInputDevice7A *m;
    DIDATAFORMAT f = {sizeof(DIDATAFORMAT), sizeof(DIOBJECTDATAFORMAT), DIDF_RELAXIS, 16, 7, mouse_objs};
    DIPROPDWORD p;
    LONG state[4];
    DWORD i;
    HRESULT hr;
    const GUID *axes[3] = {&GUID_XAxis, &GUID_YAxis, &GUID_ZAxis};

    hr = IDirectInput7_CreateDeviceEx(di, &GUID_SysMouse, &IID_IDirectInputDevice7A, (void **)&m, NULL);
    HR("mouse", hr);
    if (FAILED(hr))
        return;
    caps(m, "mouse caps");
    for (i = 0; i < 3; i++)
    {
        mouse_objs[i].pguid = axes[i];
        mouse_objs[i].dwOfs = i * 4;
        mouse_objs[i].dwType = DIDFT_AXIS | DIDFT_ANYINSTANCE | (i == 2 ? DIDFT_OPTIONAL : 0);
    }
    for (i = 3; i < 7; i++)
    {
        mouse_objs[i].pguid = &GUID_Button;
        mouse_objs[i].dwOfs = 12 + (i - 3);
        mouse_objs[i].dwType = DIDFT_BUTTON | DIDFT_ANYINSTANCE | (i > 4 ? DIDFT_OPTIONAL : 0);
    }
    f.dwDataSize = 15;
    HR("mouse format unaligned", IDirectInputDevice7_SetDataFormat(m, &f));
    f.dwDataSize = 16;
    HR("mouse format", IDirectInputDevice7_SetDataFormat(m, &f));
    memset(&p, 0, sizeof(p));
    p.diph.dwSize = sizeof(p);
    p.diph.dwHeaderSize = sizeof(DIPROPHEADER);
    p.diph.dwHow = DIPH_DEVICE;
    HR("mouse axismode", IDirectInputDevice7_GetProperty(m, DIPROP_AXISMODE, &p.diph));
    printf("  axismode %lu\n", p.dwData);
    p.diph.dwHow = DIPH_BYOFFSET;
    p.diph.dwObj = 0;
    HR("mouse granularity x", IDirectInputDevice7_GetProperty(m, DIPROP_GRANULARITY, &p.diph));
    printf("  granularity %lu\n", p.dwData);
    p.diph.dwObj = 8;
    HR("mouse granularity z", IDirectInputDevice7_GetProperty(m, DIPROP_GRANULARITY, &p.diph));
    printf("  granularity z %lu\n", p.dwData);
    HR("mouse coop", IDirectInputDevice7_SetCooperativeLevel(m, hwnd, DISCL_NONEXCLUSIVE | DISCL_BACKGROUND));
    hr = IDirectInputDevice7_Acquire(m);
    HR("mouse acquire", hr);
    if (SUCCEEDED(hr))
    {
        memset(state, 0x77, sizeof(state));
        HR("mouse state", IDirectInputDevice7_GetDeviceState(m, sizeof(state), state));
        printf("  buttons %08lx\n", (unsigned long)state[3]);
        IDirectInputDevice7_Unacquire(m);
    }
    printf("mouse release %lu\n", IDirectInputDevice7_Release(m));
}

int main(void)
{
    CREATEEX create;
    IDirectInput7A *di = NULL;
    IDirectInputA *di1 = NULL;
    HINSTANCE hinst = GetModuleHandleA(NULL);
    IDirectInputDevice7A *dev;
    HWND hwnd;
    HRESULT hr;

    dx_load(L"dinput");
    create = (CREATEEX)dx_proc("DirectInputCreateEx");
    HR("create future version", create(hinst, 0x0900, &IID_IDirectInput7A, (void **)&di, NULL));
    HR("create null hinst", create(NULL, 0x0700, &IID_IDirectInput7A, (void **)&di, NULL));
    if (di)
        IDirectInput7_Release(di);
    di = NULL;
    HR("create v5", create(hinst, 0x0500, &IID_IDirectInputA, (void **)&di1, NULL));
    if (di1)
        printf("  v5 release %lu\n", IDirectInput_Release(di1));
    hr = create(hinst, 0x0700, &IID_IDirectInput7A, (void **)&di, NULL);
    HR("create v7", hr);
    if (FAILED(hr))
        return DONE();
    hwnd = dx_window();
    HR("enum all", IDirectInput7_EnumDevices(di, 0, dev_cb, NULL, DIEDFL_ALLDEVICES));
    printf("devices %d\n", devs);
    devs = 0;
    HR("enum keyboards", IDirectInput7_EnumDevices(di, DIDEVTYPE_KEYBOARD, dev_cb, NULL, DIEDFL_ATTACHEDONLY));
    printf("keyboards %d\n", devs);
    HR("status kbd", IDirectInput7_GetDeviceStatus(di, &GUID_SysKeyboard));
    HR("status bogus", IDirectInput7_GetDeviceStatus(di, &GUID_Key));
    HR("create bogus device", IDirectInput7_CreateDeviceEx(di, &GUID_Key, &IID_IDirectInputDevice7A,
                                                           (void **)&dev, NULL));
    keyboard(di, hwnd);
    mouse(di, hwnd);
    printf("di release %lu\n", IDirectInput7_Release(di));
    DestroyWindow(hwnd);
    return DONE();
}
