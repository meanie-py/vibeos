/*
 * PROJECT:     Solitaire
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * FILE:        base/applications/games/solitaire/solitaire.h
 * PURPOSE:     Shared declarations for Klondike Solitaire
 * PROGRAMMER:  Copyright 2026 vibeos contributors
 *
 * This is also the precompiled header (see CMakeLists.txt), so it carries the
 * Win32 surface all three translation units need.  solundo.cpp includes it and
 * is NOT ours to change, so everything it reaches for - the region pointers,
 * the id constants, activepile, VisiblePileCards, lScore, GetScoreMode,
 * SetUndoMenuState and UpdateStatusBar - is declared here in the shape that
 * file already expects.
 */

#ifndef _SOLITAIRE_PCH_
#define _SOLITAIRE_PCH_

#include <cardlib.h>

#include <winreg.h>
#include <commctrl.h>
#include <shellapi.h>
#include <tchar.h>

#include "resource.h"

/* ---------------------------------------------------------------------- *
 *  The table
 * ---------------------------------------------------------------------- */

#define NUM_ROW_STACKS      7
#define NUM_SUIT_STACKS     4
#define NUM_CARDS           52

/* Gap between piles, and between a pile and the edge of the table. */
#define X_BORDER            10
#define Y_BORDER            10

/*
 * Region ids.  solundo.cpp maps an id back to a region and treats anything
 * below 1, or above ROW_ID + NUM_ROW_STACKS - 1, as "no move recorded", so the
 * numbering has to start at 1, stay dense, and keep the suit and row stacks in
 * consecutive runs.
 */
#define DECK_ID             1
#define PILE_ID             2
#define SUIT_ID             3                               /* .. SUIT_ID + 3 */
#define ROW_ID              (SUIT_ID + NUM_SUIT_STACKS)     /* .. ROW_ID + 6  */

/* Scoring systems, as offered by IDD_OPTIONS and stored in the registry. */
#define SCORE_NONE          0
#define SCORE_STD           1
#define SCORE_VEGAS         2

/* Standard scoring deltas. */
#define SCORE_STD_TO_SUIT       10
#define SCORE_STD_FROM_SUIT     15
#define SCORE_STD_PILE_TO_ROW    5
#define SCORE_STD_TURNOVER       5
/* The classic time bonus: quicker deals pay more, and only when the player
   was actually watching the clock. */
#define SCORE_STD_BONUS_BASE    700000
#define SCORE_STD_BONUS_MINTIME 30

/* Vegas: the deal is bought, and each card banked pays back a fifth of it. */
#define SCORE_VEGAS_STAKE       52
#define SCORE_VEGAS_PER_CARD     5

/*
 * The card-back picker.  The twelve backs are consecutive slots in the bitmap
 * strip starting at ecbCROSSHATCH, and IDD_CARDBACK's twelve statics are
 * consecutive too, so one index walks both.
 */
#define CARDBACK_START      IDC_CARDBACK1
#define CARDBACK_END        IDC_CARDBACK12
#define NUM_CARDBACKS       (CARDBACK_END - CARDBACK_START + 1)
#define CARDBACK_RES_START  ecbCROSSHATCH

/* Our own child-window and timer ids; resource.h owns everything below 1400. */
#define IDC_STATUSBAR       2000
#define IDT_PLAYTIMER       1

/*
 * Posted to the frame when the fourth foundation fills up.
 *
 * The win sequence must NOT run inside the foundation's add callback: that
 * callback fires in the middle of the library's drop handling, which goes on
 * to touch the SOURCE region afterwards.  Dealing a new game from in there
 * would leave the library turning a card of the fresh deal.
 */
#define WM_SOL_GAMEWON      (WM_APP + 1)

/* ---------------------------------------------------------------------- *
 *  solitaire.cpp - frame window, dialogs, options, status bar
 * ---------------------------------------------------------------------- */

extern HINSTANCE   hInstance;
extern HWND        hwndMain;
extern HWND        hwndStatus;
extern CARDWINDOW *SolWnd;

extern TCHAR szAppName[];
extern TCHAR MsgQuit[];
extern TCHAR MsgAbout[];
extern TCHAR MsgWin[];
extern TCHAR MsgDeal[];
extern TCHAR MsgScore[];
extern TCHAR MsgTime[];

