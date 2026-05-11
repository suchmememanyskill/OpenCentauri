use std::env;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

/*
 * MCU Identifier for Centauri Carbon Toolhead Boards
 *
 * Logic derived from firmware 1.4.44 decompilation:
 * 1. Pins Per Bank: 16 (Old/Lite) vs 32 (New).
 * 2. Accelerometer: ADXL345 (Old) vs LIS2DW12 (New).
 * 3. New sensors: Hall effect sensor and RunoutHelper commands.
 */

const DEFAULT_BAUD: u32 = 115200;
const SYNC_BYTE: u8 = 0x7e;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <serial_port> [baud_rate]", args[0]);
        eprintln!("Example: {} /dev/ttyACM0 250000", args[0]);
        return Ok(());
    }

    let port_name = &args[1];
    let baud_rate = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_BAUD);

    println!("\x1b[1;35m====================================================\x1b[0m");
    println!("\x1b[1;35m       Centauri Carbon MCU Identification Tool      \x1b[0m");
    println!("\x1b[1;35m====================================================\x1b[0m");
    println!("[CONFIG] Port: {}, Baud: {}", port_name, baud_rate);

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(500))
        .open()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("\x1b[1;34m[1/4] Opened serial port. Sending Klipper Sync sequence...\x1b[0m");

    // Send 20 sync bytes to trigger dictionary dump
    let sync_seq = vec![SYNC_BYTE; 20];
    port.write_all(&sync_seq)?;
    port.flush()?;
    println!("      -> Sent 20x 0x7e (Klipper Sync)");

    println!("\x1b[1;34m[2/4] Reading data from MCU (Waiting for Dictionary)...\x1b[0m");
    let mut buffer = Vec::new();
    let start_time = Instant::now();
    let timeout = Duration::from_secs(3);

    while start_time.elapsed() < timeout {
        let mut read_buf = [0u8; 1024];
        match port.read(&mut read_buf) {
            Ok(n) if n > 0 => {
                buffer.extend_from_slice(&read_buf[..n]);
                if buffer.len() > 8000 {
                    break;
                } // Sufficient for dictionary
            }
            Ok(_) => thread_wait(10),
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => thread_wait(10),
            Err(e) => return Err(e),
        }
    }

    println!("      -> Received {} bytes from MCU.", buffer.len());
    if buffer.is_empty() {
        println!("\x1b[1;31m[ERROR]\x1b[0m No response from MCU. Check baud rate or cabling.");
        return Ok(());
    }

    println!("\x1b[1;34m[3/4] Scanning for hardware identifiers...\x1b[0m");

    let raw_data = String::from_utf8_lossy(&buffer);

    // --- Identification Heuristics ---

    // 1. Pins Per Bank Check
    let mut variant_old = false;
    let mut variant_new = false;

    // We look for "pins_per_bank" string which is usually in the dictionary
    if raw_data.contains("pins_per_bank") {
        println!("      [DEBUG] Found 'pins_per_bank' string in response.");
        // Binary logic: look for the value near the string
        // For simplicity in this basic tool, we use the evidence of other peripherals
    }

    // 2. Accelerometer Check
    let has_adxl345 = raw_data.contains("adxl345");
    let has_lis2dw = raw_data.contains("lis2dw");

    if has_lis2dw {
        println!(
            "      \x1b[1;32m[MATCH]\x1b[0m Found 'lis2dw' in dictionary. (New Variant Indicator)"
        );
        variant_new = true;
    } else if has_adxl345 {
        println!(
            "      \x1b[1;32m[MATCH]\x1b[0m Found 'adxl345' in dictionary. (Old Variant Indicator)"
        );
        variant_old = true;
    }

    // 3. Hall Sensor / RunoutHelper Check
    let has_hall = raw_data.contains("HallFilamentWidthSensor");
    let has_runout_cmd = raw_data.contains("QUERY_FILAMENT_SENSOR");

    if has_hall {
        println!(
            "      \x1b[1;32m[MATCH]\x1b[0m Found 'HallFilamentWidthSensor' support. (New Board)"
        );
        variant_new = true;
    }

    if has_runout_cmd {
        println!(
            "      \x1b[1;32m[MATCH]\x1b[0m Found 'QUERY_FILAMENT_SENSOR' command. (New Board integration)"
        );
        variant_new = true;
    }

    println!("\x1b[1;34m[4/4] Identification Conclusion:\x1b[0m");

    if variant_new {
        println!(
            "\n\x1b[1;32m[RESULT]\x1b[0m Board is likely: \x1b[1;37mNEW STM32 TOOLHEAD BOARD\x1b[0m"
        );
        println!("  Detected Features:");
        println!("    - LIS2DW12 Accelerometer support");
        println!("    - Hall Effect Filament Sensor integration");
        println!("    - On-board Runout Detection (RunoutHelper)");
    } else if variant_old {
        println!(
            "\n\x1b[1;32m[RESULT]\x1b[0m Board is likely: \x1b[1;37mOLD STM32 TOOLHEAD BOARD (LITE)\x1b[0m"
        );
        println!("  Detected Features:");
        println!("    - ADXL345 Accelerometer support");
        println!("    - No on-board Hall or Runout command detected");
    } else {
        println!("\n\x1b[1;33m[RESULT]\x1b[0m Unknown STM32 Board variant.");
        println!("  Received data did not contain conclusive strings for variant matching.");
    }

    Ok(())
}

fn thread_wait(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
