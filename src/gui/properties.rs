use std::collections::{HashSet};
use std::hash::{Hash, Hasher};
use std::mem;

use webrender::api::ColorF;

#[derive(Clone, Debug)]
pub struct Extent{
    pub x:f32,
    pub y:f32,
    pub w:f32,
    pub h:f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Overflow{
    Hidden,
    Wrap,
    Scroll,
}

#[derive(Clone, Debug)]
pub enum Unit{
    Extent,
    Pixel(f32),
    Stretch(f32),
}
impl PartialEq for Unit {
    fn eq(&self, other:&Unit) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}
impl Eq for Unit{}
impl Hash for Unit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state)
    }
}

#[derive(Clone, Debug)]
pub enum Property{
    Size(i32), //in pixels
    Family(String),
    Left(Unit), //in pixels or stretches
    Width(Unit), //in pixels or stretches
    Right(Unit), //in pixels or stretches
    Top(Unit), //in pixels or stretches
    Height(Unit), //in pixels or stretches
    Bottom(Unit), //in pixels or stretches
    Color(ColorF),
    BgColor(ColorF),
    OverflowX(Overflow),
    OverflowY(Overflow),
}

lazy_static!{
    pub static ref SIZE: Property = Property::Size(0);
    pub static ref FAMILY: Property = Property::Family(String::from(""));
    pub static ref LEFT: Property = Property::Left(Unit::Stretch(0.0));
    pub static ref WIDTH: Property = Property::Width(Unit::Stretch(1.0));
    pub static ref RIGHT: Property = Property::Right(Unit::Stretch(0.0));
    pub static ref TOP: Property = Property::Top(Unit::Stretch(0.0));
    pub static ref HEIGHT: Property = Property::Height(Unit::Stretch(1.0));
    pub static ref BOTTOM: Property = Property::Bottom(Unit::Stretch(0.0));
    pub static ref COLOR: Property = Property::Color(ColorF{
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    });
    pub static ref BG_COLOR: Property = Property::BgColor(ColorF{
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    });
    pub static ref OVERFLOW_X: Property = Property::OverflowX(Overflow::Hidden);
    pub static ref OVERFLOW_Y: Property = Property::OverflowY(Overflow::Hidden);
}

impl PartialEq for Property {
    fn eq(& self, other:& Property) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}
impl Eq for Property{}

impl Hash for Property{
    fn hash<H: Hasher>(&self, state:&mut H){
        mem::discriminant(self).hash(state)
    }
}

#[derive(Clone, Debug)]
pub struct Properties(HashSet<Property>);

impl Properties {
    pub fn new() -> Properties{
        Properties(HashSet::new())
    }

    pub fn default(& mut self) -> & mut Properties {
        self.set(Property::Size(12))
            .set(Property::Family(String::from("Arial")))
            .set(Property::Left(Unit::Stretch(0.0)))
            .set(Property::Width(Unit::Stretch(1.0)))
            .set(Property::Right(Unit::Stretch(0.0)))
            .set(Property::Top(Unit::Stretch(0.0)))
            .set(Property::Height(Unit::Stretch(1.0)))
            .set(Property::Bottom(Unit::Stretch(0.0)))
            .set(Property::Color(ColorF::new(0.2,0.2,0.2,1.0)))
            .set(Property::BgColor(ColorF::new(1.0,1.0,1.0,1.0)))
            .set(Property::OverflowX(Overflow::Hidden))
            .set(Property::OverflowY(Overflow::Hidden))
    }

    pub fn set(&mut self, property: Property) -> &mut Properties {
        {
            let x = &mut self.0;
            x.replace(property);
        }
        self
    }

    pub fn get(&self, property: &Property) -> Option<&Property>{
        self.0.get(property)
    }
}
