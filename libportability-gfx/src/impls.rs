use hal::{command as com, memory, pass, pso, queue};
use hal::{
    DescriptorPool, Device, Features, Instance, PhysicalDevice, QueueFamily,
    Surface, Swapchain as HalSwapchain, FrameSync,
};
use hal::buffer::IndexBufferView;
use hal::command::RawCommandBuffer;
use hal::device::WaitFor;
use hal::pool::RawCommandPool;
use hal::queue::RawCommandQueue;

use smallvec::SmallVec;

use std::ffi::{CStr, CString};
#[cfg(feature = "renderdoc")]
use std::os::raw::c_void;
use std::{mem, ptr};

use super::*;

const VERSION: (u32, u32, u32) = (1, 0, 66);
const DRIVER_VERSION: u32 = 1;

#[macro_export]
macro_rules! proc_addr {
    ($name:expr, $($vk:ident, $pfn_vk:ident => $gfx:expr,)*) => (
        match $name {
            $(
                stringify!($vk) => unsafe {
                    mem::transmute::<$pfn_vk, _>(Some(*&$gfx))
                },
            )*
            _ => None
        }
    );
}

#[inline]
pub extern "C" fn gfxCreateInstance(
    pCreateInfo: *const VkInstanceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult {
    #[cfg(feature = "env_logger")]
    {
        use env_logger;
        let _ = env_logger::try_init();
    }

    let backend = back::Instance::create("portability", 1);
    let adapters = backend
        .enumerate_adapters()
        .into_iter()
        .map(Handle::new)
        .collect();

    unsafe {
        let create_info = &*pCreateInfo;

        let application_info = create_info.pApplicationInfo.as_ref();

        if let Some(ai) = application_info {
            // Compare major and minor parts of version only - patch is ignored
            let (supported_major, supported_minor, _) = VERSION;
            let requested_major_minor = ai.apiVersion >> 12;
            let version_supported = requested_major_minor
                & (supported_major << 10 | supported_minor)
                == requested_major_minor;
            if !version_supported {
                return VkResult::VK_ERROR_INCOMPATIBLE_DRIVER;
            }
        }

        let enabled_extensions = if create_info.enabledExtensionCount == 0 {
            Vec::new()
        } else {
            let extensions = slice::from_raw_parts(
                create_info.ppEnabledExtensionNames,
                create_info.enabledExtensionCount as _,
            ).iter()
                .map(|raw| {
                    CStr::from_ptr(*raw)
                        .to_str()
                        .expect("Invalid extension name")
                        .to_owned()
                })
                .collect::<Vec<_>>();
            for extension in &extensions {
                if !INSTANCE_EXTENSION_NAMES.contains(&extension.as_str()) {
                    return VkResult::VK_ERROR_EXTENSION_NOT_PRESENT;
                }
            }
            extensions
        };

        *pInstance = Handle::new(RawInstance {
            backend,
            adapters,
            enabled_extensions,
        });
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxDestroyInstance(
    instance: VkInstance,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(i) = instance.unbox() {
        for adapter in i.adapters {
            let _ = adapter.unbox();
        }
    }
    #[cfg(feature = "nightly")]
    {
        Handle::report_leaks();
    }
}

#[inline]
pub extern "C" fn gfxEnumeratePhysicalDevices(
    instance: VkInstance,
    pPhysicalDeviceCount: *mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
) -> VkResult {
    let num_adapters = instance.adapters.len();

    // If NULL, number of devices is returned.
    if pPhysicalDevices.is_null() {
        unsafe { *pPhysicalDeviceCount = num_adapters as _ };
        return VkResult::VK_SUCCESS;
    }

    let output = unsafe { slice::from_raw_parts_mut(pPhysicalDevices, *pPhysicalDeviceCount as _) };
    let num_output = output.len();
    let (code, count) = if num_output < num_adapters {
        (VkResult::VK_INCOMPLETE, num_output)
    } else {
        (VkResult::VK_SUCCESS, num_adapters)
    };

    output.copy_from_slice(&instance.adapters[..count]);
    unsafe { *pPhysicalDeviceCount = count as _ };

    code
}

#[inline]
pub extern "C" fn gfxGetPhysicalDeviceQueueFamilyProperties(
    adapter: VkPhysicalDevice,
    pQueueFamilyPropertyCount: *mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
) {
    let families = &adapter.queue_families;

    // If NULL, number of queue families is returned.
    if pQueueFamilyProperties.is_null() {
        unsafe { *pQueueFamilyPropertyCount = families.len() as _ };
        return;
    }

    let output = unsafe {
        slice::from_raw_parts_mut(pQueueFamilyProperties, *pQueueFamilyPropertyCount as _)
    };
    if output.len() > families.len() {
        unsafe { *pQueueFamilyPropertyCount = families.len() as _ };
    }
    for (ref mut out, ref family) in output.iter_mut().zip(families.iter()) {
        **out = VkQueueFamilyProperties {
            queueFlags: match family.queue_type() {
                hal::QueueType::General => {
                    VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT as u32
                        | VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT as u32
                }
                hal::QueueType::Graphics => VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT as u32,
                hal::QueueType::Compute => VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT as u32,
                hal::QueueType::Transfer => VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT as u32,
            },
            queueCount: family.max_queues() as _,
            timestampValidBits: 0, //TODO
            minImageTransferGranularity: VkExtent3D {
                width: 0,
                height: 0,
                depth: 0,
            }, //TODO
        }
    }
}

#[inline]
pub extern "C" fn gfxGetPhysicalDeviceFeatures(
    adapter: VkPhysicalDevice,
    pFeatures: *mut VkPhysicalDeviceFeatures,
) {
    let features = adapter.physical_device.features();

    unsafe {
        *pFeatures = VkPhysicalDeviceFeatures {
            robustBufferAccess: features.contains(Features::ROBUST_BUFFER_ACCESS) as _,
            fullDrawIndexUint32: features.contains(Features::FULL_DRAW_INDEX_U32) as _,
            imageCubeArray: features.contains(Features::IMAGE_CUBE_ARRAY) as _,
            independentBlend: features.contains(Features::INDEPENDENT_BLENDING) as _,
            geometryShader: features.contains(Features::GEOMETRY_SHADER) as _,
            tessellationShader: features.contains(Features::TESSELLATION_SHADER) as _,
            sampleRateShading: features.contains(Features::SAMPLE_RATE_SHADING) as _,
            dualSrcBlend: features.contains(Features::DUAL_SRC_BLENDING) as _,
            logicOp: features.contains(Features::LOGIC_OP) as _,
            multiDrawIndirect: features.contains(Features::MULTI_DRAW_INDIRECT) as _,
            drawIndirectFirstInstance: features.contains(Features::DRAW_INDIRECT_FIRST_INSTANCE) as _,
            depthClamp: features.contains(Features::DEPTH_CLAMP) as _,
            depthBiasClamp: features.contains(Features::DEPTH_BIAS_CLAMP) as _,
            fillModeNonSolid: features.contains(Features::NON_FILL_POLYGON_MODE) as _,
            depthBounds: features.contains(Features::DEPTH_BOUNDS) as _,
            wideLines: features.contains(Features::LINE_WIDTH) as _,
            largePoints: features.contains(Features::POINT_SIZE) as _,
            alphaToOne: features.contains(Features::ALPHA_TO_ONE) as _,
            multiViewport: features.contains(Features::MULTI_VIEWPORTS) as _,
            samplerAnisotropy: features.contains(Features::SAMPLER_ANISOTROPY) as _,
            textureCompressionETC2: features.contains(Features::FORMAT_ETC2) as _,
            textureCompressionASTC_LDR: features.contains(Features::FORMAT_ASTC_LDR) as _,
            textureCompressionBC: features.contains(Features::FORMAT_BC) as _,
            occlusionQueryPrecise: features.contains(Features::PRECISE_OCCLUSION_QUERY) as _,
            pipelineStatisticsQuery: features.contains(Features::PIPELINE_STATISTICS_QUERY) as _,
            vertexPipelineStoresAndAtomics: features.contains(Features::VERTEX_STORES_AND_ATOMICS) as _,
            fragmentStoresAndAtomics: features.contains(Features::FRAGMENT_STORES_AND_ATOMICS) as _,
            shaderTessellationAndGeometryPointSize: features.contains(Features::SHADER_TESSELLATION_AND_GEOMETRY_POINT_SIZE) as _,
            shaderImageGatherExtended: features.contains(Features::SHADER_IMAGE_GATHER_EXTENDED) as _,
            shaderStorageImageExtendedFormats: features.contains(Features::SHADER_STORAGE_IMAGE_EXTENDED_FORMATS) as _,
            shaderStorageImageMultisample: features.contains(Features::SHADER_STORAGE_IMAGE_MULTISAMPLE) as _,
            shaderStorageImageReadWithoutFormat: features.contains(Features::SHADER_STORAGE_IMAGE_READ_WITHOUT_FORMAT) as _,
            shaderStorageImageWriteWithoutFormat: features.contains(Features::SHADER_STORAGE_IMAGE_WRITE_WITHOUT_FORMAT) as _,
            shaderUniformBufferArrayDynamicIndexing: features.contains(Features::SHADER_UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING) as _,
            shaderSampledImageArrayDynamicIndexing: features.contains(Features::SHADER_SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING) as _,
            shaderStorageBufferArrayDynamicIndexing: features.contains(Features::SHADER_STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING) as _,
            shaderStorageImageArrayDynamicIndexing: features.contains(Features::SHADER_STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING) as _,
            shaderClipDistance: features.contains(Features::SHADER_CLIP_DISTANCE) as _,
            shaderCullDistance: features.contains(Features::SHADER_CULL_DISTANCE) as _,
            shaderFloat64: features.contains(Features::SHADER_FLOAT64) as _,
            shaderInt64: features.contains(Features::SHADER_INT64) as _,
            shaderInt16: features.contains(Features::SHADER_INT16) as _,
            shaderResourceResidency: features.contains(Features::SHADER_RESOURCE_RESIDENCY) as _,
            shaderResourceMinLod: features.contains(Features::SHADER_RESOURCE_MIN_LOD) as _,
            sparseBinding: features.contains(Features::SPARSE_BINDING) as _,
            sparseResidencyBuffer: features.contains(Features::SPARSE_RESIDENCY_BUFFER) as _,
            sparseResidencyImage2D: features.contains(Features::SHADER_RESIDENCY_IMAGE_2D) as _,
            sparseResidencyImage3D: features.contains(Features::SHADER_RESIDENSY_IMAGE_3D) as _,
            sparseResidency2Samples: features.contains(Features::SPARSE_RESIDENCY_2_SAMPLES) as _,
            sparseResidency4Samples: features.contains(Features::SPARSE_RESIDENCY_4_SAMPLES) as _,
            sparseResidency8Samples: features.contains(Features::SPARSE_RESIDENCY_8_SAMPLES) as _,
            sparseResidency16Samples: features.contains(Features::SPARSE_RESIDENCY_16_SAMPLES) as _,
            sparseResidencyAliased: features.contains(Features::SPARSE_RESIDENCY_ALIASED) as _,
            variableMultisampleRate: features.contains(Features::VARIABLE_MULTISAMPLE_RATE) as _,
            inheritedQueries: features.contains(Features::INHERITED_QUERIES) as _,
        };
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceFormatProperties(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    pFormatProperties: *mut VkFormatProperties,
) {
    let properties = adapter.physical_device.format_properties(conv::map_format(format));
    unsafe {
        *pFormatProperties = conv::format_properties_from_hal(properties);
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceImageFormatProperties(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    typ: VkImageType,
    tiling: VkImageTiling,
    usage: VkImageUsageFlags,
    create_flags: VkImageCreateFlags,
    pImageFormatProperties: *mut VkImageFormatProperties,
) -> VkResult {
    let properties = adapter.physical_device.image_format_properties(
        conv::map_format(format).unwrap(),
        match typ {
            VkImageType::VK_IMAGE_TYPE_1D => 1,
            VkImageType::VK_IMAGE_TYPE_2D => 2,
            VkImageType::VK_IMAGE_TYPE_3D => 3,
            _ => panic!("Unexpected image type: {:?}", typ),
        },
        conv::map_tiling(tiling),
        conv::map_image_usage(usage),
        unsafe { mem::transmute(create_flags) },
    );
    match properties {
        Some(props) => unsafe {
            *pImageFormatProperties = conv::image_format_properties_from_hal(props);
            VkResult::VK_SUCCESS
        },
        None => VkResult::VK_ERROR_FORMAT_NOT_SUPPORTED
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceProperties(
    adapter: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
) {
    let adapter_info = &adapter.info;
    let limits = conv::limits_from_hal(adapter.physical_device.limits());
    let sparse_properties = unsafe { mem::zeroed() }; // TODO
    let (major, minor, patch) = VERSION;

    let device_name = {
        let c_string = CString::new(adapter_info.name.clone()).unwrap();
        let c_str = c_string.as_bytes_with_nul();
        let mut name = [0; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE as _];
        let len = name.len().min(c_str.len()) - 1;
        name[..len].copy_from_slice(&c_str[..len]);
        unsafe { mem::transmute(name) }
    };

    unsafe {
        *pProperties = VkPhysicalDeviceProperties {
            apiVersion: (major << 22) | (minor << 12) | patch,
            driverVersion: DRIVER_VERSION,
            vendorID: adapter_info.vendor as _,
            deviceID: adapter_info.device as _,
            deviceType: VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU, // TODO
            deviceName: device_name,
            pipelineCacheUUID: [0; 16usize],
            limits,
            sparseProperties: sparse_properties,
        };
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceMemoryProperties(
    adapter: VkPhysicalDevice,
    pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties,
) {
    let properties = adapter.physical_device.memory_properties();
    let memory_properties = unsafe { &mut *pMemoryProperties };

    let num_types = properties.memory_types.len();
    memory_properties.memoryTypeCount = num_types as _;
    for i in 0..num_types {
        let flags = conv::memory_properties_from_hal(properties.memory_types[i].properties);
        memory_properties.memoryTypes[i] = VkMemoryType {
            propertyFlags: flags, // propertyFlags
            heapIndex: properties.memory_types[i].heap_index as _,
        };
    }

    let num_heaps = properties.memory_heaps.len();
    memory_properties.memoryHeapCount = num_heaps as _;
    for i in 0..num_heaps {
        memory_properties.memoryHeaps[i] = VkMemoryHeap {
            size: properties.memory_heaps[i],
            flags: 0, // TODO
        };
    }
}
#[inline]
pub extern "C" fn gfxGetInstanceProcAddr(
    _instance: VkInstance,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    let name = unsafe { CStr::from_ptr(pName) };
    let name = match name.to_str() {
        Ok(name) => name,
        Err(_) => return None,
    };

    let device_addr = gfxGetDeviceProcAddr(DispatchHandle::null(), pName);
    if device_addr.is_some() {
        return device_addr;
    }

    proc_addr!{ name,
        vkCreateInstance, PFN_vkCreateInstance => gfxCreateInstance,
        vkDestroyInstance, PFN_vkDestroyInstance => gfxDestroyInstance,
        vkCreateDevice, PFN_vkCreateDevice => gfxCreateDevice,
        vkGetDeviceProcAddr, PFN_vkGetDeviceProcAddr => gfxGetDeviceProcAddr,

        vkEnumeratePhysicalDevices, PFN_vkEnumeratePhysicalDevices => gfxEnumeratePhysicalDevices,
        vkEnumerateInstanceLayerProperties, PFN_vkEnumerateInstanceLayerProperties => gfxEnumerateInstanceLayerProperties,
        vkEnumerateInstanceExtensionProperties, PFN_vkEnumerateInstanceExtensionProperties => gfxEnumerateInstanceExtensionProperties,
        vkEnumerateDeviceExtensionProperties, PFN_vkEnumerateDeviceExtensionProperties => gfxEnumerateDeviceExtensionProperties,
        vkEnumerateDeviceLayerProperties, PFN_vkEnumerateDeviceLayerProperties => gfxEnumerateDeviceLayerProperties,

        vkGetPhysicalDeviceFeatures, PFN_vkGetPhysicalDeviceFeatures => gfxGetPhysicalDeviceFeatures,
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

        vkCreateWin32SurfaceKHR, PFN_vkCreateWin32SurfaceKHR => gfxCreateWin32SurfaceKHR,
        vkCreateMacOSSurfaceMVK, PFN_vkCreateMacOSSurfaceMVK => gfxCreateMacOSSurfaceMVK,

        vkDestroySurfaceKHR, PFN_vkDestroySurfaceKHR => gfxDestroySurfaceKHR,
    }
}

#[inline]
pub extern "C" fn gfxGetDeviceProcAddr(
    device: VkDevice,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    let name = unsafe { CStr::from_ptr(pName) };
    let name = match name.to_str() {
        Ok(name) => name,
        Err(_) => return None,
    };

    // Requesting the function pointer to an extensions which is available but not
    // enabled with an valid device requires returning NULL.
    if let Some(device) = device.as_ref() {
        match name {
            "vkCreateSwapchainKHR"
            | "vkDestroySwapchainKHR"
            | "vkGetSwapchainImagesKHR"
            | "vkAcquireNextImageKHR"
            | "vkQueuePresentKHR" => {
                let swapchain_extension_enabled = device
                    .enabled_extensions
                    .iter()
                    .any(|e| e == DEVICE_EXTENSION_NAME_VK_KHR_SWAPCHAIN);
                if !swapchain_extension_enabled {
                    return None;
                }
            }
            _ => {}
        }
    }

    proc_addr!{ name,
        vkGetDeviceProcAddr, PFN_vkGetDeviceProcAddr => gfxGetDeviceProcAddr,
        vkDestroyDevice, PFN_vkDestroyDevice => gfxDestroyDevice,
        vkGetDeviceMemoryCommitment, PFN_vkGetDeviceMemoryCommitment => gfxGetDeviceMemoryCommitment,

        vkCreateSwapchainKHR, PFN_vkCreateSwapchainKHR => gfxCreateSwapchainKHR,
        vkDestroySwapchainKHR, PFN_vkDestroySwapchainKHR => gfxDestroySwapchainKHR,
        vkGetSwapchainImagesKHR, PFN_vkGetSwapchainImagesKHR => gfxGetSwapchainImagesKHR,
        vkAcquireNextImageKHR, PFN_vkAcquireNextImageKHR => gfxAcquireNextImageKHR,
        vkQueuePresentKHR, PFN_vkQueuePresentKHR => gfxQueuePresentKHR,

        vkCreateSampler, PFN_vkCreateSampler => gfxCreateSampler,
        vkDestroySampler, PFN_vkDestroySampler => gfxDestroySampler,
        vkCreateShaderModule, PFN_vkCreateShaderModule => gfxCreateShaderModule,
        vkDestroyShaderModule, PFN_vkDestroyShaderModule => gfxDestroyShaderModule,
        vkGetDeviceQueue, PFN_vkGetDeviceQueue => gfxGetDeviceQueue,

        vkAllocateMemory, PFN_vkAllocateMemory => gfxAllocateMemory,
        vkFreeMemory, PFN_vkFreeMemory => gfxFreeMemory,
        vkMapMemory, PFN_vkMapMemory => gfxMapMemory,
        vkUnmapMemory, PFN_vkUnmapMemory => gfxUnmapMemory,
        vkFlushMappedMemoryRanges, PFN_vkFlushMappedMemoryRanges => gfxFlushMappedMemoryRanges,
        vkInvalidateMappedMemoryRanges, PFN_vkInvalidateMappedMemoryRanges => gfxInvalidateMappedMemoryRanges,

        vkCreateBuffer, PFN_vkCreateBuffer => gfxCreateBuffer,
        vkDestroyBuffer, PFN_vkDestroyBuffer => gfxDestroyBuffer,
        vkGetBufferMemoryRequirements, PFN_vkGetBufferMemoryRequirements => gfxGetBufferMemoryRequirements,
        vkBindBufferMemory, PFN_vkBindBufferMemory => gfxBindBufferMemory,
        vkCreateBufferView, PFN_vkCreateBufferView => gfxCreateBufferView,
        vkDestroyBufferView, PFN_vkDestroyBufferView => gfxDestroyBufferView,

        vkCreateImage, PFN_vkCreateImage => gfxCreateImage,
        vkDestroyImage, PFN_vkDestroyImage => gfxDestroyImage,
        vkGetImageMemoryRequirements, PFN_vkGetImageMemoryRequirements => gfxGetImageMemoryRequirements,
        vkGetImageSparseMemoryRequirements, PFN_vkGetImageSparseMemoryRequirements => gfxGetImageSparseMemoryRequirements,
        vkBindImageMemory, PFN_vkBindImageMemory => gfxBindImageMemory,
        vkCreateImageView, PFN_vkCreateImageView => gfxCreateImageView,
        vkDestroyImageView, PFN_vkDestroyImageView => gfxDestroyImageView,
        vkGetImageSubresourceLayout, PFN_vkGetImageSubresourceLayout => gfxGetImageSubresourceLayout,

        vkCreateRenderPass, PFN_vkCreateRenderPass => gfxCreateRenderPass,
        vkDestroyRenderPass, PFN_vkDestroyRenderPass => gfxDestroyRenderPass,
        vkCreateFramebuffer, PFN_vkCreateFramebuffer => gfxCreateFramebuffer,
        vkDestroyFramebuffer, PFN_vkDestroyFramebuffer => gfxDestroyFramebuffer,
        vkGetRenderAreaGranularity, PFN_vkGetRenderAreaGranularity => gfxGetRenderAreaGranularity,

        vkCreatePipelineLayout, PFN_vkCreatePipelineLayout => gfxCreatePipelineLayout,
        vkDestroyPipelineLayout, PFN_vkDestroyPipelineLayout => gfxDestroyPipelineLayout,
        vkCreateGraphicsPipelines, PFN_vkCreateGraphicsPipelines => gfxCreateGraphicsPipelines,
        vkCreateComputePipelines, PFN_vkCreateComputePipelines => gfxCreateComputePipelines,
        vkDestroyPipeline, PFN_vkDestroyPipeline => gfxDestroyPipeline,
        vkCreatePipelineCache, PFN_vkCreatePipelineCache => gfxCreatePipelineCache,
        vkDestroyPipelineCache, PFN_vkDestroyPipelineCache => gfxDestroyPipelineCache,
        vkGetPipelineCacheData, PFN_vkGetPipelineCacheData => gfxGetPipelineCacheData,
        vkMergePipelineCaches, PFN_vkMergePipelineCaches => gfxMergePipelineCaches,

        vkCreateCommandPool, PFN_vkCreateCommandPool => gfxCreateCommandPool,
        vkDestroyCommandPool, PFN_vkDestroyCommandPool => gfxDestroyCommandPool,
        vkResetCommandPool, PFN_vkResetCommandPool => gfxResetCommandPool,
        vkAllocateCommandBuffers, PFN_vkAllocateCommandBuffers => gfxAllocateCommandBuffers,
        vkFreeCommandBuffers, PFN_vkFreeCommandBuffers => gfxFreeCommandBuffers,
        vkBeginCommandBuffer, PFN_vkBeginCommandBuffer => gfxBeginCommandBuffer,
        vkEndCommandBuffer, PFN_vkEndCommandBuffer => gfxEndCommandBuffer,
        vkResetCommandBuffer, PFN_vkResetCommandBuffer => gfxResetCommandBuffer,

        vkCreateDescriptorSetLayout, PFN_vkCreateDescriptorSetLayout => gfxCreateDescriptorSetLayout,
        vkDestroyDescriptorSetLayout, PFN_vkDestroyDescriptorSetLayout => gfxDestroyDescriptorSetLayout,
        vkCreateDescriptorPool, PFN_vkCreateDescriptorPool => gfxCreateDescriptorPool,
        vkDestroyDescriptorPool, PFN_vkDestroyDescriptorPool => gfxDestroyDescriptorPool,
        vkResetDescriptorPool, PFN_vkResetDescriptorPool => gfxResetDescriptorPool,
        vkAllocateDescriptorSets, PFN_vkAllocateDescriptorSets => gfxAllocateDescriptorSets,
        vkFreeDescriptorSets, PFN_vkFreeDescriptorSets => gfxFreeDescriptorSets,
        vkUpdateDescriptorSets, PFN_vkUpdateDescriptorSets => gfxUpdateDescriptorSets,

        vkCreateFence, PFN_vkCreateFence => gfxCreateFence,
        vkDestroyFence, PFN_vkDestroyFence => gfxDestroyFence,
        vkWaitForFences, PFN_vkWaitForFences => gfxWaitForFences,
        vkResetFences, PFN_vkResetFences => gfxResetFences,
        vkGetFenceStatus, PFN_vkGetFenceStatus => gfxGetFenceStatus,

        vkCreateSemaphore, PFN_vkCreateSemaphore => gfxCreateSemaphore,
        vkDestroySemaphore, PFN_vkDestroySemaphore => gfxDestroySemaphore,

        vkCreateEvent, PFN_vkCreateEvent => gfxCreateEvent,
        vkDestroyEvent, PFN_vkDestroyEvent => gfxDestroyEvent,
        vkGetEventStatus, PFN_vkGetEventStatus => gfxGetEventStatus,
        vkSetEvent, PFN_vkSetEvent => gfxSetEvent,
        vkResetEvent, PFN_vkResetEvent => gfxResetEvent,

        vkQueueSubmit, PFN_vkQueueSubmit => gfxQueueSubmit,
        vkQueueBindSparse, PFN_vkQueueBindSparse => gfxQueueBindSparse,
        vkQueueWaitIdle, PFN_vkQueueWaitIdle => gfxQueueWaitIdle,
        vkDeviceWaitIdle, PFN_vkDeviceWaitIdle => gfxDeviceWaitIdle,

        vkCreateQueryPool, PFN_vkCreateQueryPool => gfxCreateQueryPool,
        vkDestroyQueryPool, PFN_vkDestroyQueryPool => gfxDestroyQueryPool,
        vkGetQueryPoolResults, PFN_vkGetQueryPoolResults => gfxGetQueryPoolResults,

        vkCmdBindPipeline, PFN_vkCmdBindPipeline => gfxCmdBindPipeline,
        vkCmdSetViewport, PFN_vkCmdSetViewport => gfxCmdSetViewport,
        vkCmdSetScissor, PFN_vkCmdSetScissor => gfxCmdSetScissor,
        vkCmdSetLineWidth, PFN_vkCmdSetLineWidth => gfxCmdSetLineWidth,
        vkCmdSetDepthBias, PFN_vkCmdSetDepthBias => gfxCmdSetDepthBias,
        vkCmdSetBlendConstants, PFN_vkCmdSetBlendConstants => gfxCmdSetBlendConstants,
        vkCmdSetDepthBounds, PFN_vkCmdSetDepthBounds => gfxCmdSetDepthBounds,
        vkCmdSetStencilCompareMask, PFN_vkCmdSetStencilCompareMask => gfxCmdSetStencilCompareMask,
        vkCmdSetStencilWriteMask, PFN_vkCmdSetStencilWriteMask => gfxCmdSetStencilWriteMask,
        vkCmdSetStencilReference, PFN_vkCmdSetStencilReference => gfxCmdSetStencilReference,
        vkCmdBindDescriptorSets, PFN_vkCmdBindDescriptorSets => gfxCmdBindDescriptorSets,
        vkCmdBindIndexBuffer, PFN_vkCmdBindIndexBuffer => gfxCmdBindIndexBuffer,
        vkCmdBindVertexBuffers, PFN_vkCmdBindVertexBuffers => gfxCmdBindVertexBuffers,
        vkCmdDraw, PFN_vkCmdDraw => gfxCmdDraw,
        vkCmdDrawIndexed, PFN_vkCmdDrawIndexed => gfxCmdDrawIndexed,
        vkCmdDrawIndirect, PFN_vkCmdDrawIndirect => gfxCmdDrawIndirect,
        vkCmdDrawIndexedIndirect, PFN_vkCmdDrawIndexedIndirect => gfxCmdDrawIndexedIndirect,
        vkCmdDispatch, PFN_vkCmdDispatch => gfxCmdDispatch,
        vkCmdDispatchIndirect, PFN_vkCmdDispatchIndirect => gfxCmdDispatchIndirect,
        vkCmdCopyBuffer, PFN_vkCmdCopyBuffer => gfxCmdCopyBuffer,
        vkCmdCopyImage, PFN_vkCmdCopyImage => gfxCmdCopyImage,
        vkCmdBlitImage, PFN_vkCmdBlitImage => gfxCmdBlitImage,
        vkCmdCopyBufferToImage, PFN_vkCmdCopyBufferToImage => gfxCmdCopyBufferToImage,
        vkCmdCopyImageToBuffer, PFN_vkCmdCopyImageToBuffer => gfxCmdCopyImageToBuffer,
        vkCmdUpdateBuffer, PFN_vkCmdUpdateBuffer => gfxCmdUpdateBuffer,
        vkCmdFillBuffer, PFN_vkCmdFillBuffer => gfxCmdFillBuffer,
        vkCmdClearColorImage, PFN_vkCmdClearColorImage => gfxCmdClearColorImage,
        vkCmdClearDepthStencilImage, PFN_vkCmdClearDepthStencilImage => gfxCmdClearDepthStencilImage,
        vkCmdClearAttachments, PFN_vkCmdClearAttachments => gfxCmdClearAttachments,
        vkCmdResolveImage, PFN_vkCmdResolveImage => gfxCmdResolveImage,
        vkCmdSetEvent, PFN_vkCmdSetEvent => gfxCmdSetEvent,
        vkCmdResetEvent, PFN_vkCmdResetEvent => gfxCmdResetEvent,
        vkCmdWaitEvents, PFN_vkCmdWaitEvents => gfxCmdWaitEvents,
        vkCmdBeginQuery, PFN_vkCmdBeginQuery => gfxCmdBeginQuery,
        vkCmdEndQuery, PFN_vkCmdEndQuery => gfxCmdEndQuery,
        vkCmdResetQueryPool, PFN_vkCmdResetQueryPool => gfxCmdResetQueryPool,
        vkCmdWriteTimestamp, PFN_vkCmdWriteTimestamp => gfxCmdWriteTimestamp,
        vkCmdCopyQueryPoolResults, PFN_vkCmdCopyQueryPoolResults => gfxCmdCopyQueryPoolResults,
        vkCmdPushConstants, PFN_vkCmdPushConstants => gfxCmdPushConstants,
        vkCmdNextSubpass, PFN_vkCmdNextSubpass => gfxCmdNextSubpass,
        vkCmdExecuteCommands, PFN_vkCmdExecuteCommands => gfxCmdExecuteCommands,
        vkCmdPipelineBarrier, PFN_vkCmdPipelineBarrier => gfxCmdPipelineBarrier,
        vkCmdBeginRenderPass, PFN_vkCmdBeginRenderPass => gfxCmdBeginRenderPass,
        vkCmdEndRenderPass, PFN_vkCmdEndRenderPass => gfxCmdEndRenderPass,
    }
}

#[inline]
pub extern "C" fn gfxCreateDevice(
    adapter: VkPhysicalDevice,
    pCreateInfo: *const VkDeviceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pDevice: *mut VkDevice,
) -> VkResult {
    let dev_info = unsafe { &*pCreateInfo };
    let queue_infos = unsafe {
        slice::from_raw_parts(
            dev_info.pQueueCreateInfos,
            dev_info.queueCreateInfoCount as _,
        )
    };
    let max_queue_count = queue_infos
        .iter()
        .map(|info| info.queueCount as usize)
        .max()
        .unwrap_or(0);
    let priorities = vec![1.0; max_queue_count];
    let request_infos = queue_infos
        .iter()
        .map(|info| {
            let family = &adapter.queue_families[info.queueFamilyIndex as usize];
            (family, &priorities[.. info.queueCount as usize])
        })
        .collect::<Vec<_>>();

    if let Some(ef) = unsafe { dev_info.pEnabledFeatures.as_ref() } {
        let supported = adapter.physical_device.features();
        if (ef.robustBufferAccess != 0 && !supported.contains(Features::ROBUST_BUFFER_ACCESS)) ||
            (ef.fullDrawIndexUint32 != 0 && !supported.contains(Features::FULL_DRAW_INDEX_U32)) ||
            (ef.imageCubeArray != 0 && !supported.contains(Features::IMAGE_CUBE_ARRAY)) ||
            (ef.independentBlend != 0 && !supported.contains(Features::INDEPENDENT_BLENDING)) ||
            (ef.geometryShader != 0 && !supported.contains(Features::GEOMETRY_SHADER)) ||
            (ef.tessellationShader != 0 && !supported.contains(Features::TESSELLATION_SHADER)) ||
            (ef.sampleRateShading != 0 && !supported.contains(Features::SAMPLE_RATE_SHADING)) ||
            (ef.dualSrcBlend != 0 && !supported.contains(Features::DUAL_SRC_BLENDING)) ||
            (ef.logicOp != 0 && !supported.contains(Features::LOGIC_OP)) ||
            (ef.multiDrawIndirect != 0 && !supported.contains(Features::MULTI_DRAW_INDIRECT)) ||
            (ef.drawIndirectFirstInstance != 0 && !supported.contains(Features::DRAW_INDIRECT_FIRST_INSTANCE)) ||
            (ef.depthClamp != 0 && !supported.contains(Features::DEPTH_CLAMP)) ||
            (ef.depthBiasClamp != 0 && !supported.contains(Features::DEPTH_BIAS_CLAMP)) ||
            (ef.fillModeNonSolid != 0 && !supported.contains(Features::NON_FILL_POLYGON_MODE)) ||
            (ef.depthBounds != 0 && !supported.contains(Features::DEPTH_BOUNDS)) ||
            (ef.wideLines != 0 && !supported.contains(Features::LINE_WIDTH)) ||
            (ef.largePoints != 0 && !supported.contains(Features::POINT_SIZE)) ||
            (ef.alphaToOne != 0 && !supported.contains(Features::ALPHA_TO_ONE)) ||
            (ef.multiViewport != 0 && !supported.contains(Features::MULTI_VIEWPORTS)) ||
            (ef.samplerAnisotropy != 0 && !supported.contains(Features::SAMPLER_ANISOTROPY)) ||
            (ef.textureCompressionETC2 != 0 && !supported.contains(Features::FORMAT_ETC2)) ||
            (ef.textureCompressionASTC_LDR != 0 && !supported.contains(Features::FORMAT_ASTC_LDR)) ||
            (ef.textureCompressionBC != 0 && !supported.contains(Features::FORMAT_BC)) ||
            (ef.occlusionQueryPrecise != 0 && !supported.contains(Features::PRECISE_OCCLUSION_QUERY)) ||
            (ef.pipelineStatisticsQuery != 0 && !supported.contains(Features::PIPELINE_STATISTICS_QUERY)) ||
            (ef.vertexPipelineStoresAndAtomics != 0 && !supported.contains(Features::VERTEX_STORES_AND_ATOMICS)) ||
            (ef.fragmentStoresAndAtomics != 0 && !supported.contains(Features::FRAGMENT_STORES_AND_ATOMICS)) ||
            (ef.shaderTessellationAndGeometryPointSize != 0 && !supported.contains(Features::SHADER_TESSELLATION_AND_GEOMETRY_POINT_SIZE)) ||
            (ef.shaderImageGatherExtended != 0 && !supported.contains(Features::SHADER_IMAGE_GATHER_EXTENDED)) ||
            (ef.shaderStorageImageExtendedFormats != 0 && !supported.contains(Features::SHADER_STORAGE_IMAGE_EXTENDED_FORMATS)) ||
            (ef.shaderStorageImageMultisample != 0 && !supported.contains(Features::SHADER_STORAGE_IMAGE_MULTISAMPLE)) ||
            (ef.shaderStorageImageReadWithoutFormat != 0 && !supported.contains(Features::SHADER_STORAGE_IMAGE_READ_WITHOUT_FORMAT)) ||
            (ef.shaderStorageImageWriteWithoutFormat != 0 && !supported.contains(Features::SHADER_STORAGE_IMAGE_WRITE_WITHOUT_FORMAT)) ||
            (ef.shaderUniformBufferArrayDynamicIndexing != 0 && !supported.contains(Features::SHADER_UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING)) ||
            (ef.shaderSampledImageArrayDynamicIndexing != 0 && !supported.contains(Features::SHADER_SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING)) ||
            (ef.shaderStorageBufferArrayDynamicIndexing != 0 && !supported.contains(Features::SHADER_STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING)) ||
            (ef.shaderStorageImageArrayDynamicIndexing != 0 && !supported.contains(Features::SHADER_STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING)) ||
            (ef.shaderClipDistance != 0 && !supported.contains(Features::SHADER_CLIP_DISTANCE)) ||
            (ef.shaderCullDistance != 0 && !supported.contains(Features::SHADER_CULL_DISTANCE)) ||
            (ef.shaderFloat64 != 0 && !supported.contains(Features::SHADER_FLOAT64)) ||
            (ef.shaderInt64 != 0 && !supported.contains(Features::SHADER_INT64)) ||
            (ef.shaderInt16 != 0 && !supported.contains(Features::SHADER_INT16)) ||
            (ef.shaderResourceResidency != 0 && !supported.contains(Features::SHADER_RESOURCE_RESIDENCY)) ||
            (ef.shaderResourceMinLod != 0 && !supported.contains(Features::SHADER_RESOURCE_MIN_LOD)) ||
            (ef.sparseBinding != 0 && !supported.contains(Features::SPARSE_BINDING)) ||
            (ef.sparseResidencyBuffer != 0 && !supported.contains(Features::SPARSE_RESIDENCY_BUFFER)) ||
            (ef.sparseResidencyImage2D != 0 && !supported.contains(Features::SHADER_RESIDENCY_IMAGE_2D)) ||
            (ef.sparseResidencyImage3D != 0 && !supported.contains(Features::SHADER_RESIDENSY_IMAGE_3D)) ||
            (ef.sparseResidency2Samples != 0 && !supported.contains(Features::SPARSE_RESIDENCY_2_SAMPLES)) ||
            (ef.sparseResidency4Samples != 0 && !supported.contains(Features::SPARSE_RESIDENCY_4_SAMPLES)) ||
            (ef.sparseResidency8Samples != 0 && !supported.contains(Features::SPARSE_RESIDENCY_8_SAMPLES)) ||
            (ef.sparseResidency16Samples != 0 && !supported.contains(Features::SPARSE_RESIDENCY_16_SAMPLES)) ||
            (ef.sparseResidencyAliased != 0 && !supported.contains(Features::SPARSE_RESIDENCY_ALIASED)) ||
            (ef.variableMultisampleRate != 0 && !supported.contains(Features::VARIABLE_MULTISAMPLE_RATE)) ||
            (ef.inheritedQueries != 0 && !supported.contains(Features::INHERITED_QUERIES)) {
            return VkResult::VK_ERROR_FEATURE_NOT_PRESENT;
        }
    }

    #[cfg(feature = "renderdoc")]
    let mut renderdoc = {
        use renderdoc::RenderDoc;
        RenderDoc::new().expect("Failed to init renderdoc")
    };

    let gpu = adapter.physical_device.open(&request_infos);

    match gpu {
        Ok(mut gpu) => {
            let queues = queue_infos
                .iter()
                .map(|info| {
                    let id = queue::QueueFamilyId(info.queueFamilyIndex as usize);
                    let group = gpu.queues.take_raw(id).unwrap();
                    let queues = group
                        .into_iter()
                        .map(DispatchHandle::new)
                        .collect();

                    (info.queueFamilyIndex, queues)
                })
                .collect();

            #[cfg(feature = "renderdoc")]
            let rd_device = {
                use renderdoc::api::RenderDocV100;

                let rd_device = unsafe { gpu.device.as_raw() };
                renderdoc.start_frame_capture(rd_device, ::std::ptr::null());
                rd_device
            };

            let enabled_extensions = if dev_info.enabledExtensionCount == 0 {
                Vec::new()
            } else {
                let extensions = unsafe {
                        slice::from_raw_parts(dev_info.ppEnabledExtensionNames, dev_info.enabledExtensionCount as _)
                            .iter()
                            .map(|raw| CStr::from_ptr(*raw)
                                .to_str()
                                .expect("Invalid extension name")
                                .to_owned()
                            )
                            .collect::<Vec<_>>()
                    };
                for extension in &extensions {
                    if !DEVICE_EXTENSION_NAMES.contains(&extension.as_ref()) {
                        return VkResult::VK_ERROR_EXTENSION_NOT_PRESENT;
                    }
                }
                extensions
            };

            let gpu = Gpu {
                device: gpu.device,
                queues,
                enabled_extensions,
                #[cfg(feature = "renderdoc")]
                renderdoc,
                #[cfg(feature = "renderdoc")]
                capturing: rd_device as *mut _,
            };

            unsafe {
                *pDevice = DispatchHandle::new(gpu);
            }

            VkResult::VK_SUCCESS
        }
        Err(err) => conv::map_err_device_creation(err),
    }
}

#[inline]
pub extern "C" fn gfxDestroyDevice(gpu: VkDevice, _pAllocator: *const VkAllocationCallbacks) {
    // release all the owned command queues
    if let Some(mut d) = gpu.unbox() {
        #[cfg(feature = "renderdoc")]
        {
            use renderdoc::api::RenderDocV100;
            let device = gpu.capturing as *mut c_void;
            d.renderdoc.end_frame_capture(device as *mut _, ptr::null());
        }

        for (_, family) in d.queues.drain() {
            for queue in family {
                let _ = queue.unbox();
            }
        }
    }
}

// TODO: Avoid redefining these somehow
static INSTANCE_EXTENSION_NAME_VK_KHR_SURFACE: &str = "VK_KHR_surface";
#[cfg(target_os="windows")]
static INSTANCE_EXTENSION_NAME_VK_KHR_WIN32_SURFACE: &str = "VK_KHR_win32_surface";
#[cfg(target_os="macos")]
static INSTANCE_EXTENSION_NAME_VK_MACOS_SURFACE: &str = "VK_MVK_macos_surface";
static DEVICE_EXTENSION_NAME_VK_KHR_SWAPCHAIN: &str = "VK_KHR_swapchain";

lazy_static! {
    // TODO: Request from backend
    static ref INSTANCE_EXTENSION_NAMES: Vec<&'static str> = {
        vec![
            INSTANCE_EXTENSION_NAME_VK_KHR_SURFACE,
            #[cfg(target_os="windows")]
            INSTANCE_EXTENSION_NAME_VK_KHR_WIN32_SURFACE,
            #[cfg(target_os="macos")]
            INSTANCE_EXTENSION_NAME_VK_MACOS_SURFACE,
        ]
    };

    static ref INSTANCE_EXTENSIONS: Vec<VkExtensionProperties> = {
        let mut extensions = [
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_SURFACE_EXTENSION_NAME
                specVersion: VK_KHR_SURFACE_SPEC_VERSION,
            },
            #[cfg(target_os="windows")]
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_WIN32_SURFACE_EXTENSION_NAME
                specVersion: VK_KHR_WIN32_SURFACE_SPEC_VERSION,
            },
            #[cfg(target_os="macos")]
            VkExtensionProperties {
                extensionName: [0; 256], // VK_MVK_MACOS_SURFACE_EXTENSION_NAME
                specVersion: VK_MVK_MACOS_SURFACE_SPEC_VERSION,
            },
        ];

        extensions[0]
            .extensionName[..VK_KHR_SURFACE_EXTENSION_NAME.len()]
            .copy_from_slice(unsafe {
                mem::transmute(VK_KHR_SURFACE_EXTENSION_NAME as &[u8])
            });
        #[cfg(target_os="windows")]
        extensions[1]
            .extensionName[..VK_KHR_WIN32_SURFACE_EXTENSION_NAME.len()]
            .copy_from_slice(unsafe {
                mem::transmute(VK_KHR_WIN32_SURFACE_EXTENSION_NAME as &[u8])
            });
        #[cfg(target_os = "macos")]
        extensions[1]
            .extensionName[..VK_MVK_MACOS_SURFACE_EXTENSION_NAME.len()]
            .copy_from_slice(unsafe {
                mem::transmute(VK_MVK_MACOS_SURFACE_EXTENSION_NAME as &[u8])
            });

        extensions.to_vec()
    };

    static ref DEVICE_EXTENSION_NAMES: Vec<&'static str> = {
        vec![
            DEVICE_EXTENSION_NAME_VK_KHR_SWAPCHAIN,
        ]
    };

    static ref DEVICE_EXTENSIONS: Vec<VkExtensionProperties> = {
        let mut extensions = [
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_SWAPCHAIN_EXTENSION_NAME
                specVersion: VK_KHR_SWAPCHAIN_SPEC_VERSION,
            },
        ];

        extensions[0]
            .extensionName[..VK_KHR_SWAPCHAIN_EXTENSION_NAME.len()]
            .copy_from_slice(unsafe {
                mem::transmute(VK_KHR_SWAPCHAIN_EXTENSION_NAME as &[u8])
            });

        extensions.to_vec()
    };
}

#[inline]
pub extern "C" fn gfxEnumerateInstanceExtensionProperties(
    _pLayerName: *const ::std::os::raw::c_char,
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
        let properties =
            unsafe { slice::from_raw_parts_mut(pProperties, *property_count as usize) };
        for i in 0..*property_count as usize {
            properties[i] = INSTANCE_EXTENSIONS[i];
        }

        if *property_count < num_extensions {
            return VkResult::VK_INCOMPLETE;
        }
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxEnumerateDeviceExtensionProperties(
    _physicalDevice: VkPhysicalDevice,
    _pLayerName: *const ::std::os::raw::c_char,
    pPropertyCount: *mut u32,
    pProperties: *mut VkExtensionProperties,
) -> VkResult {
    let property_count = unsafe { &mut *pPropertyCount };
    let num_extensions = DEVICE_EXTENSIONS.len() as u32;

    if pProperties.is_null() {
        *property_count = num_extensions;
    } else {
        if *property_count > num_extensions {
            *property_count = num_extensions;
        }
        let properties =
            unsafe { slice::from_raw_parts_mut(pProperties, *property_count as usize) };
        for i in 0..*property_count as usize {
            properties[i] = DEVICE_EXTENSIONS[i];
        }

        if *property_count < num_extensions {
            return VkResult::VK_INCOMPLETE;
        }
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxEnumerateInstanceLayerProperties(
    pPropertyCount: *mut u32,
    _pProperties: *mut VkLayerProperties,
) -> VkResult {
    warn!("TODO: gfxEnumerateInstanceLayerProperties");
    unsafe { *pPropertyCount = 0; }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxEnumerateDeviceLayerProperties(
    _physicalDevice: VkPhysicalDevice,
    pPropertyCount: *mut u32,
    _pProperties: *mut VkLayerProperties,
) -> VkResult {
    warn!("TODO: gfxEnumerateDeviceLayerProperties");
    unsafe { *pPropertyCount = 0; }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxGetDeviceQueue(
    gpu: VkDevice,
    queueFamilyIndex: u32,
    queueIndex: u32,
    pQueue: *mut VkQueue,
) {
    let queue = gpu.queues
        .get(&queueFamilyIndex)
        .unwrap()[queueIndex as usize];
    unsafe {
        *pQueue = queue;
    }
}
#[inline]
pub extern "C" fn gfxQueueSubmit(
    mut queue: VkQueue,
    submitCount: u32,
    pSubmits: *const VkSubmitInfo,
    fence: VkFence,
) -> VkResult {
    assert_eq!(submitCount, 1); // TODO;

    let submission = unsafe { *pSubmits };
    let cmd_slice = unsafe {
        slice::from_raw_parts(submission.pCommandBuffers, submission.commandBufferCount as _)
    };
    let wait_semaphores = unsafe {
        let semaphores = slice::from_raw_parts(submission.pWaitSemaphores, submission.waitSemaphoreCount as _);
        let stages = slice::from_raw_parts(submission.pWaitDstStageMask, submission.waitSemaphoreCount as _);

        stages.into_iter()
            .zip(semaphores.into_iter())
            .map(|(stage, semaphore)| (&**semaphore, conv::map_pipeline_stage_flags(*stage)))
            .collect::<Vec<_>>()
    };
    let signal_semaphores = unsafe {
        slice::from_raw_parts(submission.pSignalSemaphores, submission.signalSemaphoreCount as _)
            .into_iter()
            .map(|semaphore| &**semaphore)
            .collect::<Vec<_>>()
    };

    let submission = hal::queue::RawSubmission {
        cmd_buffers: cmd_slice.iter().cloned(),
        wait_semaphores: &wait_semaphores,
        signal_semaphores: &signal_semaphores,
    };

    unsafe { queue.submit_raw(submission, fence.as_ref()); }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxQueueWaitIdle(queue: VkQueue) -> VkResult {
    let _ = queue.wait_idle();
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDeviceWaitIdle(gpu: VkDevice) -> VkResult {
    let _ = gpu.device.wait_idle();
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxAllocateMemory(
    gpu: VkDevice,
    pAllocateInfo: *const VkMemoryAllocateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pMemory: *mut VkDeviceMemory,
) -> VkResult {
    let info = unsafe { &*pAllocateInfo };
    let memory = gpu.device
        .allocate_memory(
            hal::MemoryTypeId(info.memoryTypeIndex as _),
            info.allocationSize,
        )
        .unwrap(); // TODO:

    unsafe {
        *pMemory = Handle::new(memory);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxFreeMemory(
    gpu: VkDevice,
    memory: VkDeviceMemory,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(mem) = memory.unbox() {
        gpu.device.free_memory(mem);
    }
}
#[inline]
pub extern "C" fn gfxMapMemory(
    gpu: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    _flags: VkMemoryMapFlags,
    ppData: *mut *mut ::std::os::raw::c_void,
) -> VkResult {
    let range = if size == VK_WHOLE_SIZE as VkDeviceSize {
        (Some(offset), None)
    } else {
        (Some(offset), Some(offset + size))
    };

    unsafe {
        *ppData = gpu.device
            .map_memory(&memory, range)
            .unwrap() as *mut _; // TODO
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxUnmapMemory(gpu: VkDevice, memory: VkDeviceMemory) {
    gpu.device.unmap_memory(&memory);
}
#[inline]
pub extern "C" fn gfxFlushMappedMemoryRanges(
    gpu: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    let ranges = unsafe {
            slice::from_raw_parts(pMemoryRanges, memoryRangeCount as _)
        }
        .iter()
        .map(|r| {
            let range = if r.size == VK_WHOLE_SIZE as VkDeviceSize {
                (Some(r.offset), None)
            } else {
                (Some(r.offset), Some(r.offset + r.size))
            };

            (&*r.memory, range)
        });

    gpu.device.flush_mapped_memory_ranges(ranges);
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxInvalidateMappedMemoryRanges(
    gpu: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    let ranges = unsafe {
            slice::from_raw_parts(pMemoryRanges, memoryRangeCount as _)
        }
        .iter()
        .map(|r| {
            let range = if r.size == VK_WHOLE_SIZE as VkDeviceSize {
                (Some(r.offset), None)
            } else {
                (Some(r.offset), Some(r.offset + r.size))
            };

            (&*r.memory, range)
        });

    gpu.device.invalidate_mapped_memory_ranges(ranges);
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxGetDeviceMemoryCommitment(
    _device: VkDevice,
    _memory: VkDeviceMemory,
    _pCommittedMemoryInBytes: *mut VkDeviceSize,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxBindBufferMemory(
    gpu: VkDevice,
    mut buffer: VkBuffer,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    let new = match mem::replace(&mut *buffer, unsafe { mem::zeroed() }) {
        Buffer::Buffer(_) => panic!("A non-sparse buffer can only be bound once!"),
        Buffer::Unbound(unbound) => {
            Buffer::Buffer(
                gpu.device
                    .bind_buffer_memory(&memory, memoryOffset, unbound)
                    .unwrap() // TODO
            )
        }
    };

    // We need to move the value out of the Handle here,
    // and then put something else back in.
    let temp = mem::replace(&mut *buffer, new);
    mem::forget(temp);

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxBindImageMemory(
    gpu: VkDevice,
    mut image: VkImage,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    let new = match mem::replace(&mut *image, unsafe { mem::zeroed() }) {
        Image::Image { .. } => panic!("An non-sparse image can only be bound once!"),
        Image::Unbound { raw, mip_levels, array_layers } => {
            Image::Image {
                raw: gpu.device.bind_image_memory(&memory, memoryOffset, raw).unwrap(), // TODO
                mip_levels,
                array_layers,
            }
        }
    };

    // We need to move the value out of the Handle here,
    // and then put something else back in.
    let temp = mem::replace(&mut *image, new);
    mem::forget(temp);

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxGetBufferMemoryRequirements(
    gpu: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    let req = match *buffer {
        Buffer::Buffer(_) => unimplemented!(),
        Buffer::Unbound(ref buffer) => gpu.device.get_buffer_requirements(buffer),
    };

    *unsafe { &mut *pMemoryRequirements } = VkMemoryRequirements {
        size: req.size,
        alignment: req.alignment,
        memoryTypeBits: req.type_mask as _,
    };
}
#[inline]
pub extern "C" fn gfxGetImageMemoryRequirements(
    gpu: VkDevice,
    image: VkImage,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    let req = match *image {
        Image::Image { .. } => unimplemented!(),
        Image::Unbound { ref raw, .. } => gpu.device.get_image_requirements(raw),
    };

    *unsafe { &mut *pMemoryRequirements } = VkMemoryRequirements {
        size: req.size,
        alignment: req.alignment,
        memoryTypeBits: req.type_mask as _,
    };
}

#[inline]
pub extern "C" fn gfxGetImageSparseMemoryRequirements(
    _device: VkDevice,
    _image: VkImage,
    _pSparseMemoryRequirementCount: *mut u32,
    _pSparseMemoryRequirements: *mut VkSparseImageMemoryRequirements,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceSparseImageFormatProperties(
    _physicalDevice: VkPhysicalDevice,
    _format: VkFormat,
    _type_: VkImageType,
    _samples: VkSampleCountFlagBits,
    _usage: VkImageUsageFlags,
    _tiling: VkImageTiling,
    pPropertyCount: *mut u32,
    _pProperties: *mut VkSparseImageFormatProperties,
) {
    unsafe { *pPropertyCount = 0; } //TODO
}
#[inline]
pub extern "C" fn gfxQueueBindSparse(
    _queue: VkQueue,
    _bindInfoCount: u32,
    _pBindInfo: *const VkBindSparseInfo,
    _fence: VkFence,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateFence(
    gpu: VkDevice,
    pCreateInfo: *const VkFenceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pFence: *mut VkFence,
) -> VkResult {
    let flags = unsafe { (*pCreateInfo).flags };
    let signalled = flags & VkFenceCreateFlagBits::VK_FENCE_CREATE_SIGNALED_BIT as u32 != 0;

    let fence = gpu
        .device
        .create_fence(signalled);

    unsafe {
        *pFence = Handle::new(fence);
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyFence(
    gpu: VkDevice,
    fence: VkFence,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(fence) = fence.unbox() {
        gpu.device.destroy_fence(fence);
    }
}
#[inline]
pub extern "C" fn gfxResetFences(
    gpu: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
) -> VkResult {
    let fence_slice = unsafe {
        slice::from_raw_parts(pFences, fenceCount as _)
    };
    let fences = fence_slice
        .into_iter()
        .map(|fence| &**fence);

    gpu.device.reset_fences(fences);
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxGetFenceStatus(gpu: VkDevice, fence: VkFence) -> VkResult {
    if gpu.device.get_fence_status(&*fence) {
        VkResult::VK_SUCCESS
    } else {
        VkResult::VK_NOT_READY
    }
}
#[inline]
pub extern "C" fn gfxWaitForFences(
    gpu: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
    waitAll: VkBool32,
    timeout: u64,
) -> VkResult {
    let fence_slice = unsafe {
        slice::from_raw_parts(pFences, fenceCount as _)
    };
    let fences = fence_slice
        .into_iter()
        .map(|fence| &**fence);

    let wait_for = match waitAll {
        VK_FALSE => WaitFor::Any,
        _ => WaitFor::All,
    };

    if gpu.device.wait_for_fences(fences, wait_for, timeout as _) {
        VkResult::VK_SUCCESS
    } else {
        VkResult::VK_TIMEOUT
    }
}
#[inline]
pub extern "C" fn gfxCreateSemaphore(
    gpu: VkDevice,
    _pCreateInfo: *const VkSemaphoreCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pSemaphore: *mut VkSemaphore,
) -> VkResult {
    let semaphore = gpu.device
        .create_semaphore();

    unsafe {
        *pSemaphore = Handle::new(semaphore);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroySemaphore(
    gpu: VkDevice,
    semaphore: VkSemaphore,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(sem) = semaphore.unbox() {
        gpu.device.destroy_semaphore(sem);
    }
}
#[inline]
pub extern "C" fn gfxCreateEvent(
    _gpu: VkDevice,
    _pCreateInfo: *const VkEventCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    _pEvent: *mut VkEvent,
) -> VkResult {
    // Vulkan portability doesn't currently support events, but some
    // test cases use them so fail with an obvious error message.
    VkResult::VK_ERROR_DEVICE_LOST
}
#[inline]
pub extern "C" fn gfxDestroyEvent(
    _gpu: VkDevice,
    event: VkEvent,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if event != ptr::null_mut() {
        unimplemented!()
    }
}
#[inline]
pub extern "C" fn gfxGetEventStatus(_gpu: VkDevice, _event: VkEvent) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxSetEvent(_gpu: VkDevice, _event: VkEvent) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxResetEvent(_gpu: VkDevice, _event: VkEvent) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateQueryPool(
    _gpu: VkDevice,
    _pCreateInfo: *const VkQueryPoolCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pQueryPool: *mut VkQueryPool,
) -> VkResult {
    //TODO
    unsafe { *pQueryPool = ptr::null_mut() };
    VkResult::VK_ERROR_DEVICE_LOST
}
#[inline]
pub extern "C" fn gfxDestroyQueryPool(
    _gpu: VkDevice,
    queryPool: VkQueryPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if queryPool != ptr::null_mut() {
        unimplemented!()
    }
}
#[inline]
pub extern "C" fn gfxGetQueryPoolResults(
    _gpu: VkDevice,
    _queryPool: VkQueryPool,
    _firstQuery: u32,
    _queryCount: u32,
    dataSize: usize,
    pData: *mut ::std::os::raw::c_void,
    _stride: VkDeviceSize,
    _flags: VkQueryResultFlags,
) -> VkResult {
    error!("Query pools are not implemented");
    let slice = unsafe {
        slice::from_raw_parts_mut(pData as *mut u8, dataSize)
    };
    for d in slice {
        *d = 0; //TODO
    }
    VkResult::VK_ERROR_DEVICE_LOST
}
#[inline]
pub extern "C" fn gfxCreateBuffer(
    gpu: VkDevice,
    pCreateInfo: *const VkBufferCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pBuffer: *mut VkBuffer,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    assert_eq!(info.sharingMode, VkSharingMode::VK_SHARING_MODE_EXCLUSIVE); // TODO
    assert_eq!(info.flags, 0); // TODO

    let buffer = gpu.device
        .create_buffer(info.size, conv::map_buffer_usage(info.usage))
        .expect("Error on creating buffer");

    unsafe {
        *pBuffer = Handle::new(Buffer::Unbound(buffer));
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyBuffer(
    gpu: VkDevice,
    buffer: VkBuffer,
    _pAllocator: *const VkAllocationCallbacks,
) {
    match buffer.unbox() {
        Some(Buffer::Buffer(buffer)) => gpu.device.destroy_buffer(buffer),
        Some(Buffer::Unbound(_)) => {
            warn!("Trying to destroy a non-bound buffer, ignoring");
        }
        None => {}
    }
}
#[inline]
pub extern "C" fn gfxCreateBufferView(
    gpu: VkDevice,
    pCreateInfo: *const VkBufferViewCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pView: *mut VkBufferView,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    let end = if info.range as i32 == VK_WHOLE_SIZE {
        None
    } else {
        Some(info.offset + info.range)
    };

    let view_result = gpu.device
        .create_buffer_view(
            match *info.buffer {
                Buffer::Buffer(ref buffer) => buffer,
                Buffer::Unbound(_) => unimplemented!(),
            },
            conv::map_format(info.format),
            (Some(info.offset), end),
        );

    match view_result {
        Ok(view) => {
            unsafe {
                *pView = Handle::new(view);
            }
            VkResult::VK_SUCCESS
        }
        Err(e) => {
            error!("Buffer view not supported: {:?}", e);
            VkResult::VK_INCOMPLETE
        }
    }
}
#[inline]
pub extern "C" fn gfxDestroyBufferView(
    gpu: VkDevice,
    view: VkBufferView,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(v) = view.unbox() {
        gpu.device.destroy_buffer_view(v);
    }
}
#[inline]
pub extern "C" fn gfxCreateImage(
    gpu: VkDevice,
    pCreateInfo: *const VkImageCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pImage: *mut VkImage,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    assert_eq!(info.sharingMode, VkSharingMode::VK_SHARING_MODE_EXCLUSIVE); // TODO
    assert_eq!(info.initialLayout, VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED); // TODO

    let image = gpu.device
        .create_image(
            conv::map_image_kind(
                info.imageType,
                info.extent,
                info.arrayLayers as _,
                info.samples,
            ),
            info.mipLevels as _,
            conv::map_format(info.format)
                .expect(&format!("Unsupported image format: {:?}", info.format)),
            conv::map_tiling(info.tiling),
            conv::map_image_usage(info.usage),
            unsafe { mem::transmute(info.flags) },
        )
        .expect("Error on creating image");

    unsafe {
        *pImage = Handle::new(Image::Unbound {
            raw: image,
            mip_levels: info.mipLevels,
            array_layers: info.arrayLayers,
        });
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyImage(
    gpu: VkDevice,
    image: VkImage,
    _pAllocator: *const VkAllocationCallbacks,
) {
    match image.unbox() {
        Some(Image::Image { raw, .. }) => gpu.device.destroy_image(raw),
        Some(Image::Unbound { .. }) => {
            warn!("Trying to destroy a non-bound image, ignoring");
        }
        None => {}
    }
}
#[inline]
pub extern "C" fn gfxGetImageSubresourceLayout(
    gpu: VkDevice,
    image: VkImage,
    pSubresource: *const VkImageSubresource,
    pLayout: *mut VkSubresourceLayout,
) {
    let footprint = gpu.device.get_image_subresource_footprint(
        image.expect("Bound image expected!"),
        image.map_subresource(unsafe { *pSubresource} ),
    );
    let sub_layout = VkSubresourceLayout {
        offset: footprint.slice.start,
        size: footprint.slice.end - footprint.slice.start,
        rowPitch: footprint.row_pitch,
        depthPitch: footprint.depth_pitch,
        arrayPitch: footprint.array_pitch,
    };
    unsafe {
        *pLayout = sub_layout;
    }
}
#[inline]
pub extern "C" fn gfxCreateImageView(
    gpu: VkDevice,
    pCreateInfo: *const VkImageViewCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pView: *mut VkImageView,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    assert!(info.subresourceRange.levelCount != VK_REMAINING_MIP_LEVELS as _); // TODO
    assert!(info.subresourceRange.layerCount != VK_REMAINING_ARRAY_LAYERS as _); // TODO

    // Non-sparse images must be bound prior.
    let image = info.image.expect("Can't create view for unbound image");

    let view = gpu.device.create_image_view(
        image,
        conv::map_view_kind(info.viewType),
        conv::map_format(info.format).unwrap(),
        conv::map_swizzle(info.components),
        info.image.map_subresource_range(info.subresourceRange),
    );

    match view {
        Ok(view) => {
            unsafe { *pView = Handle::new(view) };
            VkResult::VK_SUCCESS
        }
        Err(err) => panic!("Unexpected image view creation error: {:?}", err),
    }
}
#[inline]
pub extern "C" fn gfxDestroyImageView(
    gpu: VkDevice,
    imageView: VkImageView,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(view) = imageView.unbox() {
        gpu.device.destroy_image_view(view);
    }
}
#[inline]
pub extern "C" fn gfxCreateShaderModule(
    gpu: VkDevice,
    pCreateInfo: *const VkShaderModuleCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pShaderModule: *mut VkShaderModule,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    let code = unsafe {
        slice::from_raw_parts(info.pCode as *const u8, info.codeSize as usize)
    };

    let shader_module = gpu
        .device
        .create_shader_module(code)
        .expect("Error creating shader module"); // TODO

    unsafe {
        *pShaderModule = Handle::new(shader_module);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyShaderModule(
    gpu: VkDevice,
    shaderModule: VkShaderModule,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(module) = shaderModule.unbox() {
        gpu.device.destroy_shader_module(module);
    }
}
#[inline]
pub extern "C" fn gfxCreatePipelineCache(
    _gpu: VkDevice,
    _pCreateInfo: *const VkPipelineCacheCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pPipelineCache: *mut VkPipelineCache,
) -> VkResult {
    //TODO
    unsafe { *pPipelineCache = Handle::new(()) };
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyPipelineCache(
    _gpu: VkDevice,
    pipelineCache: VkPipelineCache,
    _pAllocator: *const VkAllocationCallbacks,
) {
    let _ = pipelineCache.unbox();
}
#[inline]
pub extern "C" fn gfxGetPipelineCacheData(
    _gpu: VkDevice,
    _pipelineCache: VkPipelineCache,
    pDataSize: *mut usize,
    _pData: *mut ::std::os::raw::c_void,
) -> VkResult {
    unsafe { *pDataSize = 0; }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxMergePipelineCaches(
    _device: VkDevice,
    _dstCache: VkPipelineCache,
    _srcCacheCount: u32,
    _pSrcCaches: *const VkPipelineCache,
) -> VkResult {
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxCreateGraphicsPipelines(
    gpu: VkDevice,
    _pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkGraphicsPipelineCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    let infos = unsafe {
        slice::from_raw_parts(pCreateInfos, createInfoCount as _)
    };

    const NUM_SHADER_STAGES: usize = 5;
    let mut shader_stages = Vec::with_capacity(infos.len() * NUM_SHADER_STAGES);

    // Collect all information which we will borrow later. Need to work around
    // the borrow checker here.
    // TODO: try to refactor it once we have a more generic API
    for info in infos {
        let stages = unsafe {
            slice::from_raw_parts(info.pStages, info.stageCount as _)
        };

        for stage in stages {
            let name = unsafe { CStr::from_ptr(stage.pName).to_owned() };
            let specialization = unsafe { stage
                .pSpecializationInfo
                .as_ref()
                .map(conv::map_specialization_info)
                .unwrap_or_default()
            };

            shader_stages.push((
                name.into_string().unwrap(),
                specialization,
            ));
        }
    }

    let mut cur_shader_stage = 0;

    let descs = infos.into_iter().map(|info| {
        let (rasterizer, rasterizer_discard) = {
            let state = unsafe { &*info.pRasterizationState };

            let rasterizer = pso::Rasterizer {
                polygon_mode: conv::map_polygon_mode(state.polygonMode, state.lineWidth),
                cull_face: conv::map_cull_face(state.cullMode),
                front_face: conv::map_front_face(state.frontFace),
                depth_clamping: state.depthClampEnable == VK_TRUE,
                depth_bias: if state.depthBiasEnable == VK_TRUE {
                    Some(pso::DepthBias {
                        const_factor: state.depthBiasConstantFactor,
                        clamp: state.depthBiasClamp,
                        slope_factor: state.depthBiasSlopeFactor,
                    })
                } else {
                    None
                },
                conservative: false,
            };

            (rasterizer, state.rasterizerDiscardEnable == VK_TRUE)
        };

        let shaders = {
            let mut set = pso::GraphicsShaderSet {
                vertex: unsafe { mem::zeroed() }, // fake entry point
                hull: None,
                domain: None,
                geometry: None,
                fragment: None,
            };

            let stages = unsafe {
                slice::from_raw_parts(info.pStages, info.stageCount as _)
            };

            for stage in stages {
                use super::VkShaderStageFlagBits::*;

                let (ref name, ref specialization) = shader_stages[cur_shader_stage];
                cur_shader_stage += 1;

                let entry_point = pso::EntryPoint {
                    entry: &name,
                    module: &*stage.module,
                    specialization: &specialization,
                };

                match stage.stage {
                    VK_SHADER_STAGE_VERTEX_BIT => {
                        let fake_vs_entry = mem::replace(&mut set.vertex, entry_point);
                        mem::forget(fake_vs_entry);
                    }
                    VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT => { set.hull = Some(entry_point); }
                    VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT => { set.domain = Some(entry_point); }
                    VK_SHADER_STAGE_GEOMETRY_BIT => { set.geometry = Some(entry_point); }
                    VK_SHADER_STAGE_FRAGMENT_BIT if !rasterizer_discard => { set.fragment = Some(entry_point); }
                    stage => panic!("Unexpected shader stage: {:?}", stage),
                }
            }

            set
        };

        let (vertex_buffers, attributes) = {
            let input_state = unsafe { &*info.pVertexInputState };

            let bindings = unsafe {
                slice::from_raw_parts(
                    input_state.pVertexBindingDescriptions,
                    input_state.vertexBindingDescriptionCount as _,
                )
            };

            let attributes = unsafe {
                slice::from_raw_parts(
                    input_state.pVertexAttributeDescriptions,
                    input_state.vertexAttributeDescriptionCount as _,
                )
            };

            let bindings = bindings
                .iter()
                .map(|binding| {
                    let rate = match binding.inputRate {
                        VkVertexInputRate::VK_VERTEX_INPUT_RATE_VERTEX => 0,
                        VkVertexInputRate::VK_VERTEX_INPUT_RATE_INSTANCE => 1,
                        rate => panic!("Unexpected input rate: {:?}", rate),
                    };

                    pso::VertexBufferDesc {
                        binding: binding.binding,
                        stride: binding.stride,
                        rate,
                    }
                })
                .collect::<Vec<_>>();

            let attributes = attributes
                .into_iter()
                .map(|attrib| {
                    pso::AttributeDesc {
                        location: attrib.location,
                        binding: attrib.binding,
                        element: pso::Element {
                            format: conv::map_format(attrib.format).unwrap(), // TODO: undefined allowed?
                            offset: attrib.offset,
                        },
                    }
                })
                .collect::<Vec<_>>();

            (bindings, attributes)
        };

        let input_assembler = {
            let input_state = unsafe { &*info.pInputAssemblyState };
            let tessellation_state = shaders
                .hull
                .as_ref()
                .map(|_| unsafe { &*info.pTessellationState });

            assert_eq!(input_state.primitiveRestartEnable, VK_FALSE); // TODO

            let primitive = match conv::map_primitive_topology(
                input_state.topology,
                tessellation_state
                    .map(|state| state.patchControlPoints as _)
                    .unwrap_or(0),
            ) {
                Some(primitive) => primitive,
                None => {
                    error!("Primitive topology {:?} is not supported", input_state.topology);
                    hal::Primitive::PointList
                },
            };

            pso::InputAssemblerDesc {
                primitive,
                primitive_restart: pso::PrimitiveRestart::Disabled, // TODO
            }
        };

        // TODO: `pColorBlendState` could contain garbage, but implementations
        //        can ignore it in some circumstances. How to handle it?
        let blender = {
            let mut blend_desc = pso::BlendDesc::default();

            if let Some(state) = unsafe { info.pColorBlendState.as_ref() } {
                if state.logicOpEnable == VK_TRUE {
                    blend_desc.logic_op = Some(conv::map_logic_op(state.logicOp));
                }

                let attachments = unsafe {
                    slice::from_raw_parts(state.pAttachments, state.attachmentCount as _)
                };
                blend_desc.targets = attachments
                    .into_iter()
                    .map(|attachment| {
                        let color_mask = conv::map_color_components(attachment.colorWriteMask);

                        let blend = if attachment.blendEnable == VK_TRUE {
                            pso::BlendState::On {
                                color: conv::map_blend_op(
                                    attachment.colorBlendOp,
                                    attachment.srcColorBlendFactor,
                                    attachment.dstColorBlendFactor,
                                ),
                                alpha: conv::map_blend_op(
                                    attachment.alphaBlendOp,
                                    attachment.srcAlphaBlendFactor,
                                    attachment.dstAlphaBlendFactor,
                                ),
                            }
                        } else {
                            pso::BlendState::Off
                        };

                        pso::ColorBlendDesc(color_mask, blend)
                    })
                    .collect();
            }

            blend_desc
        };

        let multisampling = if !rasterizer_discard && !info.pMultisampleState.is_null() {
            let multisampling = unsafe { *info.pMultisampleState };

            Some(pso::Multisampling {
                rasterization_samples: multisampling.rasterizationSamples as _,
                sample_shading: if multisampling.sampleShadingEnable == VK_TRUE {
                    Some(multisampling.minSampleShading)
                } else {
                    None
                },
                sample_mask: !0, // TODO
                alpha_coverage: multisampling.alphaToCoverageEnable == VK_TRUE,
                alpha_to_one: multisampling.alphaToOneEnable == VK_TRUE,
            })
        } else {
            None
        };

        let empty_dyn_states = [];
        let dyn_states = match unsafe { info.pDynamicState.as_ref() } {
            Some(state) if !rasterizer_discard => unsafe {
                slice::from_raw_parts(state.pDynamicStates, state.dynamicStateCount as _)
            },
            _ => &empty_dyn_states,
        };

        // TODO: `pDepthStencilState` could contain garbage, but implementations
        //        can ignore it in some circumstances. How to handle it?
        let depth_stencil = if !rasterizer_discard {
            unsafe { info
                .pDepthStencilState
                .as_ref()
                .map(|state| {
                    let depth_test = if state.depthTestEnable == VK_TRUE {
                        pso::DepthTest::On {
                            fun: conv::map_compare_op(state.depthCompareOp),
                            write: state.depthWriteEnable == VK_TRUE,
                        }
                    } else {
                        pso::DepthTest::Off
                    };

                    fn map_stencil_state(state: VkStencilOpState, dyn_states: &[VkDynamicState]) -> pso::StencilFace {
                        // TODO: reference value
                        pso::StencilFace {
                            fun: conv::map_compare_op(state.compareOp),
                            mask_read: if dyn_states.iter().any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK) {
                                pso::State::Dynamic
                            } else {
                                pso::State::Static(state.compareMask)
                            },
                            mask_write: if dyn_states.iter().any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_STENCIL_WRITE_MASK) {
                                pso::State::Dynamic
                            } else {
                                pso::State::Static(state.writeMask)
                            },
                            reference: if dyn_states.iter().any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_STENCIL_REFERENCE) {
                                pso::State::Dynamic
                            } else {
                                pso::State::Static(state.reference)
                            },
                            op_fail: conv::map_stencil_op(state.failOp),
                            op_depth_fail: conv::map_stencil_op(state.depthFailOp),
                            op_pass: conv::map_stencil_op(state.passOp),
                        }
                    }

                    let stencil_test = if state.stencilTestEnable == VK_TRUE {
                        pso::StencilTest::On {
                            front: map_stencil_state(state.front, &dyn_states),
                            back: map_stencil_state(state.back, &dyn_states),
                        }
                    } else {
                        pso::StencilTest::Off
                    };

                    // TODO: depth bounds

                    pso::DepthStencilDesc {
                        depth: depth_test,
                        depth_bounds: state.depthBoundsTestEnable == VK_TRUE,
                        stencil: stencil_test,
                    }
                })
                .unwrap_or_default()
            }
        } else {
            pso::DepthStencilDesc::default()
        };

        let vp_state = if !rasterizer_discard {
            unsafe { info.pViewportState.as_ref() }
        } else {
            None
        };
        let baked_states = pso::BakedStates {
            viewport: if dyn_states.iter().any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_VIEWPORT) {
                None
            } else {
                vp_state
                    .and_then(|vp| unsafe { vp.pViewports.as_ref() })
                    .map(conv::map_viewport)
            },
            scissor: if dyn_states.iter().any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_SCISSOR) {
                None
            } else {
                vp_state
                    .and_then(|vp| unsafe { vp.pScissors.as_ref() })
                    .map(conv::map_rect)
            },
            blend_color: if dyn_states.iter().any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_BLEND_CONSTANTS) {
                None
            } else {
                unsafe { info.pColorBlendState.as_ref() }
                    .map(|cbs| cbs.blendConstants)
            },
            depth_bounds: if dyn_states.iter().any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_DEPTH_BOUNDS) {
                None
            } else {
                unsafe { info.pDepthStencilState.as_ref() }
                    .map(|db| db.minDepthBounds .. db.maxDepthBounds)
            },
        };

        let layout = &*info.layout;
        let subpass = pass::Subpass {
            index: info.subpass as _,
            main_pass: &*info.renderPass,
        };

        let flags = {
            let mut flags = pso::PipelineCreationFlags::empty();

            if info.flags & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT as u32 != 0 {
                flags |= pso::PipelineCreationFlags::DISABLE_OPTIMIZATION;
            }
            if info.flags & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT as u32 != 0 {
                flags |= pso::PipelineCreationFlags::ALLOW_DERIVATIVES;
            }

            flags
        };

        let parent = {
            let is_derivative = info.flags & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DERIVATIVE_BIT as u32 != 0;

            if let Some(base_pso) = info.basePipelineHandle.as_ref() {
                match *base_pso {
                    Pipeline::Graphics(ref pso) => pso::BasePipeline::Pipeline(pso),
                    Pipeline::Compute(_) => panic!("Base pipeline handle must be a graphics pipeline"),
                }
            } else if is_derivative && info.basePipelineIndex > 0 {
                pso::BasePipeline::Index(info.basePipelineIndex as _)
            } else {
                pso::BasePipeline::None // TODO
            }
        };

        pso::GraphicsPipelineDesc {
            shaders,
            rasterizer,
            vertex_buffers,
            attributes,
            input_assembler,
            blender,
            depth_stencil,
            multisampling,
            baked_states,
            layout,
            subpass,
            flags,
            parent,
        }
    });

    let pipelines = gpu.device.create_graphics_pipelines(descs);

    let pipelines = unsafe {
        slice::from_raw_parts_mut(pPipelines, infos.len())
            .into_iter()
            .zip(pipelines)
    };

    for (pipeline, raw) in pipelines {
        if let Ok(raw) = raw {
            *pipeline = Handle::new(Pipeline::Graphics(raw));
        }
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxCreateComputePipelines(
    gpu: VkDevice,
    _pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkComputePipelineCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    let infos = unsafe {
        slice::from_raw_parts(pCreateInfos, createInfoCount as _)
    };

    // Collect all information which we will borrow later. Need to work around
    // the borrow checker here.
    // TODO: try to refactor it once we have a more generic API
    let shader_stages = infos
        .iter()
        .map(|info| {
            let name = unsafe { CStr::from_ptr(info.stage.pName).to_owned() };
            let specialization = unsafe { info.stage
                .pSpecializationInfo
                .as_ref()
                .map(conv::map_specialization_info)
                .unwrap_or_default()
            };

            (
                name.into_string().unwrap(),
                specialization,
            )
        })
        .collect::<Vec<_>>();

    let descs = infos
        .into_iter()
        .zip(&shader_stages)
        .map(|(info, &(ref entry, ref specialization))| {
            let shader = pso::EntryPoint {
                entry,
                module: &*info.stage.module,
                specialization,
            };
            let layout = &*info.layout;

            let flags = {
                let mut flags = pso::PipelineCreationFlags::empty();

                if info.flags & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT as u32 != 0 {
                    flags |= pso::PipelineCreationFlags::DISABLE_OPTIMIZATION;
                }
                if info.flags & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT as u32 != 0 {
                    flags |= pso::PipelineCreationFlags::ALLOW_DERIVATIVES;
                }

                flags
            };

            let parent = {
                let is_derivative = info.flags & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DERIVATIVE_BIT as u32 != 0;

                if let Some(base_pso) = info.basePipelineHandle.as_ref() {
                    match *base_pso {
                        Pipeline::Graphics(_) => panic!("Base pipeline handle must be a compute pipeline"),
                        Pipeline::Compute(ref pso) => pso::BasePipeline::Pipeline(pso),
                    }
                } else if is_derivative && info.basePipelineIndex > 0 {
                    pso::BasePipeline::Index(info.basePipelineIndex as _)
                } else {
                    pso::BasePipeline::None // TODO
                }
            };

            pso::ComputePipelineDesc {
                shader,
                layout,
                flags,
                parent,
            }
        })
        .collect::<Vec<_>>();

    let pipelines = gpu.device.create_compute_pipelines(&descs);

    let pipelines = unsafe {
        slice::from_raw_parts_mut(pPipelines, descs.len())
            .into_iter()
            .zip(pipelines.into_iter())
    };

    for (pipeline, raw) in pipelines {
        if let Ok(raw) = raw {
            *pipeline = Handle::new(Pipeline::Compute(raw));
        }
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyPipeline(
    gpu: VkDevice,
    pipeline: VkPipeline,
    _pAllocator: *const VkAllocationCallbacks,
) {
    match pipeline.unbox() {
        Some(Pipeline::Graphics(pipeline)) => gpu.device.destroy_graphics_pipeline(pipeline),
        Some(Pipeline::Compute(pipeline)) => gpu.device.destroy_compute_pipeline(pipeline),
        None => {}
    }
}
#[inline]
pub extern "C" fn gfxCreatePipelineLayout(
    gpu: VkDevice,
    pCreateInfo: *const VkPipelineLayoutCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pPipelineLayout: *mut VkPipelineLayout,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    let set_layouts = unsafe {
        slice::from_raw_parts(info.pSetLayouts, info.setLayoutCount as _)
    };
    let push_constants = unsafe {
        slice::from_raw_parts(info.pPushConstantRanges, info.pushConstantRangeCount as _)
    };

    let layouts = set_layouts
        .iter()
        .map(|layout| &**layout);

    let ranges = push_constants
        .iter()
        .map(|constant| {
            let stages = conv::map_stage_flags(constant.stageFlags);
            let start = constant.offset / 4;
            let size = constant.size / 4;

            (stages, start .. start+size)
        });

    let pipeline_layout = gpu.device
        .create_pipeline_layout(layouts, ranges);

    unsafe { *pPipelineLayout = Handle::new(pipeline_layout); }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyPipelineLayout(
    gpu: VkDevice,
    pipelineLayout: VkPipelineLayout,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(layout) = pipelineLayout.unbox() {
        gpu.device.destroy_pipeline_layout(layout);
    }
}
#[inline]
pub extern "C" fn gfxCreateSampler(
    gpu: VkDevice,
    pCreateInfo: *const VkSamplerCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pSampler: *mut VkSampler,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    let gfx_info = hal::image::SamplerInfo {
        min_filter: conv::map_filter(info.minFilter),
        mag_filter: conv::map_filter(info.magFilter),
        mip_filter: conv::map_mipmap_filter(info.mipmapMode),
        wrap_mode: (
            conv::map_wrap_mode(info.addressModeU),
            conv::map_wrap_mode(info.addressModeV),
            conv::map_wrap_mode(info.addressModeW),
        ),
        lod_bias: info.mipLodBias.into(),
        lod_range: info.minLod.into() .. info.maxLod.into(),
        comparison: if info.compareEnable == VK_TRUE {
            Some(conv::map_compare_op(info.compareOp))
        } else {
            None
        },
        border: [0.0; 4].into(), // TODO
        anisotropic: if info.anisotropyEnable == VK_TRUE {
            hal::image::Anisotropic::On(info.maxAnisotropy as _)
        } else {
            hal::image::Anisotropic::Off
        },
    };
    let sampler = gpu.device.create_sampler(gfx_info);
    unsafe { *pSampler = Handle::new(sampler); }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroySampler(
    gpu: VkDevice,
    sampler: VkSampler,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(sam) = sampler.unbox() {
        gpu.device.destroy_sampler(sam);
    }
}
#[inline]
pub extern "C" fn gfxCreateDescriptorSetLayout(
    gpu: VkDevice,
    pCreateInfo: *const VkDescriptorSetLayoutCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pSetLayout: *mut VkDescriptorSetLayout,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    let layout_bindings = unsafe {
        slice::from_raw_parts(info.pBindings, info.bindingCount as _)
    };

    let sampler_iter = layout_bindings
        .iter()
        .flat_map(|binding| {
            if binding.pImmutableSamplers.is_null() {
                (&[]).into_iter().cloned()
            } else {
                let slice = unsafe {
                    slice::from_raw_parts(binding.pImmutableSamplers, binding.descriptorCount as _)
                };
                slice.iter().cloned()
            }
        });

    let mut bindings = Vec::with_capacity(layout_bindings.len());
    for binding in layout_bindings {
        bindings.push(pso::DescriptorSetLayoutBinding {
            binding: binding.binding as _,
            ty: conv::map_descriptor_type(binding.descriptorType),
            count: binding.descriptorCount as _,
            stage_flags: conv::map_stage_flags(binding.stageFlags),
            immutable_samplers: !binding.pImmutableSamplers.is_null(),
        });
    }

    let set_layout = gpu.device
        .create_descriptor_set_layout(&bindings, sampler_iter);

    unsafe { *pSetLayout = Handle::new(set_layout); }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyDescriptorSetLayout(
    gpu: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(layout) = descriptorSetLayout.unbox() {
        gpu.device.destroy_descriptor_set_layout(layout);
    }
}
#[inline]
pub extern "C" fn gfxCreateDescriptorPool(
    gpu: VkDevice,
    pCreateInfo: *const VkDescriptorPoolCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pDescriptorPool: *mut VkDescriptorPool,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };

    let pool_sizes = unsafe {
        slice::from_raw_parts(info.pPoolSizes, info.poolSizeCount as _)
    };

    let ranges = pool_sizes
        .iter()
        .map(|pool| {
            pso::DescriptorRangeDesc {
                ty: conv::map_descriptor_type(pool.type_),
                count: pool.descriptorCount as _,
            }
        })
        .collect::<Vec<_>>();

    let pool = super::DescriptorPool {
        raw: gpu.device
            .create_descriptor_pool(info.maxSets as _, &ranges),
        sets: if info.flags & VkDescriptorPoolCreateFlagBits::VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT as u32 != 0 {
            None
        } else {
            Some(Vec::new())
        },
    };

    unsafe { *pDescriptorPool = Handle::new(pool); }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyDescriptorPool(
    gpu: VkDevice,
    descriptorPool: VkDescriptorPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(pool) = descriptorPool.unbox() {
        gpu.device.destroy_descriptor_pool(pool.raw);
        if let Some(sets) = pool.sets {
            for set in sets {
                let _ = set.unbox();
            }
        }
    }
}
#[inline]
pub extern "C" fn gfxResetDescriptorPool(
    _gpu: VkDevice,
    mut descriptorPool: VkDescriptorPool,
    _flags: VkDescriptorPoolResetFlags,
) -> VkResult {
    descriptorPool.raw.reset();
    if let Some(ref mut sets) = descriptorPool.sets {
        for set in sets.drain(..) {
            let _ = set.unbox();
        }
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxAllocateDescriptorSets(
    _gpu: VkDevice,
    pAllocateInfo: *const VkDescriptorSetAllocateInfo,
    pDescriptorSets: *mut VkDescriptorSet,
) -> VkResult {
    let info = unsafe { &mut *(pAllocateInfo as *mut VkDescriptorSetAllocateInfo) };
    let pool = &mut info.descriptorPool;

    let set_layouts = unsafe {
        slice::from_raw_parts(info.pSetLayouts, info.descriptorSetCount as _)
    };
    let layouts = set_layouts
        .iter()
        .map(|layout| &**layout);

    let descriptor_sets = pool.raw.allocate_sets(layouts);
    let sets = unsafe {
        slice::from_raw_parts_mut(pDescriptorSets, info.descriptorSetCount as _)
    };
    for (set, raw_set) in sets.iter_mut().zip(descriptor_sets.into_iter()) {
        *set = match raw_set {
            Ok(set) => Handle::new(set),
            Err(e) => return match e {
                pso::AllocationError::OutOfHostMemory => VkResult::VK_ERROR_OUT_OF_HOST_MEMORY,
                pso::AllocationError::OutOfDeviceMemory => VkResult::VK_ERROR_OUT_OF_DEVICE_MEMORY,
                pso::AllocationError::OutOfPoolMemory => VkResult::VK_ERROR_OUT_OF_POOL_MEMORY_KHR,
                pso::AllocationError::IncompatibleLayout => VkResult::VK_ERROR_DEVICE_LOST,
                pso::AllocationError::FragmentedPool => VkResult::VK_ERROR_FRAGMENTED_POOL,
            },
        };
        if let Some(ref mut local_sets) = pool.sets {
            local_sets.push(*set);
        }
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxFreeDescriptorSets(
    _device: VkDevice,
    mut descriptorPool: VkDescriptorPool,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
) -> VkResult {
    let descriptor_sets = unsafe {
        slice::from_raw_parts(pDescriptorSets, descriptorSetCount as _)
    };
    assert!(descriptorPool.sets.is_none());

    descriptorPool.raw.free_sets(
        descriptor_sets
            .into_iter()
            .filter_map(|set| set.unbox())
    );

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxUpdateDescriptorSets(
    gpu: VkDevice,
    descriptorWriteCount: u32,
    pDescriptorWrites: *const VkWriteDescriptorSet,
    descriptorCopyCount: u32,
    pDescriptorCopies: *const VkCopyDescriptorSet,
) {
    let write_infos = unsafe {
        slice::from_raw_parts(pDescriptorWrites, descriptorWriteCount as _)
    };
    //TODO: investigate the safety of passing one giant iterator here
    let mut writes = SmallVec::<[pso::DescriptorSetWrite<_, _>; 16]>::with_capacity(write_infos.len());

    for write in write_infos {
        let image_info = unsafe {
            slice::from_raw_parts(write.pImageInfo, write.descriptorCount as _)
        };
        let buffer_info = unsafe {
            slice::from_raw_parts(write.pBufferInfo, write.descriptorCount as _)
        };
        let texel_buffer_views = unsafe {
            slice::from_raw_parts(write.pTexelBufferView, write.descriptorCount as _)
        };

        let ty = conv::map_descriptor_type(write.descriptorType);
        let descriptors: SmallVec<[_; 4]> = match ty {
            pso::DescriptorType::Sampler => {
                image_info
                    .into_iter()
                    .map(|image| pso::Descriptor::Sampler(&*image.sampler))
                    .collect()
            }
            pso::DescriptorType::InputAttachment |
            pso::DescriptorType::SampledImage |
            pso::DescriptorType::StorageImage => {
                image_info
                    .into_iter()
                    .map(|image| pso::Descriptor::Image(
                        &*image.imageView,
                        conv::map_image_layout(image.imageLayout),
                    ))
                    .collect()
            }
            pso::DescriptorType::UniformTexelBuffer => {
                texel_buffer_views
                    .into_iter()
                    .map(|view| pso::Descriptor::UniformTexelBuffer(
                        &**view,
                    ))
                    .collect()
            }
            pso::DescriptorType::StorageTexelBuffer => {
                texel_buffer_views
                    .into_iter()
                    .map(|view| pso::Descriptor::StorageTexelBuffer(
                        &**view,
                    ))
                    .collect()
            }
            pso::DescriptorType::UniformBuffer |
            pso::DescriptorType::StorageBuffer |
            pso::DescriptorType::UniformBufferDynamic |
            pso::DescriptorType::StorageBufferDynamic => {
                buffer_info
                    .into_iter()
                    .map(|buffer| {
                        let end = if buffer.range as i32 == VK_WHOLE_SIZE {
                            None
                        } else {
                            Some(buffer.offset + buffer.range)
                        };
                        pso::Descriptor::Buffer(
                            // Non-sparse buffer need to be bound to device memory.
                            buffer.buffer.expect("Buffer needs to be bound"),
                            Some(buffer.offset) .. end,
                        )
                    })
                    .collect()
            }
            pso::DescriptorType::CombinedImageSampler => {
                image_info
                    .into_iter()
                    .map(|image| pso::Descriptor::CombinedImageSampler(
                        &*image.imageView,
                        conv::map_image_layout(image.imageLayout),
                        &*image.sampler,
                    ))
                    .collect()
            }
        };

        writes.push(pso::DescriptorSetWrite {
            set: &*write.dstSet,
            binding: write.dstBinding as _,
            array_offset: write.dstArrayElement as _,
            descriptors,
        });
    }

    let copies = unsafe {
            slice::from_raw_parts(pDescriptorCopies, descriptorCopyCount as _)
        }
        .iter()
        .map(|copy| {
            pso::DescriptorSetCopy {
                src_set: &*copy.srcSet,
                src_binding: copy.srcBinding as _,
                src_array_offset: copy.srcArrayElement as _,
                dst_set: &*copy.dstSet,
                dst_binding: copy.dstBinding as _,
                dst_array_offset: copy.dstArrayElement as _,
                count: copy.descriptorCount as _,
            }
        });

    gpu.device.write_descriptor_sets(writes);
    gpu.device.copy_descriptor_sets(copies);
}
#[inline]
pub extern "C" fn gfxCreateFramebuffer(
    gpu: VkDevice,
    pCreateInfo: *const VkFramebufferCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pFramebuffer: *mut VkFramebuffer,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };

    let attachments_slice = unsafe {
        slice::from_raw_parts(info.pAttachments, info.attachmentCount as _)
    };
    let attachments = attachments_slice
        .into_iter()
        .map(|attachment| &**attachment);

    let extent = hal::image::Extent {
        width: info.width,
        height: info.height,
        depth: info.layers,
    };

    let framebuffer = gpu
        .device
        .create_framebuffer(&*info.renderPass, attachments, extent)
        .unwrap();

    unsafe {
        *pFramebuffer = Handle::new(framebuffer);
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyFramebuffer(
    gpu: VkDevice,
    framebuffer: VkFramebuffer,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(fbo) = framebuffer.unbox() {
        gpu.device.destroy_framebuffer(fbo);
    }
}
#[inline]
pub extern "C" fn gfxCreateRenderPass(
    gpu: VkDevice,
    pCreateInfo: *const VkRenderPassCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pRenderPass: *mut VkRenderPass,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };

    // Attachment descriptions
    let attachments = unsafe {
        slice::from_raw_parts(info.pAttachments, info.attachmentCount as _)
    };
    let attachments = attachments
        .into_iter()
        .map(|attachment| {
            assert_eq!(attachment.flags, 0); // TODO

            let initial_layout = conv::map_image_layout(attachment.initialLayout);
            let final_layout = conv::map_image_layout(attachment.finalLayout);

            pass::Attachment {
                format: conv::map_format(attachment.format),
                samples: attachment.samples as u32 as _,
                ops: pass::AttachmentOps {
                    load: conv::map_attachment_load_op(attachment.loadOp),
                    store: conv::map_attachment_store_op(attachment.storeOp),
                },
                stencil_ops: pass::AttachmentOps {
                    load: conv::map_attachment_load_op(attachment.stencilLoadOp),
                    store: conv::map_attachment_store_op(attachment.stencilStoreOp),
                },
                layouts: initial_layout .. final_layout,
            }
        })
        .collect::<Vec<_>>();

    // Subpass descriptions
    let subpasses = unsafe {
        slice::from_raw_parts(info.pSubpasses, info.subpassCount as _)
    };

    // Store all attachment references, referenced by the subpasses.
    let mut attachment_refs = Vec::with_capacity(subpasses.len());
    struct AttachmentRefs {
        input: Vec<pass::AttachmentRef>,
        color: Vec<pass::AttachmentRef>,
        resolve: Vec<pass::AttachmentRef>,
        depth_stencil: Option<pass::AttachmentRef>,
        preserve: Vec<usize>,
    }

    fn map_attachment_ref(att_ref: &VkAttachmentReference) -> pass::AttachmentRef {
        (att_ref.attachment as _, conv::map_image_layout(att_ref.layout))
    }

    for subpass in subpasses {
        let input = unsafe {
            slice::from_raw_parts(subpass.pInputAttachments, subpass.inputAttachmentCount as _)
                .into_iter()
                .map(map_attachment_ref)
                .collect()
        };
        let color = unsafe {
            slice::from_raw_parts(subpass.pColorAttachments, subpass.colorAttachmentCount as _)
                .into_iter()
                .map(map_attachment_ref)
                .collect()
        };
        let resolve = if subpass.pResolveAttachments.is_null() {
            Vec::new()
        } else {
            unsafe {
                slice::from_raw_parts(subpass.pResolveAttachments, subpass.colorAttachmentCount as _)
                    .into_iter()
                    .map(map_attachment_ref)
                    .collect()
            }
        };
        let depth_stencil = unsafe {
            subpass
                .pDepthStencilAttachment
                .as_ref()
                .map(map_attachment_ref)
        };

        let preserve = unsafe {
            slice::from_raw_parts(subpass.pPreserveAttachments, subpass.preserveAttachmentCount as _)
                .into_iter()
                .map(|id| *id as usize)
                .collect::<Vec<_>>()
        };

        attachment_refs.push(AttachmentRefs {
            input,
            color,
            resolve,
            depth_stencil,
            preserve,
        });
    }

    let subpasses = attachment_refs
        .iter()
        .map(|attachment_ref| pass::SubpassDesc {
            colors: &attachment_ref.color,
            depth_stencil: attachment_ref.depth_stencil.as_ref(),
            inputs: &attachment_ref.input,
            resolves: &attachment_ref.resolve,
            preserves: &attachment_ref.preserve,
        })
        .collect::<Vec<_>>();

    // Subpass dependencies
    let dependencies = unsafe {
        slice::from_raw_parts(info.pDependencies, info.dependencyCount as _)
    };

    fn map_subpass_ref(subpass: u32) -> pass::SubpassRef {
        if subpass == VK_SUBPASS_EXTERNAL as u32 {
            pass::SubpassRef::External
        } else {
            pass::SubpassRef::Pass(subpass as _)
        }
    }

    let dependencies = dependencies
        .into_iter()
        .map(|dependency| {
            // assert_eq!(dependency.dependencyFlags, 0); // TODO

            let src_pass = map_subpass_ref(dependency.srcSubpass);
            let dst_pass = map_subpass_ref(dependency.dstSubpass);

            let src_stage = conv::map_pipeline_stage_flags(dependency.srcStageMask);
            let dst_stage = conv::map_pipeline_stage_flags(dependency.dstStageMask);

            // Our portability implementation only supports image access flags atm.
            // Global buffer barriers can't be handled currently.
            let src_access = conv::map_image_access(dependency.srcAccessMask);
            let dst_access = conv::map_image_access(dependency.dstAccessMask);

            pass::SubpassDependency {
                passes: src_pass .. dst_pass,
                stages: src_stage .. dst_stage,
                accesses: src_access .. dst_access,
            }
        })
        .collect::<Vec<_>>();

    let render_pass = gpu
        .device
        .create_render_pass(&attachments, &subpasses, &dependencies);

    unsafe {
        *pRenderPass = Handle::new(render_pass);
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyRenderPass(
    gpu: VkDevice,
    renderPass: VkRenderPass,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(rp) = renderPass.unbox() {
        gpu.device.destroy_render_pass(rp);
    }
}
#[inline]
pub extern "C" fn gfxGetRenderAreaGranularity(
    _gpu: VkDevice,
    _renderPass: VkRenderPass,
    pGranularity: *mut VkExtent2D,
) {
    let granularity = VkExtent2D { width: 1, height: 1 }; //TODO?
    unsafe { *pGranularity = granularity };
}

#[inline]
pub extern "C" fn gfxCreateCommandPool(
    gpu: VkDevice,
    pCreateInfo: *const VkCommandPoolCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pCommandPool: *mut VkCommandPool,
) -> VkResult {
    use hal::pool::CommandPoolCreateFlags;

    let info = unsafe { &*pCreateInfo };
    let family = queue::QueueFamilyId(info.queueFamilyIndex as _);

    let mut flags = CommandPoolCreateFlags::empty();
    if info.flags & VkCommandPoolCreateFlagBits::VK_COMMAND_POOL_CREATE_TRANSIENT_BIT as u32 != 0 {
        flags |= CommandPoolCreateFlags::TRANSIENT;
    }
    if info.flags
        & VkCommandPoolCreateFlagBits::VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT as u32
        != 0
    {
        flags |= CommandPoolCreateFlags::RESET_INDIVIDUAL;
    }

    let pool = CommandPool {
        pool: gpu.device.create_command_pool(family, flags),
        buffers: Vec::new(),
    };
    unsafe { *pCommandPool = Handle::new(pool) };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxDestroyCommandPool(
    gpu: VkDevice,
    commandPool: VkCommandPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(cp) = commandPool.unbox() {
        for cmd_buf in cp.buffers {
            let _ = cmd_buf.unbox();
        }
        gpu.device.destroy_command_pool(cp.pool);
    }
}

#[inline]
pub extern "C" fn gfxResetCommandPool(
    _gpu: VkDevice,
    mut commandPool: VkCommandPool,
    _flags: VkCommandPoolResetFlags,
) -> VkResult {
    commandPool.pool.reset();
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxAllocateCommandBuffers(
    _gpu: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult {
    let info = unsafe { &mut *(pAllocateInfo as *mut VkCommandBufferAllocateInfo) };
    let level = match info.level {
        VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY => com::RawLevel::Primary,
        VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_SECONDARY => com::RawLevel::Secondary,
        level => panic!("Unexpected command buffer lvel: {:?}", level),
    };

    let count = info.commandBufferCount as usize;
    let cmd_bufs = info.commandPool.pool.allocate(count, level);

    let output = unsafe { slice::from_raw_parts_mut(pCommandBuffers, count) };
    for (out, cmd_buf) in output.iter_mut().zip(cmd_bufs) {
        *out = DispatchHandle::new(cmd_buf);
    }
    info.commandPool.buffers.extend_from_slice(output);

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxFreeCommandBuffers(
    _gpu: VkDevice,
    mut commandPool: VkCommandPool,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    let slice = unsafe {
        slice::from_raw_parts(pCommandBuffers, commandBufferCount as _)
    };
    commandPool.buffers.retain(|buf| !slice.contains(buf));

    let buffers = slice.iter().filter_map(|buffer| buffer.unbox()).collect();
    unsafe { commandPool.pool.free(buffers) };
}

#[inline]
pub extern "C" fn gfxBeginCommandBuffer(
    mut commandBuffer: VkCommandBuffer,
    pBeginInfo: *const VkCommandBufferBeginInfo,
) -> VkResult {
    let info = unsafe { &*pBeginInfo };
    let inheritance = match unsafe { info.pInheritanceInfo.as_ref() } {
        Some(ii) => com::CommandBufferInheritanceInfo {
            subpass: ii.renderPass.as_ref().map(|main_pass| pass::Subpass {
                main_pass,
                index: ii.subpass as _,
            }),
            framebuffer: ii.framebuffer.as_ref(),
            occlusion_query_enable: ii.occlusionQueryEnable != VK_FALSE,
            occlusion_query_flags: conv::map_query_control(ii.queryFlags),
            pipeline_statistics: conv::map_pipeline_statistics(ii.pipelineStatistics),
        },
        None => com::CommandBufferInheritanceInfo::default(),
    };
    commandBuffer.begin(conv::map_cmd_buffer_usage(info.flags), inheritance);

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxEndCommandBuffer(mut commandBuffer: VkCommandBuffer) -> VkResult {
    commandBuffer.finish();

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxResetCommandBuffer(
    mut commandBuffer: VkCommandBuffer,
    flags: VkCommandBufferResetFlags,
) -> VkResult {
    let release_resources = flags & VkCommandBufferResetFlagBits::VK_COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT as u32 != 0;
    commandBuffer.reset(release_resources);
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxCmdBindPipeline(
    mut commandBuffer: VkCommandBuffer,
    _pipelineBindPoint: VkPipelineBindPoint, // ignore, needs to match by spec
    pipeline: VkPipeline,
) {
    match *pipeline {
        Pipeline::Graphics(ref pipeline) => commandBuffer.bind_graphics_pipeline(pipeline),
        Pipeline::Compute(ref pipeline) => commandBuffer.bind_compute_pipeline(pipeline),
    }
}
#[inline]
pub extern "C" fn gfxCmdSetViewport(
    mut commandBuffer: VkCommandBuffer,
    firstViewport: u32,
    viewportCount: u32,
    pViewports: *const VkViewport,
) {
    let viewports = unsafe {
        slice::from_raw_parts(pViewports, viewportCount as _)
            .into_iter()
            .map(conv::map_viewport)
    };

    commandBuffer.set_viewports(firstViewport, viewports);
}
#[inline]
pub extern "C" fn gfxCmdSetScissor(
    mut commandBuffer: VkCommandBuffer,
    firstScissor: u32,
    scissorCount: u32,
    pScissors: *const VkRect2D,
) {
    let scissors = unsafe {
        slice::from_raw_parts(pScissors, scissorCount as _)
            .into_iter()
            .map(conv::map_rect)
    };

    commandBuffer.set_scissors(firstScissor, scissors);
}
#[inline]
pub extern "C" fn gfxCmdSetLineWidth(mut commandBuffer: VkCommandBuffer, lineWidth: f32) {
    commandBuffer.set_line_width(lineWidth);
}
#[inline]
pub extern "C" fn gfxCmdSetDepthBias(
    mut commandBuffer: VkCommandBuffer,
    depthBiasConstantFactor: f32,
    depthBiasClamp: f32,
    depthBiasSlopeFactor: f32,
) {
    commandBuffer.set_depth_bias(pso::DepthBias {
        const_factor: depthBiasConstantFactor,
        clamp: depthBiasClamp,
        slope_factor: depthBiasSlopeFactor,
    });
}
#[inline]
pub extern "C" fn gfxCmdSetBlendConstants(
    mut commandBuffer: VkCommandBuffer,
    blendConstants: *const f32,
) {
    let value = unsafe { *(blendConstants as *const hal::pso::ColorValue) };
    commandBuffer.set_blend_constants(value);
}
#[inline]
pub extern "C" fn gfxCmdSetDepthBounds(
    mut commandBuffer: VkCommandBuffer,
    minDepthBounds: f32,
    maxDepthBounds: f32,
) {
    commandBuffer.set_depth_bounds(minDepthBounds .. maxDepthBounds);
}
#[inline]
pub extern "C" fn gfxCmdSetStencilCompareMask(
    mut commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    compareMask: u32,
) {
    commandBuffer.set_stencil_read_mask(
        conv::map_stencil_face(faceMask),
        compareMask,
    );
}
#[inline]
pub extern "C" fn gfxCmdSetStencilWriteMask(
    mut commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    writeMask: u32,
) {
    commandBuffer.set_stencil_write_mask(
        conv::map_stencil_face(faceMask),
        writeMask,
    );
}
#[inline]
pub extern "C" fn gfxCmdSetStencilReference(
    mut commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    reference: u32,
) {
    commandBuffer.set_stencil_reference(
        conv::map_stencil_face(faceMask),
        reference,
    );
}
#[inline]
pub extern "C" fn gfxCmdBindDescriptorSets(
    mut commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    layout: VkPipelineLayout,
    firstSet: u32,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
    dynamicOffsetCount: u32,
    pDynamicOffsets: *const u32,
) {
    let descriptor_sets = unsafe {
        slice::from_raw_parts(pDescriptorSets, descriptorSetCount as _)
            .into_iter()
            .map(|set| &**set)
    };
    let offsets = unsafe {
        slice::from_raw_parts(pDynamicOffsets, dynamicOffsetCount as _)
    };

    match pipelineBindPoint {
        VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS => {
            commandBuffer.bind_graphics_descriptor_sets(
                &*layout,
                firstSet as _,
                descriptor_sets,
                offsets,
            );
        }
        VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_COMPUTE => {
            commandBuffer.bind_compute_descriptor_sets(
                &*layout,
                firstSet as _,
                descriptor_sets,
                offsets,
            );
        }
        _ => panic!("Unexpected pipeline bind point: {:?}", pipelineBindPoint),
    }
}
#[inline]
pub extern "C" fn gfxCmdBindIndexBuffer(
    mut commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    indexType: VkIndexType,
) {
    commandBuffer.bind_index_buffer(
        IndexBufferView {
            buffer: buffer.expect("Bound index buffer expected."),
            offset,
            index_type: conv::map_index_type(indexType),
        }
    );
}

#[inline]
pub extern "C" fn gfxCmdBindVertexBuffers(
    mut commandBuffer: VkCommandBuffer,
    firstBinding: u32,
    bindingCount: u32,
    pBuffers: *const VkBuffer,
    pOffsets: *const VkDeviceSize,
) {
    let buffers = unsafe {
        slice::from_raw_parts(pBuffers, bindingCount as _)
    };
    let offsets = unsafe {
        slice::from_raw_parts(pOffsets, bindingCount as _)
    };

    let views = buffers
        .into_iter()
        .zip(offsets)
        .map(|(buffer, offset)| {
            let buffer = buffer.expect("Non-sparse buffers need to be bound to device memory.");
            (buffer, *offset)
        });

    commandBuffer.bind_vertex_buffers(firstBinding, views);
}
#[inline]
pub extern "C" fn gfxCmdDraw(
    mut commandBuffer: VkCommandBuffer,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    commandBuffer.draw(
        firstVertex .. firstVertex + vertexCount,
        firstInstance .. firstInstance + instanceCount,
    )
}
#[inline]
pub extern "C" fn gfxCmdDrawIndexed(
    mut commandBuffer: VkCommandBuffer,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    vertexOffset: i32,
    firstInstance: u32,
) {
    commandBuffer.draw_indexed(
        firstIndex .. firstIndex + indexCount,
        vertexOffset,
        firstInstance .. firstInstance + instanceCount,
    )
}
#[inline]
pub extern "C" fn gfxCmdDrawIndirect(
    mut commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    commandBuffer.draw_indirect(
        buffer.expect("Bound buffer expected!"),
        offset,
        drawCount,
        stride,
    )
}
#[inline]
pub extern "C" fn gfxCmdDrawIndexedIndirect(
    mut commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    commandBuffer.draw_indexed_indirect(
        buffer.expect("Bound buffer expected!"),
        offset,
        drawCount,
        stride,
    )
}
#[inline]
pub extern "C" fn gfxCmdDispatch(
    mut commandBuffer: VkCommandBuffer,
    groupCountX: u32,
    groupCountY: u32,
    groupCountZ: u32,
) {
    commandBuffer.dispatch([groupCountX, groupCountY, groupCountZ])
}
#[inline]
pub extern "C" fn gfxCmdDispatchIndirect(
    mut commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
) {
    commandBuffer.dispatch_indirect(
        buffer.expect("Bound buffer expected!"),
        offset,
    )
}
#[inline]
pub extern "C" fn gfxCmdCopyBuffer(
    mut commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferCopy,
) {
    let regions = unsafe {
            slice::from_raw_parts(pRegions, regionCount as _)
        }
        .iter()
        .map(|r| com::BufferCopy {
            src: r.srcOffset,
            dst: r.dstOffset,
            size: r.size,
        });

    commandBuffer.copy_buffer(
        srcBuffer.expect("Bound src buffer expected!"),
        dstBuffer.expect("Bound dst buffer expected!"),
        regions,
    );
}
#[inline]
pub extern "C" fn gfxCmdCopyImage(
    mut commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageCopy,
) {
    let regions = unsafe {
            slice::from_raw_parts(pRegions, regionCount as _)
        }
        .iter()
        .map(|r| com::ImageCopy {
            src_subresource: srcImage.map_subresource_layers(r.srcSubresource),
            src_offset: conv::map_offset(r.srcOffset),
            dst_subresource: dstImage.map_subresource_layers(r.dstSubresource),
            dst_offset: conv::map_offset(r.dstOffset),
            extent: conv::map_extent(r.extent),
        });

    commandBuffer.copy_image(
        srcImage.expect("Bound src image expected!"),
        conv::map_image_layout(srcImageLayout),
        dstImage.expect("Bound dst image expected!"),
        conv::map_image_layout(dstImageLayout),
        regions,
    );
}
#[inline]
pub extern "C" fn gfxCmdBlitImage(
    mut commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageBlit,
    filter: VkFilter,
) {
    let regions = unsafe {
            slice::from_raw_parts(pRegions, regionCount as _)
        }
        .iter()
        .map(|r| com::ImageBlit {
            src_subresource: srcImage.map_subresource_layers(r.srcSubresource),
            src_bounds: conv::map_offset(r.srcOffsets[0]) .. conv::map_offset(r.srcOffsets[1]),
            dst_subresource: dstImage.map_subresource_layers(r.dstSubresource),
            dst_bounds: conv::map_offset(r.dstOffsets[0]) .. conv::map_offset(r.dstOffsets[1]),
        });

    commandBuffer.blit_image(
        srcImage.expect("Bound src image expected!"),
        conv::map_image_layout(srcImageLayout),
        dstImage.expect("Bound dst image expected!"),
        conv::map_image_layout(dstImageLayout),
        conv::map_filter(filter),
        regions,
    );
}
#[inline]
pub extern "C" fn gfxCmdCopyBufferToImage(
    mut commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkBufferImageCopy,
) {
    let regions = unsafe {
            slice::from_raw_parts(pRegions, regionCount as _)
        }
        .iter()
        .map(|r| com::BufferImageCopy {
            buffer_offset: r.bufferOffset,
            buffer_width: r.bufferRowLength,
            buffer_height: r.bufferImageHeight,
            image_layers: dstImage.map_subresource_layers(r.imageSubresource),
            image_offset: conv::map_offset(r.imageOffset),
            image_extent: conv::map_extent(r.imageExtent),
        });

    commandBuffer.copy_buffer_to_image(
        srcBuffer.expect("Bound buffer expected!"),
        dstImage.expect("Bound image expected!"),
        conv::map_image_layout(dstImageLayout),
        regions,
    );
}
#[inline]
pub extern "C" fn gfxCmdCopyImageToBuffer(
    mut commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferImageCopy,
) {
    let regions = unsafe {
            slice::from_raw_parts(pRegions, regionCount as _)
        }
        .iter()
        .map(|r| com::BufferImageCopy {
            buffer_offset: r.bufferOffset,
            buffer_width: r.bufferRowLength,
            buffer_height: r.bufferImageHeight,
            image_layers: srcImage.map_subresource_layers(r.imageSubresource),
            image_offset: conv::map_offset(r.imageOffset),
            image_extent: conv::map_extent(r.imageExtent),
        });

    commandBuffer.copy_image_to_buffer(
        srcImage.expect("Bound image expected!"),
        conv::map_image_layout(srcImageLayout),
        dstBuffer.expect("Bound buffer expected!"),
        regions,
    );
}
#[inline]
pub extern "C" fn gfxCmdUpdateBuffer(
    mut commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    dataSize: VkDeviceSize,
    pData: *const ::std::os::raw::c_void,
) {
    commandBuffer.update_buffer(
        dstBuffer.expect("Bound buffer expected!"),
        dstOffset,
        unsafe {
            slice::from_raw_parts(pData as _, dataSize as _)
        },
    );
}
#[inline]
pub extern "C" fn gfxCmdFillBuffer(
    mut commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    size: VkDeviceSize,
    data: u32,
) {
    let range = if size == VK_WHOLE_SIZE as VkDeviceSize {
        (Some(dstOffset), None)
    } else {
        (Some(dstOffset), Some(dstOffset + size))
    };
    commandBuffer.fill_buffer(
        dstBuffer.expect("Bound buffer expected!"),
        range,
        data,
    );
}
#[inline]
pub extern "C" fn gfxCmdClearColorImage(
    mut commandBuffer: VkCommandBuffer,
    image: VkImage,
    imageLayout: VkImageLayout,
    pColor: *const VkClearColorValue,
    rangeCount: u32,
    pRanges: *const VkImageSubresourceRange,
) {
    let subresource_ranges = unsafe {
        slice::from_raw_parts(pRanges, rangeCount as _)
    };
    commandBuffer.clear_image(
        image.expect("Bound image expected!"),
        conv::map_image_layout(imageLayout),
        unsafe { mem::transmute(*pColor) },
        unsafe { mem::zeroed() },
        subresource_ranges
            .iter()
            .map(|&range| image.map_subresource_range(range)),
    );
}
#[inline]
pub extern "C" fn gfxCmdClearDepthStencilImage(
    mut commandBuffer: VkCommandBuffer,
    image: VkImage,
    imageLayout: VkImageLayout,
    pDepthStencil: *const VkClearDepthStencilValue,
    rangeCount: u32,
    pRanges: *const VkImageSubresourceRange,
) {
    let subresource_ranges = unsafe {
        slice::from_raw_parts(pRanges, rangeCount as _)
    };
    commandBuffer.clear_image(
        image.expect("Bound image expected!"),
        conv::map_image_layout(imageLayout),
        unsafe { mem::zeroed() },
        unsafe { mem::transmute(*pDepthStencil) },
        subresource_ranges
            .iter()
            .map(|&range| image.map_subresource_range(range)),
    );
}
#[inline]
pub extern "C" fn gfxCmdClearAttachments(
    mut commandBuffer: VkCommandBuffer,
    attachmentCount: u32,
    pAttachments: *const VkClearAttachment,
    rectCount: u32,
    pRects: *const VkClearRect,
) {
    let attachments = unsafe {
        slice::from_raw_parts(pAttachments, attachmentCount as _)
    };
    let rects = unsafe {
        slice::from_raw_parts(pRects, rectCount as _)
    };
    commandBuffer.clear_attachments(
        attachments.iter().map(|at| {
            use VkImageAspectFlagBits::*;
            if at.aspectMask & VK_IMAGE_ASPECT_COLOR_BIT as u32 != 0 {
                com::AttachmentClear::Color {
                    index: at.colorAttachment as _,
                    value: unsafe { at.clearValue.color.float32 }.into(), //TODO?
                }
            } else {
                com::AttachmentClear::DepthStencil {
                    depth: if at.aspectMask & VK_IMAGE_ASPECT_DEPTH_BIT as u32 != 0 {
                        Some(unsafe { at.clearValue.depthStencil.depth })
                    } else { None },
                    stencil: if at.aspectMask & VK_IMAGE_ASPECT_STENCIL_BIT as u32 != 0 {
                        Some(unsafe { at.clearValue.depthStencil.stencil })
                    } else { None },
                }
            }
        }),
        rects.iter().map(conv::map_clear_rect),
    );
}
#[inline]
pub extern "C" fn gfxCmdResolveImage(
    mut commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageResolve,
) {
    let regions = unsafe {
        slice::from_raw_parts(pRegions, regionCount as _)
    }.iter().cloned().map(|resolve| com::ImageResolve {
        src_subresource: srcImage.map_subresource_layers(resolve.srcSubresource),
        src_offset: conv::map_offset(resolve.srcOffset),
        dst_subresource: srcImage.map_subresource_layers(resolve.dstSubresource),
        dst_offset: conv::map_offset(resolve.dstOffset),
        extent: conv::map_extent(resolve.extent),
    });

    commandBuffer.resolve_image(
        srcImage.expect("Bound image expected!"),
        conv::map_image_layout(srcImageLayout),
        dstImage.expect("Bound image expected!"),
        conv::map_image_layout(dstImageLayout),
        regions,
    );
}
#[inline]
pub extern "C" fn gfxCmdSetEvent(
    _commandBuffer: VkCommandBuffer,
    _event: VkEvent,
    _stageMask: VkPipelineStageFlags,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdResetEvent(
    _commandBuffer: VkCommandBuffer,
    _event: VkEvent,
    _stageMask: VkPipelineStageFlags,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdWaitEvents(
    _commandBuffer: VkCommandBuffer,
    _eventCount: u32,
    _pEvents: *const VkEvent,
    _srcStageMask: VkPipelineStageFlags,
    _dstStageMask: VkPipelineStageFlags,
    _memoryBarrierCount: u32,
    _pMemoryBarriers: *const VkMemoryBarrier,
    _bufferMemoryBarrierCount: u32,
    _pBufferMemoryBarriers: *const VkBufferMemoryBarrier,
    _imageMemoryBarrierCount: u32,
    _pImageMemoryBarriers: *const VkImageMemoryBarrier,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdPipelineBarrier(
    mut commandBuffer: VkCommandBuffer,
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
    let global_barriers = unsafe {
            slice::from_raw_parts(pMemoryBarriers, memoryBarrierCount as _)
        }
        .iter()
        .flat_map(|b| {
            let buf = conv::map_buffer_access(b.srcAccessMask) .. conv::map_buffer_access(b.dstAccessMask);
            let buf_bar = if !buf.start.is_empty() || !buf.end.is_empty() {
                Some(memory::Barrier::AllBuffers(buf))
            } else {
                None
            };
            let img = conv::map_image_access(b.srcAccessMask) .. conv::map_image_access(b.dstAccessMask);
            let img_bar = if !img.start.is_empty() || !img.end.is_empty() {
                Some(memory::Barrier::AllImages(img))
            } else {
                None
            };
            buf_bar.into_iter().chain(img_bar)
        });

    let buffer_barriers = unsafe {
            slice::from_raw_parts(pBufferMemoryBarriers, bufferMemoryBarrierCount as _)
        }
        .iter()
        .map(|b| memory::Barrier::Buffer {
            states: conv::map_buffer_access(b.srcAccessMask) .. conv::map_buffer_access(b.dstAccessMask),
            target: b.buffer.expect("Bound buffer is needed here!"),
        });

    let image_barriers = unsafe {
            slice::from_raw_parts(pImageMemoryBarriers, imageMemoryBarrierCount as _)
        }
        .iter()
        .map(|b| memory::Barrier::Image {
            states:
                (conv::map_image_access(b.srcAccessMask), conv::map_image_layout(b.oldLayout)) ..
                (conv::map_image_access(b.dstAccessMask), conv::map_image_layout(b.newLayout)),
            target: b.image.expect("Bound image is needed here!"),
            range: b.image.map_subresource_range(b.subresourceRange),
        });

    commandBuffer.pipeline_barrier(
        conv::map_pipeline_stage_flags(srcStageMask) .. conv::map_pipeline_stage_flags(dstStageMask),
        memory::Dependencies::from_bits(dependencyFlags as _).unwrap_or(memory::Dependencies::empty()),
        global_barriers.chain(buffer_barriers).chain(image_barriers),
    );
}
#[inline]
pub extern "C" fn gfxCmdBeginQuery(
    _commandBuffer: VkCommandBuffer,
    _queryPool: VkQueryPool,
    _query: u32,
    _flags: VkQueryControlFlags,
) {
    error!("Query pools are not implemented");
}
#[inline]
pub extern "C" fn gfxCmdEndQuery(
    _commandBuffer: VkCommandBuffer,
    _queryPool: VkQueryPool,
    _query: u32,
) {
    error!("Query pools are not implemented");
}
#[inline]
pub extern "C" fn gfxCmdResetQueryPool(
    _commandBuffer: VkCommandBuffer,
    _queryPool: VkQueryPool,
    _firstQuery: u32,
    _queryCount: u32,
) {
    error!("Query pools are not implemented");
}
#[inline]
pub extern "C" fn gfxCmdWriteTimestamp(
    _commandBuffer: VkCommandBuffer,
    _pipelineStage: VkPipelineStageFlagBits,
    _queryPool: VkQueryPool,
    _query: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdCopyQueryPoolResults(
    _commandBuffer: VkCommandBuffer,
    _queryPool: VkQueryPool,
    _firstQuery: u32,
    _queryCount: u32,
    _dstBuffer: VkBuffer,
    _dstOffset: VkDeviceSize,
    _stride: VkDeviceSize,
    _flags: VkQueryResultFlags,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdPushConstants(
    mut commandBuffer: VkCommandBuffer,
    layout: VkPipelineLayout,
    stageFlags: VkShaderStageFlags,
    offset: u32,
    size: u32,
    pValues: *const ::std::os::raw::c_void,
) {
    assert_eq!(size % 4, 0);

    let values = unsafe {
        slice::from_raw_parts(pValues as *const u32, size as usize / 4)
    };

    if stageFlags & VkShaderStageFlagBits::VK_SHADER_STAGE_COMPUTE_BIT as u32 != 0 {
        commandBuffer.push_compute_constants(
            &*layout,
            offset,
            values,
        );
    }
    if stageFlags & VkShaderStageFlagBits::VK_SHADER_STAGE_ALL_GRAPHICS as u32 != 0 {
        commandBuffer.push_graphics_constants(
            &*layout,
            conv::map_stage_flags(stageFlags),
            offset,
            values,
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdBeginRenderPass(
    mut commandBuffer: VkCommandBuffer,
    pRenderPassBegin: *const VkRenderPassBeginInfo,
    contents: VkSubpassContents,
) {
    let info = unsafe { &*pRenderPassBegin };

    let render_area = pso::Rect {
        x: info.renderArea.offset.x as _,
        y: info.renderArea.offset.y as _,
        w: info.renderArea.extent.width as _,
        h: info.renderArea.extent.height as _,
    };
    let clear_values = unsafe {
        slice::from_raw_parts(info.pClearValues, info.clearValueCount as _)
            .into_iter()
            .map(|cv| {
                // HAL and Vulkan clear value union sharing same memory representation
                mem::transmute::<_, com::ClearValueRaw>(*cv)
            })
    };
    let contents = conv::map_subpass_contents(contents);

    commandBuffer.begin_render_pass(
        &*info.renderPass,
        &*info.framebuffer,
        render_area,
        clear_values,
        contents,
    );
}
#[inline]
pub extern "C" fn gfxCmdNextSubpass(
    mut commandBuffer: VkCommandBuffer,
    contents: VkSubpassContents,
) {
    commandBuffer.next_subpass(conv::map_subpass_contents(contents));
}
#[inline]
pub extern "C" fn gfxCmdEndRenderPass(
    mut commandBuffer: VkCommandBuffer,
) {
    commandBuffer.end_render_pass();
}
#[inline]
pub extern "C" fn gfxCmdExecuteCommands(
    mut commandBuffer: VkCommandBuffer,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    let cmd_buffers = unsafe {
        slice::from_raw_parts(pCommandBuffers, commandBufferCount as _)
    };
    commandBuffer.execute_commands(cmd_buffers.iter().map(|cb| *cb));
}

#[inline]
pub extern "C" fn gfxDestroySurfaceKHR(
    _instance: VkInstance,
    surface: VkSurfaceKHR,
    _: *const VkAllocationCallbacks,
) {
    let _ = surface.unbox(); //TODO
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
pub extern "C" fn gfxGetPhysicalDeviceSurfaceCapabilitiesKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceCapabilities: *mut VkSurfaceCapabilitiesKHR,
) -> VkResult {
    let (caps, _, _) = surface.compatibility(&adapter.physical_device);

    let output = VkSurfaceCapabilitiesKHR {
        minImageCount: caps.image_count.start,
        maxImageCount: caps.image_count.end - 1,
        currentExtent: match caps.current_extent {
            Some(extent) => conv::extent2d_from_hal(extent),
            None => VkExtent2D {
                width: !0,
                height: !0,
            },
        },
        minImageExtent: conv::extent2d_from_hal(caps.extents.start),
        maxImageExtent: conv::extent2d_from_hal(caps.extents.end),
        maxImageArrayLayers: caps.max_image_layers as _,
        supportedTransforms: VkSurfaceTransformFlagBitsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR
            as _,
        currentTransform: VkSurfaceTransformFlagBitsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        supportedCompositeAlpha: VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR
            as _,
        supportedUsageFlags: 0,
    };

    unsafe { *pSurfaceCapabilities = output };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxGetPhysicalDeviceSurfaceFormatsKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceFormatCount: *mut u32,
    pSurfaceFormats: *mut VkSurfaceFormatKHR,
) -> VkResult {
    let formats = surface
        .compatibility(&adapter.physical_device)
        .1
        .map(|formats| formats.into_iter().map(conv::format_from_hal).collect())
        .unwrap_or(vec![VkFormat::VK_FORMAT_UNDEFINED]);

    if pSurfaceFormats.is_null() {
        // Return only the number of formats
        unsafe { *pSurfaceFormatCount = formats.len() as u32 };
    } else {
        let output =
            unsafe { slice::from_raw_parts_mut(pSurfaceFormats, *pSurfaceFormatCount as usize) };
        if output.len() > formats.len() {
            unsafe { *pSurfaceFormatCount = formats.len() as u32 };
        }
        for (out, format) in output.iter_mut().zip(formats) {
            *out = VkSurfaceFormatKHR {
                format,
                colorSpace: VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR, //TODO
            };
        }
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxGetPhysicalDeviceSurfacePresentModesKHR(
    adapter: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pPresentModeCount: *mut u32,
    pPresentModes: *mut VkPresentModeKHR,
) -> VkResult {
    let (_, _, present_modes) = surface
        .compatibility(&adapter.physical_device);

    let num_present_modes = present_modes.len();

    // If NULL, number of present modes is returned.
    if pPresentModes.is_null() {
        unsafe { *pPresentModeCount = num_present_modes as _ };
        return VkResult::VK_SUCCESS;
    }

    let output = unsafe { slice::from_raw_parts_mut(pPresentModes, *pPresentModeCount as _) };
    let num_output = output.len();
    let (code, count) = if num_output < num_present_modes {
        (VkResult::VK_INCOMPLETE, num_output)
    } else {
        (VkResult::VK_SUCCESS, num_present_modes)
    };

    for (out, present_mode) in output.iter_mut().zip(present_modes) {
        *out = conv::map_present_mode_from_hal(present_mode);
    }

    unsafe { *pPresentModeCount = count as _ };
    
    code
}

#[inline]
pub extern "C" fn gfxCreateSwapchainKHR(
    gpu: VkDevice,
    pCreateInfo: *const VkSwapchainCreateInfoKHR,
    _pAllocator: *const VkAllocationCallbacks,
    pSwapchain: *mut VkSwapchainKHR,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    // TODO: more checks
    assert_eq!(info.clipped, VK_TRUE); // TODO
    assert_eq!(
        info.imageSharingMode,
        VkSharingMode::VK_SHARING_MODE_EXCLUSIVE
    ); // TODO

    let config = hal::SwapchainConfig {
        present_mode: conv::map_present_mode(info.presentMode),
        color_format: conv::map_format(info.imageFormat).unwrap(),
        depth_stencil_format: None,
        image_count: info.minImageCount,
        image_usage: conv::map_image_usage(info.imageUsage),
    };
    let (swapchain, backbuffers) = gpu.device.create_swapchain(
        &mut info.surface.clone(),
        config,
        info.oldSwapchain.unbox().map(|s| s.raw),
        &conv::map_extent2d(info.imageExtent),
    );

    let images = match backbuffers {
        hal::Backbuffer::Images(images) => images
            .into_iter()
            .map(|image| Handle::new(Image::Image {
                raw: image,
                mip_levels: 1,
                array_layers: 1,
            }))
            .collect(),
        hal::Backbuffer::Framebuffer(_) => panic!(
            "Expected backbuffer images. Backends returning only framebuffers are not supported!"
        ),
    };

    let swapchain = Swapchain {
        raw: swapchain,
        images,
    };

    unsafe { *pSwapchain = Handle::new(swapchain) };
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroySwapchainKHR(
    _gpu: VkDevice,
    mut swapchain: VkSwapchainKHR,
    _pAllocator: *const VkAllocationCallbacks,
) {
    for image in &mut swapchain.images {
        let _ = image.unbox();
    }
    let _ = swapchain.unbox();
}
#[inline]
pub extern "C" fn gfxGetSwapchainImagesKHR(
    _gpu: VkDevice,
    swapchain: VkSwapchainKHR,
    pSwapchainImageCount: *mut u32,
    pSwapchainImages: *mut VkImage,
) -> VkResult {
    debug_assert!(!pSwapchainImageCount.is_null());

    let swapchain_image_count = unsafe { &mut *pSwapchainImageCount };
    let available_images = swapchain.images.len() as u32;

    if pSwapchainImages.is_null() {
        // If NULL the number of presentable images is returned.
        *swapchain_image_count = available_images;
    } else {
        *swapchain_image_count = available_images.min(*swapchain_image_count);
        let swapchain_images =
            unsafe { slice::from_raw_parts_mut(pSwapchainImages, *swapchain_image_count as _) };

        for i in 0..*swapchain_image_count as _ {
            swapchain_images[i] = swapchain.images[i];
        }

        if *swapchain_image_count < available_images {
            return VkResult::VK_INCOMPLETE;
        }
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxCmdProcessCommandsNVX(
    _commandBuffer: VkCommandBuffer,
    _pProcessCommandsInfo: *const VkCmdProcessCommandsInfoNVX,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdReserveSpaceForCommandsNVX(
    _commandBuffer: VkCommandBuffer,
    _pReserveSpaceInfo: *const VkCmdReserveSpaceForCommandsInfoNVX,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateIndirectCommandsLayoutNVX(
    _gpu: VkDevice,
    _pCreateInfo: *const VkIndirectCommandsLayoutCreateInfoNVX,
    _pAllocator: *const VkAllocationCallbacks,
    _pIndirectCommandsLayout: *mut VkIndirectCommandsLayoutNVX,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyIndirectCommandsLayoutNVX(
    _gpu: VkDevice,
    _indirectCommandsLayout: VkIndirectCommandsLayoutNVX,
    _pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateObjectTableNVX(
    _gpu: VkDevice,
    _pCreateInfo: *const VkObjectTableCreateInfoNVX,
    _pAllocator: *const VkAllocationCallbacks,
    _pObjectTable: *mut VkObjectTableNVX,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyObjectTableNVX(
    _gpu: VkDevice,
    _objectTable: VkObjectTableNVX,
    _pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxRegisterObjectsNVX(
    _gpu: VkDevice,
    _objectTable: VkObjectTableNVX,
    _objectCount: u32,
    _ppObjectTableEntries: *const *const VkObjectTableEntryNVX,
    _pObjectIndices: *const u32,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxUnregisterObjectsNVX(
    _gpu: VkDevice,
    _objectTable: VkObjectTableNVX,
    _objectCount: u32,
    _pObjectEntryTypes: *const VkObjectEntryTypeNVX,
    _pObjectIndices: *const u32,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceGeneratedCommandsPropertiesNVX(
    _physicalDevice: VkPhysicalDevice,
    _pFeatures: *mut VkDeviceGeneratedCommandsFeaturesNVX,
    _pLimits: *mut VkDeviceGeneratedCommandsLimitsNVX,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetViewportWScalingNV(
    _commandBuffer: VkCommandBuffer,
    _firstViewport: u32,
    _viewportCount: u32,
    _pViewportWScalings: *const VkViewportWScalingNV,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxReleaseDisplayEXT(
    _physicalDevice: VkPhysicalDevice,
    _display: VkDisplayKHR,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceSurfaceCapabilities2EXT(
    _physicalDevice: VkPhysicalDevice,
    _surface: VkSurfaceKHR,
    _pSurfaceCapabilities: *mut VkSurfaceCapabilities2EXT,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDisplayPowerControlEXT(
    _gpu: VkDevice,
    _display: VkDisplayKHR,
    _pDisplayPowerInfo: *const VkDisplayPowerInfoEXT,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxRegisterDeviceEventEXT(
    _gpu: VkDevice,
    _pDeviceEventInfo: *const VkDeviceEventInfoEXT,
    _pAllocator: *const VkAllocationCallbacks,
    _pFence: *mut VkFence,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxRegisterDisplayEventEXT(
    _gpu: VkDevice,
    _display: VkDisplayKHR,
    _pDisplayEventInfo: *const VkDisplayEventInfoEXT,
    _pAllocator: *const VkAllocationCallbacks,
    _pFence: *mut VkFence,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetSwapchainCounterEXT(
    _gpu: VkDevice,
    _swapchain: VkSwapchainKHR,
    _counter: VkSurfaceCounterFlagBitsEXT,
    _pCounterValue: *mut u64,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetDiscardRectangleEXT(
    _commandBuffer: VkCommandBuffer,
    _firstDiscardRectangle: u32,
    _discardRectangleCount: u32,
    _pDiscardRectangles: *const VkRect2D,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateWin32SurfaceKHR(
    instance: VkInstance,
    pCreateInfo: *const VkWin32SurfaceCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    assert!(pAllocator.is_null());
    let info = unsafe { &*pCreateInfo };
    #[cfg(all(feature = "gfx-backend-vulkan", target_os = "windows"))]
    {
        unsafe {
            assert_eq!(info.flags, 0);
            *pSurface = Handle::new(
                instance.backend.create_surface_from_hwnd(info.hinstance, info.hwnd),
            );
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(feature = "gfx-backend-dx12")]
    {
        unsafe {
            assert_eq!(info.flags, 0);
            *pSurface = Handle::new(instance.backend.create_surface_from_hwnd(info.hwnd));
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(feature = "gfx-backend-dx11")]
    {
        unsafe {
            assert_eq!(info.flags, 0);
            *pSurface = Handle::new(instance.backend.create_surface_from_hwnd(info.hwnd));
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (instance, info, pSurface);
        unreachable!()
    }
}
pub extern "C" fn gfxCreateXcbSurfaceKHR(
    instance: VkInstance,
    pCreateInfo: *const VkXcbSurfaceCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    assert!(pAllocator.is_null());
    let info = unsafe { &*pCreateInfo };
    #[cfg(all(feature = "gfx-backend-vulkan", target_os = "linux"))]
    {
        unsafe {
            assert_eq!(info.flags, 0);
            *pSurface = Handle::new(
                instance.backend.create_surface_from_xcb(info.connection as _, info.window),
            );
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(not(all(feature = "gfx-backend-vulkan", target_os = "linux")))]
    {
        let _ = (instance, info, pSurface);
        unreachable!()
    }
}
#[inline]
pub extern "C" fn gfxAcquireNextImageKHR(
    _device: VkDevice,
    mut swapchain: VkSwapchainKHR,
    _timeout: u64, // TODO
    semaphore: VkSemaphore,
    fence: VkFence,
    pImageIndex: *mut u32,
) -> VkResult {
    let sync = match semaphore.as_ref() {
        Some(sem) => FrameSync::Semaphore(sem),
        None => FrameSync::Fence(&*fence),
    };

    let frame = swapchain.raw.acquire_image(sync).unwrap();
    unsafe { *pImageIndex = frame; }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxQueuePresentKHR(
    mut queue: VkQueue,
    pPresentInfo: *const VkPresentInfoKHR,
) -> VkResult {
    let info = unsafe { &*pPresentInfo };

    let swapchain_slice = unsafe {
        slice::from_raw_parts(info.pSwapchains, info.swapchainCount as _)
    };
    let index_slice = unsafe {
        slice::from_raw_parts(info.pImageIndices, info.swapchainCount as _)
    };
    let swapchains = swapchain_slice
        .into_iter()
        .zip(index_slice)
        .map(|(swapchain, index)| (&swapchain.raw, *index));

    let wait_semaphores = unsafe {
        slice::from_raw_parts(info.pWaitSemaphores, info.waitSemaphoreCount as _)
            .into_iter()
            .map(|semaphore| &**semaphore)
    };

    queue.present(swapchains, wait_semaphores).unwrap();

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxCreateMacOSSurfaceMVK(
    instance: VkInstance,
    pCreateInfo: *const VkMacOSSurfaceCreateInfoMVK,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    assert!(pAllocator.is_null());
    let info = unsafe { &*pCreateInfo };
    #[cfg(target_os="macos")]
    unsafe {
        assert_eq!(info.flags, 0);
        *pSurface = Handle::new(
            instance.backend.create_surface_from_nsview(info.pView),
        );
        VkResult::VK_SUCCESS
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (instance, info, pSurface);
        unreachable!()
    }
}
