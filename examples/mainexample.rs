
extern crate skryn;
extern crate webrender;

use skryn::gui::properties::Property;
use skryn::elements::{Element, HasChildren, ElementEvent, TextBox, VBox, ScrollBox, HBox, Button};

use webrender::api::ColorF;

fn main () {

    let mut sbox= ScrollBox::new();
    sbox.set(Property::BgColor(ColorF::new(0.0,0.5,0.5,1.0)));
    sbox.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("sbox clicked");
        false
    });

    let mut container = HBox::new();
    container.set(Property::BgColor(ColorF::new(1.0,1.0,0.5,1.0)));
    container.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("container clicked");
        false
    });



    let mut d1= VBox::new();
    d1.set(Property::BgColor(ColorF::new(1.0,0.8,0.8,1.0)));
    d1.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("d1 box clicked");
        false
    });

    let mut d2= VBox::new();
    d2.set(Property::BgColor(ColorF::new(0.8,0.8,1.0,1.0)));
    d2.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("d2 box clicked");
        false
    });

    let mut b1 = Button::new(String::from("I'm a Button. Click me!"));
    b1.set(Property::Size(16));
    b1.set_handler(ElementEvent::Clicked, |_elm, _e|{
        println!("button 1 clicked");
        let  x = match _elm.as_any_mut().downcast_mut::<Button>() {
            Some(button) => button,
            None => panic!("not a button!"),
        };
        x.set_value(String::from("I have been clicked!!"));
        true
    });


    //text boxes begin

    let mut t1 = TextBox::new(String::from("i'm a text box\nand\ni am proud of it!"));
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

    //textboxes end


    d1.append(Box::new(b1));
    container.append(Box::new(d1));
    container.append(Box::new(d2));
    sbox.append(Box::new(container));

    let mut w = skryn::gui::window::Window::new( Box::new(sbox),String::from("Main window"), 300.0, 200.0);

    w.start();
}