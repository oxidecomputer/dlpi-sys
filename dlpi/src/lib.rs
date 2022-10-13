//! This is a crate for interacting with layer-2 network devices through DLPI.
//! For more details on DLPI see `man dlpi` and `man libdlpi`.
//!
//! ```no_run
//! use std::io::Result;
//! use std::thread::spawn;
//!
//! fn main() -> Result<()> {
//!     // L2 multicast address to send packets to
//!     let mc = [0xff, 0xff, 0x00, 0x00, 0x00, 0x47];
//!
//!     // Open up an interface called sim0 and attach to the ethertype 0x4000
//!     let dh_recv = dlpi::open("sim0", 0).expect("open recv");
//!     dlpi::bind(dh_recv, 0x4000).expect("bind recv");
//!     dlpi::enable_multicast(dh_recv, &mc).expect("enable multicast");
//!
//!     // strt a receiver thread
//!     let t = spawn(move || {
//!
//!         // allocated buffers for a senders L2 address and a message
//!         let mut src = [0u8; dlpi::sys::DLPI_PHYSADDR_MAX];
//!         let mut msg = [0; 256];
//!
//!         // wait for a message
//!         let n = match dlpi::recv(dh_recv, &mut src, &mut msg, -1, None) {
//!             Ok((_, len)) => len,
//!             Err(e) => panic!("recv: {}", e),
//!         };
//!
//!     });
//!
//!     // Open up an interface called sim1 and attach to ethertype 0x4000
//!     let dh_send = dlpi::open("sim1", 0).expect("send");
//!     dlpi::bind(dh_send, 0x4000).expect("bind");
//!
//!     // Send a message
//!     let message = b"do you know the muffin man?";
//!     dlpi::send(dh_send, &mc, &message[..], None).expect("send");
//!
//!     // wait on the receiver and then shut down
//!     t.join().expect("join recv thread");
//!     dlpi::close(dh_send);
//!     dlpi::close(dh_recv);
//!
//!     Ok(())
//! }
//! ```

use std::future::Future;
use std::io::{Error, ErrorKind, Result};
use std::os::raw::{c_char, c_void};
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll};
use num_enum::TryFromPrimitive;
use thiserror::Error;

pub use libdlpi_sys as sys;

/// Result of a DLPI operation.
#[repr(i32)]
#[derive(PartialEq, Eq, Error, Debug, Copy, Clone, TryFromPrimitive)]
pub enum ResultCode {
    #[error("success")]
    Success = 10000, /* DLPI operation succeeded */
    #[error("invalid argument")]
    EInval, /* invalid argument */
    #[error("invalid link name")]
    ELinkNameInval, /* invalid DLPI linkname */
    #[error("link does not exist")]
    ENoLink, /* DLPI link does not exist */
    #[error("bad link")]
    EBadLink, /* bad DLPI link */
    #[error("invalid handle")]
    EInHandle, /* invalid DLPI handle */
    #[error("operation timed out")]
    ETimedout, /* DLPI operation timed out */
    #[error("unsupported version")]
    EVerNotSup, /* unsupported DLPI Version */
    #[error("unsupported connection mode")]
    EModeNotSup, /* unsupported DLPI connection mode */
    #[error("unavailable service access point")]
    EUnavailSAP, /* unavailable DLPI SAP */
    #[error("failure")]
    Failure, /* DLPI operation failed */
    #[error("style-2 node reports style-1")]
    ENotStyle2, /* DLPI style-2 node reports style-1 */
    #[error("bad message")]
    EBadMsg, /* bad DLPI message */
    #[error("raw mode not supported")]
    ERawNotSup, /* DLPI raw mode not supported */
    #[error("invalid notification type")]
    ENoteInval, /* invalid DLPI notification type */
    #[error("notification not supported by link")]
    ENoteNotSup, /* DLPI notification not supported by link */
    #[error("invalid notification id")]
    ENoteIdInval, /* invalid DLPI notification id */
    #[error("ipnetinfo not supported")]
    EIpNetInfoNotSup, /* DLPI_IPNETINFO not supported */
    #[error("error max")]
    ErrMax, /* Highest + 1 libdlpi error code */
}

