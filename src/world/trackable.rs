use crate::math::{Point2, Vec2};

// could maybe be replaced by a Transform trait ?
pub struct Offsetted<'base, T: Trackable> {
    base: &'base T,
    transform: Vec2,
}

impl<'base, T: Trackable> Offsetted<'base, T> {
    fn new<U: AsRef<Vec2>>(base: &'base T, transform: U) -> Self {
        Self {
            base,
            transform: *transform.as_ref(),
        }
    }
}

impl<'base, T: Trackable> Trackable for Offsetted<'base, T> {
    fn get_pos(&self) -> Point2 {
        self.base.get_pos() + self.transform
    }
}

impl<'base, T: Trackable> AsRef<Offsetted<'base, T>> for Offsetted<'base, T> {
    fn as_ref(&self) -> &Offsetted<'base, T> {
        self
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

    fn plus(&self, t: &Vec2) -> Offsetted<'_, Self> {
        Offsetted::new(self, t)
    }

    fn minus(&self, t: &Vec2) -> Offsetted<'_, Self> {
        Offsetted::new(self, *t * -1.0)
    }
}

impl<T: Fn() -> R, R: Trackable> Trackable for T {
    fn get_pos(&self) -> Point2 {
        self().get_pos()
    }
}