/* Options, as persisted under HKCU\Software\ReactOS\Solitaire. */
extern INT  nDrawCards;     /* 1 or 3 cards turned per click on the stock */
extern INT  nScoreMode;     /* SCORE_NONE / SCORE_STD / SCORE_VEGAS       */
extern INT  nCardBack;      /* strip slot, CARDBACK_RES_START .. +11      */
extern BOOL fKeepScore;     /* Vegas: carry the bank between deals        */
extern BOOL fShowTime;      /* timed game - shows, and pays a bonus       */
extern BOOL fShowStatus;    /* show the status bar                        */

extern INT   lScore;        /* solundo.cpp saves and restores this        */
extern DWORD dwElapsed;     /* seconds since play started                 */
extern BOOL  fGameStarted;

INT  GetScoreMode(void);
void UpdateStatusBar(void);
void SetUndoMenuState(BOOL bEnable);
void StartPlayTimer(void);
void StopPlayTimer(void);

/* ---------------------------------------------------------------------- *
 *  solcreate.cpp - the regions and where they sit
 * ---------------------------------------------------------------------- */

extern CARDREGION *pDeck;                       /* the stock        */
extern CARDREGION *pPile;                       /* the waste        */
extern CARDREGION *pSuitStack[NUM_SUIT_STACKS]; /* the foundations  */
extern CARDREGION *pRowStack[NUM_ROW_STACKS];   /* the tableau      */

extern INT yRowStackCardOffset;   /* how far a buried tableau card peeks out */
extern INT xPileAdjust;           /* the waste's unfanned centring offset    */

void CreateSol(void);

/* ---------------------------------------------------------------------- *
 *  solgame.cpp - the rules
 * ---------------------------------------------------------------------- */

/*
 * The waste, in full.  pPile is only a WINDOW onto it: with one card turned it
 * holds the whole pile squared up, and with three it holds just the fanned
 * top few.  solundo.cpp rebuilds that window from activepile after an undo,
 * which is why the pile lives in a plain CARDSTACK and not in the region.
 */
extern CARDSTACK activepile;
extern INT       VisiblePileCards;

void NewGame(void);
void GameFinished(void);
void UpdatePileDisplay(INT nVisible);
void UpdatePilePlacement(void);
void TurnRowStackCard(CARDREGION *pRegion);

BOOL CARDLIBPROC PileDragProc      (CARDREGION *pRegion, INT nNumDragging);
BOOL CARDLIBPROC SuitStackDragProc (CARDREGION *pRegion, INT nNumDragging);
BOOL CARDLIBPROC RowStackDragProc  (CARDREGION *pRegion, INT nNumDragging);

BOOL CARDLIBPROC SuitStackDropProc (CARDREGION *pRegion, CARDSTACK *pDragCards);
BOOL CARDLIBPROC RowStackDropProc  (CARDREGION *pRegion, CARDSTACK *pDragCards);

void CARDLIBPROC DeckClickProc        (CARDREGION *pRegion, INT nNumCards);
void CARDLIBPROC RowStackClickProc    (CARDREGION *pRegion, INT nNumCards);
void CARDLIBPROC PileDblClickProc     (CARDREGION *pRegion, INT nNumCards);
void CARDLIBPROC RowStackDblClickProc (CARDREGION *pRegion, INT nNumCards);

void CARDLIBPROC SuitStackAddProc     (CARDREGION *pRegion, const CARDSTACK *pAdded);
void CARDLIBPROC PileRemoveProc       (CARDREGION *pRegion, INT nRemoved);
void CARDLIBPROC RowStackRemoveProc   (CARDREGION *pRegion, INT nRemoved);

/* ---------------------------------------------------------------------- *
 *  solundo.cpp - one move deep, and not ours to change
 * ---------------------------------------------------------------------- */

void SetUndo(int set_source_id,
             int set_destination_id,
             int set_number_of_cards,
             int set_prev_score,
             int set_prev_visible_pile_cards);
void ClearUndo(void);
void Undo(void);

#endif /* _SOLITAIRE_PCH_ */
