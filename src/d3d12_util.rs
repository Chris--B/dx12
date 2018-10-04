use std::{
    mem,
};

use winapi::{
    shared::winerror,
    um::d3d12::*,
};

pub trait TTD3d12FeatureData {
    const FEATURE: u32;
    const NAME:    &'static str;
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_D3D12_OPTIONS {
    const FEATURE: u32       = D3D12_FEATURE_D3D12_OPTIONS;
    const NAME: &'static str = "D3d12 Options";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_ARCHITECTURE {
    const FEATURE: u32       = D3D12_FEATURE_ARCHITECTURE;
    const NAME: &'static str = "Architecture";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_FEATURE_LEVELS {
    const FEATURE: u32       = D3D12_FEATURE_FEATURE_LEVELS;
    const NAME: &'static str = "Feature Levels";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_FORMAT_SUPPORT {
    const FEATURE: u32       = D3D12_FEATURE_FORMAT_SUPPORT;
    const NAME: &'static str = "Format Support";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
    const FEATURE: u32       = D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS;
    const NAME: &'static str = "Multisample Quality Levels";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_FORMAT_INFO {
    const FEATURE: u32       = D3D12_FEATURE_FORMAT_INFO;
    const NAME: &'static str = "Format Info";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_GPU_VIRTUAL_ADDRESS_SUPPORT {
    const FEATURE: u32       = D3D12_FEATURE_GPU_VIRTUAL_ADDRESS_SUPPORT;
    const NAME: &'static str = "Gpu Virtual Address Support";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_SHADER_MODEL {
    const FEATURE: u32       = D3D12_FEATURE_SHADER_MODEL;
    const NAME: &'static str = "Shader Model";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_D3D12_OPTIONS1 {
    const FEATURE: u32       = D3D12_FEATURE_D3D12_OPTIONS1;
    const NAME: &'static str = "D3d12 Options1";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_ROOT_SIGNATURE {
    const FEATURE: u32       = D3D12_FEATURE_ROOT_SIGNATURE;
    const NAME: &'static str = "Root Signature";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_ARCHITECTURE1 {
    const FEATURE: u32       = D3D12_FEATURE_ARCHITECTURE1;
    const NAME: &'static str = "Architecture1";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_D3D12_OPTIONS2 {
    const FEATURE: u32       = D3D12_FEATURE_D3D12_OPTIONS2;
    const NAME: &'static str = "D3d12 Options2";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_SHADER_CACHE {
    const FEATURE: u32       = D3D12_FEATURE_SHADER_CACHE;
    const NAME: &'static str = "Shader Cache";
}

impl TTD3d12FeatureData for D3D12_FEATURE_DATA_COMMAND_QUEUE_PRIORITY {
    const FEATURE: u32       = D3D12_FEATURE_COMMAND_QUEUE_PRIORITY;
    const NAME: &'static str = "Command Queue Priority";
}

// ---- I think these are not included in winapi-rs? ---------------------------

#[cfg(feature="missing_d3d12_features")]
impl TTD3d12FeatureData for D3D12_FEATURE_DATA_PROTECTED_RESOURCE_SESSION_SUPPORT {
    const FEATURE: u32       = D3D12_FEATURE_PROTECTED_RESOURCE_SESSION_SUPPORT;
    const NAME: &'static str = "Protected Resource Session Support";
}

#[cfg(feature="missing_d3d12_features")]
impl TTD3d12FeatureData for D3D12_FEATURE_DATA_D3D12_OPTIONS3 {
    const FEATURE: u32       = D3D12_FEATURE_D3D12_OPTIONS3;
    const NAME: &'static str = "D3d12 Options3";
}

#[cfg(feature="missing_d3d12_features")]
impl TTD3d12FeatureData for D3D12_FEATURE_DATA_EXISTING_HEAPS {
    const FEATURE: u32       = D3D12_FEATURE_EXISTING_HEAPS;
    const NAME: &'static str = "Existing Heaps";
}

#[cfg(feature="missing_d3d12_features")]
impl TTD3d12FeatureData for D3D12_FEATURE_DATA_D3D12_OPTIONS4 {
    const FEATURE: u32       = D3D12_FEATURE_D3D12_OPTIONS4;
    const NAME: &'static str = "D3d12 Options4";
}

#[cfg(feature="missing_d3d12_features")]
impl TTD3d12FeatureData for D3D12_FEATURE_DATA_SERIALIZATION {
    const FEATURE: u32       = D3D12_FEATURE_SERIALIZATION;
    const NAME: &'static str = "Serialization";
}

#[cfg(feature="missing_d3d12_features")]
impl TTD3d12FeatureData for D3D12_FEATURE_DATA_CROSS_NODE {
    const FEATURE: u32       = D3D12_FEATURE_CROSS_NODE;
    const NAME: &'static str = "Cross Node";
}

// DXR
// See: https://docs.microsoft.com/en-us/windows/desktop/api/d3d12/ns-d3d12-d3d12_feature_data_d3d12_options5
#[cfg(feature="missing_d3d12_features")]
impl TTD3d12FeatureData for D3D12_FEATURE_DATA_OPTIONS5 {
    const FEATURE: u32       = D3D12_FEATURE_OPTIONS5;
    const NAME: &'static str = "Options5";
}

pub fn check_device_feature<Feature>(device: &ID3D12Device) -> Result<Feature, u32>
    where Feature: TTD3d12FeatureData
{
    unsafe {
        let mut data: Feature = mem::zeroed();
        let hr = device.CheckFeatureSupport(Feature::FEATURE,
                                            &mut data as *mut _ as *mut _,
                                            mem::size_of_val(&data) as u32);
        println!("NAME: {}", Feature::NAME);
        if winerror::SUCCEEDED(hr) {
            Ok(data)
        } else {
            Err(hr as u32)
        }
    }
}
