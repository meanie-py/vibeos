//! MI_Instance: a class name, a namespace and server, and a list of elements, each an owned
//! value with a name, an MI_Type and flags. Instances made here also carry an MI_ClassDecl
//! (name and one MI_PropertyDecl per element), because MI_Instance exposes `classDecl` as a
//! field and a client may read it without going through the function table.

use crate::sys::*;
use crate::types::*;
use crate::value::*;
use crate::wstr::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ptr::{null, null_mut};

pub struct Elem {
    pub name: WString,
    pub flags: u32,
    pub value: Owned,
}

#[repr(C)]
pub struct Inst {
    pub base: MI_Instance, // first: an *MI_Instance is an *Inst
    pub class_name: WString,
    pub namespace: Option<WString>,
    pub server: Option<WString>,
    pub elems: Vec<Elem>,
    decl: MI_ClassDecl,
    props: Vec<MI_PropertyDecl>,
    prop_ptrs: Vec<*const MI_PropertyDecl>,
}

const INSTANCE_MAGIC: isize = 0x4d49_6e73; // "MIns"

pub unsafe fn as_inst<'a>(i: *const MI_Instance) -> Option<&'a mut Inst> {
    if i.is_null() || (*i).ft != &INSTANCE_FT as *const _ || (*i).reserved[0] != INSTANCE_MAGIC {
        return None;
    }
    Some(&mut *(i as *mut Inst))
}

impl Inst {
    pub fn new(class_name: WString) -> Box<Inst> {
        let mut i = Box::new(Inst {
            base: MI_Instance {
                ft: &INSTANCE_FT,
                classDecl: null(),
                serverName: null(),
                nameSpace: null(),
                reserved: [INSTANCE_MAGIC, 0, 0, 0],
            },
            class_name,
            namespace: None,
            server: None,
            elems: Vec::new(),
            decl: unsafe { core::mem::zeroed() },
            props: Vec::new(),
            prop_ptrs: Vec::new(),
        });
        i.sync();
        i
    }

    pub fn into_mi(self: Box<Self>) -> *mut MI_Instance {
        Box::into_raw(self) as *mut MI_Instance
    }

    /// Re-point the public fields and the class decl at the current contents.
    pub fn sync(&mut self) {
        self.base.nameSpace = self.namespace.as_ref().map_or(null(), |s| s.as_ptr());
        self.base.serverName = self.server.as_ref().map_or(null(), |s| s.as_ptr());
        self.props.clear();
        for e in &self.elems {
            let mut p: MI_PropertyDecl = unsafe { core::mem::zeroed() };
            p.flags = MI_FLAG_PROPERTY | (e.flags & FLAG_KEY);
            p.name = e.name.as_ptr();
            p.type_ = e.value.ty as u32;
            self.props.push(p);
        }
        self.prop_ptrs = self.props.iter().map(|p| p as *const _).collect();
        self.decl.flags = MI_FLAG_CLASS;
        self.decl.name = self.class_name.as_ptr();
        self.decl.properties = self.prop_ptrs.as_ptr();
        self.decl.numProperties = self.prop_ptrs.len() as u32;
        self.decl.size = core::mem::size_of::<MI_Instance>() as u32;
        self.base.classDecl = &self.decl;
    }

    pub fn find(&self, name: *const u16) -> Option<usize> {
        self.elems.iter().position(|e| unsafe { ieq_ptr(&e.name, name) })
    }

    pub fn push(&mut self, name: WString, flags: u32, value: Owned) {
        self.elems.push(Elem { name, flags: flags & !FLAG_NULL & !FLAG_BORROW, value });
        self.sync();
    }

    pub unsafe fn deep_clone(&self) -> Result<Box<Inst>, MI_Result> {
        let mut n = Inst::new(self.class_name.clone());
        n.namespace = self.namespace.clone();
        n.server = self.server.clone();
        for e in &self.elems {
            n.elems.push(Elem { name: e.name.clone(), flags: e.flags, value: e.value.deep_clone()? });
        }
        n.sync();
        Ok(n)
    }
}

unsafe fn get_out(e: &Elem, value: *mut MI_Value, ty: *mut MI_Type, flags: *mut MI_Uint32) {
    if !value.is_null() {
        *value = e.value.val;
    }
    if !ty.is_null() {
        *ty = e.value.ty;
    }
    if !flags.is_null() {
        *flags = e.flags | e.value.null_flag();
    }
}

macro_rules! inst_or {
    ($p:expr) => {
        match as_inst($p) {
            Some(i) => i,
            None => return MI_RESULT_INVALID_PARAMETER,
        }
    };
}

unsafe extern "system" fn i_clone(s: *const MI_Instance, out: *mut *mut MI_Instance) -> MI_Result {
    let i = inst_or!(s);
    if out.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    match i.deep_clone() {
        Ok(n) => {
            *out = n.into_mi();
            MI_RESULT_OK
        }
        Err(e) => e,
    }
}

