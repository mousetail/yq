use std::{
    env,
    ffi::{CString, OsString},
    os::unix::ffi::OsStringExt,
    time::Duration,
};

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
            WatchMask::DELETE_SELF
                | WatchMask::CREATE
                | WatchMask::MOVE_SELF
                | WatchMask::ONESHOT
                | WatchMask::OPEN,
        )
        .expect("Failed creating watch");

    let mut buffer = [0; 1024];
    let mut stream = inotify.into_event_stream(&mut buffer).unwrap();

    stream.next().await;
}

fn osstr_to_c_str(osstr: OsString) -> CString {
    CString::new(osstr.into_vec()).unwrap()
}

pub async fn restart() {
    let executable_path = env::current_exe().unwrap();

    std::thread::sleep(Duration::from_millis(100));
    while !executable_path.is_file() {
        std::thread::sleep(Duration::from_millis(100));
    }
    println!("Finished waiting for the executable path to exist");

    use nix::unistd::execve;
    execve::<CString, CString>(
        &osstr_to_c_str(executable_path.into_os_string()),
        &std::env::args_os().map(osstr_to_c_str).collect::<Vec<_>>(),
        &[],
    )
    .unwrap();
}
