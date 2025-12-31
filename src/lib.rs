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
    status: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<&'a str>,
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
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    struct TestSession(*mut MagikaSessionHandle);
    impl Drop for TestSession {
        fn drop(&mut self) {
            magika_session_free(self.0);
        }
    }

    struct TestString(*const c_char);
    impl Drop for TestString {
        fn drop(&mut self) {
            magika_string_free(self.0);
        }
    }

    #[test]
    fn test_magika_identify_content_json() {
        let session_handle = magika_session_new();
        assert!(!session_handle.is_null());
        let _session = TestSession(session_handle);

        let content =
            b"let content: string;\nconst counter: number = 5;\ncontent = readFile(filePath);";
        let json_ptr =
            magika_identify_content_json(session_handle, content.as_ptr(), content.len());
        assert!(!json_ptr.is_null());
        let _json = TestString(json_ptr);

        let json_str = unsafe { CStr::from_ptr(json_ptr) }.to_str().unwrap();
        let v: Value = serde_json::from_str(json_str).unwrap();

        // Note: these outputs depend on the underlying model
        assert_eq!(v["status"], "ok");
        assert_eq!(v["type"], "file");
        assert_eq!(v["value"]["output"]["label"], "typescript");
        assert_eq!(v["value"]["output"]["mime_type"], "application/typescript");
        assert_eq!(v["value"]["output"]["group"], "code");
        assert_eq!(v["value"]["output"]["description"], "TypeScript source");
        assert_eq!(
            v["value"]["output"]["extensions"],
            serde_json::json!(["ts", "mts", "cts"])
        );
        assert_eq!(v["value"]["output"]["is_text"], true);
        assert!(v["value"]["score"].as_f64().unwrap() > 0.7);
    }
}
