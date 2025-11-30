
use std::{env, fs::{self, File}, io::{self, Read, Write}, path::PathBuf, process::exit, thread::sleep, time::Duration};

use openssl::symm::{Cipher, Crypter, Mode};
use md5::{Md5, Digest};

mod key;
use key::{DECRYPTION_IV, DECRYPTION_KEY};
use zip::ZipArchive;

fn main() {
    sleep(Duration::from_millis(5000));
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        exit(1);
    }

    let in_path = PathBuf::from(&args[1]);
    let out_path = PathBuf::from(&args[2]);
    let out_path_tmp = out_path.with_extension("tmp");

    if !in_path.exists() {
        eprintln!("Input file does not exist: {:?}", in_path);
        exit(1);
    }

    if out_path.exists() {
        eprintln!("Output file already exists: {:?}", out_path);
        exit(1);
    }

    let mut in_file = File::open(&in_path).expect("Failed to open input file");
    let mut out_file = File::create(&out_path).expect("Failed to create output file");
    let mut out_file_tmp = File::create(&out_path_tmp).expect("Failed to create temporary output file");

    let mut header = [0u8; 0x20];

    in_file.read_exact(&mut header[..]).expect("Failed to read header");

    let expected_md5 = &header[0x10..0x20];

    let mut hasher = Md5::new();
    let mut decrypter = Crypter::new(
         Cipher::aes_256_cbc(),
         Mode::Decrypt,
        &DECRYPTION_KEY,
        Some(&DECRYPTION_IV),
    ).expect("Failed to setup crypto");

    decrypter.pad(false);
    
    let mut buffer = [0u8; 4096];
    let mut out_buffer = [0u8; 8192];

    loop {
        let len = in_file.read(&mut buffer).expect("Failed to read from input file");
        hasher.update(&buffer[..len]);

        if len == 0 {
            let out_len = decrypter.finalize(&mut out_buffer).expect("Failed to finalize decryption");
            if out_len > 0 {
                out_file_tmp.write_all(&out_buffer[..out_len]).expect("Failed to write to output file");
            }
            break;
        }

        let out_len = decrypter.update(&buffer[..len], &mut out_buffer).expect("Decryption failed");
        out_file_tmp.write_all(&out_buffer[..out_len]).expect("Failed to write to output file");
    }

    let result_md5 = hasher.finalize();

    if result_md5[..] != expected_md5[..] {
        eprintln!("MD5 checksum mismatch! File may be corrupted.");
        exit(1);
    }

    drop(out_file_tmp);
    let out_file_tmp = File::open(&out_path_tmp).expect("Failed to open temporary output file");

    let mut archive = ZipArchive::new(out_file_tmp).expect("Failed to read zip archive");
    let mut first_file = archive.by_index(0).expect("Failed to read first zip entry");

    io::copy(&mut first_file, &mut out_file).expect("Failed to extract file from zip archive");
    sleep(Duration::from_millis(5000));

    drop(first_file);
    drop(archive);
    fs::remove_file(out_path_tmp).expect("Failed to remove temporary file");
}
