pub mod line;
pub mod point;
pub mod rect;
pub mod vec;

pub use line::*;
pub use point::*;
pub use rect::*;
pub use vec::*;

use std::f64::consts::{PI, TAU};

pub fn angle_difference(alpha1: f64, alpha2: f64) -> f64 {
    let diff = alpha1 - alpha2;
    match diff {
        d if d > PI => d - TAU,
        d if d < -PI => d + TAU,
        d => d,
    }
}