unsafe extern "system" fn i_destruct(s: *mut MI_Instance) -> MI_Result {
    let i = inst_or!(s);
    i.elems.clear();
    i.sync();
    MI_RESULT_OK
}

unsafe extern "system" fn i_delete(s: *mut MI_Instance) -> MI_Result {
    if as_inst(s).is_none() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    (*s).reserved[0] = 0;
    drop(Box::from_raw(s as *mut Inst));
    MI_RESULT_OK
}

unsafe extern "system" fn i_is_a(s: *const MI_Instance, decl: *const MI_ClassDecl, flag: *mut MI_Boolean) -> MI_Result {
    let i = inst_or!(s);
    if decl.is_null() || flag.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    let mut d = decl;
    let mut yes = false;
    while !d.is_null() {
        if ieq_ptr(&i.class_name, (*d).name) {
            yes = true;
            break;
        }
        d = (*d).superClassDecl;
    }
    *flag = yes as u8;
    MI_RESULT_OK
}

unsafe extern "system" fn i_get_class_name(s: *const MI_Instance, name: *mut *const MI_Char) -> MI_Result {
    let i = inst_or!(s);
    if name.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *name = i.class_name.as_ptr();
    MI_RESULT_OK
}

unsafe extern "system" fn i_set_namespace(s: *mut MI_Instance, ns: *const MI_Char) -> MI_Result {
    let i = inst_or!(s);
    i.namespace = wdup(ns);
    i.sync();
    MI_RESULT_OK
}

unsafe extern "system" fn i_get_namespace(s: *const MI_Instance, ns: *mut *const MI_Char) -> MI_Result {
    let i = inst_or!(s);
    if ns.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *ns = i.namespace.as_ref().map_or(null(), |v| v.as_ptr());
    MI_RESULT_OK
}

unsafe extern "system" fn i_get_element_count(s: *const MI_Instance, count: *mut MI_Uint32) -> MI_Result {
    let i = inst_or!(s);
    if count.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *count = i.elems.len() as u32;
    MI_RESULT_OK
}

unsafe extern "system" fn i_add_element(
    s: *mut MI_Instance,
    name: *const MI_Char,
    value: *const MI_Value,
    ty: MI_Type,
    flags: MI_Uint32,
) -> MI_Result {
    let i = inst_or!(s);
    let Some(n) = wdup(name) else { return MI_RESULT_INVALID_PARAMETER };
    if i.find(name).is_some() {
        return MI_RESULT_ALREADY_EXISTS;
    }
    match Owned::copy_in(ty, value, flags) {
        Ok(v) => {
            i.push(n, flags, v);
            MI_RESULT_OK
        }
        Err(e) => e,
    }
}

unsafe fn set_at(i: &mut Inst, k: usize, value: *const MI_Value, ty: MI_Type, flags: MI_Uint32) -> MI_Result {
    if ty != i.elems[k].value.ty {
        return MI_RESULT_TYPE_MISMATCH;
    }
    match Owned::copy_in(ty, value, flags) {
        Ok(v) => {
            i.elems[k].value = v;
            i.sync();
            MI_RESULT_OK
        }
        Err(e) => e,
    }
}

unsafe extern "system" fn i_set_element(
    s: *mut MI_Instance,
    name: *const MI_Char,
    value: *const MI_Value,
    ty: MI_Type,
    flags: MI_Uint32,
) -> MI_Result {
    let i = inst_or!(s);
    match i.find(name) {
        Some(k) => set_at(i, k, value, ty, flags),
        None => MI_RESULT_NO_SUCH_PROPERTY,
    }
}

unsafe extern "system" fn i_set_element_at(
    s: *mut MI_Instance,
    index: MI_Uint32,
    value: *const MI_Value,
    ty: MI_Type,
    flags: MI_Uint32,
) -> MI_Result {
    let i = inst_or!(s);
    if index as usize >= i.elems.len() {
        return MI_RESULT_NO_SUCH_PROPERTY;
    }
    set_at(i, index as usize, value, ty, flags)
}

unsafe extern "system" fn i_get_element(
    s: *const MI_Instance,
    name: *const MI_Char,
    value: *mut MI_Value,
    ty: *mut MI_Type,
    flags: *mut MI_Uint32,
    index: *mut MI_Uint32,
) -> MI_Result {
    let i = inst_or!(s);
    match i.find(name) {
        Some(k) => {
            get_out(&i.elems[k], value, ty, flags);
            if !index.is_null() {
                *index = k as u32;
            }
            MI_RESULT_OK
        }
        None => MI_RESULT_NO_SUCH_PROPERTY,
    }
}

