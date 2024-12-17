use crate::math::{Point2, Vec2};

// pub trait Transform {
//     fn apply<O: Trackable>(&self, o: &O) -> Point2;
// }

// impl<T: Fn(&Point2) -> Point2> Transform for T {
//     fn apply<O: Trackable>(&self, o: &O) -> Point2 {
//         self(&o.get_pos())
//     }
// }

pub trait ReactiveVec2: Sized + Clone {
    fn get_vec(&self) -> Vec2;
    fn normalized(self) -> impl ReactiveVec2 {
        move || self.get_vec().normalized()
    }
    fn add<T: ReactiveVec2>(self, rhs: T) -> impl ReactiveVec2 {
        move || self.get_vec() + rhs.get_vec()
    }
    fn mul(self, rhs: f32) -> impl ReactiveVec2 {
        move || self.get_vec() * rhs
    }
    fn angle(&self) -> f32 {
        self.get_vec().angle()
    }
    fn norm(&self) -> f32 {
        self.get_vec().norm()
    }
}

impl<'a, T: Fn() -> Vec2 + Clone> ReactiveVec2 for T {
    fn get_vec(&self) -> Vec2 {
        self()
    }
}

// could maybe be replaced by a Transform trait ?
#[derive(Clone, Copy)]
pub struct ReactivePoint<'base, B: Trackable + Clone, T: ReactiveVec2 + Clone> {
    base: &'base B,
    transform: T,
}

impl<'base, B: Trackable, T: ReactiveVec2 + Clone> ReactivePoint<'base, B, T> {
    pub fn new(base: &'base B, transform: T) -> Self {
        Self { base, transform }
    }
}

impl<'base, B: Trackable, T: ReactiveVec2 + Clone> Trackable for ReactivePoint<'base, B, T> {
    fn get_pos(&self) -> Point2 {
        self.base.get_pos() + self.transform.get_vec()
    }
}

/// something which can give it's position when asked to
pub trait Trackable: Sized + Clone {
    fn get_pos(&self) -> Point2;

    fn to<T: Trackable>(self, rhs: T) -> impl ReactiveVec2 {
        move || Point2::to(&self.get_pos(), rhs.get_pos())
    }

    fn distance_to<T: Trackable>(&self, rhs: &T) -> f32 {
        (rhs.get_pos() - self.get_pos()).norm()
    }

    fn plus<T: ReactiveVec2>(&self, t: T) -> ReactivePoint<'_, Self, T> {
        ReactivePoint::new(self, t)
    }

    fn minus<'t, T: ReactiveVec2>(&self, t: T) -> ReactivePoint<'_, Self, impl ReactiveVec2> {
        ReactivePoint::new(self, t.mul(-1.))
    }
}

impl<T: Fn() -> R + Clone, R: Trackable> Trackable for T {
    fn get_pos(&self) -> Point2 {
        self().get_pos()
    }
}
