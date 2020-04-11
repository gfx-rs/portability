use hal::{
    adapter::PhysicalDevice,
    buffer::IndexBufferView,
    command::CommandBuffer,
    device::{Device, WaitFor},
    pool::CommandPool as _,
    pso::DescriptorPool,
    queue::{CommandQueue as _, QueueFamily},
    window::{PresentMode, PresentationSurface as _, Surface as _},
    {command as com, memory, pass, pso, queue},
    {Features, Instance},
};

use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    os::raw::{c_int, c_void},
    mem, ptr, str,
};
#[cfg(feature = "gfx-backend-metal")]
use std::env;

use super::*;

const VERSION: (u32, u32, u32) = (1, 0, 66);
const DRIVER_VERSION: u32 = 1;

fn map_oom(oom: hal::device::OutOfMemory) -> VkResult {
    match oom {
        hal::device::OutOfMemory::Host => VkResult::VK_ERROR_OUT_OF_HOST_MEMORY,
        hal::device::OutOfMemory::Device => VkResult::VK_ERROR_OUT_OF_DEVICE_MEMORY,
    }
}

fn map_alloc_error(alloc_error: hal::device::AllocationError) -> VkResult {
    match alloc_error {
        hal::device::AllocationError::OutOfMemory(oom) => map_oom(oom),
        hal::device::AllocationError::TooManyObjects => VkResult::VK_ERROR_TOO_MANY_OBJECTS,
    }
}

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
        let _ = env_logger::try_init();
        let backend = if cfg!(feature = "gfx-backend-vulkan") {
            "Vulkan"
        } else if cfg!(feature = "gfx-backend-dx12") {
            "DX12"
        } else if cfg!(feature = "gfx-backend-metal") {
            "Metal"
        } else {
            "Other"
        };
        println!("gfx-portability backend: {}", backend);
    }

    #[allow(unused_mut)]
    // Metal branch performs mutation, so we silence the warning on other backends.
    let mut backend =
        back::Instance::create("portability", 1).expect("failed to create backend instance");

    #[cfg(feature = "gfx-backend-metal")]
    {
        if let Ok(value) = env::var("GFX_METAL_ARGUMENTS") {
            backend.experiments.argument_buffers = match value.to_lowercase().as_str() {
                "yes" => true,
                "no" => false,
                other => panic!("unknown arguments option: {}", other),
            };
            println!(
                "GFX: arguments override {:?}",
                backend.experiments.argument_buffers
            );
        }
    }

    let adapters = backend
        .enumerate_adapters()
        .into_iter()
        .map(Handle::new)
        .collect();

    let create_info = unsafe { &*pCreateInfo };
    let application_info = unsafe { create_info.pApplicationInfo.as_ref() };

    if let Some(ai) = application_info {
        // Compare major and minor parts of version only - patch is ignored
        let (supported_major, supported_minor, _) = VERSION;
        let requested_major_minor = ai.apiVersion >> 12;
        let version_supported = requested_major_minor & (supported_major << 10 | supported_minor)
            == requested_major_minor;
        if !version_supported {
            return VkResult::VK_ERROR_INCOMPATIBLE_DRIVER;
        }
    }

    let mut enabled_extensions = Vec::new();
    if create_info.enabledExtensionCount != 0 {
        for raw in unsafe {
            slice::from_raw_parts(
                create_info.ppEnabledExtensionNames,
                create_info.enabledExtensionCount as _,
            )
        } {
            let cstr = unsafe { CStr::from_ptr(*raw) };
            if !INSTANCE_EXTENSION_NAMES.contains(&cstr.to_bytes_with_nul()) {
                return VkResult::VK_ERROR_EXTENSION_NOT_PRESENT;
            }
            let owned = cstr.to_str().expect("Invalid extension name").to_owned();
            enabled_extensions.push(owned);
        }
    }

    unsafe {
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

    output[..count].copy_from_slice(&instance.adapters[..count]);
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
                hal::queue::QueueType::General => {
                    VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT as u32
                        | VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT as u32
                        | VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT as u32
                }
                hal::queue::QueueType::Graphics => {
                    VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT as u32
                        | VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT as u32
                }
                hal::queue::QueueType::Compute => {
                    VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT as u32
                        | VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT as u32
                }
                hal::queue::QueueType::Transfer => VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT as u32,
            },
            queueCount: family.max_queues() as _,
            timestampValidBits: 0, //TODO
            minImageTransferGranularity: VkExtent3D {
                width: 1,
                height: 1,
                depth: 1,
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
        *pFeatures = conv::features_from_hal(features);
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceFeatures2KHR(
    adapter: VkPhysicalDevice,
    pFeatures: *mut VkPhysicalDeviceFeatures2KHR,
) {
    let features = adapter.physical_device.features();
    let mut ptr = pFeatures as *const VkStructureType;
    while !ptr.is_null() {
        ptr = match unsafe { *ptr } {
            VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2_KHR => {
                let data = unsafe { (ptr as *mut VkPhysicalDeviceFeatures2KHR).as_mut().unwrap() };
                data.features = conv::features_from_hal(features);
                data.pNext
            }
            VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PORTABILITY_SUBSET_FEATURES_EXTX => {
                let data = unsafe {
                    (ptr as *mut VkPhysicalDevicePortabilitySubsetFeaturesEXTX)
                        .as_mut()
                        .unwrap()
                };
                if features.contains(hal::Features::TRIANGLE_FAN) {
                    data.triangleFans = VK_TRUE;
                }
                if features.contains(hal::Features::SEPARATE_STENCIL_REF_VALUES) {
                    data.separateStencilMaskRef = VK_TRUE;
                }
                if features.contains(hal::Features::SAMPLER_MIP_LOD_BIAS) {
                    data.samplerMipLodBias = VK_TRUE;
                }
                //TODO: turn this into a feature flag
                if !cfg!(feature = "gfx-backend-metal") {
                    data.standardImageViews = VK_TRUE;
                }
                data.pNext
            }
            other => {
                warn!("Unrecognized {:?}, skipping", other);
                unsafe {
                    (ptr as *const VkBaseStruct)
                        .as_ref()
                        .unwrap()
                }
                .pNext
            }
        } as *const VkStructureType;
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceFormatProperties(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    pFormatProperties: *mut VkFormatProperties,
) {
    let properties = adapter
        .physical_device
        .format_properties(conv::map_format(format));
    unsafe {
        *pFormatProperties = conv::format_properties_from_hal(properties);
    }
}

fn get_physical_device_image_format_properties(
    adapter: VkPhysicalDevice,
    info: &VkPhysicalDeviceImageFormatInfo2KHR,
) -> Option<VkImageFormatProperties> {
    adapter
        .physical_device
        .image_format_properties(
            conv::map_format(info.format).unwrap(),
            match info.type_ {
                VkImageType::VK_IMAGE_TYPE_1D => 1,
                VkImageType::VK_IMAGE_TYPE_2D => 2,
                VkImageType::VK_IMAGE_TYPE_3D => 3,
                other => panic!("Unexpected image type: {:?}", other),
            },
            conv::map_tiling(info.tiling),
            conv::map_image_usage(info.usage),
            conv::map_image_create_flags(info.flags),
        )
        .map(conv::image_format_properties_from_hal)
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceImageFormatProperties(
    adapter: VkPhysicalDevice,
    format: VkFormat,
    type_: VkImageType,
    tiling: VkImageTiling,
    usage: VkImageUsageFlags,
    flags: VkImageCreateFlags,
    pImageFormatProperties: *mut VkImageFormatProperties,
) -> VkResult {
    let info = VkPhysicalDeviceImageFormatInfo2KHR {
        sType: VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR,
        pNext: ptr::null(),
        format,
        type_,
        tiling,
        usage,
        flags,
    };
    match get_physical_device_image_format_properties(adapter, &info) {
        Some(props) => unsafe {
            *pImageFormatProperties = props;
            VkResult::VK_SUCCESS
        },
        None => VkResult::VK_ERROR_FORMAT_NOT_SUPPORTED,
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceImageFormatProperties2KHR(
    adapter: VkPhysicalDevice,
    pImageFormatInfo: *const VkPhysicalDeviceImageFormatInfo2KHR,
    pImageFormatProperties: *mut VkImageFormatProperties2KHR,
) -> VkResult {
    let mut properties = None;

    let mut ptr = pImageFormatInfo as *const VkStructureType;
    while !ptr.is_null() {
        ptr = match unsafe { *ptr } {
            VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR => {
                let data = unsafe {
                    (ptr as *const VkPhysicalDeviceImageFormatInfo2KHR)
                        .as_ref()
                        .unwrap()
                };
                properties = get_physical_device_image_format_properties(adapter, data);
                data.pNext
            }
            VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_VIEW_SUPPORT_EXTX => {
                let data = unsafe {
                    (ptr as *const VkPhysicalDeviceImageViewSupportEXTX)
                        .as_ref()
                        .unwrap()
                };
                #[cfg(feature = "gfx-backend-metal")]
                {
                    if !adapter.physical_device.supports_swizzle(
                        conv::map_format(data.format).unwrap(),
                        conv::map_swizzle(data.components),
                    ) {
                        return VkResult::VK_ERROR_FORMAT_NOT_SUPPORTED;
                    }
                }
                data.pNext
            }
            other => {
                warn!("Unrecognized {:?}, skipping", other);
                unsafe {
                    (ptr as *const VkBaseStruct)
                        .as_ref()
                        .unwrap()
                }
                .pNext
            }
        } as *const VkStructureType;
    }

    match properties {
        Some(props) => unsafe {
            (*pImageFormatProperties).imageFormatProperties = props;
            VkResult::VK_SUCCESS
        },
        None => VkResult::VK_ERROR_FORMAT_NOT_SUPPORTED,
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

    use hal::adapter::DeviceType;
    let device_type = match adapter.info.device_type {
        DeviceType::IntegratedGpu => VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU,
        DeviceType::DiscreteGpu => VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU,
        DeviceType::VirtualGpu => VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU,
        DeviceType::Other => VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_OTHER,
        DeviceType::Cpu => VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_CPU,
    };

    unsafe {
        *pProperties = VkPhysicalDeviceProperties {
            apiVersion: (major << 22) | (minor << 12) | patch,
            driverVersion: DRIVER_VERSION,
            vendorID: adapter_info.vendor as _,
            deviceID: adapter_info.device as _,
            deviceType: device_type,
            deviceName: device_name,
            pipelineCacheUUID: [0; 16usize],
            limits,
            sparseProperties: sparse_properties,
        };
    }
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceProperties2KHR(
    adapter: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties2KHR,
) {
    let mut ptr = pProperties as *const VkStructureType;
    while !ptr.is_null() {
        ptr = match unsafe { *ptr } {
            VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2_KHR => {
                let data = unsafe {
                    (ptr as *mut VkPhysicalDeviceProperties2KHR).as_mut().unwrap()
                };
                gfxGetPhysicalDeviceProperties(adapter, &mut data.properties);
                data.pNext
            }
            VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PORTABILITY_SUBSET_PROPERTIES_EXTX => {
                let data = unsafe {
                    (ptr as *mut VkPhysicalDevicePortabilitySubsetPropertiesEXTX).as_mut().unwrap()
                };
                let limits = adapter.physical_device.limits();
                data.minVertexInputBindingStrideAlignment = limits.min_vertex_input_binding_stride_alignment as u32;
                data.pNext
            }
            other => {
                warn!("Unrecognized {:?}, skipping", other);
                unsafe {
                    (ptr as *const VkBaseStruct).as_ref().unwrap()
                }.pNext
            }
        } as *const VkStructureType;
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

    proc_addr! { name,
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
        vkGetPhysicalDeviceFeatures2KHR, PFN_vkGetPhysicalDeviceFeatures2KHR => gfxGetPhysicalDeviceFeatures2KHR,
        vkGetPhysicalDeviceProperties, PFN_vkGetPhysicalDeviceProperties => gfxGetPhysicalDeviceProperties,
        vkGetPhysicalDeviceProperties2KHR, PFN_vkGetPhysicalDeviceProperties2KHR => gfxGetPhysicalDeviceProperties2KHR,
        vkGetPhysicalDeviceFormatProperties, PFN_vkGetPhysicalDeviceFormatProperties => gfxGetPhysicalDeviceFormatProperties,
        vkGetPhysicalDeviceImageFormatProperties, PFN_vkGetPhysicalDeviceImageFormatProperties => gfxGetPhysicalDeviceImageFormatProperties,
        vkGetPhysicalDeviceImageFormatProperties2KHR, PFN_vkGetPhysicalDeviceImageFormatProperties2KHR => gfxGetPhysicalDeviceImageFormatProperties2KHR,
        vkGetPhysicalDeviceMemoryProperties, PFN_vkGetPhysicalDeviceMemoryProperties => gfxGetPhysicalDeviceMemoryProperties,
        vkGetPhysicalDeviceQueueFamilyProperties, PFN_vkGetPhysicalDeviceQueueFamilyProperties => gfxGetPhysicalDeviceQueueFamilyProperties,
        vkGetPhysicalDeviceSparseImageFormatProperties, PFN_vkGetPhysicalDeviceSparseImageFormatProperties => gfxGetPhysicalDeviceSparseImageFormatProperties,

        vkGetPhysicalDeviceSurfaceSupportKHR, PFN_vkGetPhysicalDeviceSurfaceSupportKHR => gfxGetPhysicalDeviceSurfaceSupportKHR,
        vkGetPhysicalDeviceSurfaceCapabilitiesKHR, PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR => gfxGetPhysicalDeviceSurfaceCapabilitiesKHR,
        vkGetPhysicalDeviceSurfaceCapabilities2KHR, PFN_vkGetPhysicalDeviceSurfaceCapabilities2KHR => gfxGetPhysicalDeviceSurfaceCapabilities2KHR,
        vkGetPhysicalDeviceSurfaceFormatsKHR, PFN_vkGetPhysicalDeviceSurfaceFormatsKHR => gfxGetPhysicalDeviceSurfaceFormatsKHR,
        vkGetPhysicalDeviceSurfaceFormats2KHR, PFN_vkGetPhysicalDeviceSurfaceFormats2KHR => gfxGetPhysicalDeviceSurfaceFormats2KHR,
        vkGetPhysicalDeviceSurfacePresentModesKHR, PFN_vkGetPhysicalDeviceSurfacePresentModesKHR => gfxGetPhysicalDeviceSurfacePresentModesKHR,
        vkGetPhysicalDeviceWin32PresentationSupportKHR, PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR => gfxGetPhysicalDeviceWin32PresentationSupportKHR,

        vkCreateWin32SurfaceKHR, PFN_vkCreateWin32SurfaceKHR => gfxCreateWin32SurfaceKHR,
        vkCreateMetalSurfaceEXT, PFN_vkCreateMetalSurfaceEXT => gfxCreateMetalSurfaceEXT,
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
                let search_name = str::from_utf8(
                    &VK_KHR_SWAPCHAIN_EXTENSION_NAME[..VK_KHR_SWAPCHAIN_EXTENSION_NAME.len() - 1],
                )
                .unwrap();
                if !device
                    .enabled_extensions
                    .iter()
                    .any(|ext| ext == search_name)
                {
                    return None;
                }
            }
            _ => {}
        }
    }

    proc_addr! { name,
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
        //vkGetImageMemoryRequirements2KHR, PFN_vkGetImageMemoryRequirements2KHR => gfxGetImageMemoryRequirements2KHR,
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
            (family, &priorities[..info.queueCount as usize])
        })
        .collect::<Vec<_>>();

    let enabled = if let Some(ef) = unsafe { dev_info.pEnabledFeatures.as_ref() } {
        fn feat(on: u32, flag: Features) -> Features {
            if on != 0 {
                flag
            } else {
                Features::empty()
            }
        }

        // Attributes on expressions are experimental for now. Use function as workaround.
        #[rustfmt::skip]
        fn feats(ef: &VkPhysicalDeviceFeatures) -> Features {
            feat(ef.robustBufferAccess, Features::ROBUST_BUFFER_ACCESS) |
            feat(ef.fullDrawIndexUint32, Features::FULL_DRAW_INDEX_U32) |
            feat(ef.imageCubeArray, Features::IMAGE_CUBE_ARRAY) |
            feat(ef.independentBlend, Features::INDEPENDENT_BLENDING) |
            feat(ef.geometryShader, Features::GEOMETRY_SHADER) |
            feat(ef.tessellationShader, Features::TESSELLATION_SHADER) |
            feat(ef.sampleRateShading, Features::SAMPLE_RATE_SHADING) |
            feat(ef.dualSrcBlend, Features::DUAL_SRC_BLENDING) |
            feat(ef.logicOp, Features::LOGIC_OP) |
            feat(ef.multiDrawIndirect, Features::MULTI_DRAW_INDIRECT) |
            feat(ef.drawIndirectFirstInstance, Features::DRAW_INDIRECT_FIRST_INSTANCE) |
            feat(ef.depthClamp, Features::DEPTH_CLAMP) |
            feat(ef.depthBiasClamp, Features::DEPTH_BIAS_CLAMP) |
            feat(ef.fillModeNonSolid, Features::NON_FILL_POLYGON_MODE) |
            feat(ef.depthBounds, Features::DEPTH_BOUNDS) |
            feat(ef.wideLines, Features::LINE_WIDTH) |
            feat(ef.largePoints, Features::POINT_SIZE) |
            feat(ef.alphaToOne, Features::ALPHA_TO_ONE) |
            feat(ef.multiViewport, Features::MULTI_VIEWPORTS) |
            feat(ef.samplerAnisotropy, Features::SAMPLER_ANISOTROPY) |
            feat(ef.textureCompressionETC2, Features::FORMAT_ETC2) |
            feat(ef.textureCompressionASTC_LDR, Features::FORMAT_ASTC_LDR) |
            feat(ef.textureCompressionBC, Features::FORMAT_BC) |
            feat(ef.occlusionQueryPrecise, Features::PRECISE_OCCLUSION_QUERY) |
            feat(ef.pipelineStatisticsQuery, Features::PIPELINE_STATISTICS_QUERY) |
            feat(ef.vertexPipelineStoresAndAtomics, Features::VERTEX_STORES_AND_ATOMICS) |
            feat(ef.fragmentStoresAndAtomics, Features::FRAGMENT_STORES_AND_ATOMICS) |
            feat(ef.shaderTessellationAndGeometryPointSize, Features::SHADER_TESSELLATION_AND_GEOMETRY_POINT_SIZE) |
            feat(ef.shaderImageGatherExtended, Features::SHADER_IMAGE_GATHER_EXTENDED) |
            feat(ef.shaderStorageImageExtendedFormats, Features::SHADER_STORAGE_IMAGE_EXTENDED_FORMATS) |
            feat(ef.shaderStorageImageMultisample, Features::SHADER_STORAGE_IMAGE_MULTISAMPLE) |
            feat(ef.shaderStorageImageReadWithoutFormat, Features::SHADER_STORAGE_IMAGE_READ_WITHOUT_FORMAT) |
            feat(ef.shaderStorageImageWriteWithoutFormat, Features::SHADER_STORAGE_IMAGE_WRITE_WITHOUT_FORMAT) |
            feat(ef.shaderUniformBufferArrayDynamicIndexing, Features::SHADER_UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING) |
            feat(ef.shaderSampledImageArrayDynamicIndexing, Features::SHADER_SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING) |
            feat(ef.shaderStorageBufferArrayDynamicIndexing, Features::SHADER_STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING) |
            feat(ef.shaderStorageImageArrayDynamicIndexing, Features::SHADER_STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING) |
            feat(ef.shaderClipDistance, Features::SHADER_CLIP_DISTANCE) |
            feat(ef.shaderCullDistance, Features::SHADER_CULL_DISTANCE) |
            feat(ef.shaderFloat64, Features::SHADER_FLOAT64) |
            feat(ef.shaderInt64, Features::SHADER_INT64) |
            feat(ef.shaderInt16, Features::SHADER_INT16) |
            feat(ef.shaderResourceResidency, Features::SHADER_RESOURCE_RESIDENCY) |
            feat(ef.shaderResourceMinLod, Features::SHADER_RESOURCE_MIN_LOD) |
            feat(ef.sparseBinding, Features::SPARSE_BINDING) |
            feat(ef.sparseResidencyBuffer, Features::SPARSE_RESIDENCY_BUFFER) |
            feat(ef.sparseResidencyImage2D, Features::SPARSE_RESIDENCY_IMAGE_2D) |
            feat(ef.sparseResidencyImage3D, Features::SPARSE_RESIDENCY_IMAGE_3D) |
            feat(ef.sparseResidency2Samples, Features::SPARSE_RESIDENCY_2_SAMPLES) |
            feat(ef.sparseResidency4Samples, Features::SPARSE_RESIDENCY_4_SAMPLES) |
            feat(ef.sparseResidency8Samples, Features::SPARSE_RESIDENCY_8_SAMPLES) |
            feat(ef.sparseResidency16Samples, Features::SPARSE_RESIDENCY_16_SAMPLES) |
            feat(ef.sparseResidencyAliased, Features::SPARSE_RESIDENCY_ALIASED) |
            feat(ef.variableMultisampleRate, Features::VARIABLE_MULTISAMPLE_RATE) |
            feat(ef.inheritedQueries, Features::INHERITED_QUERIES)
        }
        feats(&ef)
    } else {
        Features::empty()
    };

    #[cfg(feature = "renderdoc")]
    let mut renderdoc = {
        use renderdoc::RenderDoc;
        RenderDoc::new().expect("Failed to init renderdoc")
    };

    let gpu = unsafe { adapter.physical_device.open(&request_infos, enabled) };

    match gpu {
        Ok(mut gpu) => {
            #[cfg(feature = "gfx-backend-metal")]
            {
                use back::OnlineRecording;

                if let Ok(value) = env::var("GFX_METAL_RECORDING") {
                    gpu.device.online_recording = match value.to_lowercase().as_str() {
                        "immediate" => OnlineRecording::Immediate,
                        "deferred" => OnlineRecording::Deferred,
                        //"remote" => OnlineRecording::Remote(dispatch::QueuePriority::Default),
                        other => panic!("unknown recording option: {}", other),
                    };
                    println!("GFX: recording override {:?}", gpu.device.online_recording);
                }
            }

            let queues = queue_infos
                .iter()
                .map(|info| {
                    let queues = gpu
                        .queue_groups
                        .iter()
                        .position(|group| group.family.0 == info.queueFamilyIndex as usize)
                        .map(|i| gpu.queue_groups.swap_remove(i).queues)
                        .unwrap()
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

            let mut enabled_extensions = Vec::new();
            if dev_info.enabledExtensionCount != 0 {
                for raw in unsafe {
                    slice::from_raw_parts(
                        dev_info.ppEnabledExtensionNames,
                        dev_info.enabledExtensionCount as _,
                    )
                } {
                    let cstr = unsafe { CStr::from_ptr(*raw) };
                    if !DEVICE_EXTENSION_NAMES.contains(&cstr.to_bytes_with_nul()) {
                        return VkResult::VK_ERROR_EXTENSION_NOT_PRESENT;
                    }
                    let owned = cstr.to_str().expect("Invalid extension name").to_owned();
                    enabled_extensions.push(owned);
                }
            }

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
        Err(err) => {
            error!("{:?}", err);
            conv::map_err_device_creation(err)
        }
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

lazy_static! {
    // TODO: Request from backend
    static ref INSTANCE_EXTENSION_NAMES: Vec<&'static [u8]> = {
        vec![
            VK_KHR_SURFACE_EXTENSION_NAME,
            #[cfg(target_os="windows")]
            VK_KHR_WIN32_SURFACE_EXTENSION_NAME,
            #[cfg(feature="gfx-backend-metal")]
            VK_EXT_METAL_SURFACE_EXTENSION_NAME,
            #[cfg(target_os="macos")]
            VK_MVK_MACOS_SURFACE_EXTENSION_NAME,
            VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME,
            VK_KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME,
        ]
    };

    static ref INSTANCE_EXTENSIONS: Vec<VkExtensionProperties> = {
        let mut extensions = vec![
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_SURFACE_EXTENSION_NAME
                specVersion: VK_KHR_SURFACE_SPEC_VERSION,
            },
            #[cfg(target_os="windows")]
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_WIN32_SURFACE_EXTENSION_NAME
                specVersion: VK_KHR_WIN32_SURFACE_SPEC_VERSION,
            },
            #[cfg(feature="gfx-backend-metal")]
            VkExtensionProperties {
                extensionName: [0; 256], // VK_EXT_METAL_SURFACE_EXTENSION_NAME
                specVersion: VK_EXT_METAL_SURFACE_SPEC_VERSION,
            },
            #[cfg(target_os="macos")]
            VkExtensionProperties {
                extensionName: [0; 256], // VK_MVK_MACOS_SURFACE_EXTENSION_NAME
                specVersion: VK_MVK_MACOS_SURFACE_SPEC_VERSION,
            },
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME
                specVersion: VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_SPEC_VERSION,
            },
        ];

        for (&name, extension) in INSTANCE_EXTENSION_NAMES.iter().zip(&mut extensions) {
            extension
                .extensionName[.. name.len()]
                .copy_from_slice(unsafe {
                    mem::transmute(name)
                });
        }

        extensions
    };

    static ref DEVICE_EXTENSION_NAMES: Vec<&'static [u8]> = {
        vec![
            VK_KHR_SWAPCHAIN_EXTENSION_NAME,
            VK_KHR_MAINTENANCE1_EXTENSION_NAME,
            VK_EXTX_PORTABILITY_SUBSET_EXTENSION_NAME,
        ]
    };

    static ref DEVICE_EXTENSIONS: Vec<VkExtensionProperties> = {
        let mut extensions = [
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_SWAPCHAIN_EXTENSION_NAME
                specVersion: VK_KHR_SWAPCHAIN_SPEC_VERSION,
            },
            VkExtensionProperties {
                extensionName: [0; 256], // VK_KHR_MAINTENANCE1_EXTENSION_NAME
                specVersion: VK_KHR_MAINTENANCE1_SPEC_VERSION,
            },
            VkExtensionProperties {
                extensionName: [0; 256], // VK_EXTX_PORTABILITY_SUBSET_EXTENSION_NAME
                specVersion: VK_EXTX_PORTABILITY_SUBSET_SPEC_VERSION,
            },
        ];

        for (&name, extension) in DEVICE_EXTENSION_NAMES.iter().zip(&mut extensions) {
            extension
                .extensionName[.. name.len()]
                .copy_from_slice(unsafe { mem::transmute(name) });
        }

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
    unsafe {
        *pPropertyCount = 0;
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxEnumerateDeviceLayerProperties(
    _physicalDevice: VkPhysicalDevice,
    pPropertyCount: *mut u32,
    _pProperties: *mut VkLayerProperties,
) -> VkResult {
    warn!("TODO: gfxEnumerateDeviceLayerProperties");
    unsafe {
        *pPropertyCount = 0;
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxGetDeviceQueue(
    gpu: VkDevice,
    queueFamilyIndex: u32,
    queueIndex: u32,
    pQueue: *mut VkQueue,
) {
    let queue = gpu.queues.get(&queueFamilyIndex).unwrap()[queueIndex as usize];

    #[cfg(feature = "gfx-backend-metal")]
    {
        if let Ok(value) = env::var("GFX_METAL_STITCHING") {
            let mut q = queue;
            q.stitch_deferred = match value.to_lowercase().as_str() {
                "yes" => true,
                "no" => false,
                other => panic!("unknown stitching option: {}", other),
            };
            println!("GFX: stitching override {:?}", q.stitch_deferred);
        }
    }

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
    let submits = unsafe { slice::from_raw_parts(pSubmits, submitCount as usize) };
    for (i, submission) in submits.iter().enumerate() {
        let cmd_slice = unsafe {
            slice::from_raw_parts(
                submission.pCommandBuffers,
                submission.commandBufferCount as _,
            )
        };
        let wait_semaphores = unsafe {
            let semaphores = slice::from_raw_parts(
                submission.pWaitSemaphores,
                submission.waitSemaphoreCount as _,
            );
            let stages = slice::from_raw_parts(
                submission.pWaitDstStageMask,
                submission.waitSemaphoreCount as _,
            );

            stages
                .into_iter()
                .zip(semaphores)
                .filter(|(_, semaphore)| !semaphore.is_fake)
                .map(|(stage, semaphore)| (&semaphore.raw, conv::map_pipeline_stage_flags(*stage)))
        };
        let signal_semaphores = unsafe {
            slice::from_raw_parts(
                submission.pSignalSemaphores,
                submission.signalSemaphoreCount as _,
            )
            .into_iter()
            .map(|semaphore| {
                semaphore.as_mut().unwrap().is_fake = false;
                &semaphore.raw
            })
        };

        let submission = hal::queue::Submission {
            command_buffers: cmd_slice.iter(),
            wait_semaphores,
            signal_semaphores,
        };

        // only provide the fence for the last submission
        //TODO: support multiple submissions at gfx-hal level
        let fence = if i + 1 == submits.len() {
            fence.as_ref().map(|f| &f.raw)
        } else {
            None
        };
        unsafe {
            queue.submit(submission, fence);
        }
    }

    // sometimes, all you need is a fence...
    if submits.is_empty() {
        use std::iter::empty;
        let submission = hal::queue::Submission {
            command_buffers: empty(),
            wait_semaphores: empty(),
            signal_semaphores: empty(),
        };
        type RawSemaphore = <B as hal::Backend>::Semaphore;
        unsafe {
            queue.submit::<VkCommandBuffer, _, RawSemaphore, _, _>(submission, fence.as_ref().map(|f| &f.raw))
        };
    }

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
    unsafe {
        let info = &*pAllocateInfo;
        let memory = gpu
            .device
            .allocate_memory(
                hal::MemoryTypeId(info.memoryTypeIndex as _),
                info.allocationSize,
            )
            .unwrap(); // TODO:

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
        unsafe {
            gpu.device.free_memory(mem);
        }
    }
}
#[inline]
pub extern "C" fn gfxMapMemory(
    gpu: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    _flags: VkMemoryMapFlags,
    ppData: *mut *mut c_void,
) -> VkResult {
    let range = hal::memory::Segment {
        offset,
        size: if size == VK_WHOLE_SIZE as VkDeviceSize { None } else { Some(size) },
    };
    unsafe {
        *ppData = gpu.device.map_memory(&memory, range).unwrap() as *mut _; // TODO
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxUnmapMemory(gpu: VkDevice, memory: VkDeviceMemory) {
    unsafe {
        gpu.device.unmap_memory(&memory);
    }
}
#[inline]
pub extern "C" fn gfxFlushMappedMemoryRanges(
    gpu: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    let ranges = unsafe { slice::from_raw_parts(pMemoryRanges, memoryRangeCount as _) }
        .iter()
        .map(|r| {
            let range = hal::memory::Segment {
                offset: r.offset,
                size: if r.size == VK_WHOLE_SIZE as VkDeviceSize { None } else { Some(r.size) }
            };
            (&*r.memory, range)
        });

    match unsafe { gpu.device.flush_mapped_memory_ranges(ranges) } {
        Ok(()) => VkResult::VK_SUCCESS,
        Err(oom) => map_oom(oom),
    }
}
#[inline]
pub extern "C" fn gfxInvalidateMappedMemoryRanges(
    gpu: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    let ranges = unsafe { slice::from_raw_parts(pMemoryRanges, memoryRangeCount as _) }
        .iter()
        .map(|r| {
            let range = hal::memory::Segment {
                offset: r.offset,
                size: if r.size == VK_WHOLE_SIZE as VkDeviceSize { None } else { Some(r.size) }
            };
            (&*r.memory, range)
        });

    match unsafe { gpu.device.invalidate_mapped_memory_ranges(ranges) } {
        Ok(()) => VkResult::VK_SUCCESS,
        Err(oom) => map_oom(oom),
    }
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
    unsafe {
        gpu.device
            .bind_buffer_memory(&memory, memoryOffset, &mut *buffer)
            .unwrap(); //TODO
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxBindImageMemory(
    gpu: VkDevice,
    mut image: VkImage,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    let raw = match *image {
        Image::Native { ref mut raw, .. } => raw,
        Image::SwapchainFrame { .. } => panic!("Unexpected swapchain image"),
    };
    unsafe {
        gpu.device
            .bind_image_memory(&memory, memoryOffset, raw)
            .unwrap(); //TODO
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxGetBufferMemoryRequirements(
    gpu: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    let req = unsafe { gpu.device.get_buffer_requirements(&*buffer) };

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
    let raw = image.to_native().unwrap().raw;
    let req = unsafe { gpu.device.get_image_requirements(raw) };

    *unsafe { &mut *pMemoryRequirements } = VkMemoryRequirements {
        size: req.size,
        alignment: req.alignment,
        memoryTypeBits: req.type_mask as _,
    };
}
/*
#[inline]
pub extern "C" fn gfxGetImageMemoryRequirements2KHR(
    gpu: VkDevice,
    image: VkImage,
    pMemoryRequirements: *mut VkMemoryRequirements2KHR,
) {
    let mut ptr = pMemoryRequirements as *const VkStructureType;
    while !ptr.is_null() {
        ptr = match unsafe { *ptr } {
            VkStructureType::VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2_KHR => {
                let data = unsafe {
                    (ptr as *mut VkMemoryRequirements2KHR).as_mut().unwrap()
                };
                gfxGetImageMemoryRequirements(gpu, image, &mut data.memoryRequirements);
                data.features = conv::features_from_hal(features);
                data.pNext
            }
            other => {
                warn!("Unrecognized {:?}, skipping", other);
                unsafe {
                    (ptr as *const VkBaseStruct).as_ref().unwrap()
                }.pNext
            }
        } as *const VkStructureType;
    }
}*/

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
    unsafe {
        *pPropertyCount = 0;
    } //TODO
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

    let fence = match gpu.device.create_fence(signalled) {
        Ok(raw) => Fence { raw, is_fake: false },
        Err(oom) => return map_oom(oom),
    };

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
        unsafe {
            gpu.device.destroy_fence(fence.raw);
        }
    }
}
#[inline]
pub extern "C" fn gfxResetFences(
    gpu: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
) -> VkResult {
    let fence_slice = unsafe { slice::from_raw_parts(pFences, fenceCount as _) };
    let fences = fence_slice.iter().map(|fence| {
        fence.as_mut().unwrap().is_fake = false;
        &fence.raw
    });

    match unsafe { gpu.device.reset_fences(fences) } {
        Ok(()) => VkResult::VK_SUCCESS,
        Err(oom) => map_oom(oom),
    }
}
#[inline]
pub extern "C" fn gfxGetFenceStatus(gpu: VkDevice, fence: VkFence) -> VkResult {
    if fence.is_fake {
        VkResult::VK_SUCCESS
    } else {
        match unsafe { gpu.device.get_fence_status(&fence.raw) } {
            Ok(true) => VkResult::VK_SUCCESS,
            Ok(false) => VkResult::VK_NOT_READY,
            Err(hal::device::DeviceLost) => VkResult::VK_ERROR_DEVICE_LOST,
        }
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
    let result = match fenceCount {
        0 => Ok(true),
        1 if !unsafe { (*pFences) }.is_fake => {
            unsafe { gpu.device.wait_for_fence(&(*pFences).raw, timeout) }
        }
        _ => {
            let fence_slice = unsafe { slice::from_raw_parts(pFences, fenceCount as _) };
            if fence_slice.iter().all(|fence| fence.is_fake) {
                return VkResult::VK_SUCCESS
            }
            let fences = fence_slice
                .iter()
                .filter(|fence| !fence.is_fake)
                .map(|fence| &fence.raw);
            let wait_for = match waitAll {
                VK_FALSE => WaitFor::Any,
                _ => WaitFor::All,
            };
            unsafe { gpu.device.wait_for_fences(fences, wait_for, timeout) }
        }
    };

    match result {
        Ok(true) => VkResult::VK_SUCCESS,
        Ok(false) => VkResult::VK_TIMEOUT,
        Err(hal::device::OomOrDeviceLost::OutOfMemory(oom)) => map_oom(oom),
        Err(hal::device::OomOrDeviceLost::DeviceLost(hal::device::DeviceLost)) => {
            VkResult::VK_ERROR_DEVICE_LOST
        }
    }
}
#[inline]
pub extern "C" fn gfxCreateSemaphore(
    gpu: VkDevice,
    _pCreateInfo: *const VkSemaphoreCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pSemaphore: *mut VkSemaphore,
) -> VkResult {
    let semaphore = match gpu.device.create_semaphore() {
        Ok(raw) => Semaphore { raw, is_fake: false },
        Err(oom) => return map_oom(oom),
    };

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
        unsafe {
            gpu.device.destroy_semaphore(sem.raw);
        }
    }
}
#[inline]
pub extern "C" fn gfxCreateEvent(
    gpu: VkDevice,
    _pCreateInfo: *const VkEventCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pEvent: *mut VkEvent,
) -> VkResult {
    let event = match gpu.device.create_event() {
        Ok(e) => e,
        Err(oom) => return map_oom(oom),
    };

    unsafe {
        *pEvent = Handle::new(event);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyEvent(
    gpu: VkDevice,
    event: VkEvent,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(event) = event.unbox() {
        unsafe {
            gpu.device.destroy_event(event);
        }
    }
}
#[inline]
pub extern "C" fn gfxGetEventStatus(gpu: VkDevice, event: VkEvent) -> VkResult {
    match unsafe { gpu.device.get_event_status(&event) } {
        Ok(true) => VkResult::VK_EVENT_SET,
        Ok(false) => VkResult::VK_EVENT_RESET,
        Err(hal::device::OomOrDeviceLost::OutOfMemory(oom)) => map_oom(oom),
        Err(hal::device::OomOrDeviceLost::DeviceLost(hal::device::DeviceLost)) => {
            VkResult::VK_ERROR_DEVICE_LOST
        }
    }
}
#[inline]
pub extern "C" fn gfxSetEvent(gpu: VkDevice, event: VkEvent) -> VkResult {
    match unsafe { gpu.device.set_event(&event) } {
        Ok(()) => VkResult::VK_SUCCESS,
        Err(oom) => map_oom(oom),
    }
}
#[inline]
pub extern "C" fn gfxResetEvent(gpu: VkDevice, event: VkEvent) -> VkResult {
    match unsafe { gpu.device.reset_event(&event) } {
        Ok(()) => VkResult::VK_SUCCESS,
        Err(oom) => map_oom(oom),
    }
}
#[inline]
pub extern "C" fn gfxCreateQueryPool(
    gpu: VkDevice,
    pCreateInfo: *const VkQueryPoolCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pQueryPool: *mut VkQueryPool,
) -> VkResult {
    let pool = unsafe {
        let info = &*pCreateInfo;
        gpu.device.create_query_pool(
            conv::map_query_type(info.queryType, info.pipelineStatistics),
            info.queryCount,
        )
    };

    match pool {
        Ok(pool) => {
            unsafe { *pQueryPool = Handle::new(pool) };
            VkResult::VK_SUCCESS
        }
        Err(_) => {
            unsafe { *pQueryPool = Handle::null() };
            VkResult::VK_ERROR_OUT_OF_DEVICE_MEMORY
        }
    }
}
#[inline]
pub extern "C" fn gfxDestroyQueryPool(
    gpu: VkDevice,
    queryPool: VkQueryPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(pool) = queryPool.unbox() {
        unsafe {
            gpu.device.destroy_query_pool(pool);
        }
    }
}
#[inline]
pub extern "C" fn gfxGetQueryPoolResults(
    gpu: VkDevice,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
    dataSize: usize,
    pData: *mut c_void,
    stride: VkDeviceSize,
    flags: VkQueryResultFlags,
) -> VkResult {
    let result = unsafe {
        gpu.device.get_query_pool_results(
            &*queryPool,
            firstQuery..firstQuery + queryCount,
            slice::from_raw_parts_mut(pData as *mut u8, dataSize),
            stride,
            conv::map_query_result(flags),
        )
    };
    match result {
        Ok(true) => VkResult::VK_SUCCESS,
        Ok(false) => VkResult::VK_NOT_READY,
        Err(_) => VkResult::VK_ERROR_OUT_OF_DEVICE_MEMORY,
    }
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

    unsafe {
        let buffer = gpu
            .device
            .create_buffer(info.size, conv::map_buffer_usage(info.usage))
            .expect("Error on creating buffer");
        *pBuffer = Handle::new(buffer);
    };
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyBuffer(
    gpu: VkDevice,
    buffer: VkBuffer,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(buffer) = buffer.unbox() {
        unsafe {
            gpu.device.destroy_buffer(buffer);
        }
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
    let view_result = unsafe {
        gpu.device.create_buffer_view(
            &info.buffer,
            conv::map_format(info.format),
            hal::buffer::SubRange {
                offset: info.offset,
                size: if info.range as i32 == VK_WHOLE_SIZE { None } else { Some(info.range) },
            },
        )
    };

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
        unsafe {
            gpu.device.destroy_buffer_view(v);
        }
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
    if info.initialLayout != VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED {
        warn!("unexpected initial layout: {:?}", info.initialLayout);
    }

    let kind = conv::map_image_kind(
        info.imageType,
        info.extent,
        info.arrayLayers as _,
        info.samples,
    );
    unsafe {
        let image = gpu
            .device
            .create_image(
                kind,
                info.mipLevels as _,
                conv::map_format(info.format)
                    .expect(&format!("Unsupported image format: {:?}", info.format)),
                conv::map_tiling(info.tiling),
                conv::map_image_usage(info.usage),
                conv::map_image_create_flags(info.flags),
            )
            .expect("Error on creating image");

        *pImage = Handle::new(Image::Native {
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
    if let Some(Image::Native { raw, .. }) = image.unbox() {
        unsafe {
            gpu.device.destroy_image(raw);
        }
    }
}
#[inline]
pub extern "C" fn gfxGetImageSubresourceLayout(
    gpu: VkDevice,
    image: VkImage,
    pSubresource: *const VkImageSubresource,
    pLayout: *mut VkSubresourceLayout,
) {
    let img = image.to_native().unwrap();
    let footprint = unsafe {
        gpu.device
            .get_image_subresource_footprint(img.raw, img.map_subresource(*pSubresource))
    };

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
    if let Image::SwapchainFrame { swapchain, frame } = *info.image {
        unsafe {
            *pView = Handle::new(ImageView::SwapchainFrame {
                swapchain,
                frame,
            });
        }
        return VkResult::VK_SUCCESS;
    }

    let img = info.image.to_native().unwrap();
    let view = unsafe {
        gpu.device.create_image_view(
            img.raw,
            conv::map_view_kind(info.viewType),
            conv::map_format(info.format).unwrap(),
            conv::map_swizzle(info.components),
            img.map_subresource_range(info.subresourceRange),
        )
    };

    match view {
        Ok(view) => {
            unsafe { *pView = Handle::new(ImageView::Native(view)) };
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
    if let Some(ImageView::Native(view)) = imageView.unbox() {
        unsafe {
            gpu.device.destroy_image_view(view);
        }
    }
}
#[inline]
pub extern "C" fn gfxCreateShaderModule(
    gpu: VkDevice,
    pCreateInfo: *const VkShaderModuleCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pShaderModule: *mut VkShaderModule,
) -> VkResult {
    unsafe {
        let info = &*pCreateInfo;
        let code = slice::from_raw_parts(info.pCode, info.codeSize / 4);
        let shader_module = gpu
            .device
            .create_shader_module(code)
            .expect("Error creating shader module"); // TODO
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
        unsafe {
            gpu.device.destroy_shader_module(module);
        }
    }
}
#[inline]
pub extern "C" fn gfxCreatePipelineCache(
    gpu: VkDevice,
    pCreateInfo: *const VkPipelineCacheCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pPipelineCache: *mut VkPipelineCache,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    let data = if info.initialDataSize != 0 {
        Some(unsafe {
            slice::from_raw_parts(
                info.pInitialData as *const u8,
                info.initialDataSize,
            )
        })
    } else {
        None
    };

    let cache = match unsafe { gpu.device.create_pipeline_cache(data) } {
        Ok(cache) => cache,
        Err(oom) => return map_oom(oom),
    };
    unsafe { *pPipelineCache = Handle::new(cache) };

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyPipelineCache(
    gpu: VkDevice,
    pipelineCache: VkPipelineCache,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(cache) = pipelineCache.unbox() {
        unsafe {
            gpu.device.destroy_pipeline_cache(cache);
        }
    }
}
#[inline]
pub extern "C" fn gfxGetPipelineCacheData(
    _gpu: VkDevice,
    _pipelineCache: VkPipelineCache,
    pDataSize: *mut usize,
    _pData: *mut c_void,
) -> VkResult {
    //TODO: save
    unsafe {
        *pDataSize = 0;
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxMergePipelineCaches(
    gpu: VkDevice,
    dstCache: VkPipelineCache,
    srcCacheCount: u32,
    pSrcCaches: *const VkPipelineCache,
) -> VkResult {
    match unsafe {
        let caches = slice::from_raw_parts(pSrcCaches, srcCacheCount as usize);
        gpu.device
            .merge_pipeline_caches(&*dstCache, caches.iter().map(|h| &**h))
    } {
        Ok(()) => VkResult::VK_SUCCESS,
        Err(oom) => map_oom(oom),
    }
}

#[inline]
pub extern "C" fn gfxCreateGraphicsPipelines(
    gpu: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkGraphicsPipelineCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    let infos = unsafe { slice::from_raw_parts(pCreateInfos, createInfoCount as _) };

    let mut spec_constants = Vec::new();
    let mut spec_data = Vec::new();

    // Collect all information which we will borrow later. Need to work around
    // the borrow checker here.
    for info in infos {
        let stages = unsafe { slice::from_raw_parts(info.pStages, info.stageCount as _) };
        for stage in stages {
            if let Some(spec_info) = unsafe { stage.pSpecializationInfo.as_ref() } {
                let entries = unsafe {
                    slice::from_raw_parts(spec_info.pMapEntries, spec_info.mapEntryCount as _)
                };
                for entry in entries {
                    let base = spec_data.len() as u16 + entry.offset as u16;
                    spec_constants.push(pso::SpecializationConstant {
                        id: entry.constantID,
                        range: base..base + (entry.size as u16),
                    });
                }
                spec_data.extend_from_slice(unsafe {
                    slice::from_raw_parts(spec_info.pData as *const u8, spec_info.dataSize)
                });
            }
        }
    }

    let mut cur_specialization = 0;

    let descs = infos.into_iter().map(|info| {
        let rasterizer_discard =
            unsafe { &*info.pRasterizationState }.rasterizerDiscardEnable == VK_TRUE;

        let empty_dyn_states = [];
        let dyn_states = match unsafe { info.pDynamicState.as_ref() } {
            Some(state) if !rasterizer_discard => unsafe {
                slice::from_raw_parts(state.pDynamicStates, state.dynamicStateCount as _)
            },
            _ => &empty_dyn_states,
        };

        let rasterizer = {
            let state = unsafe { &*info.pRasterizationState };
            pso::Rasterizer {
                polygon_mode: match state.polygonMode {
                    VkPolygonMode::VK_POLYGON_MODE_FILL => pso::PolygonMode::Fill,
                    VkPolygonMode::VK_POLYGON_MODE_LINE => pso::PolygonMode::Line,
                    VkPolygonMode::VK_POLYGON_MODE_POINT => pso::PolygonMode::Point,
                    mode => panic!("Unexpected polygon mode: {:?}", mode),
                },
                cull_face: conv::map_cull_face(state.cullMode),
                front_face: conv::map_front_face(state.frontFace),
                depth_clamping: state.depthClampEnable == VK_TRUE,
                depth_bias: if state.depthBiasEnable == VK_TRUE {
                    Some(
                        if dyn_states
                            .iter()
                            .any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_DEPTH_BIAS)
                        {
                            pso::State::Dynamic
                        } else {
                            pso::State::Static(pso::DepthBias {
                                const_factor: state.depthBiasConstantFactor,
                                clamp: state.depthBiasClamp,
                                slope_factor: state.depthBiasSlopeFactor,
                            })
                        },
                    )
                } else {
                    None
                },
                conservative: false,
                line_width: if dyn_states
                    .iter()
                    .any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_LINE_WIDTH)
                {
                    pso::State::Dynamic
                } else {
                    pso::State::Static(state.lineWidth)
                },
            }
        };

        let shaders = {
            let mut vertex = mem::MaybeUninit::uninit();
            let mut hull = None;
            let mut domain = None;
            let mut geometry = None;
            let mut fragment = None;

            let stages = unsafe { slice::from_raw_parts(info.pStages, info.stageCount as _) };

            for stage in stages {
                use super::VkShaderStageFlagBits::*;

                let name = unsafe { CStr::from_ptr(stage.pName) };
                let spec_count = unsafe {
                    stage
                        .pSpecializationInfo
                        .as_ref()
                        .map(|spec_info| spec_info.mapEntryCount as usize)
                        .unwrap_or(0)
                };
                let entry_point = pso::EntryPoint {
                    entry: name.to_str().unwrap(),
                    module: &*stage.module,
                    specialization: pso::Specialization {
                        constants: Cow::from(
                            &spec_constants[cur_specialization..cur_specialization + spec_count],
                        ),
                        data: Cow::from(&spec_data),
                    },
                };
                cur_specialization += spec_count;

                match stage.stage {
                    VK_SHADER_STAGE_VERTEX_BIT => {
                        vertex = mem::MaybeUninit::new(entry_point);
                    }
                    VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT => {
                        hull = Some(entry_point);
                    }
                    VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT => {
                        domain = Some(entry_point);
                    }
                    VK_SHADER_STAGE_GEOMETRY_BIT => {
                        geometry = Some(entry_point);
                    }
                    VK_SHADER_STAGE_FRAGMENT_BIT if !rasterizer_discard => {
                        fragment = Some(entry_point);
                    }
                    stage => panic!("Unexpected shader stage: {:?}", stage),
                }
            }

            pso::GraphicsShaderSet {
                vertex: unsafe { vertex.assume_init() },
                hull,
                domain,
                geometry,
                fragment,
            }
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
                        VkVertexInputRate::VK_VERTEX_INPUT_RATE_VERTEX => {
                            pso::VertexInputRate::Vertex
                        }
                        VkVertexInputRate::VK_VERTEX_INPUT_RATE_INSTANCE => {
                            pso::VertexInputRate::Instance(1)
                        }
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

            if input_state.primitiveRestartEnable != VK_FALSE {
                warn!("Primitive restart may not work as expected!");
            }

            let (primitive, with_adjacency) = match conv::map_primitive_topology(
                input_state.topology,
                tessellation_state
                    .map(|state| state.patchControlPoints as _)
                    .unwrap_or(0),
            ) {
                Some(mapped) => mapped,
                None => {
                    error!(
                        "Primitive topology {:?} is not supported",
                        input_state.topology
                    );
                    (hal::pso::Primitive::PointList, false)
                }
            };

            pso::InputAssemblerDesc {
                primitive,
                with_adjacency,
                restart_index: None, // TODO
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
                        let mask = conv::map_color_components(attachment.colorWriteMask);

                        let blend = if attachment.blendEnable == VK_TRUE {
                            Some(pso::BlendState {
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
                            })
                        } else {
                            None
                        };

                        pso::ColorBlendDesc { mask, blend }
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

        // TODO: `pDepthStencilState` could contain garbage, but implementations
        //        can ignore it in some circumstances. How to handle it?
        let depth_stencil = if !rasterizer_discard {
            unsafe {
                info.pDepthStencilState
                    .as_ref()
                    .map(|state| {
                        let depth_test = if state.depthTestEnable == VK_TRUE {
                            Some(pso::DepthTest {
                                fun: conv::map_compare_op(state.depthCompareOp),
                                write: state.depthWriteEnable == VK_TRUE,
                            })
                        } else {
                            None
                        };

                        fn map_stencil_state(state: VkStencilOpState) -> pso::StencilFace {
                            pso::StencilFace {
                                fun: conv::map_compare_op(state.compareOp),
                                op_fail: conv::map_stencil_op(state.failOp),
                                op_depth_fail: conv::map_stencil_op(state.depthFailOp),
                                op_pass: conv::map_stencil_op(state.passOp),
                            }
                        }

                        let stencil_test = if state.stencilTestEnable == VK_TRUE {
                            Some(pso::StencilTest {
                                faces: pso::Sided {
                                    front: map_stencil_state(state.front),
                                    back: map_stencil_state(state.back),
                                },
                                read_masks: if dyn_states.iter().any(|&ds| {
                                    ds == VkDynamicState::VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK
                                }) {
                                    pso::State::Dynamic
                                } else {
                                    pso::State::Static(pso::Sided {
                                        front: state.front.compareMask,
                                        back: state.back.compareMask,
                                    })
                                },
                                write_masks: if dyn_states.iter().any(|&ds| {
                                    ds == VkDynamicState::VK_DYNAMIC_STATE_STENCIL_WRITE_MASK
                                }) {
                                    pso::State::Dynamic
                                } else {
                                    pso::State::Static(pso::Sided {
                                        front: state.front.writeMask,
                                        back: state.back.writeMask,
                                    })
                                },
                                reference_values: if dyn_states.iter().any(|&ds| {
                                    ds == VkDynamicState::VK_DYNAMIC_STATE_STENCIL_REFERENCE
                                }) {
                                    pso::State::Dynamic
                                } else {
                                    pso::State::Static(pso::Sided {
                                        front: state.front.reference,
                                        back: state.back.reference,
                                    })
                                },
                            })
                        } else {
                            None
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
            viewport: if dyn_states
                .iter()
                .any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_VIEWPORT)
            {
                None
            } else {
                vp_state
                    .and_then(|vp| unsafe { vp.pViewports.as_ref() })
                    .map(conv::map_viewport)
            },
            scissor: if dyn_states
                .iter()
                .any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_SCISSOR)
            {
                None
            } else {
                vp_state
                    .and_then(|vp| unsafe { vp.pScissors.as_ref() })
                    .map(conv::map_rect)
            },
            blend_color: if dyn_states
                .iter()
                .any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_BLEND_CONSTANTS)
            {
                None
            } else {
                unsafe { info.pColorBlendState.as_ref() }.map(|cbs| cbs.blendConstants)
            },
            depth_bounds: if dyn_states
                .iter()
                .any(|&ds| ds == VkDynamicState::VK_DYNAMIC_STATE_DEPTH_BOUNDS)
            {
                None
            } else {
                unsafe { info.pDepthStencilState.as_ref() }
                    .map(|db| db.minDepthBounds..db.maxDepthBounds)
            },
        };

        let layout = &*info.layout;
        let subpass = pass::Subpass {
            index: info.subpass as _,
            main_pass: &*info.renderPass,
        };

        let flags = {
            let mut flags = pso::PipelineCreationFlags::empty();

            if info.flags
                & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT as u32
                != 0
            {
                flags |= pso::PipelineCreationFlags::DISABLE_OPTIMIZATION;
            }
            if info.flags
                & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT as u32
                != 0
            {
                flags |= pso::PipelineCreationFlags::ALLOW_DERIVATIVES;
            }

            flags
        };

        let parent = {
            let is_derivative = info.flags
                & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DERIVATIVE_BIT as u32
                != 0;

            if let Some(base_pso) = info.basePipelineHandle.as_ref() {
                match *base_pso {
                    Pipeline::Graphics(ref pso) => pso::BasePipeline::Pipeline(pso),
                    Pipeline::Compute(_) => {
                        panic!("Base pipeline handle must be a graphics pipeline")
                    }
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

    let pipelines = unsafe {
        gpu.device
            .create_graphics_pipelines(descs, pipelineCache.as_ref())
    };
    let out_pipelines = unsafe { slice::from_raw_parts_mut(pPipelines, infos.len()) };

    if pipelines.iter().any(|p| p.is_err()) {
        for pipeline in pipelines {
            if let Err(e) = pipeline {
                error!("{:?}", e);
            }
        }
        for op in out_pipelines {
            *op = Handle::null();
        }
        VkResult::VK_ERROR_INCOMPATIBLE_DRIVER
    } else {
        for (op, raw) in out_pipelines.iter_mut().zip(pipelines) {
            *op = Handle::new(Pipeline::Graphics(raw.unwrap()));
        }
        VkResult::VK_SUCCESS
    }
}
#[inline]
pub extern "C" fn gfxCreateComputePipelines(
    gpu: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkComputePipelineCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    let infos = unsafe { slice::from_raw_parts(pCreateInfos, createInfoCount as _) };

    // Collect all information which we will borrow later. Need to work around
    // the borrow checker here.
    let mut spec_constants = Vec::new();
    let mut spec_data = Vec::new();

    // Collect all information which we will borrow later. Need to work around
    // the borrow checker here.
    for info in infos {
        if let Some(spec_info) = unsafe { info.stage.pSpecializationInfo.as_ref() } {
            let entries = unsafe {
                slice::from_raw_parts(spec_info.pMapEntries, spec_info.mapEntryCount as _)
            };
            for entry in entries {
                let base = spec_data.len() as u16 + entry.offset as u16;
                spec_constants.push(pso::SpecializationConstant {
                    id: entry.constantID,
                    range: base..base + (entry.size as u16),
                });
            }
            spec_data.extend_from_slice(unsafe {
                slice::from_raw_parts(spec_info.pData as *const u8, spec_info.dataSize)
            });
        }
    }

    let mut cur_specialization = 0;
    let descs = infos.iter().map(|info| {
        let name = unsafe { CStr::from_ptr(info.stage.pName) };
        let spec_count = unsafe {
            info.stage
                .pSpecializationInfo
                .as_ref()
                .map(|spec_info| spec_info.mapEntryCount as usize)
                .unwrap_or(0)
        };
        let shader = pso::EntryPoint {
            entry: name.to_str().unwrap(),
            module: &*info.stage.module,
            specialization: pso::Specialization {
                constants: Cow::from(
                    &spec_constants[cur_specialization..cur_specialization + spec_count],
                ),
                data: Cow::from(&spec_data),
            },
        };
        cur_specialization += spec_count;

        let layout = &*info.layout;
        let flags = {
            let mut flags = pso::PipelineCreationFlags::empty();

            if info.flags
                & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT as u32
                != 0
            {
                flags |= pso::PipelineCreationFlags::DISABLE_OPTIMIZATION;
            }
            if info.flags
                & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT as u32
                != 0
            {
                flags |= pso::PipelineCreationFlags::ALLOW_DERIVATIVES;
            }

            flags
        };

        let parent = {
            let is_derivative = info.flags
                & VkPipelineCreateFlagBits::VK_PIPELINE_CREATE_DERIVATIVE_BIT as u32
                != 0;

            if let Some(base_pso) = info.basePipelineHandle.as_ref() {
                match *base_pso {
                    Pipeline::Graphics(_) => {
                        panic!("Base pipeline handle must be a compute pipeline")
                    }
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
    });

    let pipelines = unsafe {
        gpu.device
            .create_compute_pipelines(descs, pipelineCache.as_ref())
    };
    let out_pipelines = unsafe { slice::from_raw_parts_mut(pPipelines, infos.len()) };

    if pipelines.iter().any(|p| p.is_err()) {
        for pipeline in pipelines {
            if let Err(e) = pipeline {
                error!("{:?}", e);
            }
        }
        for op in out_pipelines {
            *op = Handle::null();
        }
        VkResult::VK_ERROR_INCOMPATIBLE_DRIVER
    } else {
        for (op, raw) in out_pipelines.iter_mut().zip(pipelines) {
            *op = Handle::new(Pipeline::Compute(raw.unwrap()));
        }
        VkResult::VK_SUCCESS
    }
}
#[inline]
pub extern "C" fn gfxDestroyPipeline(
    gpu: VkDevice,
    pipeline: VkPipeline,
    _pAllocator: *const VkAllocationCallbacks,
) {
    match pipeline.unbox() {
        Some(Pipeline::Graphics(pipeline)) => unsafe {
            gpu.device.destroy_graphics_pipeline(pipeline)
        },
        Some(Pipeline::Compute(pipeline)) => unsafe {
            gpu.device.destroy_compute_pipeline(pipeline)
        },
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
    let set_layouts = unsafe { slice::from_raw_parts(info.pSetLayouts, info.setLayoutCount as _) };
    let push_constants = unsafe {
        slice::from_raw_parts(info.pPushConstantRanges, info.pushConstantRangeCount as _)
    };

    let layouts = set_layouts.iter().map(|layout| &**layout);

    let ranges = push_constants.iter().map(|constant| {
        let stages = conv::map_stage_flags(constant.stageFlags);
        (stages, constant.offset..constant.offset + constant.size)
    });

    let pipeline_layout = match unsafe { gpu.device.create_pipeline_layout(layouts, ranges) } {
        Ok(pipeline) => pipeline,
        Err(oom) => return map_oom(oom),
    };

    unsafe {
        *pPipelineLayout = Handle::new(pipeline_layout);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyPipelineLayout(
    gpu: VkDevice,
    pipelineLayout: VkPipelineLayout,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(layout) = pipelineLayout.unbox() {
        unsafe {
            gpu.device.destroy_pipeline_layout(layout);
        }
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
    let gfx_info = hal::image::SamplerDesc {
        min_filter: conv::map_filter(info.minFilter),
        mag_filter: conv::map_filter(info.magFilter),
        mip_filter: conv::map_mipmap_filter(info.mipmapMode),
        wrap_mode: (
            conv::map_wrap_mode(info.addressModeU),
            conv::map_wrap_mode(info.addressModeV),
            conv::map_wrap_mode(info.addressModeW),
        ),
        lod_bias: hal::image::Lod(info.mipLodBias),
        lod_range: hal::image::Lod(info.minLod)..hal::image::Lod(info.maxLod),
        comparison: if info.compareEnable == VK_TRUE {
            Some(conv::map_compare_op(info.compareOp))
        } else {
            None
        },
        border: [0.0; 4].into(), // TODO
        normalized: info.unnormalizedCoordinates == VK_FALSE,
        anisotropy_clamp: if info.anisotropyEnable == VK_TRUE {
            Some(info.maxAnisotropy as _)
        } else {
            None
        },
    };
    let sampler = match unsafe { gpu.device.create_sampler(&gfx_info) } {
        Ok(s) => s,
        Err(alloc) => return map_alloc_error(alloc),
    };
    unsafe {
        *pSampler = Handle::new(sampler);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroySampler(
    gpu: VkDevice,
    sampler: VkSampler,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(sam) = sampler.unbox() {
        unsafe {
            gpu.device.destroy_sampler(sam);
        }
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
    let layout_bindings = unsafe { slice::from_raw_parts(info.pBindings, info.bindingCount as _) };

    let sampler_iter = layout_bindings.iter().flat_map(|binding| {
        if binding.pImmutableSamplers.is_null() {
            (&[]).into_iter().cloned()
        } else {
            let slice = unsafe {
                slice::from_raw_parts(binding.pImmutableSamplers, binding.descriptorCount as _)
            };
            slice.iter().cloned()
        }
    });

    let bindings = layout_bindings
        .iter()
        .map(|binding| pso::DescriptorSetLayoutBinding {
            binding: binding.binding,
            ty: conv::map_descriptor_type(binding.descriptorType),
            count: binding.descriptorCount as _,
            stage_flags: conv::map_stage_flags(binding.stageFlags),
            immutable_samplers: !binding.pImmutableSamplers.is_null(),
        });

    let set_layout = match unsafe {
        gpu.device
            .create_descriptor_set_layout(bindings, sampler_iter)
    } {
        Ok(sl) => sl,
        Err(oom) => return map_oom(oom),
    };

    unsafe {
        *pSetLayout = Handle::new(set_layout);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyDescriptorSetLayout(
    gpu: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(layout) = descriptorSetLayout.unbox() {
        unsafe {
            gpu.device.destroy_descriptor_set_layout(layout);
        }
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
    let max_sets = info.maxSets as usize;

    let pool_sizes = unsafe { slice::from_raw_parts(info.pPoolSizes, info.poolSizeCount as _) };

    let ranges = pool_sizes.iter().map(|pool| pso::DescriptorRangeDesc {
        ty: conv::map_descriptor_type(pool.type_),
        count: pool.descriptorCount as _,
    });

    let pool = super::DescriptorPool {
        raw: match unsafe {
            gpu.device.create_descriptor_pool(
                max_sets,
                ranges,
                pso::DescriptorPoolCreateFlags::from_bits_truncate(info.flags),
            )
        } {
            Ok(pool) => pool,
            Err(oom) => return map_oom(oom),
        },
        temp_sets: Vec::with_capacity(max_sets),
        set_handles: if info.flags
            & VkDescriptorPoolCreateFlagBits::VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT
                as u32
            != 0
        {
            None
        } else {
            Some(Vec::with_capacity(max_sets))
        },
    };

    unsafe {
        *pDescriptorPool = Handle::new(pool);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyDescriptorPool(
    gpu: VkDevice,
    descriptorPool: VkDescriptorPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(pool) = descriptorPool.unbox() {
        unsafe {
            gpu.device.destroy_descriptor_pool(pool.raw);
        }
        if let Some(sets) = pool.set_handles {
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
    unsafe {
        descriptorPool.raw.reset();
    }
    if let Some(ref mut sets) = descriptorPool.set_handles {
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
    let super::DescriptorPool {
        ref mut raw,
        ref mut temp_sets,
        ref mut set_handles,
    } = *info.descriptorPool;

    let out_sets =
        unsafe { slice::from_raw_parts_mut(pDescriptorSets, info.descriptorSetCount as _) };
    let set_layouts =
        unsafe { slice::from_raw_parts(info.pSetLayouts, info.descriptorSetCount as _) };
    let layouts = set_layouts.iter().map(|layout| &**layout);

    match unsafe { raw.allocate(layouts, temp_sets) } {
        Ok(()) => {
            assert_eq!(temp_sets.len(), info.descriptorSetCount as usize);
            for (set, raw_set) in out_sets.iter_mut().zip(temp_sets.drain(..)) {
                *set = Handle::new(raw_set);
            }
            if let Some(ref mut local_sets) = set_handles {
                local_sets.extend_from_slice(out_sets);
            }
            VkResult::VK_SUCCESS
        }
        Err(e) => {
            assert!(temp_sets.is_empty());
            for set in out_sets.iter_mut() {
                *set = Handle::null();
            }
            error!("{:?}", e);
            match e {
                pso::AllocationError::OutOfMemory(oom) => map_oom(oom),
                pso::AllocationError::OutOfPoolMemory => VkResult::VK_ERROR_OUT_OF_POOL_MEMORY_KHR,
                pso::AllocationError::IncompatibleLayout => VkResult::VK_ERROR_DEVICE_LOST,
                pso::AllocationError::FragmentedPool => VkResult::VK_ERROR_FRAGMENTED_POOL,
            }
        }
    }
}
#[inline]
pub extern "C" fn gfxFreeDescriptorSets(
    _device: VkDevice,
    mut descriptorPool: VkDescriptorPool,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
) -> VkResult {
    let descriptor_sets =
        unsafe { slice::from_raw_parts(pDescriptorSets, descriptorSetCount as _) };
    assert!(descriptorPool.set_handles.is_none());

    let sets = descriptor_sets.into_iter().filter_map(|set| set.unbox());

    unsafe {
        descriptorPool.raw.free(sets);
    }

    VkResult::VK_SUCCESS
}

struct DescriptorIter<'a> {
    ty: pso::DescriptorType,
    image_infos: slice::Iter<'a, VkDescriptorImageInfo>,
    buffer_infos: slice::Iter<'a, VkDescriptorBufferInfo>,
    texel_buffer_views: slice::Iter<'a, VkBufferView>,
}
impl<'a> Iterator for DescriptorIter<'a> {
    type Item = pso::Descriptor<'a, B>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.ty {
            pso::DescriptorType::Sampler => self
                .image_infos
                .next()
                .map(|image| pso::Descriptor::Sampler(&*image.sampler)),

            pso::DescriptorType::Image {
                ty: pso::ImageDescriptorType::Sampled { with_sampler: true },
            } => self.image_infos.next().map(|image| {
                // It is valid for the sampler to be NULL in case the descriptor is
                // actually associated with an immutable sampler.
                // It's still bad to try to derefence it, even theough the implementation
                // will not try to use the value. (TODO: make this nicer)
                if image.sampler != Handle::null() {
                    pso::Descriptor::CombinedImageSampler(
                        image.imageView.to_native().unwrap(),
                        conv::map_image_layout(image.imageLayout),
                        &*image.sampler,
                    )
                } else {
                    pso::Descriptor::Image(
                        image.imageView.to_native().unwrap(),
                        conv::map_image_layout(image.imageLayout),
                    )
                }
            }),

            pso::DescriptorType::InputAttachment | pso::DescriptorType::Image { .. } => {
                self.image_infos.next().map(|image| {
                    pso::Descriptor::Image(
                        image.imageView.to_native().unwrap(),
                        conv::map_image_layout(image.imageLayout),
                    )
                })
            }

            pso::DescriptorType::Buffer { format, .. } => {
                match format {
                    pso::BufferDescriptorFormat::Texel => {
                        self.texel_buffer_views
                            .next()
                            .map(|view| pso::Descriptor::TexelBuffer(&**view))
                    }
                    pso::BufferDescriptorFormat::Structured { .. } => {
                        self.buffer_infos.next().map(|buffer| {
                            let range = hal::buffer::SubRange {
                                offset: buffer.offset,
                                size: if buffer.range as i32 == VK_WHOLE_SIZE { None } else { Some(buffer.range) },
                            };
                            // Non-sparse buffer need to be bound to device memory.
                            pso::Descriptor::Buffer(&*buffer.buffer, range)
                        })
                    }
                }
            }
        }
    }
}

#[inline]
pub extern "C" fn gfxUpdateDescriptorSets(
    gpu: VkDevice,
    descriptorWriteCount: u32,
    pDescriptorWrites: *const VkWriteDescriptorSet,
    descriptorCopyCount: u32,
    pDescriptorCopies: *const VkCopyDescriptorSet,
) {
    let write_infos =
        unsafe { slice::from_raw_parts(pDescriptorWrites, descriptorWriteCount as _) };
    let writes = write_infos.iter().map(|write| {
        let descriptors = DescriptorIter {
            ty: conv::map_descriptor_type(write.descriptorType),
            image_infos: unsafe {
                slice::from_raw_parts(write.pImageInfo, write.descriptorCount as _)
            }
            .iter(),
            buffer_infos: unsafe {
                slice::from_raw_parts(write.pBufferInfo, write.descriptorCount as _)
            }
            .iter(),
            texel_buffer_views: unsafe {
                slice::from_raw_parts(write.pTexelBufferView, write.descriptorCount as _)
            }
            .iter(),
        };
        pso::DescriptorSetWrite {
            set: &*write.dstSet,
            binding: write.dstBinding,
            array_offset: write.dstArrayElement as _,
            descriptors,
        }
    });

    let copies = unsafe { slice::from_raw_parts(pDescriptorCopies, descriptorCopyCount as _) }
        .iter()
        .map(|copy| pso::DescriptorSetCopy {
            src_set: &*copy.srcSet,
            src_binding: copy.srcBinding,
            src_array_offset: copy.srcArrayElement as _,
            dst_set: &*copy.dstSet,
            dst_binding: copy.dstBinding,
            dst_array_offset: copy.dstArrayElement as _,
            count: copy.descriptorCount as _,
        });

    unsafe {
        gpu.device.write_descriptor_sets(writes);
        gpu.device.copy_descriptor_sets(copies);
    }
}
#[inline]
pub extern "C" fn gfxCreateFramebuffer(
    gpu: VkDevice,
    pCreateInfo: *const VkFramebufferCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pFramebuffer: *mut VkFramebuffer,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    let extent = hal::image::Extent {
        width: info.width,
        height: info.height,
        depth: info.layers,
    };

    let attachments_slice =
        unsafe { slice::from_raw_parts(info.pAttachments, info.attachmentCount as _) };
    let framebuffer = if attachments_slice.iter().any(|attachment| match **attachment {
        ImageView::Native(_) => false,
        ImageView::SwapchainFrame { .. } => true,
    }) {
        Framebuffer::Lazy {
            extent,
            views: attachments_slice.to_vec(),
        }
    } else {
        let attachments = attachments_slice
            .iter()
            .map(|attachment| attachment.to_native().unwrap());
        Framebuffer::Native(unsafe {
            gpu
                .device
                .create_framebuffer(&*info.renderPass, attachments, extent)
                .unwrap()
        })
    };

    unsafe {
        *pFramebuffer = Handle::new(framebuffer)
    };
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyFramebuffer(
    gpu: VkDevice,
    framebuffer: VkFramebuffer,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(fbo) = framebuffer.unbox() {
        match fbo {
            Framebuffer::Native(raw) => unsafe {
                gpu.device.destroy_framebuffer(raw);
            },
            Framebuffer::Lazy { .. } => {}
        }
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
    let raw_attachments =
        unsafe { slice::from_raw_parts(info.pAttachments, info.attachmentCount as _) };
    let attachments = raw_attachments.into_iter().map(|attachment| {
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
            layouts: initial_layout..final_layout,
        }
    });

    // Subpass descriptions
    let subpasses_raw = unsafe { slice::from_raw_parts(info.pSubpasses, info.subpassCount as _) };

    // Store all attachment references, referenced by the subpasses.
    let mut attachment_refs = Vec::with_capacity(subpasses_raw.len());
    struct AttachmentRefs {
        input: Vec<pass::AttachmentRef>,
        color: Vec<pass::AttachmentRef>,
        resolve: Vec<pass::AttachmentRef>,
        depth_stencil: Option<pass::AttachmentRef>,
        preserve: Vec<usize>,
    }

    fn map_attachment_ref(att_ref: &VkAttachmentReference) -> pass::AttachmentRef {
        (
            att_ref.attachment as _,
            conv::map_image_layout(att_ref.layout),
        )
    }

    for subpass in subpasses_raw {
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
                slice::from_raw_parts(
                    subpass.pResolveAttachments,
                    subpass.colorAttachmentCount as _,
                )
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
                .filter(|ds| ds.0 as c_int != VK_ATTACHMENT_UNUSED)
        };

        let preserve = unsafe {
            slice::from_raw_parts(
                subpass.pPreserveAttachments,
                subpass.preserveAttachmentCount as _,
            )
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
        });

    // Subpass dependencies
    let dependencies =
        unsafe { slice::from_raw_parts(info.pDependencies, info.dependencyCount as _) };

    fn map_subpass_ref(subpass: u32) -> Option<pass::SubpassId> {
        if subpass == VK_SUBPASS_EXTERNAL as u32 {
            None
        } else {
            Some(subpass as _)
        }
    }

    let dependencies = dependencies.into_iter().map(|dependency| {
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
            passes: src_pass..dst_pass,
            stages: src_stage..dst_stage,
            accesses: src_access..dst_access,
            flags: conv::map_dependency_flags(dependency.dependencyFlags),
        }
    });

    let render_pass = match unsafe {
        gpu.device
            .create_render_pass(attachments, subpasses, dependencies)
    } {
        Ok(raw) => raw,
        Err(oom) => return map_oom(oom),
    };

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
        unsafe {
            gpu.device.destroy_render_pass(rp);
        }
    }
}
#[inline]
pub extern "C" fn gfxGetRenderAreaGranularity(
    _gpu: VkDevice,
    _renderPass: VkRenderPass,
    pGranularity: *mut VkExtent2D,
) {
    let granularity = VkExtent2D {
        width: 1,
        height: 1,
    }; //TODO?
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
        pool: match unsafe { gpu.device.create_command_pool(family, flags) } {
            Ok(pool) => pool,
            Err(oom) => return map_oom(oom),
        },
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
        unsafe {
            gpu.device.destroy_command_pool(cp.pool);
        }
    }
}

#[inline]
pub extern "C" fn gfxResetCommandPool(
    _gpu: VkDevice,
    mut commandPool: VkCommandPool,
    flags: VkCommandPoolResetFlags,
) -> VkResult {
    let release = (flags
        & VkCommandPoolResetFlagBits::VK_COMMAND_POOL_RESET_RELEASE_RESOURCES_BIT as u32)
        != 0;
    unsafe {
        commandPool.pool.reset(release);
    }
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
        VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY => com::Level::Primary,
        VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_SECONDARY => com::Level::Secondary,
        level => panic!("Unexpected command buffer lvel: {:?}", level),
    };

    let output =
        unsafe { slice::from_raw_parts_mut(pCommandBuffers, info.commandBufferCount as usize) };
    for out in output.iter_mut() {
        let cmd_buf = unsafe { info.commandPool.pool.allocate_one(level) };
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
    let slice = unsafe { slice::from_raw_parts(pCommandBuffers, commandBufferCount as _) };
    commandPool.buffers.retain(|buf| !slice.contains(buf));

    let buffers = slice.iter().filter_map(|buffer| buffer.unbox());
    unsafe {
        commandPool.pool.free(buffers);
    }
}

#[inline]
pub extern "C" fn gfxBeginCommandBuffer(
    mut commandBuffer: VkCommandBuffer,
    pBeginInfo: *const VkCommandBufferBeginInfo,
) -> VkResult {
    let info = unsafe { &*pBeginInfo };
    let inheritance = match unsafe { info.pInheritanceInfo.as_ref() } {
        Some(ii) => com::CommandBufferInheritanceInfo {
            subpass: ii.renderPass.as_ref().map(|rp| pass::Subpass {
                main_pass: &*rp,
                index: ii.subpass as _,
            }),
            framebuffer: ii.framebuffer.as_ref().map(|fbo| fbo.resolve(ii.renderPass)),
            occlusion_query_enable: ii.occlusionQueryEnable != VK_FALSE,
            occlusion_query_flags: conv::map_query_control(ii.queryFlags),
            pipeline_statistics: conv::map_pipeline_statistics(ii.pipelineStatistics),
        },
        None => com::CommandBufferInheritanceInfo::default(),
    };
    unsafe {
        commandBuffer.begin(conv::map_cmd_buffer_usage(info.flags), inheritance);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxEndCommandBuffer(mut commandBuffer: VkCommandBuffer) -> VkResult {
    unsafe {
        commandBuffer.finish();
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxResetCommandBuffer(
    mut commandBuffer: VkCommandBuffer,
    flags: VkCommandBufferResetFlags,
) -> VkResult {
    let release_resources = flags
        & VkCommandBufferResetFlagBits::VK_COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT as u32
        != 0;
    unsafe {
        commandBuffer.reset(release_resources);
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxCmdBindPipeline(
    mut commandBuffer: VkCommandBuffer,
    _pipelineBindPoint: VkPipelineBindPoint, // ignore, needs to match by spec
    pipeline: VkPipeline,
) {
    match *pipeline {
        Pipeline::Graphics(ref pipeline) => unsafe {
            commandBuffer.bind_graphics_pipeline(pipeline)
        },
        Pipeline::Compute(ref pipeline) => unsafe { commandBuffer.bind_compute_pipeline(pipeline) },
    }
}
#[inline]
pub extern "C" fn gfxCmdSetViewport(
    mut commandBuffer: VkCommandBuffer,
    firstViewport: u32,
    viewportCount: u32,
    pViewports: *const VkViewport,
) {
    unsafe {
        let viewports = slice::from_raw_parts(pViewports, viewportCount as _)
            .into_iter()
            .map(conv::map_viewport);
        commandBuffer.set_viewports(firstViewport, viewports);
    }
}
#[inline]
pub extern "C" fn gfxCmdSetScissor(
    mut commandBuffer: VkCommandBuffer,
    firstScissor: u32,
    scissorCount: u32,
    pScissors: *const VkRect2D,
) {
    unsafe {
        let scissors = slice::from_raw_parts(pScissors, scissorCount as _)
            .into_iter()
            .map(conv::map_rect);
        commandBuffer.set_scissors(firstScissor, scissors);
    }
}
#[inline]
pub extern "C" fn gfxCmdSetLineWidth(mut commandBuffer: VkCommandBuffer, lineWidth: f32) {
    unsafe {
        commandBuffer.set_line_width(lineWidth);
    }
}
#[inline]
pub extern "C" fn gfxCmdSetDepthBias(
    mut commandBuffer: VkCommandBuffer,
    depthBiasConstantFactor: f32,
    depthBiasClamp: f32,
    depthBiasSlopeFactor: f32,
) {
    unsafe {
        commandBuffer.set_depth_bias(pso::DepthBias {
            const_factor: depthBiasConstantFactor,
            clamp: depthBiasClamp,
            slope_factor: depthBiasSlopeFactor,
        });
    }
}
#[inline]
pub extern "C" fn gfxCmdSetBlendConstants(
    mut commandBuffer: VkCommandBuffer,
    blendConstants: *const f32,
) {
    unsafe {
        let value = *(blendConstants as *const pso::ColorValue);
        commandBuffer.set_blend_constants(value);
    }
}
#[inline]
pub extern "C" fn gfxCmdSetDepthBounds(
    mut commandBuffer: VkCommandBuffer,
    minDepthBounds: f32,
    maxDepthBounds: f32,
) {
    unsafe {
        commandBuffer.set_depth_bounds(minDepthBounds..maxDepthBounds);
    }
}
#[inline]
pub extern "C" fn gfxCmdSetStencilCompareMask(
    mut commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    compareMask: u32,
) {
    unsafe {
        commandBuffer.set_stencil_read_mask(conv::map_stencil_face(faceMask), compareMask);
    }
}
#[inline]
pub extern "C" fn gfxCmdSetStencilWriteMask(
    mut commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    writeMask: u32,
) {
    unsafe {
        commandBuffer.set_stencil_write_mask(conv::map_stencil_face(faceMask), writeMask);
    }
}
#[inline]
pub extern "C" fn gfxCmdSetStencilReference(
    mut commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    reference: u32,
) {
    unsafe {
        commandBuffer.set_stencil_reference(conv::map_stencil_face(faceMask), reference);
    }
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
    let offsets = unsafe { slice::from_raw_parts(pDynamicOffsets, dynamicOffsetCount as _) };

    match pipelineBindPoint {
        VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS => unsafe {
            commandBuffer.bind_graphics_descriptor_sets(
                &*layout,
                firstSet as _,
                descriptor_sets,
                offsets,
            );
        },
        VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_COMPUTE => unsafe {
            commandBuffer.bind_compute_descriptor_sets(
                &*layout,
                firstSet as _,
                descriptor_sets,
                offsets,
            );
        },
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
    unsafe {
        commandBuffer.bind_index_buffer(IndexBufferView {
            buffer: &*buffer,
            range: hal::buffer::SubRange { offset, size: None },
            index_type: conv::map_index_type(indexType),
        });
    }
}

#[inline]
pub extern "C" fn gfxCmdBindVertexBuffers(
    mut commandBuffer: VkCommandBuffer,
    firstBinding: u32,
    bindingCount: u32,
    pBuffers: *const VkBuffer,
    pOffsets: *const VkDeviceSize,
) {
    let buffers = unsafe { slice::from_raw_parts(pBuffers, bindingCount as _) };
    let offsets = unsafe { slice::from_raw_parts(pOffsets, bindingCount as _) };

    let views = buffers
        .into_iter()
        .zip(offsets)
        .map(|(buffer, &offset)| (*buffer, hal::buffer::SubRange { offset, size: None }));

    unsafe {
        commandBuffer.bind_vertex_buffers(firstBinding, views);
    }
}
#[inline]
pub extern "C" fn gfxCmdDraw(
    mut commandBuffer: VkCommandBuffer,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    unsafe {
        commandBuffer.draw(
            firstVertex..firstVertex + vertexCount,
            firstInstance..firstInstance + instanceCount,
        );
    }
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
    unsafe {
        commandBuffer.draw_indexed(
            firstIndex..firstIndex + indexCount,
            vertexOffset,
            firstInstance..firstInstance + instanceCount,
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdDrawIndirect(
    mut commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    unsafe {
        commandBuffer.draw_indirect(&*buffer, offset, drawCount, stride);
    }
}
#[inline]
pub extern "C" fn gfxCmdDrawIndexedIndirect(
    mut commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    unsafe {
        commandBuffer.draw_indexed_indirect(&*buffer, offset, drawCount, stride);
    }
}
#[inline]
pub extern "C" fn gfxCmdDispatch(
    mut commandBuffer: VkCommandBuffer,
    groupCountX: u32,
    groupCountY: u32,
    groupCountZ: u32,
) {
    unsafe {
        commandBuffer.dispatch([groupCountX, groupCountY, groupCountZ]);
    }
}
#[inline]
pub extern "C" fn gfxCmdDispatchIndirect(
    mut commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
) {
    unsafe {
        commandBuffer.dispatch_indirect(&*buffer, offset);
    }
}
#[inline]
pub extern "C" fn gfxCmdCopyBuffer(
    mut commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferCopy,
) {
    let regions = unsafe { slice::from_raw_parts(pRegions, regionCount as _) }
        .iter()
        .map(|r| com::BufferCopy {
            src: r.srcOffset,
            dst: r.dstOffset,
            size: r.size,
        });

    unsafe {
        commandBuffer.copy_buffer(&*srcBuffer, &*dstBuffer, regions);
    }
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
    let src = match srcImage.to_native() {
        Ok(img) => img,
        Err(_) => {
            warn!("Unable to copy from a swapchain image!");
            return;
        }
    };
    let dst = match dstImage.to_native() {
        Ok(img) => img,
        Err(_) => {
            warn!("Unable to copy into a swapchain image!");
            return;
        }
    };

    let regions = unsafe { slice::from_raw_parts(pRegions, regionCount as _) }
        .iter()
        .map(|r| com::ImageCopy {
            src_subresource: src.map_subresource_layers(r.srcSubresource),
            src_offset: conv::map_offset(r.srcOffset),
            dst_subresource: dst.map_subresource_layers(r.dstSubresource),
            dst_offset: conv::map_offset(r.dstOffset),
            extent: conv::map_extent(r.extent),
        });

    unsafe {
        commandBuffer.copy_image(
            src.raw,
            conv::map_image_layout(srcImageLayout),
            dst.raw,
            conv::map_image_layout(dstImageLayout),
            regions,
        );
    }
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
    let src = match srcImage.to_native() {
        Ok(img) => img,
        Err(_) => {
            warn!("Unable to copy from a swapchain image!");
            return;
        }
    };
    let dst = match dstImage.to_native() {
        Ok(img) => img,
        Err(_) => {
            warn!("Unable to copy into a swapchain image!");
            return;
        }
    };

    let regions = unsafe { slice::from_raw_parts(pRegions, regionCount as _) }
        .iter()
        .map(|r| com::ImageBlit {
            src_subresource: src.map_subresource_layers(r.srcSubresource),
            src_bounds: conv::map_offset(r.srcOffsets[0])..conv::map_offset(r.srcOffsets[1]),
            dst_subresource: dst.map_subresource_layers(r.dstSubresource),
            dst_bounds: conv::map_offset(r.dstOffsets[0])..conv::map_offset(r.dstOffsets[1]),
        });

    unsafe {
        commandBuffer.blit_image(
            src.raw,
            conv::map_image_layout(srcImageLayout),
            dst.raw,
            conv::map_image_layout(dstImageLayout),
            conv::map_filter(filter),
            regions,
        );
    }
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
    let dst = dstImage.to_native().unwrap();

    let regions = unsafe { slice::from_raw_parts(pRegions, regionCount as _) }
        .iter()
        .map(|r| com::BufferImageCopy {
            buffer_offset: r.bufferOffset,
            buffer_width: r.bufferRowLength,
            buffer_height: r.bufferImageHeight,
            image_layers: dst.map_subresource_layers(r.imageSubresource),
            image_offset: conv::map_offset(r.imageOffset),
            image_extent: conv::map_extent(r.imageExtent),
        });

    unsafe {
        commandBuffer.copy_buffer_to_image(
            &*srcBuffer,
            dst.raw,
            conv::map_image_layout(dstImageLayout),
            regions,
        );
    }
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
    let src = srcImage.to_native().unwrap();

    let regions = unsafe { slice::from_raw_parts(pRegions, regionCount as _) }
        .iter()
        .map(|r| com::BufferImageCopy {
            buffer_offset: r.bufferOffset,
            buffer_width: r.bufferRowLength,
            buffer_height: r.bufferImageHeight,
            image_layers: src.map_subresource_layers(r.imageSubresource),
            image_offset: conv::map_offset(r.imageOffset),
            image_extent: conv::map_extent(r.imageExtent),
        });

    unsafe {
        commandBuffer.copy_image_to_buffer(
            src.raw,
            conv::map_image_layout(srcImageLayout),
            &*dstBuffer,
            regions,
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdUpdateBuffer(
    mut commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    dataSize: VkDeviceSize,
    pData: *const c_void,
) {
    unsafe {
        commandBuffer.update_buffer(
            &*dstBuffer,
            dstOffset,
            slice::from_raw_parts(pData as _, dataSize as _),
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdFillBuffer(
    mut commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    size: VkDeviceSize,
    data: u32,
) {
    let range = hal::buffer::SubRange {
        offset: dstOffset,
        size: if size == VK_WHOLE_SIZE as VkDeviceSize { None } else { Some(size) },
    };
    unsafe {
        commandBuffer.fill_buffer(&*dstBuffer, range, data);
    }
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
    let img = match image.to_native() {
        Ok(img) => img,
        Err(_) => {
            warn!("Unable to clear a swapchain image!");
            return;
        }
    };
    let subresource_ranges = unsafe { slice::from_raw_parts(pRanges, rangeCount as _) }

        .iter()
        .map(|&range| img.map_subresource_range(range));

    unsafe {
        commandBuffer.clear_image(
            img.raw,
            conv::map_image_layout(imageLayout),
            com::ClearValue {
                color: mem::transmute(*pColor),
            },
            subresource_ranges,
        );
    }
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
    let img = image.to_native().unwrap();
    let subresource_ranges = unsafe { slice::from_raw_parts(pRanges, rangeCount as _) }
        .iter()
        .map(|&range| img.map_subresource_range(range));

    unsafe {
        commandBuffer.clear_image(
            img.raw,
            conv::map_image_layout(imageLayout),
            com::ClearValue {
                depth_stencil: mem::transmute(*pDepthStencil),
            },
            subresource_ranges,
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdClearAttachments(
    mut commandBuffer: VkCommandBuffer,
    attachmentCount: u32,
    pAttachments: *const VkClearAttachment,
    rectCount: u32,
    pRects: *const VkClearRect,
) {
    let attachments = unsafe { slice::from_raw_parts(pAttachments, attachmentCount as _) }
        .iter()
        .map(|at| {
            use crate::VkImageAspectFlagBits::*;
            if at.aspectMask & VK_IMAGE_ASPECT_COLOR_BIT as u32 != 0 {
                com::AttachmentClear::Color {
                    index: at.colorAttachment as _,
                    value: com::ClearColor {
                        float32: unsafe { at.clearValue.color.float32 },
                    }, //TODO?
                }
            } else {
                com::AttachmentClear::DepthStencil {
                    depth: if at.aspectMask & VK_IMAGE_ASPECT_DEPTH_BIT as u32 != 0 {
                        Some(unsafe { at.clearValue.depthStencil.depth })
                    } else {
                        None
                    },
                    stencil: if at.aspectMask & VK_IMAGE_ASPECT_STENCIL_BIT as u32 != 0 {
                        Some(unsafe { at.clearValue.depthStencil.stencil })
                    } else {
                        None
                    },
                }
            }
        });

    let rects = unsafe { slice::from_raw_parts(pRects, rectCount as _) }
        .iter()
        .map(conv::map_clear_rect);

    unsafe {
        commandBuffer.clear_attachments(attachments, rects);
    }
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
    let src = srcImage.to_native().unwrap();
    let dst = dstImage.to_native().unwrap();

    let regions = unsafe { slice::from_raw_parts(pRegions, regionCount as _) }
        .iter()
        .cloned()
        .map(|resolve| com::ImageResolve {
            src_subresource: src.map_subresource_layers(resolve.srcSubresource),
            src_offset: conv::map_offset(resolve.srcOffset),
            dst_subresource: dst.map_subresource_layers(resolve.dstSubresource),
            dst_offset: conv::map_offset(resolve.dstOffset),
            extent: conv::map_extent(resolve.extent),
        });

    unsafe {
        commandBuffer.resolve_image(
            src.raw,
            conv::map_image_layout(srcImageLayout),
            dst.raw,
            conv::map_image_layout(dstImageLayout),
            regions,
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdSetEvent(
    mut commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    unsafe {
        commandBuffer.set_event(&event, conv::map_pipeline_stage_flags(stageMask));
    }
}
#[inline]
pub extern "C" fn gfxCmdResetEvent(
    mut commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    unsafe {
        commandBuffer.reset_event(&event, conv::map_pipeline_stage_flags(stageMask));
    }
}

fn make_barriers<'a>(
    raw_globals: &'a [VkMemoryBarrier],
    raw_buffers: &'a [VkBufferMemoryBarrier],
    raw_images: &'a [VkImageMemoryBarrier],
) -> impl Iterator<Item = memory::Barrier<'a, back::Backend>> {
    let globals = raw_globals.iter().flat_map(|b| {
        let buf =
            conv::map_buffer_access(b.srcAccessMask)..conv::map_buffer_access(b.dstAccessMask);
        let buf_bar = if !buf.start.is_empty() || !buf.end.is_empty() {
            Some(memory::Barrier::AllBuffers(buf))
        } else {
            None
        };
        let img = conv::map_image_access(b.srcAccessMask)..conv::map_image_access(b.dstAccessMask);
        let img_bar = if !img.start.is_empty() || !img.end.is_empty() {
            Some(memory::Barrier::AllImages(img))
        } else {
            None
        };
        buf_bar.into_iter().chain(img_bar)
    });

    let buffers = raw_buffers.iter().map(|b| memory::Barrier::Buffer {
        states: conv::map_buffer_access(b.srcAccessMask)..conv::map_buffer_access(b.dstAccessMask),
        target: &*b.buffer,
        families: None,
        range: hal::buffer::SubRange {
            offset: b.offset,
            size: if b.size as i32 == VK_WHOLE_SIZE { None } else { Some(b.size) },
        },
    });
    let images = raw_images.iter().map(|b| {
        let img = b.image.to_native().unwrap();
        let from = (
            conv::map_image_access(b.srcAccessMask),
            conv::map_image_layout(b.oldLayout),
        );
        let to = (
            conv::map_image_access(b.dstAccessMask),
            conv::map_image_layout(b.newLayout),
        );
        memory::Barrier::Image {
            states: from .. to,
            target: img.raw,
            range: img.map_subresource_range(b.subresourceRange),
            families: None,
        }
    });

    globals.chain(buffers).chain(images)
}

#[inline]
pub extern "C" fn gfxCmdWaitEvents(
    mut commandBuffer: VkCommandBuffer,
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
    let raw_globals = unsafe { slice::from_raw_parts(pMemoryBarriers, memoryBarrierCount as _) };
    let raw_buffers =
        unsafe { slice::from_raw_parts(pBufferMemoryBarriers, bufferMemoryBarrierCount as _) };
    let raw_images =
        unsafe { slice::from_raw_parts(pImageMemoryBarriers, imageMemoryBarrierCount as _) };

    let barriers = make_barriers(raw_globals, raw_buffers, raw_images);

    unsafe {
        commandBuffer.wait_events(
            slice::from_raw_parts(pEvents, eventCount as usize)
                .iter()
                .map(|ev| &**ev),
            conv::map_pipeline_stage_flags(srcStageMask)
                ..conv::map_pipeline_stage_flags(dstStageMask),
            barriers,
        );
    }
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
    let raw_globals = unsafe { slice::from_raw_parts(pMemoryBarriers, memoryBarrierCount as _) };
    let raw_buffers =
        unsafe { slice::from_raw_parts(pBufferMemoryBarriers, bufferMemoryBarrierCount as _) };
    let raw_images =
        unsafe { slice::from_raw_parts(pImageMemoryBarriers, imageMemoryBarrierCount as _) };

    let barriers = make_barriers(raw_globals, raw_buffers, raw_images);

    unsafe {
        commandBuffer.pipeline_barrier(
            conv::map_pipeline_stage_flags(srcStageMask)
                ..conv::map_pipeline_stage_flags(dstStageMask),
            memory::Dependencies::from_bits(dependencyFlags)
                .unwrap_or(memory::Dependencies::empty()),
            barriers,
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdBeginQuery(
    mut commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    query: u32,
    flags: VkQueryControlFlags,
) {
    let query = hal::query::Query {
        pool: &*queryPool,
        id: query,
    };
    unsafe {
        commandBuffer.begin_query(query, conv::map_query_control(flags));
    }
}
#[inline]
pub extern "C" fn gfxCmdEndQuery(
    mut commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    query: u32,
) {
    let query = hal::query::Query {
        pool: &*queryPool,
        id: query,
    };
    unsafe {
        commandBuffer.end_query(query);
    }
}
#[inline]
pub extern "C" fn gfxCmdResetQueryPool(
    mut commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
) {
    unsafe {
        commandBuffer.reset_query_pool(&*queryPool, firstQuery..firstQuery + queryCount);
    }
}
#[inline]
pub extern "C" fn gfxCmdWriteTimestamp(
    mut commandBuffer: VkCommandBuffer,
    pipelineStage: VkPipelineStageFlagBits,
    queryPool: VkQueryPool,
    query: u32,
) {
    let query = hal::query::Query {
        pool: &*queryPool,
        id: query,
    };
    unsafe {
        commandBuffer.write_timestamp(conv::map_pipeline_stage_flags(pipelineStage as u32), query);
    }
}
#[inline]
pub extern "C" fn gfxCmdCopyQueryPoolResults(
    mut commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    stride: VkDeviceSize,
    flags: VkQueryResultFlags,
) {
    unsafe {
        commandBuffer.copy_query_pool_results(
            &*queryPool,
            firstQuery..firstQuery + queryCount,
            &*dstBuffer,
            dstOffset,
            stride,
            conv::map_query_result(flags),
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdPushConstants(
    mut commandBuffer: VkCommandBuffer,
    layout: VkPipelineLayout,
    stageFlags: VkShaderStageFlags,
    offset: u32,
    size: u32,
    pValues: *const c_void,
) {
    assert_eq!(size % 4, 0);
    unsafe {
        let values = slice::from_raw_parts(pValues as *const u32, size as usize / 4);

        if stageFlags & VkShaderStageFlagBits::VK_SHADER_STAGE_COMPUTE_BIT as u32 != 0 {
            commandBuffer.push_compute_constants(&*layout, offset, values);
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
                mem::transmute::<_, com::ClearValue>(*cv)
            })
    };
    let contents = conv::map_subpass_contents(contents);
    let framebuffer = info.framebuffer.resolve(info.renderPass);

    unsafe {
        commandBuffer.begin_render_pass(
            &*info.renderPass,
            framebuffer,
            render_area,
            clear_values,
            contents,
        );
    }
}
#[inline]
pub extern "C" fn gfxCmdNextSubpass(
    mut commandBuffer: VkCommandBuffer,
    contents: VkSubpassContents,
) {
    unsafe {
        commandBuffer.next_subpass(conv::map_subpass_contents(contents));
    }
}
#[inline]
pub extern "C" fn gfxCmdEndRenderPass(mut commandBuffer: VkCommandBuffer) {
    unsafe {
        commandBuffer.end_render_pass();
    }
}
#[inline]
pub extern "C" fn gfxCmdExecuteCommands(
    mut commandBuffer: VkCommandBuffer,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    unsafe {
        commandBuffer.execute_commands(slice::from_raw_parts(
            pCommandBuffers,
            commandBufferCount as _,
        ));
    }
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
pub extern "C" fn gfxGetPhysicalDeviceSurfaceSupportKHR(
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
    let caps = surface.capabilities(&adapter.physical_device);

    let output = VkSurfaceCapabilitiesKHR {
        minImageCount: *caps.image_count.start(),
        maxImageCount: *caps.image_count.end(),
        currentExtent: match caps.current_extent {
            Some(extent) => conv::extent2d_from_hal(extent),
            None => VkExtent2D {
                width: !0,
                height: !0,
            },
        },
        minImageExtent: conv::extent2d_from_hal(*caps.extents.start()),
        maxImageExtent: conv::extent2d_from_hal(*caps.extents.end()),
        maxImageArrayLayers: caps.max_image_layers as _,
        supportedTransforms: VkSurfaceTransformFlagBitsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR
            as _,
        currentTransform: VkSurfaceTransformFlagBitsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        supportedCompositeAlpha: caps.composite_alpha_modes.bits(),
        // Ignoring `caps.usage` since we only work with the new swapchain model here.
        supportedUsageFlags: VkImageUsageFlagBits::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as _,
    };

    unsafe { *pSurfaceCapabilities = output };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxGetPhysicalDeviceSurfaceCapabilities2KHR(
    adapter: VkPhysicalDevice,
    pSurfaceInfo: *const VkPhysicalDeviceSurfaceInfo2KHR,
    pSurfaceCapabilities: *mut VkSurfaceCapabilities2KHR,
) -> VkResult {
    let surface = unsafe { (*pSurfaceInfo).surface };
    let mut ptr = pSurfaceCapabilities as *const VkStructureType;
    while !ptr.is_null() {
        ptr = match unsafe { *ptr } {
            VkStructureType::VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR => {
                let data = unsafe { (ptr as *mut VkSurfaceCapabilities2KHR).as_mut().unwrap() };
                gfxGetPhysicalDeviceSurfaceCapabilitiesKHR(adapter, surface, &mut data.surfaceCapabilities);
                data.pNext
            }
            other => {
                warn!("Unrecognized {:?}, skipping", other);
                unsafe {
                    (ptr as *const VkBaseStruct)
                        .as_ref()
                        .unwrap()
                }
                .pNext
            }
        } as *const VkStructureType;
    }
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
        .supported_formats(&adapter.physical_device)
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
pub extern "C" fn gfxGetPhysicalDeviceSurfaceFormats2KHR(
    adapter: VkPhysicalDevice,
    pSurfaceInfo: *const VkPhysicalDeviceSurfaceInfo2KHR,
    pSurfaceFormatCount: *mut u32,
    pSurfaceFormats: *mut VkSurfaceFormat2KHR,
) -> VkResult {
    let formats = unsafe { (*pSurfaceInfo).surface }
        .supported_formats(&adapter.physical_device)
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
            out.surfaceFormat = VkSurfaceFormatKHR {
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
    let present_modes = surface.capabilities(&adapter.physical_device).present_modes;

    let num_present_modes = present_modes.bits().count_ones();

    // If NULL, number of present modes is returned.
    if pPresentModes.is_null() {
        unsafe { *pPresentModeCount = num_present_modes };
        return VkResult::VK_SUCCESS;
    }

    let num_output = unsafe { *pPresentModeCount };
    let output = unsafe { slice::from_raw_parts_mut(pPresentModes, num_output as _) };
    let (code, count) = if num_output < num_present_modes {
        (VkResult::VK_INCOMPLETE, num_output)
    } else {
        (VkResult::VK_SUCCESS, num_present_modes)
    };

    let mut out_idx = 0;
    for i in 0..PresentMode::all().bits().count_ones() {
        let present_mode = PresentMode::from_bits_truncate(1 << i);
        if present_modes.contains(present_mode) {
            output[out_idx] = unsafe { mem::transmute(i) };
            out_idx += 1;
        }
    }

    unsafe { *pPresentModeCount = count };
    code
}

#[inline]
pub extern "C" fn gfxGetPhysicalDeviceWin32PresentationSupportKHR(
    _adapter: VkPhysicalDevice,
    _queueFamilyIndex: u32,
) -> VkBool32 {
    VK_TRUE
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

    if info.imageUsage != VkImageUsageFlagBits::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as _ {
        warn!("Unsupported swapchain usage: {:?}", info.imageUsage);
    }

    let config = hal::window::SwapchainConfig {
        present_mode: conv::map_present_mode(info.presentMode),
        composite_alpha_mode: conv::map_composite_alpha(info.compositeAlpha),
        format: conv::map_format(info.imageFormat).unwrap(),
        extent: conv::map_extent2d(info.imageExtent),
        image_count: info.minImageCount,
        image_layers: 1,
        image_usage: hal::image::Usage::COLOR_ATTACHMENT,
    };

    match unsafe {
        info.surface
            .as_mut()
            .unwrap()
            .configure_swapchain(&gpu.device, config)
    } {
        Ok(()) => {
            let swapchain = Swapchain {
                gpu,
                surface: info.surface,
                count: info.minImageCount as u8,
                current_index: 0,
                active: None,
                lazy_framebuffers: Vec::with_capacity(1),
            };
            unsafe { *pSwapchain = Handle::new(swapchain) };
            VkResult::VK_SUCCESS
        }
        Err(err) => {
            use hal::window::CreationError as Ce;
            match err {
                Ce::OutOfMemory(oom) => map_oom(oom),
                Ce::DeviceLost(hal::device::DeviceLost) =>
                    VkResult::VK_ERROR_DEVICE_LOST,
                Ce::SurfaceLost(hal::device::SurfaceLost) =>
                    VkResult::VK_ERROR_SURFACE_LOST_KHR,
                Ce::WindowInUse(hal::device::WindowInUse) =>
                    VkResult::VK_ERROR_NATIVE_WINDOW_IN_USE_KHR,
            }
        }
    }
}
#[inline]
pub extern "C" fn gfxDestroySwapchainKHR(
    gpu: VkDevice,
    swapchain: VkSwapchainKHR,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if let Some(mut sc) = swapchain.unbox() {
        unsafe {
            sc.surface.unconfigure_swapchain(&gpu.device)
        };
    }
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
    let available_images = swapchain.count as u32;

    if pSwapchainImages.is_null() {
        // If NULL the number of presentable images is returned.
        *swapchain_image_count = available_images;
    } else {
        *swapchain_image_count = available_images.min(*swapchain_image_count);

        for frame in 0 .. *swapchain_image_count as u8 {
            unsafe {
                *pSwapchainImages.offset(frame as isize) = Handle::new(Image::SwapchainFrame {
                    swapchain,
                    frame,
                });
            };
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
                instance
                    .backend
                    .create_surface_from_hwnd(info.hinstance, info.hwnd),
            );
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(any(feature = "gfx-backend-dx12", feature = "gfx-backend-dx11"))]
    {
        unsafe {
            assert_eq!(info.flags, 0);
            *pSurface = Handle::new(instance.backend.create_surface_from_hwnd(info.hwnd));
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(not(all(
        target_os = "windows",
        any(
            feature = "gfx-backend-vulkan",
            feature = "gfx-backend-dx12",
            feature = "gfx-backend-dx11"
        )
    )))]
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
                instance
                    .backend
                    .create_surface_from_xcb(info.connection as _, info.window),
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
    _gpu: VkDevice,
    mut swapchain: VkSwapchainKHR,
    timeout: u64,
    semaphore: VkSemaphore,
    fence: VkFence,
    pImageIndex: *mut u32,
) -> VkResult {
    if let Some(fence) = fence.as_mut() {
        fence.is_fake = true;
    }
    if let Some(sem) = semaphore.as_mut() {
        sem.is_fake = true;
    }

    if let Some(_old_frame) = swapchain.active.take() {
        warn!("Swapchain frame {} was not presented!", swapchain.current_index);
    }
    match unsafe { swapchain.surface.acquire_image(timeout) } {
        Ok((frame, suboptimal)) => {
            swapchain.current_index = (swapchain.current_index + 1) % swapchain.count;
            swapchain.active = Some(frame);
            unsafe {
                *pImageIndex = swapchain.current_index as u32;
            }
            match suboptimal {
                Some(_) => VkResult::VK_SUBOPTIMAL_KHR,
                None => VkResult::VK_SUCCESS,
            }
        }
        Err(hal::window::AcquireError::NotReady) => VkResult::VK_NOT_READY,
        Err(hal::window::AcquireError::OutOfDate) => VkResult::VK_ERROR_OUT_OF_DATE_KHR,
        Err(hal::window::AcquireError::SurfaceLost(_)) => VkResult::VK_ERROR_SURFACE_LOST_KHR,
        Err(hal::window::AcquireError::DeviceLost(_)) => VkResult::VK_ERROR_DEVICE_LOST,
        Err(hal::window::AcquireError::Timeout) => VkResult::VK_TIMEOUT,
        Err(hal::window::AcquireError::OutOfMemory(oom)) => map_oom(oom),
    }
}
#[inline]
pub extern "C" fn gfxQueuePresentKHR(
    mut queue: VkQueue,
    pPresentInfo: *const VkPresentInfoKHR,
) -> VkResult {
    let info = unsafe { &*pPresentInfo };

    let swapchain_slice =
        unsafe { slice::from_raw_parts(info.pSwapchains, info.swapchainCount as _) };
    let index_slice =
        unsafe { slice::from_raw_parts(info.pImageIndices, info.swapchainCount as _) };
    let wait_semaphores = unsafe {
        slice::from_raw_parts(info.pWaitSemaphores, info.waitSemaphoreCount as _)
    };
    if wait_semaphores.len() > 1 {
        warn!("Only one semaphore is supported for present, {} are given", wait_semaphores.len());
    }

    for (swapchain, index) in swapchain_slice.iter().zip(index_slice) {
        let sc = swapchain.as_mut().unwrap();
        let frame = sc.active.take().expect("Frame was not acquired properly!");
        if sc.current_index == *index as u8 {
            let sem = wait_semaphores.first().map(|s| &s.raw);
            if let Err(_) = unsafe {
                queue.present_surface(&mut *sc.surface, frame, sem)
            } {
                return VkResult::VK_ERROR_SURFACE_LOST_KHR;
            }
        } else {
            warn!("Swapchain frame {} is stale, can't be presented.", *index);
        }
        for framebuffer in sc.lazy_framebuffers.drain(..) {
            unsafe {
                sc.gpu.device.destroy_framebuffer(framebuffer)
            };
        }
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxCreateMetalSurfaceEXT(
    instance: VkInstance,
    pCreateInfo: *const VkMetalSurfaceCreateInfoEXT,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    assert!(pAllocator.is_null());
    let info = unsafe { &*pCreateInfo };
    #[cfg(feature = "gfx-backend-metal")]
    unsafe {
        let enable_signposts = env::var("GFX_METAL_SIGNPOSTS").is_ok();
        if enable_signposts {
            println!("GFX: enabled signposts");
        }
        assert_eq!(info.flags, 0);
        *pSurface = Handle::new(
            instance
                .backend
                .create_surface_from_layer(info.pLayer as *mut _, enable_signposts),
        );
        VkResult::VK_SUCCESS
    }
    #[cfg(not(feature = "gfx-backend-metal"))]
    {
        let _ = (instance, info, pSurface);
        unreachable!()
    }
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
    #[cfg(all(target_os = "macos", feature = "gfx-backend-metal"))]
    unsafe {
        let enable_signposts = env::var("GFX_METAL_SIGNPOSTS").is_ok();
        if enable_signposts {
            println!("GFX: enabled signposts");
        }
        assert_eq!(info.flags, 0);
        *pSurface = Handle::new(
            instance
                .backend
                .create_surface_from_nsview(info.pView, enable_signposts),
        );
        VkResult::VK_SUCCESS
    }
    #[cfg(not(all(target_os = "macos", feature = "gfx-backend-metal")))]
    {
        let _ = (instance, info, pSurface);
        unreachable!()
    }
}
