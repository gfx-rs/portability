/// Sample code adopted from https://github.com/LunarG/VulkanSamples

/*
 * Vulkan Samples
 *
 * Copyright (C) 2015-2016 Valve Corporation
 * Copyright (C) 2015-2016 LunarG, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


#if defined(_WIN32)
#define VK_USE_PLATFORM_WIN32_KHR
#endif

#include <vulkan/vulkan.h>
#include <assert.h>
#include <stdio.h>
#include <vector>
#include "window.hpp"

int main() {
    printf("starting the portability test\n");

    VkInstance instance;
    VkResult res = (VkResult)0;
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

    const uint32_t width = 800;
    const uint32_t height = 600;

    // Window initialization
    Config config = { 10, 10, width, height };
    Window window = new_window(config);

    VkSurfaceKHR surface;

#if defined(_WIN32)
    VkWin32SurfaceCreateInfoKHR surface_info = {};
    surface_info.sType = VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR;
    surface_info.hinstance = window.instance;
    surface_info.hwnd = window.window;
    vkCreateWin32SurfaceKHR(instance, &surface_info, NULL, &surface);
#endif
    printf("\tvkCreateSurfaceKHR\n");

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

    VkSwapchainCreateInfoKHR swapchain_ci = {};
    swapchain_ci.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
    swapchain_ci.surface = surface;
    swapchain_ci.minImageCount = desiredNumberOfSwapChainImages;
    swapchain_ci.imageFormat = surfFormats[0].format;
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
    swapchain_ci.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE;

    VkSwapchainKHR swapchain = 0;
    res = vkCreateSwapchainKHR(device, &swapchain_ci, NULL, &swapchain);
    printf("\tvkCreateSwapchainKHR: res=%d\n", res);


    uint32_t image_count = 0;
    res = vkGetSwapchainImagesKHR(device, swapchain, &image_count, NULL);
    printf("\tvkCreateSwapchainKHR (query): res=%d image_count=%d\n", res, image_count);
    assert(!res);

    std::vector<VkImage> swapchain_images(image_count);
    res = vkGetSwapchainImagesKHR(device, swapchain, &image_count, &swapchain_images[0]);
    printf("\tvkCreateSwapchainKHR: res=%d\n", res);
    assert(!res);

    std::vector<VkImageView> swapchain_views(image_count);
    for(auto i = 0; i < image_count; i++) {
        VkImageViewCreateInfo color_image_view = {};
        color_image_view.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
        color_image_view.pNext = NULL;
        color_image_view.flags = 0;
        color_image_view.image = swapchain_images[i];
        color_image_view.viewType = VK_IMAGE_VIEW_TYPE_2D;
        color_image_view.format = swapchain_ci.imageFormat;
        color_image_view.components.r = VK_COMPONENT_SWIZZLE_R;
        color_image_view.components.g = VK_COMPONENT_SWIZZLE_G;
        color_image_view.components.b = VK_COMPONENT_SWIZZLE_B;
        color_image_view.components.a = VK_COMPONENT_SWIZZLE_A;
        color_image_view.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        color_image_view.subresourceRange.baseMipLevel = 0;
        color_image_view.subresourceRange.levelCount = 1;
        color_image_view.subresourceRange.baseArrayLayer = 0;
        color_image_view.subresourceRange.layerCount = 1;

        res = vkCreateImageView(device, &color_image_view, NULL, &swapchain_views[i]);
        printf("\tvkCreateImageView: res=%d\n", res);
        assert(!res);
    }

    VkImageCreateInfo image_info = {};
    const VkFormat depth_format = VK_FORMAT_D16_UNORM;
    VkFormatProperties props;
    vkGetPhysicalDeviceFormatProperties(physical_devices[0], depth_format, &props);
    printf("\tvkGetPhysicalDeviceFormatProperties\n");
    printf(
        "\t\tlinear_tiling_features: %x\n"
        "\t\toptimal_tiling_features: %x\n"
        "\t\tbuffer_features: %x\n",
            props.linearTilingFeatures,
            props.optimalTilingFeatures,
            props.bufferFeatures);

    if (props.linearTilingFeatures & VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT) {
        image_info.tiling = VK_IMAGE_TILING_LINEAR;
    } else if (props.optimalTilingFeatures & VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT) {
        image_info.tiling = VK_IMAGE_TILING_OPTIMAL;
    } else {
        printf("VK_FORMAT_D16_UNORM unsupported.\n");
        return -1;
    }

    image_info.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    image_info.pNext = NULL;
    image_info.imageType = VK_IMAGE_TYPE_2D;
    image_info.format = depth_format;
    image_info.extent.width = width;
    image_info.extent.height = height;
    image_info.extent.depth = 1;
    image_info.mipLevels = 1;
    image_info.arrayLayers = 1;
    image_info.samples = VK_SAMPLE_COUNT_1_BIT;
    image_info.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    image_info.usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
    image_info.queueFamilyIndexCount = 0;
    image_info.pQueueFamilyIndices = NULL;
    image_info.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    image_info.flags = 0;

    VkMemoryAllocateInfo mem_alloc = {};
    mem_alloc.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    mem_alloc.pNext = NULL;
    mem_alloc.allocationSize = 0;
    mem_alloc.memoryTypeIndex = 0;

    VkImageViewCreateInfo view_info = {};
    view_info.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    view_info.pNext = NULL;
    view_info.image = VK_NULL_HANDLE;
    view_info.format = depth_format;
    view_info.components.r = VK_COMPONENT_SWIZZLE_R;
    view_info.components.g = VK_COMPONENT_SWIZZLE_G;
    view_info.components.b = VK_COMPONENT_SWIZZLE_B;
    view_info.components.a = VK_COMPONENT_SWIZZLE_A;
    view_info.subresourceRange.aspectMask = VK_IMAGE_ASPECT_DEPTH_BIT;
    view_info.subresourceRange.baseMipLevel = 0;
    view_info.subresourceRange.levelCount = 1;
    view_info.subresourceRange.baseArrayLayer = 0;
    view_info.subresourceRange.layerCount = 1;
    view_info.viewType = VK_IMAGE_VIEW_TYPE_2D;
    view_info.flags = 0;

    VkMemoryRequirements mem_reqs;

    /* Create image */
    VkImage depth_image = 0;
    res = vkCreateImage(device, &image_info, NULL, &depth_image);
    printf("\tvkCreateImage: res=%d\n", res);
    assert(!res);

    vkGetImageMemoryRequirements(device, depth_image, &mem_reqs);
    printf("\tvkGetImageMemoryRequirements\n");
    printf(
        "\t\tsize: %llx\n"
        "\t\talignment: %llx\n"
        "\t\tmemoryTypeBits: %x\n",
            mem_reqs.size,
            mem_reqs.alignment,
            mem_reqs.memoryTypeBits);

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
    while(poll_events()) {

    }

    for(auto view : swapchain_views) {
        vkDestroyImageView(device, view, NULL);
        printf("\tvkDestroyImageView\n");
    }
    vkDestroySwapchainKHR(device, swapchain, NULL);
    printf("\tvkDestroySwapchainKHR\n");
    vkFreeCommandBuffers(device, cmd_pool, 1, &cmd_buffer);
    printf("\tvkFreeCommandBuffers\n");
    vkDestroyCommandPool(device, cmd_pool, NULL);
    printf("\tvkDestroyCommandPool\n");
    vkDestroySurfaceKHR(instance, surface, NULL);
    printf("\tvkDestroySurfaceKHR\n");
    vkDestroyDevice(device, NULL);
    printf("\tvkDestroyDevice\n");
    vkDestroyInstance(instance, NULL);

    printf("done.\n");
    return 0;
}
