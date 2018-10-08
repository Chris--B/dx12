
use std::{
    fmt,
    ptr,
    str,
};

use winapi::{
    shared::winerror,
    shared::winerror::HRESULT,
    shared::ntdef::HANDLE,

    um::winnt,
};

use termcolor;

pub type WindowsResult<T> = Result<T, WindowsError>;

#[derive(Copy, Clone)]
pub enum WindowsError {
    NotImplemented,
    Hresult(HRESULT),
}

impl fmt::Debug for WindowsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ::error::WindowsError as W;
        match self {
            W::NotImplemented => write!(f, "NotImplemented"),
            W::Hresult(hr)    => write!(f, "HRESULT={}", win_error_msg(*hr)),
        }
    }
}

impl From<HRESULT> for WindowsError {
    fn from(hresult: HRESULT) -> WindowsError {
        WindowsError::Hresult(hresult)
    }
}

fn hresult_code(hresult: HRESULT) -> u32 {
    // https://docs.microsoft.com/en-us/windows/desktop/api/winerror/nf-winerror-hresult_code
    (hresult as u32) & 0xFFFF
}

pub fn dxgi_error_msg(hresult: HRESULT) -> Option<&'static str> {
    match hresult {
        winerror::DXGI_ERROR_ACCESS_DENIED                => Some("DXGI_ERROR_ACCESS_DENIED"),
        winerror::DXGI_ERROR_ACCESS_LOST                  => Some("DXGI_ERROR_ACCESS_LOST"),
        winerror::DXGI_ERROR_CANNOT_PROTECT_CONTENT       => Some("DXGI_ERROR_CANNOT_PROTECT_CONTENT"),
        winerror::DXGI_ERROR_DEVICE_HUNG                  => Some("DXGI_ERROR_DEVICE_HUNG"),
        winerror::DXGI_ERROR_DEVICE_REMOVED               => Some("DXGI_ERROR_DEVICE_REMOVED"),
        winerror::DXGI_ERROR_DEVICE_RESET                 => Some("DXGI_ERROR_DEVICE_RESET"),
        winerror::DXGI_ERROR_DRIVER_INTERNAL_ERROR        => Some("DXGI_ERROR_DRIVER_INTERNAL_ERROR"),
        winerror::DXGI_ERROR_FRAME_STATISTICS_DISJOINT    => Some("DXGI_ERROR_FRAME_STATISTICS_DISJOINT"),
        winerror::DXGI_ERROR_GRAPHICS_VIDPN_SOURCE_IN_USE => Some("DXGI_ERROR_GRAPHICS_VIDPN_SOURCE_IN_USE"),
        winerror::DXGI_ERROR_INVALID_CALL                 => Some("DXGI_ERROR_INVALID_CALL"),
        winerror::DXGI_ERROR_MORE_DATA                    => Some("DXGI_ERROR_MORE_DATA"),
        winerror::DXGI_ERROR_NAME_ALREADY_EXISTS          => Some("DXGI_ERROR_NAME_ALREADY_EXISTS"),
        winerror::DXGI_ERROR_NONEXCLUSIVE                 => Some("DXGI_ERROR_NONEXCLUSIVE"),
        winerror::DXGI_ERROR_NOT_CURRENTLY_AVAILABLE      => Some("DXGI_ERROR_NOT_CURRENTLY_AVAILABLE"),
        winerror::DXGI_ERROR_NOT_FOUND                    => Some("DXGI_ERROR_NOT_FOUND"),
        winerror::DXGI_ERROR_REMOTE_CLIENT_DISCONNECTED   => Some("DXGI_ERROR_REMOTE_CLIENT_DISCONNECTED"),
        winerror::DXGI_ERROR_REMOTE_OUTOFMEMORY           => Some("DXGI_ERROR_REMOTE_OUTOFMEMORY"),
        winerror::DXGI_ERROR_RESTRICT_TO_OUTPUT_STALE     => Some("DXGI_ERROR_RESTRICT_TO_OUTPUT_STALE"),
        winerror::DXGI_ERROR_SDK_COMPONENT_MISSING        => Some("DXGI_ERROR_SDK_COMPONENT_MISSING"),
        winerror::DXGI_ERROR_SESSION_DISCONNECTED         => Some("DXGI_ERROR_SESSION_DISCONNECTED"),
        winerror::DXGI_ERROR_UNSUPPORTED                  => Some("DXGI_ERROR_UNSUPPORTED"),
        winerror::DXGI_ERROR_WAIT_TIMEOUT                 => Some("DXGI_ERROR_WAIT_TIMEOUT"),
        winerror::DXGI_ERROR_WAS_STILL_DRAWING            => Some("DXGI_ERROR_WAS_STILL_DRAWING"),
        code if (code as u32 >> 16) == 0x887a             => Some("Unknown DXGI_ERROR"),
        _                                                 => None,
    }
}

