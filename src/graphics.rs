
#[derive(Debug)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

#[derive(Debug)]
pub struct Bounds<T> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
}
