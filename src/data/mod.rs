
use std::collections::HashMap;

pub struct Observable<T: Clone>{
    next_id: u64,
    value: T,
    observers: HashMap<u64,Box<FnMut(&T)+Send>>
}

impl <T: Clone> Observable <T> {
    pub fn new(value:T)-> Self{
        Self{
            next_id: 0,
            value,
            observers:HashMap::new(),
        }
    }

    pub fn get_value(&self) -> T {
        self.value.clone()
    }

    pub fn observe(&mut self, observer: Box<FnMut(&T)+Send>) -> u64 {
        self.observers.insert(self.next_id,observer);
        let tmp = self.next_id;
        self.next_id += 1;

        tmp
    }

    pub fn stop(&mut self, id: u64){
        self.observers.remove(&id);
    }

    fn notify_observers(&mut self){
        for o in self.observers.values_mut(){
            o(&self.value);
        }
    }
}

pub enum Action<T: Clone>{
    Add(T),
    Remove(T),
    Update(T),
}

pub trait Update<T: Clone> {
    fn update(&mut self, value:Action<T>);
}

impl <T: Clone> Update<T> for Observable<T>{
    fn update(&mut self, value: Action<T>) {
        match value {
            Action::Update(v) => {
                self.value = v;
                self.notify_observers();
            },
            _ => (),
        }
    }
}

pub type ObservableU32 = Observable<u32>;
pub type ObservableString = Observable<String>;

/*pub type ObservableI8 = Observable<i8>;
pub type ObservableU8 = Observable<u8>;
pub type ObservableI16 = Observable<i16>;
pub type ObservableU16 = Observable<u16>;
pub type ObservableI32 = Observable<i32>;
pub type ObservableU32 = Observable<u32>;
pub type ObservableI64 = Observable<i64>;
pub type ObservableU64 = Observable<u64>;
pub type ObservableBool = Observable<bool>;
pub type ObservableChar = Observable<char>;
pub type ObservableString = Observable<String>;
pub type ObservableISize = Observable<isize>;
pub type ObservableUSize = Observable<usize>;
pub type ObservableF32 = Observable<f32>;
pub type ObservableF64 = Observable<f64>;

pub type ObservableVec<T> = Observable<Vec<T>>;*/