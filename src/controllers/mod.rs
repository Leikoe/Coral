use std::fmt::Debug;

use crate::world::AllyRobot;

pub mod sim_controller;

pub trait RobotController<R, E>
where
    E: Debug,
{
    fn send_proper_command_for(
        &mut self,
        robots: impl Iterator<Item = AllyRobot>,
    ) -> impl std::future::Future<Output = Result<R, E>> + Send;

    fn close(self) -> impl std::future::Future<Output = Result<(), E>> + Send;
}
