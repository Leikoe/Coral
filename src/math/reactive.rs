use super::{Point2, Vec2};

/// helper function to not have to cast our implementor when it implements multiple Reactive<T>
pub fn get_reactive<T>(v: &impl Reactive<T>) -> T {
    v.get_reactive()
}

pub trait Reactive<T> {
    fn get_reactive(&self) -> T;
}

impl<'a, T: Fn() -> Vec2> Reactive<Vec2> for T {
    fn get_reactive(&self) -> Vec2 {
        self()
    }
}

impl<'a, T: Fn() -> Point2> Reactive<Point2> for T {
    fn get_reactive(&self) -> Point2 {
        self()
    }
}

pub trait ReactiveVec2Ext: Reactive<Vec2> + Sized {
    fn normalized(&self) -> impl Reactive<Vec2> {
        move || self.get_reactive().normalized()
    }
    fn add<T: Reactive<Vec2>>(&self, rhs: T) -> impl Reactive<Vec2> {
        move || self.get_reactive() + rhs.get_reactive()
    }
    fn mul(self, rhs: f32) -> impl Reactive<Vec2> {
        move || self.get_reactive() * rhs
    }
    fn angle(&self) -> f64 {
        self.get_reactive().angle()
    }
    fn norm(&self) -> f32 {
        self.get_reactive().norm()
    }
}

impl<T: Reactive<Vec2>> ReactiveVec2Ext for T {}

pub trait ReactivePoint2Ext: Reactive<Point2> + Sized {
    fn to<T: Reactive<Point2>>(&self, rhs: &T) -> impl Reactive<Vec2> {
        move || Point2::to(&self.get_reactive(), rhs.get_reactive())
    }

    fn distance_to<T: Reactive<Point2>>(&self, rhs: &T) -> f32 {
        (rhs.get_reactive() - self.get_reactive()).norm()
    }

    fn plus<T: Reactive<Vec2>>(&self, t: T) -> ReactivePoint<'_, Self, T> {
        ReactivePoint::new(self, t)
    }

    fn minus<'t, T: ReactiveVec2Ext>(&self, t: T) -> ReactivePoint<'_, Self, impl Reactive<Vec2>> {
        ReactivePoint::new(self, t.mul(-1.))
    }
}

impl<T: Reactive<Point2>> ReactivePoint2Ext for T {}

// could maybe be replaced by a Transform trait ?
#[derive(Clone, Copy)]
pub struct ReactivePoint<'base, B: Reactive<Point2>, T: Reactive<Vec2>> {
    base: &'base B,
    transform: T,
}

impl<'base, B: Reactive<Point2>, T: Reactive<Vec2>> ReactivePoint<'base, B, T> {
    pub fn new(base: &'base B, transform: T) -> Self {
        Self { base, transform }
    }
}

impl<'base, B: Reactive<Point2>, T: Reactive<Vec2>> Reactive<Point2>
    for ReactivePoint<'base, B, T>
{
    fn get_reactive(&self) -> Point2 {
        self.base.get_reactive() + self.transform.get_reactive()
    }
}
