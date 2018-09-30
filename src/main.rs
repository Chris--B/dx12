#![feature(termination_trait_lib)]

extern crate clap;
extern crate termcolor;
extern crate winapi;
extern crate wio;

use std::{
    mem,
    ptr,
};

use wio::com::ComPtr;

use winapi::{
    um::unknwnbase::IUnknown,
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
    shared::dxgiformat::*,
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
                .help("Force using the warp adapter")
                .long("warp")
                .short("w")
                .required(false)
                .overrides_with("adapter"))
        // TODO: Some way to select an adapter

        // Debug options
        .arg(Arg::with_name("debug-layer")
                .display_order(3000)
                .help("Enable the DX12 runtime debug layer")
                .long("debug-layer")
                .overrides_with("no-debug-layer"))
        .arg(Arg::with_name("no-debug-layer")
                .display_order(3001)
                .help("Disable the DX12 runtime debug layer")
                .long("no-debug-layer")
                .overrides_with("debug-layer"))

        // I change this enough to just make it an option.
        .arg(Arg::with_name("feature-level")
                .help("Force using a specific feature level for CreateDevice")
                .long("feature-level")
                .possible_values(&["11", "11.0", "11_0",
                                         "11.1", "11_1",
                                   "12", "12.0", "12_0",
                                         "12.1", "12_1"])
                .default_value("11_0"))

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
            check_hresult!(hr, D3D12GetDebugInterface)?;
            let debug: ComPtr<ID3D12Debug> = ComPtr::from_raw(p_debug);
            debug.EnableDebugLayer();
        }
    }

    let dxgi_factory: ComPtr<IDXGIFactory4> = unsafe {
        let mut p_dxgi_factory: *mut IDXGIFactory4 = ptr::null_mut();
        let hr = CreateDXGIFactory(&IDXGIFactory4::uuidof(),
                                   &mut p_dxgi_factory as *mut _ as *mut _);
        check_hresult!(hr, CreateDXGIFactory)?;
        ComPtr::from_raw(p_dxgi_factory)
    };

    let warp_adapter: ComPtr<IDXGIAdapter>;
    unsafe {
        let mut p_adapter: *mut IDXGIAdapter = ptr::null_mut();
        let hr = dxgi_factory.EnumWarpAdapter(&IDXGIAdapter::uuidof(),
                                              &mut p_adapter as *mut _ as *mut _);
        check_hresult!(hr, IDXGIFactory4::EnumWarpAdapter)?;
        warp_adapter = ComPtr::from_raw(p_adapter);
    }

    let feature_level: u32 = match matches.value_of("feature-level").unwrap() {
        "11" | "11.0" | "11_0" => D3D_FEATURE_LEVEL_11_0,
               "11.1" | "11_1" => D3D_FEATURE_LEVEL_11_1,
        "12" | "12.0" | "12_0" => D3D_FEATURE_LEVEL_12_0,
               "12.1" | "12_1" => D3D_FEATURE_LEVEL_12_1,
        text                   => {
            panic!("Unrecognized feature level \"{}\": This is a bug.", text);
        },
    };

    let device: ComPtr<ID3D12Device> = unsafe {
        // We'll either use the default adapter (NULL), or the software renderer
        // if the user asked for that.
        let p_adapter = if matches.is_present("warp") {
            // TODO: Does CreateDevice take ownership of the adapter
            //       we give it?
            warp_adapter.as_raw()
        } else {
            ptr::null()
        };

        let mut p_device: *mut ID3D12Device = ptr::null_mut();
        let hr = D3D12CreateDevice(p_adapter as *mut IUnknown,
                                   feature_level,
                                   &ID3D12Device::uuidof(),
                                   &mut p_device as *mut _ as *mut _);
        check_hresult!(hr, D3D12CreateDevice)?;
        ComPtr::from_raw(p_device)
    };

    let fence: ComPtr<ID3D12Fence>;
    unsafe {
        let mut p_fence: *mut ID3D12Fence = ptr::null_mut();
        let hr = device.CreateFence(0,
                                    D3D12_FENCE_FLAG_NONE,
                                    &ID3D12Fence::uuidof(),
                                    &mut p_fence as *mut _ as *mut _);
        check_hresult!(hr, D3D12Device::CreateFence)?;
        fence = ComPtr::from_raw(p_fence);
    }

    unsafe {
        let rtv_desc_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
        let dsv_desc_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_DSV);
        let cbv_srv_desc_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
        println!("rtv_desc_size     = {}", rtv_desc_size);
        println!("dsv_desc_size     = {}", dsv_desc_size);
        println!("cbv_srv_desc_size = {}", cbv_srv_desc_size);
    }

    let mut ms_quality = D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
        Format:             DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        SampleCount:        4,
        Flags:              D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_NONE,
        NumQualityLevels:   0,
    };
    unsafe {
        let hr = device.CheckFeatureSupport(D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS,
                                            &mut ms_quality as *mut _ as *mut _,
                                            mem::size_of_val(&ms_quality) as u32);
        check_hresult!(hr, D3D12Device::CheckFeatureSupport)?;
    }
    println!("{:#?}", ms_quality);

    Ok(())
}
