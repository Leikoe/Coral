use crate::{Robot, Vec2f32};
use std::future::join;

pub async fn do_square(robot: &Robot) {
    robot.goto(Vec2f32::new(0., 1.)).await;
    robot.goto(Vec2f32::new(1., 1.)).await;
    robot.goto(Vec2f32::new(1., 0.)).await;
    robot.goto(Vec2f32::new(0., 0.)).await;
    println!("reached dest!");
}

pub async fn three_attackers_attack(left_winger: &Robot, fronter: &Robot, right_winger: &Robot) {
    join!(
        left_winger.goto(Vec2f32::new(0.5, 0.5)),
        right_winger.goto(Vec2f32::new(0.5, -0.5)),
        fronter.goto(Vec2f32::new(0.5, 0.)),
    )
    .await;
}

pub async fn go_get_ball(robot: &Robot, ball_pos: Vec2f32) {
    // TODO: ball should be a ref to a ball with internal mutable state
    robot.enable_dribbler();
    robot.goto(ball_pos).await;
    println!("got ball!");
}
