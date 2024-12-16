use crabe_async::{
    actions::*,
    controllers::{sim_controller::SimRobotController, RobotController},
    math::{Point2, Rect, Vec2},
    vision::Vision,
    world::{Ball, Robot, RobotId, TeamColor, Trackable, World},
    CONTROL_PERIOD,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    select,
    sync::Notify,
    task::JoinHandle,
    time::{sleep, Interval},
};

#[derive(Debug)]
pub struct RobotCommand {
    vel: Vec2,
    angular_vel: f32,
    dribble: bool,
    kick: bool,
}

fn take_next_commands(robots: &mut HashMap<RobotId, Robot>) -> HashMap<RobotId, RobotCommand> {
    robots
        .values()
        .map(|r| {
            (
                r.get_id(),
                RobotCommand {
                    vel: r.get_target_vel(),
                    angular_vel: r.get_target_angular_vel(),
                    dribble: r.should_dribble(),
                    kick: false,
                },
            )
        })
        .collect()
}

async fn control_loop<T, E: Debug, C: RobotController<T, E> + Send + 'static>(
    mut world: World,
    mut vision: Vision,
    side: TeamColor,
    mut interval: Interval,
    controller: &mut C,
) {
    loop {
        interval.tick().await; // first tick ticks immediately that's why it's at the beginning
        for packet in vision.get_pending_packets().await {
            if let Some(detection) = packet.detection {
                let (allies, ennemies) = match side {
                    TeamColor::Blue => (detection.robots_blue, detection.robots_yellow),
                    TeamColor::Yellow => (detection.robots_yellow, detection.robots_blue),
                };
                for ally_detection in allies {
                    let rid = ally_detection.robot_id() as u8;
                    if let Some(r) = world.team.get_mut(&rid) {
                        r.set_orientation(ally_detection.orientation());
                        r.set_pos(Point2::new(
                            ally_detection.x / 1000.,
                            ally_detection.y / 1000.,
                        ));
                    }
                }
                if let Some(ball_detection) = detection.balls.get(0) {
                    world
                        .ball
                        .set_pos(Point2::new(ball_detection.x, ball_detection.y));
                }
            }
        }

        println!("[DEBUG] world state");
        println!("\tball pos: {:?}", world.ball.get_pos());
        for r in world.team.values() {
            println!(
                "\trobot {} | is_dribbling: {} | pos: {:?} | orientation: {}",
                r.get_id(),
                r.should_dribble(),
                r.get_pos(),
                r.get_orientation()
            );
        }

        println!("[DEBUG] sending commands!\n");
        // ugly hack, could have been a `impl Iterator<Item &Robot>` if I was better at rust :/
        let robots = world
            .team
            .values()
            .map(|r| r.clone())
            .collect::<Vec<Robot>>();
        let sent = controller
            .send_proper_command_for(robots.into_iter())
            .await
            .expect("couldn't send commands");
    }
}

fn launch_control_thread<T, E: Debug>(
    mut world: World,
    vision_address: &'static str, // the vision ip string stays valid for the whole app's duration
    vision_port: Option<u16>,
    real: bool,
    side: TeamColor,
    mut controller: impl RobotController<T, E> + Send + 'static,
) -> (Arc<Notify>, JoinHandle<()>) {
    let notifier = Arc::new(tokio::sync::Notify::new());
    let notifier_clone = notifier.clone();
    let interval = tokio::time::interval(CONTROL_PERIOD);
    let handle = tokio::spawn(async move {
        let vision = Vision::new(vision_address, vision_port, real);

        // Robot control loop, 1Hz
        select! {
            _ = control_loop(world, vision, side, interval, &mut controller) => {}
            _ = notifier_clone.notified() => {}
        };

        sleep(Duration::from_millis(100)).await;
        controller.close().await.expect("couldn't close controller");
    });
    (notifier, handle)
}

fn make_ball_spin(ball: Ball, timeout: Option<Duration>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let start = Instant::now();
        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while timeout.is_none() || start.elapsed() < timeout.unwrap() {
            interval.tick().await;
            let elapsed_secs = start.elapsed().as_secs_f32();
            ball.set_pos(Point2::new(elapsed_secs.cos(), elapsed_secs.sin()));
        }
    })
}

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    let mut world = World {
        // TODO: don't assume field dims
        field: Rect::new(Point2::new(-3.5, 1.75), Point2::new(3.5, -1.75)),
        ball: Ball::new(Point2::new(-0.6, -0.2), Vec2::new(0.4, 0.4)),
        team: HashMap::new(),
    };
    world
        .team
        .insert(0, Robot::new(0, Point2::new(-2., 0.), 0.));
    // world.team.insert(1, Robot::new(1, Point2::zero(), 0.));
    // world
    //     .team
    //     .insert(2, Robot::new(2, Point2::new(0., -1.), 0.));

    // robot aliases
    // let (r0, r1, r2) = (
    //     world.team.get(&0).unwrap(),
    //     world.team.get(&1).unwrap(),
    //     world.team.get(&2).unwrap(),
    // );
    let r0 = world.team.get(&0).unwrap();

    let color = TeamColor::Blue;
    let controller = SimRobotController::new(color).await;
    let (control_loop_thread_stop_notifier, control_loop_thread_handle) =
        launch_control_thread(world.clone(), "224.5.23.2", None, false, color, controller);

    // do a square
    // r0.set_target_vel(Vec2::new(0.5, 0.));
    // sleep(Duration::from_secs(1)).await;
    do_square(r0).await;

    // r0.goto(&Point2::zero(), None).await;

    // // do a "three_attackers_attack" and simulate a penalty after 2s to early stop
    // let _ = tokio::time::timeout(
    //     Duration::from_millis(500),
    //     three_attackers_attack(r1, r2, r0),
    // )
    // .await;

    // // now we spin the ball and make the robot try to go get it to showcase the Trackable trait
    // make_ball_spin(world.ball.clone(), Some(Duration::from_secs(5)));
    // go_get_ball(r0, &world.ball).await;

    // // do a ball interception
    // intercept(r0, &world.ball).await;

    // showcase obstacle avoidance goto
    // teleport robots in place
    // r0.debug_tp(&Point2::new(-1., 0.), None);
    // r1.debug_tp(&Point2::new(0., 0.), None);
    // r2.debug_tp(&Point2::new(0., -1.), None);
    // let path = r0
    //     .goto_rrt(&world, &Point2::new(1., 0.), None)
    //     .await
    //     .unwrap();

    control_loop_thread_stop_notifier.notify_one(); // ask for stop
    control_loop_thread_handle
        .await
        .expect("failed to stop control loop thread!"); // wait done stopping
    sleep(Duration::from_secs(1)).await;
}
