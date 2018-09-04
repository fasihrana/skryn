
extern crate skryn;
extern crate webrender;

use std::sync::{Arc,Mutex};
use std::any::Any;
use std::thread;
use std::time::Duration;

use skryn::data::*;
use skryn::gui::font::FontStore;
use skryn::gui::properties::{Property, Extent, Properties, IdGenerator};
use skryn::elements::{Element, HasChildren, ElementEvent, TextBox, VBox, ScrollBox, HBox, Button, PrimitiveEvent};

use webrender::api::{ColorF, DisplayListBuilder};


struct Person{
    name: ObservableString,
    age: ObservableU32,
}

impl Person{
    fn new(name:String,age:u32) -> Person {
        Person{
            name:ObservableString::new(name),
            age:ObservableU32::new(age),
        }
    }

    fn on_name_change(&mut self,listener: Box<FnMut(&String)+Send>) -> u64{
        self.name.observe(listener)
    }

    fn on_age_change(&mut self,listener: Box<FnMut(&u32)+Send>) -> u64 {
        self.age.observe(listener)
    }

    fn remove_name_listener(&mut self, id: u64){
        self.name.stop(id);
    }

    fn remove_age_listener(&mut self,id: u64){
        self.age.stop(id);
    }
}


struct PersonElm{
    id: u64,
    person: Arc<Mutex<Person>>,
    name_elm: Arc<Mutex<TextBox>>,
    age_elm: Arc<Mutex<TextBox>>,
    vbox: Arc<Mutex<VBox>>,
    bounds: Extent,
}

impl PersonElm{
    fn new(p:Arc<Mutex<Person>>) -> PersonElm{
        let mut _p = p.lock().unwrap();
        let name = Arc::new(Mutex::new( TextBox::new(_p.name.get_value())));
        let age= Arc::new(Mutex::new( TextBox::new(String::from(format!("{}",_p.age.get_value())))));
        let mut v = Arc::new(Mutex::new( VBox::new()));
        match v.lock() {
            Ok(ref mut v) => {
                v.append(name.clone());
                v.append(age.clone());
            },
            Err(_err_str) => panic!("unable to lock element : {}", _err_str)
        }

        let _tmp_age = age.clone();
        _p.age.observe(Box::new(move |v|{
            let _ageelm = _tmp_age.lock().unwrap().set_value(format!("{}",v));
        }));

        PersonElm{
            id:0,
            person: p.clone(),
            name_elm: name,
            age_elm: age,
            vbox: v,
            bounds: Extent{
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            },
        }
    }
}

impl Element for PersonElm {
    fn get_ext_id(&self) -> u64 {
        self.id
    }

    fn set(&mut self, prop: Property) {}

    fn get(&self, prop: &Property) -> Option<&Property> {
        None
    }

    fn render(&mut self, builder: &mut DisplayListBuilder, extent: Extent, font_store: &mut FontStore, props: Option<Arc<Properties>>, gen: &mut IdGenerator) {
        match self.vbox.lock() {
            Ok(ref mut elm) => {
                elm.render(builder,extent,font_store,None, gen);
                self.bounds = elm.get_bounds();
            },
            Err(_err_str) => panic!("unable to lock element : {}",_err_str)
        }
    }

    fn get_bounds(&self) -> Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, ext_ids: &[(u64, u16)], e: PrimitiveEvent) -> bool {
        match self.vbox.lock() {
            Ok(ref mut elm) => {
                return elm.on_primitive_event(ext_ids,e);
            },
            Err(_err_str) => panic!("unable to lock element : {}",_err_str)
        }
        return false;
    }

    fn set_handler(&mut self, _e: ElementEvent, _f: fn(&mut Element, &Any) -> bool) {

    }

    fn get_handler(&mut self, _e: ElementEvent) -> fn(&mut Element, &Any) -> bool {
        skryn::elements::default_fn
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn is_invalid(&self)->bool{
        self.vbox.lock().unwrap().is_invalid()
    }
}


fn main () {

    let mut person = Person::new(String::from("Fasih Rana"), 0);

    let person = Arc::new(Mutex::new(person));
    let tmp_person = person.clone();

    thread::spawn(move ||{
        loop {
            let mut x = tmp_person.lock().unwrap();
            let t = x.age.get_value();
            x.age.update(Action::Update(t + 1));
            thread::sleep(Duration::from_millis(1000));
        }
    });

    /*person.on_age_change(Box::new(|age|{
        println!("age has changed to {}", age);
    }));

    person.age.update(Action::Update(35));*/



    let form = PersonElm::new(person);

    let mut w = skryn::gui::window::Window::new( Box::new(form),String::from("Main window"), 300.0, 200.0);

    w.start();


    /*let mut sbox= ScrollBox::new();
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

    //let mut bclicks = Cell::new(0);

    let mut b1 = Button::new(String::from("I'm a Button. Click me!"));
    b1.set(Property::Size(16));
    b1.set_handler(ElementEvent::Clicked, |_elm, _e|{
        let  x = match _elm.as_any_mut().downcast_mut::<Button>() {
            Some(button) => button,
            None => panic!("not a button!"),
        };
        x.set_value(format!("I have been clicked times"));
        true
    });

    //text boxes begin

    let mut t1 = TextBox::new(String::from("1st box 1st box 1st box 1st box 1st box 1st box"));
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

    let mut t2 = TextBox::new(String::from("2nd box"));
    t2.set(Property::Color(ColorF::new(0.5,0.5,1.0,1.0)));
    d2.append(Box::new(t2));

    //textboxes end

    d1.append(Box::new(b1));
    container.append(Box::new(d1));
    container.append(Box::new(d2));
    sbox.append(Box::new(container));

    let mut w = skryn::gui::window::Window::new( Box::new(sbox),String::from("Main window"), 300.0, 200.0);

    w.start();*/
}