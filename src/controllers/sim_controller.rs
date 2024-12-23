use std::{collections::HashMap, future::Future, net::Ipv4Addr};

use crate::{
    league_protocols::simulation_packet::{
        robot_move_command, MoveLocalVelocity, MoveWheelVelocity, RobotCommand, RobotControl,
        RobotControlResponse, RobotFeedback, RobotMoveCommand,
    },
    net::{udp_transceiver::UdpTransceiver, ReceiveError, SendError},
    world::{AllyRobot, RobotId, TeamColor},
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

    pub async fn receive_feedback(&mut self) -> Result<RobotControlResponse, ReceiveError> {
        self.socket.receive::<RobotControlResponse>().await
    }
}

impl RobotController<HashMap<RobotId, RobotFeedback>, SendError> for SimRobotController {
    fn send_proper_command_for(
        &mut self,
        robots: impl Iterator<Item = AllyRobot>,
    ) -> impl Future<Output = Result<HashMap<RobotId, RobotFeedback>, SendError>> + Send {
        let mut packet = RobotControl::default();

        for robot in robots {
            // let (kick_speed, kick_angle) = match &command.kick {
            //     None => (0.0, 0.0),
            //     Some(Kick::StraightKick { power }) => (*power, 0.0),
            //     Some(Kick::ChipKick { power }) => (*power, 45.0),
            // };

            let (kick_speed, kick_angle) = if robot.take_should_kick() {
                (Some(5.), Some(0.0))
            } else {
                (None, None)
            };

            let target_vel = robot.get_target_vel();

            let dribbler_speed = if robot.should_dribble() {
                Some(1500.) // RPM ?
            } else {
                Some(0.)
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

        async {
            self.socket.send(packet).await?;
            let mut feedback_per_robot = HashMap::new();
            while let Ok(feedback_packet) = self.receive_feedback().await {
                for feedback in feedback_packet.feedback {
                    feedback_per_robot.insert(feedback.id as RobotId, feedback);
                }
            }
            Ok(feedback_per_robot)
        }
    }

    // workaround for async Drop, to be replaced when std::future::AsyncDrop is stabilized
    async fn close(self) -> Result<(), SendError> {
        let mut packet = RobotControl::default();
        for rid in 0..16 {
            packet.robot_commands.push(RobotCommand {
                id: rid,
                move_command: Some(RobotMoveCommand {
                    command: Some(robot_move_command::Command::WheelVelocity(
                        MoveWheelVelocity {
                            front_right: 0.,
                            back_right: 0.,
                            back_left: 0.,
                            front_left: 0.,
                        },
                    )),
                }),
                kick_speed: Some(5.),
                kick_angle: Some(0.),
                dribbler_speed: Some(0.),
            });
        }
        println!("stopping robots..");
        self.socket.send(packet).await.map(|_| ())
    }
}
