
extern crate skryn;

use std::{thread, time};


fn main () {
    let e = skryn::elements::textbox::TextBox::new(String::from("i'm a text box\nand\ni am proud of it!"));
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