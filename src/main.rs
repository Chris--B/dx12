#![feature(termination_trait_lib)]

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

struct U32HexWrapper(u32);

impl From<u32> for U32HexWrapper {
    fn from(num: u32) -> U32HexWrapper { U32HexWrapper(num) }
}

impl fmt::Debug for U32HexWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

fn main() -> Result<(), U32HexWrapper> {
    let conf = config::Config::load();
    println!("{:#?}\n", conf);

    let d3d12_debug: ComPtr<ID3D12Debug> = unsafe {
        let mut p_debug: *mut ID3D12Debug = ptr::null_mut();
        let hr = D3D12GetDebugInterface(&ID3D12Debug::uuidof(),
                                        &mut p_debug as *mut _ as *mut _);
        check_hresult!(hr, D3D12GetDebugInterface)?;
        ComPtr::from_raw(p_debug)
    };

    if conf.enable_debug {
        unsafe { d3d12_debug.EnableDebugLayer(); }
    }

    let h_wnd = win32_window::init_window("Dx12?")?;

    let dxgi_factory: ComPtr<IDXGIFactory4> = unsafe {
        let mut p_dxgi_factory: *mut IDXGIFactory4 = ptr::null_mut();
        let hr = CreateDXGIFactory(&IDXGIFactory4::uuidof(),
                                   &mut p_dxgi_factory as *mut _ as *mut _);
        check_hresult!(hr, CreateDXGIFactory)?;
        ComPtr::from_raw(p_dxgi_factory)
    };

    let warp_adapter: ComPtr<IDXGIAdapter> = unsafe {
        let mut p_adapter: *mut IDXGIAdapter = ptr::null_mut();
        let hr = dxgi_factory.EnumWarpAdapter(&IDXGIAdapter::uuidof(),
                                              &mut p_adapter as *mut _ as *mut _);
        check_hresult!(hr, IDXGIFactory4::EnumWarpAdapter)?;
        ComPtr::from_raw(p_adapter)
    };

    let device: ComPtr<ID3D12Device> = unsafe {
        // We'll either use the default adapter (NULL), or the software renderer
        // if the user asked for that.
        let p_adapter = if conf.force_warp {
            warp_adapter.as_raw()
        } else {
            ptr::null()
        };

        let mut p_device: *mut ID3D12Device = ptr::null_mut();
        let hr = D3D12CreateDevice(p_adapter as *mut IUnknown,
                                   conf.feature_level.into(),
                                   &ID3D12Device::uuidof(),
                                   &mut p_device as *mut _ as *mut _);
        check_hresult!(hr, D3D12CreateDevice)?;
        ComPtr::from_raw(p_device)
    };

    let _fence: ComPtr<ID3D12Fence> = unsafe {
        let mut p_fence: *mut ID3D12Fence = ptr::null_mut();
        let hr = device.CreateFence(0,
                                    D3D12_FENCE_FLAG_NONE,
                                    &ID3D12Fence::uuidof(),
                                    &mut p_fence as *mut _ as *mut _);
        check_hresult!(hr, ID3D12Device::CreateFence)?;
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

    let mut ms_quality = D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
        Format:             backbuffer_format,
        SampleCount:        4,
        Flags:              D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_NONE,
        NumQualityLevels:   0,
    };
    unsafe {
        let hr = device.CheckFeatureSupport(D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS,
                                            &mut ms_quality as *mut _ as *mut _,
                                            mem::size_of_val(&ms_quality) as u32);
        check_hresult!(hr, ID3D12Device::CheckFeatureSupport)?;
    };
    let ms_quality = ms_quality;
    println!("{:#?}\n", ms_quality);

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
        let hr = device.CreateCommandQueue(&queue_desc,
                                           &ID3D12CommandQueue::uuidof(),
                                           &mut p_cmd_queue as *mut _ as *mut _);
        check_hresult!(hr, ID3D12Device::CreateCommandQueue)?;
        ComPtr::from_raw(p_cmd_queue)
    };

    let cmd_alloc: ComPtr<ID3D12CommandAllocator> = unsafe {
        let mut p_cmd_alloc: *mut ID3D12CommandAllocator = ptr::null_mut();
        let hr = device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT,
                                               &ID3D12CommandAllocator::uuidof(),
                                               &mut p_cmd_alloc as *mut _ as *mut _);
        check_hresult!(hr, ID3D12Device::CreateCommandAllocator)?;
        ComPtr::from_raw(p_cmd_alloc)
    };

    let _gfx_cmd_list: ComPtr<ID3D12GraphicsCommandList> = unsafe {
        let mut p_gfx_cmd_list: *mut ID3D12GraphicsCommandList = ptr::null_mut();
        let hr = device.CreateCommandList(0, // node mask
                                          D3D12_COMMAND_LIST_TYPE_DIRECT,
                                          cmd_alloc.as_raw(),
                                          ptr::null_mut(), // Initial PSO
                                          &ID3D12GraphicsCommandList::uuidof(),
                                          &mut p_gfx_cmd_list as *mut _ as *mut _);
        check_hresult!(hr, ID3D12Device::CreateCommandList)?;
        ComPtr::from_raw(p_gfx_cmd_list)
    };

    let dxgi_debug: ComPtr<IDXGIDebug>;
    if conf.enable_debug {
        unsafe {
            let mut p_dxgi_debug: *mut IDXGIDebug = ptr::null_mut();
            let hr = DXGIGetDebugInterface1(0, // flags, unused
                                            &IDXGIDebug::uuidof(),
                                            &mut p_dxgi_debug as *mut _ as *mut _);
            // MS Docs:
            //      The DXGIGetDebugInterface1 function returns E_NOINTERFACE on
            //      systems without the Windows Software Development Kit (SDK)
            //      installed, because it's a development-time aid.
            // So we report but ignore an error here.
            let _ = check_hresult!(hr, DXGIGetDebugInterface1);
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
            Count: 4,
            Quality: 3,
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
        let hr = dxgi_factory.CreateSwapChain(cmd_queue.as_raw() as *mut _,
                                              &mut swapchain_desc,
                                              &mut p_swapchain);
        check_hresult!(hr, IDXGIFactory::CreateSwapChain)?;
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
