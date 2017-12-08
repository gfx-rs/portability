#![allow(non_snake_case)]

extern crate portability_gfx;

use portability_gfx::*;

use std::ffi::CStr;
use std::mem;
use std::ptr;

const ICD_VERSION: u32 = 5;

macro_rules! proc_addr {
    ($name:expr, $($vk:pat => $gfx:expr),*) => (
        match $name {
            $(
                stringify!($vk) => unsafe { mem::transmute::<_, PFN_vkVoidFunction>($gfx as *const ()) }
            ),*
            _ => None
        }
    );
}

#[no_mangle]
pub extern fn vk_icdGetInstanceProcAddr(
    instance: VkInstance, pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    let name = unsafe { CStr::from_ptr(pName) };
    let name = match name.to_str() {
        Ok(name) => name,
        Err(_) => return None,
    };

    proc_addr!{ name,
        vkCreateInstance => gfxCreateInstance,
        vkEnumerateInstanceExtensionProperties => gfxEnumerateInstanceExtensionProperties
    }
}

#[no_mangle]
pub extern fn vk_icdNegotiateLoaderICDInterfaceVersion(
    pSupportedVersion: *mut ::std::os::raw::c_uint,
) -> VkResult {
    let supported_version = unsafe { &mut *pSupportedVersion };
    if *supported_version > ICD_VERSION {
        *supported_version = ICD_VERSION;
    }

    VkResult::VK_SUCCESS
}

#[no_mangle]
pub extern fn vk_icdGetPhysicalDeviceProcAddr(
    instance: VkInstance, pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    unimplemented!()
}

