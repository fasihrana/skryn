

use std::sync::{Arc,Mutex};
use std::collections::HashMap;

use webrender::api::{FontKey,FontInstanceKey, Transaction, RenderApi, DocumentId};
use floader::system_fonts;
use app_units;
use rusttype;

lazy_static! {
    static ref MAP: Mutex<HashMap<String,(Arc<Vec<u8>>,Arc<rusttype::Font<'static>>)>> = Mutex::new(HashMap::new());
}

fn find_in_map(family : &str) -> Option<(Arc<Vec<u8>>,Arc<rusttype::Font>)>{
    let map = MAP.lock().unwrap();
    let tuple =  map.get(family);
    match tuple {
        None => None,
        Some(val) => {
            Some(val.clone())
        }
    }
}

fn add_to_map(family: &str, bytes: Arc<Vec<u8>>) {
    let mut map = MAP.lock().unwrap();
    let font_coll = rusttype::FontCollection::from_bytes(bytes.to_vec()).unwrap();
    let font_type =  Arc::new(font_coll.into_font().unwrap());
    map.insert(String::from(family),(bytes,font_type));
}

fn get_font(family : &str) -> (Arc<Vec<u8>>,Arc<rusttype::Font>){
    let mut bytes = find_in_map(family);
    if bytes.is_none() {
        let mut fprops = system_fonts::FontPropertyBuilder::new()
            .family(family)
            .build();
        let (font_bytes, _) = system_fonts::get(&mut fprops).unwrap();
        let font_bytes = Arc::new(font_bytes);
        add_to_map(family, font_bytes);
        bytes = find_in_map(family);
    }
    let bytes = bytes.unwrap();
    return bytes.clone();
}

struct InstanceKeys {
    key: FontKey,
    instances: HashMap<i32,FontInstanceKey>
}

impl InstanceKeys{
    fn new(_key:FontKey) -> InstanceKeys {
        InstanceKeys{
            key: _key.clone(),
            instances: HashMap::new(),
        }
    }

    fn get_instance_key(&mut self, size: i32, api: &RenderApi, document_id: DocumentId) -> FontInstanceKey{
        let x = self.instances.get(&size);
        if x.is_some(){
            return x.unwrap().clone();
        }
        let ikey = api.generate_font_instance_key();

        let mut txn = Transaction::new();
        txn.add_font_instance(ikey.clone(),
                              self.key,
                              app_units::Au::from_px(size),
                              None,
                              None,
                              Vec::new());
        api.send_transaction(document_id,txn);
        return ikey;
    }
}

pub struct FontStore{
    store: HashMap<String, InstanceKeys>,
    api: RenderApi,
    document_id: DocumentId,
}

impl FontStore {
    pub fn new (api: RenderApi, document_id: DocumentId) -> FontStore {
        FontStore{
            api,
            document_id,
            store: HashMap::new(),
        }
    }

    pub fn get_font_instance_key(&mut self, family: &String, size: i32) -> FontInstanceKey {
        {
            let keys = self.store.get_mut(family);
            if let Some(mut _keys) = keys {
                let ik = _keys.get_instance_key(size, &(self.api), self.document_id);
                return ik.clone();
            }
        }
        {
            let bytes = get_font(&family).0.clone();

            let fkey = self.api.generate_font_key();
            let mut txn = Transaction::new();
            txn.add_raw_font(fkey, bytes.to_vec(), 0);
            self.api.send_transaction(self.document_id, txn);

            let mut _keys = InstanceKeys::new(fkey);
            let _ikey = _keys.get_instance_key(size, &(self.api), self.document_id);
            self.store.insert(family.clone(), _keys);

            return _ikey;
        }
    }

    pub fn get_font_type<'a>(&'a mut self, family:&'a String) -> Arc<rusttype::Font<'a>>{
        get_font(family).1
    }
}
