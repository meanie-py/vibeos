/*
 * PROJECT:     Solitaire
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * FILE:        base/applications/games/solitaire/solitaire.cpp
 * PURPOSE:     Frame window, menus, dialogs, options and status bar for
 *              Klondike Solitaire
 * PROGRAMMER:  Copyright 2026 vibeos contributors
 */

#include "solitaire.h"

HINSTANCE   hInstance;
HWND        hwndMain;
HWND        hwndStatus;
CARDWINDOW *SolWnd;

TCHAR szAppName[128];
TCHAR MsgQuit[128];
TCHAR MsgAbout[128];
TCHAR MsgWin[128];
TCHAR MsgDeal[128];
TCHAR MsgScore[64];
TCHAR MsgTime[64];

INT  nDrawCards  = 1;
INT  nScoreMode  = SCORE_STD;
INT  nCardBack   = CARDBACK_RES_START;
BOOL fKeepScore  = FALSE;
BOOL fShowTime   = TRUE;
BOOL fShowStatus = TRUE;

INT   lScore;
DWORD dwElapsed;
BOOL  fGameStarted;

static BOOL fTimerRunning = FALSE;

#define REGISTRY_KEY _T("Software\\ReactOS\\Solitaire")

static LRESULT CALLBACK WndProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);

/* ---------------------------------------------------------------------- *
 *  Settings
 * ---------------------------------------------------------------------- */

static void LoadDword(HKEY hKey, LPCTSTR pszName, INT *pnValue)
{
    DWORD dwType = 0;
    DWORD dwValue = 0;
    DWORD cbData = sizeof(dwValue);

    if (RegQueryValueEx(hKey, pszName, NULL, &dwType, (LPBYTE)&dwValue, &cbData) == ERROR_SUCCESS &&
        dwType == REG_DWORD && cbData == sizeof(dwValue))
    {
        *pnValue = (INT)dwValue;
    }
}

static void SaveDword(HKEY hKey, LPCTSTR pszName, INT nValue)
{
    DWORD dwValue = (DWORD)nValue;

    RegSetValueEx(hKey, pszName, 0, REG_DWORD, (const BYTE *)&dwValue, sizeof(dwValue));
}

static void LoadSettings(void)
{
    HKEY hKey;

    if (RegOpenKeyEx(HKEY_CURRENT_USER, REGISTRY_KEY, 0, KEY_QUERY_VALUE, &hKey) == ERROR_SUCCESS)
    {
        LoadDword(hKey, _T("DrawCards"), &nDrawCards);
        LoadDword(hKey, _T("ScoreMode"), &nScoreMode);
        LoadDword(hKey, _T("CardBack"),  &nCardBack);
        LoadDword(hKey, _T("KeepScore"), &fKeepScore);
        LoadDword(hKey, _T("ShowTime"),  &fShowTime);
        LoadDword(hKey, _T("StatusBar"), &fShowStatus);

        RegCloseKey(hKey);
    }

    /* Anything out of range is a hand-edited or newer registry.  Fall back
       rather than, say, indexing the bitmap strip off its end. */
    if (nDrawCards != 1 && nDrawCards != 3)
        nDrawCards = 1;
    if (nScoreMode < SCORE_NONE || nScoreMode > SCORE_VEGAS)
        nScoreMode = SCORE_STD;
    if (nCardBack < CARDBACK_RES_START || nCardBack >= CARDBACK_RES_START + NUM_CARDBACKS)
        nCardBack = CARDBACK_RES_START;

    fKeepScore  = fKeepScore  ? TRUE : FALSE;
    fShowTime   = fShowTime   ? TRUE : FALSE;
    fShowStatus = fShowStatus ? TRUE : FALSE;
}

static void SaveSettings(void)
{
    HKEY hKey;
    DWORD dwDisposition;

    if (RegCreateKeyEx(HKEY_CURRENT_USER, REGISTRY_KEY, 0, NULL, REG_OPTION_NON_VOLATILE,
                       KEY_SET_VALUE, NULL, &hKey, &dwDisposition) != ERROR_SUCCESS)
    {
        return;
    }

    SaveDword(hKey, _T("DrawCards"), nDrawCards);
    SaveDword(hKey, _T("ScoreMode"), nScoreMode);
    SaveDword(hKey, _T("CardBack"),  nCardBack);
    SaveDword(hKey, _T("KeepScore"), fKeepScore);
    SaveDword(hKey, _T("ShowTime"),  fShowTime);
    SaveDword(hKey, _T("StatusBar"), fShowStatus);

    RegCloseKey(hKey);
}

