#![allow(non_snake_case)]

extern crate portability_gfx;

use portability_gfx::*;

#[no_mangle]
pub extern fn vkCreateInstance(
    pCreateInfo: *const VkInstanceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult {
    gfxCreateInstance(pCreateInfo, pAllocator, pInstance)
}

#[no_mangle]
pub extern fn vkDestroyInstance(
    instance: VkInstance,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyInstance(instance, pAllocator)
}

#[no_mangle]
pub extern fn vkEnumeratePhysicalDevices(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult {
    gfxEnumeratePhysicalDevices(instance, pPhysicalDeviceCount, pPhysicalDevices)
}

#[no_mangle]
pub extern fn vkGetPhysicalDeviceQueueFamilyProperties(
    adapter: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
) {
    gfxGetPhysicalDeviceQueueFamilyProperties(adapter, pQueueFamilyPropertyCount, pQueueFamilyProperties)
}
#[no_mangle]
pub extern fn vkGetPhysicalDeviceMemoryProperties(
    physicalDevice: VkPhysicalDevice,
    pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties,
) {
    gfxGetPhysicalDeviceMemoryProperties(physicalDevice, pMemoryProperties)
}
#[no_mangle]
pub extern fn vkCreateDevice(
    adapter: VkPhysicalDevice,
    pCreateInfo: *const VkDeviceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDevice: *mut VkDevice,
) -> VkResult {
    gfxCreateDevice(adapter, pCreateInfo, pAllocator, pDevice)
}
#[no_mangle]
pub extern fn vkAllocateMemory(
    device: VkDevice,
    pAllocateInfo: *const VkMemoryAllocateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pMemory: *mut VkDeviceMemory,
) -> VkResult {
    gfxAllocateMemory(device, pAllocateInfo, pAllocator, pMemory)
}
#[no_mangle]
pub extern fn vkFreeMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxFreeMemory(device, memory, pAllocator)
}
#[no_mangle]
pub extern fn vkBindImageMemory(
    device: VkDevice,
    image: VkImage,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    gfxBindImageMemory(device, image, memory, memoryOffset)
}
#[no_mangle]
pub extern fn vkBindBufferMemory(
    device: VkDevice,
    buffer: VkBuffer,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    gfxBindBufferMemory(device, buffer, memory, memoryOffset)
}
#[no_mangle]
pub extern fn vkDestroyDevice(
    device: VkDevice,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyDevice(device, pAllocator)
}
#[no_mangle]
pub extern fn vkCreateImage(
    device: VkDevice,
    pCreateInfo: *const VkImageCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pImage: *mut VkImage,
) -> VkResult {
    gfxCreateImage(device, pCreateInfo, pAllocator, pImage)
}
#[no_mangle]
pub extern fn vkCreateImageView(
    device: VkDevice,
    pCreateInfo: *const VkImageViewCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pView: *mut VkImageView,
) -> VkResult {
    gfxCreateImageView(device, pCreateInfo, pAllocator, pView)
}
#[no_mangle]
pub extern fn vkGetImageMemoryRequirements(
    device: VkDevice,
    image: VkImage,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    gfxGetImageMemoryRequirements(device, image, pMemoryRequirements)
}
#[no_mangle]
pub extern fn vkDestroyImageView(
    device: VkDevice,
    imageView: VkImageView,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyImageView(device, imageView, pAllocator)
}
#[no_mangle]
pub extern fn vkGetPhysicalDeviceFormatProperties(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    pFormatProperties: *mut VkFormatProperties,
) {
    gfxGetPhysicalDeviceFormatProperties(adapter, format, pFormatProperties)
}

#[no_mangle]
pub extern fn vkCreateCommandPool(
    device: VkDevice,
    pCreateInfo: *const VkCommandPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pCommandPool: *mut VkCommandPool,
) -> VkResult {
    gfxCreateCommandPool(device, pCreateInfo, pAllocator, pCommandPool)
}

#[no_mangle]
pub extern fn vkDestroyCommandPool(
    device: VkDevice,
    commandPool: VkCommandPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyCommandPool(device, commandPool, pAllocator)
}

#[no_mangle]
pub extern fn vkResetCommandPool(
    device: VkDevice,
    commandPool: VkCommandPool,
    flags: VkCommandPoolResetFlags,
) -> VkResult {
    gfxResetCommandPool(device, commandPool, flags)
}

#[no_mangle]
pub extern fn vkAllocateCommandBuffers(
    device: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult {
    gfxAllocateCommandBuffers(device, pAllocateInfo, pCommandBuffers)
}

#[no_mangle]
pub extern fn vkFreeCommandBuffers(
    device: VkDevice,
    commandPool: VkCommandPool,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    gfxFreeCommandBuffers(device, commandPool, commandBufferCount, pCommandBuffers)
}

#[no_mangle]
pub extern fn vkDestroySurfaceKHR(
    instance: VkInstance,
    surface: VkSurfaceKHR,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroySurfaceKHR(instance, surface, pAllocator)
}

#[no_mangle]
pub extern fn vkGetPhysicalDeviceSurfaceSupportKHR(
    adapter: VkPhysicalDevice,
    queueFamilyIndex: u32,
    surface: VkSurfaceKHR,
    pSupported: *mut VkBool32,
) -> VkResult {
    gfxGetPhysicalDeviceSurfaceSupportKHR(adapter, queueFamilyIndex, surface, pSupported)
}

#[no_mangle]
pub extern fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceCapabilities: *mut VkSurfaceCapabilitiesKHR,
) -> VkResult {
    gfxGetPhysicalDeviceSurfaceCapabilitiesKHR(adapter, surface, pSurfaceCapabilities)
}

#[no_mangle]
pub extern fn vkGetPhysicalDeviceSurfaceFormatsKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceFormatCount: *mut u32,
    pSurfaceFormats: *mut VkSurfaceFormatKHR,
) -> VkResult {
    gfxGetPhysicalDeviceSurfaceFormatsKHR(adapter, surface, pSurfaceFormatCount, pSurfaceFormats)
}

