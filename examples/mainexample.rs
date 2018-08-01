
extern crate skryn;
extern crate webrender;

use skryn::gui::properties::Property;
use skryn::elements::{Element, HasChildren, ElementEvent,TextBox, DivBox};
use std::{thread, time};
use webrender::api::ColorF;

fn main () {
    let mut container = DivBox::new();
    container.set(Property::BgColor(ColorF::new(1.0,1.0,0.9,1.0)));
    container.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("container clicked");
        false
    });

    let mut d1= DivBox::new();
    d1.set(Property::BgColor(ColorF::new(0.8,0.8,1.0,1.0)));
    d1.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("d1 box clicked");
        false
    });

    let mut d2= DivBox::new();
    d2.set(Property::BgColor(ColorF::new(1.0,0.8,0.8,1.0)));
    d2.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("d2 box clicked");
        false
    });

    /*let mut t1 = TextBox::new(String::from("i'm a text box\nand\ni am proud of it!"));
    t1.set(Property::Color(ColorF::new(1.0,0.5,0.5,1.0)));
    t1.set_handler(ElementEvent::FocusChange,|_elm, _e|{
        let e = _e.downcast_ref::<bool>().unwrap();
        println!("t1 gained focus? {}", e);
        true
    });
    t1.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("t1 clicked");
        true
    });
    d1.append(Box::new(t1));

    let mut t2 = TextBox::new(String::from("I'm a text box as well!"));
    t2.set(Property::Color(ColorF::new(0.5,0.5,1.0,1.0)));
    d2.append(Box::new(t2));

    */

    container.append(Box::new(d1));
    container.append(Box::new(d2));

    let mut w = skryn::gui::window::Window::new(String::from("Main window"), 600.0, 400.0);
    //w.set_root(Box::new(container));

    w.tick(container);

    /*loop {
        if w.tick() {
            break;
        }

        let dur = time::Duration::from_millis(50);

        thread::sleep(dur);
    }
    w.deinit();*/
}