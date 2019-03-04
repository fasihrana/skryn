use app_units;
use font_kit;
use font_kit::{family_name::FamilyName, font, source::SystemSource};
use gui::properties::*;
use std::collections::HashMap;
use webrender::api::*;

use util::unicode_compose;
use super::properties::Align;
use gui::font::shaper::GlyphMetric;
use itertools::*;

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
                        hb_font_get_glyph_extents,
                        hb_buffer_add_utf8,
                        hb_shape,
                        hb_buffer_get_glyph_infos,
                        hb_buffer_get_glyph_positions,
                        hb_buffer_set_direction,
                        hb_buffer_set_script};
    //harfbuzz structs
    use harfbuzz_sys::{ hb_blob_t, hb_face_t,hb_font_t, hb_glyph_extents_t, hb_tag_t };
    //harfbuzz consts
    use harfbuzz_sys::{HB_MEMORY_MODE_READONLY, HB_DIRECTION_RTL, HB_SCRIPT_ARABIC, HB_DIRECTION_LTR};
    use harfbuzz_sys::hb_font_extents_t;

    pub type Dimensions = ((f32, f32), (f32, f32));
    pub type Glyph = (GlyphIndex, GlyphMetric);

    #[derive(Debug, Clone)]
    pub struct Point{
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, Clone)]
    pub struct GlyphMetric{
        pub advance: Point,
        pub offset: Point,
        pub bearing: Point,
        pub width: f32,
        pub height: f32,
    }

    #[derive(Debug, Clone)]
    struct HB_Font{
        blob: usize,
        face: usize,
        font: usize,
        bytes: Vec<u8>,
    }

    lazy_static!(
            static ref FONT : Arc<Mutex<HashMap<String, HB_Font>>> = Arc::new(Mutex::new(HashMap::new()));
    );


    pub fn shape_text(val: &str, size: u32, family: &str, rtl: bool) -> Vec<Glyph>{
        unsafe {

            let hb_font = {
                let mut font_map = FONT.lock().unwrap();
                if !font_map.contains_key(family) {
                    let font = super::load_font_by_name(family);
                    let font_vec : Vec<u8> = (*(font.copy_font_data().unwrap())).clone();
                    let tmp_len = font_vec.len();
                    let tmp = (&font_vec).as_ptr();

                    //let tmp = (tmp).buffer();
                    let blob = hb_blob_create(tmp as *const c_char,
                                              tmp_len as c_uint,
                                              HB_MEMORY_MODE_READONLY,
                                              ptr::null_mut() as *mut c_void,
                                              None);

                    let face = hb_face_create(blob, 1 as c_uint);

                    let font = hb_font_create(face);

                    let hb_font = HB_Font{
                        blob: blob as *const hb_blob_t as usize,
                        face: face as *const hb_face_t as usize,
                        font: font as *const hb_font_t as usize,
                        bytes: font_vec,
                    };

                    font_map.insert(family.to_owned(), hb_font);
                }

                font_map.get(family).unwrap().clone().font as *const hb_font_t as *mut hb_font_t
            };



            hb_font_set_ppem(hb_font,size,size);
            hb_font_set_scale(hb_font, size as i32, size as i32);

            let buf = hb_buffer_create();
            hb_buffer_add_utf8(buf,
                               val.as_ptr() as *const c_char,
                               val.len() as c_int,
                               0,
                               val.len() as c_int);
            if rtl {
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

            //let mut cursor_x = 0;
            for i in 0..g_count {
                let info = g_info.offset(i as isize);
                let pos = g_pos.offset(i as isize);

                let mut extent = hb_glyph_extents_t{
                    x_bearing: 0,
                    y_bearing: 0,
                    width: 0,
                    height: 0,
                };
                hb_font_get_glyph_extents(hb_font,(*info).codepoint,&mut extent as *mut hb_glyph_extents_t);

                let metric = GlyphMetric{
                    advance: Point{ x: (*pos).x_advance as f32, y: (*pos).y_advance as f32},
                    offset: Point{ x: (*pos).x_offset as f32, y: (*pos).y_offset as f32 },
                    bearing: Point{ x: extent.x_bearing as f32, y: extent.y_bearing as f32 },
                    width: extent.width as f32,
                    height: extent.height as f32,
                };

                let glyphid = (*info).codepoint;

                g_vec.push((glyphid, metric) );
            }

            //destroy all
            hb_buffer_destroy(buf);

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
pub struct Char{
    char: char,
    metric: GlyphMetric,
    index: usize,
    position: shaper::Point,
    rtl: bool,
    glyph: GlyphIndex,
}

impl Char {
    fn new(char: char, index: usize, rtl: bool) -> Char {
        Char{
            char,
            metric: GlyphMetric {
                advance: shaper::Point { x: 0.0, y: 0.0 },
                offset: shaper::Point { x: 0.0, y: 0.0 },
                bearing: shaper::Point { x: 0.0, y: 0.0 },
                width: 0.0,
                height: 0.0
            },
            index,
            position: shaper::Point { x: 0.0, y: 0.0 },
            rtl,
            glyph: 0
        }
    }
}

#[derive(Debug, Clone)]
pub struct Word{
    text: Vec<Char>,
    glyphs: Vec<GlyphInstance>,
    rtl: bool,
    extent: Extent,
}

impl Word {
    fn from_chars(value: &Vec<char>, begin:usize, end:usize, rtl:bool) -> Word
    {
        let mut word = Word{
            text: vec![],
            glyphs: vec![],
            rtl,
            extent: Extent {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0
            }
        };

        let mut i = begin;
        for c in &value[begin..end] {
            if rtl {
                word.text.insert(0,Char::new(c.clone(), i, rtl));
            } else {
                word.text.push(Char::new(c.clone(), i, rtl));
            }
        }

        word
    }

    fn len(&self) -> usize {
        self.text.len()
    }

    fn shape(
        &mut self,
        size: f32,
        family: &str,
    ) {

        let value : String = /*if self.rtl {
            let tmp = self.text.iter().rev();
            let mut val = String::new();
            for x in tmp {
                val.push(x.char.clone());
            }
            val
        } else {*/
            self.text.iter().map(|c|{c.char}).collect()
        //}
        ;



        let glyphs = shaper::shape_text(value.as_str(), size as u32, family, self.rtl);

        self.glyphs.clear();

        let mut _x = 0.;

        let mut i = 0;
        while i < self.text.len() {
            let (glyph, ref metric) = glyphs[i];

            self.text[i].glyph = glyph;
            self.text[i].metric = metric.clone();
            self.text[i].position.x = _x;
            self.text[i].position.y = size;

            i+=1;
            _x += metric.advance.x;
        }

        self.extent.h = size;
        self.extent.w = _x;
    }

    fn position(&mut self, x: f32, y: f32) {
        self.glyphs.clear();
        for ch in self.text.iter_mut() {
            ch.position.x += x;
            ch.position.y += y;

            self.glyphs.push(GlyphInstance{ index: ch.glyph, point: LayoutPoint::new(ch.position.x, ch.position.y) });
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextRun{
    words: Vec<Word>,
    extent: Extent,
    rtl: bool
}

impl TextRun{
    fn shape(
        &mut self,
        size: f32,
        family: &str,
    ){
        for word in self.words.iter_mut() {
            word.shape(size,family);
        }
    }

    fn position (&mut self, line_x: f32, x:f32, y:f32, w:f32, h:f32, size: f32, text_align:&Align) {
        let mut _x = x;
        let mut _y = y;
        let mut _w = 0.;
        let mut max_w = 0.;
        for word in self.words.iter_mut() {


            println!{"{}, {} ---> {:?}", _x, _y, word};

            if _x + word.extent.w > (x+w) {
                if max_w < _w {
                    max_w = _w;
                }
                _x = line_x;
                _y += size;
                _w = 0.;
            }
            word.position(_x,_y);
            _w += word.extent.w;

            _x += word.extent.w + size/4.;
        }

        self.extent.w = max_w;
        self.extent.h = _y - y;
    }

    fn len(&self) -> usize {
        let mut len = 0;
        for word in self.words.iter() {
            len += word.len() + 1; //Add 1 for a space character
        }

        len - 1 //TextRun character length is (Sum of word(1..n).len) + n - 1
    }

    fn append(arr: &mut Vec<TextRun>, word: Word){
        if arr.len() == 0 {
            let rtl = word.rtl;
            arr.push(TextRun{ words: vec![word], extent: Extent {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0
            }, rtl });
            return;
        }

        let indx = arr.len() - 1;

        if arr[indx].rtl == word.rtl {
            arr[indx].words.push(word);
        } else {
            let rtl = word.rtl;
            arr.push(TextRun{ words: vec![word], extent: Extent {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0
            }, rtl });
        }
    }

    pub fn index_chars(arr: &mut Vec<TextRun>, begin: usize){

        let mut curr = begin;
        for item in arr.iter_mut() {
            if item.rtl {
                let len = item.len();
                let mut ind = begin + len - 1;
                for word in item.words.iter_mut(){
                    for ch in word.text.iter_mut() {
                        ch.index = ind;
                        if ind > curr {
                            ind -= 1;
                        }
                    }
                    if ind > curr {
                        ind -= 1;
                    }
                }
                curr += len;
            } else {
                for word in item.words.iter_mut() {
                    for ch in word.text.iter_mut(){
                        ch.index = curr;
                        curr+=1;
                    }
                    curr+=1;
                }
            }
        }
    }

    pub fn from_chars(value: &Vec<char>, begin:usize, end:usize) -> Vec<TextRun> {
        let mut arr = vec![];

        let positions = value[begin..end].iter().positions(|&c|{ c == ' '});

        let mut curr = begin;

        for pos in positions {
            let tmp : String = value[curr..pos+begin].iter().collect();
            let (_, bidi) = unicode_compose(&tmp);

            let word = Word::from_chars(value, curr, pos+begin, bidi.paragraphs[0].level.is_rtl());
            Self::append(&mut arr,word);

            curr = pos+begin+1;
        }
        let tmp : String = value[curr..end].iter().collect();
        let (_, bidi) = unicode_compose(&tmp);
        let mut rtl = false;
        if bidi.paragraphs.len() > 0 {
            rtl = bidi.paragraphs[0].level.is_rtl();
        }
        let word = Word::from_chars(value, curr, end, rtl);
        Self::append(&mut arr,word);

        Self::index_chars(&mut arr, begin);

        arr
    }


}

#[derive(Debug, Clone)]
pub struct Paragraph {
    runs: Vec<TextRun>,
    rtl: bool
}

impl Paragraph {
    fn len(&self) -> usize {
        let mut len = 0;
        for run in self.runs.iter() {
            len += run.len() + 1;
        }

        len - 1
    }

    fn shape(
        &mut self,
        size: f32,
        family: &str,
    ){
        for run in self.runs.iter_mut() {
            run.shape(size,family);
        }
    }

    fn positon(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        size: f32,
        text_align: &Align
    ){
        let line_x = x;
        let mut _y = y;

        for run in self.runs.iter_mut() {
            run.position(line_x,x,y,w,h,size,text_align);

            _y += run.extent.y;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Paragraphs{
    list: Vec<Paragraph>,
    extent: Extent,
}

impl Paragraphs{
    pub fn from_chars(text: &Vec<char>) -> Paragraphs {
        let mut arr = vec![];

        let len = text.len();

        let mut curr = 0;

        let mut positions = text.iter().positions(|&c|{c == '\n'});

        for pos in positions {
            let runs = TextRun::from_chars(text,curr,pos);
            let rtl = if runs.len() < 0 {runs[0].rtl} else {false};
            arr.push(Paragraph{runs,rtl});
            curr = pos+1;
        }
        let runs = TextRun::from_chars(text,curr,len);
        let rtl = if runs.len() < 0 {runs[0].rtl} else {false};

        arr.push(Paragraph{runs,rtl});

        Paragraphs{list:arr, extent: Extent {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            dpi: 0.0
        } }
    }

    pub fn shape(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        size: f32,
        family: &str,
        text_align: &Align,
    ) -> Extent {
        let mut extent = Extent{
            x,
            y,
            w,
            h: size,
            dpi: 0.0,
        };

        let mut _y = y;
        for para in self.list.iter_mut() {
            para.shape(size,family);
            _y += size;
        }

        extent
    }

    pub fn position(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        size: f32,
        text_align: &Align
    ){
        let mut _y = y;
        for para in self.list.iter_mut() {
            para.positon(x,_y,w,h,size,text_align);

            _y+=size;
        }
    }

    pub fn glyphs(&self) -> Vec<GlyphInstance> {
        let mut arr = vec![];
        for para in self.list.iter() {
            for line in para.runs.iter() {
                for word in line.words.iter() {
                    arr.append(&mut word.glyphs.clone());
                }
            }
        }
        arr
    }
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