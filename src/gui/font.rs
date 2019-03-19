use app_units;
use font_kit;
use font_kit::{family_name::FamilyName, font, source::SystemSource};
use super::properties::*;
use std::collections::HashMap;
use webrender::api::*;

use super::properties::{Align, Position};
use self::shaper::GlyphMetric;
use unicode_bidi::BidiClass;
use unicode_bidi::BidiInfo;

mod shaper {
    use std::collections::HashMap;
    use std::os::raw::{c_char, c_int, c_uint, c_void};
    use std::ptr;
    use std::sync::{Arc, Mutex};
    use webrender::api::GlyphIndex;
    //harfbuzz functions
    use harfbuzz_sys::{
        hb_blob_create, hb_buffer_add_utf8, hb_buffer_create, hb_buffer_destroy,
        hb_buffer_get_glyph_infos, hb_buffer_get_glyph_positions, hb_buffer_set_direction,
        hb_buffer_set_script, hb_face_create, hb_font_create,
        hb_font_get_glyph_extents, hb_font_set_ppem, hb_font_set_scale, hb_shape,
        //hb_blob_destroy, hb_face_destroy, hb_font_destroy,
    };
    //harfbuzz structs
    use harfbuzz_sys::{
        hb_blob_t, hb_face_t, hb_font_t, hb_glyph_extents_t,
    };
    //harfbuzz consts
    use harfbuzz_sys::{
        HB_DIRECTION_LTR, HB_DIRECTION_RTL, HB_MEMORY_MODE_READONLY,
    };

    use super::super::properties::Position;

    //pub type Dimensions = ((f32, f32), (f32, f32));
    pub type Glyph = (GlyphIndex, GlyphMetric);

    /*#[derive(Debug, Clone)]
    pub struct Point{
        pub x: f32,
        pub y: f32,
    }*/

    #[derive(Debug, Clone)]
    pub struct GlyphMetric {
        pub advance: Position,
        pub offset: Position,
        pub bearing: Position,
        pub width: f32,
        pub height: f32,
        pub size: f32,
        pub baseline: f32,
    }

    #[derive(Debug, Clone)]
    struct HBFont {
        blob: usize,
        face: usize,
        font: usize,
        bytes: Vec<u8>,
    }

    lazy_static! {
        static ref FONT: Arc<Mutex<HashMap<String, HBFont>>> = Arc::new(Mutex::new(HashMap::new()));
    }

