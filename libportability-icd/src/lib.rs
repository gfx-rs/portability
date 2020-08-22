#![allow(non_snake_case)]

use portability_gfx::*;

use std::ffi::CStr;
use std::mem;

const ICD_VERSION: u32 = 5;

#[no_mangle]
pub unsafe extern "C" fn vk_icdGetInstanceProcAddr(
    instance: VkInstance,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    gfxGetInstanceProcAddr(instance, pName)
}

#[no_mangle]
pub unsafe extern "C" fn vk_icdNegotiateLoaderICDInterfaceVersion(
    pSupportedVersion: *mut ::std::os::raw::c_uint,
) -> VkResult {
    let supported_version = &mut *pSupportedVersion;
    if *supported_version > ICD_VERSION {
        *supported_version = ICD_VERSION;
    }

    VkResult::VK_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn vk_icdGetPhysicalDeviceProcAddr(
    _instance: VkInstance,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    let name = CStr::from_ptr(pName);
    let name = match name.to_str() {
        Ok(name) => name,
        Err(_) => return None,
    };

    proc_addr! { name,
        vkGetPhysicalDeviceFeatures, PFN_vkGetPhysicalDeviceFeatures => gfxGetPhysicalDeviceFeatures,
        vkGetPhysicalDeviceFeatures2KHR, PFN_vkGetPhysicalDeviceFeatures2KHR => gfxGetPhysicalDeviceFeatures2KHR,
        vkGetPhysicalDeviceProperties, PFN_vkGetPhysicalDeviceProperties => gfxGetPhysicalDeviceProperties,
        vkGetPhysicalDeviceFormatProperties, PFN_vkGetPhysicalDeviceFormatProperties => gfxGetPhysicalDeviceFormatProperties,
        vkGetPhysicalDeviceImageFormatProperties, PFN_vkGetPhysicalDeviceImageFormatProperties => gfxGetPhysicalDeviceImageFormatProperties,
        vkGetPhysicalDeviceMemoryProperties, PFN_vkGetPhysicalDeviceMemoryProperties => gfxGetPhysicalDeviceMemoryProperties,
        vkGetPhysicalDeviceQueueFamilyProperties, PFN_vkGetPhysicalDeviceQueueFamilyProperties => gfxGetPhysicalDeviceQueueFamilyProperties,
        vkGetPhysicalDeviceSparseImageFormatProperties, PFN_vkGetPhysicalDeviceSparseImageFormatProperties => gfxGetPhysicalDeviceSparseImageFormatProperties,

        vkGetPhysicalDeviceSurfaceSupportKHR, PFN_vkGetPhysicalDeviceSurfaceSupportKHR => gfxGetPhysicalDeviceSurfaceSupportKHR,
        vkGetPhysicalDeviceSurfaceCapabilitiesKHR, PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR => gfxGetPhysicalDeviceSurfaceCapabilitiesKHR,
        vkGetPhysicalDeviceSurfaceFormatsKHR, PFN_vkGetPhysicalDeviceSurfaceFormatsKHR => gfxGetPhysicalDeviceSurfaceFormatsKHR,
        vkGetPhysicalDeviceSurfacePresentModesKHR, PFN_vkGetPhysicalDeviceSurfacePresentModesKHR => gfxGetPhysicalDeviceSurfacePresentModesKHR,
        vkGetPhysicalDeviceWin32PresentationSupportKHR, PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR => gfxGetPhysicalDeviceWin32PresentationSupportKHR,
    }
}
