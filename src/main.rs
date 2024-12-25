use crabe_async::{
    actions::{backwards_strike, do_square_rrt, keep, place_ball, three_attackers_attack},
    controllers::sim_controller::SimRobotController,
    game_controller::GameController,
    launch_control_thread,
    league_protocols::game_controller_packet::referee::Command,
    math::{Point2, Reactive, Vec2},
    trajectories::{bangbang2d::BangBang2d, Trajectory},
    viewer::run_viewer_server_forever,
    vision::Vision,
    world::{AllyRobot, AvoidanceMode, EnnemyRobot, RobotId, TeamColor, World},
    CONTROL_PERIOD, DETECTION_SCALING_FACTOR,
};
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};
use tokio::{join, select, time::sleep};

#[derive(Debug, Clone, Copy)]
enum HaltedState {
    Halt,
    Timeout,
}

#[derive(Debug, Clone, Copy)]
enum StoppedState {
    Stop,
    PrepareKickoff,
    BallPlacementUs,
    BallPlacementThem,
    PreparePenalty,
}

#[derive(Debug, Clone, Copy)]
enum RunningState {
    Kickoff,
    FreeKickUs,
    FreeKickThem,
    Penalty,
    Run,
}

#[derive(Debug, Clone, Copy)]
enum GameState {
    Halted(HaltedState),
    Stopped(StoppedState),
    Running(RunningState),
}

