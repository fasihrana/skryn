
extern crate skryn;
extern crate webrender;

use skryn::elements::element::Element;
use std::{thread, time};
use webrender::api::ColorF;

fn main () {
    let mut e = skryn::elements::textbox::TextBox::new(String::from("i'm a text box\nand\ni am proud of it!"));
    e.set(skryn::gui::properties::Property::Color(ColorF::new(1.0,0.5,0.5,1.0)));
    let mut w = skryn::gui::window::Window::new(String::from("Main window"), 600.0, 400.0);
    w.set_root(Box::new(e));

    loop {
        if w.tick() {
            break;
        }

        let dur = time::Duration::from_millis(50);

        thread::sleep(dur);
    }
    w.deinit();
}