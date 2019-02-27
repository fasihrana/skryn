use app_units;
use font_kit;
use font_kit::{family_name::FamilyName, font, source::SystemSource};
use gui::properties::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, Arc};
use webrender::api::*;

use util::unicode_compose;

mod shaper{
    use std::collections::HashMap;
    use std::sync::{Mutex, Arc};
    use std::os::raw::{c_char, c_uint, c_int, c_void};
    use std::ptr;
    use webrender::api::GlyphIndex;
    use font_kit::font;
    //harfbuzz functions
    use harfbuzz_sys::{ hb_face_create, hb_font_create, hb_blob_create, hb_buffer_create,
                        hb_face_destroy, hb_font_destroy, hb_blob_destroy, hb_buffer_destroy,
                        hb_font_set_scale,
                        hb_font_set_ppem,
                        hb_buffer_add_utf8,
                        hb_shape,
                        hb_buffer_get_glyph_infos,
                        hb_buffer_get_glyph_positions,
                        hb_buffer_set_direction,
                        hb_buffer_set_script,
                        hb_buffer_set_language,
                        hb_language_from_string};
    //harfbuzz structs
    use harfbuzz_sys::{ hb_face_t,hb_font_t, hb_blob_t, hb_buffer_t,
                        hb_glyph_info_t, hb_glyph_position_t, hb_language_t};
    //harfbuzz consts
    use harfbuzz_sys::{HB_MEMORY_MODE_READONLY, HB_DIRECTION_RTL, HB_SCRIPT_ARABIC, HB_SCRIPT_LATIN, HB_DIRECTION_LTR};

    pub type Dimensions = ((f32, f32), (f32, f32));
    pub type Glyph = (GlyphIndex, (f32, f32));

    pub fn shape_text(val: &str, size: u32, font:font::Font, RTL: bool) -> Vec<Glyph>{
        unsafe {
            let tmp = &*font.copy_font_data().unwrap();
            let blob = hb_blob_create(tmp.as_ptr() as *const c_char,
                                      tmp.len() as c_uint,
                                      HB_MEMORY_MODE_READONLY,
                                      ptr::null_mut() as *mut c_void,
                                      None);

            let face = hb_face_create(blob, 1 as c_uint);

            let hb_font = hb_font_create(face);
            hb_font_set_ppem(hb_font,size,size);
            hb_font_set_scale(hb_font, size as i32, size as i32);

            let buf = hb_buffer_create();
            hb_buffer_add_utf8(buf,
                               val.as_ptr() as *const c_char,
                               val.len() as c_int,
                               0,
                               val.len() as c_int);
            if RTL {
                hb_buffer_set_direction(buf, HB_DIRECTION_RTL);
                hb_buffer_set_script(buf, HB_SCRIPT_ARABIC);
                //let lang = hb_language_from_string("URD".as_ptr() as *const c_char, 3);
                //hb_buffer_set_language(buf,lang);
            } else {
                hb_buffer_set_direction(buf, HB_DIRECTION_LTR);
                //hb_buffer_set_script(buf, HB_SCRIPT_LATIN);
                //let lang = hb_language_from_string("ENG".as_ptr() as *const c_char, 3);
                //hb_buffer_set_language(buf,lang);
            }


            hb_shape(hb_font, buf, ptr::null_mut(), 0);

            let mut g_count = 0;
            let mut p_count = 0;
            let g_info = hb_buffer_get_glyph_infos(buf, &mut g_count);
            let g_pos = hb_buffer_get_glyph_positions(buf, &mut p_count);

            let mut g_vec = Vec::new();

            let mut cursor_x = 0;
            for i in 0..g_count {
                let info = g_info.offset(i as isize);
                let pos = g_pos.offset(i as isize);
                let glyphid = (*info).codepoint;
                let x_advance = (*pos).x_advance;

                g_vec.push((glyphid, (x_advance as f32, size as f32)) );

                cursor_x += x_advance;
            }

            //destroy all
            hb_buffer_destroy(buf);
            hb_font_destroy(hb_font);
            hb_face_destroy(face);
            hb_blob_destroy(blob);

            g_vec
        }
    }
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

#[derive(Debug, Clone)]
pub struct Word{
    text: Vec<char>,
    glyphs: Vec<GlyphInstance>,
    dim: Vec<shaper::Dimensions>,
    RTL: bool,
    extent: Extent,
}

impl Word {
    fn shape(
        &mut self,
        x: f32,
        y: f32,
        _width: f32,
        _height: f32,
        size: f32,
        family: &str,
        //font_store: &mut FontStore,
    ) {

        self.extent.x = x.clone();
        self.extent.y = y.clone();
        self.extent.h = size.clone();

        let mut value : String = self.text.iter().collect();

        let glyphs = shaper::shape_text(value.as_str(), size as u32,load_font_by_name(family), self.RTL);

        self.glyphs.clear();
        self.dim.clear();

        let mut _x = x;

        for (gi, _offset) in glyphs {
            self.glyphs.push(GlyphInstance{ index: gi, point: LayoutPoint::new(_x, y + _offset.1) });
            let tmp_x = _x + _offset.0;
            self.dim.push(((_x,y),(tmp_x,y+size)));
            _x = tmp_x;
        }

        self.extent.w = _x - x;
    }
}

#[derive(Debug, Clone)]
pub struct TextRun{
    words: Vec<Word>,
    RTL: bool
}

impl TextRun{
    pub fn from_string(value: String) -> Vec<TextRun> {
        let mut arr = vec![];

        let mut text_run = TextRun{ words: Vec::new(), RTL: false };

        let lines = value.lines();
        for line in lines {
            let words: Vec<&str> = value.split(' ').collect();
            for word in words {
                let word = word.to_string();
                let word_tmp = word.clone();
                let (ucs, bidi) = unicode_compose(&word_tmp);

                if bidi.paragraphs[0].level.is_rtl() != text_run.RTL {
                    if text_run.words.len() > 0 {
                        arr.push(text_run);
                    }
                    text_run = TextRun{ words: Vec::new(), RTL: bidi.paragraphs[0].level.is_rtl()};
                }

                if bidi.paragraphs[0].level.is_rtl() {
                    text_run.words.insert(0,Word {
                        text: word.chars().collect(),
                        glyphs: vec![],
                        dim: vec![],
                        RTL: true,
                        extent: Extent {
                            x: 0.0,
                            y: 0.0,
                            w: 0.0,
                            h: 0.0,
                            dpi: 0.0,
                        },
                    });
                }
                else {
                    text_run.words.push(Word {
                        text: word.chars().collect(),
                        glyphs: vec![],
                        dim: vec![],
                        RTL: false,
                        extent: Extent {
                            x: 0.0,
                            y: 0.0,
                            w: 0.0,
                            h: 0.0,
                            dpi: 0.0,
                        },
                    });
                }
            }
        }

        arr.push(text_run);

        arr
    }

