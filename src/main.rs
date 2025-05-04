use evdev::{Device, EventType, KeyCode};
use std::io;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use signal_hook::consts::SIGTERM;
use signal_hook::iterator::Signals;
use thiserror::Error;

#[derive(Error, Debug)]
enum SupipiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Ctrl+C error: {0}")]
    CtrlC(#[from] ctrlc::Error),
}

fn main() -> Result<(), SupipiError> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("Received Ctrl+C, shutting down...");
    })?;

    let mut signals = Signals::new(&[SIGTERM])?;
    let r = running.clone();
    std::thread::spawn(move || {
        for _ in signals.forever() {
            r.store(false, Ordering::SeqCst);
            println!("Received SIGTERM, shutting down...");
        }
    });

    let mut device = Device::open("/dev/input/event3").map_err(|e| {
        eprintln!("Device open error: {:?}", e);
        SupipiError::Io(e)
    })?;
    let timeout = Duration::from_millis(200);
    let mut last_press = Instant::now();
    let mut tap_count = 0;

    println!("Supipi started, listening on event3...");

    while running.load(Ordering::SeqCst) {
        match device.fetch_events() {
            Ok(events) => {
                for ev in events {
                    if ev.event_type() == EventType::KEY && ev.code() == KeyCode::KEY_LEFTMETA.0 {
                        if ev.value() == 1 {
                            let now = Instant::now();
                            if now.duration_since(last_press) < timeout {
                                tap_count += 1;
                                if tap_count == 2 {
                                    match Command::new("wofi").args(["--show", "drun"]).spawn() {
                                        Ok(_) => println!("Launched wofi"),
                                        Err(e) => eprintln!("Failed to launch wofi: {}", e),
                                    }
                                    tap_count = 0;
                                }
                            } else {
                                tap_count = 1;
                            }
                            last_press = now;
                        }
                    }
                }
            }
            Err(e) => eprintln!("Event fetch error: {}", e),
        }
    }

    println!("Closing device...");
    drop(device);
    Ok(())
}