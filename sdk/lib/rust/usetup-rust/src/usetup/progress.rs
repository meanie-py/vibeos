use crate::usetup::consup::CONSOLE_SetTextXY;
use crate::usetup::console::FillConsoleOutputAttribute;
use crate::usetup::console::FillConsoleOutputCharacterA;
use crate::usetup::console::WriteConsoleOutputCharacterA;
use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "system" fn UpdateProgressPercentage(
    mut Bar: PPROGRESSBAR,
    mut AlwaysUpdate: BOOLEAN,
    mut Buffer: PSTR,
    mut cchBufferSize: SIZE_T,
) -> BOOLEAN {
    let mut OldProgress: ULONG = (*Bar).Progress;
    if (*Bar).StepCount == 0 {
        (*Bar).Progress = 0;
    } else {
        (*Bar).Progress = (100 as ULONG)
            .wrapping_mul((*Bar).CurrentStep)
            .wrapping_add((*Bar).StepCount.wrapping_div(2))
            .wrapping_div((*Bar).StepCount);
    }
    if !(*Bar).ProgressFormatText.is_null()
        && (AlwaysUpdate as ::core::ffi::c_int != 0 || (*Bar).Progress != OldProgress)
    {
        RtlStringCchPrintfA!(
            Buffer as NTSTRSAFE_PSTR,
            cchBufferSize as size_t,
            (*Bar).ProgressFormatText as NTSTRSAFE_PCSTR,
            (*Bar).Progress,
        );
        return (TRUE) as BOOLEAN;
    }
    return (FALSE) as BOOLEAN;
}

unsafe extern "C" fn DrawBorder(Bar: PPROGRESSBAR) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    let mut i: SHORT = 0;

    coPos.X = (*Bar).Left;
    coPos.Y = ((*Bar).Top as i32 + 1) as SHORT;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperLeftCorner,
        1,
        coPos,
        &mut Written,
    );

    coPos.X = ((*Bar).Left as i32 + 1) as SHORT;
    coPos.Y = ((*Bar).Top as i32 + 1) as SHORT;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharHorizontalLine,
        ((*Bar).Right as i32 - (*Bar).Left as i32 - 1) as DWORD,
        coPos,
        &mut Written,
    );

    coPos.X = (*Bar).Right;
    coPos.Y = ((*Bar).Top as i32 + 1) as SHORT;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperRightCorner,
        1,
        coPos,
        &mut Written,
    );

    i = ((*Bar).Top as i32 + 2) as SHORT;
    while (i as i32) < (*Bar).Bottom as i32 {
        coPos.X = (*Bar).Left;
        coPos.Y = i;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &mut Written,
        );

        coPos.X = (*Bar).Right;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &mut Written,
        );
        i += 1;
    }

    coPos.X = (*Bar).Left;
    coPos.Y = (*Bar).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharLowerLeftCorner,
        1,
        coPos,
        &mut Written,
    );

    coPos.X = ((*Bar).Left as i32 + 1) as SHORT;
    coPos.Y = (*Bar).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharHorizontalLine,
        ((*Bar).Right as i32 - (*Bar).Left as i32 - 1) as DWORD,
        coPos,
        &mut Written,
    );

    coPos.X = (*Bar).Right;
    coPos.Y = (*Bar).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharLowerRightCorner,
        1,
        coPos,
        &mut Written,
    );
}

unsafe extern "C" fn DrawThickBorder(Bar: PPROGRESSBAR) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    let mut i: SHORT = 0;

    /* draw upper left corner */
    coPos.X = (*Bar).Left;
    coPos.Y = ((*Bar).Top as i32 + 1) as SHORT;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharDoubleUpperLeftCorner,
        1,
        coPos,
        &mut Written,
    );

    /* draw upper edge */
    coPos.X = ((*Bar).Left as i32 + 1) as SHORT;
    coPos.Y = ((*Bar).Top as i32 + 1) as SHORT;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharDoubleHorizontalLine,
        ((*Bar).Right as i32 - (*Bar).Left as i32 - 1) as DWORD,
        coPos,
        &mut Written,
    );

    /* draw upper right corner */
    coPos.X = (*Bar).Right;
    coPos.Y = ((*Bar).Top as i32 + 1) as SHORT;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharDoubleUpperRightCorner,
        1,
        coPos,
        &mut Written,
    );

    /* draw left and right edge */
    i = ((*Bar).Top as i32 + 2) as SHORT;
    while (i as i32) < (*Bar).Bottom as i32 {
        coPos.X = (*Bar).Left;
        coPos.Y = i;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharDoubleVerticalLine,
            1,
            coPos,
            &mut Written,
        );

        coPos.X = (*Bar).Right;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharDoubleVerticalLine,
            1,
            coPos,
            &mut Written,
        );
        i += 1;
    }

    /* draw lower left corner */
    coPos.X = (*Bar).Left;
    coPos.Y = (*Bar).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharDoubleLowerLeftCorner,
        1,
        coPos,
        &mut Written,
    );

    /* draw lower edge */
    coPos.X = ((*Bar).Left as i32 + 1) as SHORT;
    coPos.Y = (*Bar).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharDoubleHorizontalLine,
        ((*Bar).Right as i32 - (*Bar).Left as i32 - 1) as DWORD,
        coPos,
        &mut Written,
    );

    /* draw lower right corner */
    coPos.X = (*Bar).Right;
    coPos.Y = (*Bar).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharDoubleLowerRightCorner,
        1,
        coPos,
        &mut Written,
    );
}

