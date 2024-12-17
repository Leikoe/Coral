use crabe_async::{
    actions::*,
    controllers::{sim_controller::SimRobotController, RobotController},
    league_protocols::vision_packet::SslWrapperPacket,
    math::{Point2, Rect, Vec2},
    vision::Vision,
    world::{AvoidanceMode, Ball, Robot, TeamColor, Trackable, World},
    CONTROL_PERIOD,
};
use plotters::{
    chart::{ChartBuilder, LabelAreaPosition},
    prelude::{BitMapBackend, Circle, IntoDrawingArea},
    series::LineSeries,
    style::{full_palette::GREEN, Color, BLUE, RED, WHITE},
};
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    select,
    sync::Notify,
    task::JoinHandle,
    time::{sleep, Interval},
};

async fn control_loop<T, E: Debug, C: RobotController<T, E> + Send + 'static>(
    world: Arc<Mutex<World>>,
    mut vision: Vision,
    side: TeamColor,
    mut interval: Interval,
    controller: &mut C,
) {
    loop {
        interval.tick().await; // first tick ticks immediately that's why it's at the beginning
        let mut pending_packets_count = 0;
        let pending_packets_iterator = vision.take_pending_packets().await;
        {
            let mut w = world.lock().unwrap();
            let ball = w.ball.clone();
            for packet in pending_packets_iterator {
                if let Some(detection) = packet.detection {
                    if let Some(ball_detection) = detection.balls.get(0) {
                        ball.set_pos(Point2::new(
                            ball_detection.x / 1000.,
                            ball_detection.y / 1000.,
                        ));
                    }

                    // TODO: handle ennemies
                    let (allies, _ennemies) = match side {
                        TeamColor::Blue => (detection.robots_blue, detection.robots_yellow),
                        TeamColor::Yellow => (detection.robots_yellow, detection.robots_blue),
                    };
                    for ally_detection in allies {
                        let rid = ally_detection.robot_id() as u8;
                        let detected_pos =
                            Point2::new(ally_detection.x / 1000., ally_detection.y / 1000.);
                        let detected_orientation = ally_detection.orientation();
                        if w.team.get_mut(&rid).is_none() {
                            println!("[DEBUG] ally {} was added to team!", rid);
                            let r = Robot::new(rid, detected_pos, detected_orientation);
                            w.team.insert(rid, r);
                        }
                        // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                        let r = w.team.get_mut(&rid).unwrap();
                        r.set_orientation(detected_orientation);
                        r.set_pos(detected_pos);
                        let r_to_ball = r.to(&ball);
                        let has_ball = r_to_ball.angle().abs() < 20. && r_to_ball.norm() < 0.02;
                        r.set_has_ball(has_ball);
                    }
                }

                pending_packets_count += 1;
            }
        }

        // println!(
        //     "[TRACE] {} packets were pending, they were applied!",
        //     pending_packets_count
        // );

        // println!("[DEBUG] world state");
        // println!("\tball pos: {:?}", world.lock().unwrap().ball.get_pos());
        // for r in world.lock().unwrap().team.values() {
        //     println!(
        //         "\trobot {} | is_dribbling: {} | pos: {:?} | orientation: {}",
        //         r.get_id(),
        //         r.should_dribble(),
        //         r.get_pos(),
        //         r.get_orientation()
        //     );
        // }

        // println!("[DEBUG] sending commands!\n");
        // ugly hack, could have been a `impl Iterator<Item &Robot>` if I was better at rust :/
        let robots = world
            .lock()
            .unwrap()
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
    world: Arc<Mutex<World>>,
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
    let world = Arc::new(Mutex::new(World {
        // TODO: don't assume field dims
        field: Rect::new(Point2::new(-4.5, 3.), Point2::new(4.5, -3.)),
        ball: Ball::new(Point2::new(-0.6, -0.2), Vec2::new(0.4, 0.4)),
        team: HashMap::new(),
    }));
    // world
    //     .lock()
    //     .unwrap()
    //     .team
    //     .insert(0, Robot::new(0, Point2::new(-2., 0.), 0.));

    // world.team.insert(1, Robot::new(1, Point2::zero(), 0.));
    // world
    //     .team
    //     .insert(2, Robot::new(2, Point2::new(0., -1.), 0.));

    let color = TeamColor::Blue;
    let controller = SimRobotController::new(color).await;
    let (control_loop_thread_stop_notifier, control_loop_thread_handle) =
        launch_control_thread(world.clone(), "224.5.23.2", None, false, color, controller);
    sleep(CONTROL_PERIOD * 2).await; // AWAIT ROBOTS DETECTION

    // robot aliases
    // let (r0, r1, r2) = (
    //     world.team.get(&0).unwrap(),
    //     world.team.get(&1).unwrap(),
    //     world.team.get(&2).unwrap(),
    // );

    let r0 = world.lock().unwrap().team.get(&0).unwrap().clone();
    let ball = world.lock().unwrap().ball.clone();

    // shoot(&world, &r0, &ball).await;

    // do a square
    // r0.set_target_vel(Vec2::new(1., 0.));
    // sleep(Duration::from_secs(1)).await;
    // let path = do_square_rrt(&world, &r0)
    //     .await
    //     .expect("couldn't find a path");

    let goal = Point2::new(-3., 0.);
    let path = r0
        .goto_rrt(&world, &goal, None, AvoidanceMode::AvoidRobotsAndBall)
        .await
        .unwrap();

    {
        // PLOT
        let root_area = BitMapBackend::new("plot.png", (600, 400)).into_drawing_area();
        root_area.fill(&WHITE).unwrap();

        let to_int = |f: f32| (f * 10.) as i32;

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption("Evitement", ("sans-serif", 40))
            .build_cartesian_2d(-45..45, -30..30)
            .unwrap();

        ctx.configure_mesh().draw().unwrap();

        ctx.draw_series(
            world.lock().unwrap().team.iter().map(|(id, r)| {
                Circle::new((to_int(r.get_pos().x), to_int(r.get_pos().y)), 5, &BLUE)
            }),
        )
        .unwrap();

        ctx.draw_series(
            path.iter()
                .map(|p| Circle::new((to_int(p.x), to_int(p.y)), 5, GREEN.filled()))
                .take(1),
        )
        .unwrap();

        // ctx.draw_series(
        //     vec![goal]
        //         .iter()
        //         .map(|p| Circle::new((to_int(p.x), to_int(p.y)), 5, RED.filled()))
        //         .take(1),
        // )
        // .unwrap();

        ctx.draw_series(LineSeries::new(
            path.iter().map(|p| (to_int(p.x), to_int(p.y))),
            &RED,
        ))
        .unwrap();
    }

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

    // sleep(Duration::from_secs(4)).await;

    sleep(Duration::from_millis(100)).await;
    control_loop_thread_stop_notifier.notify_one(); // ask for stop
    control_loop_thread_handle
        .await
        .expect("failed to stop control loop thread!"); // wait done stopping
    sleep(Duration::from_millis(100)).await;
}
