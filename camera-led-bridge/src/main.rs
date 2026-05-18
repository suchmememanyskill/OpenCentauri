use std::fs;
use std::io;
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

use nix::ioctl_readwrite;
use serde::Deserialize;

const V4L2_CTRL_CLASS_USER: u32 = 0x00980000;
const V4L2_CID_BASE: u32 = V4L2_CTRL_CLASS_USER | 0x900;
const V4L2_CID_BACKLIGHT_COMPENSATION: u32 = V4L2_CID_BASE + 28;

#[repr(C)]
struct V4l2Control {
    id: u32,
    value: i32,
}

// VIDIOC_S_CTRL = _IOWR('V', 28, struct v4l2_control)
ioctl_readwrite!(vidioc_s_ctrl, b'V', 28, V4l2Control);

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const LED_QUERY_URL: &str = "http://localhost/printer/objects/query?led%20case";

#[derive(Deserialize)]
struct QueryResponse {
    result: QueryResult,
}

#[derive(Deserialize)]
struct QueryResult {
    status: QueryStatus,
}

#[derive(Deserialize)]
struct QueryStatus {
    #[serde(rename = "led case")]
    led_case: LedCase,
}

#[derive(Deserialize)]
struct LedCase {
    color_data: Vec<Vec<f64>>,
}

/// Open /dev/video0 and set V4L2_CID_BACKLIGHT_COMPENSATION to `light_state`.
fn camera_control_light(light_state: bool) -> io::Result<()> {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/video0")
        .map_err(|e| {
            eprintln!("open camera failed: {}", e);
            e
        })?;

    let mut control = V4l2Control {
        id: V4L2_CID_BACKLIGHT_COMPENSATION,
        value: light_state as i32,
    };

    unsafe { vidioc_s_ctrl(file.as_raw_fd(), &mut control) }.map_err(|e| {
        let err = io::Error::from(e);
        eprintln!("camera control light error: {}", err);
        err
    })?;

    Ok(())
}

/// Query the LED state from the Moonraker API.
/// Returns `true` if any channel in color_data is above 0.
fn query_led_state() -> Result<bool, Box<dyn std::error::Error>> {
    let response: QueryResponse = ureq::get(LED_QUERY_URL)
        .call()?
        .body_mut()
        .read_json()?;

    let lit = response
        .result
        .status
        .led_case
        .color_data
        .iter()
        .flatten()
        .any(|&v| v > 0.0);
    Ok(lit)
}

fn main() {
    let mut last_state = false;

    loop {
        thread::sleep(POLL_INTERVAL);

        let current_state = match query_led_state() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to query LED state: {}", e);
                continue;
            }
        };

        if current_state != last_state {
            println!("LED state changed: {} -> {}", last_state, current_state);
            last_state = current_state;
            if let Err(e) = camera_control_light(current_state) {
                eprintln!("Failed to control camera light: {}", e);
            }
        }
    }
}