/// A DLPI handle wrapper that implements `Send` and `Sync`.
#[derive(Clone, Copy)]
pub struct DlpiHandle(pub *mut sys::dlpi_handle_t);
unsafe impl Send for DlpiHandle {}
unsafe impl Sync for DlpiHandle {}

/// A wrapper for DlpiHandle that closes the DLPI instance when dropped.
pub struct DropHandle(pub DlpiHandle);
impl Drop for DropHandle {
    /// Closes underlying DLPI instance.
    fn drop(&mut self) {
        close(self.0);
    }
}

impl DropHandle {
    /// Get the filesystem descriptor associated with this drop handle.
    pub fn fd(&self) -> Result<i32> {
        fd(self.0)
    }
}

/// Creates a DLPI link instance.
pub fn open(linkname: impl AsRef<str>, flags: u32) -> Result<DlpiHandle> {
    let linkname = format!("{}\0", linkname.as_ref());

    let mut dhp = sys::null_dlpi_handle();
    let ret = unsafe {
        sys::dlpi_open(
            linkname.as_str().as_ptr() as *const c_char,
            &mut dhp,
            flags,
        )
    };
    check_return(ret)?;
    Ok(DlpiHandle(dhp))
}

/// Send a message over a DLPI link.
pub fn send(
    h: DlpiHandle,
    dst: &[u8],
    msg: &[u8],
    info: Option<&sys::dlpi_sendinfo_t>,
) -> Result<()> {
    let ret = unsafe {
        sys::dlpi_send(
            h.0,
            dst.as_ptr() as *const c_void,
            dst.len(),
            msg.as_ptr() as *const c_void,
            msg.len(),
            match info {
                Some(info) => info as *const sys::dlpi_sendinfo_t,
                None => ptr::null(),
            },
        )
    };

    check_return(ret)?;
    Ok(())
}

/// Receive a message from a DLPI link.
///
/// Data is placed into provided buffer.  Return values is (address bytes read,
/// message bytes read).
///
/// If no message is received within `msec` milliseconds, returns
/// [`ResultCode::ETimedout`].
///
/// **`src` must be at least [`sys::DLPI_PHYSADDR_MAX`] in length**.
pub fn recv(
    h: DlpiHandle,
    src: &mut [u8],
    msg: &mut [u8],
    msec: i32,
    info: Option<&mut sys::dlpi_recvinfo_t>,
) -> Result<(usize, usize)> {
    let mut src_read = src.len();
    let mut msg_read = msg.len();
    let ret = unsafe {
        sys::dlpi_recv(
            h.0,
            src.as_mut_ptr() as *mut c_void,
            &mut src_read,
            msg.as_mut_ptr() as *mut c_void,
            &mut msg_read,
            msec,
            match info {
                Some(info) => info as *mut sys::dlpi_recvinfo_t,
                None => ptr::null_mut(),
            },
        )
    };

    check_return(ret)?;
    Ok((src_read, msg_read))
}

/// A receiver object returned from [`recv_async`] wrapped in a future. Calling
/// `await` on this object yields the same result as [`recv`].
pub struct DlpiRecv<'a> {
    h: DlpiHandle,
    src: &'a mut [u8],
    msg: &'a mut [u8],
    info: Option<&'a mut sys::dlpi_recvinfo_t>,
}

/// An `async` version of [`recv`]. Calling `await` on result yields same
/// result as [`recv`].
///
/// **`src` must be at least [`sys::DLPI_PHYSADDR_MAX`] in length**.
/*pub fn recv_async<'a>(
    h: DlpiHandle,
    src: &'a mut [u8],
    msg: &'a mut [u8],
    info: Option<&'a mut sys::dlpi_recvinfo_t>,
) -> DlpiRecv<'a> {
    DlpiRecv::<'a> { h, src, msg, info }
}
*/

pub async fn recv_async<'a>(
    h: DlpiHandle,
    src: &'a mut [u8],
    msg: &'a mut [u8],
    info: Option<&'a mut sys::dlpi_recvinfo_t>,
) -> Result<(usize, usize)> {
    let afd = tokio::io::unix::AsyncFd::new(fd(h)?)?;
    let mut _guard = afd.readable().await?;
    recv(
        h, src, msg, 0, // non blocking
        info,
    )
}