unsafe extern "C" fn DrawProgressBar(Bar: PPROGRESSBAR) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    let mut BarBorder: PROGRESSBAR = *Bar;
    let mut TextBuffer: [CHAR; 256] = [0; 256];

    if (*Bar).DoubleEdge != 0 {
        BarBorder.Top = (BarBorder.Top as i32 - 5) as SHORT;
        BarBorder.Bottom = (BarBorder.Bottom as i32 + 2) as SHORT;
        BarBorder.Right = (BarBorder.Right as i32 + 5) as SHORT;
        BarBorder.Left = (BarBorder.Left as i32 - 5) as SHORT;
        DrawThickBorder(&mut BarBorder);
    }

    DrawBorder(Bar);

    if !(*Bar).DescriptionText.is_null() {
        CONSOLE_SetTextXY((*Bar).TextTop, (*Bar).TextRight, (*Bar).DescriptionText);
    }

    if !(*Bar).UpdateProgressProc.is_none()
        && (*Bar).UpdateProgressProc.unwrap_unchecked()(
            Bar,
            TRUE as u8,
            TextBuffer.as_mut_ptr(),
            (ARRAYSIZE!(TextBuffer)) as _,
        ) != 0
    {
        coPos.X = ((*Bar).Left as i32
            + ((*Bar).Width as i32 - strlen(TextBuffer.as_mut_ptr()) as i32
                + 1)
                / 2) as SHORT;
        coPos.Y = (*Bar).Top;
        WriteConsoleOutputCharacterA(
            StdOutput,
            TextBuffer.as_mut_ptr(),
            (strlen(TextBuffer.as_mut_ptr())) as DWORD,
            coPos,
            &mut Written,
        );
    }

    coPos.X = ((*Bar).Left as i32 + 1) as SHORT;
    coPos.Y = ((*Bar).Top as i32 + 2) as SHORT;
    while coPos.Y as i32 <= (*Bar).Bottom as i32 - 1 {
        FillConsoleOutputAttribute(
            StdOutput,
            ((*Bar).ProgressColour) as WORD,
            ((*Bar).Width as i32 - 2) as DWORD,
            coPos,
            &mut Written,
        );

        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as CHAR,
            ((*Bar).Width as i32 - 2) as DWORD,
            coPos,
            &mut Written,
        );

        coPos.Y += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreateProgressBarEx(
    Left: SHORT,
    Top: SHORT,
    Right: SHORT,
    Bottom: SHORT,
    TextTop: SHORT,
    TextRight: SHORT,
    DoubleEdge: BOOLEAN,
    ProgressColour: SHORT,
    StepCount: ULONG,
    DescriptionText: PCSTR,
    ProgressFormatText: PCSTR,
    UpdateProgressProc: PUPDATE_PROGRESS,
) -> PPROGRESSBAR {
    let mut Bar: PPROGRESSBAR = core::ptr::null_mut();

    Bar = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<PROGRESSBAR>()) as SIZE_T,
    ) as PPROGRESSBAR;

    if Bar.is_null() {
        return core::ptr::null_mut();
    }

    (*Bar).Left = Left;
    (*Bar).Top = Top;
    (*Bar).Right = Right;
    (*Bar).Bottom = Bottom;
    (*Bar).TextTop = TextTop;
    (*Bar).TextRight = TextRight;

    (*Bar).Width = ((*Bar).Right as i32 - (*Bar).Left as i32 + 1) as SHORT;

    (*Bar).DoubleEdge = DoubleEdge;
    (*Bar).ProgressColour = ProgressColour;
    (*Bar).DescriptionText = DescriptionText;
    (*Bar).ProgressFormatText = ProgressFormatText;

    (*Bar).UpdateProgressProc = UpdateProgressProc;

    ProgressSetStepCount(Bar, StepCount);

    Bar
}
#[no_mangle]
pub unsafe extern "C" fn CreateProgressBar(Left: SHORT, Top: SHORT, Right: SHORT, Bottom: SHORT, TextTop: SHORT, TextRight: SHORT, DoubleEdge: BOOLEAN, DescriptionText: PCSTR) -> PPROGRESSBAR {
    return CreateProgressBarEx(
        Left,
        Top,
        Right,
        Bottom,
        TextTop,
        TextRight,
        DoubleEdge,
        (FOREGROUND_YELLOW | (BACKGROUND_BLUE) as u32) as SHORT,
        0,
        DescriptionText,
        b"%-3lu%%\0".as_ptr() as PCSTR,
        Some(UpdateProgressPercentage),
    );
}
#[no_mangle]
pub unsafe extern "C" fn DestroyProgressBar(Bar: PPROGRESSBAR) {
    RtlFreeHeap(ProcessHeap, 0, (Bar).cast());
}
#[no_mangle]
pub unsafe extern "C" fn ProgressSetStepCount(Bar: PPROGRESSBAR, StepCount: ULONG) {
    (*Bar).CurrentStep = 0;
    (*Bar).StepCount = StepCount;
    (*Bar).Progress = 0;
    (*Bar).Pos = 0;
    DrawProgressBar(Bar);
}
#[no_mangle]
pub unsafe extern "C" fn ProgressNextStep(Bar: PPROGRESSBAR) {
    ProgressSetStep(Bar, (*Bar).CurrentStep + 1);
}
#[no_mangle]
pub unsafe extern "C" fn ProgressSetStep(Bar: PPROGRESSBAR, Step: ULONG) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    let mut NewPos: ULONG = 0;
    let mut TextBuffer: [CHAR; 256] = [0; 256];

    if Step > (*Bar).StepCount {
        return;
    }

    (*Bar).CurrentStep = Step;

    if !(*Bar).UpdateProgressProc.is_none()
        && (((*Bar).UpdateProgressProc).unwrap_unchecked())(
            Bar,
            FALSE as u8,
            TextBuffer.as_mut_ptr() as PSTR,
            (ARRAYSIZE!(TextBuffer)) as _,
        ) != 0
    {
        coPos.X = ((*Bar).Left as i32
            + ((*Bar).Width as i32
                - strlen(TextBuffer.as_mut_ptr() as *mut CHAR) as u16 as i32
                + 1)
                / 2) as SHORT;
        coPos.Y = (*Bar).Top;
        WriteConsoleOutputCharacterA(
            StdOutput,
            TextBuffer.as_mut_ptr() as *const CHAR,
            strlen(TextBuffer.as_mut_ptr() as *mut CHAR) as DWORD,
            coPos,
            &mut Written,
        );
    }

    NewPos = ((((*Bar).Width as ULONG - 2)
        * 2)
        .wrapping_mul((*Bar).CurrentStep)
        .wrapping_add((*Bar).StepCount / 2))
        / (*Bar).StepCount;

    if (*Bar).Pos as ULONG != NewPos {
        (*Bar).Pos = NewPos as SHORT;

        coPos.Y = ((*Bar).Top as i32 + 2) as SHORT;
        while coPos.Y as i32 <= (*Bar).Bottom as i32 - 1 {
            coPos.X = ((*Bar).Left as i32 + 1) as SHORT;
            FillConsoleOutputCharacterA(
                StdOutput,
                CharBlock,
                ((*Bar).Pos as ULONG / 2) as DWORD,
                coPos,
                &mut Written,
            );
            coPos.X = (coPos.X as i32 + (*Bar).Pos as i32 / 2) as SHORT;

            if NewPos & 1 != 0 {
                FillConsoleOutputCharacterA(
                    StdOutput,
                    CharHalfBlock,
                    1,
                    coPos,
                    &mut Written,
                );
                coPos.X += 1;
            }

            if coPos.X as i32 <= (*Bar).Right as i32 - 1 {
                FillConsoleOutputCharacterA(
                    StdOutput,
                    ' ' as CHAR,
                    ((*Bar).Right as i32 - coPos.X as i32) as DWORD,
                    coPos,
                    &mut Written,
                );
            }

            coPos.Y += 1;
        }
    }
}
