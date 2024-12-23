use crabe_async::{
    actions::three_attackers_attack,
    controllers::sim_controller::SimRobotController,
    game_controller::GameController,
    launch_control_thread,
    league_protocols::game_controller_packet::referee::Command,
    math::{Point2, Vec2},
    trajectories::{bangbang2d::BangBang2d, Trajectory},
    vision::Vision,
    world::{AllyRobot, AvoidanceMode, EnnemyRobot, TeamColor, World},
    CONTROL_PERIOD, DETECTION_SCALING_FACTOR,
};
use std::time::{Duration, Instant, SystemTime};
use tokio::{join, select, time::sleep};

#[derive(Debug)]
enum HaltedState {
    Halt,
    Timeout,
}

#[derive(Debug)]
enum StoppedState {
    Stop,
    PrepareKickoff,
    BallPlacement,
    PreparePenalty,
}

#[derive(Debug)]
enum RunningState {
    Kickoff,
    FreeKick,
    Penalty,
    Run,
}

#[derive(Debug)]
enum GameState {
    Halted(HaltedState),
    Stopped(StoppedState),
    Running(RunningState),
}

// impl GameState {
//     pub fn update(self, event: GameEvent) -> Self {
//         match (self, event) {
//             // (from any state) Halt -> Halt
//             (_, GameEvent::RefereeCommand(Command::Halt)) => GameState::Halted(HaltedState::Halt),
//             // Halted
//             (GameState::Halted(HaltedState::Timeout), GameEvent::RefereeCommand(Command::Stop)) => {
//                 GameState::Stopped(StoppedState::Stop)
//             }
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::Timeout),
//             ) => GameState::Halted(HaltedState::Timeout),
//             // Stopped
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::PrepareKickoff),
//             ) => GameState::Stopped(StoppedState::PrepareKickoff),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::BallPlacement),
//             ) => GameState::Stopped(StoppedState::BallPlacement),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::PreparePenalty),
//             ) => GameState::Stopped(StoppedState::PreparePenalty),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::FreeKick),
//             ) => GameState::Running(RunningState::FreeKick),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::ForceStart),
//             ) => GameState::Running(RunningState::Run),
//             (
//                 GameState::Stopped(StoppedState::PrepareKickoff),
//                 GameEvent::RefereeCommand(Command::NormalStart),
//             ) => GameState::Running(RunningState::Kickoff),
//             (
//                 GameState::Stopped(StoppedState::BallPlacement),
//                 GameEvent::RefereeCommand(Command::Stop),
//             ) => GameState::Stopped(StoppedState::Stop),
//             (
//                 GameState::Stopped(StoppedState::BallPlacement),
//                 GameEvent::RefereeCommand(Command::Continue),
//             ) => GameState::Running(RunningState::FreeKick),
//             (
//                 GameState::Stopped(StoppedState::PreparePenalty),
//                 GameEvent::RefereeCommand(Command::NormalStart),
//             ) => GameState::Running(RunningState::Penalty),
//             // Running
//             // TODO: fix the AfterXSeconds(_)
//             (
//                 GameState::Running(RunningState::Kickoff),
//                 GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
//             ) => GameState::Running(RunningState::Run),
//             (
//                 GameState::Running(RunningState::FreeKick),
//                 GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
//             ) => GameState::Running(RunningState::Run),
//             (GameState::Running(RunningState::Run), GameEvent::Stop) => {
//                 GameState::Stopped(StoppedState::Stop)
//             }

//             _ => {
//                 println!("[ERROR] unexpected game state and event combination");
//                 unreachable!();
//             }
//         }
//     }
// }

enum GameEvent {
    RefereeCommand(Command),
}

async fn play(world: World, mut gc: GameController) {
    let r0 = world.team.lock().unwrap().get(&3).unwrap().clone();
    let r1 = world.team.lock().unwrap().get(&4).unwrap().clone();
    let r2 = world.team.lock().unwrap().get(&5).unwrap().clone();
    let ball = world.ball.clone();

    let state = GameState::Halted(HaltedState::Halt);

    let mut interval = tokio::time::interval(CONTROL_PERIOD);
    // loop {
    //     interval.tick().await; // YIELD

    //     // let gc_pending_packets = gc.take_pending_packets().await;
    //     // if let Some(p) = gc_pending_packets.last() {
    //     //     state = state.update(GameEvent::RefereeCommand(p.command()));
    //     //     dbg!(&state);
    //     // }

    //     // r0.set_target_vel(Vec2::new(1., 0.));
    //     let _ = r0
    //         .goto(&world, &Point2::zero(), None, AvoidanceMode::None)
    //         .await;
    // }

    let _ = r0
        .goto(&world, &Point2::zero(), None, AvoidanceMode::None)
        .await;

    // let start = Instant::now();
    // let traj = BangBang2d::new(
    //     Point2::zero(),
    //     Vec2::new(3., 0.),
    //     Point2::new(0., 2.),
    //     5.,
    //     10.,
    //     0.1,
    // );
    // traj.get_velocity(start.elapsed().as_secs_f64());
    // println!("duree totale: {}s", traj.get_total_runtime());
}

