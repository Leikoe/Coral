use std::path::{Path, PathBuf};

fn compile_packet(filename: &str, protos: &[impl AsRef<Path>], includes: &[impl AsRef<Path>]) {
    let mut build = prost_build::Config::new();
    build
        .default_package_filename(filename)
        .out_dir(PathBuf::from("src/league_protocols"))
        .compile_protos(protos, includes)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to compile {} protobuf files (reason: {})",
                filename, e
            )
        });
}

fn main() {
    compile_packet(
        "simulation_packet",
        &[
            "league_protocols_definitions/simulation/ssl_simulation_control.proto",
            "league_protocols_definitions/simulation/ssl_simulation_robot_control.proto",
            "league_protocols_definitions/simulation/ssl_simulation_robot_feedback.proto",
        ],
        &["league_protocols_definitions/simulation/"],
    );

    compile_packet(
        "vision_packet",
        &["league_protocols_definitions/vision/messages_robocup_ssl_wrapper.proto"],
        &["league_protocols_definitions/vision"],
    );

    compile_packet(
        "game_controller_packet",
        &["league_protocols_definitions/game_controller/ssl_gc_referee_message.proto"],
        &["league_protocols_definitions/game_controller"],
    );

    compile_packet(
        "robot_packet",
        &["league_protocols_definitions/robot/base_wrapper.proto"],
        &["league_protocols_definitions/robot"],
    );
}
