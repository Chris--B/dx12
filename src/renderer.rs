
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
    shared::winerror::HRESULT,
    shared::ntdef::HANDLE,
    um::winuser,

    // These functions include a namespace in their names, so we won't
    // double-namespace them.
    // e.g. `d3d12::D3D12CreateDevice`
    shared::dxgi1_4::*,
    shared::dxgi::*,
    shared::dxgiformat::*,
    shared::dxgitype::*,

    um::d3d12::*,
    um::d3d12sdklayers::*,
    um::dxgidebug::*,
};

use config;
use error::*;

const FRAME_COUNT: usize = 3;

enum Vendor {
    Amd,            // 0x1002
    Imgtec,         // 0x1010
    Nvidia,         // 0x10DE
    Arm,            // 0x13B5
    Qualcomm,       // 0x5143
    Intel,          // 0x8086
    Microsoft,      // 0x1414
    Unknown(u32),
}

fn vid_to_vendor(vid: u32) -> Vendor {
    match vid {
        0x1002 => Vendor::Amd,
        0x1010 => Vendor::Imgtec,
        0x10DE => Vendor::Nvidia,
        0x13B5 => Vendor::Arm,
        0x5143 => Vendor::Qualcomm,
        0x8086 => Vendor::Intel,
        0x1414 => Vendor::Microsoft,
        _      => Vendor::Unknown(vid),
    }
}

impl ::std::fmt::Display for Vendor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Vendor::Amd          => write!(f, "Amd"),
            Vendor::Imgtec       => write!(f, "Imgtec"),
            Vendor::Nvidia       => write!(f, "Nvidia"),
            Vendor::Arm          => write!(f, "Arm"),
            Vendor::Qualcomm     => write!(f, "Qualcomm"),
            Vendor::Intel        => write!(f, "Intel"),
            Vendor::Microsoft    => write!(f, "Microsoft"),
            Vendor::Unknown(vid) => write!(f, "Unknown (0x{:x})", vid),
        }
    }
}

#[repr(C)]
pub struct Vertex {
    position: [f32; 3],
}

#[derive(Debug, Copy, Clone)]
pub struct RendererOptions {
    use_warp: bool,
    use_debug: bool,
}

impl Default for RendererOptions {
    fn default() -> RendererOptions {
        RendererOptions { use_warp: false, use_debug: true }
    }
}

struct PerFrame {
    rt_view:                        ComPtr<ID3D12Resource>,
}

pub struct Renderer {
    // ---- Pipeline Objects --------
    viewport:                       D3D12_VIEWPORT,
    scissor:                        D3D12_RECT,
    swapchain:                      ComPtr<IDXGISwapChain3>,
    device:                         ComPtr<ID3D12Device>,
    per_frame:                      [PerFrame; FRAME_COUNT],
    cmd_alloc:                      ComPtr<ID3D12CommandAllocator>,
    cmd_queue:                      ComPtr<ID3D12CommandQueue>,
    root_sig:                       ComPtr<ID3D12RootSignature>,
    rtv_heap:                       ComPtr<ID3D12DescriptorHeap>,
    pso:                            ComPtr<ID3D12PipelineState>,
    gfx_cmd_list:                   ComPtr<ID3D12GraphicsCommandList>,
    rtvd_size:                      u32,

    // ---- Resources --------
    vertex_buf:                     ComPtr<ID3D12Resource>,
    vertex_view:                    D3D12_VERTEX_BUFFER_VIEW,

    // ---- Synchronization Objects --------
    frame_idx:                      usize,
    fence_event:                    HANDLE,
    fence:                          ComPtr<ID3D12Fence>,
    fence_value:                    u64,
}

impl Renderer {
    /// Initialize a renderer, or return an error describing why we couldn't.
    pub fn create(config: &config::Config) -> Result<Renderer, WindowsError> {
        if config.enable_debug {
            init_debug_objects()?;
        }

        let dxgi_factory = init_dxgi_factory()?;

        let warp_adapter = init_warp_adapter(&dxgi_factory)?;
        let adapters = enum_adapters(&dxgi_factory)?;

        let adapter: ComPtr<IDXGIAdapter>;
        if config.force_warp {
            adapter = warp_adapter;
        } else {
            adapter = adapters[0].clone().cast()?;
        }
        let device = init_device(&adapter, config.feature_level)?;

        let fence = create_fence(&device, D3D12_FENCE_FLAG_NONE)?;

        let rtvd_size = unsafe {
            device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
        };

        // This is arbitrary right now.
        let backbuffer_format = DXGI_FORMAT_R8G8B8A8_UNORM_SRGB;

        let mut ms_quality = D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
            Format:             backbuffer_format,
            SampleCount:        4,
            Flags:              D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_NONE,
            NumQualityLevels:   0,
        };
        check_feature_multisample_quality(&device, &mut ms_quality)?;
        println!("MS Quality: {}", ms_quality.NumQualityLevels);

