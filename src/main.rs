use crabe_async::{
    actions::{backwards_strike, do_square_rrt},
    controllers::sim_controller::SimRobotController,
    game_controller::GameController,
    launch_control_thread, update_world_with_vision_forever, viewer,
    world::{TeamColor, World},
};
use std::{str::FromStr, time::Duration};
use tokio::{join, select, time::sleep};
use tracing::info;
use tracing_subscriber::EnvFilter;

// #[derive(Debug, Clone, Copy)]
// enum HaltedState {
//     Halt,
//     Timeout,
// }

// #[derive(Debug, Clone, Copy)]
// enum StoppedState {
//     Stop,
//     PrepareKickoff,
//     BallPlacementUs,
//     BallPlacementThem,
//     PreparePenalty,
// }

// #[derive(Debug, Clone, Copy)]
// enum RunningState {
//     Kickoff,
//     FreeKickUs,
//     FreeKickThem,
//     Penalty,
//     Run,
// }

// #[derive(Debug, Clone, Copy)]
// enum GameState {
//     Halted(HaltedState),
//     Stopped(StoppedState),
//     Running(RunningState),
// }

// impl GameState {
//     pub fn update(self, event: GameEvent, color: TeamColor) -> Self {
//         match (self, event, color) {
//             // (from any state) Halt -> Halt
//             (_, GameEvent::RefereeCommand(Command::Halt), _) => {
//                 GameState::Halted(HaltedState::Halt)
//             }
//             // Halted
//             (
//                 GameState::Halted(HaltedState::Halt | HaltedState::Timeout),
//                 GameEvent::RefereeCommand(Command::Stop),
//                 _,
//             ) => GameState::Stopped(StoppedState::Stop),
//             // (
//             //     GameState::Stopped(StoppedState::Stop),
//             //     GameEvent::RefereeCommand(Command::Timeout),
//             // ) => GameState::Halted(HaltedState::Timeout),
//             // Stopped
//             // (
//             //     GameState::Stopped(StoppedState::Stop),
//             //     GameEvent::RefereeCommand(Command::PrepareKickoff),
//             // ) => GameState::Stopped(StoppedState::PrepareKickoff),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::BallPlacementBlue),
//                 TeamColor::Blue,
//             ) => GameState::Stopped(StoppedState::BallPlacementUs),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::BallPlacementBlue),
//                 TeamColor::Yellow,
//             ) => GameState::Stopped(StoppedState::BallPlacementThem),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::BallPlacementYellow),
//                 TeamColor::Blue,
//             ) => GameState::Stopped(StoppedState::BallPlacementThem),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::BallPlacementYellow),
//                 TeamColor::Yellow,
//             ) => GameState::Stopped(StoppedState::BallPlacementUs),
//             // (
//             //     GameState::Stopped(StoppedState::Stop),
//             //     GameEvent::RefereeCommand(Command::PreparePenalty),
//             // ) => GameState::Stopped(StoppedState::PreparePenalty),
//             // FREE KICKS
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::DirectFreeBlue),
//                 TeamColor::Blue,
//             ) => GameState::Running(RunningState::FreeKickUs),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::DirectFreeYellow),
//                 TeamColor::Blue,
//             ) => GameState::Running(RunningState::FreeKickThem),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::DirectFreeBlue),
//                 TeamColor::Yellow,
//             ) => GameState::Running(RunningState::FreeKickThem),
//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::DirectFreeYellow),
//                 TeamColor::Yellow,
//             ) => GameState::Running(RunningState::FreeKickUs),