impl<'a> Future for DlpiRecv<'a> {
    type Output = Result<(usize, usize)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut src_read = self.src.len();
        let mut msg_read = self.msg.len();
        let s = self.get_mut();

        let ret = unsafe {
            sys::dlpi_recv(
                s.h.0,
                s.src.as_mut_ptr() as *mut c_void,
                &mut src_read,
                s.msg.as_mut_ptr() as *mut c_void,
                &mut msg_read,
                0, // non blocking
                match s.info {
                    Some(ref mut info) => *info as *mut sys::dlpi_recvinfo_t,
                    None => ptr::null_mut(),
                },
            )
        };

        if ret == ResultCode::Success as i32 {
            Poll::Ready(Ok((src_read, msg_read)))
        } else if ret == ResultCode::ETimedout as i32 {
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Err(to_io_error(ret)))
        }
    }
}

/// Bind a DLPI link to a service access point type.
///
/// This will restrict the DLPI link to only operate on the provided service
/// access point. For ethernet the service access point is the ethertype.
pub fn bind(h: DlpiHandle, sap: u32) -> Result<u32> {
    let mut bound_sap = 0;
    let ret = unsafe { sys::dlpi_bind(h.0, sap, &mut bound_sap) };

    check_return(ret)?;
    Ok(bound_sap)
}

/// Enable reception of messages destined to the provided layer-2 address.
///
/// In most cases the layer 2 address will be a mac address. For example,
/// something in the range `33:33:00:00:00:00`-`33:33:FF:FF:FF:FF` for IPv6
/// multicast.
pub fn enable_multicast(h: DlpiHandle, addr: &[u8]) -> Result<()> {
    let ret = unsafe {
        sys::dlpi_enabmulti(h.0, addr.as_ptr() as *const c_void, addr.len())
    };

    check_return(ret)?;
    Ok(())
}

/// Disable reception of messages destined to the provided layer-2 address.
///
/// In most cases the layer 2 address will be a mac address. For example,
/// something in the range `33:33:00:00:00:00`-`33:33:FF:FF:FF:FF` for IPv6
/// multicast.
pub fn disable_multicast(h: DlpiHandle, addr: &[u8]) -> Result<()> {
    let ret = unsafe {
        sys::dlpi_disabmulti(h.0, addr.as_ptr() as *const c_void, addr.len())
    };

    check_return(ret)?;
    Ok(())
}

/// Enable promiscuous mode for the specified handle. See DL_PROMISC_* for
/// levels.
pub fn promisc_on(h: DlpiHandle, level: u32) -> Result<()> {
    let ret = unsafe { sys::dlpi_promiscon(h.0, level) };
    match ret {
        -1 => Err(Error::from_raw_os_error(libc::EINVAL)),
        _ => Ok(()),
    }
}

/// Disable promiscuous mode for the specified handle. See DL_PROMISC_* for
/// levels.
pub fn promisc_off(h: DlpiHandle, level: u32) -> Result<()> {
    let ret = unsafe { sys::dlpi_promiscoff(h.0, level) };
    match ret {
        -1 => Err(Error::from_raw_os_error(libc::EINVAL)),
        _ => Ok(()),
    }
}

/// Get a file descriptor associated with the provided handle.
pub fn fd(h: DlpiHandle) -> Result<i32> {
    let ret = unsafe { sys::dlpi_fd(h.0) };
    match ret {
        -1 => Err(Error::from_raw_os_error(libc::EINVAL)),
        _ => Ok(ret),
    }
}

/// Close the provided handle.
pub fn close(h: DlpiHandle) {
    unsafe { sys::dlpi_close(h.0) };
}

fn check_return(ret: i32) -> Result<()> {
    if ret == ResultCode::Success as i32 {
        return Ok(());
    }

    Err(to_io_error(ret))
}

fn to_io_error(ret: i32) -> Error {
    if ret == sys::DL_SYSERR {
        return Error::last_os_error();
    }

    match ResultCode::try_from(ret) {
        Ok(rc) => Error::new(ErrorKind::Other, rc),
        Err(_) => Error::from_raw_os_error(ret),
    }
}

#[cfg(test)]
mod test;