        let cmd_queue = init_cmd_queue(&device)?;
        let cmd_alloc = init_cmd_alloc(&device)?;
        let gfx_cmd_list = init_gfx_cmd_list(&device, &cmd_alloc)?;

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
                Count: ms_quality.NumQualityLevels,
                Quality: ms_quality.NumQualityLevels-1,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: FRAME_COUNT as u32,
            OutputWindow: ptr::null_mut(), // h_wnd,
            Windowed: 1,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH,
        };
        println!("{:#?}\n", swapchain_desc);
        let swapchain = init_swapchain(&dxgi_factory,
                                       &cmd_queue,
                                       swapchain_desc)?;

        Err(WindowsError::NotImplemented)
    }

    /// Update inter-frame state.
    pub fn update(&mut self) {
        // Update these things:
        //      - Constant buffers
        //      - Vertex buffers
        //      - Index buffers
        //      - Anything else?
    }

    /// Generate and present a single frame.
    pub fn render(&mut self) {
        // Populate the command list
        //   Reset self.cmd_alloc
        //   Reset the command list
        //   Set graphics root signature
        //   Set viewport and scissor rectangles
        //   Set a resource barrier - backbuffer is used as a rt
        //   Record commands
        //   Resource barrier - backbuffer presenting after cmd_list
        //   Close the command list
        // Exec command list
        // Present the frame
        // Wait for GPU to finish

        let _this_frame = &mut self.per_frame;
    }
}

impl Drop for Renderer {
    /// Uninitialize all resources owned by the renderer.
    fn drop(&mut self) {
        // All of our resources are wrapped in `ComPtr<>` and other RAII types,
        // so all that we really need to do is idle the GPU and make sure that
        // nothing is in use.
        // TODO: Wait on all GPU work to finish, then return.
    }
}

fn enum_adapters(dxgi_factory: &ComPtr<IDXGIFactory4>) -> WindowsResult<Vec<ComPtr<IDXGIAdapter3>>> {
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

            // Encooooodingggggggggggggggg
            let description = {
                use std::ffi::OsString;
                use std::os::windows::prelude::*;
                OsString::from_wide(&desc.Description)
                        .into_string()
                        .unwrap_or_else(|_err|
                            "<Invalid Description String>".into())
            };
            println!("    Description:           {}", description);

            println!("    Vendor:                {}",     vid_to_vendor(desc.VendorId));
            println!("    DeviceId:              0x{:x}", desc.DeviceId);
            println!("    SubSysId:              0x{:x}", desc.SubSysId);
            println!("    Revision:              {}",     desc.Revision);
            println!("    DedicatedVideoMemory:  0x{:x}", desc.DedicatedVideoMemory);
            println!("    DedicatedSystemMemory: 0x{:x}", desc.DedicatedSystemMemory);
            println!("    SharedSystemMemory:    0x{:x}", desc.SharedSystemMemory);
        }
    }

    Ok(adapters)
}

fn create_fence(device: &ComPtr<ID3D12Device>, flags: u32) -> WindowsResult<ComPtr<ID3D12Fence>> {
    unsafe {
        let mut p_fence: *mut ID3D12Fence = ptr::null_mut();
        hr!(device.CreateFence(0,
                               flags,
                               &ID3D12Fence::uuidof(),
                               &mut p_fence as *mut _ as *mut _))?;
        Ok(ComPtr::from_raw(p_fence))
    }
}

fn check_feature_multisample_quality(
        device: &ComPtr<ID3D12Device>,
        data:   &mut D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS)
    -> WindowsResult<()>
{
    unsafe {
        hr!(device
            .CheckFeatureSupport(D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS,
                                 data as *mut _ as *mut _,
                                 mem::size_of_val(data) as u32))?;
    };
    Ok(())
}

// Initialization is a lot, so we break it apart into named functions.
// You may notice some repetitive code: still working on how to make this pretty.

