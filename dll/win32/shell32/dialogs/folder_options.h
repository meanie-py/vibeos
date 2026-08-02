/*
 *    Folder Options
 *
 * Copyright 2007 Johannes Anderwald <johannes.anderwald@reactos.org>
 * Copyright 2016-2018 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
 */

// IDD_FOLDER_OPTIONS_GENERAL
EXTERN_C INT_PTR
CALLBACK
FolderOptionsGeneralDlg(
    HWND hwndDlg,
    UINT uMsg,
    WPARAM wParam,
    LPARAM lParam);

// IDD_FOLDER_OPTIONS_VIEW
EXTERN_C INT_PTR CALLBACK
FolderOptionsViewDlg(
    HWND    hwndDlg,
    UINT    uMsg,
    WPARAM  wParam,
    LPARAM  lParam);

// IDD_FOLDER_OPTIONS_FILETYPES
EXTERN_C INT_PTR CALLBACK
FolderOptionsFileTypesDlg(
    HWND hwndDlg,
    UINT uMsg,
    WPARAM wParam,
    LPARAM lParam);

/* EXTERN_C: defined in Rust now (shell32-rust/src/dialogs/folder_options.rs), and
   filetypes.cpp calls BitmapFromIcon. Rust cannot emit a C++-mangled name, so the linkage
   has to be C. Plain __cdecl — no WINAPI — so the Rust side declares them `extern "C"`,
   not `extern "system"`; i386 is the only arch whose decoration catches that difference. */
EXTERN_C HBITMAP Create24BppBitmap(HDC hDC, INT cx, INT cy);
EXTERN_C HBITMAP BitmapFromIcon(HICON hIcon, INT cx, INT cy);

/* Also Rust-defined, and filetypes.cpp reads both. These are POINTER-VALUED globals — the
   C consumer loads the variable, not a buffer address — so the Rust side must export a
   `static mut` of pointer type, not the array it points at. */
EXTERN_C LPCWSTR g_pszShell32;
EXTERN_C LPCWSTR g_pszSpace;
