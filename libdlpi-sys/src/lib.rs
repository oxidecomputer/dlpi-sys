#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_void};

/// Maximum size for a physical address.
pub const DLPI_PHYSADDR_MAX: usize = 64;

/// DLPI Flags

/// Exclusive open.
pub const DLPI_EXCL: c_uint = 0x0001;
/// Open DLPI link in passive mode.
pub const DLPI_PASSIVE: c_uint = 0x0002;
/// Open DLPI link in raw mode.
pub const DLPI_RAW: c_uint = 0x0004;
/// Synchronous serial line interface.
pub const DLPI_SERIAL: c_uint = 0x0008;
/// Do not attach PPA.
pub const DLPI_NOATTACH: c_uint = 0x0010;
/// Open DLPI link in native mode.
pub const DLPI_NATIVE: c_uint = 0x0020;
/// Open DLPI link under /dev only.
pub const DLPI_DEVONLY: c_uint = 0x0040;
/// Open IP DLPI link under /dev/ipnet.
pub const DLPI_DEVIPNET: c_uint = 0x0080;
/// Request ipnetinfo headers.
pub const DLPI_IPNETINFO: c_uint = 0x0100;
/// Promiscuous mode at the phys level
pub const DL_PROMISC_PHYS: c_uint = 0x01;
/// Promiscuous mode at the sap level
pub const DL_PROMISC_SAP: c_uint = 0x02;
/// Promiscuous mode for multicast
pub const DL_PROMISC_MULTI: c_uint = 0x03;
/// Promiscuous mode for rx only
pub const DL_PROMISC_RX_ONLY: c_uint = 0x04;

/// DLPI operation succeeded
pub const DLPI_SUCCESS: c_int = 10000;
/// invalid argument
pub const DLPI_EINVAL: c_int = 10001;
/// invalid DLPI linkname
pub const DLPI_ELINKNAMEINVAL: c_int = 10002;
/// DLPI link does not exist
pub const DLPI_ENOLINK: c_int = 10003;
/// bad DLPI link
pub const DLPI_EBADLINK: c_int = 10004;
/// invalid DLPI handle
pub const DLPI_EINHANDLE: c_int = 10005;
/// DLPI operation timed out
pub const DLPI_ETIMEDOUT: c_int = 10006;
/// unsupported DLPI Version
pub const DLPI_EVERNOTSUP: c_int = 10007;
/// unsupported DLPI connection mode
pub const DLPI_EMODENOTSUP: c_int = 10008;
/// unavailable DLPI SAP
pub const DLPI_EUNAVAILSAP: c_int = 10009;
/// DLPI operation failed
pub const DLPI_FAILURE: c_int = 10010;
/// DLPI style-2 node reports style-1
pub const DLPI_ENOTSTYLE2: c_int = 10011;
/// bad DLPI message
pub const DLPI_EBADMSG: c_int = 10012;
/// DLPI raw mode not supported
pub const DLPI_ERAWNOTSUP: c_int = 10013;
/// invalid DLPI notification type
pub const DLPI_ENOTEINVAL: c_int = 10014;
/// DLPI notif. not supported by link
pub const DLPI_ENOTENOTSUP: c_int = 10015;
/// invalid DLPI notification id
pub const DLPI_ENOTEIDINVAL: c_int = 10016;
/// DLPI_IPNETINFO not supported
pub const DLPI_EIPNETINFONOTSUP: c_int = 10017;


/// Information used to send DLPI traffic.
#[repr(C)]
#[derive(Debug)]
pub struct dlpi_sendinfo_t {
    /// Service access point to use. For ethernet, this is the ethertype.
    pub dsi_sap: c_uint,
    /// Range spans from 0 to 100 where 0 is the highest.
    pub dsi_prio: dl_priority_t,
}

/// Information from received DLPI traffic.
#[repr(C)]
pub struct dlpi_recvinfo_t {
    pub dri_destaddr: [c_uchar; DLPI_PHYSADDR_MAX],
    pub dri_destaddrlen: c_uchar,
    pub dri_destaddrtype: dlpi_addrtype_t,
    pub dri_totmsglen: usize,
}

