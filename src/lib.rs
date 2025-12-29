use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar};
use std::ptr;

#[repr(C)]
pub struct MagikaSessionHandle(magika::Session);

#[derive(serde::Serialize)]
struct MagikaValue<'a> {
    output: &'a magika::TypeInfo,
    score: f32,
}

#[derive(serde::Serialize)]
struct MagikaResponse<'a> {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<MagikaValue<'a>>,
}

const INTERNAL_ERROR_JSON: &str = r#"{"status":"error","message":"Internal serialization error"}"#;

fn filetype_to_json(ft: &magika::FileType) -> String {
    use magika::FileType;
    let r#type = match ft {
        FileType::Directory => "directory",
        FileType::Symlink => "symlink",
        _ => "file",
    };

    let response = MagikaResponse {
        status: "ok",
        message: None,
        r#type: Some(r#type),
        value: Some(MagikaValue {
            output: ft.info(),
            score: ft.score(),
        }),
    };
    serde_json::to_string(&response).unwrap_or_else(|_| INTERNAL_ERROR_JSON.to_string())
}

fn error_to_json(message: String) -> String {
    let response: MagikaResponse<'_> = MagikaResponse {
        status: "error",
        message: Some(message),
        r#type: None,
        value: None,
    };
    serde_json::to_string(&response).unwrap_or_else(|_| INTERNAL_ERROR_JSON.to_string())
}

unsafe fn cstring_or_null(s: String) -> *const c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn magika_session_new() -> *mut MagikaSessionHandle {
    match magika::Session::new() {
        Ok(session) => Box::into_raw(Box::new(MagikaSessionHandle(session))),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn magika_session_free(handle: *mut MagikaSessionHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn magika_identify_path_json(
    handle: *mut MagikaSessionHandle,
    path: *const c_char,
) -> *const c_char {
    if handle.is_null() || path.is_null() {
        return ptr::null();
    }
    let h = unsafe { &mut *handle };
    let path = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();
    let json = match h.0.identify_file_sync(&path) {
        Ok(ft) => filetype_to_json(&ft),
        Err(e) => error_to_json(format!("{e}")),
    };
    unsafe { cstring_or_null(json) }
}

#[unsafe(no_mangle)]
pub extern "C" fn magika_identify_content_json(
    handle: *mut MagikaSessionHandle,
    data: *const c_uchar,
    len: usize,
) -> *const c_char {
    if handle.is_null() || data.is_null() {
        return ptr::null();
    }
    let h = unsafe { &mut *handle };
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let json = match h.0.identify_content_sync(slice) {
        Ok(ft) => filetype_to_json(&ft),
        Err(e) => error_to_json(format!("{e}")),
    };
    unsafe { cstring_or_null(json) }
}

#[unsafe(no_mangle)]
pub extern "C" fn magika_string_free(s: *const c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s as *mut c_char));
        }
    }
}
