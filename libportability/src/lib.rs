#![allow(non_snake_case)]

use portability_gfx::*;

// These are only shims, reexporting the gfx functions with an vk prefix.
// IMPORTANT: These should only forward parameters to the gfx implementation,
//            don't include any further logic.

#[no_mangle]
pub unsafe extern "C" fn vkCreateInstance(
    pCreateInfo: *const VkInstanceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult {
    gfxCreateInstance(pCreateInfo, pAllocator, pInstance)
}

#[no_mangle]
pub unsafe extern "C" fn vkDestroyInstance(
    instance: VkInstance,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyInstance(instance, pAllocator)
}

#[no_mangle]
pub unsafe extern "C" fn vkEnumeratePhysicalDevices(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult {
    gfxEnumeratePhysicalDevices(instance, pPhysicalDeviceCount, pPhysicalDevices)
}

#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceQueueFamilyProperties(
    adapter: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
) {
    gfxGetPhysicalDeviceQueueFamilyProperties(
        adapter,
        pQueueFamilyPropertyCount,
        pQueueFamilyProperties,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceQueueFamilyProperties2KHR(
    adapter: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties2KHR,
) {
    gfxGetPhysicalDeviceQueueFamilyProperties2KHR(
        adapter,
        pQueueFamilyPropertyCount,
        pQueueFamilyProperties,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceMemoryProperties(
    physicalDevice: VkPhysicalDevice,
    pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties,
) {
    gfxGetPhysicalDeviceMemoryProperties(physicalDevice, pMemoryProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceMemoryProperties2KHR(
    physicalDevice: VkPhysicalDevice,
    pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties2KHR,
) {
    gfxGetPhysicalDeviceMemoryProperties2KHR(physicalDevice, pMemoryProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateDevice(
    adapter: VkPhysicalDevice,
    pCreateInfo: *const VkDeviceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDevice: *mut VkDevice,
) -> VkResult {
    gfxCreateDevice(adapter, pCreateInfo, pAllocator, pDevice)
}
#[no_mangle]
pub unsafe extern "C" fn vkAllocateMemory(
    device: VkDevice,
    pAllocateInfo: *const VkMemoryAllocateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pMemory: *mut VkDeviceMemory,
) -> VkResult {
    gfxAllocateMemory(device, pAllocateInfo, pAllocator, pMemory)
}
#[no_mangle]
pub unsafe extern "C" fn vkFreeMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxFreeMemory(device, memory, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkBindImageMemory(
    device: VkDevice,
    image: VkImage,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    gfxBindImageMemory(device, image, memory, memoryOffset)
}
#[no_mangle]
pub unsafe extern "C" fn vkBindBufferMemory(
    device: VkDevice,
    buffer: VkBuffer,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    gfxBindBufferMemory(device, buffer, memory, memoryOffset)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDevice(
    device: VkDevice,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyDevice(device, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateImage(
    device: VkDevice,
    pCreateInfo: *const VkImageCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pImage: *mut VkImage,
) -> VkResult {
    gfxCreateImage(device, pCreateInfo, pAllocator, pImage)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateImageView(
    device: VkDevice,
    pCreateInfo: *const VkImageViewCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pView: *mut VkImageView,
) -> VkResult {
    gfxCreateImageView(device, pCreateInfo, pAllocator, pView)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetImageMemoryRequirements(
    device: VkDevice,
    image: VkImage,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    gfxGetImageMemoryRequirements(device, image, pMemoryRequirements)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyImageView(
    device: VkDevice,
    imageView: VkImageView,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyImageView(device, imageView, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceFormatProperties(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    pFormatProperties: *mut VkFormatProperties,
) {
    gfxGetPhysicalDeviceFormatProperties(adapter, format, pFormatProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceFormatProperties2KHR(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    pFormatProperties: *mut VkFormatProperties2KHR,
) {
    gfxGetPhysicalDeviceFormatProperties2KHR(adapter, format, pFormatProperties)
}

#[no_mangle]
pub unsafe extern "C" fn vkCreateCommandPool(
    device: VkDevice,
    pCreateInfo: *const VkCommandPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pCommandPool: *mut VkCommandPool,
) -> VkResult {
    gfxCreateCommandPool(device, pCreateInfo, pAllocator, pCommandPool)
}

#[no_mangle]
pub unsafe extern "C" fn vkDestroyCommandPool(
    device: VkDevice,
    commandPool: VkCommandPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyCommandPool(device, commandPool, pAllocator)
}

#[no_mangle]
pub unsafe extern "C" fn vkResetCommandPool(
    device: VkDevice,
    commandPool: VkCommandPool,
    flags: VkCommandPoolResetFlags,
) -> VkResult {
    gfxResetCommandPool(device, commandPool, flags)
}

#[no_mangle]
pub unsafe extern "C" fn vkTrimCommandPoolKHR(
    device: VkDevice,
    commandPool: VkCommandPool,
    flags: VkCommandPoolTrimFlagsKHR,
) {
    gfxTrimCommandPoolKHR(device, commandPool, flags)
}

#[no_mangle]
pub unsafe extern "C" fn vkAllocateCommandBuffers(
    device: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult {
    gfxAllocateCommandBuffers(device, pAllocateInfo, pCommandBuffers)
}

#[no_mangle]
pub unsafe extern "C" fn vkFreeCommandBuffers(
    device: VkDevice,
    commandPool: VkCommandPool,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    gfxFreeCommandBuffers(device, commandPool, commandBufferCount, pCommandBuffers)
}

#[no_mangle]
pub unsafe extern "C" fn vkDestroySurfaceKHR(
    instance: VkInstance,
    surface: VkSurfaceKHR,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroySurfaceKHR(instance, surface, pAllocator)
}

#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceSurfaceSupportKHR(
    adapter: VkPhysicalDevice,
    queueFamilyIndex: u32,
    surface: VkSurfaceKHR,
    pSupported: *mut VkBool32,
) -> VkResult {
    gfxGetPhysicalDeviceSurfaceSupportKHR(adapter, queueFamilyIndex, surface, pSupported)
}

#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceCapabilities: *mut VkSurfaceCapabilitiesKHR,
) -> VkResult {
    gfxGetPhysicalDeviceSurfaceCapabilitiesKHR(adapter, surface, pSurfaceCapabilities)
}

#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceSurfaceFormatsKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceFormatCount: *mut u32,
    pSurfaceFormats: *mut VkSurfaceFormatKHR,
) -> VkResult {
    gfxGetPhysicalDeviceSurfaceFormatsKHR(adapter, surface, pSurfaceFormatCount, pSurfaceFormats)
}

#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceSurfacePresentModesKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pPresentModeCount: *mut u32,
    pPresentModes: *mut VkPresentModeKHR,
) -> VkResult {
    gfxGetPhysicalDeviceSurfacePresentModesKHR(adapter, surface, pPresentModeCount, pPresentModes)
}

#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceWin32PresentationSupportKHR(
    adapter: VkPhysicalDevice,
    queueFamilyIndex: u32,
) -> VkBool32 {
    gfxGetPhysicalDeviceWin32PresentationSupportKHR(adapter, queueFamilyIndex)
}

#[no_mangle]
pub unsafe extern "C" fn vkCreateSwapchainKHR(
    device: VkDevice,
    pCreateInfo: *const VkSwapchainCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSwapchain: *mut VkSwapchainKHR,
) -> VkResult {
    gfxCreateSwapchainKHR(device, pCreateInfo, pAllocator, pSwapchain)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroySwapchainKHR(
    device: VkDevice,
    swapchain: VkSwapchainKHR,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroySwapchainKHR(device, swapchain, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetSwapchainImagesKHR(
    device: VkDevice,
    swapchain: VkSwapchainKHR,
    pSwapchainImageCount: *mut u32,
    pSwapchainImages: *mut VkImage,
) -> VkResult {
    gfxGetSwapchainImagesKHR(device, swapchain, pSwapchainImageCount, pSwapchainImages)
}

#[no_mangle]
pub unsafe extern "C" fn vkCreateWin32SurfaceKHR(
    instance: VkInstance,
    pCreateInfos: *const VkWin32SurfaceCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    gfxCreateWin32SurfaceKHR(instance, pCreateInfos, pAllocator, pSurface)
}

#[no_mangle]
pub unsafe extern "C" fn vkCreateMacOSSurfaceMVK(
    instance: VkInstance,
    pCreateInfos: *const VkMacOSSurfaceCreateInfoMVK,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    gfxCreateMacOSSurfaceMVK(instance, pCreateInfos, pAllocator, pSurface)
}

#[no_mangle]
pub unsafe extern "C" fn vkCreateMetalSurfaceEXT(
    instance: VkInstance,
    pCreateInfos: *const VkMetalSurfaceCreateInfoEXT,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    gfxCreateMetalSurfaceEXT(instance, pCreateInfos, pAllocator, pSurface)
}

#[no_mangle]
pub unsafe extern "C" fn vkCreateXcbSurfaceKHR(
    instance: VkInstance,
    pCreateInfos: *const VkXcbSurfaceCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    gfxCreateXcbSurfaceKHR(instance, pCreateInfos, pAllocator, pSurface)
}

#[no_mangle]
pub unsafe extern "C" fn vkMapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    flags: VkMemoryMapFlags,
    ppData: *mut *mut ::std::os::raw::c_void,
) -> VkResult {
    gfxMapMemory(device, memory, offset, size, flags, ppData)
}

#[no_mangle]
pub unsafe extern "C" fn vkUnmapMemory(device: VkDevice, memory: VkDeviceMemory) {
    gfxUnmapMemory(device, memory)
}

#[no_mangle]
pub unsafe extern "C" fn vkDestroyImage(
    device: VkDevice,
    image: VkImage,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyImage(device, image, pAllocator)
}

#[no_mangle]
pub unsafe extern "C" fn vkCreateBuffer(
    device: VkDevice,
    pCreateInfo: *const VkBufferCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pBuffer: *mut VkBuffer,
) -> VkResult {
    gfxCreateBuffer(device, pCreateInfo, pAllocator, pBuffer)
}

#[no_mangle]
pub unsafe extern "C" fn vkDestroyBuffer(
    device: VkDevice,
    buffer: VkBuffer,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyBuffer(device, buffer, pAllocator)
}

#[no_mangle]
pub unsafe extern "C" fn vkGetBufferMemoryRequirements(
    device: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    gfxGetBufferMemoryRequirements(device, buffer, pMemoryRequirements)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetInstanceProcAddr(
    instance: VkInstance,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    gfxGetInstanceProcAddr(instance, pName)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetDeviceProcAddr(
    device: VkDevice,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    gfxGetDeviceProcAddr(device, pName)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceFeatures(
    physicalDevice: VkPhysicalDevice,
    pFeatures: *mut VkPhysicalDeviceFeatures,
) {
    gfxGetPhysicalDeviceFeatures(physicalDevice, pFeatures)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceFeatures2KHR(
    physicalDevice: VkPhysicalDevice,
    pFeatures: *mut VkPhysicalDeviceFeatures2KHR,
) {
    gfxGetPhysicalDeviceFeatures2KHR(physicalDevice, pFeatures)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceImageFormatProperties(
    physicalDevice: VkPhysicalDevice,
    format: VkFormat,
    type_: VkImageType,
    tiling: VkImageTiling,
    usage: VkImageUsageFlags,
    flags: VkImageCreateFlags,
    pImageFormatProperties: *mut VkImageFormatProperties,
) -> VkResult {
    gfxGetPhysicalDeviceImageFormatProperties(
        physicalDevice,
        format,
        type_,
        tiling,
        usage,
        flags,
        pImageFormatProperties,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceImageFormatProperties2KHR(
    physicalDevice: VkPhysicalDevice,
    pImageFormatInfo: *const VkPhysicalDeviceImageFormatInfo2KHR,
    pImageFormatProperties: *mut VkImageFormatProperties2KHR,
) -> VkResult {
    gfxGetPhysicalDeviceImageFormatProperties2KHR(
        physicalDevice,
        pImageFormatInfo,
        pImageFormatProperties,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceProperties(
    physicalDevice: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
) {
    gfxGetPhysicalDeviceProperties(physicalDevice, pProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceProperties2KHR(
    physicalDevice: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties2KHR,
) {
    gfxGetPhysicalDeviceProperties2KHR(physicalDevice, pProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkEnumerateDeviceExtensionProperties(
    physicalDevice: VkPhysicalDevice,
    pLayerName: *const ::std::os::raw::c_char,
    pPropertyCount: *mut u32,
    pProperties: *mut VkExtensionProperties,
) -> VkResult {
    gfxEnumerateDeviceExtensionProperties(physicalDevice, pLayerName, pPropertyCount, pProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkEnumerateInstanceLayerProperties(
    pPropertyCount: *mut u32,
    pProperties: *mut VkLayerProperties,
) -> VkResult {
    gfxEnumerateInstanceLayerProperties(pPropertyCount, pProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkEnumerateDeviceLayerProperties(
    physicalDevice: VkPhysicalDevice,
    pPropertyCount: *mut u32,
    pProperties: *mut VkLayerProperties,
) -> VkResult {
    gfxEnumerateDeviceLayerProperties(physicalDevice, pPropertyCount, pProperties)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetDeviceQueue(
    device: VkDevice,
    queueFamilyIndex: u32,
    queueIndex: u32,
    pQueue: *mut VkQueue,
) {
    gfxGetDeviceQueue(device, queueFamilyIndex, queueIndex, pQueue)
}
#[no_mangle]
pub unsafe extern "C" fn vkQueueSubmit(
    queue: VkQueue,
    submitCount: u32,
    pSubmits: *const VkSubmitInfo,
    fence: VkFence,
) -> VkResult {
    gfxQueueSubmit(queue, submitCount, pSubmits, fence)
}
#[no_mangle]
pub unsafe extern "C" fn vkQueueWaitIdle(queue: VkQueue) -> VkResult {
    gfxQueueWaitIdle(queue)
}
#[no_mangle]
pub unsafe extern "C" fn vkDeviceWaitIdle(device: VkDevice) -> VkResult {
    gfxDeviceWaitIdle(device)
}
#[no_mangle]
pub unsafe extern "C" fn vkFlushMappedMemoryRanges(
    device: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    gfxFlushMappedMemoryRanges(device, memoryRangeCount, pMemoryRanges)
}
#[no_mangle]
pub unsafe extern "C" fn vkInvalidateMappedMemoryRanges(
    device: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    gfxInvalidateMappedMemoryRanges(device, memoryRangeCount, pMemoryRanges)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetDeviceMemoryCommitment(
    device: VkDevice,
    memory: VkDeviceMemory,
    pCommittedMemoryInBytes: *mut VkDeviceSize,
) {
    gfxGetDeviceMemoryCommitment(device, memory, pCommittedMemoryInBytes)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetImageSparseMemoryRequirements(
    device: VkDevice,
    image: VkImage,
    pSparseMemoryRequirementCount: *mut u32,
    pSparseMemoryRequirements: *mut VkSparseImageMemoryRequirements,
) {
    gfxGetImageSparseMemoryRequirements(
        device,
        image,
        pSparseMemoryRequirementCount,
        pSparseMemoryRequirements,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceSparseImageFormatProperties(
    physicalDevice: VkPhysicalDevice,
    format: VkFormat,
    type_: VkImageType,
    samples: VkSampleCountFlagBits,
    usage: VkImageUsageFlags,
    tiling: VkImageTiling,
    pPropertyCount: *mut u32,
    pProperties: *mut VkSparseImageFormatProperties,
) {
    gfxGetPhysicalDeviceSparseImageFormatProperties(
        physicalDevice,
        format,
        type_,
        samples,
        usage,
        tiling,
        pPropertyCount,
        pProperties,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceSparseImageFormatProperties2KHR(
    physicalDevice: VkPhysicalDevice,
    pFormatInfo: *const VkPhysicalDeviceSparseImageFormatInfo2KHR,
    pPropertyCount: *mut u32,
    pProperties: *mut VkSparseImageFormatProperties2KHR,
) {
    gfxGetPhysicalDeviceSparseImageFormatProperties2KHR(
        physicalDevice,
        pFormatInfo,
        pPropertyCount,
        pProperties,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkQueueBindSparse(
    queue: VkQueue,
    bindInfoCount: u32,
    pBindInfo: *const VkBindSparseInfo,
    fence: VkFence,
) -> VkResult {
    gfxQueueBindSparse(queue, bindInfoCount, pBindInfo, fence)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateFence(
    device: VkDevice,
    pCreateInfo: *const VkFenceCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pFence: *mut VkFence,
) -> VkResult {
    gfxCreateFence(device, pCreateInfo, pAllocator, pFence)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyFence(
    device: VkDevice,
    fence: VkFence,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyFence(device, fence, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkResetFences(
    device: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
) -> VkResult {
    gfxResetFences(device, fenceCount, pFences)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetFenceStatus(device: VkDevice, fence: VkFence) -> VkResult {
    gfxGetFenceStatus(device, fence)
}
#[no_mangle]
pub unsafe extern "C" fn vkWaitForFences(
    device: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
    waitAll: VkBool32,
    timeout: u64,
) -> VkResult {
    gfxWaitForFences(device, fenceCount, pFences, waitAll, timeout)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateSemaphore(
    device: VkDevice,
    pCreateInfo: *const VkSemaphoreCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSemaphore: *mut VkSemaphore,
) -> VkResult {
    gfxCreateSemaphore(device, pCreateInfo, pAllocator, pSemaphore)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroySemaphore(
    device: VkDevice,
    semaphore: VkSemaphore,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroySemaphore(device, semaphore, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateEvent(
    device: VkDevice,
    pCreateInfo: *const VkEventCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pEvent: *mut VkEvent,
) -> VkResult {
    gfxCreateEvent(device, pCreateInfo, pAllocator, pEvent)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyEvent(
    device: VkDevice,
    event: VkEvent,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyEvent(device, event, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetEventStatus(device: VkDevice, event: VkEvent) -> VkResult {
    gfxGetEventStatus(device, event)
}
#[no_mangle]
pub unsafe extern "C" fn vkSetEvent(device: VkDevice, event: VkEvent) -> VkResult {
    gfxSetEvent(device, event)
}
#[no_mangle]
pub unsafe extern "C" fn vkResetEvent(device: VkDevice, event: VkEvent) -> VkResult {
    gfxResetEvent(device, event)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateQueryPool(
    device: VkDevice,
    pCreateInfo: *const VkQueryPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pQueryPool: *mut VkQueryPool,
) -> VkResult {
    gfxCreateQueryPool(device, pCreateInfo, pAllocator, pQueryPool)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyQueryPool(
    device: VkDevice,
    queryPool: VkQueryPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyQueryPool(device, queryPool, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetQueryPoolResults(
    device: VkDevice,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
    dataSize: usize,
    pData: *mut ::std::os::raw::c_void,
    stride: VkDeviceSize,
    flags: VkQueryResultFlags,
) -> VkResult {
    gfxGetQueryPoolResults(
        device, queryPool, firstQuery, queryCount, dataSize, pData, stride, flags,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateBufferView(
    device: VkDevice,
    pCreateInfo: *const VkBufferViewCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pView: *mut VkBufferView,
) -> VkResult {
    gfxCreateBufferView(device, pCreateInfo, pAllocator, pView)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyBufferView(
    device: VkDevice,
    bufferView: VkBufferView,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyBufferView(device, bufferView, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetImageSubresourceLayout(
    device: VkDevice,
    image: VkImage,
    pSubresource: *const VkImageSubresource,
    pLayout: *mut VkSubresourceLayout,
) {
    gfxGetImageSubresourceLayout(device, image, pSubresource, pLayout)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateShaderModule(
    device: VkDevice,
    pCreateInfo: *const VkShaderModuleCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pShaderModule: *mut VkShaderModule,
) -> VkResult {
    gfxCreateShaderModule(device, pCreateInfo, pAllocator, pShaderModule)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyShaderModule(
    device: VkDevice,
    shaderModule: VkShaderModule,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyShaderModule(device, shaderModule, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreatePipelineCache(
    device: VkDevice,
    pCreateInfo: *const VkPipelineCacheCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelineCache: *mut VkPipelineCache,
) -> VkResult {
    gfxCreatePipelineCache(device, pCreateInfo, pAllocator, pPipelineCache)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyPipelineCache(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyPipelineCache(device, pipelineCache, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetPipelineCacheData(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    pDataSize: *mut usize,
    pData: *mut ::std::os::raw::c_void,
) -> VkResult {
    gfxGetPipelineCacheData(device, pipelineCache, pDataSize, pData)
}
#[no_mangle]
pub unsafe extern "C" fn vkMergePipelineCaches(
    device: VkDevice,
    dstCache: VkPipelineCache,
    srcCacheCount: u32,
    pSrcCaches: *const VkPipelineCache,
) -> VkResult {
    gfxMergePipelineCaches(device, dstCache, srcCacheCount, pSrcCaches)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateGraphicsPipelines(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkGraphicsPipelineCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    gfxCreateGraphicsPipelines(
        device,
        pipelineCache,
        createInfoCount,
        pCreateInfos,
        pAllocator,
        pPipelines,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateComputePipelines(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkComputePipelineCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    gfxCreateComputePipelines(
        device,
        pipelineCache,
        createInfoCount,
        pCreateInfos,
        pAllocator,
        pPipelines,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyPipeline(
    device: VkDevice,
    pipeline: VkPipeline,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyPipeline(device, pipeline, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreatePipelineLayout(
    device: VkDevice,
    pCreateInfo: *const VkPipelineLayoutCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelineLayout: *mut VkPipelineLayout,
) -> VkResult {
    gfxCreatePipelineLayout(device, pCreateInfo, pAllocator, pPipelineLayout)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyPipelineLayout(
    device: VkDevice,
    pipelineLayout: VkPipelineLayout,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyPipelineLayout(device, pipelineLayout, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateSampler(
    device: VkDevice,
    pCreateInfo: *const VkSamplerCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSampler: *mut VkSampler,
) -> VkResult {
    gfxCreateSampler(device, pCreateInfo, pAllocator, pSampler)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroySampler(
    device: VkDevice,
    sampler: VkSampler,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroySampler(device, sampler, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateDescriptorSetLayout(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorSetLayoutCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSetLayout: *mut VkDescriptorSetLayout,
) -> VkResult {
    gfxCreateDescriptorSetLayout(device, pCreateInfo, pAllocator, pSetLayout)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorSetLayout(
    device: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyDescriptorSetLayout(device, descriptorSetLayout, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateDescriptorPool(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDescriptorPool: *mut VkDescriptorPool,
) -> VkResult {
    gfxCreateDescriptorPool(device, pCreateInfo, pAllocator, pDescriptorPool)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyDescriptorPool(device, descriptorPool, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkResetDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    flags: VkDescriptorPoolResetFlags,
) -> VkResult {
    gfxResetDescriptorPool(device, descriptorPool, flags)
}
#[no_mangle]
pub unsafe extern "C" fn vkAllocateDescriptorSets(
    device: VkDevice,
    pAllocateInfo: *const VkDescriptorSetAllocateInfo,
    pDescriptorSets: *mut VkDescriptorSet,
) -> VkResult {
    gfxAllocateDescriptorSets(device, pAllocateInfo, pDescriptorSets)
}
#[no_mangle]
pub unsafe extern "C" fn vkFreeDescriptorSets(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
) -> VkResult {
    gfxFreeDescriptorSets(device, descriptorPool, descriptorSetCount, pDescriptorSets)
}
#[no_mangle]
pub unsafe extern "C" fn vkUpdateDescriptorSets(
    device: VkDevice,
    descriptorWriteCount: u32,
    pDescriptorWrites: *const VkWriteDescriptorSet,
    descriptorCopyCount: u32,
    pDescriptorCopies: *const VkCopyDescriptorSet,
) {
    gfxUpdateDescriptorSets(
        device,
        descriptorWriteCount,
        pDescriptorWrites,
        descriptorCopyCount,
        pDescriptorCopies,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateFramebuffer(
    device: VkDevice,
    pCreateInfo: *const VkFramebufferCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pFramebuffer: *mut VkFramebuffer,
) -> VkResult {
    gfxCreateFramebuffer(device, pCreateInfo, pAllocator, pFramebuffer)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyFramebuffer(
    device: VkDevice,
    framebuffer: VkFramebuffer,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyFramebuffer(device, framebuffer, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkCreateRenderPass(
    device: VkDevice,
    pCreateInfo: *const VkRenderPassCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pRenderPass: *mut VkRenderPass,
) -> VkResult {
    gfxCreateRenderPass(device, pCreateInfo, pAllocator, pRenderPass)
}
#[no_mangle]
pub unsafe extern "C" fn vkDestroyRenderPass(
    device: VkDevice,
    renderPass: VkRenderPass,
    pAllocator: *const VkAllocationCallbacks,
) {
    gfxDestroyRenderPass(device, renderPass, pAllocator)
}
#[no_mangle]
pub unsafe extern "C" fn vkGetRenderAreaGranularity(
    device: VkDevice,
    renderPass: VkRenderPass,
    pGranularity: *mut VkExtent2D,
) {
    gfxGetRenderAreaGranularity(device, renderPass, pGranularity)
}

#[no_mangle]
pub unsafe extern "C" fn vkBeginCommandBuffer(
    commandBuffer: VkCommandBuffer,
    pBeginInfo: *const VkCommandBufferBeginInfo,
) -> VkResult {
    gfxBeginCommandBuffer(commandBuffer, pBeginInfo)
}
#[no_mangle]
pub unsafe extern "C" fn vkEndCommandBuffer(commandBuffer: VkCommandBuffer) -> VkResult {
    gfxEndCommandBuffer(commandBuffer)
}
#[no_mangle]
pub unsafe extern "C" fn vkResetCommandBuffer(
    commandBuffer: VkCommandBuffer,
    flags: VkCommandBufferResetFlags,
) -> VkResult {
    gfxResetCommandBuffer(commandBuffer, flags)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindPipeline(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    pipeline: VkPipeline,
) {
    gfxCmdBindPipeline(commandBuffer, pipelineBindPoint, pipeline)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetViewport(
    commandBuffer: VkCommandBuffer,
    firstViewport: u32,
    viewportCount: u32,
    pViewports: *const VkViewport,
) {
    gfxCmdSetViewport(commandBuffer, firstViewport, viewportCount, pViewports)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetScissor(
    commandBuffer: VkCommandBuffer,
    firstScissor: u32,
    scissorCount: u32,
    pScissors: *const VkRect2D,
) {
    gfxCmdSetScissor(commandBuffer, firstScissor, scissorCount, pScissors)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetLineWidth(commandBuffer: VkCommandBuffer, lineWidth: f32) {
    gfxCmdSetLineWidth(commandBuffer, lineWidth)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetDepthBias(
    commandBuffer: VkCommandBuffer,
    depthBiasConstantFactor: f32,
    depthBiasClamp: f32,
    depthBiasSlopeFactor: f32,
) {
    gfxCmdSetDepthBias(
        commandBuffer,
        depthBiasConstantFactor,
        depthBiasClamp,
        depthBiasSlopeFactor,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetBlendConstants(
    commandBuffer: VkCommandBuffer,
    blendConstants: *const f32,
) {
    gfxCmdSetBlendConstants(commandBuffer, blendConstants)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetDepthBounds(
    commandBuffer: VkCommandBuffer,
    minDepthBounds: f32,
    maxDepthBounds: f32,
) {
    gfxCmdSetDepthBounds(commandBuffer, minDepthBounds, maxDepthBounds)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetStencilCompareMask(
    commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    compareMask: u32,
) {
    gfxCmdSetStencilCompareMask(commandBuffer, faceMask, compareMask)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetStencilWriteMask(
    commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    writeMask: u32,
) {
    gfxCmdSetStencilWriteMask(commandBuffer, faceMask, writeMask)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetStencilReference(
    commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    reference: u32,
) {
    gfxCmdSetStencilReference(commandBuffer, faceMask, reference)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindDescriptorSets(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    layout: VkPipelineLayout,
    firstSet: u32,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
    dynamicOffsetCount: u32,
    pDynamicOffsets: *const u32,
) {
    gfxCmdBindDescriptorSets(
        commandBuffer,
        pipelineBindPoint,
        layout,
        firstSet,
        descriptorSetCount,
        pDescriptorSets,
        dynamicOffsetCount,
        pDynamicOffsets,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindIndexBuffer(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    indexType: VkIndexType,
) {
    gfxCmdBindIndexBuffer(commandBuffer, buffer, offset, indexType)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdBindVertexBuffers(
    commandBuffer: VkCommandBuffer,
    firstBinding: u32,
    bindingCount: u32,
    pBuffers: *const VkBuffer,
    pOffsets: *const VkDeviceSize,
) {
    gfxCmdBindVertexBuffers(
        commandBuffer,
        firstBinding,
        bindingCount,
        pBuffers,
        pOffsets,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdDraw(
    commandBuffer: VkCommandBuffer,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    gfxCmdDraw(
        commandBuffer,
        vertexCount,
        instanceCount,
        firstVertex,
        firstInstance,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdDrawIndexed(
    commandBuffer: VkCommandBuffer,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    vertexOffset: i32,
    firstInstance: u32,
) {
    gfxCmdDrawIndexed(
        commandBuffer,
        indexCount,
        instanceCount,
        firstIndex,
        vertexOffset,
        firstInstance,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdDrawIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    gfxCmdDrawIndirect(commandBuffer, buffer, offset, drawCount, stride)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdDrawIndexedIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    gfxCmdDrawIndexedIndirect(commandBuffer, buffer, offset, drawCount, stride)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdDispatch(
    commandBuffer: VkCommandBuffer,
    groupCountX: u32,
    groupCountY: u32,
    groupCountZ: u32,
) {
    gfxCmdDispatch(commandBuffer, groupCountX, groupCountY, groupCountZ)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdDispatchIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
) {
    gfxCmdDispatchIndirect(commandBuffer, buffer, offset)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdCopyBuffer(
    commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferCopy,
) {
    gfxCmdCopyBuffer(commandBuffer, srcBuffer, dstBuffer, regionCount, pRegions)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdCopyImage(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageCopy,
) {
    gfxCmdCopyImage(
        commandBuffer,
        srcImage,
        srcImageLayout,
        dstImage,
        dstImageLayout,
        regionCount,
        pRegions,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdBlitImage(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageBlit,
    filter: VkFilter,
) {
    gfxCmdBlitImage(
        commandBuffer,
        srcImage,
        srcImageLayout,
        dstImage,
        dstImageLayout,
        regionCount,
        pRegions,
        filter,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdCopyBufferToImage(
    commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkBufferImageCopy,
) {
    gfxCmdCopyBufferToImage(
        commandBuffer,
        srcBuffer,
        dstImage,
        dstImageLayout,
        regionCount,
        pRegions,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdCopyImageToBuffer(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferImageCopy,
) {
    gfxCmdCopyImageToBuffer(
        commandBuffer,
        srcImage,
        srcImageLayout,
        dstBuffer,
        regionCount,
        pRegions,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdUpdateBuffer(
    commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    dataSize: VkDeviceSize,
    pData: *const ::std::os::raw::c_void,
) {
    gfxCmdUpdateBuffer(commandBuffer, dstBuffer, dstOffset, dataSize, pData)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdFillBuffer(
    commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    size: VkDeviceSize,
    data: u32,
) {
    gfxCmdFillBuffer(commandBuffer, dstBuffer, dstOffset, size, data)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdClearColorImage(
    commandBuffer: VkCommandBuffer,
    image: VkImage,
    imageLayout: VkImageLayout,
    pColor: *const VkClearColorValue,
    rangeCount: u32,
    pRanges: *const VkImageSubresourceRange,
) {
    gfxCmdClearColorImage(
        commandBuffer,
        image,
        imageLayout,
        pColor,
        rangeCount,
        pRanges,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdClearDepthStencilImage(
    commandBuffer: VkCommandBuffer,
    image: VkImage,
    imageLayout: VkImageLayout,
    pDepthStencil: *const VkClearDepthStencilValue,
    rangeCount: u32,
    pRanges: *const VkImageSubresourceRange,
) {
    gfxCmdClearDepthStencilImage(
        commandBuffer,
        image,
        imageLayout,
        pDepthStencil,
        rangeCount,
        pRanges,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdClearAttachments(
    commandBuffer: VkCommandBuffer,
    attachmentCount: u32,
    pAttachments: *const VkClearAttachment,
    rectCount: u32,
    pRects: *const VkClearRect,
) {
    gfxCmdClearAttachments(
        commandBuffer,
        attachmentCount,
        pAttachments,
        rectCount,
        pRects,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdResolveImage(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageResolve,
) {
    gfxCmdResolveImage(
        commandBuffer,
        srcImage,
        srcImageLayout,
        dstImage,
        dstImageLayout,
        regionCount,
        pRegions,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdSetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    gfxCmdSetEvent(commandBuffer, event, stageMask)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdResetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    gfxCmdResetEvent(commandBuffer, event, stageMask)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdWaitEvents(
    commandBuffer: VkCommandBuffer,
    eventCount: u32,
    pEvents: *const VkEvent,
    srcStageMask: VkPipelineStageFlags,
    dstStageMask: VkPipelineStageFlags,
    memoryBarrierCount: u32,
    pMemoryBarriers: *const VkMemoryBarrier,
    bufferMemoryBarrierCount: u32,
    pBufferMemoryBarriers: *const VkBufferMemoryBarrier,
    imageMemoryBarrierCount: u32,
    pImageMemoryBarriers: *const VkImageMemoryBarrier,
) {
    gfxCmdWaitEvents(
        commandBuffer,
        eventCount,
        pEvents,
        srcStageMask,
        dstStageMask,
        memoryBarrierCount,
        pMemoryBarriers,
        bufferMemoryBarrierCount,
        pBufferMemoryBarriers,
        imageMemoryBarrierCount,
        pImageMemoryBarriers,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdPipelineBarrier(
    commandBuffer: VkCommandBuffer,
    srcStageMask: VkPipelineStageFlags,
    dstStageMask: VkPipelineStageFlags,
    dependencyFlags: VkDependencyFlags,
    memoryBarrierCount: u32,
    pMemoryBarriers: *const VkMemoryBarrier,
    bufferMemoryBarrierCount: u32,
    pBufferMemoryBarriers: *const VkBufferMemoryBarrier,
    imageMemoryBarrierCount: u32,
    pImageMemoryBarriers: *const VkImageMemoryBarrier,
) {
    gfxCmdPipelineBarrier(
        commandBuffer,
        srcStageMask,
        dstStageMask,
        dependencyFlags,
        memoryBarrierCount,
        pMemoryBarriers,
        bufferMemoryBarrierCount,
        pBufferMemoryBarriers,
        imageMemoryBarrierCount,
        pImageMemoryBarriers,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdBeginQuery(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    query: u32,
    flags: VkQueryControlFlags,
) {
    gfxCmdBeginQuery(commandBuffer, queryPool, query, flags)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdEndQuery(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    query: u32,
) {
    gfxCmdEndQuery(commandBuffer, queryPool, query)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdResetQueryPool(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
) {
    gfxCmdResetQueryPool(commandBuffer, queryPool, firstQuery, queryCount)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdWriteTimestamp(
    commandBuffer: VkCommandBuffer,
    pipelineStage: VkPipelineStageFlagBits,
    queryPool: VkQueryPool,
    query: u32,
) {
    gfxCmdWriteTimestamp(commandBuffer, pipelineStage, queryPool, query)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdCopyQueryPoolResults(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    stride: VkDeviceSize,
    flags: VkQueryResultFlags,
) {
    gfxCmdCopyQueryPoolResults(
        commandBuffer,
        queryPool,
        firstQuery,
        queryCount,
        dstBuffer,
        dstOffset,
        stride,
        flags,
    )
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdPushConstants(
    commandBuffer: VkCommandBuffer,
    layout: VkPipelineLayout,
    stageFlags: VkShaderStageFlags,
    offset: u32,
    size: u32,
    pValues: *const ::std::os::raw::c_void,
) {
    gfxCmdPushConstants(commandBuffer, layout, stageFlags, offset, size, pValues)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdBeginRenderPass(
    commandBuffer: VkCommandBuffer,
    pRenderPassBegin: *const VkRenderPassBeginInfo,
    contents: VkSubpassContents,
) {
    gfxCmdBeginRenderPass(commandBuffer, pRenderPassBegin, contents)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdNextSubpass(
    commandBuffer: VkCommandBuffer,
    contents: VkSubpassContents,
) {
    gfxCmdNextSubpass(commandBuffer, contents)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdEndRenderPass(commandBuffer: VkCommandBuffer) {
    gfxCmdEndRenderPass(commandBuffer)
}
#[no_mangle]
pub unsafe extern "C" fn vkCmdExecuteCommands(
    commandBuffer: VkCommandBuffer,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    gfxCmdExecuteCommands(commandBuffer, commandBufferCount, pCommandBuffers)
}
#[no_mangle]
pub unsafe extern "C" fn vkAcquireNextImageKHR(
    device: VkDevice,
    swapchain: VkSwapchainKHR,
    timeout: u64,
    semaphore: VkSemaphore,
    fence: VkFence,
    pImageIndex: *mut u32,
) -> VkResult {
    gfxAcquireNextImageKHR(device, swapchain, timeout, semaphore, fence, pImageIndex)
}
#[no_mangle]
pub unsafe extern "C" fn vkQueuePresentKHR(
    queue: VkQueue,
    pPresentInfo: *const VkPresentInfoKHR,
) -> VkResult {
    gfxQueuePresentKHR(queue, pPresentInfo)
}
#[no_mangle]
pub unsafe extern "C" fn vkEnumerateInstanceExtensionProperties(
    pLayerName: *const ::std::os::raw::c_char,
    pPropertyCount: *mut u32,
    pProperties: *mut VkExtensionProperties,
) -> VkResult {
    gfxEnumerateInstanceExtensionProperties(pLayerName, pPropertyCount, pProperties)
}

//TODO: remove this once Dota2 stops asking for it
#[no_mangle]
pub unsafe extern "C" fn vkGetPhysicalDeviceMetalFeaturesMVK(
    _adapter: VkPhysicalDevice,
    _metal_features: *mut ::std::os::raw::c_void,
) {
}
