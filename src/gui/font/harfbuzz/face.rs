use std::{mem, ops, ptr, slice};
use std::os::raw::{c_char, c_uint};
use harfbuzz_sys as sys;

use crate::gui::font::harfbuzz::blob::Blob;

pub struct Face {
    raw: *mut sys::hb_face_t,
    blob: Blob,
}

/*impl Face{
    pub fn new() -> Self {

    }
}*/