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

    let _r = renderer::Renderer::create(&conf)?;

    Ok(())
}

fn main2() -> Result<(), error::WindowsError> {
    let conf = config::Config::load();
    println!("{:#?}\n", conf);

    let d3d12_debug: ComPtr<ID3D12Debug> = unsafe {
        let mut p_debug: *mut ID3D12Debug = ptr::null_mut();
        hr!(D3D12GetDebugInterface(&ID3D12Debug::uuidof(),
                                        &mut p_debug as *mut _ as *mut _))?;
        ComPtr::from_raw(p_debug)
    };

    if conf.enable_debug {
        unsafe { d3d12_debug.EnableDebugLayer(); }
    }

    let h_wnd = win32_window::init_window("Dx12?")?;

    let dxgi_factory: ComPtr<IDXGIFactory4> = unsafe {
        let mut p_dxgi_factory: *mut IDXGIFactory4 = ptr::null_mut();
        hr!(CreateDXGIFactory(&IDXGIFactory4::uuidof(),
                                   &mut p_dxgi_factory as *mut _ as *mut _))?;
        ComPtr::from_raw(p_dxgi_factory)
    };

    // Load the warp adapter
    let warp_adapter: ComPtr<IDXGIAdapter> = unsafe {
        let mut p_adapter: *mut IDXGIAdapter = ptr::null_mut();
        hr!(dxgi_factory.EnumWarpAdapter(&IDXGIAdapter::uuidof(),
                                              &mut p_adapter as *mut _ as *mut _))?;
        ComPtr::from_raw(p_adapter)
    };

    // Load all the other adapters
    let mut adapters: Vec<ComPtr<IDXGIAdapter3>> = vec![];
    unsafe {
        let mut i = 0;
        loop {
            let mut adapter: *mut IDXGIAdapter = ptr::null_mut();
            let hr = dxgi_factory.EnumAdapters(i, &mut adapter as *mut _);
            if hr == winerror::DXGI_ERROR_NOT_FOUND {
                break;
            }
            hr!(hr)?;
            i += 1;
            adapters.push(ComPtr::from_raw(adapter as *mut _));
        }
    }

    for (adapter, i) in adapters.iter().zip(1..) {
        unsafe {
            let mut desc: DXGI_ADAPTER_DESC = mem::zeroed();
            hr!(adapter.GetDesc(&mut desc as *mut _))?;
            println!("Adapter {}:", i);
            // println!("    Description:           {}", description);
            println!("    VendorId:              0x{:x}", desc.VendorId);
            println!("    DeviceId:              0x{:x}", desc.DeviceId);
            println!("    SubSysId:              0x{:x}", desc.SubSysId);
            println!("    Revision:              {}",     desc.Revision);
            println!("    DedicatedVideoMemory:  0x{:x}", desc.DedicatedVideoMemory);
            println!("    DedicatedSystemMemory: 0x{:x}", desc.DedicatedSystemMemory);
            println!("    SharedSystemMemory:    0x{:x}", desc.SharedSystemMemory);
        }
    }

    let device: ComPtr<ID3D12Device> = unsafe {
        // We'll either use the default adapter (NULL), or the software renderer
        // if the user asked for that.
        let p_adapter = if conf.force_warp {
            warp_adapter.as_raw()
        } else {
            ptr::null()
        };

        let mut p_device: *mut ID3D12Device = ptr::null_mut();
        hr!(D3D12CreateDevice(p_adapter as *mut IUnknown,
                                   conf.feature_level.into(),
                                   &ID3D12Device::uuidof(),
                                   &mut p_device as *mut _ as *mut _))?;
        ComPtr::from_raw(p_device)
    };

    let _fence: ComPtr<ID3D12Fence> = unsafe {
        let mut p_fence: *mut ID3D12Fence = ptr::null_mut();
        hr!(device.CreateFence(0,
                                    D3D12_FENCE_FLAG_NONE,
                                    &ID3D12Fence::uuidof(),
                                    &mut p_fence as *mut _ as *mut _))?;
        ComPtr::from_raw(p_fence)
    };

    // This is arbitrary right now.
    let backbuffer_format = DXGI_FORMAT_R8G8B8A8_UNORM_SRGB;
    unsafe {
        let rtv_desc_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
        let dsv_desc_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_DSV);
        let cbv_srv_desc_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
        println!("rtv_desc_size     = {}", rtv_desc_size);
        println!("dsv_desc_size     = {}", dsv_desc_size);
        println!("cbv_srv_desc_size = {}", cbv_srv_desc_size);
        println!("");
    }

    let ms_quality: u32;
    {
        let mut multisample_quality = D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
            Format:             backbuffer_format,
            SampleCount:        4,
            Flags:              D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_NONE,
            NumQualityLevels:   0,
        };
        unsafe {
            hr!(device.CheckFeatureSupport(D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS,
                                                &mut multisample_quality as *mut _ as *mut _,
                                                mem::size_of_val(&multisample_quality) as u32))?;
        };
        println!("{:#?}\n", multisample_quality);
        ms_quality = multisample_quality.NumQualityLevels;
    }

    let gpu_va;
    {
        let mut gpu_va_info: D3D12_FEATURE_DATA_GPU_VIRTUAL_ADDRESS_SUPPORT  ;
        unsafe {
            gpu_va_info = mem::zeroed();
            hr!(device.CheckFeatureSupport(D3D12_FEATURE_GPU_VIRTUAL_ADDRESS_SUPPORT,
                                                &mut gpu_va_info as *mut _ as *mut _,
                                                mem::size_of_val(&gpu_va_info) as u32))?;
        };
        gpu_va = gpu_va_info;
    }
    println!("{:#?}\n", gpu_va);

    let vidmem: DXGI_QUERY_VIDEO_MEMORY_INFO;
    unsafe {
        let mut vidmem_info: DXGI_QUERY_VIDEO_MEMORY_INFO = mem::zeroed();
        hr!(adapters[0].QueryVideoMemoryInfo(0, // Node index
                                                  DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                                                  &mut vidmem_info as *mut _))?;
        vidmem = vidmem_info;
    }
    println!("{:#?}\n", vidmem);

    //
    // ---- Create command objects ------------
    //
    let cmd_queue: ComPtr<ID3D12CommandQueue> = unsafe {
        let queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            ..mem::zeroed()
        };

        let mut p_cmd_queue: *mut ID3D12CommandQueue = ptr::null_mut();
        hr!(device.CreateCommandQueue(&queue_desc,
                                           &ID3D12CommandQueue::uuidof(),
                                           &mut p_cmd_queue as *mut _ as *mut _))?;
        ComPtr::from_raw(p_cmd_queue)
    };

    let cmd_alloc: ComPtr<ID3D12CommandAllocator> = unsafe {
        let mut p_cmd_alloc: *mut ID3D12CommandAllocator = ptr::null_mut();
        hr!(device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT,
                                               &ID3D12CommandAllocator::uuidof(),
                                               &mut p_cmd_alloc as *mut _ as *mut _))?;
        ComPtr::from_raw(p_cmd_alloc)
    };

    let _gfx_cmd_list: ComPtr<ID3D12GraphicsCommandList> = unsafe {
        let mut p_gfx_cmd_list: *mut ID3D12GraphicsCommandList = ptr::null_mut();
        hr!(device.CreateCommandList(0, // node mask
                                          D3D12_COMMAND_LIST_TYPE_DIRECT,
                                          cmd_alloc.as_raw(),
                                          ptr::null_mut(), // Initial PSO
                                          &ID3D12GraphicsCommandList::uuidof(),
                                          &mut p_gfx_cmd_list as *mut _ as *mut _))?;
        ComPtr::from_raw(p_gfx_cmd_list)
    };

    let dxgi_debug: ComPtr<IDXGIDebug>;
    if conf.enable_debug {
        unsafe {
            let mut p_dxgi_debug: *mut IDXGIDebug = ptr::null_mut();
            hr!(DXGIGetDebugInterface1(0, // flags, unused
                                            &IDXGIDebug::uuidof(),
                                            &mut p_dxgi_debug as *mut _ as *mut _))?;
            // MS Docs:
            //      The DXGIGetDebugInterface1 function returns E_NOINTERFACE on
            //      systems without the Windows Software Development Kit (SDK)
            //      installed, because it's a development-time aid.
            // So we report but ignore an error here.
            dxgi_debug = ComPtr::from_raw(p_dxgi_debug);
            dxgi_debug.ReportLiveObjects(DXGI_DEBUG_ALL, DXGI_DEBUG_RLO_ALL);
        }
    }

    let mut swapchain_desc = DXGI_SWAP_CHAIN_DESC {
        BufferDesc: DXGI_MODE_DESC {
            Width:  1024,
            Height: 1024,
            RefreshRate: DXGI_RATIONAL { Numerator: 60, Denominator: 1},
            Format: backbuffer_format,
            ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
            Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
        },
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: ms_quality,
            Quality: ms_quality-1,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 3, // swapchainBufferCount
        OutputWindow: h_wnd,
        Windowed: 1,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH,
    };
    println!("{:#?}\n", swapchain_desc);

    let _swapchain: ComPtr<IDXGISwapChain> = unsafe {
        let mut p_swapchain: *mut IDXGISwapChain = ptr::null_mut();
        hr!(dxgi_factory.CreateSwapChain(cmd_queue.as_raw() as *mut _,
                                              &mut swapchain_desc,
                                              &mut p_swapchain))?;
        ComPtr::from_raw(p_swapchain)
    };

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
