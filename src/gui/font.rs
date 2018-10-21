use euclid;
use gui;
use lazy_static;
use font_kit;
use font_kit::{source::SystemSource, font::Font, family_name::FamilyName};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use webrender::api::*;
use app_units;

lazy_static!(
    static ref FONTDIRECTORY :Arc<Mutex<HashMap<String,Font>>> = Arc::new(Mutex::new(HashMap::new()));
);

fn load_font_by_name(name: &String) -> Font {
    let mut props = font_kit::properties::Properties::new();

    props.weight = font_kit::properties::Weight::NORMAL;
    props.stretch = font_kit::properties::Stretch::NORMAL;
    props.style = font_kit::properties::Style::Normal;

    let source = SystemSource::new();

    source
        .select_best_match(&[FamilyName::Title(name.clone())], &props)
        .unwrap().load().unwrap()

    /*if let Ok(ref mut dict) = FONTDIRECTORY.lock() {
        dict.insert(name.clone(), font);
    }*/
}

fn get_font(name: &String) -> Option<Arc<Vec<u8>>> {
    let mut load_font = false;
    let mut f = {
        if let Ok(dict) = FONTDIRECTORY.lock() {
            if let Some(e) = dict.get(name) {
                e.copy_font_data()
            } else {
                load_font = true;
                None
            }
        } else {
            None
        }
    };

    if load_font {
        load_font_by_name(name);
        f = FONTDIRECTORY.lock().unwrap().get(name).unwrap().copy_font_data();
    }

    f
}


fn add_font(name: &String, api: &RenderApi, document_id: DocumentId) -> FontKey {
    let font = load_font_by_name(name);
    let f = font.copy_font_data().unwrap();//get_font(name).unwrap();
    let key = api.generate_font_key();

    let mut txn = Transaction::new();
    txn.add_raw_font(key.clone(), (*f).to_owned(), 0);
    api.send_transaction(document_id, txn);

    key
}

struct InstanceKeys {
    key: FontKey,
    instances: HashMap<i32, FontInstanceKey>,
}

impl InstanceKeys {
    fn new(_key: FontKey) -> InstanceKeys {
        InstanceKeys {
            key: _key.clone(),
            instances: HashMap::new(),
        }
    }

    fn get_instance_key(&mut self, size: i32, api: &RenderApi, document_id: DocumentId) -> FontInstanceKey {
        let x = self.instances.get(&size);
        if x.is_some() {
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
        api.send_transaction(document_id, txn);

        return ikey;
    }
}

pub struct FontStore {
    store: HashMap<String, InstanceKeys>,
    api: RenderApi,
    document_id: DocumentId,
}

impl FontStore {
    pub fn new(api: RenderApi, document_id: DocumentId) -> FontStore {
        FontStore {
            api,
            document_id,
            store: HashMap::new(),
        }
    }

    pub fn get_font_instance_key(&mut self, family: &String, size: i32) -> (FontKey, FontInstanceKey) {
        {
            let ikeys = self.store.get_mut(family);
            if let Some(mut _keys) = ikeys {
                let ik = _keys.get_instance_key(size, &(self.api), self.document_id);
                return (_keys.key.clone(), ik.clone());
            }
        }
        {
            let fkey = add_font(family, &self.api, self.document_id);

            let mut _keys = InstanceKeys::new(fkey);
            let _ikey = _keys.get_instance_key(size, &self.api, self.document_id);

            self.store.insert(family.clone(), _keys);

            return (fkey.clone(), _ikey);
        }
    }

    pub fn get_glyphs(&self, f_key: FontKey, fi_key: FontInstanceKey, val: &HashSet<char>) -> HashMap<char, (GlyphIndex, GlyphDimensions)> {
        let mut name = "".to_owned();
        for (family, _f) in &self.store {
            if _f.key == f_key {
                name = family.clone();
                break;
            }
        }

        let mut map: HashMap<char, (GlyphIndex, GlyphDimensions)> = HashMap::new();

        let mut str_val = "".to_owned();
        for _c in val.iter() {
            str_val = format!("{}{}", str_val, _c);
        };

        let gi = self.api.get_glyph_indices(f_key, &str_val);
        let gi: Vec<u32> = gi.iter().map(|_gi| {
            match _gi {
                Some(v) => v.clone(),
                _ => 0
            }
        }).collect();
        let gd = self.api.get_glyph_dimensions(fi_key, gi.clone());

        let mut i = 0;
        for c in val.iter().cloned() {
            if let Some(gd) = gd[i] {
                map.insert(c, (gi[i], gd));
            }
            i += 1;
        }

        map
    }

    pub fn deinit(&mut self) {
        let mut txn = Transaction::new();
        for (_, ik) in &self.store {
            let _k = ik.key.clone();
            for (_, _ik) in &ik.instances {
                txn.delete_font_instance(_ik.clone());
            }
            txn.delete_font(_k);
        }
        self.api.send_transaction(self.document_id, txn);
    }
}


