use docopt::Docopt;
use log::LevelFilter;
use rendy::init::winit::{
    event_loop::EventLoop,
    monitor::{MonitorHandle, VideoMode},
    window::Fullscreen,
};
use serde::Deserialize;
use std::io::{stdin, stdout, Write};

const USAGE: &'static str = "
Usage:
  phantoma [-h] [-v LEVEL] [-fbsi]

Options:
  -h, --help         Show this screen.
  -v LEVEL           Set the log level. Higher numbers increase verbosity.
  -s, --no-vsync     Disable vertical sync to the monitor's refresh rate.
  -f, --fullscreen   Start in fullscreen mode.
  -b, --borderless   Start in fullscreen mode.
  -i, --interactive  Select the fullscreen monitor and video mode interactively.
";

#[derive(Debug, Deserialize)]
pub struct CmdArgs {
    flag_v: u32,
    flag_fullscreen: bool,
    flag_borderless: bool,
    flag_interactive: bool,
    flag_no_vsync: bool,
}

pub struct Args {
    pub vsync: bool,
    pub fullscreen: Option<Fullscreen>,
    pub log_level: LevelFilter,
}

pub fn args(event_loop: &EventLoop<()>) -> Args {
    let cmd: CmdArgs = Docopt::new(USAGE)
        .and_then(|d| d.argv(std::env::args().into_iter()).deserialize())
        .unwrap_or_else(|e| e.exit());

    let monitor = select_monitor(event_loop, cmd.flag_interactive);
    let mode = select_video_mode(&monitor, cmd.flag_interactive && !cmd.flag_borderless);

    let fullscreen = match cmd.flag_fullscreen {
        true => match cmd.flag_borderless {
            true => Some(Fullscreen::Borderless(monitor)),
            false => Some(Fullscreen::Exclusive(mode)),
        },
        false => None,
    };

    let log_level = match cmd.flag_v {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Args {
        vsync: !cmd.flag_no_vsync,
        fullscreen,
        log_level,
    }
}

fn select_monitor(event_loop: &EventLoop<()>, interactive: bool) -> MonitorHandle {
    match interactive {
        false => event_loop.available_monitors().next().unwrap(),
        true => prompt_for_monitor(event_loop),
    }
}

fn prompt_for_monitor(event_loop: &EventLoop<()>) -> MonitorHandle {
    for (num, monitor) in event_loop.available_monitors().enumerate() {
        println!("Monitor #{}: {:?}", num, monitor.name());
    }

    print!("Use Monitor: ");
    stdout().flush().unwrap();

    let mut num = String::new();
    stdin().read_line(&mut num).unwrap();
    let num = num.trim().parse().ok().expect("Please enter a number");
    let monitor = event_loop
        .available_monitors()
        .nth(num)
        .expect("Please select a valid monitor");

    monitor
}

fn select_video_mode(monitor: &MonitorHandle, interactive: bool) -> VideoMode {
    match interactive {
        false => {
            let max_hsz = monitor
                .video_modes()
                .max_by(|x, y| x.size().width.cmp(&y.size().width))
                .unwrap();

            monitor
                .video_modes()
                .filter(|m| m.size().width == max_hsz.size().width)
                .max_by(|x, y| x.refresh_rate().cmp(&y.refresh_rate()))
                .unwrap()
        }
        true => prompt_for_video_mode(monitor),
    }
}

fn prompt_for_video_mode(monitor: &MonitorHandle) -> VideoMode {
    for (i, video_mode) in monitor.video_modes().enumerate() {
        println!("Video mode #{}: {}", i, video_mode);
    }

    print!("Use Video Mode: ");
    stdout().flush().unwrap();

    let mut num = String::new();
    stdin().read_line(&mut num).unwrap();
    let num = num.trim().parse().ok().expect("Please enter a number");
    let video_mode = monitor
        .video_modes()
        .nth(num)
        .expect("Please select a valid mode");

    println!("Using {}", video_mode);

    video_mode
}
