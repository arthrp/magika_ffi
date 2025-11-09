use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar};
use std::ptr;

#[repr(C)]
pub struct MagikaSessionHandle(magika::Session);

#[derive(serde::Serialize)]
struct PredictionJson<'a> {
    status: &'static str,
    output: &'a str,
    score: f32,
    overwrite_reason: &'static str,
    dl: &'a str,
    r#type: &'static str,
}

#[derive(serde::Serialize)]
struct ErrorJson<'a> {
    status: &'static str,
    message: &'a str,
}

fn filetype_to_json(ft: &magika::FileType) -> String {
    use magika::{FileType, OverwriteReason};
    match ft {
        FileType::Directory => serde_json::to_string(&PredictionJson {
            status: "ok",
            output: ft.info().label,
            score: ft.score(),
            overwrite_reason: "none",
            dl: "undefined",
            r#type: "directory",
        })
        .unwrap(),
        FileType::Symlink => serde_json::to_string(&PredictionJson {
            status: "ok",
            output: ft.info().label,
            score: ft.score(),
            overwrite_reason: "none",
            dl: "undefined",
            r#type: "symlink",
        })
        .unwrap(),
        FileType::Ruled(_) => serde_json::to_string(&PredictionJson {
            status: "ok",
            output: ft.info().label,
            score: ft.score(),
            overwrite_reason: "none",
            dl: "undefined",
            r#type: "file",
        })
        .unwrap(),
        FileType::Inferred(inf) => {
            let overwrite_reason = match inf.content_type {
                None => "none",
                Some((_, OverwriteReason::LowConfidence)) => "low-confidence",
                Some((_, OverwriteReason::OverwriteMap)) => "overwrite-map",
            };
            serde_json::to_string(&PredictionJson {
                status: "ok",
                output: ft.info().label,
                score: ft.score(),
                overwrite_reason,
                dl: inf.inferred_type.info().label,
                r#type: "file",
            })
            .unwrap()
        }
    }
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
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    let json = match h.0.identify_file_sync(&path) {
        Ok(ft) => filetype_to_json(&ft),
        Err(e) => serde_json::to_string(&ErrorJson { status: "error", message: &format!("{e}") })
            .unwrap(),
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
        Err(e) => serde_json::to_string(&ErrorJson { status: "error", message: &format!("{e}") })
            .unwrap(),
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

