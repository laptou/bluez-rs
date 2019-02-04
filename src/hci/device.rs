use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use errno::{errno, Errno};
use failure::{Error, Fail};
use libc;

use crate::Address;
use crate::bt;

#[derive(Debug, Fail)]
pub enum DeviceError {
    #[fail(display = "Could not get the device: {}.", err)]
    CouldNotGetDevice { err: Errno },
    #[fail(display = "Could not open the device: {}.", err)]
    CouldNotOpenDevice { err: Errno },
    #[fail(display = "Could not close the device: {}.", err)]
    CouldNotCloseDevice { err: Errno },
}

pub struct Device {
    pub(crate) id: i32,
}

impl Device {
    pub fn default() -> Result<Device, Error> {
        let id = unsafe { bt::hci_get_route(ptr::null_mut()) };

        if id < 0 {
            Err(DeviceError::CouldNotGetDevice { err: errno() }.into())
        } else {
            Ok(Device { id })
        }
    }

    pub fn from_address(addr: &str) -> Result<Device, Error> {
        let addr = CString::new(addr)?;
        let id = unsafe { bt::hci_devid(addr.as_ptr()) };

        if id < 0 {
            Err(DeviceError::CouldNotGetDevice { err: errno() }.into())
        } else {
            Ok(Device { id })
        }
    }

    pub fn open(&self) -> Result<Socket, Error> {
        let handle = unsafe { bt::hci_open_dev(self.id) };
        if handle < 0 {
            Err(DeviceError::CouldNotOpenDevice { err: errno() }.into())
        } else {
            Ok(Socket { handle })
        }
    }

    pub fn close(&self) -> Result<(), Error> {
        let result = unsafe { bt::hci_close_dev(self.id) };
        if result < 0 {
            Err(DeviceError::CouldNotCloseDevice { err: errno() }.into())
        } else {
            Ok(())
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.close().expect("could not close device");
    }
}

pub struct Socket {
    pub(crate) handle: i32,
}

#[derive(Debug, Fail)]
pub enum SocketError {
    #[fail(display = "Could not read device name: {}.", err)]
    CouldNotReadName { err: Errno },
}

impl Socket {
    pub fn get_friendly_name(&self, address: Address, timeout: i32) -> Result<String, Error>
    {
        let mut buf: [c_char; 248] = [0; 248];
        let result = unsafe {
            bt::hci_read_remote_name(
                self.handle,
                &address.to_bdaddr(),
                248,
                buf.as_mut_ptr() as *mut c_char,
                timeout)
        };

        if result < 0 {
            return Err(SocketError::CouldNotReadName { err: errno() }.into());
        }

        Ok(unsafe { CStr::from_ptr(buf.as_ptr()) }.to_owned().into_string()?)
    }

    pub fn close(self) {
        let result = unsafe { libc::close(self.handle) };
        if result != 0 {
            panic!("closing socket failed");
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let result = unsafe { libc::close(self.handle) };
        if result != 0 {
            panic!("closing socket failed");
        }
    }
}