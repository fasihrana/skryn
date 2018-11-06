
#![windows_subsystem = "windows"]
extern crate winit;
extern crate glutin;
pub extern crate webrender;
extern crate gleam;
extern crate app_units;
extern crate euclid;
//extern crate rusttype;
//extern crate font_loader as floader;
extern crate clipboard;
extern crate font_kit;
//#[macro_use] extern crate scan_rules;
#[macro_use] extern crate lazy_static;

pub mod data;
mod util;
pub mod elements;
pub mod gui;
//pub use self::webrender;
