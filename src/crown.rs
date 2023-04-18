use std::process::Command;

pub fn run() {
    Command::new("crown")
        .arg("main.rs")
        .arg("preprocess")
        .arg("in-place")
        .output()
        .expect("failed to run crown main.rs preprocess in-place");

    Command::new("crown")
        .arg("main.rs")
        .arg("explicit-addr")
        .arg("in-place")
        .output()
        .expect("failed to run crown main.rs explicit-addr in-place");

    Command::new("crown")
        .arg("main.rs")
        .arg("rewrite")
        .output()
        .expect("failed to run crown main.rs rewrite");
}
