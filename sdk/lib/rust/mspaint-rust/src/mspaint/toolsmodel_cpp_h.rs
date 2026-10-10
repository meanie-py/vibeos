use crate::prelude::*;
use core::ffi::c_void;






















































impl ToolsModel {
    pub unsafe fn GetOrCreateTool(&mut self, nTool: TOOLTYPE) -> *mut ToolBase {
        let this_: *mut Self = self;
        ToolBase__delete((*this_).m_pToolObject);
        // Source contains a stray token `lObject;` after the delete call; it has no
        // defined meaning in the C and is not translated.
        (*this_).m_pToolObject = createToolObject(nTool);
        return (*this_).m_pToolObject;
    }
    pub unsafe fn SendSetCursor(&mut self) {
        let this_: *mut Self = self;
        canvasWindow.__base0.SendMessageW__0(WM_SETCURSOR, (canvasWindow.operator_HWND()) as WPARAM,
                                 MAKELPARAM!(HTCLIENT, WM_MOUSEMOVE));
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).m_penWidth = 1;
            (*this_).m_lineWidth = (*this_).m_penWidth as i32;
            (*this_).m_brushWidth = 4;
            (*this_).m_shapeStyle = 0;
            (*this_).m_brushStyle = BrushStyleRound;
            (*this_).m_activeTool = TOOL_PEN;
            (*this_).m_oldActiveTool = (*this_).m_activeTool;
            (*this_).m_airBrushRadius = 5;
            (*this_).m_rubberRadius = 4;
            (*this_).m_transpBg = (FALSE) as i32;
            (*this_).m_zoom = (DEFAULT_ZOOM as i32);
            (*this_).m_pToolObject = (*this_).GetOrCreateTool((*this_).m_activeTool);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        ToolBase__delete((*this_).m_pToolObject);
        (*this_).m_pToolObject = core::ptr::null_mut();
    }
    pub unsafe fn IsSelection(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).GetActiveTool() == TOOL_RECTSEL || (*this_).GetActiveTool() == TOOL_FREESEL) as BOOL;
    }
    pub unsafe fn GetLineWidth(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).m_lineWidth
    }
    pub unsafe fn SetLineWidth(&mut self, nLineWidth: i32) {
        let this_: *mut Self = self;
        (*this_).m_lineWidth = nLineWidth;
        (*this_).NotifyToolSettingsChanged();
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn MakeLineThickerOrThinner(&mut self, bThinner: BOOL) {
        let this_: *mut Self = self;
        let thickness: INT = (*this_).GetLineWidth();
        (*this_).SetLineWidth(if bThinner != 0 {
            max(1, thickness - 1)
        } else {
            thickness + 1
        });
    }
    pub unsafe fn GetPenWidth(&mut self) -> INT {
        let this_: *mut Self = self;
        (*this_).m_penWidth
    }
    pub unsafe fn SetPenWidth(&mut self, nPenWidth: INT) {
        let this_: *mut Self = self;
        (*this_).m_penWidth = nPenWidth;
        (*this_).NotifyToolSettingsChanged();
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn MakePenThickerOrThinner(&mut self, bThinner: BOOL) {
        let this_: *mut Self = self;
        let thickness: INT = (*this_).GetPenWidth();
        (*this_).SetPenWidth(if bThinner != 0 {
            max(1, thickness - 1)
        } else {
            thickness + 1
        });
    }
    pub unsafe fn GetShapeStyle(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).m_shapeStyle
    }
    pub unsafe fn SetShapeStyle(&mut self, nShapeStyle: i32) {
        let this_: *mut Self = self;
        (*this_).m_shapeStyle = nShapeStyle;
        (*this_).NotifyToolSettingsChanged();
    }
    pub unsafe fn GetBrushWidth(&mut self) -> INT {
        let this_: *mut Self = self;
        (*this_).m_brushWidth
    }
    pub unsafe fn SetBrushWidth(&mut self, nBrushWidth: INT) {
        let this_: *mut Self = self;
        (*this_).m_brushWidth = nBrushWidth;
        (*this_).NotifyToolSettingsChanged();
        (*this_).SendSetCursor();
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn MakeBrushThickerOrThinner(&mut self, bThinner: BOOL) {
        let this_: *mut Self = self;
        let thickness: INT = (*this_).GetBrushWidth();
        (*this_).SetBrushWidth(if bThinner != 0 {
            max(1, thickness - 1)
        } else {
            thickness + 1
        });
    }
    pub unsafe fn GetBrushStyle(&mut self) -> BrushStyle {
        let this_: *mut Self = self;
        (*this_).m_brushStyle
    }
    pub unsafe fn SetBrushStyle(&mut self, nBrushStyle: BrushStyle) {
        let this_: *mut Self = self;
        (*this_).m_brushStyle = nBrushStyle;
        (*this_).SendSetCursor();
        (*this_).NotifyToolSettingsChanged();
    }
    pub unsafe fn GetActiveTool(&mut self) -> TOOLTYPE {
        let this_: *mut Self = self;
        (*this_).m_activeTool
    }
    pub unsafe fn GetOldActiveTool(&mut self) -> TOOLTYPE {
        let this_: *mut Self = self;
        (*this_).m_oldActiveTool
    }
    pub unsafe fn SetActiveTool(&mut self, nActiveTool: TOOLTYPE) {
        let this_: *mut Self = self;
        (*this_).OnEndDraw((FALSE) as i32);
    
        selectionModel.Landing();
    
        (*this_).m_activeTool = nActiveTool;
    
        match (*this_).m_activeTool {
            TOOL_FREESEL | TOOL_RECTSEL | TOOL_COLOR | TOOL_ZOOM | TOOL_TEXT => {
                // The active tool is not an actually drawing tool
            }
    
            TOOL_LINE | TOOL_BEZIER | TOOL_RECT | TOOL_SHAPE | TOOL_ELLIPSE | TOOL_FILL | TOOL_AIRBRUSH | TOOL_RRECT | TOOL_RUBBER | TOOL_BRUSH | TOOL_PEN => {
                // The active tool is an actually drawing tool. Save it for TOOL_COLOR to restore
                (*this_).m_oldActiveTool = nActiveTool;
            }
    
            _ => {}
        }
    
        (*this_).m_pToolObject = (*this_).GetOrCreateTool((*this_).m_activeTool);
    
        (*this_).NotifyToolChanged();
    }
    pub unsafe fn GetAirBrushRadius(&mut self) -> INT {
        let this_: *mut Self = self;
        (*this_).m_airBrushRadius
    }
    pub unsafe fn SetAirBrushRadius(&mut self, nAirBrushRadius: INT) {
        let this_: *mut Self = self;
        (*this_).m_airBrushRadius = nAirBrushRadius;
        (*this_).NotifyToolSettingsChanged();
    }
    pub unsafe fn MakeAirBrushThickerOrThinner(&mut self, bThinner: BOOL) {
        let this_: *mut Self = self;
        let thickness: INT = (*this_).GetAirBrushRadius();
        (*this_).SetAirBrushRadius(if bThinner != 0 {
            max(1, thickness - 1)
        } else {
            thickness + 1
        });
    }
    pub unsafe fn GetRubberRadius(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).m_rubberRadius
    }
    pub unsafe fn SetRubberRadius(&mut self, nRubberRadius: i32) {
        let this_: *mut Self = self;
        (*this_).m_rubberRadius = nRubberRadius;
        (*this_).SendSetCursor();
        (*this_).NotifyToolSettingsChanged();
    }
    pub unsafe fn MakeRubberThickerOrThinner(&mut self, bThinner: BOOL) {
        let this_: *mut Self = self;
        let mut thickness: INT = (*this_).GetRubberRadius();
        (*this_).SetRubberRadius(if bThinner != 0 {
            max(1, thickness - 1)
        } else {
            thickness + 1
        });
    }
    pub unsafe fn GetToolSize(&mut self) -> SIZE {
        let this_: *mut Self = self;
        let mut size: SIZE = core::mem::zeroed();
        match (*this_).m_activeTool {
            TOOL_FREESEL | TOOL_RECTSEL => {
                size.cx = selectionModel.m_rc.Width();
                size.cy = selectionModel.m_rc.Height();
            }
            TOOL_COLOR | TOOL_ZOOM | TOOL_TEXT | TOOL_FILL => {
                size.cx = 1;
                size.cy = 1;
            }
            TOOL_LINE | TOOL_BEZIER | TOOL_RECT | TOOL_RRECT | TOOL_SHAPE | TOOL_ELLIPSE => {
                size.cx = (*this_).GetLineWidth();
                size.cy = size.cx;
            }
            TOOL_AIRBRUSH => {
                size.cx = (*this_).GetAirBrushRadius() * 2;
                size.cy = size.cx;
            }
            TOOL_RUBBER => {
                size.cx = (*this_).GetRubberRadius() * 2;
                size.cy = size.cx;
            }
            TOOL_BRUSH => {
                size.cx = (*this_).GetBrushWidth();
                size.cy = size.cx;
            }
            TOOL_PEN => {
                size.cx = (*this_).GetPenWidth();
                size.cy = size.cx;
            }
            _ => {}
        }
        if size.cx < 1 {
            size.cx = 1;
        }
        if size.cy < 1 {
            size.cy = 1;
        }
        return size;
    }
    pub unsafe fn IsBackgroundTransparent(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_transpBg
    }
    pub unsafe fn SetBackgroundTransparent(&mut self, bTransparent: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_transpBg = bTransparent;
        (*this_).NotifyToolSettingsChanged();
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn GetZoom(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).m_zoom
    }
    pub unsafe fn SetZoom(&mut self, nZoom: i32) {
        let this_: *mut Self = self;
        (*this_).m_zoom = nZoom;
        (*this_).NotifyZoomChanged();
        (*this_).SendSetCursor();
    }
    pub unsafe fn resetTool(&mut self) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolObject).reset();
    }
    pub unsafe fn selectAll(&mut self) {
        let this_: *mut Self = self;
        (*this_).SetActiveTool(TOOL_RECTSEL);
        (*this_).OnButtonDown((TRUE) as i32, 0, 0, (FALSE) as i32);
        (*this_).OnMouseMove((TRUE) as i32, imageModel.GetWidth(), imageModel.GetHeight());
        (*this_).OnButtonUp((TRUE) as i32, imageModel.GetWidth(), imageModel.GetHeight());
    }
    pub unsafe fn NotifyToolChanged(&mut self) {
        let this_: *mut Self = self;
        if (*core::ptr::addr_of_mut!(toolBoxContainer)).__base0.IsWindow() != 0 {
            (*core::ptr::addr_of_mut!(toolBoxContainer)).__base0.SendMessageW__0(WM_TOOLSMODELTOOLCHANGED, ((*this_).m_activeTool) as UINT, 0);
        }
        if (*core::ptr::addr_of_mut!(toolSettingsWindow)).__base0.IsWindow() != 0 {
            (*core::ptr::addr_of_mut!(toolSettingsWindow)).__base0.SendMessageW__0(WM_TOOLSMODELTOOLCHANGED, ((*this_).m_activeTool) as UINT, 0);
        }
        if (*core::ptr::addr_of_mut!(fontsDialog)).__base0.IsWindow() != 0 {
            (*core::ptr::addr_of_mut!(fontsDialog)).__base0.SendMessageW__0(WM_TOOLSMODELTOOLCHANGED, ((*this_).m_activeTool) as UINT, 0);
        }
        if (*core::ptr::addr_of_mut!(textEditWindow)).__base0.IsWindow() != 0 {
            (*core::ptr::addr_of_mut!(textEditWindow)).__base0.SendMessageW__0(WM_TOOLSMODELTOOLCHANGED, ((*this_).m_activeTool) as UINT, 0);
        }
    }
    pub unsafe fn NotifyToolSettingsChanged(&mut self) {
        let this_: *mut Self = self;
        if toolSettingsWindow.__base0.IsWindow() != 0 {
            toolSettingsWindow.__base0.SendMessageW__0(WM_TOOLSMODELSETTINGSCHANGED, 0, 0);
        }
        if textEditWindow.__base0.IsWindow() != 0 {
            textEditWindow.__base0.SendMessageW__0(WM_TOOLSMODELSETTINGSCHANGED, 0, 0);
        }
    }
    pub unsafe fn NotifyZoomChanged(&mut self) {
        let this_: *mut Self = self;
        if (toolSettingsWindow.__base0.IsWindow()) != 0 {
            toolSettingsWindow.__base0.SendMessageW__0(WM_TOOLSMODELZOOMCHANGED, 0, 0);
        }
        if (textEditWindow.__base0.IsWindow()) != 0 {
            textEditWindow.__base0.SendMessageW__0(WM_TOOLSMODELZOOMCHANGED, 0, 0);
        }
        if (canvasWindow.__base0.IsWindow()) != 0 {
            canvasWindow.__base0.SendMessageW__0(WM_TOOLSMODELZOOMCHANGED, 0, 0);
        }
    }
    pub unsafe fn GetFgBrush(&mut self) -> HBRUSH {
        let this_: *mut Self = self;
        return (*this_).m_fgBrush.GetBrush((*core::ptr::addr_of_mut!(paletteModel)).SelectedPalette(), (*core::ptr::addr_of_mut!(paletteModel)).GetFgColor());
    }
    pub unsafe fn GetBgBrush(&mut self) -> HBRUSH {
        let this_: *mut Self = self;
        return (*this_).m_bgBrush.GetBrush(paletteModel.SelectedPalette(), paletteModel.GetBgColor());
    }
    pub unsafe fn CreateBrush(palette: PAL_TYPE, color: COLORREF) -> HBRUSH {
        if palette == PAL_MONOCHROME {
            return CreateDitherBrush(color, RGB!(0, 0, 0), RGB!(255, 255, 255));
        } else {
            return CreateSolidBrush(color);
        }
    }
}

impl VirtualBrush {
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hBrush.is_null() {
            DeleteObject(((*this_).m_hBrush).cast());
        }
    }
    pub unsafe fn GetBrush(&mut self, palette: PAL_TYPE, rgbColor: COLORREF) -> HBRUSH {
        let this_: *mut Self = self;
        if !((*this_).m_hBrush).is_null()
            && (*this_).m_palette == palette
            && (*this_).m_rgbColor == rgbColor
        {
            return (*this_).m_hBrush;
        }
    
        if !((*this_).m_hBrush).is_null() {
            DeleteObject(((*this_).m_hBrush).cast());
        }
    
        (*this_).m_hBrush = CreateBrush(palette, rgbColor);
        (*this_).m_palette = palette;
        (*this_).m_rgbColor = rgbColor;
        return (*this_).m_hBrush;
    }
}
