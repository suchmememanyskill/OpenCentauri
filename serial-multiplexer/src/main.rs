use clap::Parser;
use serialport::{SerialPort, TTYPort};
use std::{
    collections::HashMap, fs::{self, create_dir, File}, hash::Hash, io::{Read, Write}, os::unix::fs::symlink, path::PathBuf, process::exit, thread, time::Duration
};

use crate::config::{Args, SerialEntry, SerialEntryRaw};
mod config;

fn main() {
    println!("Hello, world!");
    let args = Args::parse();
    if (!args.with_virtual_ports && !args.with_real_ports) || (args.with_virtual_ports && args.with_real_ports) {
        eprintln!("You must specify either --receive or --send");
        exit(1);
    }

    let config_path = PathBuf::from(&args.config);
    if !config_path.exists() {
        eprintln!("Config file does not exist: {}", config_path.display());
        exit(2);
    }

    let config = fs::read_to_string(&config_path).unwrap();
    let serial_ports_raw: HashMap<String, SerialEntryRaw> = toml::from_str(&config).unwrap();
    if serial_ports_raw.is_empty() {
        eprintln!("No serial ports found in the config file.");
        exit(3);
    }

    let mut multiplexed_port = match serialport::new(&args.device, args.baud).open_native() {
        Ok(port) => port,
        Err(e) => {
            eprintln!(
                "Failed to open multiplexed serial port {}: {}",
                args.device, e
            );
            exit(5);
        }
    };

    if args.with_real_ports {
        let mut serial_ports: HashMap<u32, SerialEntry> = serial_ports_raw
            .iter()
            .map(|f| {
                let entry = f.1;
                (
                    entry.id as u32,
                    config::SerialEntry {
                        name: f.0.clone(),
                        device: match serialport::new(&entry.device_path, entry.baud_rate).open_native() {
                            Ok(port) => port,
                            Err(e) => {
                                eprintln!("Failed to open serial port {}: {}", entry.device_path, e);
                                exit(4);
                            }
                        },
                        id: entry.id,
                    },
                )
            })
            .collect();

        communicate(&mut multiplexed_port, &mut serial_ports);
    }
    else {
        let mut serial_ports : HashMap<u32, SerialEntry> = serial_ports_raw
            .iter()
            .map(|f| {
                let entry = f.1;
                let (mut master, mut slave) = TTYPort::pair().expect("Unable to create ptty pair");

                let name = slave.name().unwrap();

                let mut link_path = std::env::home_dir().unwrap_or(PathBuf::from("/dev"));

                link_path.push("vtty");
                if !link_path.exists()
                {
                    create_dir(&link_path).unwrap();
                }

                link_path.push(f.0);

                symlink(name, link_path).unwrap();

                (
                    entry.id as u32,
                    config::SerialEntry {
                        name: f.0.clone(),
                        id: entry.id,
                        device: master,
                    }
                )
            })
            .collect();

        communicate(&mut multiplexed_port, &mut serial_ports);
    }
}

fn communicate(
    multiplexed_port: &mut TTYPort,
    serial_ports: &mut HashMap<u32, SerialEntry>,
) {
    let mut multiplexed_port_clone = multiplexed_port.try_clone_native().unwrap();

    let mut serial_ports_clone: HashMap<u32, SerialEntry> = HashMap::new();

    serial_ports.iter().for_each(|port| {
        serial_ports_clone.insert(
            port.0.clone(),
            SerialEntry { 
                name: port.1.name.clone(), 
                device: port.1.device.try_clone_native().unwrap(), 
                id: port.1.id
            },
        );
    });

    std::thread::spawn(move || {
        let mut local_map = serial_ports_clone;

        loop {
            local_map.iter_mut().for_each(|port| {
                let mut buff = [0u8; 255];
                loop {
                    if let Ok(bytes_read) = port.1.device.read(&mut buff) {
                        if bytes_read > 0 {
                            let mut mini_buff = [0u8; 2];
                            mini_buff[0] = port.0.clone() as u8;
                            mini_buff[1] = bytes_read as u8;

                            if multiplexed_port_clone.write(&mini_buff).is_ok()
                            {
                                let _ = multiplexed_port_clone.write_all(&buff[..bytes_read]);
                            }
                            else 
                            {
                                println!("Connection to {} lost, retrying in 1000ms", multiplexed_port_clone.name().unwrap());
                                thread::sleep(Duration::from_secs(1u64));
                            }
                        }
                    }
                }
            });
        }
    });

    loop {
        let mut mini_buff = [0u8; 2];
        if multiplexed_port.read_exact(&mut mini_buff).is_ok() {
            let id = mini_buff[0];
            let length = mini_buff[1] as usize;

            let mut buff = vec![0u8; length];
            if multiplexed_port.read_exact(&mut buff).is_ok() {
                let port = serial_ports.get_mut(&(id as u32)).unwrap();
                port.device.write_all(&buff).unwrap();
            }
        }
    }
}