/* ---------------------------------------------------------------------- *
 *  Status bar and clock
 * ---------------------------------------------------------------------- */

INT GetScoreMode(void)
{
    return nScoreMode;
}

void UpdateStatusBar(void)
{
    TCHAR szText[128];

    if (hwndStatus == NULL)
        return;

    if (GetScoreMode() == SCORE_NONE)
        szText[0] = _T('\0');
    else
        wsprintf(szText, MsgScore, lScore);

    SendMessage(hwndStatus, SB_SETTEXT, 0, (LPARAM)szText);

    if (fShowTime)
        wsprintf(szText, MsgTime, (INT)dwElapsed);
    else
        szText[0] = _T('\0');

    SendMessage(hwndStatus, SB_SETTEXT, 1, (LPARAM)szText);
}

void SetUndoMenuState(BOOL bEnable)
{
    HMENU hMenu = GetMenu(hwndMain);

    if (hMenu != NULL)
    {
        EnableMenuItem(hMenu, IDM_GAME_UNDO,
                       MF_BYCOMMAND | (bEnable ? MF_ENABLED : MF_GRAYED));
    }
}

void StartPlayTimer(void)
{
    if (!fTimerRunning)
    {
        SetTimer(hwndMain, IDT_PLAYTIMER, 1000, NULL);
        fTimerRunning = TRUE;
    }
}

void StopPlayTimer(void)
{
    if (fTimerRunning)
    {
        KillTimer(hwndMain, IDT_PLAYTIMER);
        fTimerRunning = FALSE;
    }
}

static INT StatusBarHeight(void)
{
    RECT rc;

    if (hwndStatus == NULL || !IsWindowVisible(hwndStatus))
        return 0;

    GetWindowRect(hwndStatus, &rc);
    return rc.bottom - rc.top;
}

/* Score on the left, clock on the right. */
static void SizeStatusBar(void)
{
    RECT rc;
    INT parts[2];

    if (hwndStatus == NULL)
        return;

    GetClientRect(hwndMain, &rc);
    parts[0] = rc.right / 2;
    parts[1] = -1;

    SendMessage(hwndStatus, SB_SETPARTS, 2, (LPARAM)parts);
}

static void LayoutChildren(void)
{
    RECT rc;

    if (hwndMain == NULL)
        return;

    GetClientRect(hwndMain, &rc);

    if (hwndStatus != NULL)
    {
        SendMessage(hwndStatus, WM_SIZE, 0, 0);
        SizeStatusBar();
    }

    if (SolWnd != NULL)
        MoveWindow(CardWindow_GetHwnd(SolWnd), 0, 0, rc.right, rc.bottom - StatusBarHeight(), TRUE);

    UpdateStatusBar();
}

static void ApplyStatusBarVisibility(void)
{
    if (hwndStatus == NULL)
        return;

    ShowWindow(hwndStatus, fShowStatus ? SW_SHOW : SW_HIDE);
    LayoutChildren();
}

/* ---------------------------------------------------------------------- *
 *  Options
 * ---------------------------------------------------------------------- */

static INT ScoreModeToId(INT nMode)
{
    switch (nMode)
    {
        case SCORE_VEGAS: return IDC_OPT_VEGAS;
        case SCORE_NONE:  return IDC_OPT_NOSCORE;
        default:          return IDC_OPT_STANDARD;
    }
}

