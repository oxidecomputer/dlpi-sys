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

/// Information used to send DLPI traffic.
#[repr(C)]
#[derive(Debug)]
pub struct dlpi_sendinfo_t {
    /// Service access point to use. For ethernet, this is the ethertype.
    pub sap: c_uint,
    /// Range spans from 0 to 100 where 0 is the highest.
    pub prio: dl_priority_t,
}

/// Information from received DLPI traffic.
#[repr(C)]
pub struct dlpi_recvinfo_t {
    pub destaddr: [c_uchar; DLPI_PHYSADDR_MAX],
    pub destaddrlen: c_uchar,
    pub destaddrtype: dlpi_addrtype_t,
    pub totmsglen: usize,
}

impl Default for dlpi_recvinfo_t {
    fn default() -> Self {
        dlpi_recvinfo_t {
            destaddr: [0; DLPI_PHYSADDR_MAX],
            destaddrlen: 0,
            destaddrtype: dlpi_addrtype_t::Unicast,
            totmsglen: 0,
        }
    }
}

/// Indicates if an address is unicast or not.
#[repr(C)]
pub enum dlpi_addrtype_t {
    Unicast,
    Group,
}

/// Defines priority for DLPI traffic sent.
#[repr(C)]
#[derive(Debug)]
pub struct dl_priority_t {
    pub min: u32,
    pub max: u32,
}

/// Result of a DLPI operation.
#[repr(C)]
#[derive(PartialEq, Debug)]
pub enum ResultCode {
    Success = 10000,  /* DLPI operation succeeded */
    EInval,           /* invalid argument */
    ELinkNameInval,   /* invalid DLPI linkname */
    ENoLink,          /* DLPI link does not exist */
    EBadLink,         /* bad DLPI link */
    EInHandle,        /* invalid DLPI handle */
    ETimedout,        /* DLPI operation timed out */
    EVerNotSup,       /* unsupported DLPI Version */
    EModeNotSup,      /* unsupported DLPI connection mode */
    EUnavailSAP,      /* unavailable DLPI SAP */
    Failure,          /* DLPI operation failed */
    ENotStyle2,       /* DLPI style-2 node reports style-1 */
    EBadMsg,          /* bad DLPI message */
    ERawNotSup,       /* DLPI raw mode not supported */
    ENoteInval,       /* invalid DLPI notification type */
    ENoteNotSup,      /* DLPI notification not supported by link */
    ENoteIdInval,     /* invalid DLPI notification id */
    EIpNetInfoNotSup, /* DLPI_IPNETINFO not supported */
    ErrMax,           /* Highest + 1 libdlpi error code */
}

/// Opaque handle to a DLPI link instance.
#[repr(C)]
#[derive(Clone)]
pub struct dlpi_handle {
    private: [u8; 0],
}
unsafe impl Send for dlpi_handle {}
unsafe impl Sync for dlpi_handle {}

extern "C" {

    /// Creates an instance of the DLPI version 2 link named by linnkname.
    ///
    /// Associates with it a dynamically-allocated dlpi_handle_t, which is
    /// returend to the caller upon success. This handle is used in other DLPI
    /// function calls for sending and receiving traffic and otherwise managing
    /// a DLPI link.
    pub fn dlpi_open(
        linkname: *const c_char,
        dhp: *mut *mut dlpi_handle,
        flags: c_uint,
    ) -> ResultCode;

    /// Closes the open DLPI link instance associated with the provided handle.
    pub fn dlpi_close(dh: *mut dlpi_handle);

    /// Attempt to send the contets of msgbuf over the DLPI link instance
    /// associated with the provided DLPI handle.
    pub fn dlpi_send(
        dh: *mut dlpi_handle,
        daddrp: *const c_void,
        daddrlen: usize,
        msgbuf: *const c_void,
        msglen: usize,
        sendp: *const dlpi_sendinfo_t,
    ) -> ResultCode;

    /// Attempt to receive data messages over the DLIP link instance associated
    /// with the provided DLPI handle.
    pub fn dlpi_recv(
        dh: *mut dlpi_handle,
        saddrp: *mut c_void,
        saddrlenp: *mut usize,
        msgbuf: *mut c_void,
        msglenp: *mut usize,
        msec: c_int,
        recvp: *mut dlpi_recvinfo_t,
    ) -> ResultCode;

    /// Attempt to bind the provided DLPI handle to the service access point
    /// `sap`.
    pub fn dlpi_bind(
        dh: *mut dlpi_handle,
        sap: c_uint,
        boundsap: *mut c_uint,
    ) -> ResultCode;

    /// Enable reception of messages destined to the multicast address pointed
    /// to by addrp.
    pub fn dlpi_enabmulti(
        dh: *mut dlpi_handle,
        addrp: *const c_void,
        addrlen: usize,
    ) -> ResultCode;

    /// Disable reception of messages destined to the multicast address pointed
    /// to by addrp.
    pub fn dlpi_disabmulti(
        dh: *mut dlpi_handle,
        addrp: *const c_void,
        addrlen: usize,
    ) -> ResultCode;

    /// Returns a file descriptor that can be used to directly operate on the
    /// open DLPI stream associated with the provided handle.
    pub fn dlpi_fd(dh: *mut dlpi_handle) -> i32;

}

/// A convenience method for creating a null dlpi handle to be later initialized
/// by `dlpi_open`.
pub fn null_dlpi_handle() -> *mut dlpi_handle {
    std::ptr::null_mut::<dlpi_handle>()
}
