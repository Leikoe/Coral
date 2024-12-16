use crate::world::Robot;

pub mod sim_controller;

pub trait RobotController<R, E> {
    async fn send_proper_command_for(
        &mut self,
        robots: impl Iterator<Item = Robot>,
    ) -> Result<R, E>;
}
