/* INERT since 2026-09-24: not in CMake. This class is Rust now, on the crate's atlwin
 * substrate (sdk/lib/rust/shell32-rust/src/atl_stub_windows.rs). Kept on disk as the
 * reference the port was checked against. */

/* rust_changenotify_broker_stub.cpp — the ONE ATL window class changenotify.cpp still needs.
 *
 * `changenotify.cpp` is otherwise fully translated (shell32-rust/src/changenotify.rs) and is out
 * of the CMake source list. What could not come with it is `CChangeNotifyBroker`, a
 * `CWindowImpl<>` whose per-instance THUNK and message map ATL generates from C++ templates —
 * there is no Rust spelling for either. Same category as `rust_folder_options_stub.cpp`.
 *
 * Unlike that one the message map here is NOT empty: MESSAGE_HANDLER routes
 * WM_BROKER_NOTIFICATION to OnBrokerNotification, so the thunk genuinely dispatches and a
 * hand-rolled WNDPROC would have to reproduce ATL's message-map return/bHandled protocol.
 * That is why the whole class stays rather than just its registration.
 *
 * `CreateNotificationBroker` comes with it because it is inseparable from the class: it `new`s
 * the broker, calls SubclassWindow, and `delete`s it on the failure path. The object then
 * outlives the call — ATL's thunk must stay alive as long as the window does, and OnFinalMessage
 * does the `delete this` after the SERVER destroys the window. Nothing on the Rust side owns it,
 * which is precisely why none of this can be a Rust reimplementation.
 *
 * Kept `extern "C"` WITHOUT a `WINAPI`/`_RustBridge` dressing on purpose. It was a file-local
 * `static` (so __cdecl), and this is not a wrapper invented around some other real function —
 * it is the same function, which simply did not move. Declaring it stdcall or renaming it
 * would make it look like a bridge and, on i386, would silently change the ABI.
 *
 * BrokerNotification calls SHChangeNotification_Lock/_Unlock, which ARE now Rust. Those resolve
 * because shlobj.h declares them inside its `extern "C" {` block (lines 35-2528), so C++ never
 * mangles them and the `#[no_mangle]` definitions match. No lockstep was needed for this flip.
 */

#include "precomp.h"

WINE_DEFAULT_DEBUG_CHANNEL(shcn);

// This class brokers all notifications that don't have the SHCNRF_NewDelivery flag
class CChangeNotifyBroker :
    public CWindowImpl<CChangeNotifyBroker, CWindow, CWorkerTraits>
{
public:
    CChangeNotifyBroker(HWND hwndClient, UINT uMsg) :
        m_hwndClient(hwndClient), m_uMsg(uMsg)
    {
    }

    // Message handlers
    LRESULT OnBrokerNotification(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL& bHandled)
    {
        return BrokerNotification((HANDLE)wParam, (DWORD)lParam);
    }

    void OnFinalMessage(HWND)
    {
        // The server will destroy this window.
        // After the window gets destroyed we can delete this broker here.
        delete this;
    }

    DECLARE_WND_CLASS_EX(L"WorkerW", 0, 0)

    BEGIN_MSG_MAP(CChangeNotifyBroker)
        MESSAGE_HANDLER(WM_BROKER_NOTIFICATION, OnBrokerNotification)
    END_MSG_MAP()

private:
    HWND m_hwndClient;
    UINT m_uMsg;

    BOOL BrokerNotification(HANDLE hTicket, DWORD dwOwnerPID)
    {
        // lock the ticket
        PIDLIST_ABSOLUTE *ppidl = NULL;
        LONG lEvent;
        HANDLE hLock = SHChangeNotification_Lock(hTicket, dwOwnerPID, &ppidl, &lEvent);
        if (hLock == NULL)
        {
            ERR("hLock is NULL\n");
            return FALSE;
        }

        // perform the delivery
        TRACE("broker notifying: %p, 0x%x, %p, 0x%lx\n",
              m_hwndClient, m_uMsg, ppidl, lEvent);
        SendMessageW(m_hwndClient, m_uMsg, (WPARAM)ppidl, lEvent);

        // unlock the ticket
        SHChangeNotification_Unlock(hLock);
        return TRUE;
    }
};

// This function creates a notification broker for old method. Used in SHChangeNotifyRegister.
extern "C" HWND
CreateNotificationBroker(HWND hwnd, UINT wMsg)
{
    // Create a new broker. It will be freed when the window gets destroyed
    CChangeNotifyBroker* pBroker = new CChangeNotifyBroker(hwnd, wMsg);
    if (pBroker == NULL)
    {
        ERR("Out of memory\n");
        return NULL;
    }

    HWND hwndBroker = SHCreateDefaultWorkerWindow();
    if (hwndBroker == NULL)
    {
        ERR("hwndBroker == NULL\n");
        delete pBroker;
        return NULL;
    }

    pBroker->SubclassWindow(hwndBroker);
    return hwndBroker;
}
