use std::{fs::{create_dir, remove_file, OpenOptions}, io::{Read, Write}, os::unix::fs::symlink, path::PathBuf, thread, time::Duration};

use clap::Parser;
use serialport::{SerialPort, TTYPort};

#[derive(Parser, Debug)]
#[command(name = "stream-to-serial", about = "Convert a file stream (like /dev/urandom) to a virtual serial device", version = "0.1")]
pub struct Args 
{
    #[arg(required = true)]
    pub device : String,

    #[arg(required = true)]
    pub serial_device : String
}

fn main() {
    println!("Hello, world!");
    let args = Args::parse();

    let (mut master_reader, slave) = TTYPort::pair().expect("Unable to create ptty pair");
    master_reader.set_timeout(Duration::MAX).unwrap();
    let master_writer = master_reader.try_clone_native().unwrap();

    let name = slave.name().unwrap();

    let link_path = PathBuf::from(args.serial_device);
    let _ = remove_file(&link_path);

    symlink(name, &link_path).unwrap();

    println!("Created virtual device {}", link_path.display());

    let stream_reader = OpenOptions::new()
        .read(true)
        .open(&args.device).unwrap();

    let mut stream_writer = stream_reader.try_clone().unwrap();

    //let writer = OpenOptions::new()
    //    .write(true)
    //    .open(&args.device).unwrap();

    
    thread::spawn(move || {
        let mut buff = [0u8; 128];
        let mut reader = stream_reader;
        let mut writer = master_writer;
        loop {
            if let Ok(n) = reader.read(&mut buff) {
                if n > 0 
                {
                    #[cfg(debug_assertions)]
                    {
                        println!("Read {} bytes from device", n);
                    }
                    
                    writer.write_all(&buff[..n]).unwrap();
                }
                
            }
        }
    });

    loop {
        let mut buff = [0u8; 128];

        if let Ok(n) = master_reader.read(&mut buff)
        {
            #[cfg(debug_assertions)]
            {
                println!("Wrote {} bytes to device", n);
            }

            stream_writer.write_all(&buff[..n]).unwrap();
        }
    }
}
