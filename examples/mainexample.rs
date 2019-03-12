extern crate skryn;
extern crate webrender;

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use skryn::data::*;
use skryn::elements::*;
use skryn::gui::font::FontStore;
use skryn::gui::properties::{Extent, IdGenerator, Properties, Property};

use webrender::api::{ColorF, DisplayListBuilder, RenderApi};

/*
    Lets start with a Person struct. A
    Person has a name and age. Both can change
    so we will use the Observable<T> struct.
    For convenience ObservableString and
    ObservableU32 already exist as types.
*/

struct Person {
    name: ObservableString,
    age: ObservableU32,
}

/*
    The Observables fire events when their value changes.
    So we must have a way for adding event functions for
    both the name and age. Also, we must have functions
    so that we remove the listeners when they are
    not needed.
*/

impl Person {
    fn new(name: String, age: u32) -> Person {
        Person {
            name: ObservableString::new(name),
            age: ObservableU32::new(age),
        }
    }

    #[allow(unused)]
    fn on_name_change(&mut self, listener: Box<FnMut(&String) + Send>) -> u64 {
        self.name.observe(listener)
    }

    fn on_age_change(&mut self, listener: Box<FnMut(&u32) + Send>) -> u64 {
        self.age.observe(listener)
    }

    fn remove_name_listener(&mut self, id: u64) {
        self.name.stop(id);
    }

    fn remove_age_listener(&mut self, id: u64) {
        self.age.stop(id);
    }
}

/*
    Now that we have our Person with two
    Obsevable fields, we need to create an
    Element to encapsulate the view.
*/

struct PersonElm {
    //Every element is given an id. This
    //is used in referencing the main bounding
    //box of Elements
    id: u64,
    //Have a reference to Person. As you'll see
    //below, the age is incremented every second
    //by another thread
    person: Arc<Mutex<Person>>,
    //For ease, we will use a built in Element
    //VBox (Vertical growing box). There is also
    //HBox.
    vbox: Arc<Mutex<VBox>>,
    //Every element must know what area it takes.
    //This is used in the rendering and must be
    //updated if the underlying views change in
    //value or bound
    bounds: Extent,
    //The following ids will be used to remove
    //Observable listeners when the PersonElm
    //is removed
    age_observer_id: Option<u64>,
    name_observer_id: Option<u64>,
}

impl PersonElm {
    fn new(p: Arc<Mutex<Person>>) -> PersonElm {
        //Create two TextBoxes and display their initial value
        let mut _p = p.lock().unwrap();
        let mut _tbox = TextBox::new("سِتار-و-Guitar".to_owned());
        _tbox.set_placeholder("<enter name here>".to_owned());
        let name = Arc::new(Mutex::new( _tbox ));
        let age = Arc::new(Mutex::new(TextBox::new(String::from(format!(
            "{}",
            _p.age.get_value()
        )))));
        //This is an alert button just to show how easy it is to spawn new windows.
        let alert_button = Arc::new(Mutex::new(Button::new(format!("Press here-یہاں-وہاں"))));
        let cancel_button = Arc::new(Mutex::new(Button::new(format!("Fasih فصیح احمد رانا phir kiya kerogay?"))));
        let h = Arc::new(Mutex::new(HBox::new()));
        match h.lock() {
            Ok(ref mut h) => {
                h.set(skryn::gui::properties::Property::Left(
                    skryn::gui::properties::Unit::Stretch(1.0),
                ));
                h.set(skryn::gui::properties::Property::Right(
                    skryn::gui::properties::Unit::Stretch(1.0),
                ));
                h.append(alert_button.clone());
                h.append(cancel_button.clone());
                h.set(skryn::gui::properties::Property::Height(
                    skryn::gui::properties::Unit::Pixel(25.0),
                ));
                h.set(skryn::gui::properties::Property::BgColor(ColorF::new(
                    0.2, 1.0, 0.2, 1.0,
                )));
            }
            Err(_err_str) => panic!("unable to lock element : {}", _err_str),
        }

        //skryn has 4 Length Units at the moment.
        //This is to simplify the rendering of boxes
        //  -> Natural means what ever the natural space an element takes based on the components inside it.
        //  -> Extent means the extent (bounding box) of parent element
        //  -> Stretch means use a percentage of the parent's extent (bounding box)
        //  -> Pixel is the static length
        name.lock()
            .unwrap()
            .set(skryn::gui::properties::Property::Height(
                skryn::gui::properties::Unit::Stretch(1.0),
            ));
        age.lock()
            .unwrap()
            .set(skryn::gui::properties::Property::Height(
                skryn::gui::properties::Unit::Stretch(1.0),
            ));
        age.lock().unwrap().set_editable(false);
        alert_button
            .lock()
            .unwrap()
            .set(skryn::gui::properties::Property::Width(
                skryn::gui::properties::Unit::Pixel(100.0),
            ));
        cancel_button
            .lock()
            .unwrap()
            .set(skryn::gui::properties::Property::Width(
                skryn::gui::properties::Unit::Pixel(100.0),
            ));
        //Here we have used the Stretch unit for elements above to make sure our VBox below is utilized to the full.
        let v = Arc::new(Mutex::new(VBox::new()));
        v.lock().unwrap().set(skryn::gui::properties::Property::Top(
            skryn::gui::properties::Unit::Stretch(1.0),
        ));
        v.lock()
            .unwrap()
            .set(skryn::gui::properties::Property::Bottom(
                skryn::gui::properties::Unit::Stretch(1.0),
            ));
        v.lock()
            .unwrap()
            .set(skryn::gui::properties::Property::Left(
                skryn::gui::properties::Unit::Stretch(0.2),
            ));
        v.lock()
            .unwrap()
            .set(skryn::gui::properties::Property::Right(
                skryn::gui::properties::Unit::Stretch(0.2),
            ));
        match v.lock() {
            Ok(ref mut v) => {
                v.append(name.clone());
                v.append(age.clone());
                v.append(h.clone());
                v.set(skryn::gui::properties::Property::BgColor(ColorF::new(
                    1.0, 0.2, 0.2, 1.0,
                )));
            }
            Err(_err_str) => panic!("unable to lock element : {}", _err_str),
        }

        //The following is a simple action taken when our button is clicked.
        //An alert window is created.
        alert_button.lock().unwrap().set_handler(
            ElementEvent::Clicked,
            EventFn::new(Arc::new(Mutex::new(move |_e: &mut Element, _d: &Any| {
                Alert::show("This is an Alert Box".to_owned(), "Alert".to_owned());
                true
            }))),
        );

        // make sure you sae the observer id for age
        // so that we can remove the listener when
        // this Element is no longer required.
        let age_o_id = _p.on_age_change(Box::new(move |v| {
            let _ageelm = age.lock().unwrap().set_value(format!("{}", v));
        }));

        //finally return the constructed element
        PersonElm {
            id: 0,
            person: p.clone(),
            vbox: v,
            bounds: Extent {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            },
            age_observer_id: Some(age_o_id),
            name_observer_id: None,
        }
    }
}

