// Tiger's BangBang1d implementation in rust

use std::cmp::Ordering;

use super::Trajectory;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct BangBangPart {
    end_time: f64,
    initial_pos: f64,
    initial_vel: f64,
    accel: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BangBang1d {
    bangbang_parts: [BangBangPart; 3],
    n_parts: usize,
}

/// the position where the object will stop if it decelerates from its initial velocity to zero with the maximum deceleration
fn pos_if_brake_until_zero_vel(s0: f64, v0: f64, a_max: f64) -> f64 {
    let a = if 0. >= v0 { a_max } else { -a_max };
    let time_to_stop = -v0 / a;
    s0 + (0.5 * v0 * time_to_stop) // pos we'll be at once stopped
}

fn pos_if_brake_until_zero_vel_triangle(s0: f64, v0: f64, v1: f64, a_max: f64) -> f64 {
    let (a1, a2) = if v1 >= v0 {
        (a_max, -a_max)
    } else {
        (-a_max, a_max)
    };

    let t1 = (v1 - v0) / a1;
    let s1 = s0 + (0.5 * (v0 + v1) * t1);

    let t2 = -v1 / a2;
    s1 + (0.5 * v1 * t2)
}

impl BangBang1d {
    pub fn new(
        initial_position: f64,
        initial_vel: f64,
        target_position: f64,
        max_vel: f64,
        max_accel: f64,
    ) -> Self {
        // assuming we are at zero acceleration
        let pos_at_zero_acc = pos_if_brake_until_zero_vel(initial_position, initial_vel, max_accel);

        if pos_at_zero_acc <= target_position {
            // we're good, we can do it by just braking until zero speed
            let pos_end = pos_if_brake_until_zero_vel_triangle(
                initial_position,
                initial_vel,
                max_vel,
                max_accel,
            );

            if pos_end >= target_position {
                // Triangular profile
                calc_tri(initial_position, initial_vel, target_position, max_accel)
            } else {
                // Trapezoidal profile
                calc_trapz(
                    initial_position,
                    initial_vel,
                    max_vel,
                    target_position,
                    max_accel,
                )
            }
        } else {
            // we are going to overshoot if we just brake until zero speed, need to break MOREEE => we accel the other way
            let pos_end = pos_if_brake_until_zero_vel_triangle(
                initial_position,
                initial_vel,
                -max_vel,
                max_accel,
            );

            if pos_end <= target_position {
                // Triangular profile
                calc_tri(initial_position, initial_vel, target_position, -max_accel)
            } else {
                // Trapezoidal profile
                calc_trapz(
                    initial_position,
                    initial_vel,
                    -max_vel,
                    target_position,
                    max_accel,
                )
            }
        }
    }

    fn find_part_idx(&self, t: f64) -> usize {
        for i in 0..self.n_parts {
            if t < self.bangbang_parts[i].end_time {
                return i;
            }
        }
        return self.n_parts - 1;
    }

    fn find_part(&self, t: f64) -> BangBangPart {
        return self.bangbang_parts[self.find_part_idx(t)];
    }
}

fn calc_tri(s0: f64, v0: f64, s2: f64, a: f64) -> BangBang1d {
    let sq = if a > 0. {
        // + -
        ((a * (s2 - s0)) + (0.5 * v0 * v0)) / (a * a)
    } else {
        // - +
        ((-a * (s0 - s2)) + (0.5 * v0 * v0)) / (a * a)
    };

    let t2 = if sq > 0. { sq.sqrt() } else { 0. };
    let v1 = a * t2;
    let t1 = (v1 - v0) / a;
    let s1 = s0 + ((v0 + v1) * 0.5 * t1);

    let mut parts = [BangBangPart::default(); 3];
    parts[0].end_time = t1;
    parts[0].accel = a;
    parts[0].initial_vel = v0;
    parts[0].initial_pos = s0;
    parts[1].end_time = t1 + t2;
    parts[1].accel = -a;
    parts[1].initial_vel = v1;
    parts[1].initial_pos = s1;
    BangBang1d {
        bangbang_parts: parts,
        n_parts: 2,
    }
}

fn calc_trapz(s0: f64, v0: f64, v1: f64, s3: f64, a_max: f64) -> BangBang1d {
    let a1 = if v0 > v1 { -a_max } else { a_max };
    let a3 = if v1 > 0. { -a_max } else { a_max };
    let t1 = (v1 - v0) / a1;
    let v2 = v1;
    let t3 = -v2 / a3;
    let s1 = s0 + (0.5 * (v0 + v1) * t1);
    let s2 = s3 - (0.5 * v2 * t3);
    let t2 = (s2 - s1) / v1;

    let mut parts = [BangBangPart::default(); 3];
    parts[0].end_time = t1;
    parts[0].accel = a1;
    parts[0].initial_vel = v0;
    parts[0].initial_pos = s0;
    parts[1].end_time = t1 + t2;
    parts[1].accel = 0.;
    parts[1].initial_vel = v1;
    parts[1].initial_pos = s1;
    parts[2].end_time = t1 + t2 + t3;
    parts[2].accel = a3;
    parts[2].initial_vel = v2;
    parts[2].initial_pos = s2;
    BangBang1d {
        bangbang_parts: parts,
        n_parts: 3,
    }
}

impl Trajectory<f64, f64> for BangBang1d {
    fn get_position(&self, tt: f64) -> f64 {
        let traj_time = tt.max(0.);

        if traj_time >= self.get_total_runtime() {
            // requested time beyond final element
            let last_part = self.bangbang_parts[self.n_parts - 1];
            let t = last_part.end_time - self.bangbang_parts[self.n_parts - 2].end_time;
            return last_part.initial_pos
                + (last_part.initial_vel * t)
                + (0.5 * last_part.accel * t * t);
        }

        let piece_idx = self.find_part_idx(traj_time);
        let piece = self.bangbang_parts[piece_idx];
        let t_piece_start = if piece_idx < 1 {
            0.
        } else {
            self.bangbang_parts[piece_idx - 1].end_time
        };
        let t = traj_time - t_piece_start;
        piece.initial_pos + (piece.initial_vel * t) + (0.5 * piece.accel * t * t)
    }

    fn get_velocity(&self, tt: f64) -> f64 {
        let traj_time = tt.max(0.);

        if traj_time >= self.get_total_runtime() {
            // requested time beyond final element
            return 0.0;
        }

        let piece_idx = self.find_part_idx(traj_time);
        let piece = self.bangbang_parts[piece_idx];
        let t_piece_start = if piece_idx < 1 {
            0.
        } else {
            self.bangbang_parts[piece_idx - 1].end_time
        };
        let t = traj_time - t_piece_start;
        piece.initial_vel + (piece.accel * t)
    }

    fn get_acceleration(&self, tt: f64) -> f64 {
        let traj_time = tt.max(0.);

        if traj_time >= self.get_total_runtime() {
            // requested time beyond final element
            return 0.0;
        }

        self.find_part(traj_time).accel
    }

    fn get_total_runtime(&self) -> f64 {
        self.bangbang_parts[self.n_parts - 1].end_time
    }

    // fn get_final_destination(&self) -> f64 {
    //     self.get_position(self.get_total_runtime())
    // }

    fn get_time_sections(&self) -> impl Iterator<Item = f64> {
        self.bangbang_parts[..self.n_parts]
            .iter()
            .map(|p| p.end_time)
    }

    fn get_max_speed(&self) -> Option<f64> {
        self.get_time_sections()
            .map(|x| self.get_velocity(x))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    }
}