fn launch_vision_thread(mut world: World, real: bool) {
    tokio::spawn(async move {
        let mut vision = Vision::new(None, None, real);
        loop {
            while let Ok(packet) = vision.receive().await {
                // println!("UPDATE!");
                let mut ally_team = world.team.lock().unwrap();
                let mut ennemy_team = world.ennemies.lock().unwrap();
                let ball = world.ball.clone();
                if let Some(detection) = packet.detection {
                    let detection_time = Instant::now();
                    // world.get_creation_time() + Duration::from_secs_f64(detection.t_capture);
                    if let Some(ball_detection) = detection.balls.get(0) {
                        ball.set_pos(Point2::new(
                            ball_detection.x / DETECTION_SCALING_FACTOR,
                            ball_detection.y / DETECTION_SCALING_FACTOR,
                        ));
                    }

                    // TODO: handle ennemies
                    let (allies, ennemies) = match world.team_color {
                        TeamColor::Blue => (detection.robots_blue, detection.robots_yellow),
                        TeamColor::Yellow => (detection.robots_yellow, detection.robots_blue),
                    };
                    for ally_detection in allies {
                        let rid = ally_detection.robot_id() as u8;
                        if ally_team.get_mut(&rid).is_none() {
                            println!("[DEBUG] added ally {} to the team!", rid);
                            let r = AllyRobot::default_with_id(rid);
                            ally_team.insert(rid, r);
                        }
                        // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                        let r = ally_team.get_mut(&rid).unwrap();
                        r.update_from_packet(ally_detection, &ball, detection_time);
                    }

                    for ennemy_detection in ennemies {
                        let rid = ennemy_detection.robot_id() as u8;
                        if ennemy_team.get_mut(&rid).is_none() {
                            println!("[DEBUG] added ennemy {} to the ennemies!", rid);
                            let r = EnnemyRobot::default_with_id(rid);
                            ennemy_team.insert(rid, r);
                        }
                        // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                        let r = ennemy_team.get_mut(&rid).unwrap();
                        r.update_from_packet(ennemy_detection, &ball, detection_time);
                    }
                }
                if let Some(geometry) = packet.geometry {
                    world.field.update_from_packet(geometry.field);
                }
                // pending_packets_count += 1;
            }
        }
    });
}

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    let color = TeamColor::Blue;
    let real = false;
    let world = World::default_with_team_color(color);
    let gc = GameController::new(None, None);
    let controller = SimRobotController::new(color).await;
    let (control_loop_thread_stop_notifier, control_loop_thread_handle) =
        launch_control_thread(world.clone(), controller);
    launch_vision_thread(world.clone(), real);

    sleep(CONTROL_PERIOD * 10).await; // AWAIT ROBOTS DETECTION

    select! {
        _ = play(world, gc) => {}
        r = tokio::signal::ctrl_c() => {
            r.expect("failed to listen for event");
            println!("detected ctrl-c, stopping now!");
        }
    }

    // shoot(&world, &r0, &ball).await;

    // place_ball(&world, &r0, &ball, &goal).await;

    // r0.go_get_ball(&world, &ball).await;
    // three_attackers_attack(&world, &r1, &r0, &r2).await;
    // let (d1, d0, d2) = (
    //     Point2::new(-1., 1.),
    //     Point2::new(-1., 0.),
    //     Point2::new(-1., -1.),
    // );
    // let _ = join!(
    //     r1.goto(&world, &d1, Some(0.), AvoidanceMode::AvoidRobots,),
    //     r0.goto(&world, &d0, Some(0.), AvoidanceMode::AvoidRobots,),
    //     r2.goto(&world, &d2, Some(0.), AvoidanceMode::AvoidRobots,),
    // );

    sleep(Duration::from_millis(100)).await;
    control_loop_thread_stop_notifier.notify_one(); // ask for stop
    control_loop_thread_handle
        .await
        .expect("failed to stop control loop thread!"); // wait done stopping
    sleep(Duration::from_millis(100)).await;
}
