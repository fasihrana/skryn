use app_units;
use font_kit;
use font_kit::{family_name::FamilyName, font, source::SystemSource};
use gui::properties::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, Arc};
use webrender::api::*;

mod harfbuzz{
    use std::collections::HashMap;
    use std::sync::{Mutex, Arc};
    use std::os::raw::{c_char, c_uint, c_int, c_void};
    use std::ptr;
    use font_kit::font;
    //harfbuzz functions
    use harfbuzz_sys::{ hb_face_create, hb_font_create, hb_blob_create,
                        hb_font_set_scale, hb_font_set_ppem};
    //harfbuzz structs
    use harfbuzz_sys::{hb_face_t,hb_font_t, hb_blob_t};
    //harfbuzz consts
    use harfbuzz_sys::{HB_MEMORY_MODE_READONLY};

    lazy_static!(
        static ref HB_FACES: Arc<Mutex<HashMap<String, Arc<Mutex<*mut hb_face_t>>>>> = Arc::new(Mutex::new(HashMap::new()));
        //static ref HB_FONTS: Arc<Mutex<HashMap<u32, hb_font_t>>> = Arc::new(Mutex::new(HashMap::new()));
        static ref HB_COUNTER: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    );

    pub fn create_face(name:&String, size:u32, font:font::Font){
        if !HB_FACES.lock().unwrap().contains_key(name) {
            let tmp = &*font.copy_font_data().unwrap();
            unsafe {
                let blob = hb_blob_create(tmp.as_ptr() as *const c_char,
                                          tmp.len() as c_uint,
                                          HB_MEMORY_MODE_READONLY,
                                          ptr::null_mut() as *mut c_void,
                                          None);
                let ind = {
                    let mut x = HB_COUNTER.lock().unwrap();
                    (*x) += 1;
                    (*x).clone()
                };
                let face = hb_face_create(blob, ind as c_uint);

                let font = hb_font_create(face);

                hb_font_set_ppem(font, size, size);
                hb_font_set_scale(font, size as c_int, size as c_int);

                HB_FACES.lock().unwrap().insert(name.clone(), Arc::new(Mutex::new(face)));
            }
        }
    }

    //pub fn shape_text(val:String, )
}


fn load_font_by_name(name: &str) -> font::Font {
    let mut props = font_kit::properties::Properties::new();

    props.weight = font_kit::properties::Weight::NORMAL;
    props.stretch = font_kit::properties::Stretch::NORMAL;
    props.style = font_kit::properties::Style::Normal;

    let source = SystemSource::new();

    source
        .select_best_match(&[FamilyName::Title(name.into())], &props)
        .unwrap()
        .load()
        .unwrap()
}

fn add_font(font: &font_kit::font::Font, api: &RenderApi, document_id: DocumentId) -> FontKey {
    let f = font.copy_font_data().unwrap();
    let key = api.generate_font_key();

    let mut txn = Transaction::new();
    txn.add_raw_font(key, (*f).to_owned(), 0);
    api.send_transaction(document_id, txn);

    key
}

struct InstanceKeys {
    key: FontKey,
    font: font_kit::font::Font,
    instances: HashMap<i32, FontInstanceKey>,
}

impl InstanceKeys {
    fn new(key: FontKey, font: font_kit::font::Font) -> InstanceKeys {
        InstanceKeys {
            key,
            font,
            instances: HashMap::new(),
        }
    }