pub fn generic_error_msg(hresult: HRESULT) -> Option<&'static str> {
    match hresult {
        winerror::E_UNEXPECTED                  => Some("E_UNEXPECTED"),
        winerror::E_NOTIMPL                     => Some("E_NOTIMPL"),
        winerror::E_OUTOFMEMORY                 => Some("E_OUTOFMEMORY"),
        winerror::E_INVALIDARG                  => Some("E_INVALIDARG"),
        winerror::E_NOINTERFACE                 => Some("E_NOINTERFACE"),
        winerror::E_POINTER                     => Some("E_POINTER"),
        winerror::E_HANDLE                      => Some("E_HANDLE"),
        winerror::E_ABORT                       => Some("E_ABORT"),
        winerror::E_FAIL                        => Some("E_FAIL"),
        winerror::E_ACCESSDENIED                => Some("E_ACCESSDENIED"),
        winerror::E_PENDING                     => Some("E_PENDING"),
        winerror::E_BOUNDS                      => Some("E_BOUNDS"),
        winerror::E_CHANGED_STATE               => Some("E_CHANGED_STATE"),
        winerror::E_ILLEGAL_STATE_CHANGE        => Some("E_ILLEGAL_STATE_CHANGE"),
        winerror::E_ILLEGAL_METHOD_CALL         => Some("E_ILLEGAL_METHOD_CALL"),
        winerror::E_STRING_NOT_NULL_TERMINATED  => Some("E_STRING_NOT_NULL_TERMINATED"),
        winerror::E_ILLEGAL_DELEGATE_ASSIGNMENT => Some("E_ILLEGAL_DELEGATE_ASSIGNMENT"),
        winerror::E_ASYNC_OPERATION_NOT_STARTED => Some("E_ASYNC_OPERATION_NOT_STARTED"),
        winerror::E_APPLICATION_EXITING         => Some("E_APPLICATION_EXITING"),
        winerror::E_APPLICATION_VIEW_EXITING    => Some("E_APPLICATION_VIEW_EXITING"),
        _                                       => None,
    }
}

pub fn win_error_msg(hresult: HRESULT) -> &'static str {
    use winapi::um::winbase::{
        FormatMessageA,
        FORMAT_MESSAGE_FROM_SYSTEM,
        FORMAT_MESSAGE_IGNORE_INSERTS,
    };
    use winapi::shared::ntdef::{
        MAKELANGID,
        LANG_NEUTRAL,
        SUBLANG_DEFAULT,
    };

    if let Some(error_msg) = dxgi_error_msg(hresult) {
        return error_msg;
    }
    if let Some(error_msg) = generic_error_msg(hresult) {
        return error_msg;
    }

    static mut BUFFER: [u8; 128] = [0u8; 128];
    unsafe {
        // Fill the buffer so that our trim below can actually trim it.
        // Reminder: We're converting from C-style NULL-terminated strings to
        //           Rust's UTF8 strings.
        // We do this every call to clear the previous error message.
        ptr::write_bytes(BUFFER.as_mut_ptr(), 0u8, BUFFER.len());
        let _n = FormatMessageA(FORMAT_MESSAGE_FROM_SYSTEM |
                                FORMAT_MESSAGE_IGNORE_INSERTS,
                                ptr::null_mut(),
                                hresult_code(hresult),
                                MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
                                BUFFER.as_mut_ptr() as *mut _,
                                BUFFER.len() as u32,
                                ptr::null_mut());
        str::from_utf8(&BUFFER)
            // We do not expect this message to be malformed.
            // If it is, we have bigger problems than what this is reporting.
            .unwrap()
            // These messages often end in "\n\r\0", followed by NULLs for
            // the rest of the buffer.
            .trim_matches(|c| { c == '\n' || c == '\r' || c == '\0' })
    }
}

pub struct ColorSpecCatalog {
    pub file:        termcolor::ColorSpec,
    pub line:        termcolor::ColorSpec,
    pub func:        termcolor::ColorSpec,
    pub windows_msg: termcolor::ColorSpec,
    pub hresult:     termcolor::ColorSpec,
}

pub fn get_color_spec_catalog() -> ColorSpecCatalog {
    use termcolor::{
        Color,
        ColorSpec,
    };

    // TODO: It would be neat if this were loaded from a config file.
    let mut specs = ColorSpecCatalog {
        file:        ColorSpec::new(),
        line:        ColorSpec::new(),
        func:        ColorSpec::new(),
        windows_msg: ColorSpec::new(),
        hresult:     ColorSpec::new(),
    };

    specs.file.set_fg(None);
    specs.line.set_fg(None);
    specs.func.set_fg(None);

    specs.windows_msg.set_fg(Some(Color::Red));
    specs.windows_msg.set_intense(true);

    specs.hresult.set_fg(Some(Color::Red));

    specs
}
