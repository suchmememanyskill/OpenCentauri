use std::os::fd::{AsRawFd, OwnedFd};

use memmap2::{MmapMut, MmapOptions};
use nix::{fcntl::{open, OFlag}, ioctl_readwrite_bad, sys::stat::Mode};

use crate::{error::ApplicationError, util::{u8_slice_to_string, wrap_ioctl_negative_invalid}};


ioctl_readwrite_bad!(kbuf_mgr_dev_create_buf, 0x100, KBufBufData);
ioctl_readwrite_bad!(kbuf_mgr_dev_destroy_buf, 0x200, KBufBufData);

#[repr(C)]
#[derive(Debug)]
pub struct KBufBufData {
    name: [u8; 32],
    len: u32,
    ktype: u32,
    minor: i32,
    va: u32, // ptr, virtual address that program can directly use.
    pub pa: u32, // ptr, physical address.
}

pub struct UserWrapperBufData {
    pub buf: KBufBufData,
    mgr_fd: OwnedFd,
    map_fd: OwnedFd,
    pub addr: MmapMut,
}

impl Drop for UserWrapperBufData {
    fn drop(&mut self) {
        println!("Dropping UserWrapperBufData, cleaning up kbuf");
        let mgr_fd_raw = self.mgr_fd.as_raw_fd();
        let _ = unsafe { kbuf_mgr_dev_destroy_buf(mgr_fd_raw, &mut self.buf) };
    }
}

impl Default for KBufBufData {
    fn default() -> Self {
        let mut buf = [0u8; 32];
        buf[..4].copy_from_slice(b"test");

        Self {
            name: buf,
            len: 4 * 4096,
            ktype: 1, // KBUF_TYPE_NONCACHE,
            minor: Default::default(),
            va: Default::default(),
            pa: Default::default(),
        }
    }
}

pub fn kbuf_use_new_buf(arm_write_addr: u32) -> Result<UserWrapperBufData, ApplicationError> {
    let mut buf_data = KBufBufData::default(); // Maybe should be dynamic?
    buf_data.pa = arm_write_addr;

    let mgr_fd = open("/dev/kbuf-mgr-0", OFlag::O_RDWR, Mode::empty())?;
    let mgr_fd_raw = mgr_fd.as_raw_fd();

    unsafe { wrap_ioctl_negative_invalid(kbuf_mgr_dev_create_buf(mgr_fd_raw, &mut buf_data))? };

    println!("Created kbuf at physical address 0x{:x}", buf_data.pa);

    let map_dev_path = format!("/dev/kbuf-map-{}-{}", buf_data.minor, u8_slice_to_string(&buf_data.name));

    println!("Mapping kbuf device at path: {}", map_dev_path);

    let map_fd = open(map_dev_path.as_str(), OFlag::O_RDWR, Mode::empty())?;

    let addr = unsafe {
        MmapOptions::new()
            .len(buf_data.len as usize)
            .map_mut(&map_fd)
            .unwrap()
    };

    Ok(UserWrapperBufData {
        buf: buf_data,
        mgr_fd,
        map_fd,
        addr,
    })
}