#![feature(termination_trait_lib)]

extern crate winapi;
extern crate wio;


use std::{
    ptr,
};

use wio::com::ComPtr;

use winapi::{
    Interface,
    shared::winerror,
    um::winnt,

    // These functions include a namespace in their names, so we won't
    // double-namespace them.
    // e.g. `d3d12::D3D12CreateDevice`
    um::d3dcommon::*,
    um::d3d12::*,
};

fn hresult_code(hresult: winnt::HRESULT) -> u32 {
    // https://docs.microsoft.com/en-us/windows/desktop/api/winerror/nf-winerror-hresult_code
    (hresult as u32) & 0xFFFF
}

fn dxgi_error_msg(hresult: winnt::HRESULT) -> Option<&'static str> {
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

fn win_error_msg(hresult: winnt::HRESULT) -> &'static str {
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
        std::str::from_utf8(&BUFFER)
            // We do not expect this message to be malformed.
            // If it is, we have bigger problems than what this is reporting.
            .unwrap()
            // These messages often end in "\n\r\0", followed by NULLs for
            // the rest of the buffer.
            .trim_matches(|c| { c == '\n' || c == '\r' || c == '\0' })
    }
}

fn main() -> Result<(), u32> {
    let _device: ComPtr<ID3D12Device> = unsafe {
        let mut p_device: *mut ID3D12Device = ptr::null_mut();
        let hresult = D3D12CreateDevice(ptr::null_mut(),
                                        D3D_FEATURE_LEVEL_12_0,
                                        &ID3D12Device::uuidof(),
                                        &mut p_device as *mut _ as *mut _);

        if !winerror::SUCCEEDED(hresult) {
            eprintln!("Error creating ID3D12Device! 0x{:x}: \"{}\"",
                      hresult,
                      win_error_msg(hresult));
            return Err(1);
        }

        ComPtr::from_raw(p_device)
    };

    Ok(())
}
