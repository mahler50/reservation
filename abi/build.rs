use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .compile_protos(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();
}
