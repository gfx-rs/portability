/// Sample code adopted from https://github.com/LunarG/VulkanSamples

#include <vulkan/vulkan.h>
#include <assert.h>
#include <stdio.h>

int main() {
    printf("starting the portability test\n");

    VkInstanceCreateInfo inst_info = {};
    inst_info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;

    VkInstance instance;
    VkResult res;

    res = vkCreateInstance(&inst_info, NULL, &instance);
    if (res == VK_ERROR_INCOMPATIBLE_DRIVER) {
        printf("cannot find a compatible Vulkan ICD\n");
        return -1;
    } else if (res) {
        printf("unknown error\n");
        return -1;
    }

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
    for (unsigned int i = 0; i < queue_family_count; i++) {
        if (queue_family_properties[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) {
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

    //TODO

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
    vkDestroyDevice(device, NULL);
    vkDestroyInstance(instance, NULL);

    printf("done.\n");
    return 0;
}