static INT_PTR CALLBACK OptionsDlgProc(HWND hDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    UNREFERENCED_PARAMETER(lParam);

    switch (uMsg)
    {
        case WM_INITDIALOG:
            CheckRadioButton(hDlg, IDC_OPT_DRAWONE, IDC_OPT_DRAWTHREE,
                             (nDrawCards == 3) ? IDC_OPT_DRAWTHREE : IDC_OPT_DRAWONE);
            CheckRadioButton(hDlg, IDC_OPT_STANDARD, IDC_OPT_NOSCORE, ScoreModeToId(nScoreMode));
            CheckDlgButton(hDlg, IDC_OPT_KEEPSCORE, fKeepScore ? BST_CHECKED : BST_UNCHECKED);
            CheckDlgButton(hDlg, IDC_OPT_SHOWTIME, fShowTime ? BST_CHECKED : BST_UNCHECKED);
            CheckDlgButton(hDlg, IDC_OPT_STATUSBAR, fShowStatus ? BST_CHECKED : BST_UNCHECKED);

            /* A running bank is a Vegas idea; it means nothing to the others. */
            EnableWindow(GetDlgItem(hDlg, IDC_OPT_KEEPSCORE), nScoreMode == SCORE_VEGAS);
            return TRUE;

        case WM_COMMAND:
            switch (LOWORD(wParam))
            {
                case IDC_OPT_STANDARD:
                case IDC_OPT_VEGAS:
                case IDC_OPT_NOSCORE:
                    EnableWindow(GetDlgItem(hDlg, IDC_OPT_KEEPSCORE),
                                 IsDlgButtonChecked(hDlg, IDC_OPT_VEGAS) == BST_CHECKED);
                    return TRUE;

                case IDOK:
                {
                    INT nOldMode = nScoreMode;

                    nDrawCards = (IsDlgButtonChecked(hDlg, IDC_OPT_DRAWTHREE) == BST_CHECKED) ? 3 : 1;

                    if (IsDlgButtonChecked(hDlg, IDC_OPT_VEGAS) == BST_CHECKED)
                        nScoreMode = SCORE_VEGAS;
                    else if (IsDlgButtonChecked(hDlg, IDC_OPT_NOSCORE) == BST_CHECKED)
                        nScoreMode = SCORE_NONE;
                    else
                        nScoreMode = SCORE_STD;

                    fKeepScore  = (IsDlgButtonChecked(hDlg, IDC_OPT_KEEPSCORE) == BST_CHECKED);
                    fShowTime   = (IsDlgButtonChecked(hDlg, IDC_OPT_SHOWTIME) == BST_CHECKED);
                    fShowStatus = (IsDlgButtonChecked(hDlg, IDC_OPT_STATUSBAR) == BST_CHECKED);

                    /* Switching scoring systems mid-deal: the running total
                       means something else now, and so does the score the
                       undo module has banked.  Start both over rather than
                       mixing two currencies.  The deal itself is left alone -
                       the player keeps the game they are in. */
                    if (nScoreMode != nOldMode)
                    {
                        lScore = (nScoreMode == SCORE_VEGAS) ? -SCORE_VEGAS_STAKE : 0;
                        ClearUndo();
                    }

                    SaveSettings();
                    ApplyStatusBarVisibility();
                    UpdateStatusBar();

                    EndDialog(hDlg, TRUE);
                    return TRUE;
                }

                case IDCANCEL:
                    EndDialog(hDlg, FALSE);
                    return TRUE;
            }
            break;
    }

    return FALSE;
}

/* ---------------------------------------------------------------------- *
 *  Deck (card back) picker
 * ---------------------------------------------------------------------- */

typedef struct _CARDBACK
{
    HWND    hSelf;
    WNDPROC pfnOldProc;
    INT     nSlot;      /* slot in the card bitmap strip */
    BOOL    bSelected;
} CARDBACK;

static CARDBACK CardBacks[NUM_CARDBACKS];

/*
 * Each of the twelve statics in IDD_CARDBACK is subclassed to draw one card
 * back, blitted straight out of the library's composed strip and stretched to
 * whatever size the dialog template gave the control.
 */
