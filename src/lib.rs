#![windows_subsystem = "windows"]
extern crate app_units;
extern crate euclid;
extern crate gleam;
extern crate glutin;
pub extern crate webrender;
//pub extern crate webrender_api;
extern crate winit;
//extern crate rusttype;
//extern crate font_loader as floader;
extern crate clipboard;
extern crate font_kit;
//#[macro_use] extern crate scan_rules;
#[macro_use]
extern crate lazy_static;
extern crate itertools;
extern crate unicode_normalization;
extern crate unicode_segmentation;
extern crate unicode_bidi;
extern crate harfbuzz_sys;

mod util;
pub mod data;
pub mod elements;
pub mod gui;