fn init_debug_objects() -> WindowsResult<()> {
    let d3d12_debug: ComPtr<ID3D12Debug> = unsafe {
        let mut ptr: *mut _ = ptr::null_mut();
        hr!(D3D12GetDebugInterface(&ID3D12Debug::uuidof(),
                                   &mut ptr as *mut _ as *mut _))?;
        ComPtr::from_raw(ptr)
    };
    unsafe { d3d12_debug.EnableDebugLayer(); }

    let dxgi_debug: ComPtr<IDXGIDebug> = unsafe {
        let mut ptr: *mut IDXGIDebug = ptr::null_mut();
        // MS Docs:
        //      The DXGIGetDebugInterface1 function returns E_NOINTERFACE on
        //      systems without the Windows Software Development Kit (SDK)
        //      installed, because it's a development-time aid.
        // So we report but ignore an error here.
        let _ = (|| -> WindowsResult<()> {
            use winapi::shared::dxgi1_3::DXGIGetDebugInterface1;
            hr!(DXGIGetDebugInterface1(0, // flags, unused
                                       &IDXGIDebug::uuidof(),
                                       &mut ptr as *mut _ as *mut _))?;
            Ok(())
        })();
        ComPtr::from_raw(ptr)
    };
    unsafe { dxgi_debug.ReportLiveObjects(DXGI_DEBUG_ALL, DXGI_DEBUG_RLO_ALL); }

    Ok(())
}

fn init_dxgi_factory() -> WindowsResult<ComPtr<IDXGIFactory4>> {
    unsafe {
        let mut ptr: *mut _ = ptr::null_mut();
        hr!(CreateDXGIFactory(&IDXGIFactory4::uuidof(),
                              &mut ptr as *mut _ as *mut _))?;
        Ok(ComPtr::from_raw(ptr))
    }
}

fn init_warp_adapter(dxgi_factory: &ComPtr<IDXGIFactory4>) -> WindowsResult<ComPtr<IDXGIAdapter>> {
    unsafe {
        let mut ptr: *mut _ = ptr::null_mut();
        hr!(dxgi_factory.EnumWarpAdapter(&IDXGIAdapter::uuidof(),
                                         &mut ptr as *mut _ as *mut _))?;
        Ok(ComPtr::from_raw(ptr))
    }
}

fn init_device(adapter: &ComPtr<IDXGIAdapter>,
               feature_level: ::config::Dx12FeatureLevel)
    -> WindowsResult<ComPtr<ID3D12Device>>
{
    unsafe {
        let mut ptr: *mut _ = ptr::null_mut();
        hr!(D3D12CreateDevice(adapter.as_raw() as *mut _,
                              feature_level.into(),
                              &ID3D12Device::uuidof(),
                              &mut ptr as *mut _ as *mut _))?;
        Ok(ComPtr::from_raw(ptr))
    }
}

fn init_cmd_queue(device: &ComPtr<ID3D12Device>)
     -> WindowsResult<ComPtr<ID3D12CommandQueue>>
{
    unsafe {
        let queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            ..mem::zeroed()
        };

        let mut ptr: *mut _ = ptr::null_mut();
        hr!(device.CreateCommandQueue(&queue_desc,
                                      &ID3D12CommandQueue::uuidof(),
                                      &mut ptr as *mut _ as *mut _))?;
        Ok(ComPtr::from_raw(ptr))
    }
}

fn init_cmd_alloc(device: &ComPtr<ID3D12Device>)
    -> WindowsResult<ComPtr<ID3D12CommandAllocator>>
{
    unsafe {
        let mut ptr: *mut _ = ptr::null_mut();
        hr!(device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT,
                                          &ID3D12CommandAllocator::uuidof(),
                                          &mut ptr as *mut _ as *mut _))?;
        Ok(ComPtr::from_raw(ptr))
    }
}

fn init_gfx_cmd_list(device:    &ComPtr<ID3D12Device>,
                     cmd_alloc: &ComPtr<ID3D12CommandAllocator>)
    -> WindowsResult<ComPtr<ID3D12CommandAllocator>>
{
    unsafe {
        let mut ptr: *mut _ = ptr::null_mut();
        hr!(device.CreateCommandList(0, // Node Mask
                                     D3D12_COMMAND_LIST_TYPE_DIRECT,
                                     cmd_alloc.as_raw(),
                                     ptr::null_mut(), // Initial PSO
                                     &ID3D12GraphicsCommandList::uuidof(),
                                     &mut ptr as *mut _ as *mut _))?;
        Ok(ComPtr::from_raw(ptr))
    }
}

fn init_swapchain(dxgi_factory: &ComPtr<IDXGIFactory4>,
                  cmd_queue:    &ComPtr<ID3D12CommandQueue>,
                  mut desc:      DXGI_SWAP_CHAIN_DESC)
    -> WindowsResult<ComPtr<IDXGISwapChain>>
{
    unsafe {
        let mut ptr: *mut _ = ptr::null_mut();
        hr!(dxgi_factory.CreateSwapChain(cmd_queue.as_raw() as *mut _,
                                         &mut desc,
                                         &mut ptr))?;
        Ok(ComPtr::from_raw(ptr))
    }
}
