use crate::math::{Point2, Vec2};

pub struct Transform;

impl Transform {
    pub fn apply<O: Trackable, NO: Trackable>(&self, o: O) -> NO {
        unimplemented!()
    }
}

// could maybe be replaced by a Transform trait ?
pub struct ReactivePoint<'base, 'transform, B: Trackable> {
    base: &'base B,
    transform: &'transform Transform,
}

impl<'base, 'transform, B: Trackable> ReactivePoint<'base, 'transform, B> {
    pub fn new(base: &'base B, transform: &'transform Transform) -> Self {
        Self { base, transform }
    }
}

impl<'base, 'transform, B: Trackable> Trackable for ReactivePoint<'base, 'transform, B> {
    fn get_pos(&self) -> Point2 {
        self.base.get_pos()
    }
}

/// something which can give it's position when asked to
pub trait Trackable: Sized {
    fn get_pos(&self) -> Point2;
    fn to<T: Trackable>(&self, rhs: &T) -> Vec2 {
        self.get_pos().to(rhs.get_pos())
    }
    fn distance_to<T: Trackable>(&self, rhs: &T) -> f32 {
        (rhs.get_pos() - self.get_pos()).norm()
    }

    fn plus<'transform>(&self, t: &'transform Transform) -> ReactivePoint<'_, 'transform, Self> {
        ReactivePoint::new(self, t)
    }

    fn minus<'transform>(&self, t: &'transform Transform) -> ReactivePoint<'_, 'transform, Self> {
        unimplemented!()
        // ReactivePoint::new(self, *t * -1.0)
    }
}

impl<T: Fn() -> R, R: Trackable> Trackable for T {
    fn get_pos(&self) -> Point2 {
        self().get_pos()
    }
}
