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
                (Some(1.), Some(0.0))
            } else {
                (None, None)
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
                kick_speed,
                kick_angle,
                dribbler_speed,
            };

            packet.robot_commands.push(robot_command);
        }
        self.socket.send(packet)
    }

    // workaround for async Drop, to be replaced when std::future::AsyncDrop is stabilized
    async fn close(self) -> Result<(), SendError> {
        let mut packet = RobotControl::default();
        for rid in 0..1 {
            packet.robot_commands.push(RobotCommand {
                id: rid,
                move_command: Some(RobotMoveCommand {
                    command: Some(robot_move_command::Command::LocalVelocity(
                        MoveLocalVelocity {
                            forward: 0.,
                            left: 0.,
                            angular: 0.,
                        },
                    )),
                }),
                kick_speed: None,
                kick_angle: None,
                dribbler_speed: None,
            });
        }

        self.socket.send(packet).await.map(|_| ())
    }
}
