# Magika FFI

A simple FFI wrapper for Google's [Magika](https://securityresearch.google/magika/introduction/overview/) file type detection library.

Magika is an AI-powered file type detection tool that uses a deep learning model to provide accurate detection across 200+ content types.

This project exposes its functionality via C-compatible bindings, allowing it to be used from languages like C, C++, Python (via ctypes), or any other environment that supports C FFI.

## API Reference

The library provides the following functions:

### Session Management

`MagikaSessionHandle* magika_session_new()`

 Creates a new Magika session. Returns `NULL` on failure.

`void magika_session_free(MagikaSessionHandle* handle)`

Frees a Magika session.

### Identification

`const char* magika_identify_path_json(MagikaSessionHandle* handle, const char* path)`

Identifies a file at the given path. Returns a JSON string. Caller owns the returned string and must free it using `magika_string_free`.

`const char* magika_identify_content_json(MagikaSessionHandle* handle, const unsigned char* data, size_t len)`

Identifies a file from a memory buffer. Returns a JSON string. Caller owns the returned string and must free it using `magika_string_free`.

### Memory Management

`void magika_string_free(const char* s)`

Frees a JSON string returned by the identification functions.

You must call this function for every string returned by `magika_identify_path_json` or `magika_identify_content_json` to avoid memory leaks.

## Usage Example (C-like)

```c
// Initialize session
MagikaSessionHandle* session = magika_session_new();

// Identify a file
const char* result_json = magika_identify_path_json(session, "/path/to/example.pdf");

if (result_json != NULL) {
    printf("Result: %s\n", result_json);
    magika_string_free(result_json);
}

// Clean up session
magika_session_free(session);
```

## JSON Schema

The identification functions return a JSON object.

### Success

When `status` is "ok", the `value` field contains the detection results.

```json
{
  "status": "ok",
  "type": "file", 
  "value": {
    "output": {
      "label": "pdf",
      "mime_type": "application/pdf",
      "group": "document",
      "description": "PDF document",
      "extensions": ["pdf"],
      "is_text": false
    },
    "score": 0.99
  }
}
```

*   `status`: "ok"
*   `type`: One of "file", "directory", or "symlink".
*   `value`: Contains the detection details (only present if type is not directory/symlink and detection succeeded).
    *   `output`: The Magika `TypeInfo` object (label, mime_type, etc.).
    *   `score`: The confidence score (0.0 to 1.0).

### Error

```json
{
  "status": "error",
  "message": "Detailed error message description"
}
```

## License

This project is licensed under the [Apache-2.0 License](LICENSE).
