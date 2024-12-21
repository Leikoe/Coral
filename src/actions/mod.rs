use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    math::{Line, Point2, Reactive, ReactivePoint2Ext, ReactiveVec2Ext, Vec2},
    world::{AllyRobot, AvoidanceMode, Ball, World},
    CONTROL_PERIOD,
};
use tokio::{join, select, time::sleep};

pub async fn shoot<T: Reactive<Point2> + Clone>(
    world: &World,
    robot: &AllyRobot,
    ball: &Ball,
    goal: &T,
) {
    let ball_to_goal = ball.to(goal);
    let ball_to_behind_ball = ball_to_goal.normalized().mul(-0.3);

    let behind_ball = ball.plus(ball_to_behind_ball);
    let _ = robot
        .goto_rrt(
            world,
            &behind_ball,
            Some(ball_to_goal.angle()),
            AvoidanceMode::AvoidRobotsAndBall,
        )
        .await
        .unwrap(); // will fail if we are against the ball
    robot.enable_dribbler();
    select! {
        _ = robot
            .goto_rrt(
                world,
                ball,
                Some(ball_to_goal.angle()),
                AvoidanceMode::AvoidRobots,
            ) => {}
        _ = robot.wait_until_has_ball() => {}
    };
    robot.disable_dribbler();
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    while robot.has_ball() {
        interval.tick().await;
        robot.kick();
    }
}

pub async fn do_square(robot: &AllyRobot) {
    robot.goto(&Point2::new(0., 1.), None).await;
    robot.goto(&Point2::new(1., 1.), None).await;
    robot.goto(&Point2::new(1., 0.), None).await;
    robot.goto(&Point2::new(0., 0.), None).await;
    println!("reached dest!");
}

pub async fn place_ball(world: &World, robot: &AllyRobot, ball: &Ball, target_ball_pos: &Point2) {
    robot.enable_dribbler();
    let angle = robot.to(ball).angle();
    select! {
        _ = robot
            .goto_traj(
                world,
                ball,
                Some(angle),
                AvoidanceMode::AvoidRobots,
            ) => {}
        _ = robot.wait_until_has_ball() => {}
    };
    // go put the ball down
    let _ = robot
        .goto_traj(
            world,
            &(*target_ball_pos - robot.to(ball).get_reactive()),
            Some(angle),
            AvoidanceMode::AvoidRobots,
        )
        .await;
    sleep(Duration::from_millis(500)).await;
    robot.disable_dribbler();
    sleep(Duration::from_millis(500)).await;

    // step away
    let _ = robot
        .goto_traj(
            world,
            &(*target_ball_pos - robot.to(ball).get_reactive() * 4.),
            Some(angle),
            AvoidanceMode::AvoidRobots,
        )
        .await;
}

pub async fn do_square_rrt(world: &World, robot: &AllyRobot) -> Result<Vec<Point2>, String> {
    let poses = vec![
        Point2::new(-1., 1.),
        Point2::new(1., 1.),
        Point2::new(1., -1.),
        Point2::new(-1., -1.),
    ];

    let mut path = vec![robot.get_reactive()];
    for pos in &poses {
        path.extend(
            robot
                .goto_rrt(world, pos, None, AvoidanceMode::AvoidRobotsAndBall)
                .await?,
        );
    }
    println!("reached dest!");
    Ok(path)
}

// pub async fn three_attackers_attack(left_winger: &Robot, fronter: &Robot, right_winger: &Robot) {
//     let (p1, p2, p3) = (
//         Point2::new(0.5, 0.5),
//         Point2::new(0.5, -0.5),
//         Point2::new(0.5, 0.),
//     );
//     join!(
//         left_winger.goto(&p1, None),
//         right_winger.goto(&p2, None),
//         fronter.goto(&p3, None),
//     );
// }

// pub async fn go_get_ball(robot: &Robot, ball: &Ball) {
//     robot.enable_dribbler();
//     robot.goto(ball, None).await; // this will follow the ball even if it moves
//     println!("got ball!");
// }

// pub async fn intercept(robot: &Robot, ball: &Ball) {
//     let target_orientation = robot.to(ball).angle();
//     if ball.get_vel().norm() < 0.4 {
//         robot.goto(ball, Some(target_orientation)).await;
//         return;
//     }
//     let ball_trajectory = Line::new(
//         ball.get_pos(),
//         ball.get_pos() + ball.get_vel().normalized() * 100.,
//     );
//     let target_pos = ball_trajectory.closest_point_to(robot.get_pos());

//     robot.enable_dribbler();
//     robot.goto(&target_pos, Some(target_orientation)).await;
//     robot.disable_dribbler();
// }
