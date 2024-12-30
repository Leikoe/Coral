// Tiger's BangBang2d implementation in rust

use std::{cmp::Ordering, f64::consts::PI};

use crate::math::{Point2, Vec2};

use super::{bangbang1d::BangBang1d, Trajectory};

#[derive(Clone, Copy)]
pub struct BangBang2d {
    x: BangBang1d,
    y: BangBang1d,
}

impl BangBang2d {
    pub fn new(
        initial_pos: Point2,
        initial_vel: Vec2,
        target_pos: Point2,
        max_vel: f64,
        max_accel: f64,
        accuracy: f64,
    ) -> Self {
        let initial_posx = initial_pos.x;
        let initial_posy = initial_pos.y;
        let target_posx = target_pos.x;
        let target_posy = target_pos.y;
        let initial_velx = initial_vel.x;
        let initial_vely = initial_vel.y;

        let mut inc = PI / 8.0;
        let mut alpha = PI / 4.0;

        let mut x: BangBang1d = BangBang1d::new(
            initial_posx,
            initial_velx,
            target_posx,
            max_vel * alpha.cos(),
            max_accel * alpha.cos(),
        ); // TODO: don't do that
        let mut y: BangBang1d = BangBang1d::new(
            initial_posy,
            initial_vely,
            target_posy,
            max_vel * alpha.sin(),
            max_accel * alpha.sin(),
        );

        // binary search, some iterations (fixed)
        while inc > 1e-7 {
            let s_a = alpha.sin();
            let c_a = alpha.cos();

            x = BangBang1d::new(
                initial_posx,
                initial_velx,
                target_posx,
                max_vel * c_a,
                max_accel * c_a,
            );
            y = BangBang1d::new(
                initial_posy,
                initial_vely,
                target_posy,
                max_vel * s_a,
                max_accel * s_a,
            );

            let diff = (x.get_total_runtime() - y.get_total_runtime()).abs();
            if diff < accuracy {
                break;
            }
            if x.get_total_runtime() > y.get_total_runtime() {
                alpha -= inc;
            } else {
                alpha += inc;
            }

            inc *= 0.5;
        }

        Self { x, y }
    }
}

impl Trajectory<Point2, Vec2> for BangBang2d {
    fn get_position(&self, t: f64) -> Point2 {
        Point2::new(self.x.get_position(t), self.y.get_position(t))
    }

    fn get_velocity(&self, t: f64) -> Vec2 {
        Vec2::new(self.x.get_velocity(t), self.y.get_velocity(t))
    }

    fn get_acceleration(&self, t: f64) -> Vec2 {
        Vec2::new(self.x.get_acceleration(t), self.y.get_acceleration(t))
    }

    fn get_total_runtime(&self) -> f64 {
        self.x.get_total_runtime().max(self.y.get_total_runtime())
    }

    fn get_max_speed(&self) -> Option<f64> {
        self.get_time_sections()
            .map(|t| self.get_velocity(t))
            .max_by(|v1, v2| v1.norm().partial_cmp(&v2.norm()).unwrap_or(Ordering::Equal))
            .map(|v| v.norm())
    }

    fn get_time_sections(&self) -> impl Iterator<Item = f64> {
        self.x.get_time_sections().chain(self.y.get_time_sections())
    }
}
