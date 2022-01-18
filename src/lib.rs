use std::future::Future;
use std::io::{Error, Result};
use std::os::raw::{c_char, c_void};
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll};

pub mod sys;

/// A DLPI handle wrapper that implements `Send` and `Sync`.
#[derive(Clone, Copy)]
pub struct DlpiHandle(pub *mut sys::dlpi_handle);
unsafe impl Send for DlpiHandle {}
unsafe impl Sync for DlpiHandle {}

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
    match ret {
        sys::ResultCode::Success => Ok(DlpiHandle(dhp)),
        _ => Err(Error::from_raw_os_error(ret as i32)),
    }
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

    match ret {
        sys::ResultCode::Success => Ok(()),
        _ => Err(Error::from_raw_os_error(ret as i32)),
    }
}

/// Receive a message from a DLPI link.
///
/// Data is placed into provided buffer.  Return values is (address bytes read,
/// message bytes read).
///
/// If no message is received within `msec` milliseconds, returns
/// [`sys::ResultCode::ETimedout`].
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

    match ret {
        sys::ResultCode::Success => Ok((src_read, msg_read)),
        _ => Err(Error::from_raw_os_error(ret as i32)),
    }
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
pub fn recv_async<'a>(
    h: DlpiHandle,
    src: &'a mut [u8],
    msg: &'a mut [u8],
    info: Option<&'a mut sys::dlpi_recvinfo_t>,
) -> DlpiRecv<'a> {
    DlpiRecv::<'a> { h, src, msg, info }
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

        match ret {
            sys::ResultCode::Success => Poll::Ready(Ok((src_read, msg_read))),
            sys::ResultCode::ETimedout => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            _ => Poll::Ready(Err(Error::from_raw_os_error(ret as i32))),
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

    match ret {
        sys::ResultCode::Success => Ok(bound_sap),
        _ => Err(Error::from_raw_os_error(ret as i32)),
    }
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

    match ret {
        sys::ResultCode::Success => Ok(()),
        _ => Err(Error::from_raw_os_error(ret as i32)),
    }
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

    match ret {
        sys::ResultCode::Success => Ok(()),
        _ => Err(Error::from_raw_os_error(ret as i32)),
    }
}
