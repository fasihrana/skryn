use webrender::api::*;

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