#![feature(termination_trait_lib)]

extern crate clap;
extern crate termcolor;
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
    shared::dxgi1_4::*,
};

#[macro_use]
mod macros;
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
            check_hresult!(hr, D3D12GetDebugInterface);
            let debug: ComPtr<ID3D12Debug> = ComPtr::from_raw(p_debug);
            debug.EnableDebugLayer();
        }
    }

    let dxgi_factory: ComPtr<IDXGIFactory4> = unsafe {
        let mut p_dxgi_factory: *mut IDXGIFactory4 = ptr::null_mut();
        let hr = CreateDXGIFactory(&IDXGIFactory4::uuidof(),
                                   &mut p_dxgi_factory as *mut _ as *mut _);
        check_hresult!(hr, CreateDXGIFactory);
        ComPtr::from_raw(p_dxgi_factory)
    };

    // This is only NOT NULL when using the warp adapter.
    // We keep this NULL otherwise, to tell D3D12CreateDevice to use
    // the default adapter.
    let mut p_adapter: *mut IDXGIAdapter = ptr::null_mut();
    if matches.is_present("warp") {
        unsafe {
            let hr = dxgi_factory.EnumWarpAdapter(&IDXGIAdapter::uuidof(),
                                                  &mut p_adapter as *mut _ as *mut _);
            check_hresult!(hr, IDXGIFactory4::EnumWarpAdapter);
        }
    }
    let p_adapter = p_adapter;

    let _device: ComPtr<ID3D12Device> = unsafe {
        let mut p_device: *mut ID3D12Device = ptr::null_mut();
        let hr = D3D12CreateDevice(p_adapter as *mut _,
                                   D3D_FEATURE_LEVEL_11_0,
                                   &ID3D12Device::uuidof(),
                                   &mut p_device as *mut _ as *mut _);
        check_hresult!(hr, D3D12CreateDevice);
        ComPtr::from_raw(p_device)
    };

    unsafe {
        if let Some(adapter) = p_adapter.as_ref() {
            adapter.Release();
        }
    }

    Ok(())
}
