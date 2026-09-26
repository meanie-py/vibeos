#ifndef _SPIDER_PCH_
#define _SPIDER_PCH_

#include <cardlib.h>

#include "resource.h"

#define DIFFICULTY_ONE_COLOR     1
#define DIFFICULTY_TWO_COLORS    2
#define DIFFICULTY_FOUR_COLORS   3
#define CARDBACK_START           IDC_CARDBACK1
#define CARDBACK_END             IDC_CARDBACK12
#define NUM_CARDBACKS            (CARDBACK_END - CARDBACK_START + 1)
#define CARDBACK_RES_START       53
#define CARDBACK_OPTIONS_WIDTH   72
#define CARDBACK_OPTIONS_HEIGHT  112

#define X_BORDER                  5
#define Y_BORDER                  5

#define NUM_STACKS         10

extern HWND hwndMain;
extern CARDWINDOW *SpiderWnd;
extern TCHAR szAppName[];
extern bool fGameStarted;
extern int yRowStackCardOffset;
extern DWORD dwDifficulty;
extern TCHAR MsgDeal[];
extern TCHAR MsgWin[];

void CreateSpider(void);
void NewGame(void);

BOOL CARDLIBPROC RowStackDragProc(CARDREGION *stackobj, int iNumCards);
BOOL CARDLIBPROC RowStackDropProc(CARDREGION *stackobj,  const CARDSTACK *dragcards);

void CARDLIBPROC RowStackClickProc(CARDREGION *stackobj, int iNumClicked);

void CARDLIBPROC DeckClickProc(CARDREGION *stackobj, int iNumClicked);
void CARDLIBPROC PileDblClickProc(CARDREGION *stackobj, int iNumClicked);

void CARDLIBPROC PileRemoveProc(CARDREGION *stackobj, int iRemoved);

#endif /* _SPIDER_PCH_ */
