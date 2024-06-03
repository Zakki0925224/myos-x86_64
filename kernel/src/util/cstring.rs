use alloc::{
    slice,
    string::{String, ToString},
};

pub unsafe fn from_cstring_ptr(s_ptr: *const u8) -> String {
    // check len
    let mut len = 0;
    let mut ptr = s_ptr;
    while *ptr != 0 {
        ptr = ptr.offset(1);
        len += 1;
    }

    let s_slice = slice::from_raw_parts(s_ptr, len);
    String::from_utf8_lossy(s_slice).to_string()
}