static LRESULT CALLBACK CardImageWndProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    CARDBACK *pBack = (CARDBACK *)GetWindowLongPtr(hwnd, GWLP_USERDATA);

    if (pBack == NULL)
        return DefWindowProc(hwnd, uMsg, wParam, lParam);

    if (uMsg == WM_PAINT)
    {
        PAINTSTRUCT ps;
        RECT rc;
        HDC hdc;
        HPEN hPen, hOldPen;
        HBRUSH hOldBrush;

        hdc = BeginPaint(hwnd, &ps);
        GetClientRect(hwnd, &rc);

        hPen = CreatePen(PS_SOLID, 2,
                         pBack->bSelected ? RGB(0, 0, 0) : (COLORREF)GetSysColor(COLOR_3DFACE));
        hOldPen = (HPEN)SelectObject(hdc, hPen);
        hOldBrush = (HBRUSH)SelectObject(hdc, GetStockObject(NULL_BRUSH));

        Rectangle(hdc, rc.left + 1, rc.top + 1, rc.right, rc.bottom);

        StretchBlt(hdc, 3, 3, rc.right - 6, rc.bottom - 6,
                   CardLib_CardBitmapsDC(), pBack->nSlot * CardLib_CardWidth(), 0,
                   CardLib_CardWidth(), CardLib_CardHeight(), SRCCOPY);

        SelectObject(hdc, hOldBrush);
        SelectObject(hdc, hOldPen);
        DeleteObject(hPen);

        EndPaint(hwnd, &ps);
        return 0;
    }

    return CallWindowProc(pBack->pfnOldProc, hwnd, uMsg, wParam, lParam);
}

static INT_PTR CALLBACK CardBackDlgProc(HWND hDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    INT i;

    switch (uMsg)
    {
        case WM_INITDIALOG:
            for (i = 0; i < NUM_CARDBACKS; i++)
            {
                CardBacks[i].hSelf     = GetDlgItem(hDlg, CARDBACK_START + i);
                CardBacks[i].nSlot     = CARDBACK_RES_START + i;
                CardBacks[i].bSelected = (CARDBACK_RES_START + i == nCardBack);

                /* Hang the state off the control BEFORE subclassing it, so the
                   new proc can never run against a control that has none. */
                SetWindowLongPtr(CardBacks[i].hSelf, GWLP_USERDATA, (LONG_PTR)&CardBacks[i]);
                CardBacks[i].pfnOldProc = (WNDPROC)SetWindowLongPtr(CardBacks[i].hSelf,
                                                                    GWLP_WNDPROC,
                                                                    (LONG_PTR)CardImageWndProc);
            }
            return TRUE;

        case WM_COMMAND:
        {
            INT nId = (INT)LOWORD(wParam);

            /* IDOK first: STN_CLICKED and BN_CLICKED are both 0, so testing
               the notification code before the control id would swallow the
               buttons. */
            if (nId == IDOK)
            {
                INT nChosen = nCardBack;

                for (i = 0; i < NUM_CARDBACKS; i++)
                {
                    if (CardBacks[i].bSelected)
                        nChosen = CardBacks[i].nSlot;
                }

                EndDialog(hDlg, nChosen);
                return TRUE;
            }

            if (nId == IDCANCEL)
            {
                EndDialog(hDlg, 0);
                return TRUE;
            }

            if (nId >= CARDBACK_START && nId <= CARDBACK_END && HIWORD(wParam) == STN_CLICKED)
            {
                for (i = 0; i < NUM_CARDBACKS; i++)
                {
                    CardBacks[i].bSelected = (CardBacks[i].hSelf == (HWND)lParam);
                    InvalidateRect(CardBacks[i].hSelf, NULL, TRUE);
                }
                return TRUE;
            }
            break;
        }
    }

    return FALSE;
}

static void ShowDeckOptionsDlg(HWND hwnd)
{
    INT_PTR nSlot = DialogBox(hInstance, MAKEINTRESOURCE(IDD_CARDBACK), hwnd, CardBackDlgProc);

    if (nSlot >= CARDBACK_RES_START && nSlot < CARDBACK_RES_START + NUM_CARDBACKS)
    {
        nCardBack = (INT)nSlot;
        CardWindow_SetBackCardIdx(SolWnd, (UINT)nCardBack);
        CardWindow_Redraw(SolWnd);
        SaveSettings();
    }
}

/* ---------------------------------------------------------------------- *
 *  Frame window
 * ---------------------------------------------------------------------- */

