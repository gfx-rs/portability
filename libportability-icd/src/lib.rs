#![allow(non_snake_case)]

extern crate portability_gfx;

use portability_gfx::*;

use std::ffi::CStr;
use std::mem;
use std::ptr;

const ICD_VERSION: u32 = 5;

#[no_mangle]
pub extern "C" fn vk_icdGetInstanceProcAddr(
    instance: VkInstance,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    gfxGetInstanceProcAddr(instance, pName)
}

#[no_mangle]
pub extern "C" fn vk_icdNegotiateLoaderICDInterfaceVersion(
    pSupportedVersion: *mut ::std::os::raw::c_uint,
) -> VkResult {
    let supported_version = unsafe { &mut *pSupportedVersion };
    if *supported_version > ICD_VERSION {
        *supported_version = ICD_VERSION;
    }

    VkResult::VK_SUCCESS
}

#[no_mangle]
pub extern "C" fn vk_icdGetPhysicalDeviceProcAddr(
    instance: VkInstance,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    gfxGetPhysicslDeviceProcAddr(instance, pName)
}
