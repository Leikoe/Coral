use std::time::Duration;

use crate::{
    math::{Line, Point2, Reactive, ReactivePoint2Ext, ReactiveVec2Ext, Vec2},
    world::{AllyRobot, AvoidanceMode, Ball, GotoError, World},
};
use tokio::{join, select, time::sleep};

pub async fn strike_alone(world: &World, robot: &AllyRobot, ball: &Ball) {
    let goal = world.get_ennemy_goal_bounding_box().center();
    let ball_to_goal = ball.to(&goal);
    let ball_to_behind_ball = ball_to_goal.normalized().mul(-0.3);

    let behind_ball = ball.plus(ball_to_behind_ball);
    let _ = robot
        .goto(
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
            .goto(
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

pub async fn backwards_strike(world: &World, robot: &AllyRobot, ball: &Ball) {
    println!("backwards_strike()");
    robot.go_get_ball(world, ball).await;
    let _ = robot
        .goto(
            world,
            &Point2::new(3., 1.),
            Some(1.5708),
            AvoidanceMode::None,
        )
        .await;

    let top_goal = Point2::new(world.field.get_field_length() / 2., 0.5);
    let bottom_goal = Point2::new(world.field.get_field_length() / 2., -0.5);
    let goal_line = Line::new(top_goal, bottom_goal);
    let shoot_when_can_score = async {
        loop {
            world.next_update().await;
            let robot_to_ray_horizon = Vec2::new(
                1000. * robot.get_orientation().cos(),
                1000. * robot.get_orientation().sin(),
            );
            let ray = Line::new(robot.get_pos(), robot.get_pos() + robot_to_ray_horizon);

            if let Ok(i) = ray.intersection_lines(&goal_line) {
                if i.y < 0.5 && i.y > -0.5 {
                    // if the intersection to the ennemy side edge of the field is within their goal
                    println!("SHOOT!");
                    robot.kick();
                    break;
                }
            }
        }
    };

    let p = Point2::new(3., -1.);
    let (_, _) = join!(
        robot.goto(world, &p, Some(0.), AvoidanceMode::None),
        shoot_when_can_score
    );
    println!("done");
}

pub async fn place_ball(world: &World, robot: &AllyRobot, ball: &Ball, target_ball_pos: &Point2) {
    robot.go_get_ball(world, ball).await;
    let to_ball = robot.to(ball);
    let angle_to_ball = to_ball.angle();
    // go put the ball down
    let _ = robot
        .goto(
            world,
            &(*target_ball_pos - to_ball.get_reactive()),
            Some(angle_to_ball),
            AvoidanceMode::AvoidRobots,
        )
        .await;
    robot.disable_dribbler();
    sleep(Duration::from_millis(500)).await;

    // step away
    let _ = robot
        .goto(
            world,
            &(*target_ball_pos - robot.to(ball).get_reactive() * 4.),
            Some(angle_to_ball),
            AvoidanceMode::AvoidRobots,
        )
        .await;
}

pub async fn do_square_rrt(world: &World, robot: &AllyRobot) -> Result<(), GotoError> {
    let poses = vec![
        Point2::new(-1., 1.),
        Point2::new(1., 1.),
        Point2::new(1., -1.),
        Point2::new(-1., -1.),
    ];

    for pos in &poses {
        robot
            .goto(world, pos, None, AvoidanceMode::AvoidRobots)
            .await?;
    }
    println!("reached dest!");
    Ok(())
}

pub async fn keep(world: &World, robot: &AllyRobot, ball: &Ball) {
    let top_goal = Point2::new(-world.field.get_field_length() / 2., 0.5);
    let bottom_goal = Point2::new(-world.field.get_field_length() / 2., -0.5);
    let goal_line = Line::new(top_goal, bottom_goal);

    loop {
        let _ = robot
            .goto(
                world,
                &|| Point2::new(-4.0, ball.get_pos().y),
                Some(robot.to(ball).angle()),
                AvoidanceMode::None,
            )
            .await;

        // let ball_to_horizon = ball.get_vel() * 1000.;
        // let ray = Line::new(ball.get_pos(), ball.get_pos() + ball_to_horizon);

        // if
        // robot.goto(world, , angle, avoidance_mode)
    }
}

// TODO: require the fronter to have the ball
pub async fn three_attackers_attack(
    world: &World,
    left_winger: &AllyRobot,
    fronter: &AllyRobot,
    right_winger: &AllyRobot,
) {
    let goal = world.get_ennemy_goal_bounding_box().center();
    let (p1, p2, p3) = (
        Point2::new(2.0, 2.),
        Point2::new(0.5, 0.),
        Point2::new(2.0, -2.),
    );

    // go in pos
    let _ = join!(
        left_winger.goto(
            world,
            &p1,
            Some(p1.to(p2).angle()),
            AvoidanceMode::AvoidRobots,
        ),
        fronter.goto(
            world,
            &p2,
            Some(fronter.to(&goal).angle()),
            AvoidanceMode::AvoidRobots,
        ),
        right_winger.goto(
            world,
            &p3,
            Some(p3.to(p2).angle()),
            AvoidanceMode::AvoidRobots,
        ),
    );

    left_winger.enable_dribbler();
    right_winger.enable_dribbler();

    let chosen_striker = if rand::random::<bool>() {
        left_winger
    } else {
        right_winger
    };

    let _ = fronter.pass_to(world, chosen_striker).await;
    let _ = chosen_striker
        .goto(
            world,
            &chosen_striker.get_pos(),
            Some(chosen_striker.to(&goal).angle()),
            AvoidanceMode::AvoidRobots,
        )
        .await;
    while chosen_striker.has_ball() {
        chosen_striker.kick();
        sleep(Duration::from_secs(1)).await;
    }
}

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
