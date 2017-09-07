#include "vulkan/vulkan.h"
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

    uint32_t gpu_count = 1;
    VkPhysicalDevice physical_devices[1] = {};
    res = vkEnumeratePhysicalDevices(instance, &gpu_count, physical_devices);
    printf("\tvkEnumeratePhysicalDevices: res=%d count=%d\n", res, gpu_count);
    assert(!res && gpu_count);

    vkDestroyInstance(instance, NULL);

    printf("done.\n");
    return 0;
}
