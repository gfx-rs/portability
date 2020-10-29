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
    if *pSupportedVersion > ICD_VERSION {
        *pSupportedVersion = ICD_VERSION;
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
        vkGetPhysicalDeviceProperties2KHR, PFN_vkGetPhysicalDeviceProperties2KHR => gfxGetPhysicalDeviceProperties2KHR,
        vkGetPhysicalDeviceFormatProperties, PFN_vkGetPhysicalDeviceFormatProperties => gfxGetPhysicalDeviceFormatProperties,
        vkGetPhysicalDeviceFormatProperties2KHR, PFN_vkGetPhysicalDeviceFormatProperties2KHR => gfxGetPhysicalDeviceFormatProperties2KHR,
        vkGetPhysicalDeviceImageFormatProperties, PFN_vkGetPhysicalDeviceImageFormatProperties => gfxGetPhysicalDeviceImageFormatProperties,
        vkGetPhysicalDeviceImageFormatProperties2KHR, PFN_vkGetPhysicalDeviceImageFormatProperties2KHR => gfxGetPhysicalDeviceImageFormatProperties2KHR,
        vkGetPhysicalDeviceMemoryProperties, PFN_vkGetPhysicalDeviceMemoryProperties => gfxGetPhysicalDeviceMemoryProperties,
        vkGetPhysicalDeviceMemoryProperties2KHR, PFN_vkGetPhysicalDeviceMemoryProperties2KHR => gfxGetPhysicalDeviceMemoryProperties2KHR,
        vkGetPhysicalDeviceQueueFamilyProperties, PFN_vkGetPhysicalDeviceQueueFamilyProperties => gfxGetPhysicalDeviceQueueFamilyProperties,
        vkGetPhysicalDeviceQueueFamilyProperties2KHR, PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR => gfxGetPhysicalDeviceQueueFamilyProperties2KHR,
        vkGetPhysicalDeviceSparseImageFormatProperties, PFN_vkGetPhysicalDeviceSparseImageFormatProperties => gfxGetPhysicalDeviceSparseImageFormatProperties,
        vkGetPhysicalDeviceSparseImageFormatProperties2KHR, PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR => gfxGetPhysicalDeviceSparseImageFormatProperties2KHR,

        vkGetPhysicalDeviceSurfaceSupportKHR, PFN_vkGetPhysicalDeviceSurfaceSupportKHR => gfxGetPhysicalDeviceSurfaceSupportKHR,
        vkGetPhysicalDeviceSurfaceCapabilitiesKHR, PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR => gfxGetPhysicalDeviceSurfaceCapabilitiesKHR,
        vkGetPhysicalDeviceSurfaceFormatsKHR, PFN_vkGetPhysicalDeviceSurfaceFormatsKHR => gfxGetPhysicalDeviceSurfaceFormatsKHR,
        vkGetPhysicalDeviceSurfacePresentModesKHR, PFN_vkGetPhysicalDeviceSurfacePresentModesKHR => gfxGetPhysicalDeviceSurfacePresentModesKHR,
        vkGetPhysicalDeviceWin32PresentationSupportKHR, PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR => gfxGetPhysicalDeviceWin32PresentationSupportKHR,
    }
}
