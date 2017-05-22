extern crate libc;

use std::ffi::CStr;
use std::ptr;
use self::libc::{c_char, ssize_t, size_t};

use slp::SlpFile;
use error::ErrorKind;

// Error codes related to input values
const ERR_NULL_PATH: ssize_t = 1;
const ERR_NON_UTF8: ssize_t = 2;

// Error codes related to SLP decoding
const ERR_SLP_INVALID: ssize_t = -1;
const ERR_SLP_BAD_LENGTH: ssize_t = -2;
const ERR_UNKNOWN: ssize_t = -32767;

#[no_mangle]
pub extern "C" fn slp_free(image_data_buff: *const c_char, image_data_buff_len: size_t) {
    if image_data_buff.is_null() || image_data_buff_len == 0 {
        return;
    }

    unsafe {
        Vec::from_raw_parts(image_data_buff as *mut u8, image_data_buff_len, image_data_buff_len);
    }
}

#[no_mangle]
pub extern "C" fn slp_new_from_file(file_path: *const c_char,
   /* ptr to the 1D byte array   */ out_image_data_buff: *mut *const c_char,
   /* ptr to width of the image  */ out_width: *mut size_t,
   /* ptr to height of the image */ out_height: *mut size_t) -> ssize_t {

    unsafe {
        *out_width = 0;
        *out_height = 0;
        *out_image_data_buff = ptr::null_mut();
    }

    let c_str = unsafe {
        if file_path.is_null() {
            return ERR_NULL_PATH;
        }

        CStr::from_ptr(file_path)
    };

    let file_path = match c_str.to_str() {
        Ok(p) => p,
        Err(_) => return ERR_NON_UTF8,
    };

    let mut slp = match SlpFile::read_from_file(file_path, 2) {
        Ok(slp) => slp,
        Err(e) => {
            match *e.kind() {
                ErrorKind::InvalidSlp(_) => return ERR_SLP_INVALID,
                ErrorKind::BadLength => return ERR_SLP_BAD_LENGTH,
                _ => return ERR_UNKNOWN,
            }
        }
    };

    assert!(slp.shapes.len() > 0);
    let first_shape = slp.shapes.swap_remove(0);

    unsafe {
        *out_image_data_buff = first_shape.pixels.as_ptr() as *const c_char;
        *out_height = first_shape.header.height as usize;
        *out_width = first_shape.header.width as usize;
    }

    ::std::mem::forget(first_shape);

    0 as ssize_t
}