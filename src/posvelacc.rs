use na::Normed;

use crate::math::{Angle, Point2, Vec2};

struct OffsettedPoint2<'p, 'o, P: Pos2, O: Offset2> {
    point: &'p P,
    offset: &'o O,
}

impl<'p, 'o, P: Pos2, O: Offset2> Pos2 for OffsettedPoint2<'p, 'o, P, O> {
    fn pos(&self) -> Point2 {
        self.point.pos() + self.offset.offset()
    }
}

struct Difference<'p1, 'p2, P1: Pos2, P2: Pos2> {
    point1: &'p1 P1,
    point2: &'p2 P2,
}

impl<'p1, 'p2, P1: Pos2, P2: Pos2> Offset2 for Difference<'p1, 'p2, P1, P2> {
    fn offset(&self) -> Vec2 {
        self.point2.pos() - self.point1.pos()
    }
}

pub trait Pos2: Sized {
    fn pos(&self) -> Point2;

    fn to<'p2, P2: Pos2>(&self, other: &'p2 P2) -> Difference<'_, 'p2, Self, P2> {
        Difference {
            point1: self,
            point2: other,
        }
    }

    fn distance_to<'p2, P2: Pos2>(&self, other: &'p2 P2) -> f64 {
        self.to(other).offset().norm()
    }

    fn plus(&self, offset: impl Offset2) -> impl Pos2 {
        move || self.pos() + offset.offset()
    }

    fn minus(&self, offset: impl Offset2) -> impl Pos2 {
        move || self.pos() - offset.offset()
    }
}

pub trait Offset2 {
    fn offset(&self) -> Vec2;

    fn normalized(&self) -> impl Offset2 {
        || self.offset().normalize() as Vec2
    }

    fn angle(&self) -> impl Angle {
        || self.offset()._angle()
    }

    fn scale(&self, scale: f64) -> impl Offset2 {
        move || self.offset() * scale
    }
}

pub trait Vel2 {
    fn vel(&self) -> Vec2;
}

pub trait Acc2 {
    fn acc(&self) -> Vec2;
}

impl Pos2 for Point2 {
    fn pos(&self) -> Point2 {
        *self
    }
}

impl<F: Fn() -> Point2> Pos2 for F {
    fn pos(&self) -> Point2 {
        self()
    }
}

impl Offset2 for Vec2 {
    fn offset(&self) -> Vec2 {
        *self
    }
}

impl<F: Fn() -> Vec2> Offset2 for F {
    fn offset(&self) -> Vec2 {
        self()
    }
}
