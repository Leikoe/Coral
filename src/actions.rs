use crate::{trackable::Trackable, Ball, Line, Point2, Robot};
use std::future::join;

pub async fn do_square(robot: &Robot) {
    robot.goto(&Point2::new(0., 1.)).await;
    robot.goto(&Point2::new(1., 1.)).await;
    robot.goto(&Point2::new(1., 0.)).await;
    robot.goto(&Point2::new(0., 0.)).await;
    println!("reached dest!");
}

pub async fn three_attackers_attack(left_winger: &Robot, fronter: &Robot, right_winger: &Robot) {
    join!(
        left_winger.goto(&Point2::new(0.5, 0.5)),
        right_winger.goto(&Point2::new(0.5, -0.5)),
        fronter.goto(&Point2::new(0.5, 0.)),
    )
    .await;
}

pub async fn go_get_ball(robot: &Robot, ball: &Ball) {
    robot.enable_dribbler();
    robot.goto(ball).await; // this will follow the ball even if it moves
    println!("got ball!");
}

pub async fn intercept(robot: Robot, ball: Ball) {
    let _orientation = (ball.get_pos() - robot.get_pos()).angle();
    if ball.get_vel().norm() < 0.4 {
        robot.goto(&ball.get_pos()).await;
        return;
    }
    let trajectory = Line::new(
        ball.get_pos(),
        ball.get_pos() + ball.get_vel().normalized() * 100.,
    );
    let target = trajectory.closest_point_to(robot.get_pos());
    if robot.get_pos().distance_to(ball.get_pos()) < 0.2 {
        robot.enable_dribbler();
    }
    robot.goto(&target).await; // TODO: use orientation
    robot.disable_dribbler();
}