unsafe extern "system" fn i_get_element_at(
    s: *const MI_Instance,
    index: MI_Uint32,
    name: *mut *const MI_Char,
    value: *mut MI_Value,
    ty: *mut MI_Type,
    flags: *mut MI_Uint32,
) -> MI_Result {
    let i = inst_or!(s);
    let Some(e) = i.elems.get(index as usize) else { return MI_RESULT_NO_SUCH_PROPERTY };
    if !name.is_null() {
        *name = e.name.as_ptr();
    }
    get_out(e, value, ty, flags);
    MI_RESULT_OK
}

unsafe extern "system" fn i_clear_element(s: *mut MI_Instance, name: *const MI_Char) -> MI_Result {
    let i = inst_or!(s);
    match i.find(name) {
        Some(k) => {
            i.elems[k].value = Owned::null(i.elems[k].value.ty);
            i.sync();
            MI_RESULT_OK
        }
        None => MI_RESULT_NO_SUCH_PROPERTY,
    }
}

unsafe extern "system" fn i_clear_element_at(s: *mut MI_Instance, index: MI_Uint32) -> MI_Result {
    let i = inst_or!(s);
    match i.elems.get_mut(index as usize) {
        Some(e) => {
            e.value = Owned::null(e.value.ty);
            MI_RESULT_OK
        }
        None => MI_RESULT_NO_SUCH_PROPERTY,
    }
}

unsafe extern "system" fn i_get_server_name(s: *const MI_Instance, name: *mut *const MI_Char) -> MI_Result {
    let i = inst_or!(s);
    if name.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *name = i.server.as_ref().map_or(null(), |v| v.as_ptr());
    MI_RESULT_OK
}

unsafe extern "system" fn i_set_server_name(s: *mut MI_Instance, name: *const MI_Char) -> MI_Result {
    let i = inst_or!(s);
    i.server = wdup(name);
    i.sync();
    MI_RESULT_OK
}

unsafe extern "system" fn i_get_class(s: *const MI_Instance, out: *mut *mut MI_Class) -> MI_Result {
    let _ = inst_or!(s);
    if !out.is_null() {
        *out = null_mut();
    }
    // An MI_Class needs the schema (qualifiers, methods), which a query result does not carry.
    MI_RESULT_NOT_SUPPORTED
}

pub static INSTANCE_FT: MI_InstanceFT = MI_InstanceFT {
    Clone: Some(i_clone),
    Destruct: Some(i_destruct),
    Delete: Some(i_delete),
    IsA: Some(i_is_a),
    GetClassName: Some(i_get_class_name),
    SetNameSpace: Some(i_set_namespace),
    GetNameSpace: Some(i_get_namespace),
    GetElementCount: Some(i_get_element_count),
    AddElement: Some(i_add_element),
    SetElement: Some(i_set_element),
    SetElementAt: Some(i_set_element_at),
    GetElement: Some(i_get_element),
    GetElementAt: Some(i_get_element_at),
    ClearElement: Some(i_clear_element),
    ClearElementAt: Some(i_clear_element_at),
    GetServerName: Some(i_get_server_name),
    SetServerName: Some(i_set_server_name),
    GetClass: Some(i_get_class),
};

// ---- from WMI ----------------------------------------------------------------------------

unsafe fn get_string(obj: *mut IWbemClassObject, name: &str) -> Option<WString> {
    let n = wa(name);
    let mut v: VARIANT = core::mem::zeroed();
    let hr = com!(obj, Get(n.as_ptr(), 0, &mut v, null_mut(), null_mut()));
    if hr < 0 {
        return None;
    }
    let out = if v.n1.n2.vt == VT_BSTR && !v.n1.n2.n3.bstrVal.is_null() {
        let b = v.n1.n2.n3.bstrVal;
        Some(wfrom(core::slice::from_raw_parts(b, SysStringLen(b) as usize)))
    } else {
        None
    };
    VariantClear(&mut v);
    out
}

unsafe fn is_key(obj: *mut IWbemClassObject, prop: &[u16]) -> bool {
    let mut qs: *mut IWbemQualifierSet = null_mut();
    if com!(obj, GetPropertyQualifierSet(prop.as_ptr(), &mut qs)) < 0 || qs.is_null() {
        return false;
    }
    let qs = ComPtr(qs);
    let key = wa("key");
    let mut v: VARIANT = core::mem::zeroed();
    let hr = com!(qs.0, Get(key.as_ptr(), 0, &mut v, null_mut()));
    let yes = hr >= 0 && !(v.n1.n2.vt == VT_BOOL && v.n1.n2.n3.boolVal == 0);
    VariantClear(&mut v);
    yes
}