static LRESULT CALLBACK WndProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    switch (uMsg)
    {
        case WM_CREATE:
            /* Everything below needs this, and WM_CREATE runs before
               CreateWindow has returned it to WinMain. */
            hwndMain = hwnd;

            /* No HTML Help to point it at. */
            EnableMenuItem(GetMenu(hwnd), IDM_HELP_CONTENTS, MF_BYCOMMAND | MF_GRAYED);

            hwndStatus = CreateWindowEx(0, STATUSCLASSNAME, NULL,
                                        WS_CHILD | SBARS_SIZEGRIP,
                                        0, 0, 0, 0,
                                        hwnd, (HMENU)(UINT_PTR)IDC_STATUSBAR, hInstance, NULL);
            if (hwndStatus != NULL && fShowStatus)
                ShowWindow(hwndStatus, SW_SHOW);

            SolWnd = CardWindow_Create(hwnd, 0, WS_CHILD | WS_VISIBLE, 0, 0, 100, 100);
            if (SolWnd == NULL)
                return -1;

            CardWindow_SetBackCardIdx(SolWnd, (UINT)nCardBack);

            CreateSol();
            SetUndoMenuState(FALSE);
            NewGame();

            /* A zero-sized resize, which WM_GETMINMAXINFO clamps up to a
               table that actually fits seven columns. */
            SetWindowPos(hwnd, NULL, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOZORDER);
            return 0;

        case WM_SIZE:
            LayoutChildren();
            return 0;

        case WM_GETMINMAXINFO:
        {
            MINMAXINFO *mmi = (MINMAXINFO *)lParam;

            mmi->ptMinTrackSize.x = NUM_ROW_STACKS * CardLib_CardWidth()
                                  + (NUM_ROW_STACKS + 1) * X_BORDER
                                  + 2 * GetSystemMetrics(SM_CXFRAME);

            /* Room for the top row, a tableau column of thirteen fanned
               cards, and the chrome around them. */
            mmi->ptMinTrackSize.y = GetSystemMetrics(SM_CYCAPTION)
                                  + GetSystemMetrics(SM_CYMENU)
                                  + 2 * GetSystemMetrics(SM_CYFRAME)
                                  + 3 * Y_BORDER
                                  + 2 * CardLib_CardHeight()
                                  + 12 * yRowStackCardOffset
                                  + StatusBarHeight();
            return 0;
        }

        case WM_TIMER:
            if (wParam == (WPARAM)IDT_PLAYTIMER)
            {
                dwElapsed++;
                UpdateStatusBar();
            }
            return 0;

        case WM_SOL_GAMEWON:
            GameFinished();
            return 0;

        case WM_COMMAND:
            switch (LOWORD(wParam))
            {
                case IDM_GAME_NEW:
                    NewGame();
                    return 0;

                case IDM_GAME_UNDO:
                    Undo();
                    /* Undo rebuilds the waste behind our back, and knows
                       nothing about the fan's centring or the stock's empty
                       image.  Rebuilding the view from the restored
                       activepile reproduces exactly what Undo just put there
                       and fixes up both. */
                    UpdatePileDisplay(VisiblePileCards);
                    CardWindow_Redraw(SolWnd);
                    return 0;

                case IDM_GAME_DECK:
                    ShowDeckOptionsDlg(hwnd);
                    return 0;

                case IDM_GAME_OPTIONS:
                    DialogBox(hInstance, MAKEINTRESOURCE(IDD_OPTIONS), hwnd, OptionsDlgProc);
                    return 0;

                case IDM_GAME_EXIT:
                    PostMessage(hwnd, WM_CLOSE, 0, 0);
                    return 0;

                case IDM_HELP_CONTENTS:
                    /* Greyed out above; there is no HTML Help to show. */
                    return 0;

                case IDM_HELP_ABOUT:
                    ShellAbout(hwnd, szAppName, MsgAbout,
                               LoadIcon(hInstance, MAKEINTRESOURCE(IDI_SOLITAIRE)));
                    return 0;
            }
            return 0;

        case WM_CLOSE:
            if (!fGameStarted ||
                MessageBox(hwnd, MsgQuit, szAppName, MB_YESNO | MB_ICONQUESTION) == IDYES)
            {
                SaveSettings();
                DestroyWindow(hwnd);
            }
            return 0;

        case WM_DESTROY:
            StopPlayTimer();
            if (SolWnd != NULL)
            {
                CardWindow_Destroy(SolWnd);
                SolWnd = NULL;
            }
            PostQuitMessage(0);
            return 0;
    }

    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

