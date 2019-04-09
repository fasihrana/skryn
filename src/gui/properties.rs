use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::mem;
use std::sync::{Arc, Mutex};

use webrender::api::ColorF;
//use webrender::api::DeviceSize

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Extent {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub dpi: f32,
}

impl Extent {
    pub fn new() -> Extent {
        Extent {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            dpi: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Unit {
    Natural,
    Extent,
    Pixel(f32),
    Stretch(f32),
}
impl PartialEq for Unit {
    fn eq(&self, other: &Unit) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}
impl Eq for Unit {}
impl Hash for Unit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Align {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Debug)]
pub enum Property {
    Size(i32), //in pixels
    Family(String),
    Left(Unit),   //in pixels or stretches
    Width(Unit),  //in pixels or stretches
    Right(Unit),  //in pixels or stretches
    Top(Unit),    //in pixels or stretches
    Height(Unit), //in pixels or stretches
    Bottom(Unit), //in pixels or stretches
    MinWidth(Unit),
    MinHeight(Unit),
    Color(ColorF),
    BgColor(ColorF),
    HoverColor(ColorF),
    HoverBgColor(ColorF),
    FocusColor(ColorF),
    FocusBgColor(ColorF),
    ActiveColor(ColorF),
    ActiveBgColor(ColorF),
    DisabledColor(ColorF),
    DisabledBgColor(ColorF),
    TextAlign(Align),
}

lazy_static! {
    pub static ref SIZE: Property = Property::Size(0);
    pub static ref FAMILY: Property = Property::Family(String::from(""));
    pub static ref LEFT: Property = Property::Left(Unit::Stretch(0.0));
    pub static ref WIDTH: Property = Property::Width(Unit::Stretch(1.0));
    pub static ref RIGHT: Property = Property::Right(Unit::Stretch(0.0));
    pub static ref TOP: Property = Property::Top(Unit::Stretch(0.0));
    pub static ref HEIGHT: Property = Property::Height(Unit::Stretch(1.0));
    pub static ref BOTTOM: Property = Property::Bottom(Unit::Stretch(0.0));
    pub static ref MIN_WIDTH: Property = Property::MinWidth(Unit::Pixel(0.0));
    pub static ref MIN_HEIGHT: Property = Property::MinHeight(Unit::Pixel(0.0));
    pub static ref COLOR: Property = Property::Color(ColorF {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    });
    pub static ref BG_COLOR: Property = Property::BgColor(ColorF {
        r: 0.9,
        g: 0.9,
        b: 0.9,
        a: 1.0,
    });
    pub static ref HOVER_COLOR: Property = Property::HoverColor(ColorF {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    });
    pub static ref HOVER_BG_COLOR: Property = Property::HoverBgColor(ColorF {
        r: 0.8,
        g: 0.8,
        b: 0.8,
        a: 1.0,
    });
    pub static ref FOCUS_COLOR: Property = Property::FocusColor(ColorF {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    pub static ref FOCUS_BG_COLOR: Property = Property::FocusBgColor(ColorF {
        r: 0.9,
        g: 0.9,
        b: 0.9,
        a: 1.0,
    });
    pub static ref ACTIVE_COLOR: Property = Property::ActiveColor(ColorF {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    });
    pub static ref ACTIVE_BG_COLOR: Property = Property::ActiveBgColor(ColorF {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    });
    pub static ref DISABLED_COLOR: Property = Property::DisabledColor(ColorF {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    });
    pub static ref DISABLED_BG_COLOR: Property = Property::DisabledBgColor(ColorF {
        r: 0.8,
        g: 0.8,
        b: 0.8,
        a: 1.0,
    });
    pub static ref TEXT_ALIGN: Property = Property::TextAlign(Align::Left);
}

impl PartialEq for Property {
    fn eq(&self, other: &Property) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}
impl Eq for Property {}

impl Hash for Property {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Properties(HashSet<Property>);

impl Properties {
    pub fn new() -> Properties {
        Properties(HashSet::new())
    }

    pub fn default(&mut self) -> &mut Properties {
        #[cfg(windows)]
        {
            self.set(Property::Family(String::from("Arial")));
        }
        #[cfg(not(windows))]
        {
            self.set(Property::Family(String::from("FreeMono")));
        }
        self.set(Property::Size(16))
            .set(Property::Left(Unit::Stretch(0.0)))
            .set(Property::Width(Unit::Stretch(1.0)))
            .set(Property::Right(Unit::Stretch(0.0)))
            .set(Property::Top(Unit::Stretch(0.0)))
            .set(Property::Height(Unit::Stretch(1.0)))
            .set(Property::Bottom(Unit::Stretch(0.0)))
            .set(Property::MinWidth(Unit::Pixel(0.0)))
            .set(Property::MinHeight(Unit::Pixel(0.0)))
            .set(Property::Color(ColorF::new(0.8, 0.8, 0.8, 1.0)))
            .set(Property::BgColor(ColorF::new(1.0, 1.0, 1.0, 0.0)))
            .set(Property::FocusColor(ColorF::new(1.0, 1.0, 1.0, 1.0)))
            .set(Property::FocusBgColor(ColorF::new(0.0, 0.0, 0.0, 0.0)))
            .set(Property::HoverColor(ColorF::new(0.9, 0.9, 0.9, 1.0)))
            .set(Property::HoverBgColor(ColorF::new(0.0, 0.0, 0.0, 0.0)))
            .set(Property::DisabledColor(ColorF::new(0.5, 0.5, 0.5, 1.0)))
            .set(Property::DisabledBgColor(ColorF::new(0.0, 0.0, 0.0, 0.0)))
            .set(Property::TextAlign(Align::Left))
    }

    pub fn set(&mut self, property: Property) -> &mut Properties {
        {
            let x = &mut self.0;
            x.replace(property);
        }
        self
    }

    pub fn get(&self, property: &Property) -> Option<&Property> {
        self.0.get(property)
    }

    pub fn get_size(&self) -> i32 {
        if let Some(Property::Size(x)) = self.get(&SIZE) {
            *x
        } else {
            panic!("Size not found")
        }
    }

    pub fn get_family(&self) -> String {
        if let Some(Property::Family(x)) = self.get(&FAMILY) {
            x.clone()
        } else {
            panic!("Family not found")
        }
    }

    pub fn get_left(&self) -> Unit {
        if let Some(Property::Left(x)) = self.get(&LEFT) {
            x.clone()
        } else {
            panic!("Left not found")
        }
    }

    pub fn get_width(&self) -> Unit {
        if let Some(Property::Width(x)) = self.get(&WIDTH) {
            x.clone()
        } else {
            panic!("Width not found")
        }
    }

    pub fn get_right(&self) -> Unit {
        if let Some(Property::Right(x)) = self.get(&RIGHT) {
            x.clone()
        } else {
            panic!("Right not found")
        }
    }

    pub fn get_top(&self) -> Unit {
        if let Some(Property::Top(x)) = self.get(&TOP) {
            x.clone()
        } else {
            panic!("Top not found")
        }
    }

    pub fn get_height(&self) -> Unit {
        if let Some(Property::Height(x)) = self.get(&HEIGHT) {
            x.clone()
        } else {
            panic!("Height not found")
        }
    }

    pub fn get_bottom(&self) -> Unit {
        if let Some(Property::Bottom(x)) = self.get(&BOTTOM) {
            x.clone()
        } else {
            panic!("Bottom not found")
        }
    }

    pub fn get_min_width(&self) -> Unit {
        if let Some(Property::MinWidth(x)) = self.get(&MIN_WIDTH) {
            x.clone()
        } else {
            panic!("Min Width not found")
        }
    }

    pub fn get_min_height(&self) -> Unit {
        if let Some(Property::MinHeight(x)) = self.get(&MIN_HEIGHT) {
            x.clone()
        } else {
            panic!("Min Height not found")
        }
    }

    pub fn get_color(&self) -> ColorF {
        if let Some(Property::Color(x)) = self.get(&COLOR) {
            *x
        } else {
            panic!("Color not found")
        }
    }

    pub fn get_bg_color(&self) -> ColorF {
        if let Some(Property::BgColor(x)) = self.get(&BG_COLOR) {
            *x
        } else {
            panic!("Background Color not found")
        }
    }

    pub fn get_focus_color(&self) -> ColorF {
        if let Some(Property::FocusColor(x)) = self.get(&FOCUS_COLOR) {
            *x
        } else {
            panic!("Focus Color not found")
        }
    }

    pub fn get_focus_bg_color(&self) -> ColorF {
        if let Some(Property::FocusBgColor(x)) = self.get(&FOCUS_BG_COLOR) {
            *x
        } else {
            panic!("Focus Background Color not found")
        }
    }

    pub fn get_hover_color(&self) -> ColorF {
        if let Some(Property::HoverColor(x)) = self.get(&HOVER_COLOR) {
            *x
        } else {
            panic!("Hover Color not found")
        }
    }

    pub fn get_hover_bg_color(&self) -> ColorF {
        if let Some(Property::HoverBgColor(x)) = self.get(&HOVER_BG_COLOR) {
            *x
        } else {
            panic!("Hover Background Color not found")
        }
    }

    pub fn get_disabled_color(&self) -> ColorF {
        if let Some(Property::DisabledColor(x)) = self.get(&DISABLED_COLOR) {
            *x
        } else {
            panic!("Disabled Color not found")
        }
    }

    pub fn get_disabled_bg_color(&self) -> ColorF {
        if let Some(Property::DisabledBgColor(x)) = self.get(&DISABLED_BG_COLOR) {
            *x
        } else {
            panic!("Disabled Background Color not found")
        }
    }

    pub fn get_text_align(&self) -> Align {
        if let Some(Property::TextAlign(x)) = self.get(&TEXT_ALIGN) {
            x.clone()
        } else {
            panic!("Text Align not found")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Button {
    Left,
    Middle,
    Right,
    Other,
}

#[derive(Clone, Debug)]
pub struct IdGenerator {
    pub next_id: Arc<Mutex<u64>>,
}

impl IdGenerator {
    pub fn new(start: u64) -> Self {
        IdGenerator {
            next_id: Arc::new(Mutex::new(start)),
        }
    }
    pub fn get(&mut self) -> u64 {
        let mut counter = self.next_id.lock().unwrap();
        *counter += 1;
        *counter
    }
    pub fn zero(&mut self) {
        let mut counter = self.next_id.lock().unwrap();
        *counter = 0;
    }
}
