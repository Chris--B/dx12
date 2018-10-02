
use winapi::{
    shared::minwindef::HINSTANCE,
    shared::windef::HBRUSH,
    shared::windef::HWND,

    um::errhandlingapi::GetLastError,
    um::libloaderapi::GetModuleHandleA,
    um::wingdi::GetStockObject,
    um::wingdi::WHITE_BRUSH,
    um::winuser::{
        CreateWindowExA,
        LoadCursorW,
        LoadIconW,
        RegisterClassA,
        ShowWindow,
        UpdateWindow,
        CS_HREDRAW,
        CS_VREDRAW,
        CW_USEDEFAULT,
        IDC_ARROW,
        IDI_APPLICATION,
        WNDCLASSA,
        WS_OVERLAPPEDWINDOW,
    },
};

use std::{
    ptr,
};

extern "system" fn wnd_proc(h_wnd:   HWND,
                            msg:     u32,
                            w_param: usize,
                            l_param: isize) -> isize {
    use winuser::*;
    let param = w_param as i32;
    unsafe {
        match msg {
            WM_KEYDOWN if param == VK_ESCAPE   => { DestroyWindow(h_wnd); },
            WM_DESTROY                         => { PostQuitMessage(0);   },
            WM_LBUTTONDOWN => {
                #[repr(C)]
                #[derive(Debug)]
                struct Xy {
                    x: u16,
                    y: u16,
                    _zero: u32,
                }
                let xy: Xy = ::std::mem::transmute(l_param);
                println!("Click? w_param=0x{:08x}, l_param=0x{:08x} xy={:?}",
                         w_param,
                         l_param,
                         xy);
            },
            _ => {
                return DefWindowProcA(h_wnd, msg, w_param, l_param);
            },
        };
        // All of the branches return 0 if they handle the message.
        0
    }
}

pub fn init_window(window_title: &str) -> Result<HWND, u32> {
    unsafe {
        let h_instance = GetModuleHandleA(ptr::null_mut()) as HINSTANCE;

        let wc = WNDCLASSA {
            style:         CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc:   Some(wnd_proc),
            cbClsExtra:    0,
            cbWndExtra:    0,
            hInstance:     h_instance,
            hIcon:         LoadIconW(ptr::null_mut(), IDI_APPLICATION),
            hCursor:       LoadCursorW(ptr::null_mut(), IDC_ARROW),
            hbrBackground: GetStockObject(WHITE_BRUSH as i32) as HBRUSH,
            lpszMenuName:  ptr::null_mut(),
            lpszClassName: b"BasicWndClass".as_ptr() as *const i8,
        };

        if RegisterClassA(&wc) == 0 {
            let hresult_err = check_hresult!(GetLastError() as i32, RegisterClassA);
            assert!(hresult_err.is_err());
            hresult_err?;
        }

        let h_wnd = CreateWindowExA(0x0,                 // Ex style flags
                                    wc.lpszClassName,
                                    window_title.as_ptr() as *const i8,
                                    WS_OVERLAPPEDWINDOW, // Style flags
                                    CW_USEDEFAULT,       // x-coord
                                    CW_USEDEFAULT,       // y-coord
                                    CW_USEDEFAULT,       // width
                                    CW_USEDEFAULT,       // height
                                    ptr::null_mut(),     // Parent window
                                    ptr::null_mut(),     // Menu handle
                                    h_instance,
                                    ptr::null_mut()      /*Extra params*/);

        ShowWindow(h_wnd, 1);
        UpdateWindow(h_wnd);

        Ok(h_wnd)
    }
}
