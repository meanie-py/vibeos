use crate::prelude::*;
use core::ffi::c_void;

impl CPointerArray_CUiPrimitive {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_hDpa = DPA_Create(10);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        DPA_DestroyCallback((*this_).m_hDpa, Some(CPointerArray_CUiPrimitive::s_OnRemoveItem), (this_).cast());
    }
    pub unsafe extern "system" fn s_OnRemoveItem(ptr: PVOID, context: PVOID) -> INT {
        let self_: *mut CPointerArray_CUiPrimitive = context as *mut CPointerArray_CUiPrimitive;
        return (*self_).OnRemoveItem(ptr as *mut CUiPrimitive) as INT;
    }
    pub unsafe fn OnRemoveItem__body(&mut self, ptr: *mut CUiPrimitive,
    ) -> BOOL {
        let this_: *mut Self = self;
        (TRUE as i32)
    }
    pub unsafe fn OnCompareItems__body(&mut self, p1: *mut CUiPrimitive, p2: *mut CUiPrimitive) -> INT {
        let this_: *mut Self = self;
        let _ = this_;
        let t: INT_PTR = (p2 as INT_PTR) - (p1 as INT_PTR);
        if t > 0 {
            return 1;
        }
        if t < 0 {
            return -1;
        }
        return 0;
    }
    pub unsafe fn GetCount(&mut self) -> INT {
        let this_: *mut Self = self;
        return DPA_GetPtrCount!((*this_).m_hDpa);
    }
    pub unsafe fn Get(&mut self, i: INT) -> *mut CUiPrimitive {
        let this_: *mut Self = self;
        return DPA_GetPtr((*this_).m_hDpa, (i) as INT_PTR) as *mut CUiPrimitive;
    }
    pub unsafe fn Append(&mut self, ptr: *mut CUiPrimitive) -> INT {
        let this_: *mut Self = self;
            return DPA_InsertPtr((*this_).m_hDpa, (DA_LAST as i32), ptr as *mut core::ffi::c_void);
        }
}

impl CUiRect {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.bottom = 0;
            (*this_).__base0.top = 0;
            (*this_).__base0.right = 0;
            (*this_).__base0.left = 0;
        }
    }
    pub unsafe fn __ctor_1(&mut self, l: INT, t: INT, r: INT, b: INT) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.left = l;
            (*this_).__base0.right = r;
            (*this_).__base0.top = t;
            (*this_).__base0.bottom = b;
        }
    }
}

impl CUiMargin {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        {
        }
    }
    pub unsafe fn __ctor_1(&mut self, all: INT) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_1(all, all, all, all);
        {
        }
    }
    pub unsafe fn __ctor_2(&mut self, horz: INT, vert: INT) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_1(horz, vert, horz, vert);
        {
        }
    }
}

impl CUiMeasure {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).m_Type = CUiMeasure_MeasureType_Type_FitContent;
            (*this_).m_Value = 0;
        }
    }
    pub unsafe fn __ctor_1(&mut self, r#type: MeasureType, value: INT) {
        let this_: *mut Self = self;
        {
            (*this_).m_Type = r#type;
            (*this_).m_Value = value;
        }
    }
    pub unsafe fn ComputeMeasure(&mut self, parent: INT, content: INT) -> INT {
        let this_: *mut Self = self;
        if (*this_).m_Type == CUiMeasure_MeasureType_Type_FitContent {
            return content;
        }
        if (*this_).m_Type == CUiMeasure_MeasureType_Type_Fixed {
            return (*this_).m_Value;
        }
        if (*this_).m_Type == CUiMeasure_MeasureType_Type_Percent {
            return max!(content, parent * (*this_).m_Value / 100);
        }
        if (*this_).m_Type == CUiMeasure_MeasureType_Type_FitParent {
            return parent;
        }

        return 0;
    }
    pub unsafe fn FitContent() -> CUiMeasure {
        return CUiMeasure__make_1(CUiMeasure_MeasureType_Type_FitContent, 0);
    }
    pub unsafe fn FitParent() -> CUiMeasure {
        return CUiMeasure__make_1(CUiMeasure_MeasureType_Type_FitParent, 0);
    }
    pub unsafe fn Fixed(pixels: INT) -> CUiMeasure {
        return CUiMeasure__make_1(CUiMeasure_MeasureType_Type_Fixed, pixels);
    }
    pub unsafe fn Percent(percent: INT) -> CUiMeasure {
        return CUiMeasure__make_1(CUiMeasure_MeasureType_Type_Percent, percent);
    }
}

