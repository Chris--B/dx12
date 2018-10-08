#![feature(termination_trait_lib)]

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate clap;
extern crate termcolor;
extern crate winapi;
extern crate wio;

use std::{
    fmt,
    mem,
    ptr,
};

use wio::com::ComPtr;

use winapi::{
    Interface,
    um::unknwnbase::IUnknown,

    shared::winerror,
    um::winuser,

    // These functions include a namespace in their names, so we won't
    // double-namespace them.
    // e.g. `d3d12::D3D12CreateDevice`
    shared::dxgi1_3::DXGIGetDebugInterface1,
    shared::dxgi1_4::*,
    shared::dxgi::*,
    shared::dxgiformat::*,
    shared::dxgitype::*,

    um::d3d12::*,
    um::d3d12sdklayers::*,
    um::dxgidebug::*,
};

#[macro_use]
mod macros;
mod error;
mod config;
mod win32_window;
mod renderer;

fn main() -> Result<(), error::WindowsError> {
    let conf = config::Config::load();
    println!("{:#?}\n", conf);

    let h_wnd = win32_window::init_window("Dx12?")?;

    let _r = renderer::Renderer::create(&conf, h_wnd)?;

    loop {
        unsafe {
            let mut msg = mem::zeroed();
            let ret = winuser::GetMessageA(&mut msg,
                                           ptr::null_mut(), // hWnd
                                           0,               // wMsgFilterMin
                                           0);              // wMsgFilterMax
            if ret == 0 {
                break;
            }
            winuser::TranslateMessage(&msg);
            winuser::DispatchMessageA(&msg);
        }
    }

    Ok(())
}
