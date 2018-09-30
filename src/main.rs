#![feature(termination_trait_lib)]

extern crate clap;
extern crate winapi;
extern crate wio;

use std::{
    ptr,
};

use wio::com::ComPtr;

use winapi::{
    Interface,
    shared::winerror,

    // These functions include a namespace in their names, so we won't
    // double-namespace them.
    // e.g. `d3d12::D3D12CreateDevice`
    um::d3d12::*,
    um::d3d12sdklayers::*,
    um::d3dcommon::*,
    shared::dxgi::*,
};

mod error;

fn get_arg_matches<'a>() -> clap::ArgMatches<'a> {
    use clap::{App, Arg};
    App
        // Metadata
        ::new("Dx12 Demo")
        .author("Chris Butler <chrisbutler296@gmail.com>")
        .about("Draw things with DX12")

        // Adapter selection
        .arg(Arg::with_name("warp")
                .help("Force using the warp adapter.")
                .long("warp")
                .short("w")
                .required(false)
                .overrides_with("adapter"))
        // TODO: Some way to select an adapter

        // Debug options
        .arg(Arg::with_name("debug-layer")
                .help("Enable the DX12 runtime debug layer.")
                .long("debug-layer")
                .overrides_with("no-debug-layer"))
        .arg(Arg::with_name("no-debug-layer")
                .help("Disable the DX12 runtime debug layer.")
                .long("no-debug-layer")
                .overrides_with("debug-layer"))

        // End
        .get_matches()
}

fn main() -> Result<(), u32> {
    let matches = get_arg_matches();

    if !matches.is_present("no-debug-layer") {
        unsafe {
            let mut p_debug: *mut ID3D12Debug = ptr::null_mut();
            let hr = D3D12GetDebugInterface(&ID3D12Debug::uuidof(),
                                            &mut p_debug as *mut _ as *mut _);
            if !winerror::SUCCEEDED(hr) {
                eprintln!("D3D12GetDebugInterface: (0x{:x}) \"{}\"",
                          hr,
                          error::win_error_msg(hr));
                return Err(1);
            }
            let debug: ComPtr<ID3D12Debug> = ComPtr::from_raw(p_debug);
            debug.EnableDebugLayer();
        }
    }

    let _dxgi_factory: ComPtr<IDXGIFactory1> = unsafe {
        let mut p_dxgi_factory: *mut IDXGIFactory1 = ptr::null_mut();
        let hr = CreateDXGIFactory1(&IDXGIFactory1::uuidof(),
                                    &mut p_dxgi_factory as *mut _ as *mut _);
        if !winerror::SUCCEEDED(hr) {
            eprintln!("CreateDXGIFactory1: (0x{:x}) \"{}\"",
                      hr,
                      error::win_error_msg(hr));
            return Err(1);
        }
        ComPtr::from_raw(p_dxgi_factory)
    };

    let _adapter: ComPtr<IDXGIAdapter>;

    let _device: ComPtr<ID3D12Device> = unsafe {
        let mut p_device: *mut ID3D12Device = ptr::null_mut();
        let hr = D3D12CreateDevice(ptr::null_mut(),
                                   D3D_FEATURE_LEVEL_11_0,
                                   &ID3D12Device::uuidof(),
                                   &mut p_device as *mut _ as *mut _);
        if !winerror::SUCCEEDED(hr) {
            eprintln!("D3D12CreateDevice: (0x{:x}) \"{}\"",
                      hr,
                      error::win_error_msg(hr));
            return Err(1);
        }

        ComPtr::from_raw(p_device)
    };

    Ok(())
}
