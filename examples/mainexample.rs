
extern crate skryn;
extern crate webrender;

use skryn::gui::properties::Property;
use skryn::elements::{Element, HasChildren, ElementEvent,TextBox, DivBox};
use std::{thread, time};
use webrender::api::ColorF;

fn main () {
    let mut e= DivBox::new();
    e.set(Property::BgColor(ColorF::new(0.1,0.1,0.1,1.0)));

    let mut t1 = TextBox::new(String::from("i'm a text box\nand\ni am proud of it!"));
    t1.set(Property::Color(ColorF::new(1.0,0.5,0.5,1.0)));
    t1.set_handler(ElementEvent::FocusChange,|_elm, _e|{
        let e = _e.downcast_ref::<bool>().unwrap();
        println!("t1 gained({focus}) or lost({focus}) focus", focus=e);
    });
    e.append(Box::new(t1));

    let mut t2 = TextBox::new(String::from("I'm a text box as well!"));
    t2.set(Property::Color(ColorF::new(0.5,0.5,1.0,1.0)));
    e.append(Box::new(t2));

    e.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("div box clicked");
    });

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