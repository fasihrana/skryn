#![windows_subsystem = "windows"]
extern crate app_units;
extern crate euclid;
extern crate gleam;
extern crate glutin;
pub extern crate webrender;
extern crate winit;
extern crate clipboard;
extern crate font_kit;
#[macro_use]
extern crate lazy_static;
extern crate unicode_bidi;
extern crate harfbuzz_sys;
extern crate itertools;


mod util;
pub mod data;
pub mod elements;
pub mod gui;