impl Default for dlpi_recvinfo_t {
    fn default() -> Self {
        dlpi_recvinfo_t {
            dri_destaddr: [0; DLPI_PHYSADDR_MAX],
            dri_destaddrlen: 0,
            dri_destaddrtype: dlpi_addrtype_t::DLPI_ADDRTYPE_UNICAST,
            dri_totmsglen: 0,
        }
    }
}

/// Indicates if an address is unicast or not.
#[repr(C)]
pub enum dlpi_addrtype_t {
    DLPI_ADDRTYPE_UNICAST,
    DLPI_ADDRTYPE_GROUP,
}

/// Defines priority for DLPI traffic sent.
#[repr(C)]
#[derive(Debug)]
pub struct dl_priority_t {
    pub dl_min: u32,
    pub dl_max: u32,
}

/// Indicates a non-DLPI specific system error in a DLPI call.
pub const DL_SYSERR: c_int = 0x04;

/// Opaque handle to a DLPI link instance.
#[derive(Clone)]
pub enum dlpi_handle_t {}
unsafe impl Send for dlpi_handle_t {}
unsafe impl Sync for dlpi_handle_t {}

extern "C" {
    /// Creates an instance of the DLPI version 2 link named by linnkname.
    ///
    /// Associates with it a dynamically-allocated dlpi_handle_t, which is
    /// returend to the caller upon success. This handle is used in other DLPI
    /// function calls for sending and receiving traffic and otherwise managing
    /// a DLPI link.
    pub fn dlpi_open(
        linkname: *const c_char,
        dhp: *mut *mut dlpi_handle_t,
        flags: c_uint,
    ) -> i32;

    /// Closes the open DLPI link instance associated with the provided handle.
    pub fn dlpi_close(dh: *mut dlpi_handle_t);

    /// Attempt to send the contets of msgbuf over the DLPI link instance
    /// associated with the provided DLPI handle.
    pub fn dlpi_send(
        dh: *mut dlpi_handle_t,
        daddrp: *const c_void,
        daddrlen: usize,
        msgbuf: *const c_void,
        msglen: usize,
        sendp: *const dlpi_sendinfo_t,
    ) -> i32;

    /// Attempt to receive data messages over the DLIP link instance associated
    /// with the provided DLPI handle.
    pub fn dlpi_recv(
        dh: *mut dlpi_handle_t,
        saddrp: *mut c_void,
        saddrlenp: *mut usize,
        msgbuf: *mut c_void,
        msglenp: *mut usize,
        msec: c_int,
        recvp: *mut dlpi_recvinfo_t,
    ) -> i32;

    /// Attempt to bind the provided DLPI handle to the service access point
    /// `sap`.
    pub fn dlpi_bind(
        dh: *mut dlpi_handle_t,
        sap: c_uint,
        boundsap: *mut c_uint,
    ) -> i32;

    /// Enable reception of messages destined to the multicast address pointed
    /// to by addrp.
    pub fn dlpi_enabmulti(
        dh: *mut dlpi_handle_t,
        addrp: *const c_void,
        addrlen: usize,
    ) -> i32;

    /// Disable reception of messages destined to the multicast address pointed
    /// to by addrp.
    pub fn dlpi_disabmulti(
        dh: *mut dlpi_handle_t,
        addrp: *const c_void,
        addrlen: usize,
    ) -> i32;

    /// Enable promiscuous mode for the specified handle. See DL_PROMISC_*
    /// for levels.
    pub fn dlpi_promiscon(dh: *mut dlpi_handle_t, level: c_uint) -> i32;

    /// Disable promiscuous mode for the specified handle. See DL_PROMISC_*
    /// for levels.
    pub fn dlpi_promiscoff(dh: *mut dlpi_handle_t, level: c_uint) -> i32;

    /// Returns a file descriptor that can be used to directly operate on the
    /// open DLPI stream associated with the provided handle.
    pub fn dlpi_fd(dh: *mut dlpi_handle_t) -> i32;

}

/// A convenience method for creating a null dlpi handle to be later initialized
/// by `dlpi_open`.
pub fn null_dlpi_handle() -> *mut dlpi_handle_t {
    std::ptr::null_mut::<dlpi_handle_t>()
}
