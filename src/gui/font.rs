use app_units;
use font_kit;
use font_kit::{source::SystemSource, font::Font, family_name::FamilyName};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use webrender::api::*;
use gui::properties::Align;

fn load_font_by_name(name: &String) -> Font {
    let mut props = font_kit::properties::Properties::new();

    props.weight = font_kit::properties::Weight::NORMAL;
    props.stretch = font_kit::properties::Stretch::NORMAL;
    props.style = font_kit::properties::Style::Normal;

    let source = SystemSource::new();

    source
        .select_best_match(&[FamilyName::Title(name.clone())], &props)
        .unwrap().load().unwrap()
}

fn add_font(font: &font_kit::font::Font, api: &RenderApi, document_id: DocumentId) -> FontKey {
    //let font: font_kit::font::Font = load_font_by_name(name);

    let f = font.copy_font_data().unwrap();
    let key = api.generate_font_key();

    let mut txn = Transaction::new();
    txn.add_raw_font(key.clone(), (*f).to_owned(), 0);
    api.send_transaction(document_id, txn);

    key
}

struct InstanceKeys {
    key: FontKey,
    font: font_kit::font::Font,
    instances: HashMap<i32, FontInstanceKey>,
}

impl InstanceKeys {
    fn new(_key: FontKey, _font: font_kit::font::Font) -> InstanceKeys {
        InstanceKeys {
            key: _key.clone(),
            font: _font,
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

    pub fn get_font_instance(&mut self, family: &String, size: i32) -> (FontKey, FontInstanceKey) {
        {
            let ikeys = self.store.get_mut(family);
            if let Some(mut _keys) = ikeys {
                let ik = _keys.get_instance_key(size, &(self.api), self.document_id);
                return (_keys.key.clone(), ik.clone());
            }
        }
        {
            let font = load_font_by_name(family);
            let fkey = add_font(&font, &self.api, self.document_id);

            let mut _keys = InstanceKeys::new(fkey, font);
            let _ikey = _keys.get_instance_key(size, &self.api, self.document_id);

            self.store.insert(family.clone(), _keys);

            return (fkey.clone(), _ikey);
        }
    }

    pub fn get_font_metrics(&self, family: &String) -> Option<font_kit::metrics::Metrics>{
        let ikeys = self.store.get(family);
        if let Some(mut _keys) = ikeys {
            Some(_keys.font.metrics())
        } else {
            None
        }
    }

    pub fn get_glyphs_for_set(&self, f_key: FontKey, fi_key: FontInstanceKey, val: &HashSet<char>) -> HashMap<char, (GlyphIndex, GlyphDimensions)> {
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

    pub fn get_glyphs_for_slice(&self, f_key: FontKey, fi_key: FontInstanceKey, s:&str) -> (Vec<Option<u32>>, Vec<Option<GlyphDimensions>>) {
        let gi = self.api.get_glyph_indices(f_key, s);
        let gi_z: Vec<u32> = gi.iter().map(|_gi|{
            match _gi {
                Some(v) => v.clone(),
                _ => 0,
            }
        }).collect();
        let gd = self.api.get_glyph_dimensions(fi_key, gi_z);

        (gi,gd)
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

pub struct FontRaster;

impl FontRaster {

    pub fn place_lines(value: &String,
                       x:f32,
                       y:f32,
                       _width: f32,
                       _height: f32,
                       size: f32,
                       family: &String,
                       text_align: Align,
                       font_store: &mut FontStore){

        let next_x = x;
        let next_y = y;

        for line in value.lines() {
            println!("{:?}",Self::place_line(line,size,family,font_store));
        }
    }

    pub fn place_line(value: &str, size:f32,family:&String,font_store:&mut FontStore) -> (Vec<GlyphInstance>, f32, f32){

        let metrics = font_store.get_font_metrics(family).unwrap();

        let units = size / (metrics.ascent + (metrics.descent*-1.0));

        let (f_key, fi_key) = font_store.get_font_instance(&family, size as i32);

        let (indices, dimens) = font_store.get_glyphs_for_slice(f_key,fi_key, value);

        let mut next_x = 0.0;
        let baseline = units * metrics.ascent;

        let mut glyphs = vec![];

        for i in 0..indices.len() {
            if let Some(gi) = indices[i] {
                glyphs.push(GlyphInstance{ index: gi, point: LayoutPoint::new(next_x, baseline) });
                match dimens[i]{
                    Some(d) => next_x += d.advance,
                    _ => next_x += size/2.0,
                }
            }
        }

        (glyphs, next_x, size)
    }

    pub fn place_glyphs(value: &String,
                    x:f32,
                    y:f32,
                    _width: f32,
                    _height: f32,
                    size: f32,
                    family: &String,
                    text_align: Align,
                    font_store: &mut FontStore) -> (Vec<GlyphInstance>, f32, f32)
    {

        let (f_key, fi_key) = font_store.get_font_instance(&family, size as i32);

        let mut glyphs = vec![];

        let char_set: HashSet<char> = HashSet::from_iter(value.chars());

        let mappings = font_store.get_glyphs_for_set(f_key, fi_key, &char_set);

        let mut text_iter = value.chars();
        let mut next_x = x;
        let mut next_y = y + size;
        let mut max_x = x;

        loop {
            let _char = text_iter.next();
            if _char.is_none() {
                break;
            }
            let _char = _char.unwrap();

            if _char == '\r' || _char == '\n' {
                next_y = next_y + size;
                next_x = x;
                continue;
            }

            if _char == ' ' || _char == '\t' {
                next_x += size/3.0;
                continue;
            }

            let _glyph = mappings.get(&_char);

            if let Some((gi, gd)) = _glyph {
                glyphs.push(GlyphInstance {
                    index: gi.to_owned(),
                    point: LayoutPoint::new(next_x, next_y),
                });

                next_x = next_x + gd.advance;
            }

            if max_x < next_x {
                max_x = next_x;
            }
        }

        (glyphs, max_x, next_y)
    }
}

