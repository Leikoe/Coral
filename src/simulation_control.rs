use crate::{
    league_protocols::simulation_packet::{
        RobotId, SimulatorCommand, SimulatorControl, SimulatorResponse, TeleportRobot,
    },
    math::{Point2, Vec2},
    net::{udp_transceiver::UdpTransceiver, ReceiveError, SendError},
    world::TeamColor,
};
use std::net::Ipv4Addr;

const DEFAULT_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_PORT: u16 = 10300;

pub struct SimulationController {
    socket: UdpTransceiver,
}

#[derive(Debug)]
pub enum SimulatorControlError {
    SimulatorControlRequestError(SendError),
    SimulatorControlResponseError(ReceiveError),
}

impl SimulationController {
    pub async fn new() -> Self {
        Self {
            socket: UdpTransceiver::new(DEFAULT_IP, DEFAULT_PORT)
                .await
                .expect("Failed to setup simulator controller."),
        }
    }

    pub async fn tp_robot(
        &mut self,
        id: u8,
        team: TeamColor,
        pos: Option<Point2>,
        orientation: Option<f64>,
        vel: Option<Vec2>,
        angular_vel: Option<f64>,
    ) -> Result<SimulatorResponse, SimulatorControlError> {
        let rid = RobotId {
            id: Some(id as u32),
            team: Some(match team {
                TeamColor::Blue => 0, // TODO: check if this is right
                TeamColor::Yellow => 1,
            }),
        };
        self.socket
            .send(SimulatorCommand {
                control: Some(SimulatorControl {
                    teleport_ball: None,
                    teleport_robot: vec![TeleportRobot {
                        id: rid,
                        x: pos.map(|p| p.x as f32),
                        y: pos.map(|p| p.y as f32),
                        orientation: orientation.map(|o| o as f32),
                        v_x: vel.map(|v| v.x as f32),
                        v_y: vel.map(|v| v.y as f32),
                        v_angular: angular_vel.map(|s| s as f32),
                        present: Some(true),
                    }],
                    simulation_speed: None,
                }),
                config: None,
            })
            .await
            .map_err(SimulatorControlError::SimulatorControlRequestError)?;
        self.socket
            .receive::<SimulatorResponse>()
            .await
            .map_err(SimulatorControlError::SimulatorControlResponseError)
    }
}
