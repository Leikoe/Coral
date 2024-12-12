pub mod robot;
pub mod vec2;

use robot::{Robot, RobotId};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use vec2::Vec2f32;

const IS_CLOSE_EPSILON: f32 = 0.1;

#[derive(Debug)]
pub struct RobotCommand {
    vel: Vec2f32,
    kick: bool,
}

// pub struct RobotController {
//     robot_conductor: Arc<RobotConductor>,
//     robot: Arc<Mutex<Robot>>,
// }

// fn cmp(a: f32, b: f32) -> f32 {
//     match a.partial_cmp(&b) {
//         Some(std::cmp::Ordering::Less) => 1.,
//         Some(std::cmp::Ordering::Greater) => -1.,
//         _ => 0.,
//     }
// }

// impl RobotController {
//     pub fn new(robot_conductor: Arc<RobotConductor>, robot: Arc<Mutex<Robot>>) -> Self {
//         Self {
//             robot_conductor,
//             robot,
//         }
//     }

//     pub async fn goto(&self, desired_pos: Vec2f32) {
//         // TODO: some loop that returns once it's one ?
//         let robot = self.robot.clone();
//         let conductor = self.robot_conductor.clone();
//         let notifier = Arc::new(tokio::sync::Notify::new());
//         let notifier_clone = notifier.clone();
//         tokio::spawn(async move {
//             loop {
//                 let robot_pos = { (*robot.lock().unwrap()).get_pos() };
//                 let robot_to_destination = desired_pos - dbg!(robot_pos);
//                 if robot_to_destination.norm() < IS_CLOSE_EPSILON {
//                     notifier_clone.notify_one();
//                 }
//                 let command = RobotCommand {
//                     vel: Vec2f32::new(
//                         cmp(robot_pos.x, desired_pos.x) * 0.15,
//                         cmp(robot_pos.y, desired_pos.y) * 0.15,
//                     ),
//                     kick: false,
//                 };
//                 conductor.set_next_command(robot.clone(), command);
//                 tokio::time::sleep(Duration::from_millis(1000)).await;
//             }
//         });

//         notifier.notified().await;
//     }
// }

// pub struct RobotConductor {
//     next_commands: HashMap<RobotId, RobotCommand>,
// }

// impl RobotConductor {
//     pub fn new() -> Self {
//         Self {
//             next_commands: HashMap::new(),
//         }
//     }

//     pub fn get_robot_controller(self: Arc<Self>, robot_id: RobotId) -> RobotController {
//         let robot = Mutex::new(Robot::new(robot_id, Vec2f32::new(0., 0.)));
//         RobotController::new(self, Arc::new(robot))
//     }

//     pub fn set_next_command(&self, robot: Arc<Mutex<Robot>>, command: RobotCommand) {
//         let mut robot = robot.lock().unwrap();
//         robot.apply_vel(command.vel);
//     }
// }

// pub struct Crabe {
//     robot_conductor: Arc<RobotConductor>,
// }

// pub trait Strategy {
//     async fn run(&self, robot_conductor: Arc<RobotConductor>);
// }

// impl Crabe {
//     pub fn new() -> Self {
//         Self {
//             robot_conductor: Arc::new(RobotConductor::new()),
//         }
//     }

//     pub async fn run_strategy(&mut self, strategy: impl Strategy) {
//         strategy.run(self.robot_conductor.clone()).await; // TODO: maybe some context stuff ?
//     }
// }

// struct SquareStrategy;
// impl Strategy for SquareStrategy {
//     async fn run(&self, robot_conductor: Arc<RobotConductor>) {
//         let robot1 = robot_conductor.get_robot_controller(0);
//         robot1.goto(Vec2f32::new(0., 1.)).await;
//         robot1.goto(Vec2f32::new(1., 1.)).await;
//         robot1.goto(Vec2f32::new(1., 0.)).await;
//         robot1.goto(Vec2f32::new(0., 0.)).await;
//         println!("reached dest!");
//     }
// }

fn get_next_commands_map(
    robots: &mut HashMap<RobotId, Arc<Mutex<Robot>>>,
) -> HashMap<RobotId, Option<RobotCommand>> {
    robots
        .values()
        .map(|r| {
            let mut r = r.lock().unwrap();
            (r.get_id(), r.next_command.take())
        })
        .collect()
}

fn add_to_team(team: &mut HashMap<RobotId, Arc<Mutex<Robot>>>, robot: Robot) {
    team.insert(0, Arc::new(Mutex::new(robot)));
}

fn apply_forward_vel(robot: &mut Robot, amount: f32) {
    robot.next_command = Some(RobotCommand {
        vel: Vec2f32::new(amount, 0.),
        kick: false,
    });
}

#[tokio::main]
async fn main() {
    let mut team = HashMap::<RobotId, Arc<Mutex<Robot>>>::new();
    let robot0 = Robot::new(0, Vec2f32::new(0., 0.));
    add_to_team(&mut team, robot0);

    let mut interval = tokio::time::interval(Duration::from_secs(1));

    // Robot control loop, 1Hz
    loop {
        interval.tick().await; // first tick ticks immediately that's why it's at the beginning

        // give dumb orders
        {
            let mut r0 = team.get(&0).unwrap().lock().unwrap();
            apply_forward_vel(&mut r0, 0.1);
        }

        println!("[DEBUG] sending commands!");
        let next_commands = get_next_commands_map(&mut team);
        dbg!(next_commands); // here we would send that to the physical robots controller
    }
}