impl CUiBox {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_Margin.__ctor_0();
        {
            (*this_).m_HorizontalAlignment = UiAlign_LeftTop;
            (*this_).m_VerticalAlignment = UiAlign_LeftTop;
        }
    }
    pub unsafe fn ComputeRect__body(&mut self, mut parentRect: RECT, mut currentRect: RECT, newRect: *mut RECT) {
        let this_: *mut Self = self;
            parentRect.left += (*this_).m_Margin.__base0.__base0.left;
            parentRect.right -= (*this_).m_Margin.__base0.__base0.right;
            parentRect.top += (*this_).m_Margin.__base0.__base0.top;
            parentRect.bottom -= (*this_).m_Margin.__base0.__base0.bottom;

            if parentRect.right < parentRect.left {
                parentRect.right = parentRect.left;
            }

            if parentRect.bottom < parentRect.top {
                parentRect.bottom = parentRect.top;
            }

            let szParent: SIZE = SIZE {
                cx: parentRect.right - parentRect.left,
                cy: parentRect.bottom - parentRect.top,
            };
            let szCurrent: SIZE = SIZE {
                cx: currentRect.right - currentRect.left,
                cy: currentRect.bottom - currentRect.top,
            };

            currentRect = parentRect;

            match (*this_).m_HorizontalAlignment {
                UiAlign_LeftTop => {
                    currentRect.right = currentRect.left + szCurrent.cx;
                }
                UiAlign_Middle => {
                    currentRect.left = parentRect.left + (szParent.cx - szCurrent.cx) / 2;
                    currentRect.right = currentRect.left + szCurrent.cx;
                }
                UiAlign_RightBtm => {
                    currentRect.left = currentRect.right - szCurrent.cx;
                }
                _ => {}
            }

            match (*this_).m_VerticalAlignment {
                UiAlign_LeftTop => {
                    currentRect.bottom = currentRect.top + szCurrent.cy;
                }
                UiAlign_Middle => {
                    currentRect.top = parentRect.top + (szParent.cy - szCurrent.cy) / 2;
                    currentRect.bottom = currentRect.top + szCurrent.cy;
                }
                UiAlign_RightBtm => {
                    currentRect.top = currentRect.bottom - szCurrent.cy;
                }
                _ => {}
            }

            *newRect = currentRect;
        }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
            // Override in subclass
            (*size).cx = max!((*size).cx, 0);
            (*size).cy = min!((*size).cy, 0);
        }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT,
        hDwp: HDWP,
    ) -> HDWP {
        let this_: *mut Self = self;
        (NULL).cast()
    }
}

impl CUiPrimitive {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        (NULL).cast()
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}

