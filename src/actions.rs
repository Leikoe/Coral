use crate::{trackable::Trackable, Ball, Line, Point2, Robot, CONTROL_PERIOD};
use std::{
    future::{join, IntoFuture},
    process::exit,
    time::Duration,
};

/// Create a Future which does f() once cond() is true. (Check cond() at check_interval_period intervals)
async fn once<C: Fn() -> bool, F: IntoFuture>(cond: C, f: F, check_interval_period: Duration) {
    let mut inteval = tokio::time::interval(check_interval_period);
    loop {
        inteval.tick().await;
        if cond() {
            f.into_future().await;
            break;
        }
    }
}

pub async fn do_square(robot: &Robot) {
    robot.goto(&Point2::new(0., 1.), None).await;
    robot.goto(&Point2::new(1., 1.), None).await;
    robot.goto(&Point2::new(1., 0.), None).await;
    robot.goto(&Point2::new(0., 0.), None).await;
    println!("reached dest!");
}

pub async fn three_attackers_attack(left_winger: &Robot, fronter: &Robot, right_winger: &Robot) {
    join!(
        left_winger.goto(&Point2::new(0.5, 0.5), None),
        right_winger.goto(&Point2::new(0.5, -0.5), None),
        fronter.goto(&Point2::new(0.5, 0.), None),
    )
    .await;
}

pub async fn go_get_ball(robot: &Robot, ball: &Ball) {
    robot.enable_dribbler();
    robot.goto(ball, None).await; // this will follow the ball even if it moves
    println!("got ball!");
}

pub async fn attak(robot: &Robot, ball: &Ball) {
    robot.goto(ball, None).await;
    // robot.kick();
}

pub async fn intercept(robot: &Robot, ball: &Ball) {
    let target_orientation = robot.to(ball).angle();
    if ball.get_vel().norm() < 0.4 {
        robot.goto(&ball.get_pos(), Some(target_orientation)).await;
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