//             (
//                 GameState::Stopped(StoppedState::Stop),
//                 GameEvent::RefereeCommand(Command::ForceStart),
//                 _,
//             ) => GameState::Running(RunningState::Run),
//             (
//                 GameState::Stopped(StoppedState::PrepareKickoff),
//                 GameEvent::RefereeCommand(Command::NormalStart),
//                 _,
//             ) => GameState::Running(RunningState::Kickoff),
//             (
//                 GameState::Stopped(StoppedState::BallPlacementUs | StoppedState::BallPlacementThem),
//                 GameEvent::RefereeCommand(Command::Stop),
//                 _,
//             ) => GameState::Stopped(StoppedState::Stop),
//             // (
//             //     GameState::Stopped(StoppedState::BallPlacement),
//             //     GameEvent::RefereeCommand(Command::Continue),
//             // ) => GameState::Running(RunningState::FreeKick),
//             (
//                 GameState::Stopped(StoppedState::PreparePenalty),
//                 GameEvent::RefereeCommand(Command::NormalStart),
//                 _,
//             ) => GameState::Running(RunningState::Penalty),
//             // Running
//             // TODO: fix the AfterXSeconds(_)
//             // (
//             //     GameState::Running(RunningState::Kickoff),
//             //     GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
//             // ) => GameState::Running(RunningState::Run),
//             // (
//             //     GameState::Running(RunningState::FreeKick),
//             //     GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
//             // ) => GameState::Running(RunningState::Run),
//             // (GameState::Running(RunningState::Run), GameEvent::Stop) => {
//             //     GameState::Stopped(StoppedState::Stop)
//             // }
//             (s, e, _) => {
//                 println!(
//                     "[WARNING] unexpected game state and event combination ({:?}, {:?})",
//                     s, e
//                 );
//                 s
//             }
//         }
//     }
// }

// #[derive(Debug, Clone, Copy)]
// enum GameEvent {
//     RefereeCommand(Command),
// }

async fn play(world: World, _gc: GameController) {
    // let state = GameState::Halted(HaltedState::Halt);

    // let interval = tokio::time::interval(CONTROL_PERIOD);
    // let start = Instant::now();

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
        let r0 = if let Some(r0) = world.team.lock().unwrap().get(&3).cloned() {
            r0
        } else {
            continue;
        };
        let r1 = if let Some(r1) = world.team.lock().unwrap().get(&4).cloned() {
            r1
        } else {
            continue;
        };
        // let r2 = world.team.lock().unwrap().get(&5).unwrap().clone();
        let ball = world.ball.clone();

        // let _ = r0
        //     .goto(&world, &Point2::zero(), None, AvoidanceMode::None)
        //     .await;

        // keep(&world, &r0, &ball).await;

        let (res1, _res2) = join!(
            do_square_rrt(&world, &r0),
            backwards_strike(&world, &r1, &ball)
        );
        res1.expect("r3 couldn't do the square");
        // res2.expect("r1 couldn't do the square");

        // if let Err(e) = do_square_rrt(&world, &r0).await {
        //     println!("{:?}", e);
        // }

        // backwards_strike(&world, &r0, &ball).await;

        // three_attackers_attack(&world, &r1, &r0, &r2).await;

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
        sleep(Duration::from_secs(1)).await;
    }
}

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            // we only log >=warn OR >=info for viewer OR >=debug for this crate
            EnvFilter::from_str("warn,crabe_async::viewer=info,crabe_async=debug")
                .expect("couldn't parse log filter"),
        )
        .init();

    // options which will come from cli
    let color = TeamColor::Blue;
    let real = false;

    info!("Starting up Coral (color: {:?}, real: {})", color, real);

    let world = World::default_with_team_color(color);
    let gc = GameController::new(None, None);
    let controller = if real {
        unimplemented!("didn't write real robots controller yet");
    } else {
        SimRobotController::new(color).await
    };
    viewer::init().await;

    tokio::spawn(update_world_with_vision_forever(world.clone(), real));
    let control_thread_handle = launch_control_thread(world.clone(), controller);

    // await allies detection
    world.allies_detection().await;

    // play until ctrl-c
    select! {
        _ = play(world, gc) => {}
        r = tokio::signal::ctrl_c() => {
            r.expect("failed to listen for event");
            info!("detected ctrl-c, stopping now!");
        }
    }

    control_thread_handle.stop().await;
}
