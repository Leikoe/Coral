use std::{future::Future, net::Ipv4Addr};

use crate::{
    league_protocols::simulation_packet::{
        robot_move_command, MoveLocalVelocity, RobotCommand, RobotControl, RobotMoveCommand,
    },
    net::{udp_transceiver::UdpTransceiver, SendError},
    world::{Robot, TeamColor},
};

use super::RobotController;

pub struct SimRobotController {
    socket: UdpTransceiver,
}

impl SimRobotController {
    pub async fn new(color: TeamColor) -> Self {
        let port: u16 = match color {
            TeamColor::Blue => 10301,
            TeamColor::Yellow => 10302,
        };
        Self {
            socket: UdpTransceiver::new(Ipv4Addr::LOCALHOST, port)
                .await
                .expect("Failed to setup simulator robot controller."),
        }
    }
}

// impl RobotController<usize, SendError> for SimRobotController {
//     fn send_proper_command_for<'a>(
//         &mut self,
//         robots: impl Iterator<Item = &'a Robot>,
//     ) -> impl Future<Output = Result<usize, SendError>> + Send {
//         todo!()
//     }
// }

impl RobotController<usize, SendError> for SimRobotController {
    fn send_proper_command_for(
        &mut self,
        robots: impl Iterator<Item = Robot>,
    ) -> impl Future<Output = Result<usize, SendError>> + Send {
        let mut packet = RobotControl::default();

        for robot in robots {
            // let (kick_speed, kick_angle) = match &command.kick {
            //     None => (0.0, 0.0),
            //     Some(Kick::StraightKick { power }) => (*power, 0.0),
            //     Some(Kick::ChipKick { power }) => (*power, 45.0),
            // };

            let (kick_speed, kick_angle) = if robot.take_should_kick() {
                (1., 0.0)
            } else {
                (0.0, 0.0)
            };

            let target_vel = robot.get_target_vel();

            let dribbler_speed = if robot.should_dribble() {
                Some(1500.) // RPM ?
            } else {
                None
            };

            let robot_command = RobotCommand {
                id: robot.get_id() as u32,
                move_command: Some(RobotMoveCommand {
                    command: Some(robot_move_command::Command::LocalVelocity(
                        MoveLocalVelocity {
                            forward: target_vel.x,
                            left: target_vel.y,
                            angular: robot.get_target_angular_vel(),
                        },
                    )),
                }),
                kick_speed: Some(kick_speed),
                kick_angle: Some(kick_angle),
                dribbler_speed,
            };
            packet.robot_commands.push(robot_command);
        }
        self.socket.send(packet)
    }
}