    fn get_instance_key(
        &mut self,
        size: i32,
        api: &RenderApi,
        document_id: DocumentId,
    ) -> FontInstanceKey {
        let x = self.instances.get(&size);
        if x.is_some() {
            return *x.unwrap();
        }

        let ikey = api.generate_font_instance_key();

        let mut txn = Transaction::new();
        txn.add_font_instance(
            ikey,
            self.key,
            app_units::Au::from_px(size),
            None,
            None,
            Vec::new(),
        );
        api.send_transaction(document_id, txn);

        ikey
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

    pub fn get_font_instance(&mut self, family: &str, size: i32) -> (FontKey, FontInstanceKey) {
        {
            let ikeys = self.store.get_mut(family);
            if let Some(keys) = ikeys {
                let ik = keys.get_instance_key(size, &(self.api), self.document_id);
                return (keys.key, ik);
            }
        }

        let font = load_font_by_name(family);
        let fkey = add_font(&font, &self.api, self.document_id);

        let mut keys = InstanceKeys::new(fkey, font);
        let ikey = keys.get_instance_key(size, &self.api, self.document_id);

        self.store.insert(family.into(), keys);

        (fkey, ikey)
    }

    pub fn get_font_metrics(&self, family: &str) -> Option<font_kit::metrics::Metrics> {
        let ikeys = self.store.get(family);
        if let Some(keys) = ikeys {
            Some(keys.font.metrics())
        } else {
            None
        }
    }

    pub fn get_glyphs_for_set(
        &self,
        f_key: FontKey,
        fi_key: FontInstanceKey,
        val: &HashSet<char>,
    ) -> HashMap<char, (GlyphIndex, GlyphDimensions)> {
        let mut map: HashMap<char, (GlyphIndex, GlyphDimensions)> = HashMap::new();

        let mut str_val = "".to_owned();
        for _c in val.iter() {
            str_val = format!("{}{}", str_val, _c);
        }

        let gi = self.api.get_glyph_indices(f_key, &str_val);
        let gi: Vec<u32> = gi
            .iter()
            .map(|gi| match gi {
                Some(v) => *v,
                _ => 0,
            })
            .collect();
        let gd = self.api.get_glyph_dimensions(fi_key, gi.clone());

        for (i, c) in val.iter().cloned().enumerate() {
            if let Some(gd) = gd[i] {
                map.insert(c, (gi[i], gd));
            }
        }

        map
    }

    pub fn get_glyphs_for_slice(
        &self,
        f_key: FontKey,
        fi_key: FontInstanceKey,
        s: &str,
    ) -> (Vec<Option<u32>>, Vec<Option<GlyphDimensions>>) {
        let gi = self.api.get_glyph_indices(f_key, s);
        let gi_z: Vec<u32> = gi
            .iter()
            .map(|gi| match gi {
                Some(v) => *v,
                _ => 0,
            })
            .collect();
        let gd = self.api.get_glyph_dimensions(fi_key, gi_z);

        (gi, gd)
    }

    pub fn deinit(&mut self) {
        let mut txn = Transaction::new();
        for ik in self.store.values() {
            for k in ik.instances.values() {
                txn.delete_font_instance(*k);
            }
            txn.delete_font(ik.key);
        }
        self.api.send_transaction(self.document_id, txn);
    }
}

pub struct FontRaster;

pub type Dimensions = ((f32, f32), (f32, f32));
pub type Glyph = (GlyphIndex, (f32, f32), char);

impl FontRaster {
    pub fn place_lines(
        value: &str,
        x: f32,
        y: f32,
        _width: f32,
        _height: f32,
        size: f32,
        family: &str,
        text_align: &Align,
        font_store: &mut FontStore,
    ) -> (Vec<GlyphInstance>, Extent, Vec<Dimensions>) {
        let mut line_glyphs = vec![];
        let mut max_len = 0.0;

        let linefeed_at_end = if !value.is_empty() {
            let tmp: Vec<char> = value.chars().collect();
            tmp[tmp.len() - 1] == '\n' || tmp[tmp.len() - 1] == '\r'
        } else {
            false
        };

        let mut total_lines = 0;
        for line in value.lines() {
            let (t_g, w, h) = Self::get_line_glyphs(line, size, family, font_store);
            if max_len < w {
                max_len = w;
            }
            line_glyphs.push((t_g, w, h));
            total_lines += 1;
        }

        let mut glyphs = vec![];
        let bounds;
        let mut dims = vec![];

        let mut line_index = 0;

        match text_align {
            Align::Left => {
                let (mut _x, mut _y) = (x, y);
                for (l_g, _w, _h) in line_glyphs {
                    if line_index > 0 && line_index < total_lines {
                        dims.push(((_x, _y), (_x, _y + size)));
                    }

                    for (gi, _offset, _char) in l_g {
                        glyphs.push(GlyphInstance {
                            index: gi,
                            point: LayoutPoint::new(_x, _y + _offset.1),
                        });
                        dims.push(((_x, _y), (_x + _offset.0, _y + size)));
                        _x += _offset.0;
                    }
                    _x = x;
                    _y += size;
                    line_index += 1;
                }
                if linefeed_at_end {
                    glyphs.push(GlyphInstance {
                        index: 1,
                        point: LayoutPoint::new(_x, _y),
                    });
                    dims.push(((_x, _y), (_x, _y + size)));
                }
                bounds = Extent {
                    x,
                    y,
                    w: max_len,
                    h: _y,
                    dpi: 0.0,
                };
            }
            Align::Right => {
                let mut _y = y;
                let mut _x = x + _width;
                for (l_g, w, _h) in line_glyphs {
                    _x = x + _width - w;

                    if line_index > 0 && line_index < total_lines {
                        dims.push(((_x, _y), (_x, _y + size)));
                    }

                    for (gi, _offset, _char) in l_g {
                        glyphs.push(GlyphInstance {
                            index: gi,
                            point: LayoutPoint::new(_x, _y + _offset.1),
                        });
                        dims.push(((_x, _y), (_x + _offset.0, _y + size)));
                        _x += _offset.0;
                    }

                    _y += size;
                    //just so if it ends, it has the starting value for next line cursor
                    _x = x + _width;
                    line_index += 1;
                }
                if linefeed_at_end {
                    glyphs.push(GlyphInstance {
                        index: 1,
                        point: LayoutPoint::new(_x, _y),
                    });
                    dims.push(((_x, _y), (_x, _y + size)));
                }
                bounds = Extent {
                    x: x + _width - max_len,
                    y,
                    w: max_len,
                    h: _y,
                    dpi: 0.0,
                };
            }
            Align::Middle => {
                let mut _y = y;
                let mut _x = x + _width;
                for (l_g, w, _h) in line_glyphs {
                    _x = x + (_width - w) / 2.0;

                    if line_index > 0 && line_index < total_lines {
                        dims.push(((_x, _y), (_x, _y + size)));
                    }

                    for (gi, _offset, _char) in l_g {
                        glyphs.push(GlyphInstance {
                            index: gi,
                            point: LayoutPoint::new(_x, _y + _offset.1),
                        });
                        dims.push(((_x, _y), (_x + _offset.0, _y + size)));
                        _x += _offset.0;
                    }

                    _y += size;
                    //just so if it ends, it has the starting value for next line cursor
                    _x = x + _width / 2.0;
                    line_index += 1;
                }
                if linefeed_at_end {
                    glyphs.push(GlyphInstance {
                        index: 1,
                        point: LayoutPoint::new(_x, _y),
                    });
                    dims.push(((_x, _y), (_x, _y + size)));
                }
                bounds = Extent {
                    x: x + (_width - max_len) / 2.0,
                    y,
                    w: max_len,
                    h: _y,
                    dpi: 0.0,
                };
            }
        }

        (glyphs, bounds, dims)
    }

    fn get_line_glyphs(
        value: &str,
        size: f32,
        family: &str,
        font_store: &mut FontStore,
    ) -> (Vec<Glyph>, f32, f32) {
        let val_vec: Vec<char> = value.chars().collect();

        let metrics = font_store.get_font_metrics(family).unwrap();

        let units = size / (metrics.ascent + (metrics.descent * -1.0));

        let (f_key, fi_key) = font_store.get_font_instance(family, size as i32);

        let (indices, dimens) = font_store.get_glyphs_for_slice(f_key, fi_key, value);

        let mut next_x = 0.0;
        let baseline = units * metrics.ascent;

        let mut glyphs = vec![];

        for i in 0..indices.len() {
            if let Some(gi) = indices[i] {
                let mut _offset = (0.0, baseline);
                match dimens[i] {
                    Some(d) => _offset.0 = d.advance, //next_x += d.advance,
                    _ => _offset.0 = size / 2.0,      //next_x += size/2.0,
                }
                next_x += _offset.0;
                glyphs.push((gi, _offset, val_vec[i]));
            }
        }

        (glyphs, next_x, size)
    }

    /*pub fn place_glyphs(value: &String,
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
    }*/
}
