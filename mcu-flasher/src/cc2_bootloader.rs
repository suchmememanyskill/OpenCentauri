use std::{
    fmt, io,
    time::{Duration, Instant},
};

use serialport::SerialPort;

const FRAME_MAGIC: [u8; 2] = [0xa5, 0x5a];
const CMD_PING: u8 = 0x03;
const CMD_JUMP_TO_APP: u8 = 0x02;
const PING_VALUE: u32 = 0x1234_5678;
const MAX_PAYLOAD_LENGTH: usize = 1024;
const PING_ATTEMPTS: usize = 6;

#[derive(Debug, PartialEq, Eq)]
pub enum BootResult {
    JumpedToApp,
    AlreadyInApp,
}

#[derive(Debug)]
pub enum Error {
    Serial(serialport::Error),
    Io(io::Error),
    ShortWrite { expected: usize, actual: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serial(error) => write!(f, "serial error: {error}"),
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::ShortWrite { expected, actual } => {
                write!(f, "short write: expected {expected} bytes, wrote {actual}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<serialport::Error> for Error {
    fn from(error: serialport::Error) -> Self {
        Self::Serial(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn boot(port: &mut dyn SerialPort, timeout_seconds: u32) -> Result<BootResult, Error> {
    let timeout = Duration::from_secs(timeout_seconds as u64);

    for _ in 0..PING_ATTEMPTS {
        if ping(port, timeout)? {
            jump_to_app(port)?;
            return Ok(BootResult::JumpedToApp);
        }
    }

    Ok(BootResult::AlreadyInApp)
}

fn ping(port: &mut dyn SerialPort, timeout: Duration) -> Result<bool, Error> {
    write_packet(port, CMD_PING, &PING_VALUE.to_le_bytes())?;

    let deadline = Instant::now() + timeout;
    let mut received = Vec::new();
    let mut chunk = [0u8; 128];

    while Instant::now() < deadline {
        match port.read(&mut chunk) {
            Ok(count) => {
                received.extend_from_slice(&chunk[..count]);
                if let Some(result) = parse_ping_response(&received) {
                    return Ok(result);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(false)
}

fn jump_to_app(port: &mut dyn SerialPort) -> Result<(), Error> {
    write_packet(port, CMD_JUMP_TO_APP, &[])
}

fn write_packet(port: &mut dyn SerialPort, command: u8, payload: &[u8]) -> Result<(), Error> {
    let packet = packet(command, payload);
    let actual = std::io::Write::write(port, &packet)?;
    if actual != packet.len() {
        return Err(Error::ShortWrite {
            expected: packet.len(),
            actual,
        });
    }
    std::io::Write::flush(port)?;
    Ok(())
}

fn packet(command: u8, payload: &[u8]) -> Vec<u8> {
    let length = u16::try_from(payload.len()).expect("CC2 payload is too large");
    let mut packet = Vec::with_capacity(7 + payload.len());
    packet.extend_from_slice(&FRAME_MAGIC);
    packet.push(command);
    packet.extend_from_slice(&length.to_be_bytes());
    packet.extend_from_slice(payload);
    packet.extend_from_slice(&crc16_ccitt(payload).to_be_bytes());
    packet
}

fn parse_ping_response(data: &[u8]) -> Option<bool> {
    let start = data
        .windows(FRAME_MAGIC.len())
        .position(|window| window == FRAME_MAGIC)?;
    if data.len() < start + 7 {
        return None;
    }

    let command = data[start + 2];
    let length = u16::from_be_bytes([data[start + 3], data[start + 4]]) as usize;
    if length > MAX_PAYLOAD_LENGTH {
        return None;
    }

    let end = start + 5 + length + 2;
    if data.len() < end || command != CMD_PING {
        return None;
    }

    let payload = &data[start + 5..start + 5 + length];
    let crc_offset = start + 5 + length;
    let received_crc = u16::from_be_bytes([data[crc_offset], data[crc_offset + 1]]);
    if crc16_ccitt(payload) != received_crc || payload.len() != 4 {
        return None;
    }

    Some(u32::from_le_bytes(payload.try_into().unwrap()) == PING_VALUE)
}

fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc = 0xffff;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ping_packet_with_little_endian_payload() {
        assert_eq!(
            packet(CMD_PING, &PING_VALUE.to_le_bytes()),
            vec![0xa5, 0x5a, 0x03, 0x00, 0x04, 0x78, 0x56, 0x34, 0x12, 0x54, 0x3a]
        );
    }

    #[test]
    fn builds_jump_to_app_packet() {
        assert_eq!(
            packet(CMD_JUMP_TO_APP, &[]),
            vec![0xa5, 0x5a, 0x02, 0x00, 0x00, 0xff, 0xff]
        );
    }

    #[test]
    fn accepts_valid_ping_response() {
        let response = packet(CMD_PING, &PING_VALUE.to_le_bytes());
        assert_eq!(parse_ping_response(&response), Some(true));
    }

    #[test]
    fn rejects_wrong_ping_value() {
        let response = packet(CMD_PING, &0xdead_beefu32.to_le_bytes());
        assert_eq!(parse_ping_response(&response), Some(false));
    }

    #[test]
    fn rejects_bad_crc() {
        let mut response = packet(CMD_PING, &PING_VALUE.to_le_bytes());
        let last = response.len() - 1;
        response[last] ^= 1;
        assert_eq!(parse_ping_response(&response), None);
    }

    #[test]
    fn ignores_leading_bytes() {
        let mut response = vec![0, 1, 2];
        response.extend_from_slice(&packet(CMD_PING, &PING_VALUE.to_le_bytes()));
        assert_eq!(parse_ping_response(&response), Some(true));
    }
}
