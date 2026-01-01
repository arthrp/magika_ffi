# Magika FFI

A simple FFI wrapper for Google's [Magika](https://securityresearch.google/magika/introduction/overview/) file type detection library.

Magika is an AI-powered file type detection tool that uses a deep learning model to provide accurate detection across 200+ content types.

This project exposes its functionality via C-compatible bindings, allowing it to be used from languages like C, C++, Python (via ctypes), or any other environment that supports C FFI.

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