impl GameState {
    pub fn update(self, event: GameEvent, color: TeamColor) -> Self {
        match (self, event, color) {
            // (from any state) Halt -> Halt
            (_, GameEvent::RefereeCommand(Command::Halt), _) => {
                GameState::Halted(HaltedState::Halt)
            }
            // Halted
            (
                GameState::Halted(HaltedState::Halt | HaltedState::Timeout),
                GameEvent::RefereeCommand(Command::Stop),
                _,
            ) => GameState::Stopped(StoppedState::Stop),
            // (
            //     GameState::Stopped(StoppedState::Stop),
            //     GameEvent::RefereeCommand(Command::Timeout),
            // ) => GameState::Halted(HaltedState::Timeout),
            // Stopped
            // (
            //     GameState::Stopped(StoppedState::Stop),
            //     GameEvent::RefereeCommand(Command::PrepareKickoff),
            // ) => GameState::Stopped(StoppedState::PrepareKickoff),
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::BallPlacementBlue),
                TeamColor::Blue,
            ) => GameState::Stopped(StoppedState::BallPlacementUs),
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::BallPlacementBlue),
                TeamColor::Yellow,
            ) => GameState::Stopped(StoppedState::BallPlacementThem),
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::BallPlacementYellow),
                TeamColor::Blue,
            ) => GameState::Stopped(StoppedState::BallPlacementThem),
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::BallPlacementYellow),
                TeamColor::Yellow,
            ) => GameState::Stopped(StoppedState::BallPlacementUs),
            // (
            //     GameState::Stopped(StoppedState::Stop),
            //     GameEvent::RefereeCommand(Command::PreparePenalty),
            // ) => GameState::Stopped(StoppedState::PreparePenalty),
            // FREE KICKS
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::DirectFreeBlue),
                TeamColor::Blue,
            ) => GameState::Running(RunningState::FreeKickUs),
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::DirectFreeYellow),
                TeamColor::Blue,
            ) => GameState::Running(RunningState::FreeKickThem),
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::DirectFreeBlue),
                TeamColor::Yellow,
            ) => GameState::Running(RunningState::FreeKickThem),
            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::DirectFreeYellow),
                TeamColor::Yellow,
            ) => GameState::Running(RunningState::FreeKickUs),

            (
                GameState::Stopped(StoppedState::Stop),
                GameEvent::RefereeCommand(Command::ForceStart),
                _,
            ) => GameState::Running(RunningState::Run),
            (
                GameState::Stopped(StoppedState::PrepareKickoff),
                GameEvent::RefereeCommand(Command::NormalStart),
                _,
            ) => GameState::Running(RunningState::Kickoff),
            (
                GameState::Stopped(StoppedState::BallPlacementUs | StoppedState::BallPlacementThem),
                GameEvent::RefereeCommand(Command::Stop),
                _,
            ) => GameState::Stopped(StoppedState::Stop),
            // (
            //     GameState::Stopped(StoppedState::BallPlacement),
            //     GameEvent::RefereeCommand(Command::Continue),
            // ) => GameState::Running(RunningState::FreeKick),
            (
                GameState::Stopped(StoppedState::PreparePenalty),
                GameEvent::RefereeCommand(Command::NormalStart),
                _,
            ) => GameState::Running(RunningState::Penalty),
            // Running
            // TODO: fix the AfterXSeconds(_)
            // (
            //     GameState::Running(RunningState::Kickoff),
            //     GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
            // ) => GameState::Running(RunningState::Run),
            // (
            //     GameState::Running(RunningState::FreeKick),
            //     GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
            // ) => GameState::Running(RunningState::Run),
            // (GameState::Running(RunningState::Run), GameEvent::Stop) => {
            //     GameState::Stopped(StoppedState::Stop)
            // }
            (s, e, _) => {
                println!(
                    "[WARNING] unexpected game state and event combination ({:?}, {:?})",
                    s, e
                );
                s
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum GameEvent {
    RefereeCommand(Command),
}

async fn play(world: World, mut gc: GameController) {
    let r0 = world.team.lock().unwrap().get(&3).unwrap().clone();
    let r1 = world.team.lock().unwrap().get(&4).unwrap().clone();
    let r2 = world.team.lock().unwrap().get(&5).unwrap().clone();
    let ball = world.ball.clone();

    let mut state = GameState::Halted(HaltedState::Halt);

    let mut interval = tokio::time::interval(CONTROL_PERIOD);
    let start = Instant::now();

    // loop {
    //     let referee_event = gc.receive().await.unwrap();
    //     state = state.update(
    //         GameEvent::RefereeCommand(referee_event.command()),
    //         world.team_color,
    //     );
    //     println!(
    //         "gc: received {:?}, transitionning to {:?}",
    //         referee_event.command(),
    //         state
    //     );

    //     match state {
    //         GameState::Halted(halted_state) => todo!(),
    //         GameState::Stopped(stopped_state) => match stopped_state {
    //             StoppedState::Stop => todo!(),
    //             StoppedState::PrepareKickoff => todo!(),
    //             StoppedState::BallPlacementUs => {
    //                 if let Some(p) = referee_event.designated_position {
    //                     let p = Point2::new(p.x, p.y);
    //                     place_ball(&world, &r0, &ball, &p).await;
    //                 }
    //             }
    //             StoppedState::PreparePenalty => todo!(),
    //             StoppedState::BallPlacementThem => todo!(),
    //         },
    //         GameState::Running(running_state) => todo!(),
    //     }
    // }

    loop {
        interval.tick().await;
        // let _ = r0
        //     .goto(&world, &Point2::zero(), None, AvoidanceMode::None)
        //     .await;

        // keep(&world, &r0, &ball).await;

        do_square_rrt(&world, &r0).await;

        // backwards_strike(&world, &r0, &ball).await;

        // let _ = r0
        //     .goto(
        //         &world,
        //         &|| {
        //             Point2::new(
        //                 start.elapsed().as_secs_f64().cos() * 1.0,
        //                 start.elapsed().as_secs_f64().sin() * 1.0,
        //             )
        //         },
        //         None,
        //         AvoidanceMode::None,
        //     )
        //     .await;
    }
}

async fn update_world_with_vision_forever(mut world: World, real: bool) {
    let mut vision = Vision::new(None, None, real);
    let update_notifier = world.get_update_notifier();
    loop {
        while let Ok(packet) = vision.receive().await {
            let mut ally_team = world.team.lock().unwrap();
            let mut ennemy_team = world.ennemies.lock().unwrap();
            let ball = world.ball.clone();
            if let Some(detection) = packet.detection {
                // println!("NEW CAM PACKET!");
                let detection_time = detection.t_capture;
                if let Some(ball_detection) = detection.balls.get(0) {
                    let detected_pos = Point2::new(
                        ball_detection.x as f64 / DETECTION_SCALING_FACTOR,
                        ball_detection.y as f64 / DETECTION_SCALING_FACTOR,
                    );
                    if let Some(last_t) = ball.get_last_update() {
                        let dt = detection_time - last_t;
                        ball.set_vel((detected_pos - ball.get_pos()) / dt);
                    }
                    // println!("{:?}", detected_pos);
                    ball.set_pos(detected_pos);
                }

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
                    let r = ally_team
                        .get_mut(&rid)
                        .expect("couldn't find a robot which SHOULD have been in our team");
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
                update_notifier.notify_waiters();
            }
            if let Some(geometry) = packet.geometry {
                world.field.update_from_packet(geometry.field);
            }
        }
    }
}

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    let color = TeamColor::Blue;
    let real = false;
    let world = World::default_with_team_color(color);
    let gc = GameController::new(None, None);
    let controller = SimRobotController::new(color).await;
    run_viewer_server_forever(Ipv4Addr::new(127, 0, 0, 1), 8282).await;
    tokio::spawn(update_world_with_vision_forever(world.clone(), real));
    let (control_loop_thread_stop_notifier, control_loop_thread_handle) =
        launch_control_thread(world.clone(), controller);
    sleep(CONTROL_PERIOD * 10).await; // AWAIT ROBOTS DETECTION

    select! {
        _ = play(world, gc) => {}
        r = tokio::signal::ctrl_c() => {
            r.expect("failed to listen for event");
            println!("detected ctrl-c, stopping now!");
        }
    }

    sleep(Duration::from_millis(100)).await;
    control_loop_thread_stop_notifier.notify_one(); // ask for stop
    control_loop_thread_handle
        .await
        .expect("failed to stop control loop thread!"); // wait done stopping
    sleep(Duration::from_millis(100)).await;
}