impl CUiCollection {
    pub unsafe fn OnRemoveItem__body(&mut self, ptr: *mut CUiPrimitive) -> BOOL {
        let this_: *mut Self = self;
        CUiPrimitive__delete(ptr);
        return (TRUE) as BOOL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl CUiContainer {
    pub unsafe fn Children(&mut self) -> *mut CUiCollection {
        let this_: *mut Self = self;
        return &mut (*this_).m_Children;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_Children.__ctor_0();
    }
}

impl CUiPanel {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_Width.__ctor_0();
        (*this_).m_Height.__ctor_0();
        {
            (*this_).m_Width = CUiMeasure::FitParent();
            (*this_).m_Height = CUiMeasure::FitParent();
        }
    }
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        let mut i: INT = 0;
        while i < (*this_).__base2.m_Children.__base0.GetCount() as INT {
            let box_: *mut CUiBox = (*(*this_).__base2.m_Children.__base0.Get(i)).AsBox();
            if !box_.is_null() {
                (*box_).ComputeMinimalSize(size);
            }
            i += 1;
        }
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut i: INT = 0;
        while i < (*this_).__base2.m_Children.__base0.GetCount() {
            let box_: *mut CUiBox = (*(*this_).__base2.m_Children.__base0.Get(i)).AsBox();
            if !box_.is_null() {
                (*box_).ComputeContentBounds(rect);
            }
            i += 1;
        }
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        let mut count: INT = 0;
        let mut i: INT = 0;
        while i < (*this_).__base2.m_Children.__base0.GetCount() {
            let box_: *mut CUiBox = (*(*this_).__base2.m_Children.__base0.Get(i)).AsBox();
            if !box_.is_null() {
                count = (count as DWORD_PTR).wrapping_add((*box_).CountSizableChildren()) as INT;
            }
            i += 1;
        }
        return count as DWORD_PTR;
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
        let mut hDwp = hDwp;
        let mut rect: RECT = core::mem::zeroed();

        let mut content: SIZE = core::mem::zeroed();
        (*this_).ComputeMinimalSize(core::ptr::addr_of_mut!(content));

        let preferredWidth: INT = (*this_).m_Width.ComputeMeasure(parentRect.right - parentRect.left, content.cx);
        let preferredHeight: INT = (*this_).m_Height.ComputeMeasure(parentRect.bottom - parentRect.top, content.cy);

        rect.right = preferredWidth;
        rect.bottom = preferredHeight;

        (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

        let mut i: INT = 0;
        while i < (*this_).__base2.m_Children.__base0.GetCount() as INT {
            let r#box: *mut CUiBox = (*(*this_).__base2.m_Children.__base0.Get(i)).AsBox();
            if !r#box.is_null() {
                hDwp = (*r#box).OnParentSize(rect, hDwp);
            }
            i += 1;
        }

        return hDwp;
    }
}

impl CUiWindow_CRichEdit {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(r));
        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
        let mut rect: RECT = core::mem::zeroed();

        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rect));

        (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

        if !hDwp.is_null() {
            return DeferWindowPos(
                hDwp, (*this_).__base2.__base0.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER);
        } else {
            (*this_).__base2.__base0.SetWindowPos(
                core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
            return core::ptr::null_mut();
        }
    }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        let _ = Direction;
        TabOrderList.Add((*this_).__base2.__base0.m_hWnd);
        return;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).__base2.__base0.IsWindow() != 0 {
            (*this_).__base2.__base0.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl CUiWindow_CWindowImpl_CAppInfoDisplay {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return (&raw mut (*this_).__base1);
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base2.m_hWnd, &mut r);
        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
            let mut rect: RECT = core::mem::zeroed();

            GetWindowRect((*this_).__base2.m_hWnd, core::ptr::addr_of_mut!(rect));

            (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

            if !hDwp.is_null() {
                return DeferWindowPos(
                    hDwp, (*this_).__base2.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER);
            } else {
                (*this_).__base2.SetWindowPos(
                    core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
                return core::ptr::null_mut();
            }
        }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        let _ = TabOrderList.Add((*this_).__base2.m_hWnd);
        return;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (((*this_).__base2.IsWindow()) != 0) {
            (*this_).__base2.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
    }
}

impl CUiWindow_CWindowImpl_CAppsListView_CListView {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn GetWindow(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).__base2.__base0.m_hWnd;
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
            let mut r: RECT = core::mem::zeroed();
            GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(r));
            (*rect).left = min!((*rect).left, r.left);
            (*rect).top = min!((*rect).top, r.top);
            (*rect).right = max!((*rect).right, r.right);
            (*rect).bottom = max!((*rect).bottom, r.bottom);
        }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
        let mut rect: RECT = core::mem::zeroed();

        GetWindowRect((*this_).__base2.__base0.m_hWnd, &mut rect);

        (*this_).__base1.ComputeRect(parentRect, rect, &mut rect);

        if !hDwp.is_null() {
            return DeferWindowPos(
                hDwp, (*this_).__base2.__base0.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER);
        } else {
            (*this_).__base2.__base0.SetWindowPos(
                core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
            return core::ptr::null_mut();
        }
    }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        TabOrderList.Add((*this_).__base2.__base0.m_hWnd);
        return;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).__base2.__base0.IsWindow() != 0 {
            (*this_).__base2.__base0.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl CUiWindow_CToolbar {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn GetWindow(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).__base2.__base0.m_hWnd;
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(r));
        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
        let mut rect: RECT = core::mem::zeroed();

        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rect));

        (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

        if !hDwp.is_null() {
            return DeferWindowPos(
                hDwp, (*this_).__base2.__base0.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER);
        } else {
            (*this_).__base2.__base0.SetWindowPos(
                core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
            return core::ptr::null_mut();
        }
    }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        TabOrderList.Add((*this_).__base2.__base0.m_hWnd);
        return;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).__base2.__base0.IsWindow() != 0 {
            (*this_).__base2.__base0.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl CUiWindow_CWindowImpl_CApplicationView {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
            let mut r: RECT = core::mem::zeroed();
            GetWindowRect((*this_).__base2.m_hWnd, core::ptr::addr_of_mut!(r));
            (*rect).left = min!((*rect).left, r.left);
            (*rect).top = min!((*rect).top, r.top);
            (*rect).right = max!((*rect).right, r.right);
            (*rect).bottom = max!((*rect).bottom, r.bottom);
        }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
            let mut rect: RECT = core::mem::zeroed();

            GetWindowRect((*this_).__base2.m_hWnd, core::ptr::addr_of_mut!(rect));

            (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

            if !hDwp.is_null()
            {
                return DeferWindowPos(
                    hDwp, (*this_).__base2.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER);
            }
            else
            {
                (*this_).__base2.SetWindowPos(
                    core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
                return core::ptr::null_mut();
            }
        }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        let _ = TabOrderList.Add((*this_).__base2.m_hWnd);
        return;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (((*this_).__base2.IsWindow()) != 0) {
            (*this_).__base2.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
    }
}

impl CUiSplitPanel {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_First.__ctor_0();
        (*this_).m_Second.__ctor_0();
        (*this_).m_Width.__ctor_0();
        (*this_).m_Height.__ctor_0();
        {
            (*this_).m_hCursor = core::ptr::null_mut();
            (*this_).m_Width = CUiMeasure::FitParent();
            (*this_).m_Height = CUiMeasure::FitParent();
            (*this_).m_Pos = 100;
            (*this_).m_Horizontal = (FALSE as i32);
            (*this_).m_MinFirst = 100;
            (*this_).m_MinSecond = 100;
            (*this_).m_DynamicFirst = (FALSE as i32);
            (*this_).m_HasOldRect = (FALSE as i32);
            memset(
                core::ptr::addr_of_mut!((*this_).m_LastRect) as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<RECT>(),
            );
        }
    }
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn First(&mut self) -> &'static mut CUiCollection {
        let this_: *mut Self = self;
        return &mut *((*this_).m_First.__base2.Children());
    }
    pub unsafe fn Second(&mut self) -> &'static mut CUiCollection {
        let this_: *mut Self = self;
        return &mut *((*this_).m_Second.__base2.Children());
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        if (*this_).m_Horizontal != 0 {
            (*size).cx = max!((*size).cx, CUiSplitPanel__THICKNESS);
        } else {
            (*size).cy = max!((*size).cy, CUiSplitPanel__THICKNESS);
        }
        (*this_).m_First.ComputeMinimalSize(size);
        (*this_).m_Second.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();

        (*this_).m_First.ComputeContentBounds(rect);
        (*this_).m_Second.ComputeContentBounds(rect);

        GetWindowRect((*this_).__base2.m_hWnd, core::ptr::addr_of_mut!(r));

        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        let mut count: INT = 1;
        count = count.wrapping_add((*this_).m_First.CountSizableChildren() as INT);
        count = count.wrapping_add((*this_).m_Second.CountSizableChildren() as INT);
        return count as DWORD_PTR;
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, mut hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
            let mut rect: RECT = core::mem::zeroed();

            let mut content: SIZE = core::mem::zeroed();
            (*this_).ComputeMinimalSize(core::ptr::addr_of_mut!(content));

            let preferredWidth: INT = (*this_).m_Width.ComputeMeasure(parentRect.right - parentRect.left, content.cx);
            let preferredHeight: INT = (*this_).m_Width.ComputeMeasure(parentRect.bottom - parentRect.top, content.cy);

            rect.right = preferredWidth;
            rect.bottom = preferredHeight;

            (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

            let mut growth: SIZE = core::mem::zeroed();
            if (*this_).m_HasOldRect != 0 {
                let oldRect: RECT = (*this_).m_LastRect;

                growth.cx = (parentRect.right - parentRect.left) - (oldRect.right - oldRect.left);
                growth.cy = (parentRect.bottom - parentRect.top) - (oldRect.bottom - oldRect.top);
            }

            let mut splitter: RECT = rect;
            let mut first: RECT = rect;
            let mut second: RECT = rect;

            if (*this_).m_Horizontal != 0 {
                rect.top += (*this_).m_MinFirst;
                rect.bottom -= CUiSplitPanel__THICKNESS + (*this_).m_MinSecond;
                if (*this_).m_DynamicFirst != 0 {
                    if growth.cy > 0 {
                        (*this_).m_Pos += min!(growth.cy, rect.bottom - ((*this_).m_Pos + CUiSplitPanel__THICKNESS));
                    } else if growth.cy < 0 {
                        (*this_).m_Pos += max!(growth.cy, rect.top - (*this_).m_Pos);
                    }
                }

                if (*this_).m_Pos > rect.bottom {
                    (*this_).m_Pos = rect.bottom;
                }

                if (*this_).m_Pos < rect.top {
                    (*this_).m_Pos = rect.top;
                }

                splitter.top = (*this_).m_Pos;
                splitter.bottom = (*this_).m_Pos + CUiSplitPanel__THICKNESS;
                first.bottom = splitter.top;
                second.top = splitter.bottom;
            } else {
                rect.left += (*this_).m_MinFirst;
                rect.right -= CUiSplitPanel__THICKNESS + (*this_).m_MinSecond;
                if (*this_).m_DynamicFirst != 0 {
                    if growth.cx > 0 {
                        (*this_).m_Pos += min!(growth.cx, rect.right - ((*this_).m_Pos + CUiSplitPanel__THICKNESS));
                    } else if growth.cx < 0 {
                        (*this_).m_Pos += max!(growth.cy, rect.left - (*this_).m_Pos);
                    }
                }

                if (*this_).m_Pos > rect.right {
                    (*this_).m_Pos = rect.right;
                }

                if (*this_).m_Pos < rect.left {
                    (*this_).m_Pos = rect.left;
                }

                splitter.left = (*this_).m_Pos;
                splitter.right = (*this_).m_Pos + CUiSplitPanel__THICKNESS;
                first.right = splitter.left;
                second.left = splitter.right;
            }

            (*this_).m_LastRect = parentRect;
            (*this_).m_HasOldRect = (TRUE as i32);

            hDwp = (*this_).m_First.OnParentSize(first, hDwp);
            hDwp = (*this_).m_Second.OnParentSize(second, hDwp);

            if !hDwp.is_null() {
                return (*this_).__base2.DeferWindowPos(
                    hDwp, core::ptr::null_mut(), splitter.left, splitter.top, splitter.right - splitter.left, splitter.bottom - splitter.top,
                    SWP_NOACTIVATE | SWP_NOZORDER);
            } else {
                (*this_).__base2.SetWindowPos(
                    core::ptr::null_mut(), splitter.left, splitter.top, splitter.right - splitter.left, splitter.bottom - splitter.top,
                    SWP_NOACTIVATE | SWP_NOZORDER);
                return core::ptr::null_mut();
            }
        }
    pub unsafe fn ProcessWindowMessage(&mut self, hwnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: &mut LRESULT, dwMapId: DWORD) -> BOOL {
        let this_: *mut Self = self;
        *theResult = 0;
        if Msg == WM_SETCURSOR {
            SetCursor((*this_).m_hCursor);
            *theResult = TRUE as LRESULT;
        } else if Msg == WM_LBUTTONDOWN {
            (*this_).__base2.SetCapture();
        } else if Msg == WM_LBUTTONUP || Msg == WM_RBUTTONDOWN {
            if GetCapture() == (*this_).__base2.m_hWnd {
                ReleaseCapture();
            }
        } else if Msg == WM_MOUSEMOVE {
            if GetCapture() == (*this_).__base2.m_hWnd {
                let mut Point: POINT = core::mem::zeroed();
                GetCursorPos((core::ptr::addr_of_mut!(Point)).cast());
                ScreenToClient((*this_).__base2.GetParent().op_to_HWND(), (core::ptr::addr_of_mut!(Point)).cast());
                if (*this_).m_Horizontal != 0 {
                    (*this_).SetPos(Point.y);
                } else {
                    (*this_).SetPos(Point.x);
                }
            }
        } else {
            return (FALSE) as BOOL;
        }

        return (TRUE) as BOOL;
    }
    pub unsafe fn GetPos(&mut self) -> INT {
        let this_: *mut Self = self;
        (*this_).m_Pos
    }
    pub unsafe fn SetPos(&mut self, NewPos: INT) {
        let this_: *mut Self = self;
        let mut rcParent: RECT = (*this_).m_LastRect;

        if (*this_).m_Horizontal != 0 {
            rcParent.bottom -= CUiSplitPanel__THICKNESS;

            (*this_).m_Pos = NewPos;

            if (*this_).m_Pos < rcParent.top {
                (*this_).m_Pos = rcParent.top;
            }

            if (*this_).m_Pos > rcParent.bottom {
                (*this_).m_Pos = rcParent.bottom;
            }
        } else {
            rcParent.right -= CUiSplitPanel__THICKNESS;

            (*this_).m_Pos = NewPos;

            if (*this_).m_Pos < rcParent.left {
                (*this_).m_Pos = rcParent.left;
            }

            if (*this_).m_Pos > rcParent.right {
                (*this_).m_Pos = rcParent.right;
            }
        }

        let count: INT = (*this_).CountSizableChildren() as INT;

        let mut hdwp: HDWP = core::ptr::null_mut();
        hdwp = BeginDeferWindowPos(count);
        if !hdwp.is_null() {
            hdwp = (*this_).OnParentSize((*this_).m_LastRect, hdwp);
        }
        if !hdwp.is_null() {
            EndDeferWindowPos(hdwp);
        }
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        if (*this_).m_Horizontal != 0 {
            (*this_).m_hCursor = LoadCursor(core::ptr::null_mut(), IDC_SIZENS);
        } else {
            (*this_).m_hCursor = LoadCursor(core::ptr::null_mut(), IDC_SIZEWE);
        }

        let style: DWORD = WS_CHILD | WS_VISIBLE;
        let exStyle: DWORD = WS_EX_TRANSPARENT;

        let mut size: RECT = RECT {
            left: 205,
            top: 180,
            right: 465,
            bottom: CUiSplitPanel__THICKNESS,
        };
        size.right += size.left;
        size.bottom += size.top;

        return (*this_).CWindowImpl_Create(hwndParent, &mut size, core::ptr::null_mut(), style, exStyle, 0);
    }
}