/*
    Implementing Element trait is the minimum requirement
    for creating a custom element
*/
impl Element for PersonElm {
    fn get_ext_id(&self) -> u64 {
        self.id
    }

    fn set(&mut self, _prop: Property) {
        self.vbox.lock().unwrap().set(_prop);
    }

    fn get_properties(&self) -> skryn::gui::properties::Properties {
        self.vbox.lock().unwrap().get_properties()
    }

    fn render(
        &mut self,
        api: &RenderApi,
        builder: &mut DisplayListBuilder,
        extent: Extent,
        font_store: &mut FontStore,
        _props: Option<Arc<Properties>>,
        gen: &mut IdGenerator,
    ) {
        match self.vbox.lock() {
            Ok(ref mut elm) => {
                elm.render(api, builder, extent, font_store, None, gen);
                self.bounds = elm.get_bounds();
            }
            Err(_err_str) => panic!("unable to lock element : {}", _err_str),
        }
    }

    fn get_bounds(&self) -> Extent {
        self.bounds.clone()
    }

    /*
        Simply pass the events to VBox, which is our container in PersonElm.

        The ext_ids, is a trace of the id part of Elements where the event
        is relevant. For example, if you click the button, the ids passed
        will be that of vbox and alert_button.

        There are certain events where ext_ids are empty, but passing the
        event to the children is still required for e.g., SetFocus
    */
    fn on_primitive_event(&mut self, ext_ids: &[(u64, u16)], e: PrimitiveEvent) -> bool {
        match self.vbox.lock() {
            Ok(ref mut elm) => {
                return elm.on_primitive_event(ext_ids, e);
            }
            Err(_err_str) => panic!("unable to lock element : {}", _err_str),
        }
    }

    fn set_handler(&mut self, _e: ElementEvent, _f: EventFn) {}

    fn exec_handler(&mut self, _e: ElementEvent, _d: &Any) -> bool {
        false
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

impl Drop for PersonElm {
    fn drop(&mut self) {
        if let Some(id) = self.name_observer_id {
            self.person.lock().unwrap().remove_name_listener(id);
        }
        if let Some(id) = self.age_observer_id {
            self.person.lock().unwrap().remove_age_listener(id);
        }
    }
}

/*
    Simpler implementation.
    Here we just have an Alert builder to show
    an alert message.
*/
struct Alert;
impl Alert {
    fn show(message: String, heading: String) {
        let msg_box = TextBox::new(message);
        skryn::gui::window::Manager::add(Arc::new(Mutex::new(msg_box)), heading, 400.0, 100.0);
    }
}

fn main() {
    //create the person.
    let person = Person::new(String::from(""), 0);

    let person = Arc::new(Mutex::new(person));

    //will move this into its separate thread to update the age every second.
    let tmp_person = person.clone();

    //create an Instance of PersonElm and add it to the window manager.
    let form = PersonElm::new(person);
    skryn::gui::window::Manager::add(
        Arc::new(Mutex::new(form)),
        String::from("Main window"),
        300.0,
        200.0,
    );

    //spawn a worker thread to update the age
    thread::spawn(move || {
        let mut t = 0;
        loop {
            {
                let mut x = tmp_person.lock().unwrap();
                t = t + 1;
                x.age.update(Action::Update(t / 100));
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    //start the window manager at 60 fps
    skryn::gui::window::Manager::start(60);
}
