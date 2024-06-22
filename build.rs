extern crate embed_resource;
use std::env;
use vergen_git2::{Emitter, Git2Builder};

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        // on windows we will set our game icon as icon for the executable
        embed_resource::compile("build/windows/icon.rc");
    }

    let git = Git2Builder::all_git().unwrap();
    Emitter::default()
        .add_instructions(&git)
        .unwrap()
        .emit()
        .unwrap();
}
