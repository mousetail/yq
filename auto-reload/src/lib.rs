use std::env;

use futures_util::StreamExt;
use inotify::{Inotify, WatchMask};

pub async fn wait_reload() {
    let inotify = Inotify::init().expect("Failed to initialize inotify");

    let executable_path = env::current_exe().unwrap();
    println!("Watching {executable_path:?}");

    inotify
        .watches()
        .add(
            executable_path,
            WatchMask::DELETE_SELF | WatchMask::CREATE | WatchMask::MOVE_SELF | WatchMask::ONESHOT,
        )
        .expect("Failed creating watch");

    let mut buffer = [0; 1024];
    let mut stream = inotify.into_event_stream(&mut buffer).unwrap();

    stream.next().await;
}
