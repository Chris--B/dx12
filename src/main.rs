#![feature(termination_trait_lib)]

// I'll toggle this when developing
// #![deny(warnings)]
#![allow(dead_code)]

extern crate clap;
extern crate termcolor;
extern crate winapi;
extern crate wio;

use std::{
    mem,
    ptr,
};

use winapi::{
    um::winuser::*,
    um::winuser,
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
            let ret = GetMessageA(&mut msg,
                                  ptr::null_mut(), // hWnd
                                  0,               // wMsgFilterMin
                                  0);              // wMsgFilterMax
            if ret == 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }

    Ok(())
}
