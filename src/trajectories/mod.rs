mod bangbang1d;
pub mod bangbang2d;

pub trait Trajectory<P, V> {
    fn get_position(&self, t: f64) -> P;
    fn get_velocity(&self, t: f64) -> V;
    fn get_acceleration(&self, t: f64) -> V;
    fn get_total_runtime(&self) -> f64;
    fn get_final_destination(&self) -> P;
    fn get_max_speed(&self) -> Option<V>;
    fn get_time_sections(&self) -> impl Iterator<Item = f64>;
}