int WINAPI _tWinMain(HINSTANCE hInst, HINSTANCE hPrev, LPTSTR szCmdLine, int iCmdShow)
{
    HWND hwnd;
    MSG msg;
    WNDCLASS wndclass;
    INITCOMMONCONTROLSEX ice;
    HACCEL hAccelTable;

    UNREFERENCED_PARAMETER(hPrev);
    UNREFERENCED_PARAMETER(szCmdLine);

    hInstance = hInst;

    /* The title doubles as the window class name, so it has to exist. */
    LoadString(hInst, IDS_SOL_NAME, szAppName, sizeof(szAppName) / sizeof(szAppName[0]));
    if (szAppName[0] == _T('\0'))
        lstrcpy(szAppName, _T("Solitaire"));

    /* Loaded once here rather than every time a message box goes up. */
    LoadString(hInst, IDS_SOL_ABOUT, MsgAbout, sizeof(MsgAbout) / sizeof(MsgAbout[0]));
    LoadString(hInst, IDS_SOL_QUIT,  MsgQuit,  sizeof(MsgQuit)  / sizeof(MsgQuit[0]));
    LoadString(hInst, IDS_SOL_WIN,   MsgWin,   sizeof(MsgWin)   / sizeof(MsgWin[0]));
    LoadString(hInst, IDS_SOL_DEAL,  MsgDeal,  sizeof(MsgDeal)  / sizeof(MsgDeal[0]));
    LoadString(hInst, IDS_SOL_SCORE, MsgScore, sizeof(MsgScore) / sizeof(MsgScore[0]));
    LoadString(hInst, IDS_SOL_TIME,  MsgTime,  sizeof(MsgTime)  / sizeof(MsgTime[0]));

    LoadSettings();

    ice.dwSize = sizeof(ice);
    ice.dwICC = ICC_BAR_CLASSES;
    InitCommonControlsEx(&ice);

    /* Nothing below here can draw a card without the artwork, so a failed
       load is fatal rather than an empty green table. */
    if (!CardLib_Init())
    {
        MessageBox(NULL, _T("Failed to load the card images from cards.dll."),
                   szAppName, MB_OK | MB_ICONERROR);
        return 1;
    }

    wndclass.style         = 0;
    wndclass.lpfnWndProc   = WndProc;
    wndclass.cbClsExtra    = 0;
    wndclass.cbWndExtra    = 0;
    wndclass.hInstance     = hInst;
    wndclass.hIcon         = LoadIcon(hInst, MAKEINTRESOURCE(IDI_SOLITAIRE));
    wndclass.hCursor       = LoadCursor(NULL, IDC_ARROW);
    wndclass.hbrBackground = (HBRUSH)(COLOR_3DFACE + 1);
    wndclass.lpszMenuName  = MAKEINTRESOURCE(IDR_MENU1);
    wndclass.lpszClassName = szAppName;

    if (!RegisterClass(&wndclass))
    {
        CardLib_Term();
        return 1;
    }

    hwnd = CreateWindow(szAppName,
                        szAppName,
                        WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        0,  /* the real size comes from WM_GETMINMAXINFO */
                        0,
                        NULL,
                        NULL,
                        hInst,
                        NULL);
    if (hwnd == NULL)
    {
        CardLib_Term();
        return 1;
    }

    ShowWindow(hwnd, iCmdShow);
    UpdateWindow(hwnd);

    hAccelTable = LoadAccelerators(hInst, MAKEINTRESOURCE(IDR_ACCELERATOR1));

    while (GetMessage(&msg, NULL, 0, 0))
    {
        if (!TranslateAccelerator(hwnd, hAccelTable, &msg))
        {
            TranslateMessage(&msg);
            DispatchMessage(&msg);
        }
    }

    CardLib_Term();

    return (int)msg.wParam;
}