    pub fn shape_text(
        val: &str,
        size: u32,
        baseline: f32,
        family: &str,
        rtl: bool,
        script: super::super::script::Script,
    ) -> Vec<Glyph> {
        //println!("\"{}\"script is {:?}", val, script);
        let script = script.to_hb_script();
        unsafe {
            let hb_font = {
                let mut font_map = FONT.lock().unwrap();
                if !font_map.contains_key(family) {
                    let font = super::load_font_by_name(family);
                    let font_vec: Vec<u8> = (*(font.copy_font_data().unwrap())).clone();
                    let tmp_len = font_vec.len();
                    let tmp = (&font_vec).as_ptr();

                    //let tmp = (tmp).buffer();
                    let blob = hb_blob_create(
                        tmp as *const c_char,
                        tmp_len as c_uint,
                        HB_MEMORY_MODE_READONLY,
                        ptr::null_mut() as *mut c_void,
                        None,
                    );

                    let face = hb_face_create(blob, 1 as c_uint);

                    let font = hb_font_create(face);

                    let hb_font = HBFont {
                        blob: blob as *const hb_blob_t as usize,
                        face: face as *const hb_face_t as usize,
                        font: font as *const hb_font_t as usize,
                        bytes: font_vec,
                    };

                    font_map.insert(family.to_owned(), hb_font);
                }

                font_map.get(family).unwrap().clone().font as *const hb_font_t as *mut hb_font_t
            };

            hb_font_set_ppem(hb_font, size, size);
            hb_font_set_scale(hb_font, size as i32, size as i32);

            let buf = hb_buffer_create();
            hb_buffer_add_utf8(
                buf,
                val.as_ptr() as *const c_char,
                val.len() as c_int,
                0,
                val.len() as c_int,
            );
            hb_buffer_set_script(buf, script);
            if rtl {
                hb_buffer_set_direction(buf, HB_DIRECTION_RTL);
            } else {
                hb_buffer_set_direction(buf, HB_DIRECTION_LTR);
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

                let mut extent = hb_glyph_extents_t {
                    x_bearing: 0,
                    y_bearing: 0,
                    width: 0,
                    height: 0,
                };
                hb_font_get_glyph_extents(
                    hb_font,
                    (*info).codepoint,
                    &mut extent as *mut hb_glyph_extents_t,
                );

                let metric = GlyphMetric {
                    advance: Position {
                        x: (*pos).x_advance as f32,
                        y: (*pos).y_advance as f32,
                    },
                    offset: Position {
                        x: (*pos).x_offset as f32,
                        y: (*pos).y_offset as f32,
                    },
                    bearing: Position {
                        x: extent.x_bearing as f32,
                        y: extent.y_bearing as f32,
                    },
                    width: extent.width as f32,
                    height: extent.height as f32,
                    size: size as f32,
                    baseline,
                };

                let glyphid = (*info).codepoint;

                g_vec.push((glyphid, metric));
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
pub struct Char {
    char: char,
    metric: GlyphMetric,
    index: usize,
    position: Position,
    rtl: bool,
    glyph: GlyphIndex,
}

impl Char {
    fn new(char: char, index: usize, rtl: bool) -> Char {
        Char {
            char,
            metric: GlyphMetric {
                advance: Position { x: 0.0, y: 0.0 },
                offset: Position { x: 0.0, y: 0.0 },
                bearing: Position { x: 0.0, y: 0.0 },
                width: 0.0,
                height: 0.0,
                size: 0.0,
                baseline: 0.0,
            },
            index,
            position: Position { x: 0.0, y: 0.0 },
            rtl,
            glyph: 0,
        }
    }

    pub fn get_char(&self) -> char {
        self.char
    }
    pub fn get_metric(&self) -> GlyphMetric {
        self.metric.clone()
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
    pub fn get_position(&self) -> Position {
        self.position.clone()
    }
    pub fn get_rtl(&self) -> bool {
        self.rtl
    }
    pub fn get_glyph(&self) -> GlyphIndex {
        self.glyph
    }
}

#[derive(Debug, Clone)]
pub struct Segment {
    rtl: bool,
    extent: Extent,
    class: BidiClass,
    script: super::script::Script,
    chars: Vec<Char>,
    glyphs: Vec<GlyphInstance>,
}

impl Segment {
    pub fn resolve_class(level: &super::super::unicode_bidi::Level, class: BidiClass) -> BidiClass {
        match class {
            BidiClass::B => BidiClass::B,
            BidiClass::WS => BidiClass::WS,
            BidiClass::S => BidiClass::S,
            _ => level.bidi_class(),
        }
    }

    pub fn breaking_class(&self) -> bool {
        match self.class {
            BidiClass::B => true,
            BidiClass::WS => true,
            BidiClass::S => true,
            _ => false,
        }
    }

    fn shape(&mut self, size: f32, baseline: f32, family: &str) {
        let value: String = self.chars.iter().map(|c| c.char).collect();

        let glyphs = shaper::shape_text(
            value.as_str(),
            size as u32,
            baseline,
            family,
            self.rtl,
            self.script,
        );

        self.glyphs.clear();

        let mut _x = 0.;

        let mut i = 0;
        while i < self.chars.len() {
            let (glyph, ref metric) = glyphs[i];

            self.chars[i].glyph = glyph;
            self.chars[i].metric = metric.clone();
            self.chars[i].position.x = _x;
            self.chars[i].position.y = size;

            i += 1;
            _x += metric.advance.x;
        }
        self.extent.h = size;
        self.extent.w = _x;
    }

    fn position(&mut self, x: f32, y: f32) {
        self.glyphs.clear();

        self.extent.x = x;
        self.extent.y = y;

        let mut _x = x;
        for ch in self.chars.iter_mut() {
            ch.position.x = _x;
            ch.position.y = y + ch.metric.baseline;

            _x += ch.metric.advance.x;

            self.glyphs.push(GlyphInstance {
                index: ch.glyph,
                point: LayoutPoint::new(ch.position.x, ch.position.y),
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct SegmentRef<'a> {
    _ref: &'a Segment,
}

#[derive(Debug, Clone)]
pub struct ParaLine {
    extent: Extent,
    segments: Vec<SegmentRef<'static>>,
}

impl ParaLine {
    #[allow(mutable_transmutes)]
    fn position(&mut self, x: f32, y: f32) {
        self.extent.x = x;
        self.extent.y = y;
        let mut _x = x;
        for segment in self.segments.iter_mut() {
            let tmp = unsafe {
                std::mem::transmute::<&'static Segment, &'static mut Segment>(segment._ref)
            };
            tmp.position(_x, y);
            _x += tmp.extent.w;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParaText {
    extent: Extent,
    lines: Vec<ParaLine>,
    rtl: bool,
}

impl ParaText {
    fn position(&mut self, x: f32, y: f32, w: f32, _h: f32, size: f32, text_align: &Align) {
        let mut _y = y;
        let mut max_w = 0.;
        let mut min_x = x + w;
        for line in self.lines.iter_mut() {
            let mut tmp = 0.;
            match text_align {
                Align::Middle => {
                    tmp = (w - line.extent.w) / 2.;
                }
                Align::Right => {
                    tmp = w - line.extent.w;
                }
                _ => (),
            }

            line.position(x + tmp, _y);

            if max_w < line.extent.w {
                max_w = line.extent.w;
            }
            if min_x > line.extent.x {
                min_x = line.extent.x;
            }

            _y += size;
        }

        self.extent.x = min_x;
        self.extent.y = y;
        self.extent.w = max_w;
        self.extent.h = size * self.lines.len() as f32;
    }

    fn shape_ltr(&mut self, line_directions: Vec<(usize, bool)>, w: f32) {
        let para = self;
        let line = para.lines.pop().unwrap();

        let mut tmp_line = ParaLine {
            segments: Vec::new(),
            extent: Extent::new(),
        };

        let mut prev_rtl = false;
        let mut prev_rtl_pos = 0;
        let mut i = 0;
        for dir in line_directions.iter() {
            let _tmp = dir.1;
            let mut prev_breaking_class = false;
            for j in i..dir.0 {
                if line.segments[j]._ref.extent.w + tmp_line.extent.w > w && prev_breaking_class {
                    if para.extent.w < tmp_line.extent.w {
                        para.extent.w = tmp_line.extent.w;
                    }
                    para.lines.push(tmp_line);
                    tmp_line = ParaLine {
                        segments: Vec::new(),
                        extent: Extent::new(),
                    };
                    prev_rtl = false;
                    prev_rtl_pos = 0;
                }

                prev_breaking_class = line.segments[j]._ref.breaking_class();
                tmp_line.extent.w += line.segments[j]._ref.extent.w;

                //where to insert the word?
                if prev_rtl != line.segments[j]._ref.rtl {
                    prev_rtl = line.segments[j]._ref.rtl;
                    if prev_rtl {
                        prev_rtl_pos = tmp_line.segments.len();
                    }
                }

                if prev_rtl {
                    tmp_line
                        .segments
                        .insert(prev_rtl_pos, line.segments[j].clone());
                } else {
                    tmp_line.segments.push(line.segments[j].clone());
                }
            }
            i = dir.0;
        }
        if para.extent.w < tmp_line.extent.w {
            para.extent.w = tmp_line.extent.w;
        }
        para.lines.push(tmp_line);
    }

    fn shape_rtl(&mut self, line_directions: Vec<(usize, bool)>, w: f32) {
        let para = self;
        let line = para.lines.pop().unwrap();

        let mut tmp_line = ParaLine {
            segments: Vec::new(),
            extent: Extent::new(),
        };

        let mut i = 0;
        let mut ltr_pos: Option<usize> = None;
        for dir in line_directions.iter() {
            let mut prev_breaking_class = false;

            for j in i..dir.0 {
                if line.segments[j]._ref.extent.w + tmp_line.extent.w > w && prev_breaking_class {
                    if para.extent.w < tmp_line.extent.w {
                        para.extent.w = tmp_line.extent.w;
                    }
                    para.lines.push(tmp_line);
                    tmp_line = ParaLine {
                        segments: Vec::new(),
                        extent: Extent::new(),
                    };
                    ltr_pos = None;
                }

                let tmp_pos = if line.segments[j]._ref.rtl {
                    ltr_pos = None;
                    0
                } else {
                    if ltr_pos.is_none() {
                        ltr_pos = Some(0);
                        0
                    } else {
                        match ltr_pos {
                            Some(ref mut x) => {
                                (*x) += 1;
                                (*x).clone()
                            }
                            _ => 0,
                        }
                    }
                };

                prev_breaking_class = line.segments[j]._ref.breaking_class();
                tmp_line.extent.w += line.segments[j]._ref.extent.w;

                tmp_line.segments.insert(tmp_pos, line.segments[j].clone());
            }
            i = dir.0;
        }
        if para.extent.w < tmp_line.extent.w {
            para.extent.w = tmp_line.extent.w;
        }
        para.lines.push(tmp_line);
    }
}

#[derive(Debug, Clone)]
pub struct Paragraphs {
    extent: Extent,
    segments: Vec<Segment>,
    paras: Vec<ParaText>,
}

impl Paragraphs {
    pub fn new() -> Paragraphs {
        Paragraphs {
            segments: Vec::new(),
            paras: Vec::new(),
            extent: Extent::new(),
        }
    }

    pub fn get_extent(&mut self) -> Extent {
        self.extent.clone()
    }

    pub fn from_chars(text: &Vec<char>) -> Paragraphs {
        let mut segments = vec![];

        let c_tmp = text.iter().next();
        if c_tmp.is_some() {
            let value: String = text.iter().collect();
            let info = BidiInfo::new(&value, None);

            let mut class = Segment::resolve_class(&info.levels[0], info.original_classes[0]);
            let mut script = super::script::get_script(text[0].clone());
            let mut segment = Segment {
                chars: vec![],
                rtl: info.levels[0].is_rtl(),
                extent: Extent::new(),
                class,
                script,
                glyphs: vec![],
            };
            let mut i = 0;
            let mut j = 0;

            for c in text.iter() {
                script = super::script::get_script(c.clone());
                class = Segment::resolve_class(&info.levels[i], info.original_classes[i]);
                if class != BidiClass::B && class == segment.class && script == segment.script {
                    segment
                        .chars
                        .push(Char::new(c.clone(), j, info.levels[i].is_rtl()));
                } else {
                    segments.push(segment);
                    segment = Segment {
                        chars: vec![Char::new(c.clone(), j, info.levels[i].is_rtl())],
                        rtl: info.levels[i].is_rtl(),
                        extent: Extent::new(),
                        class,
                        script,
                        glyphs: vec![],
                    };
                }

                let c_len = c.len_utf8();
                i += c_len;
                j += 1;
            }

            segments.push(segment);
        }

        Paragraphs {
            segments,
            paras: vec![],
            extent: Extent::new(),
        }
    }

    fn init_paras<'a>(
        &'a mut self,
        size: f32,
        baseline: f32,
        family: &str,
    ) -> Vec<Vec<(usize, bool)>> {
        self.paras.clear();

        let mut ret_direction = vec![];

        let mut para = ParaText {
            lines: Vec::new(),
            extent: Extent::new(),
            rtl: false,
        };
        let mut line = ParaLine {
            segments: Vec::new(),
            extent: Extent::new(),
        };
        let mut i = 0;
        let mut direction = false;
        let mut para_direction = vec![];
        let mut rtl: Option<bool> = None;
        for segment in self.segments.iter_mut() {
            //print!("{:?}", segment.class);

            if rtl.is_none() {
                rtl = Some(true);
                para.rtl = segment.rtl;
            }
            segment.shape(size, baseline, family);

            let tmp = unsafe { std::mem::transmute::<&'a Segment, &'static Segment>(segment) };
            let tmp = SegmentRef { _ref: tmp };
            if direction != segment.rtl {
                if line.segments.len() > 0 {
                    para_direction.push((i, direction));
                }
                direction = segment.rtl;
            }

            line.segments.push(tmp);

            if segment.class == BidiClass::B {
                para_direction.push((i, direction));
                para.lines.push(line);
                self.paras.push(para);
                ret_direction.push(para_direction);
                para = ParaText {
                    lines: Vec::new(),
                    extent: Extent::new(),
                    rtl: false,
                };
                line = ParaLine {
                    segments: Vec::new(),
                    extent: Extent::new(),
                };
                para_direction = vec![];
                rtl = None;
                i = 0;
                direction = false;
            } else {
                i += 1;
            }
        }
        if line.segments.len() > 0 {
            para_direction.push((i, direction));
        }
        ret_direction.push(para_direction);
        para.lines.push(line);
        self.paras.push(para);

        ret_direction
    }

    pub fn shape<'a>(
        &'a mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        size: f32,
        baseline: f32,
        family: &str,
        text_align: &Align,
    ) {
        let mut para_directions = self.init_paras(size, baseline, family);

        for para in self.paras.iter_mut() {
            let line_directions = para_directions.remove(0);
            if para.lines.len() == 0 || para.lines[0].segments.len() == 0 {
                continue;
            }

            if para.rtl {
                para.shape_rtl(line_directions, w);
            } else {
                para.shape_ltr(line_directions, w);
            }
        }

        self.position(x, y, w, h, size, text_align);
    }

    fn position(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32, text_align: &Align) {
        let mut _y = y;
        let mut min_x = x + w;
        let mut min_y = y + h;

        let mut max_w = 0.;
        for para in self.paras.iter_mut() {
            para.position(x, _y, w, h, size, text_align);

            if para.extent.w > max_w {
                max_w = para.extent.w;
            }
            if min_x > para.extent.x {
                min_x = para.extent.x;
            }
            if min_y > para.extent.y {
                min_y = para.extent.y;
            }
            _y += para.extent.h;
        }

        self.extent.x = min_x;
        self.extent.y = min_y;
        self.extent.w = max_w;
        self.extent.h = _y - y;
    }

    pub fn get_char_at_pos(
        &self,
        _p: &super::properties::Position,
        _val: &Vec<char>,
    ) -> Option<Char> {
        //let mut cursor = 0;
        //let mut pos = super::properties::Position{ x: 0.0, y: 0.0 };
        let ret = None;
        /*if !self.list.is_empty() {
            for para in self.list.iter() {
                if para.extent.y + para.extent.h < p.y {
                    let tmp = &para.lines[para.lines.len()-1];
                    let tmp = &tmp.line[tmp.line.len()-1].0.text;
                    //cursor = tmp[tmp.len()-1].index;
                    //pos = tmp[tmp.len()-1].position.clone();
                    //ret = Some(tmp[tmp.len()-1].clone());
                }
                    else {
                        for line in para.lines.iter() {

                        }
                    }
            }
        }*/
        ret
    }

    pub fn glyphs(&self) -> Vec<GlyphInstance> {
        let mut arr = vec![];
        for para in self.paras.iter() {
            for line in para.lines.iter() {
                for segment in line.segments.iter() {
                    arr.append(&mut segment._ref.glyphs.clone());
                }
                /*let lim =  line.segments.len() - 1;
                for i in 0..lim+1{
                    if i == lim && line.segments[i]._ref.breaking_class() {
                        continue;
                    }
                    let mut tmp = line.segments[i]._ref.glyphs.clone();
                    arr.append(&mut tmp);
                }*/
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