impl CUiWindow_CComboBox {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn GetWindow(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).__base2.__base0.m_hWnd;
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base2.__base0.m_hWnd, &mut r);
        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
            let mut rect: RECT = core::mem::zeroed();

            GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rect));

            (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

            if !hDwp.is_null() {
                return DeferWindowPos(
                    hDwp, (*this_).__base2.__base0.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER);
            } else {
                (*this_).__base2.__base0.SetWindowPos(
                    core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
                return core::ptr::null_mut();
            }
        }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
            TabOrderList.Add((*this_).__base2.__base0.m_hWnd);
            return;
        }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).__base2.__base0.IsWindow() != 0 {
            (*this_).__base2.__base0.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl CUiWindow_CSearchBar {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn GetWindow(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).__base2.__base0.m_hWnd;
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(r));
        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
        let mut rect: RECT = core::mem::zeroed();

        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rect));

        (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

        if !hDwp.is_null() {
            return DeferWindowPos(
                hDwp, (*this_).__base2.__base0.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER);
        } else {
            (*this_).__base2.__base0.SetWindowPos(
                core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
            return core::ptr::null_mut();
        }
    }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        TabOrderList.Add((*this_).__base2.__base0.m_hWnd);
        return;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).__base2.__base0.IsWindow() != 0 {
            (*this_).__base2.__base0.DestroyWindow();
        }
    }
    pub unsafe fn GetWindowTextW(&mut self, szText: &mut CStringW) {
        let this_: *mut Self = self;
        let length: INT = (*this_).__base2.__base0.GetWindowTextLengthW() + 1;
        (*this_).__base2.__base0.GetWindowTextW__0(szText.GetBuffer(length), length);
        szText.ReleaseBuffer();
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl CUiWindow_CTreeView {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base2.__base0.m_hWnd, &mut r);
        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
        let mut rect: RECT = core::mem::zeroed();

        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rect));

        (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

        if !hDwp.is_null() {
            return DeferWindowPos(
                hDwp, (*this_).__base2.__base0.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER);
        } else {
            (*this_).__base2.__base0.SetWindowPos(
                core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
            return core::ptr::null_mut();
        }
    }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        let _ = Direction;
        TabOrderList.Add((*this_).__base2.__base0.m_hWnd);
        return;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (((*this_).__base2.__base0.IsWindow()) != 0) {
            (*this_).__base2.__base0.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl CUiWindow_CStatusBar {
    pub unsafe fn AsBox__body(&mut self) -> *mut CUiBox {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn ComputeMinimalSize__body(&mut self, size: *mut SIZE) {
        let this_: *mut Self = self;
        // TODO: Maybe use WM_GETMINMAXINFO?
        return (*this_).__base1.ComputeMinimalSize(size);
    }
    pub unsafe fn ComputeContentBounds__body(&mut self, rect: *mut RECT) {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(r));
        (*rect).left = min!((*rect).left, r.left);
        (*rect).top = min!((*rect).top, r.top);
        (*rect).right = max!((*rect).right, r.right);
        (*rect).bottom = max!((*rect).bottom, r.bottom);
    }
    pub unsafe fn CountSizableChildren__body(&mut self) -> DWORD_PTR {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn OnParentSize__body(&mut self, parentRect: RECT, hDwp: HDWP) -> HDWP {
        let this_: *mut Self = self;
        let mut rect: RECT = core::mem::zeroed();

        GetWindowRect((*this_).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rect));

        (*this_).__base1.ComputeRect(parentRect, rect, core::ptr::addr_of_mut!(rect));

        if !hDwp.is_null() {
            return DeferWindowPos(
                hDwp, (*this_).__base2.__base0.m_hWnd, core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER);
        } else {
            (*this_).__base2.__base0.SetWindowPos(
                core::ptr::null_mut(), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_DEFERERASE);
            return core::ptr::null_mut();
        }
    }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
            let _ = TabOrderList.Add((*this_).__base2.__base0.m_hWnd);
            return;
        }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (((*this_).__base2.__base0.IsWindow()) != 0) {
            (*this_).__base2.__base0.DestroyWindow();
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__base1.__ctor_0();
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
    }
}
