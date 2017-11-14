/// Sample code adopted from https://github.com/LunarG/VulkanSamples

#include <vulkan/vulkan.h>
#include <assert.h>
#include <stdio.h>

VkSurfaceKHR vkCreateSurfaceGFX(VkInstance);

int main() {
    printf("starting the portability test\n");

    VkInstance instance;
    VkResult res = 0;
    unsigned int i;

    VkInstanceCreateInfo inst_info = {};
    inst_info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    res = vkCreateInstance(&inst_info, NULL, &instance);
    if (res == VK_ERROR_INCOMPATIBLE_DRIVER) {
        printf("cannot find a compatible Vulkan ICD\n");
        return -1;
    } else if (res) {
        printf("unknown error\n");
        return -1;
    }

    VkSurfaceKHR surface = vkCreateSurfaceGFX(instance);
    printf("\tvkCreateSurfaceGFX\n");

    uint32_t adapter_count = 1;
    VkPhysicalDevice physical_devices[1] = {};
    res = vkEnumeratePhysicalDevices(instance, &adapter_count, physical_devices);
    printf("\tvkEnumeratePhysicalDevices: res=%d count=%d\n", res, adapter_count);
    assert(!res && adapter_count);

    VkQueueFamilyProperties queue_family_properties[5];
    uint32_t queue_family_count = sizeof(queue_family_properties) / sizeof(VkQueueFamilyProperties);

    vkGetPhysicalDeviceQueueFamilyProperties(physical_devices[0], &queue_family_count, queue_family_properties);
    printf("\tvkGetPhysicalDeviceQueueFamilyProperties: count=%d\n", queue_family_count);
    assert(queue_family_count);

    int queue_family_index = -1;
    for (i = 0; i < queue_family_count; i++) {
        VkBool32 supports_present = 0;
        vkGetPhysicalDeviceSurfaceSupportKHR(physical_devices[0], i, surface, &supports_present);
        if ((queue_family_properties[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) && supports_present) {
            queue_family_index = i;
            break;
        }
    }
    printf("\tusing queue family index %d\n", queue_family_index);
    assert(queue_family_index >= 0);

    VkSurfaceFormatKHR surfFormats[20];
    uint32_t formatCount = sizeof(surfFormats) / sizeof(surfFormats[0]);
    res = vkGetPhysicalDeviceSurfaceFormatsKHR(physical_devices[0], surface, &formatCount, surfFormats);
    printf("\tvkGetPhysicalDeviceSurfaceFormatsKHR: res=%d, count=%d\n", res, formatCount);
    assert(!res);

    VkSurfaceCapabilitiesKHR surfCapabilities;
    res = vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_devices[0], surface, &surfCapabilities);
    assert(!res);

    VkPresentModeKHR presentModes[10];
    uint32_t presentModeCount = sizeof(presentModes) / sizeof(presentModes[0]);
    res = vkGetPhysicalDeviceSurfacePresentModesKHR(physical_devices[0], surface, &presentModeCount, presentModes);
    printf("\tvkGetPhysicalDeviceSurfacePresentModesKHR: res=%d, count=%d\n", res, presentModeCount);
    assert(!res);

    VkExtent2D swapchainExtent = surfCapabilities.currentExtent;
    VkPresentModeKHR swapchainPresentMode = VK_PRESENT_MODE_FIFO_KHR;

    // Determine the number of VkImage's to use in the swap chain.
    // We need to acquire only 1 presentable image at at time.
    // Asking for minImageCount images ensures that we can acquire
    // 1 presentable image as long as we present it before attempting
    // to acquire another.
    uint32_t desiredNumberOfSwapChainImages = surfCapabilities.minImageCount;

    VkSurfaceTransformFlagBitsKHR preTransform;
    if (surfCapabilities.supportedTransforms & VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR) {
        preTransform = VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR;
    } else {
        preTransform = surfCapabilities.currentTransform;
    }

    VkCompositeAlphaFlagBitsKHR compositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;

    /*VkSwapchainCreateInfoKHR swapchain_ci = {0};
    swapchain_ci.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
    swapchain_ci.surface = info.surface;
    swapchain_ci.minImageCount = desiredNumberOfSwapChainImages;
    swapchain_ci.imageFormat = info.format;
    swapchain_ci.imageExtent.width = swapchainExtent.width;
    swapchain_ci.imageExtent.height = swapchainExtent.height;
    swapchain_ci.preTransform = preTransform;
    swapchain_ci.compositeAlpha = compositeAlpha;
    swapchain_ci.imageArrayLayers = 1;
    swapchain_ci.presentMode = swapchainPresentMode;
    swapchain_ci.oldSwapchain = VK_NULL_HANDLE;
    swapchain_ci.clipped = true;
    swapchain_ci.imageColorSpace = VK_COLORSPACE_SRGB_NONLINEAR_KHR;
    swapchain_ci.imageUsage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
    swapchain_ci.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE;*/

    VkDeviceQueueCreateInfo queue_info = {};
    float queue_priorities[1] = {0.0};
    queue_info.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    queue_info.queueCount = 1;
    queue_info.pQueuePriorities = queue_priorities;

    VkDeviceCreateInfo device_info = {};
    device_info.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
    device_info.queueCreateInfoCount = 1;
    device_info.pQueueCreateInfos = &queue_info;

    VkDevice device = 0;
    res = vkCreateDevice(physical_devices[0], &device_info, NULL, &device);
    printf("\tvkCreateDevice: res=%d\n", res);
    assert(!res);

    VkCommandPool cmd_pool = 0;
    VkCommandPoolCreateInfo cmd_pool_info = {};
    cmd_pool_info.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
    cmd_pool_info.pNext = NULL;
    cmd_pool_info.queueFamilyIndex = queue_family_index;
    cmd_pool_info.flags = 0;

    res = vkCreateCommandPool(device, &cmd_pool_info, NULL, &cmd_pool);
    printf("\tvkCreateCommandPool: res=%d\n", res);
    assert(!res);

    VkCommandBuffer cmd_buffer = 0;
    VkCommandBufferAllocateInfo cmd_alloc_info;
    cmd_alloc_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    cmd_alloc_info.pNext = NULL;
    cmd_alloc_info.commandPool = cmd_pool;
    cmd_alloc_info.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    cmd_alloc_info.commandBufferCount = 1;

    res = vkAllocateCommandBuffers(device, &cmd_alloc_info, &cmd_buffer);
    printf("\tvkAllocateCommandBuffers: res=%d\n", res);
    assert(!res);

    // Some work...

    vkFreeCommandBuffers(device, cmd_pool, 1, &cmd_buffer);
    vkDestroyCommandPool(device, cmd_pool, NULL);
    vkDestroySurfaceKHR(instance, surface, NULL);
    vkDestroyDevice(device, NULL);
    vkDestroyInstance(instance, NULL);

    printf("done.\n");
    return 0;
}
