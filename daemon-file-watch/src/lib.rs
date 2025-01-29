use nix::libc;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use nix::sys::signal::{self, SigHandler, Signal};
use notify_rust::Notification;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, process};

// Global flag to detect signals
static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);

extern "C" fn exit_signal_handler(_sig: libc::c_int) {
    println!("Signal received, cleaning up...");
    SIGNAL_RECEIVED.store(true, Ordering::SeqCst);
}

fn setup_signal_handlers() {
    unsafe {
        let handler = SigHandler::Handler(exit_signal_handler);
        signal::signal(Signal::SIGABRT, handler).expect("Failed to set SIGABRT handler");
        signal::signal(Signal::SIGINT, handler).expect("Failed to set SIGINT handler");
        signal::signal(Signal::SIGTERM, handler).expect("Failed to set SIGTERM handler");
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let instance = Inotify::init(InitFlags::empty())?;
    let watcher = instance.add_watch(
        config.path.as_str(),
        AddWatchFlags::IN_CREATE
            | AddWatchFlags::IN_DELETE
            | AddWatchFlags::IN_MODIFY
            | AddWatchFlags::IN_CLOSE_WRITE
            | AddWatchFlags::IN_MOVE_SELF,
    )?;

    setup_signal_handlers();

    loop {
        if SIGNAL_RECEIVED.load(Ordering::SeqCst) {
            println!("Exiting gracefully...");
            let _ = instance.rm_watch(watcher);
            process::exit(0);
        }

        // We read from our inotify instance for events.
        let events = instance.read_events()?;
        let mut message: Option<String> = None;

        for event in events.iter() {
            if event.mask.contains(AddWatchFlags::IN_CREATE) {
                message = Some(String::from("File created!"));
            }
            if event.mask.contains(AddWatchFlags::IN_DELETE) {
                message = Some(String::from("File deleted!"));
            }
            if event.mask.contains(AddWatchFlags::IN_MODIFY) {
                message = Some(String::from("File modified!"));
            }
            if event.mask.contains(AddWatchFlags::IN_CLOSE_WRITE) {
                message = Some(String::from("File witen and closed!"));
            }
            if event.mask.contains(AddWatchFlags::IN_MOVE_SELF) {
                message = Some(String::from("File moved!"));
            }
        }

        if let Some(msg) = message {
            Notification::new()
                .summary("DaemonFSD")
                .body(&msg)
                .icon("dialog-information")
                .show()?;
        }
    }
}

pub struct Config {
    pub path: String,
    pub file_name: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // Skip the first argument which is the binary name.

        let path = args.next().ok_or("Usage: deamonfsd PATH")?;

        let path = fs::symlink_metadata(&path)
            .and_then(|metadata| {
                if metadata.is_symlink() {
                    fs::read_link(&path).and_then(|link| {
                        link.to_str().map(|s| s.to_string()).ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid symlink target",
                            )
                        })
                    })
                } else {
                    Ok(path)
                }
            })
            .map_err(|_| "Error resolving path")?;

        println!("Path: {}", path);
        let file_name = path.split('/').collect::<Vec<&str>>();
        let file_name = file_name[file_name.len() - 1].to_string();

        Ok(Config { path, file_name })
    }
}