    fn shape(
        &mut self,
        x: f32,
        y: f32,
        _width: f32,
        _height: f32,
        size: f32,
        family: &str,
    ){
        let mut _x = x;

        for word in self.words.iter_mut() {
            word.shape(_x,y,_width,_height,size,family);
            _x += word.extent.w + (size/4.);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    lines: Vec<TextRun>,
    RTL: bool
}

impl Paragraph {
    pub fn from_string(text: String) -> Vec<Paragraph> {
        let mut arr = vec![];

        for line in text.lines(){
            let text_run = TextRun::from_string(line.to_owned());
            let rtl = if text_run.len() > 0 {
                text_run[0].RTL
            } else {
                false
            };
            let para = Paragraph{ lines: text_run, RTL: rtl };
            arr.push(para);
        }

        arr
    }

    fn shape(
        &mut self,
        x: f32,
        y: f32,
        _width: f32,
        _height: f32,
        size: f32,
        family: &str,
    ){
        let mut _y = y;

        for line in self.lines.iter_mut() {
            line.shape(x,_y,_width,_height,size,family);
            _y += size;
        }
    }
}

pub fn shape_paragraphs(
    paras: &mut Vec<Paragraph>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    size: f32,
    family: &str,
) -> Extent {
    let mut extent = Extent{
        x,
        y,
        w,
        h: size,
        dpi: 0.0,
    };

    if paras.len() > 0 {
        paras[0].shape(x,y,w,h,size,family);
        if paras.len() > 1 {
            for i in 1..paras.len()-1 {
                paras[i].shape(x,y,w,h,size,family);

            }
        }
    }

    extent
}

pub fn glyphs_from_paragraphs(paras: &Vec<Paragraph>) -> Vec<GlyphInstance> {
    let mut arr = vec![];
    for para in paras.iter() {
        for line in para.lines.iter() {
            for word in line.words.iter() {
                arr.append(&mut word.glyphs.clone());
            }
        }
    }

    arr
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