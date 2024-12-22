use crabe_async::{
    actions::three_attackers_attack,
    controllers::sim_controller::SimRobotController,
    game_controller::GameController,
    launch_control_thread,
    math::Point2,
    world::{AvoidanceMode, TeamColor, World},
    CONTROL_PERIOD,
};
use std::time::Duration;
use tokio::{join, select, time::sleep};

enum HaltedState {
    Halt,
    Timeout,
}

enum StoppedState {
    Stop,
    PrepareKickoff,
    BallPlacement,
    PreparePenalty,
}

enum RunningState {
    Kickoff,
    FreeKick,
    Penalty,
    Run,
}

enum GameState {
    HaltedState(HaltedState),
    StoppedState(StoppedState),
    RunningState(RunningState),
}

impl GameState {
    pub fn update(self, event: GameEvent) -> Self {
        match (self, event) {
            // (from any state) Halt -> Halt
            (_, GameEvent::Halt) => GameState::HaltedState(HaltedState::Halt),
            // Halted
            (GameState::HaltedState(HaltedState::Timeout), GameEvent::Stop) => {
                GameState::StoppedState(StoppedState::Stop)
            }
            (GameState::StoppedState(StoppedState::Stop), GameEvent::Timeout) => {
                GameState::HaltedState(HaltedState::Timeout)
            }
            // Stopped
            (GameState::StoppedState(StoppedState::Stop), GameEvent::PrepareKickoff) => {
                GameState::StoppedState(StoppedState::PrepareKickoff)
            }
            (GameState::StoppedState(StoppedState::Stop), GameEvent::BallPlacement) => {
                GameState::StoppedState(StoppedState::BallPlacement)
            }
            (GameState::StoppedState(StoppedState::Stop), GameEvent::PreparePenalty) => {
                GameState::StoppedState(StoppedState::PreparePenalty)
            }
            (GameState::StoppedState(StoppedState::Stop), GameEvent::FreeKick) => {
                GameState::RunningState(RunningState::FreeKick)
            }
            (GameState::StoppedState(StoppedState::Stop), GameEvent::ForceStart) => {
                GameState::RunningState(RunningState::Run)
            }
            (GameState::StoppedState(StoppedState::PrepareKickoff), GameEvent::NormalStart) => {
                GameState::RunningState(RunningState::Kickoff)
            }
            (GameState::StoppedState(StoppedState::BallPlacement), GameEvent::Stop) => {
                GameState::StoppedState(StoppedState::Stop)
            }
            (GameState::StoppedState(StoppedState::BallPlacement), GameEvent::Continue) => {
                GameState::RunningState(RunningState::FreeKick)
            }
            (GameState::StoppedState(StoppedState::PreparePenalty), GameEvent::NormalStart) => {
                GameState::RunningState(RunningState::Penalty)
            }
            // Running
            // TODO: fix the AfterXSeconds(_)
            (
                GameState::RunningState(RunningState::Kickoff),
                GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
            ) => GameState::RunningState(RunningState::Run),
            (
                GameState::RunningState(RunningState::FreeKick),
                GameEvent::AfterXSeconds(_) | GameEvent::BallMoved,
            ) => GameState::RunningState(RunningState::Run),
            (GameState::RunningState(RunningState::Run), GameEvent::Stop) => {
                GameState::StoppedState(StoppedState::Stop)
            }

            _ => {
                println!("[ERROR] unexpected game state and event combination");
                unreachable!();
            }
        }
    }
}

enum GameEvent {
    Halt,
    Stop,
    Timeout,
    PrepareKickoff,
    BallPlacement,
    PreparePenalty,
    NormalStart,
    Continue,
    ForceStart,
    FreeKick,
    BallMoved,
    AfterXSeconds(u64),
}

async fn play(world: World, mut gc: GameController) {
    let r0 = world.team.lock().unwrap().get(&3).unwrap().clone();
    let r1 = world.team.lock().unwrap().get(&4).unwrap().clone();
    let r2 = world.team.lock().unwrap().get(&5).unwrap().clone();
    let ball = world.ball.clone();

    let state = GameState::HaltedState(HaltedState::Halt);

    let mut interval = tokio::time::interval(CONTROL_PERIOD);
    loop {
        let gc_pending_packets = gc.take_pending_packets().await;
        if let Some(p) = gc_pending_packets.last() {
            dbg!(p);
        }

        interval.tick().await; // YIELD
        let _ = r0
            .goto(&world, &Point2::zero(), None, AvoidanceMode::AvoidRobots)
            .await;
        println!("r{} pos after move = {:?}", r0.get_id(), r0.get_pos());
    }
}

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    let color = TeamColor::Blue;
    let world = World::default_with_team_color(color);
    let gc = GameController::new(None, None);
    let controller = SimRobotController::new(color).await;
    let (control_loop_thread_stop_notifier, control_loop_thread_handle) =
        launch_control_thread(world.clone(), None, None, false, color, controller);
    sleep(CONTROL_PERIOD * 10).await; // AWAIT ROBOTS DETECTION

    select! {
        _ = play(world, gc) => {}
        _ = tokio::signal::ctrl_c() => {
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