#[no_mangle]
pub extern fn vkGetPhysicalDeviceSurfacePresentModesKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pPresentModeCount: *mut u32,
    pPresentModes: *mut VkPresentModeKHR,
) -> VkResult {
    gfxGetPhysicalDeviceSurfacePresentModesKHR(adapter, surface, pPresentModeCount, pPresentModes)
}

#[no_mangle]
pub extern fn vkCreateSwapchainKHR(
    device: VkDevice,
    pCreateInfo: *const VkSwapchainCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSwapchain: *mut VkSwapchainKHR,
) -> VkResult {
    gfxCreateSwapchainKHR(device, pCreateInfo, pAllocator, pSwapchain)
}
#[no_mangle]
pub extern fn vkDestroySwapchainKHR(
    device: VkDevice,
    swapchain: VkSwapchainKHR,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroySwapchainKHR(device, swapchain, pAllocator)
}
#[no_mangle]
pub extern fn vkGetSwapchainImagesKHR(
    device: VkDevice,
    swapchain: VkSwapchainKHR,
    pSwapchainImageCount: *mut u32,
    pSwapchainImages: *mut VkImage,
) -> VkResult {
    gfxGetSwapchainImagesKHR(device, swapchain, pSwapchainImageCount, pSwapchainImages)
}

#[no_mangle]
pub extern fn vkCreateWin32SurfaceKHR(
    instance: VkInstance,
    pCreateInfos: *const VkWin32SurfaceCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    gfxCreateWin32SurfaceKHR(instance, pCreateInfos, pAllocator, pSurface)
}

#[no_mangle]
pub extern fn vkMapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    flags: VkMemoryMapFlags,
    ppData: *mut *mut ::std::os::raw::c_void,
) -> VkResult {
    gfxMapMemory(
        device,
        memory,
        offset,
        size,
        flags,
        ppData,
    )
}

#[no_mangle]
pub extern fn vkUnmapMemory(device: VkDevice, memory: VkDeviceMemory) {
    gfxUnmapMemory(device, memory)
}

#[no_mangle]
pub extern fn vkDestroyImage(
    device: VkDevice,
    image: VkImage,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyImage(device, image, pAllocator)
}

#[no_mangle]
pub extern fn vkDestroyBuffer(
    device: VkDevice,
    buffer: VkBuffer,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyBuffer(device, buffer, pAllocator)
}

#[no_mangle]
pub extern fn vkGetBufferMemoryRequirements(
    device: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    gfxGetBufferMemoryRequirements(device, buffer, pMemoryRequirements)
}
