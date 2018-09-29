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