/// Convert a WMI object (an instance or an embedded object) into one of our instances.
/// `ns`/`server` are the operation's, used where the object does not say.
pub unsafe fn from_wbem(obj: *mut IWbemClassObject, ns: &[u16], server: &[u16]) -> Result<*mut MI_Instance, MI_Result> {
    let class = get_string(obj, "__CLASS").unwrap_or_else(|| alloc::vec![0]);
    let mut inst = Inst::new(class);
    inst.namespace = Some(get_string(obj, "__NAMESPACE").map(slash_ns).unwrap_or_else(|| wfrom(body(ns))));
    inst.server = Some(get_string(obj, "__SERVER").unwrap_or_else(|| wfrom(body(server))));
    if com!(obj, BeginEnumeration(tag_WBEM_CONDITION_FLAG_TYPE_WBEM_FLAG_NONSYSTEM_ONLY as i32)) < 0 {
        return Err(MI_RESULT_FAILED);
    }
    loop {
        let mut name: BSTR = null_mut();
        let mut v: VARIANT = core::mem::zeroed();
        let mut cim: CIMTYPE = 0;
        let hr = com!(obj, Next(0, &mut name, &mut v, &mut cim, null_mut()));
        if hr != 0 {
            break; // WBEM_S_NO_MORE_DATA, or a failure that ends the walk the same way
        }
        let n = Bstr(name);
        let wname = wfrom(core::slice::from_raw_parts(n.0, SysStringLen(n.0) as usize));
        let r = from_variant(&v, cim, ns, server);
        VariantClear(&mut v);
        let value = r.unwrap_or_else(|_| Owned::null(mi_type_of(cim)));
        let flags = if is_key(obj, &wname) { FLAG_KEY } else { 0 };
        inst.elems.push(Elem { name: wname, flags, value });
    }
    com!(obj, EndEnumeration());
    inst.sync();
    Ok(inst.into_mi())
}

/// WMI spells a namespace `ROOT\cimv2`; MI results use `root/cimv2`.
pub fn slash_ns(mut s: WString) -> WString {
    for c in s.iter_mut() {
        if *c == b'\\' as u16 {
            *c = b'/' as u16;
        }
    }
    s
}

/// A WMI object path (`\\SERVER\ns:Class.Key="v",Other=3`) as a keys-only instance, which is
/// what MI hands out for a reference.
pub unsafe fn from_path(path: &[u16], ns: &[u16], server: &[u16]) -> *mut MI_Instance {
    let mut p = body(path);
    let mut srv = wfrom(body(server));
    let mut nsp = wfrom(body(ns));
    if p.len() > 2 && p[0] == b'\\' as u16 && p[1] == b'\\' as u16 {
        let rest = &p[2..];
        if let Some(k) = rest.iter().position(|&c| c == b'\\' as u16) {
            srv = wfrom(&rest[..k]);
            p = &rest[k + 1..];
        }
    }
    if let Some(k) = p.iter().position(|&c| c == b':' as u16) {
        nsp = slash_ns(wfrom(&p[..k]));
        p = &p[k + 1..];
    }
    let (class, mut keys) = match p.iter().position(|&c| c == b'.' as u16 || c == b'=' as u16) {
        Some(k) => (&p[..k], &p[k + 1..]),
        None => (p, &p[p.len()..]),
    };
    let mut inst = Inst::new(wfrom(class));
    inst.namespace = Some(nsp);
    inst.server = Some(srv);
    while !keys.is_empty() {
        let Some(eq) = keys.iter().position(|&c| c == b'=' as u16) else { break };
        let name = wfrom(&keys[..eq]);
        let mut rest = &keys[eq + 1..];
        let value;
        if rest.first() == Some(&(b'"' as u16)) {
            let mut s = WString::new();
            let mut k = 1;
            while k < rest.len() && rest[k] != b'"' as u16 {
                if rest[k] == b'\\' as u16 && k + 1 < rest.len() {
                    k += 1;
                }
                s.push(rest[k]);
                k += 1;
            }
            s.push(0);
            value = Owned::string(MI_STRING, s);
            rest = &rest[(k + 1).min(rest.len())..];
        } else {
            let end = rest.iter().position(|&c| c == b',' as u16).unwrap_or(rest.len());
            value = match parse_i128(&rest[..end]) {
                Some(n) => {
                    let mut v: MI_Value = core::mem::zeroed();
                    v.sint64 = n as i64;
                    let mut o = Owned::null(MI_SINT64);
                    o.null = false;
                    o.val = v;
                    o
                }
                None => Owned::string(MI_STRING, wfrom(&rest[..end])),
            };
            rest = &rest[end..];
        }
        inst.elems.push(Elem { name, flags: FLAG_KEY, value });
        keys = match rest.first() {
            Some(&c) if c == b',' as u16 => &rest[1..],
            _ => &rest[rest.len()..],
        };
    }
    inst.sync();
    inst.into_mi()
}
