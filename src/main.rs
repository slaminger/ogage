extern crate evdev_rs as evdev;
extern crate mio;

use evdev::*;
use evdev::enums::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::os::unix::io::AsRawFd;
use mio::{Poll,Events,Token,Interest};
use mio::unix::SourceFd;

static HOTKEY:      EventCode = EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY5);
static BRIGHT_UP:   EventCode = EventCode::EV_KEY(EV_KEY::BTN_DPAD_UP);
static BRIGHT_DOWN: EventCode = EventCode::EV_KEY(EV_KEY::BTN_DPAD_DOWN);
static VOL_UP:      EventCode = EventCode::EV_KEY(EV_KEY::BTN_DPAD_RIGHT);
static VOL_DOWN:    EventCode = EventCode::EV_KEY(EV_KEY::BTN_DPAD_LEFT);
static PERF_MAX:    EventCode = EventCode::EV_KEY(EV_KEY::BTN_TR);
static PERF_NORM:   EventCode = EventCode::EV_KEY(EV_KEY::BTN_TL);
static DARK_ON:     EventCode = EventCode::EV_KEY(EV_KEY::BTN_TR2);
static DARK_OFF:    EventCode = EventCode::EV_KEY(EV_KEY::BTN_TL2);

fn process_event(_dev: &Device, ev: &InputEvent, hotkey: bool) {
//    println!("Event: time {}.{} type {} code {} value {} hotkey {}",
//             ev.time.tv_sec,
//             ev.time.tv_usec,
//             ev.event_type,
//             ev.event_code,
//             ev.value,
//             hotkey);

    if hotkey && ev.value == 1 {
        if ev.event_code == BRIGHT_UP {
            Command::new("light").args(&["-T","1.1"]).output().expect("Failed to execute light");
            Command::new("light").arg("-O").output().expect("Failed to execute light");
        }
        else if ev.event_code == BRIGHT_DOWN {
            Command::new("light").args(&["-T","0.9"]).output().expect("Failed to execute light");
            Command::new("light").arg("-O").output().expect("Failed to execute light");
        }
        else if ev.event_code == VOL_UP {
            Command::new("amixer").args(&["-q", "sset", "Playback", "1%+"]).output().expect("Failed to execute amixer");
        }
        else if ev.event_code == VOL_DOWN {
            Command::new("amixer").args(&["-q", "sset", "Playback", "1%-"]).output().expect("Failed to execute amixer");
        }
        else if ev.event_code == PERF_MAX {
            Command::new("performance").arg("on").output().expect("Failed to execute performance");
        }
        else if ev.event_code == PERF_NORM {
            Command::new("performance").arg("off").output().expect("Failed to execute performance");
        }
        else if ev.event_code == EventCode::EV_KEY(EV_KEY::KEY_POWER) {
            Command::new("sudo").args(&["shutdown", "-h", "now"]).output().expect("Failed to execute power off");
        }
        else if ev.event_code == DARK_ON {
            Command::new("light").arg("-O").output().expect("Failed to execute light");
            Command::new("light").args(&["-S", "1"]).output().expect("Failed to execute light");
            Command::new("dark").arg("on").output().expect("Failed to execute dark");
            Command::new("light").arg("-I").output().expect("Failed to execute light");
        }
        else if ev.event_code == DARK_OFF {
            Command::new("light").arg("-O").output().expect("Failed to execute light");
            Command::new("light").args(&["-S", "100"]).output().expect("Failed to execute light");
            Command::new("dark").arg("off").output().expect("Failed to execute dark");
            Command::new("light").arg("-I").output().expect("Failed to execute light");
        }
    }
    else if ev.event_code == EventCode::EV_SW(EV_SW::SW_HEADPHONE_INSERT) {
        let dest = match ev.value { 1 => "SPK", _ => "HP" };
        Command::new("amixer").args(&["-q", "sset", "'Playback Path'", dest]).output().expect("Failed to execute amixer");
    }
    else if ev.event_code == EventCode::EV_KEY(EV_KEY::KEY_POWER) && ev.value == 1 {
        Command::new("sudo").args(&["zzz"]).output().expect("Failed to execute suspend");
    }
}

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1);
    let mut devs: Vec<Device> = Vec::new();
    let mut hotkey = false;

    for (i, s) in ["/dev/input/event2", "/dev/input/event0", "/dev/input/event1"].iter().enumerate() {
        let fd = File::open(Path::new(s)).unwrap();
        let mut dev = Device::new().unwrap();
        poll.registry().register(&mut SourceFd(&fd.as_raw_fd()), Token(i), Interest::READABLE)?;
        dev.set_fd(fd)?;
        devs.push(dev);
    }

    Command::new("light").arg("-I").output().expect("Failed to execute light");

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            let dev = &mut devs[event.token().0];
            while dev.has_event_pending() {
                let e = dev.next_event(evdev_rs::ReadFlag::NORMAL);
                match e {
                    Ok(k) => {
                        let ev = &k.1;
                        if ev.event_code == HOTKEY {
                            hotkey = ev.value == 1;
                            let grab = if hotkey { GrabMode::Grab } else { GrabMode::Ungrab };
                            dev.grab(grab)?;
                        }
                        process_event(&dev, &ev, hotkey)
                    },
                    _ => ()
                }
            }
        }
    }
}
