
use super::*;
use std::mem;
use std::ops::Deref;

#[inline]
pub extern fn gfxCreateInstance(
    _pCreateInfo: *const VkInstanceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult {
    let instance = back::Instance::create("portability", 1);
    unsafe { *pInstance = Handle::new(instance) };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxDestroyInstance(
    instance: VkInstance,
    _pAllocator: *const VkAllocationCallbacks,
) {
    instance.unwrap();
    //let it drop
}

#[inline]
pub extern fn gfxEnumeratePhysicalDevices(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult {
    let adapters = instance.enumerate_adapters();
    let output = unsafe { slice::from_raw_parts_mut(pPhysicalDevices, *pPhysicalDeviceCount as _) };
    let count = cmp::min(adapters.len(), output.len());

    for (out, adapter) in output.iter_mut().zip(adapters.into_iter()) {
        *out = Handle::new(adapter);
    }

    unsafe { *pPhysicalDeviceCount = count as _ };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxGetPhysicalDeviceQueueFamilyProperties(
    adapter: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
) {
    let output = unsafe {
        slice::from_raw_parts_mut(pQueueFamilyProperties, *pQueueFamilyPropertyCount as _)
    };
    let families = &adapter.queue_families;
    if output.len() > families.len() {
        unsafe { *pQueueFamilyPropertyCount = families.len() as _ };
    }
    for (ref mut out, ref family) in output.iter_mut().zip(families.iter()) {
        **out = VkQueueFamilyProperties {
            queueFlags: match family.queue_type() {
                hal::QueueType::General => VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT as u32 | VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT as u32,
                hal::QueueType::Graphics => VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT as u32,
                hal::QueueType::Compute => VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT as u32,
                hal::QueueType::Transfer => VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT as u32,
            },
            queueCount: family.max_queues() as _,
            timestampValidBits: 0, //TODO
            minImageTransferGranularity: VkExtent3D { width: 0, height: 0, depth: 0 }, //TODO
        }
    }
}

extern "C" {
    pub fn vkGetPhysicalDeviceFeatures(physicalDevice: VkPhysicalDevice,
                                       pFeatures:
                                           *mut VkPhysicalDeviceFeatures);
}
#[inline]
pub extern fn gfxGetPhysicalDeviceFormatProperties(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    pFormatProperties: *mut VkFormatProperties,
) {
    let properties = adapter.physical_device.format_properties(conv::map_format(format));
    unsafe { *pFormatProperties = conv::format_properties_from_hal(properties); }
}
extern "C" {
    pub fn vkGetPhysicalDeviceImageFormatProperties(physicalDevice:
                                                        VkPhysicalDevice,
                                                    format: VkFormat,
                                                    type_: VkImageType,
                                                    tiling: VkImageTiling,
                                                    usage: VkImageUsageFlags,
                                                    flags: VkImageCreateFlags,
                                                    pImageFormatProperties:
                                                        *mut VkImageFormatProperties)
     -> VkResult;
}
extern "C" {
    pub fn vkGetPhysicalDeviceProperties(physicalDevice: VkPhysicalDevice,
                                         pProperties:
                                             *mut VkPhysicalDeviceProperties);
}

extern "C" {
    pub fn vkGetPhysicalDeviceMemoryProperties(physicalDevice:
                                                   VkPhysicalDevice,
                                               pMemoryProperties:
                                                   *mut VkPhysicalDeviceMemoryProperties);
}
extern "C" {
    pub fn vkGetInstanceProcAddr(instance: VkInstance,
                                 pName: *const ::std::os::raw::c_char)
     -> PFN_vkVoidFunction;
}
extern "C" {
    pub fn vkGetDeviceProcAddr(device: VkDevice,
                               pName: *const ::std::os::raw::c_char)
     -> PFN_vkVoidFunction;
}

#[inline]
pub extern fn gfxCreateDevice(
    adapter: VkPhysicalDevice,
    pCreateInfo: *const VkDeviceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pDevice: *mut VkDevice,
) -> VkResult {
    let dev_info = unsafe { &*pCreateInfo };
    let queue_infos = unsafe {
        slice::from_raw_parts(dev_info.pQueueCreateInfos, dev_info.queueCreateInfoCount as _)
    };
    let request_infos = queue_infos.iter().map(|info| {
        let family = adapter
            .queue_families[info.queueFamilyIndex as usize]
            .clone();
        (family, vec![1.0; info.queueCount as usize])
    }).collect::<Vec<_>>();

    let gpu = adapter.physical_device.clone().open(request_infos);
    unsafe { *pDevice = Handle::new(gpu) };

    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxDestroyDevice(
    device: VkDevice,
    _pAllocator: *const VkAllocationCallbacks,
) {
    let _ = device.unwrap(); //TODO?
}

lazy_static! {
    static ref INSTANCE_EXTENSIONS: [VkExtensionProperties; 1] = {
        let mut extensions = [
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_SURFACE_EXTENSION_NAME
                specVersion: VK_KHR_SURFACE_SPEC_VERSION,
            }
        ];

        extensions[0]
            .extensionName[..VK_KHR_SURFACE_EXTENSION_NAME.len()]
            .copy_from_slice(unsafe {
                mem::transmute(VK_KHR_SURFACE_EXTENSION_NAME as &[u8])
            });

        extensions
    };
}

#[inline]
pub extern fn gfxEnumerateInstanceExtensionProperties(
    pLayerName: *const ::std::os::raw::c_char,
    pPropertyCount: *mut u32,
    pProperties: *mut VkExtensionProperties,
) -> VkResult {
    let property_count = unsafe { &mut *pPropertyCount };
    let num_extensions = INSTANCE_EXTENSIONS.len() as u32;

    if pProperties.is_null() {
        *property_count = num_extensions;
    } else {
        if *property_count > num_extensions {
            *property_count = num_extensions;
        }
        let properties = unsafe { slice::from_raw_parts_mut(pProperties, *property_count as usize) };
        for i in 0..*property_count as usize {
            properties[i] = INSTANCE_EXTENSIONS[i];
        }

        if *property_count < num_extensions {
            return VkResult::VK_INCOMPLETE;
        }
    }

    VkResult::VK_SUCCESS
}

extern "C" {
    pub fn vkEnumerateDeviceExtensionProperties(physicalDevice:
                                                    VkPhysicalDevice,
                                                pLayerName:
                                                    *const ::std::os::raw::c_char,
                                                pPropertyCount: *mut u32,
                                                pProperties:
                                                    *mut VkExtensionProperties)
     -> VkResult;
}
extern "C" {
    pub fn vkEnumerateInstanceLayerProperties(pPropertyCount: *mut u32,
                                              pProperties:
                                                  *mut VkLayerProperties)
     -> VkResult;
}
extern "C" {
    pub fn vkEnumerateDeviceLayerProperties(physicalDevice: VkPhysicalDevice,
                                            pPropertyCount: *mut u32,
                                            pProperties:
                                                *mut VkLayerProperties)
     -> VkResult;
}
extern "C" {
    pub fn vkGetDeviceQueue(device: VkDevice, queueFamilyIndex: u32,
                            queueIndex: u32, pQueue: *mut VkQueue);
}
extern "C" {
    pub fn vkQueueSubmit(queue: VkQueue, submitCount: u32,
                         pSubmits: *const VkSubmitInfo, fence: VkFence)
     -> VkResult;
}
extern "C" {
    pub fn vkQueueWaitIdle(queue: VkQueue) -> VkResult;
}
extern "C" {
    pub fn vkDeviceWaitIdle(device: VkDevice) -> VkResult;
}
extern "C" {
    pub fn vkAllocateMemory(device: VkDevice,
                            pAllocateInfo: *const VkMemoryAllocateInfo,
                            pAllocator: *const VkAllocationCallbacks,
                            pMemory: *mut VkDeviceMemory) -> VkResult;
}
extern "C" {
    pub fn vkFreeMemory(device: VkDevice, memory: VkDeviceMemory,
                        pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkMapMemory(device: VkDevice, memory: VkDeviceMemory,
                       offset: VkDeviceSize, size: VkDeviceSize,
                       flags: VkMemoryMapFlags,
                       ppData: *mut *mut ::std::os::raw::c_void) -> VkResult;
}
extern "C" {
    pub fn vkUnmapMemory(device: VkDevice, memory: VkDeviceMemory);
}
extern "C" {
    pub fn vkFlushMappedMemoryRanges(device: VkDevice, memoryRangeCount: u32,
                                     pMemoryRanges:
                                         *const VkMappedMemoryRange)
     -> VkResult;
}
extern "C" {
    pub fn vkInvalidateMappedMemoryRanges(device: VkDevice,
                                          memoryRangeCount: u32,
                                          pMemoryRanges:
                                              *const VkMappedMemoryRange)
     -> VkResult;
}
extern "C" {
    pub fn vkGetDeviceMemoryCommitment(device: VkDevice,
                                       memory: VkDeviceMemory,
                                       pCommittedMemoryInBytes:
                                           *mut VkDeviceSize);
}
extern "C" {
    pub fn vkBindBufferMemory(device: VkDevice, buffer: VkBuffer,
                              memory: VkDeviceMemory,
                              memoryOffset: VkDeviceSize) -> VkResult;
}
extern "C" {
    pub fn vkBindImageMemory(device: VkDevice, image: VkImage,
                             memory: VkDeviceMemory,
                             memoryOffset: VkDeviceSize) -> VkResult;
}
extern "C" {
    pub fn vkGetBufferMemoryRequirements(device: VkDevice, buffer: VkBuffer,
                                         pMemoryRequirements:
                                             *mut VkMemoryRequirements);
}
#[inline]
pub extern fn gfxGetImageMemoryRequirements(
    gpu: VkDevice,
    image: VkImage,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    let req = match *image.deref() {
        Image::Image(ref image) => unimplemented!(),
        Image::Unbound(ref image) => {
            gpu.device.get_image_requirements(image)
        }
    };

    let memory_requirements = unsafe { &mut *pMemoryRequirements };
    memory_requirements.size = req.size;
    memory_requirements.alignment = req.alignment;
    memory_requirements.memoryTypeBits = req.type_mask as _;
}

extern "C" {
    pub fn vkGetImageSparseMemoryRequirements(device: VkDevice,
                                              image: VkImage,
                                              pSparseMemoryRequirementCount:
                                                  *mut u32,
                                              pSparseMemoryRequirements:
                                                  *mut VkSparseImageMemoryRequirements);
}
extern "C" {
    pub fn vkGetPhysicalDeviceSparseImageFormatProperties(physicalDevice:
                                                              VkPhysicalDevice,
                                                          format: VkFormat,
                                                          type_: VkImageType,
                                                          samples:
                                                              VkSampleCountFlagBits,
                                                          usage:
                                                              VkImageUsageFlags,
                                                          tiling:
                                                              VkImageTiling,
                                                          pPropertyCount:
                                                              *mut u32,
                                                          pProperties:
                                                              *mut VkSparseImageFormatProperties);
}
extern "C" {
    pub fn vkQueueBindSparse(queue: VkQueue, bindInfoCount: u32,
                             pBindInfo: *const VkBindSparseInfo,
                             fence: VkFence) -> VkResult;
}
extern "C" {
    pub fn vkCreateFence(device: VkDevice,
                         pCreateInfo: *const VkFenceCreateInfo,
                         pAllocator: *const VkAllocationCallbacks,
                         pFence: *mut VkFence) -> VkResult;
}
extern "C" {
    pub fn vkDestroyFence(device: VkDevice, fence: VkFence,
                          pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkResetFences(device: VkDevice, fenceCount: u32,
                         pFences: *const VkFence) -> VkResult;
}
extern "C" {
    pub fn vkGetFenceStatus(device: VkDevice, fence: VkFence) -> VkResult;
}
extern "C" {
    pub fn vkWaitForFences(device: VkDevice, fenceCount: u32,
                           pFences: *const VkFence, waitAll: VkBool32,
                           timeout: u64) -> VkResult;
}
extern "C" {
    pub fn vkCreateSemaphore(device: VkDevice,
                             pCreateInfo: *const VkSemaphoreCreateInfo,
                             pAllocator: *const VkAllocationCallbacks,
                             pSemaphore: *mut VkSemaphore) -> VkResult;
}
extern "C" {
    pub fn vkDestroySemaphore(device: VkDevice, semaphore: VkSemaphore,
                              pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreateEvent(device: VkDevice,
                         pCreateInfo: *const VkEventCreateInfo,
                         pAllocator: *const VkAllocationCallbacks,
                         pEvent: *mut VkEvent) -> VkResult;
}
extern "C" {
    pub fn vkDestroyEvent(device: VkDevice, event: VkEvent,
                          pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkGetEventStatus(device: VkDevice, event: VkEvent) -> VkResult;
}
extern "C" {
    pub fn vkSetEvent(device: VkDevice, event: VkEvent) -> VkResult;
}
extern "C" {
    pub fn vkResetEvent(device: VkDevice, event: VkEvent) -> VkResult;
}
extern "C" {
    pub fn vkCreateQueryPool(device: VkDevice,
                             pCreateInfo: *const VkQueryPoolCreateInfo,
                             pAllocator: *const VkAllocationCallbacks,
                             pQueryPool: *mut VkQueryPool) -> VkResult;
}
extern "C" {
    pub fn vkDestroyQueryPool(device: VkDevice, queryPool: VkQueryPool,
                              pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkGetQueryPoolResults(device: VkDevice, queryPool: VkQueryPool,
                                 firstQuery: u32, queryCount: u32,
                                 dataSize: usize,
                                 pData: *mut ::std::os::raw::c_void,
                                 stride: VkDeviceSize,
                                 flags: VkQueryResultFlags) -> VkResult;
}
extern "C" {
    pub fn vkCreateBuffer(device: VkDevice,
                          pCreateInfo: *const VkBufferCreateInfo,
                          pAllocator: *const VkAllocationCallbacks,
                          pBuffer: *mut VkBuffer) -> VkResult;
}
extern "C" {
    pub fn vkDestroyBuffer(device: VkDevice, buffer: VkBuffer,
                           pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreateBufferView(device: VkDevice,
                              pCreateInfo: *const VkBufferViewCreateInfo,
                              pAllocator: *const VkAllocationCallbacks,
                              pView: *mut VkBufferView) -> VkResult;
}
extern "C" {
    pub fn vkDestroyBufferView(device: VkDevice, bufferView: VkBufferView,
                               pAllocator: *const VkAllocationCallbacks);
}
#[inline]
pub extern fn gfxCreateImage(
    gpu: VkDevice,
    pCreateInfo: *const VkImageCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pImage: *mut VkImage,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    assert_eq!(info.sharingMode, VkSharingMode::VK_SHARING_MODE_EXCLUSIVE); // TODO
    assert_eq!(info.tiling, VkImageTiling::VK_IMAGE_TILING_OPTIMAL); // TODO
    assert_eq!(info.initialLayout, VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED); // TODO

    let image = gpu.device.create_image(
        conv::map_image_kind(info.imageType, info.flags, info.extent, info.arrayLayers, info.samples),
        info.mipLevels as _,
        conv::map_format(info.format),
        conv::map_image_usage(info.usage),
    ).expect("Error on creating image");

    unsafe { *pImage = Handle::new(Image::Unbound(image)); }
    VkResult::VK_SUCCESS
}
extern "C" {
    pub fn vkDestroyImage(device: VkDevice, image: VkImage,
                          pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkGetImageSubresourceLayout(device: VkDevice, image: VkImage,
                                       pSubresource:
                                           *const VkImageSubresource,
                                       pLayout: *mut VkSubresourceLayout);
}
#[inline]
pub extern fn gfxCreateImageView(
    gpu: VkDevice,
    pCreateInfo: *const VkImageViewCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pView: *mut VkImageView,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    assert!(info.subresourceRange.levelCount != VK_REMAINING_MIP_LEVELS as _); // TODO
    assert!(info.subresourceRange.layerCount != VK_REMAINING_ARRAY_LAYERS as _); // TODO

    let image = match *info.image.deref() {
        Image::Image(ref image) => image,
        // Non-sparse images must be bound prior.
        Image::Unbound(_) => panic!("Can't create view for unbound image"),
    };

    let view = gpu
        .device
        .create_image_view(
            image,
            conv::map_format(info.format),
            conv::map_swizzle(info.components),
            conv::map_subresource_range(info.subresourceRange),
        );

    match view {
        Ok(view) => {
            unsafe { *pView = Handle::new(view) };
            VkResult::VK_SUCCESS
        },
        Err(err) => {
            panic!("Unexpected image view creation error: {:?}", err)
        },
    }
}
#[inline]
pub extern fn gfxDestroyImageView(
    gpu: VkDevice,
    imageView: VkImageView,
    pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.destroy_image_view(*imageView.unwrap())
}
extern "C" {
    pub fn vkCreateShaderModule(device: VkDevice,
                                pCreateInfo: *const VkShaderModuleCreateInfo,
                                pAllocator: *const VkAllocationCallbacks,
                                pShaderModule: *mut VkShaderModule)
     -> VkResult;
}
extern "C" {
    pub fn vkDestroyShaderModule(device: VkDevice,
                                 shaderModule: VkShaderModule,
                                 pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreatePipelineCache(device: VkDevice,
                                 pCreateInfo:
                                     *const VkPipelineCacheCreateInfo,
                                 pAllocator: *const VkAllocationCallbacks,
                                 pPipelineCache: *mut VkPipelineCache)
     -> VkResult;
}
extern "C" {
    pub fn vkDestroyPipelineCache(device: VkDevice,
                                  pipelineCache: VkPipelineCache,
                                  pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkGetPipelineCacheData(device: VkDevice,
                                  pipelineCache: VkPipelineCache,
                                  pDataSize: *mut usize,
                                  pData: *mut ::std::os::raw::c_void)
     -> VkResult;
}
extern "C" {
    pub fn vkMergePipelineCaches(device: VkDevice, dstCache: VkPipelineCache,
                                 srcCacheCount: u32,
                                 pSrcCaches: *const VkPipelineCache)
     -> VkResult;
}
extern "C" {
    pub fn vkCreateGraphicsPipelines(device: VkDevice,
                                     pipelineCache: VkPipelineCache,
                                     createInfoCount: u32,
                                     pCreateInfos:
                                         *const VkGraphicsPipelineCreateInfo,
                                     pAllocator: *const VkAllocationCallbacks,
                                     pPipelines: *mut VkPipeline) -> VkResult;
}
extern "C" {
    pub fn vkCreateComputePipelines(device: VkDevice,
                                    pipelineCache: VkPipelineCache,
                                    createInfoCount: u32,
                                    pCreateInfos:
                                        *const VkComputePipelineCreateInfo,
                                    pAllocator: *const VkAllocationCallbacks,
                                    pPipelines: *mut VkPipeline) -> VkResult;
}
extern "C" {
    pub fn vkDestroyPipeline(device: VkDevice, pipeline: VkPipeline,
                             pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreatePipelineLayout(device: VkDevice,
                                  pCreateInfo:
                                      *const VkPipelineLayoutCreateInfo,
                                  pAllocator: *const VkAllocationCallbacks,
                                  pPipelineLayout: *mut VkPipelineLayout)
     -> VkResult;
}
extern "C" {
    pub fn vkDestroyPipelineLayout(device: VkDevice,
                                   pipelineLayout: VkPipelineLayout,
                                   pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreateSampler(device: VkDevice,
                           pCreateInfo: *const VkSamplerCreateInfo,
                           pAllocator: *const VkAllocationCallbacks,
                           pSampler: *mut VkSampler) -> VkResult;
}
extern "C" {
    pub fn vkDestroySampler(device: VkDevice, sampler: VkSampler,
                            pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreateDescriptorSetLayout(device: VkDevice,
                                       pCreateInfo:
                                           *const VkDescriptorSetLayoutCreateInfo,
                                       pAllocator:
                                           *const VkAllocationCallbacks,
                                       pSetLayout: *mut VkDescriptorSetLayout)
     -> VkResult;
}
extern "C" {
    pub fn vkDestroyDescriptorSetLayout(device: VkDevice,
                                        descriptorSetLayout:
                                            VkDescriptorSetLayout,
                                        pAllocator:
                                            *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreateDescriptorPool(device: VkDevice,
                                  pCreateInfo:
                                      *const VkDescriptorPoolCreateInfo,
                                  pAllocator: *const VkAllocationCallbacks,
                                  pDescriptorPool: *mut VkDescriptorPool)
     -> VkResult;
}
extern "C" {
    pub fn vkDestroyDescriptorPool(device: VkDevice,
                                   descriptorPool: VkDescriptorPool,
                                   pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkResetDescriptorPool(device: VkDevice,
                                 descriptorPool: VkDescriptorPool,
                                 flags: VkDescriptorPoolResetFlags)
     -> VkResult;
}
extern "C" {
    pub fn vkAllocateDescriptorSets(device: VkDevice,
                                    pAllocateInfo:
                                        *const VkDescriptorSetAllocateInfo,
                                    pDescriptorSets: *mut VkDescriptorSet)
     -> VkResult;
}
extern "C" {
    pub fn vkFreeDescriptorSets(device: VkDevice,
                                descriptorPool: VkDescriptorPool,
                                descriptorSetCount: u32,
                                pDescriptorSets: *const VkDescriptorSet)
     -> VkResult;
}
extern "C" {
    pub fn vkUpdateDescriptorSets(device: VkDevice, descriptorWriteCount: u32,
                                  pDescriptorWrites:
                                      *const VkWriteDescriptorSet,
                                  descriptorCopyCount: u32,
                                  pDescriptorCopies:
                                      *const VkCopyDescriptorSet);
}
extern "C" {
    pub fn vkCreateFramebuffer(device: VkDevice,
                               pCreateInfo: *const VkFramebufferCreateInfo,
                               pAllocator: *const VkAllocationCallbacks,
                               pFramebuffer: *mut VkFramebuffer) -> VkResult;
}
extern "C" {
    pub fn vkDestroyFramebuffer(device: VkDevice, framebuffer: VkFramebuffer,
                                pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreateRenderPass(device: VkDevice,
                              pCreateInfo: *const VkRenderPassCreateInfo,
                              pAllocator: *const VkAllocationCallbacks,
                              pRenderPass: *mut VkRenderPass) -> VkResult;
}
extern "C" {
    pub fn vkDestroyRenderPass(device: VkDevice, renderPass: VkRenderPass,
                               pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkGetRenderAreaGranularity(device: VkDevice,
                                      renderPass: VkRenderPass,
                                      pGranularity: *mut VkExtent2D);
}

#[inline]
pub extern fn gfxCreateCommandPool(
    gpu: VkDevice,
    pCreateInfo: *const VkCommandPoolCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pCommandPool: *mut VkCommandPool,
) -> VkResult {
    use hal::pool::CommandPoolCreateFlags;

    let info = unsafe { &*pCreateInfo };
    assert_eq!(info.queueFamilyIndex, 0); //TODO
    let family = gpu.queue_groups[0].family();

    let mut flags = CommandPoolCreateFlags::empty();
    if info.flags & VkCommandPoolCreateFlagBits::VK_COMMAND_POOL_CREATE_TRANSIENT_BIT as u32 != 0 {
        flags |= CommandPoolCreateFlags::TRANSIENT;
    }
    if info.flags & VkCommandPoolCreateFlagBits::VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT as u32 != 0 {
        flags |= CommandPoolCreateFlags::RESET_INDIVIDUAL;
    }

    let pool = gpu.device.create_command_pool(family, flags);
    unsafe { *pCommandPool = Handle::new(pool) };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxDestroyCommandPool(
    gpu: VkDevice,
    commandPool: VkCommandPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.destroy_command_pool(*commandPool.unwrap());
}

#[inline]
pub extern fn gfxResetCommandPool(
    _gpu: VkDevice,
    mut commandPool: VkCommandPool,
    _flags: VkCommandPoolResetFlags,
) -> VkResult {
    commandPool.reset();
    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxAllocateCommandBuffers(
    _gpu: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult {
    let info = unsafe { &mut *(pAllocateInfo as *mut VkCommandBufferAllocateInfo) };
    assert_eq!(info.level, VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY); //TODO
    let count = info.commandBufferCount as usize;

    let cmd_bufs = info.commandPool.allocate(count);

    let output = unsafe {
        slice::from_raw_parts_mut(pCommandBuffers, count)
    };
    for (out, cmd_buf) in output.iter_mut().zip(cmd_bufs) {
        *out = Handle::new(cmd_buf);
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxFreeCommandBuffers(
    _gpu: VkDevice,
    mut commandPool: VkCommandPool,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    let buffer_slice = unsafe {
        slice::from_raw_parts(pCommandBuffers, commandBufferCount as _)
    };
    let buffers = buffer_slice
      .iter()
      .map(|buffer| *buffer.unwrap())
      .collect();

    unsafe { commandPool.free(buffers) };
}

extern "C" {
    pub fn vkBeginCommandBuffer(commandBuffer: VkCommandBuffer,
                                pBeginInfo: *const VkCommandBufferBeginInfo)
     -> VkResult;
}
extern "C" {
    pub fn vkEndCommandBuffer(commandBuffer: VkCommandBuffer) -> VkResult;
}
extern "C" {
    pub fn vkResetCommandBuffer(commandBuffer: VkCommandBuffer,
                                flags: VkCommandBufferResetFlags) -> VkResult;
}
extern "C" {
    pub fn vkCmdBindPipeline(commandBuffer: VkCommandBuffer,
                             pipelineBindPoint: VkPipelineBindPoint,
                             pipeline: VkPipeline);
}
extern "C" {
    pub fn vkCmdSetViewport(commandBuffer: VkCommandBuffer,
                            firstViewport: u32, viewportCount: u32,
                            pViewports: *const VkViewport);
}
extern "C" {
    pub fn vkCmdSetScissor(commandBuffer: VkCommandBuffer, firstScissor: u32,
                           scissorCount: u32, pScissors: *const VkRect2D);
}
extern "C" {
    pub fn vkCmdSetLineWidth(commandBuffer: VkCommandBuffer, lineWidth: f32);
}
extern "C" {
    pub fn vkCmdSetDepthBias(commandBuffer: VkCommandBuffer,
                             depthBiasConstantFactor: f32,
                             depthBiasClamp: f32, depthBiasSlopeFactor: f32);
}
extern "C" {
    pub fn vkCmdSetBlendConstants(commandBuffer: VkCommandBuffer,
                                  blendConstants: *const f32);
}
extern "C" {
    pub fn vkCmdSetDepthBounds(commandBuffer: VkCommandBuffer,
                               minDepthBounds: f32, maxDepthBounds: f32);
}
extern "C" {
    pub fn vkCmdSetStencilCompareMask(commandBuffer: VkCommandBuffer,
                                      faceMask: VkStencilFaceFlags,
                                      compareMask: u32);
}
extern "C" {
    pub fn vkCmdSetStencilWriteMask(commandBuffer: VkCommandBuffer,
                                    faceMask: VkStencilFaceFlags,
                                    writeMask: u32);
}
extern "C" {
    pub fn vkCmdSetStencilReference(commandBuffer: VkCommandBuffer,
                                    faceMask: VkStencilFaceFlags,
                                    reference: u32);
}
extern "C" {
    pub fn vkCmdBindDescriptorSets(commandBuffer: VkCommandBuffer,
                                   pipelineBindPoint: VkPipelineBindPoint,
                                   layout: VkPipelineLayout, firstSet: u32,
                                   descriptorSetCount: u32,
                                   pDescriptorSets: *const VkDescriptorSet,
                                   dynamicOffsetCount: u32,
                                   pDynamicOffsets: *const u32);
}
extern "C" {
    pub fn vkCmdBindIndexBuffer(commandBuffer: VkCommandBuffer,
                                buffer: VkBuffer, offset: VkDeviceSize,
                                indexType: VkIndexType);
}
extern "C" {
    pub fn vkCmdBindVertexBuffers(commandBuffer: VkCommandBuffer,
                                  firstBinding: u32, bindingCount: u32,
                                  pBuffers: *const VkBuffer,
                                  pOffsets: *const VkDeviceSize);
}
extern "C" {
    pub fn vkCmdDraw(commandBuffer: VkCommandBuffer, vertexCount: u32,
                     instanceCount: u32, firstVertex: u32,
                     firstInstance: u32);
}
extern "C" {
    pub fn vkCmdDrawIndexed(commandBuffer: VkCommandBuffer, indexCount: u32,
                            instanceCount: u32, firstIndex: u32,
                            vertexOffset: i32, firstInstance: u32);
}
extern "C" {
    pub fn vkCmdDrawIndirect(commandBuffer: VkCommandBuffer, buffer: VkBuffer,
                             offset: VkDeviceSize, drawCount: u32,
                             stride: u32);
}
extern "C" {
    pub fn vkCmdDrawIndexedIndirect(commandBuffer: VkCommandBuffer,
                                    buffer: VkBuffer, offset: VkDeviceSize,
                                    drawCount: u32, stride: u32);
}
extern "C" {
    pub fn vkCmdDispatch(commandBuffer: VkCommandBuffer, groupCountX: u32,
                         groupCountY: u32, groupCountZ: u32);
}
extern "C" {
    pub fn vkCmdDispatchIndirect(commandBuffer: VkCommandBuffer,
                                 buffer: VkBuffer, offset: VkDeviceSize);
}
extern "C" {
    pub fn vkCmdCopyBuffer(commandBuffer: VkCommandBuffer,
                           srcBuffer: VkBuffer, dstBuffer: VkBuffer,
                           regionCount: u32, pRegions: *const VkBufferCopy);
}
extern "C" {
    pub fn vkCmdCopyImage(commandBuffer: VkCommandBuffer, srcImage: VkImage,
                          srcImageLayout: VkImageLayout, dstImage: VkImage,
                          dstImageLayout: VkImageLayout, regionCount: u32,
                          pRegions: *const VkImageCopy);
}
extern "C" {
    pub fn vkCmdBlitImage(commandBuffer: VkCommandBuffer, srcImage: VkImage,
                          srcImageLayout: VkImageLayout, dstImage: VkImage,
                          dstImageLayout: VkImageLayout, regionCount: u32,
                          pRegions: *const VkImageBlit, filter: VkFilter);
}
extern "C" {
    pub fn vkCmdCopyBufferToImage(commandBuffer: VkCommandBuffer,
                                  srcBuffer: VkBuffer, dstImage: VkImage,
                                  dstImageLayout: VkImageLayout,
                                  regionCount: u32,
                                  pRegions: *const VkBufferImageCopy);
}
extern "C" {
    pub fn vkCmdCopyImageToBuffer(commandBuffer: VkCommandBuffer,
                                  srcImage: VkImage,
                                  srcImageLayout: VkImageLayout,
                                  dstBuffer: VkBuffer, regionCount: u32,
                                  pRegions: *const VkBufferImageCopy);
}
extern "C" {
    pub fn vkCmdUpdateBuffer(commandBuffer: VkCommandBuffer,
                             dstBuffer: VkBuffer, dstOffset: VkDeviceSize,
                             dataSize: VkDeviceSize,
                             pData: *const ::std::os::raw::c_void);
}
extern "C" {
    pub fn vkCmdFillBuffer(commandBuffer: VkCommandBuffer,
                           dstBuffer: VkBuffer, dstOffset: VkDeviceSize,
                           size: VkDeviceSize, data: u32);
}
extern "C" {
    pub fn vkCmdClearColorImage(commandBuffer: VkCommandBuffer,
                                image: VkImage, imageLayout: VkImageLayout,
                                pColor: *const VkClearColorValue,
                                rangeCount: u32,
                                pRanges: *const VkImageSubresourceRange);
}
extern "C" {
    pub fn vkCmdClearDepthStencilImage(commandBuffer: VkCommandBuffer,
                                       image: VkImage,
                                       imageLayout: VkImageLayout,
                                       pDepthStencil:
                                           *const VkClearDepthStencilValue,
                                       rangeCount: u32,
                                       pRanges:
                                           *const VkImageSubresourceRange);
}
extern "C" {
    pub fn vkCmdClearAttachments(commandBuffer: VkCommandBuffer,
                                 attachmentCount: u32,
                                 pAttachments: *const VkClearAttachment,
                                 rectCount: u32, pRects: *const VkClearRect);
}
extern "C" {
    pub fn vkCmdResolveImage(commandBuffer: VkCommandBuffer,
                             srcImage: VkImage, srcImageLayout: VkImageLayout,
                             dstImage: VkImage, dstImageLayout: VkImageLayout,
                             regionCount: u32,
                             pRegions: *const VkImageResolve);
}
extern "C" {
    pub fn vkCmdSetEvent(commandBuffer: VkCommandBuffer, event: VkEvent,
                         stageMask: VkPipelineStageFlags);
}
extern "C" {
    pub fn vkCmdResetEvent(commandBuffer: VkCommandBuffer, event: VkEvent,
                           stageMask: VkPipelineStageFlags);
}
extern "C" {
    pub fn vkCmdWaitEvents(commandBuffer: VkCommandBuffer, eventCount: u32,
                           pEvents: *const VkEvent,
                           srcStageMask: VkPipelineStageFlags,
                           dstStageMask: VkPipelineStageFlags,
                           memoryBarrierCount: u32,
                           pMemoryBarriers: *const VkMemoryBarrier,
                           bufferMemoryBarrierCount: u32,
                           pBufferMemoryBarriers:
                               *const VkBufferMemoryBarrier,
                           imageMemoryBarrierCount: u32,
                           pImageMemoryBarriers: *const VkImageMemoryBarrier);
}
extern "C" {
    pub fn vkCmdPipelineBarrier(commandBuffer: VkCommandBuffer,
                                srcStageMask: VkPipelineStageFlags,
                                dstStageMask: VkPipelineStageFlags,
                                dependencyFlags: VkDependencyFlags,
                                memoryBarrierCount: u32,
                                pMemoryBarriers: *const VkMemoryBarrier,
                                bufferMemoryBarrierCount: u32,
                                pBufferMemoryBarriers:
                                    *const VkBufferMemoryBarrier,
                                imageMemoryBarrierCount: u32,
                                pImageMemoryBarriers:
                                    *const VkImageMemoryBarrier);
}
extern "C" {
    pub fn vkCmdBeginQuery(commandBuffer: VkCommandBuffer,
                           queryPool: VkQueryPool, query: u32,
                           flags: VkQueryControlFlags);
}
extern "C" {
    pub fn vkCmdEndQuery(commandBuffer: VkCommandBuffer,
                         queryPool: VkQueryPool, query: u32);
}
extern "C" {
    pub fn vkCmdResetQueryPool(commandBuffer: VkCommandBuffer,
                               queryPool: VkQueryPool, firstQuery: u32,
                               queryCount: u32);
}
extern "C" {
    pub fn vkCmdWriteTimestamp(commandBuffer: VkCommandBuffer,
                               pipelineStage: VkPipelineStageFlagBits,
                               queryPool: VkQueryPool, query: u32);
}
extern "C" {
    pub fn vkCmdCopyQueryPoolResults(commandBuffer: VkCommandBuffer,
                                     queryPool: VkQueryPool, firstQuery: u32,
                                     queryCount: u32, dstBuffer: VkBuffer,
                                     dstOffset: VkDeviceSize,
                                     stride: VkDeviceSize,
                                     flags: VkQueryResultFlags);
}
extern "C" {
    pub fn vkCmdPushConstants(commandBuffer: VkCommandBuffer,
                              layout: VkPipelineLayout,
                              stageFlags: VkShaderStageFlags, offset: u32,
                              size: u32,
                              pValues: *const ::std::os::raw::c_void);
}
extern "C" {
    pub fn vkCmdBeginRenderPass(commandBuffer: VkCommandBuffer,
                                pRenderPassBegin:
                                    *const VkRenderPassBeginInfo,
                                contents: VkSubpassContents);
}
extern "C" {
    pub fn vkCmdNextSubpass(commandBuffer: VkCommandBuffer,
                            contents: VkSubpassContents);
}
extern "C" {
    pub fn vkCmdEndRenderPass(commandBuffer: VkCommandBuffer);
}
extern "C" {
    pub fn vkCmdExecuteCommands(commandBuffer: VkCommandBuffer,
                                commandBufferCount: u32,
                                pCommandBuffers: *const VkCommandBuffer);
}

#[inline]
pub extern fn gfxDestroySurfaceKHR(
    _instance: VkInstance,
    surface: VkSurfaceKHR,
    _: *const VkAllocationCallbacks,
) {
    let _ = surface.unwrap(); //TODO
}

#[inline]
pub extern fn gfxGetPhysicalDeviceSurfaceSupportKHR(
    adapter: VkPhysicalDevice,
    queueFamilyIndex: u32,
    surface: VkSurfaceKHR,
    pSupported: *mut VkBool32,
) -> VkResult {
    let family = &adapter.queue_families[queueFamilyIndex as usize];
    let supports = surface.supports_queue_family(family);
    unsafe { *pSupported = supports as _ };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxGetPhysicalDeviceSurfaceCapabilitiesKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceCapabilities: *mut VkSurfaceCapabilitiesKHR,
) -> VkResult {
    let (caps, _) = surface.capabilities_and_formats(&adapter.physical_device);

    let output = VkSurfaceCapabilitiesKHR {
        minImageCount: caps.image_count.start,
        maxImageCount: caps.image_count.end,
        currentExtent: match caps.current_extent {
            Some(extent) => conv::extent2d_from_hal(extent),
            None => VkExtent2D {
                width: !0,
                height: !0,
            },
        },
        minImageExtent: conv::extent2d_from_hal(caps.extents.start),
        maxImageExtent: conv::extent2d_from_hal(caps.extents.end),
        maxImageArrayLayers: caps.max_image_layers,
        supportedTransforms: VkSurfaceTransformFlagBitsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR as _,
        currentTransform: VkSurfaceTransformFlagBitsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        supportedCompositeAlpha: VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR as _,
        supportedUsageFlags: 0,
    };

    unsafe { *pSurfaceCapabilities = output };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxGetPhysicalDeviceSurfaceFormatsKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceFormatCount: *mut u32,
    pSurfaceFormats: *mut VkSurfaceFormatKHR,
) -> VkResult {
    let (_, formats) = surface.capabilities_and_formats(&adapter.physical_device);
    let output = unsafe { slice::from_raw_parts_mut(pSurfaceFormats, *pSurfaceFormatCount as usize) };

    if output.len() > formats.len() {
        unsafe { *pSurfaceFormatCount = formats.len() as u32 };
    }
    for (out, format) in output.iter_mut().zip(formats) {
        *out = VkSurfaceFormatKHR {
            format: conv::format_from_hal(format),
            colorSpace: VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR, //TODO
        };
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxGetPhysicalDeviceSurfacePresentModesKHR(
    _adapter: VkPhysicalDevice,
    _surface: VkSurfaceKHR,
    pPresentModeCount: *mut u32,
    pPresentModes: *mut VkPresentModeKHR,
) -> VkResult {
    let modes = vec![VkPresentModeKHR::VK_PRESENT_MODE_FIFO_KHR]; //TODO
    let output = unsafe { slice::from_raw_parts_mut(pPresentModes, *pPresentModeCount as usize) };

    if output.len() > modes.len() {
        unsafe { *pPresentModeCount = modes.len() as u32 };
    }
    for (out, mode) in output.iter_mut().zip(modes) {
        *out = mode;
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern fn gfxCreateSwapchainKHR(
    gpu: VkDevice,
    pCreateInfo: *const VkSwapchainCreateInfoKHR,
    _pAllocator: *const VkAllocationCallbacks,
    pSwapchain: *mut VkSwapchainKHR,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    // TODO: more checks
    assert_eq!(info.clipped, VK_TRUE); // TODO
    assert_eq!(info.imageSharingMode, VkSharingMode::VK_SHARING_MODE_EXCLUSIVE); // TODO

    let config = hal::SwapchainConfig {
        color_format: conv::map_format(info.imageFormat),
        depth_stencil_format: None,
        image_count: info.minImageCount,
    };
    let (swapchain, backbuffers) = gpu.device.create_swapchain(&mut info.surface.clone(), config);

    let images = match backbuffers {
        hal::Backbuffer::Images(images) => {
            images.into_iter().map(|image| Handle::new(Image::Image(image))).collect()
        },
        hal::Backbuffer::Framebuffer(_) => {
            panic!("Expected backbuffer images. Backends returning only framebuffers are not supported!")
        },
    };

    let swapchain = Swapchain {
        raw: swapchain,
        images,
    };

    unsafe { *pSwapchain = Handle::new(swapchain) };
    VkResult::VK_SUCCESS
}
#[inline]
pub extern fn gfxDestroySwapchainKHR(
    device: VkDevice,
    mut swapchain: VkSwapchainKHR,
    pAllocator: *const VkAllocationCallbacks,
) {
    for image in &mut swapchain.images {
        let _ = image.unwrap();
    }
    let _ = swapchain.unwrap();
}
#[inline]
pub extern fn gfxGetSwapchainImagesKHR(
    device: VkDevice,
    swapchain: VkSwapchainKHR,
    pSwapchainImageCount: *mut u32,
    pSwapchainImages: *mut VkImage,
) -> VkResult {
    debug_assert!(!pSwapchainImageCount.is_null());

    let swapchain_image_count = unsafe { &mut*pSwapchainImageCount };
    let available_images = swapchain.images.len() as u32;

    if pSwapchainImages.is_null() {
        // If NULL the number of presentable images is returned.
        *swapchain_image_count = available_images;
    } else {
        *swapchain_image_count = available_images.min(*swapchain_image_count);
        let swapchain_images = unsafe {
            slice::from_raw_parts_mut(pSwapchainImages, *swapchain_image_count as _)
        };

        for i in 0..*swapchain_image_count as _ {
            swapchain_images[i] = swapchain.images[i];
        }

        if *swapchain_image_count < available_images {
            return VkResult::VK_INCOMPLETE;
        }
    }

    VkResult::VK_SUCCESS
}

extern "C" {
    pub fn vkCmdProcessCommandsNVX(commandBuffer: VkCommandBuffer,
                                   pProcessCommandsInfo:
                                       *const VkCmdProcessCommandsInfoNVX);
}
extern "C" {
    pub fn vkCmdReserveSpaceForCommandsNVX(commandBuffer: VkCommandBuffer,
                                           pReserveSpaceInfo:
                                               *const VkCmdReserveSpaceForCommandsInfoNVX);
}
extern "C" {
    pub fn vkCreateIndirectCommandsLayoutNVX(device: VkDevice,
                                             pCreateInfo:
                                                 *const VkIndirectCommandsLayoutCreateInfoNVX,
                                             pAllocator:
                                                 *const VkAllocationCallbacks,
                                             pIndirectCommandsLayout:
                                                 *mut VkIndirectCommandsLayoutNVX)
     -> VkResult;
}
extern "C" {
    pub fn vkDestroyIndirectCommandsLayoutNVX(device: VkDevice,
                                              indirectCommandsLayout:
                                                  VkIndirectCommandsLayoutNVX,
                                              pAllocator:
                                                  *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkCreateObjectTableNVX(device: VkDevice,
                                  pCreateInfo:
                                      *const VkObjectTableCreateInfoNVX,
                                  pAllocator: *const VkAllocationCallbacks,
                                  pObjectTable: *mut VkObjectTableNVX)
     -> VkResult;
}
extern "C" {
    pub fn vkDestroyObjectTableNVX(device: VkDevice,
                                   objectTable: VkObjectTableNVX,
                                   pAllocator: *const VkAllocationCallbacks);
}
extern "C" {
    pub fn vkRegisterObjectsNVX(device: VkDevice,
                                objectTable: VkObjectTableNVX,
                                objectCount: u32,
                                ppObjectTableEntries:
                                    *const *const VkObjectTableEntryNVX,
                                pObjectIndices: *const u32) -> VkResult;
}
extern "C" {
    pub fn vkUnregisterObjectsNVX(device: VkDevice,
                                  objectTable: VkObjectTableNVX,
                                  objectCount: u32,
                                  pObjectEntryTypes:
                                      *const VkObjectEntryTypeNVX,
                                  pObjectIndices: *const u32) -> VkResult;
}
extern "C" {
    pub fn vkGetPhysicalDeviceGeneratedCommandsPropertiesNVX(physicalDevice:
                                                                 VkPhysicalDevice,
                                                             pFeatures:
                                                                 *mut VkDeviceGeneratedCommandsFeaturesNVX,
                                                             pLimits:
                                                                 *mut VkDeviceGeneratedCommandsLimitsNVX);
}
extern "C" {
    pub fn vkCmdSetViewportWScalingNV(commandBuffer: VkCommandBuffer,
                                      firstViewport: u32, viewportCount: u32,
                                      pViewportWScalings:
                                          *const VkViewportWScalingNV);
}
extern "C" {
    pub fn vkReleaseDisplayEXT(physicalDevice: VkPhysicalDevice,
                               display: VkDisplayKHR) -> VkResult;
}
extern "C" {
    pub fn vkGetPhysicalDeviceSurfaceCapabilities2EXT(physicalDevice:
                                                          VkPhysicalDevice,
                                                      surface: VkSurfaceKHR,
                                                      pSurfaceCapabilities:
                                                          *mut VkSurfaceCapabilities2EXT)
     -> VkResult;
}
extern "C" {
    pub fn vkDisplayPowerControlEXT(device: VkDevice, display: VkDisplayKHR,
                                    pDisplayPowerInfo:
                                        *const VkDisplayPowerInfoEXT)
     -> VkResult;
}
extern "C" {
    pub fn vkRegisterDeviceEventEXT(device: VkDevice,
                                    pDeviceEventInfo:
                                        *const VkDeviceEventInfoEXT,
                                    pAllocator: *const VkAllocationCallbacks,
                                    pFence: *mut VkFence) -> VkResult;
}
extern "C" {
    pub fn vkRegisterDisplayEventEXT(device: VkDevice, display: VkDisplayKHR,
                                     pDisplayEventInfo:
                                         *const VkDisplayEventInfoEXT,
                                     pAllocator: *const VkAllocationCallbacks,
                                     pFence: *mut VkFence) -> VkResult;
}
extern "C" {
    pub fn vkGetSwapchainCounterEXT(device: VkDevice,
                                    swapchain: VkSwapchainKHR,
                                    counter: VkSurfaceCounterFlagBitsEXT,
                                    pCounterValue: *mut u64) -> VkResult;
}
extern "C" {
    pub fn vkCmdSetDiscardRectangleEXT(commandBuffer: VkCommandBuffer,
                                       firstDiscardRectangle: u32,
                                       discardRectangleCount: u32,
                                       pDiscardRectangles: *const VkRect2D);
}
