use itertools::Itertools;
use webrender::api::*;
//use unicode_segmentation::UnicodeSegmentation;
//use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::compose;
//use unicode_normalization::Recompositions;
use unicode_bidi::BidiInfo;
//use harfbuzz::Buffer;

pub trait HandyDandyRectBuilder<T> {
    fn to(&self, x2: T, y2: T) -> LayoutRect;
    fn by(&self, w: T, h: T) -> LayoutRect;
}
// Allows doing `(x, y).to(x2, y2)` or `(x, y).by(width, height)` with i32
// values to build a f32 LayoutRect
impl HandyDandyRectBuilder<i32> for (i32, i32) {
    fn to(&self, x2: i32, y2: i32) -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(self.0 as f32, self.1 as f32),
            LayoutSize::new((x2 - self.0) as f32, (y2 - self.1) as f32),
        )
    }

    fn by(&self, w: i32, h: i32) -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(self.0 as f32, self.1 as f32),
            LayoutSize::new(w as f32, h as f32),
        )
    }
}

impl HandyDandyRectBuilder<f32> for (f32, f32) {
    fn to(&self, x2: f32, y2: f32) -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(self.0, self.1),
            LayoutSize::new(x2 - self.0, y2 - self.1),
        )
    }

    fn by(&self, w: f32, h: f32) -> LayoutRect {
        LayoutRect::new(LayoutPoint::new(self.0, self.1), LayoutSize::new(w, h))
    }
}

pub fn unicode_compose(val: &String)-> (String, BidiInfo) {

    let tmp = BidiInfo::new(&val, None);
    //println!("{:?}", tmp);

    let mut tmp_val = "".to_owned();

    for _p in tmp.paragraphs.iter() {
        let (_, levelrun) = tmp.visual_runs(_p,_p.range.clone());
        for _lr in levelrun.iter() {
            tmp_val.push_str(&tmp.reorder_line(_p, _lr.clone()).to_owned());
        }
    }

    //println!("----------\n{}\n{:?}", tmp_val, tmp);

    (tmp_val, tmp)

    /*/let tmp = tmp_val.as_str().nfkc().collect::<String>();
    //tmp_val
    let mut ch = tmp_val.chars().collect_vec();
    //ch.reverse();
    let mut ret = "".to_owned();
    let mut iter = ch.iter_mut();
    let mut tmp1 = iter.next().cloned();
    if tmp1.is_some() {
        loop {
            let tmp2 = iter.next().cloned();
            if tmp2.is_none() {
                ret.push(tmp1.unwrap());
                break;
            }
            let tmp3 = compose(tmp1.unwrap(), tmp2.unwrap());
            if tmp3.is_none() {
                ret.push(tmp1.unwrap());
                tmp1 = tmp2;
            } else {
                tmp1 = tmp3;
            }
        }
    }

    ret*/
}

/*pub fn unicode_cluster(val: &String)->String {
    let mut ch = val.chars().collect_vec();
}

pub fn unicode_transform(val: &String) -> String{
    let tmp = unicode_compose(val);
    unicode_cluster(&tmp)
}*/