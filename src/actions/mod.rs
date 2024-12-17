use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    math::{Line, Point2, Vec2},
    world::{AvoidanceMode, Ball, Robot, Trackable, World},
    CONTROL_PERIOD,
};
use tokio::{join, time::sleep};

pub async fn do_square(robot: &Robot) {
    robot.goto(&Point2::new(0., 1.), None).await;
    robot.goto(&Point2::new(1., 1.), None).await;
    robot.goto(&Point2::new(1., 0.), None).await;
    robot.goto(&Point2::new(0., 0.), None).await;
    println!("reached dest!");
}

pub async fn do_square_rrt(
    world: &Arc<Mutex<World>>,
    robot: &Robot,
) -> Result<Vec<Point2>, String> {
    let mut path = vec![robot.get_pos()];
    path.extend(
        robot
            .goto_rrt(
                world,
                &Point2::new(-1., 1.),
                None,
                AvoidanceMode::AvoidRobotsAndBall,
            )
            .await?,
    );
    path.extend(
        robot
            .goto_rrt(
                world,
                &Point2::new(1., 1.),
                None,
                AvoidanceMode::AvoidRobotsAndBall,
            )
            .await?,
    );
    path.extend(
        robot
            .goto_rrt(
                world,
                &Point2::new(1., -1.),
                None,
                AvoidanceMode::AvoidRobotsAndBall,
            )
            .await?,
    );
    path.extend(
        robot
            .goto_rrt(
                world,
                &Point2::new(-1., -1.),
                None,
                AvoidanceMode::AvoidRobotsAndBall,
            )
            .await?,
    );
    println!("reached dest!");
    Ok(path)
}

pub async fn three_attackers_attack(left_winger: &Robot, fronter: &Robot, right_winger: &Robot) {
    let (p1, p2, p3) = (
        Point2::new(0.5, 0.5),
        Point2::new(0.5, -0.5),
        Point2::new(0.5, 0.),
    );
    join!(
        left_winger.goto(&p1, None),
        right_winger.goto(&p2, None),
        fronter.goto(&p3, None),
    );
}

pub async fn go_get_ball(robot: &Robot, ball: &Ball) {
    robot.enable_dribbler();
    robot.goto(ball, None).await; // this will follow the ball even if it moves
    println!("got ball!");
}

pub async fn shoot(world: &Arc<Mutex<World>>, robot: &Robot, ball: &Ball) -> Result<(), String> {
    robot
        .goto_rrt(
            &world,
            &(ball.get_pos() - Vec2::new(-0.2, 0.)),
            Some(robot.to(ball).angle()),
            AvoidanceMode::AvoidRobotsAndBall,
        )
        .await?;
    robot
        .goto_rrt(
            &world,
            &(ball.get_pos() - Vec2::new(-0.05, 0.)),
            None,
            AvoidanceMode::AvoidRobotsAndBall,
        )
        .await?;
    while robot.has_ball() {
        robot.kick();
        sleep(CONTROL_PERIOD).await;
    }
    println!("[INFO] done shooting!");

    Ok(())
}

pub async fn intercept(robot: &Robot, ball: &Ball) {
    let target_orientation = robot.to(ball).angle();
    if ball.get_vel().norm() < 0.4 {
        robot.goto(ball, Some(target_orientation)).await;
        return;
    }
    let ball_trajectory = Line::new(
        ball.get_pos(),
        ball.get_pos() + ball.get_vel().normalized() * 100.,
    );
    let target_pos = ball_trajectory.closest_point_to(robot.get_pos());

    robot.enable_dribbler();
    robot.goto(&target_pos, Some(target_orientation)).await;
    robot.disable_dribbler();
}
