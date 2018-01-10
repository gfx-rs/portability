use hal::{pass, pso, queue};
use hal::{
    Backend, DescriptorPool, Device, Instance, PhysicalDevice, QueueFamily,
    Surface, Swapchain as HalSwapchain, FrameSync,
};
use hal::device::WaitFor;
use hal::pool::RawCommandPool;
use hal::command::RawCommandBuffer;
use hal::queue::RawCommandQueue;

use std::ffi::CString;
use std::mem;
use std::ops::{Deref, Range};

use super::*;

const VERSION: (u32, u32, u32) = (1, 0, 66);
const DRIVER_VERSION: u32 = 1;

#[inline]
pub extern "C" fn gfxCreateInstance(
    _pCreateInfo: *const VkInstanceCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pInstance: *mut VkInstance,
) -> VkResult {
    let instance = back::Instance::create("portability", 1);
    unsafe { *pInstance = Handle::new(instance) };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxDestroyInstance(
    instance: VkInstance,
    _pAllocator: *const VkAllocationCallbacks,
) {
    instance.unwrap();
    //let it drop
}

#[inline]
pub extern "C" fn gfxEnumeratePhysicalDevices(
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
    unimplemented!()
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
    physicalDevice: VkPhysicalDevice,
    format: VkFormat,
    type_: VkImageType,
    tiling: VkImageTiling,
    usage: VkImageUsageFlags,
    flags: VkImageCreateFlags,
    pImageFormatProperties: *mut VkImageFormatProperties,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceProperties(
    adapter: VkPhysicalDevice,
    pProperties: *mut VkPhysicalDeviceProperties,
) {
    let adapter_info = &adapter.info;
    let limits = adapter.physical_device.get_limits();
    let (major, minor, patch) = VERSION;

    let device_name = {
        let c_string = CString::new(adapter_info.name.clone()).unwrap();
        let c_str = c_string.as_bytes_with_nul();
        let mut name = [0; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE as _];
        let len = name.len().min(c_str.len()) - 1;
        name[..len].copy_from_slice(&c_str[..len]);
        unsafe { mem::transmute(name) }
    };

    let limits = unsafe { mem::zeroed() }; // TODO
    let sparse_properties = unsafe { mem::zeroed() }; // TODO

    unsafe {
        *pProperties = VkPhysicalDeviceProperties {
            apiVersion: (major << 22) | (minor << 12) | patch,
            driverVersion: DRIVER_VERSION,
            vendorID: adapter_info.vendor as _,
            deviceID: adapter_info.device as _,
            deviceType: VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_OTHER, // TODO
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
    instance: VkInstance,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    unimplemented!()
}

#[inline]
pub extern "C" fn gfxGetDeviceProcAddr(
    device: VkDevice,
    pName: *const ::std::os::raw::c_char,
) -> PFN_vkVoidFunction {
    unimplemented!()
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
    let request_infos = queue_infos
        .iter()
        .map(|info| {
            let family = adapter.queue_families[info.queueFamilyIndex as usize].clone();
            (family, vec![1.0; info.queueCount as usize])
        })
        .collect::<Vec<_>>();

    let gpu = adapter.physical_device.open(request_infos);

    match gpu {
        Ok(mut gpu) => {
            let queues = queue_infos
                .iter()
                .map(|info| {
                    let id = queue::QueueFamilyId(info.queueFamilyIndex as usize);
                    let group = gpu.queues.take_raw(id).unwrap();
                    let queues = group
                        .into_iter()
                        .map(Handle::new)
                        .collect();

                    (info.queueFamilyIndex, queues)
                })
                .collect();

            let gpu = Gpu {
                device: gpu.device,
                queues,
            };

            unsafe {
                *pDevice = Handle::new(gpu);
            }
            VkResult::VK_SUCCESS
        }
        Err(err) => conv::map_err_device_creation(err),
    }
}

#[inline]
pub extern "C" fn gfxDestroyDevice(device: VkDevice, _pAllocator: *const VkAllocationCallbacks) {
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
pub extern "C" fn gfxEnumerateInstanceExtensionProperties(
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
    physicalDevice: VkPhysicalDevice,
    pLayerName: *const ::std::os::raw::c_char,
    pPropertyCount: *mut u32,
    pProperties: *mut VkExtensionProperties,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxEnumerateInstanceLayerProperties(
    pPropertyCount: *mut u32,
    pProperties: *mut VkLayerProperties,
) -> VkResult {
    // TODO: dummy implementation
    unsafe { *pPropertyCount = 0; }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxEnumerateDeviceLayerProperties(
    physicalDevice: VkPhysicalDevice,
    pPropertyCount: *mut u32,
    pProperties: *mut VkLayerProperties,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetDeviceQueue(
    gpu: VkDevice,
    queueFamilyIndex: u32,
    queueIndex: u32,
    pQueue: *mut VkQueue,
) {
    unsafe {
        *pQueue = gpu.queues.get(&queueFamilyIndex).unwrap()[queueIndex as usize];
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
    let cmd_buffers = unsafe {
        slice::from_raw_parts(submission.pCommandBuffers, submission.commandBufferCount as _)
            .into_iter()
            .map(|cmd_buffer| **cmd_buffer)
            .collect::<Vec<_>>()
    };
    let wait_semaphores = unsafe {
        let semaphores = slice::from_raw_parts(submission.pWaitSemaphores, submission.waitSemaphoreCount as _);
        let stages = slice::from_raw_parts(submission.pWaitDstStageMask, submission.waitSemaphoreCount as _);

        stages.into_iter()
            .zip(semaphores.into_iter())
            .map(|(stage, semaphore)| (semaphore.deref(), conv::map_pipeline_stage_flags(*stage)))
            .collect::<Vec<_>>()
    };
    let signal_semaphores = unsafe {
        slice::from_raw_parts(submission.pSignalSemaphores, submission.signalSemaphoreCount as _)
            .into_iter()
            .map(|semaphore| semaphore.deref())
            .collect::<Vec<_>>()
    };

    let submission = hal::queue::RawSubmission {
        cmd_buffers: &cmd_buffers,
        wait_semaphores: &wait_semaphores,
        signal_semaphores: &signal_semaphores,
    };

    let fence = if fence.is_null() { None } else { Some(&*fence) };

    unsafe { queue.submit_raw(submission, fence); }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxQueueWaitIdle(queue: VkQueue) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDeviceWaitIdle(device: VkDevice) -> VkResult {
    // TODO
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
    pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.free_memory(*memory.unwrap());
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
    if size == VK_WHOLE_SIZE as VkDeviceSize {
        unimplemented!()
    }

    unsafe {
        *ppData = gpu.device
            .map_memory(&memory, offset..offset + size)
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
    device: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxInvalidateMappedMemoryRanges(
    device: VkDevice,
    memoryRangeCount: u32,
    pMemoryRanges: *const VkMappedMemoryRange,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetDeviceMemoryCommitment(
    device: VkDevice,
    memory: VkDeviceMemory,
    pCommittedMemoryInBytes: *mut VkDeviceSize,
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
    let temp = unsafe { mem::zeroed() };

    *buffer = match mem::replace(&mut *buffer, temp) {
        Buffer::Buffer(_) => panic!("An non-sparse buffer can only be bound once!"),
        Buffer::Unbound(unbound) => {
            Buffer::Buffer(
                gpu.device
                    .bind_buffer_memory(&memory, memoryOffset, unbound)
                    .unwrap() // TODO
            )
        }
    };

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxBindImageMemory(
    gpu: VkDevice,
    mut image: VkImage,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    let temp = unsafe { mem::zeroed() };

    *image = match mem::replace(&mut *image, temp) {
        Image::Image(_) => panic!("An non-sparse image can only be bound once!"),
        Image::Unbound(unbound) => {
            Image::Image(
                gpu.device
                    .bind_image_memory(&memory, memoryOffset, unbound)
                    .unwrap() // TODO
            )
        }
    };

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxGetBufferMemoryRequirements(
    gpu: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    let req = match *buffer.deref() {
        Buffer::Buffer(ref buffer) => unimplemented!(),
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
    let req = match *image.deref() {
        Image::Image(ref image) => unimplemented!(),
        Image::Unbound(ref image) => gpu.device.get_image_requirements(image),
    };

    *unsafe { &mut *pMemoryRequirements } = VkMemoryRequirements {
        size: req.size,
        alignment: req.alignment,
        memoryTypeBits: req.type_mask as _,
    };
}

#[inline]
pub extern "C" fn gfxGetImageSparseMemoryRequirements(
    device: VkDevice,
    image: VkImage,
    pSparseMemoryRequirementCount: *mut u32,
    pSparseMemoryRequirements: *mut VkSparseImageMemoryRequirements,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceSparseImageFormatProperties(
    physicalDevice: VkPhysicalDevice,
    format: VkFormat,
    type_: VkImageType,
    samples: VkSampleCountFlagBits,
    usage: VkImageUsageFlags,
    tiling: VkImageTiling,
    pPropertyCount: *mut u32,
    pProperties: *mut VkSparseImageFormatProperties,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxQueueBindSparse(
    queue: VkQueue,
    bindInfoCount: u32,
    pBindInfo: *const VkBindSparseInfo,
    fence: VkFence,
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
    gpu.device.destroy_fence(*fence.unwrap());
}
#[inline]
pub extern "C" fn gfxResetFences(
    device: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetFenceStatus(device: VkDevice, fence: VkFence) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxWaitForFences(
    gpu: VkDevice,
    fenceCount: u32,
    pFences: *const VkFence,
    waitAll: VkBool32,
    timeout: u64,
) -> VkResult {
    let fences = unsafe {
        slice::from_raw_parts(pFences, fenceCount as _)
            .into_iter()
            .map(|fence| fence.deref())
            .collect::<Vec<_>>()
    };

    let wait_for = match waitAll {
        VK_FALSE => WaitFor::Any,
        _ => WaitFor::All,
    };

    if gpu.device.wait_for_fences(&fences, wait_for, timeout as _) {
        VkResult::VK_SUCCESS
    } else {
        VkResult::VK_TIMEOUT
    }
}
#[inline]
pub extern "C" fn gfxCreateSemaphore(
    gpu: VkDevice,
    pCreateInfo: *const VkSemaphoreCreateInfo,
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
    gpu.device.destroy_semaphore(*semaphore.unwrap());
}
#[inline]
pub extern "C" fn gfxCreateEvent(
    device: VkDevice,
    pCreateInfo: *const VkEventCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pEvent: *mut VkEvent,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyEvent(
    device: VkDevice,
    event: VkEvent,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetEventStatus(device: VkDevice, event: VkEvent) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxSetEvent(device: VkDevice, event: VkEvent) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxResetEvent(device: VkDevice, event: VkEvent) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateQueryPool(
    device: VkDevice,
    pCreateInfo: *const VkQueryPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pQueryPool: *mut VkQueryPool,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyQueryPool(
    device: VkDevice,
    queryPool: VkQueryPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetQueryPoolResults(
    device: VkDevice,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
    dataSize: usize,
    pData: *mut ::std::os::raw::c_void,
    stride: VkDeviceSize,
    flags: VkQueryResultFlags,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateBuffer(
    gpu: VkDevice,
    pCreateInfo: *const VkBufferCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
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
    pAllocator: *const VkAllocationCallbacks,
) {
    if !buffer.is_null() {
        match *buffer.unwrap() {
            Buffer::Buffer(buffer) => gpu.device.destroy_buffer(buffer),
            Buffer::Unbound(_) => unimplemented!(),
        }
    }
}
#[inline]
pub extern "C" fn gfxCreateBufferView(
    device: VkDevice,
    pCreateInfo: *const VkBufferViewCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pView: *mut VkBufferView,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyBufferView(
    device: VkDevice,
    bufferView: VkBufferView,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateImage(
    gpu: VkDevice,
    pCreateInfo: *const VkImageCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pImage: *mut VkImage,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    assert_eq!(info.sharingMode, VkSharingMode::VK_SHARING_MODE_EXCLUSIVE); // TODO
    assert_eq!(info.tiling, VkImageTiling::VK_IMAGE_TILING_OPTIMAL); // TODO
    assert_eq!(info.initialLayout, VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED); // TODO

    let image = gpu.device
        .create_image(
            conv::map_image_kind(
                info.imageType,
                info.flags,
                info.extent,
                info.arrayLayers,
                info.samples,
            ),
            info.mipLevels as _,
            conv::map_format(info.format).unwrap(),
            conv::map_image_usage(info.usage),
        )
        .expect("Error on creating image");

    unsafe {
        *pImage = Handle::new(Image::Unbound(image));
    }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyImage(
    gpu: VkDevice,
    image: VkImage,
    pAllocator: *const VkAllocationCallbacks,
) {
    if !image.is_null() {
        match *image.unwrap() {
            Image::Image(image) => gpu.device.destroy_image(image),
            Image::Unbound(_) => unimplemented!(),
        }
    }
}
#[inline]
pub extern "C" fn gfxGetImageSubresourceLayout(
    device: VkDevice,
    image: VkImage,
    pSubresource: *const VkImageSubresource,
    pLayout: *mut VkSubresourceLayout,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateImageView(
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

    let view = gpu.device.create_image_view(
        image,
        conv::map_format(info.format).unwrap(),
        conv::map_swizzle(info.components),
        conv::map_subresource_range(info.subresourceRange),
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
    pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.destroy_image_view(*imageView.unwrap())
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
    gpu.device.destroy_shader_module(*shaderModule.unwrap());
}
#[inline]
pub extern "C" fn gfxCreatePipelineCache(
    device: VkDevice,
    pCreateInfo: *const VkPipelineCacheCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelineCache: *mut VkPipelineCache,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyPipelineCache(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPipelineCacheData(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    pDataSize: *mut usize,
    pData: *mut ::std::os::raw::c_void,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxMergePipelineCaches(
    device: VkDevice,
    dstCache: VkPipelineCache,
    srcCacheCount: u32,
    pSrcCaches: *const VkPipelineCache,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateGraphicsPipelines(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkGraphicsPipelineCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateComputePipelines(
    device: VkDevice,
    pipelineCache: VkPipelineCache,
    createInfoCount: u32,
    pCreateInfos: *const VkComputePipelineCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pPipelines: *mut VkPipeline,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyPipeline(
    device: VkDevice,
    pipeline: VkPipeline,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
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
        .map(|layout| layout.deref())
        .collect::<Vec<&<B as Backend>::DescriptorSetLayout>>();

    let ranges = push_constants
        .iter()
        .map(|constant| {
            let stages = conv::map_stage_flags(constant.stageFlags);
            let start = constant.offset / 4;
            let size = constant.size / 4;

            (stages, start .. start+size)
        })
        .collect::<Vec<_>>();

    let pipeline_layout = gpu.device
        .create_pipeline_layout(&layouts, &ranges);

    unsafe { *pPipelineLayout = Handle::new(pipeline_layout); }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyPipelineLayout(
    gpu: VkDevice,
    pipelineLayout: VkPipelineLayout,
    _pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.destroy_pipeline_layout(*pipelineLayout.unwrap());
}
#[inline]
pub extern "C" fn gfxCreateSampler(
    device: VkDevice,
    pCreateInfo: *const VkSamplerCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSampler: *mut VkSampler,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroySampler(
    device: VkDevice,
    sampler: VkSampler,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
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

    let bindings = layout_bindings
        .iter()
        .map(|binding| {
            assert!(binding.pImmutableSamplers.is_null()); // TODO

            pso::DescriptorSetLayoutBinding {
                binding: binding.binding as _,
                ty: conv::map_descriptor_type(binding.descriptorType),
                count: binding.descriptorCount as _,
                stage_flags: conv::map_stage_flags(binding.stageFlags),

            }
        })
        .collect::<Vec<_>>();

    let set_layout = gpu.device
        .create_descriptor_set_layout(&bindings);

    unsafe { *pSetLayout = Handle::new(set_layout); }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyDescriptorSetLayout(
    gpu: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    _pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.destroy_descriptor_set_layout(*descriptorSetLayout.unwrap());
}
#[inline]
pub extern "C" fn gfxCreateDescriptorPool(
    gpu: VkDevice,
    pCreateInfo: *const VkDescriptorPoolCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pDescriptorPool: *mut VkDescriptorPool,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };
    assert_eq!(info.flags, 0); // TODO

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

    let pool = gpu.device
        .create_descriptor_pool(info.maxSets as _, &ranges);

    unsafe { *pDescriptorPool = Handle::new(pool); }
    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxDestroyDescriptorPool(
    gpu: VkDevice,
    descriptorPool: VkDescriptorPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.destroy_descriptor_pool(*descriptorPool.unwrap());
}
#[inline]
pub extern "C" fn gfxResetDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    flags: VkDescriptorPoolResetFlags,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxAllocateDescriptorSets(
    _device: VkDevice,
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
        .map(|layout| layout.deref())
        .collect::<Vec<_>>();

    let descriptor_sets = pool.allocate_sets(&layouts);
    let sets = unsafe {
        slice::from_raw_parts_mut(pDescriptorSets, info.descriptorSetCount as _)
    };
    for (set, raw_set) in sets.iter_mut().zip(descriptor_sets.into_iter()) {
        *set = Handle::new(raw_set);
    }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxFreeDescriptorSets(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxUpdateDescriptorSets(
    gpu: VkDevice,
    descriptorWriteCount: u32,
    pDescriptorWrites: *const VkWriteDescriptorSet,
    descriptorCopyCount: u32,
    pDescriptorCopies: *const VkCopyDescriptorSet,
) {
    assert_eq!(descriptorCopyCount, 0); // TODO

    let writes = unsafe {
        slice::from_raw_parts(pDescriptorWrites, descriptorWriteCount as _)
    };

    let writes = writes
        .iter()
        .map(|write| {
            fn map_buffer_info(buffer_info: &[VkDescriptorBufferInfo]) -> Vec<(&<B as Backend>::Buffer, Range<u64>)> {
                buffer_info
                    .into_iter()
                    .map(|buffer| {
                        assert_ne!(buffer.range as i32, VK_WHOLE_SIZE);
                        (
                            match buffer.buffer.deref() {
                                &Buffer::Buffer(ref buf) => buf,
                                // Vulkan portability restriction:
                                // Non-sparse buffer need to be bound to device memory.
                                &Buffer::Unbound(_) => panic!("Buffer needs to be bound"),
                            },
                            buffer.offset .. buffer.offset+buffer.range,
                        )
                    })
                    .collect()
            }

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
            let desc_write = match ty {
                pso::DescriptorType::Sampler => pso::DescriptorWrite::Sampler(
                    image_info
                        .into_iter()
                        .map(|image| &*image.sampler)
                        .collect()
                ),
                pso::DescriptorType::SampledImage => pso::DescriptorWrite::SampledImage(
                    image_info
                        .into_iter()
                        .map(|image| (&*image.imageView, conv::map_image_layout(image.imageLayout)))
                        .collect()
                ),
                pso::DescriptorType::StorageImage => pso::DescriptorWrite::StorageImage(
                    image_info
                        .into_iter()
                        .map(|image| (&*image.imageView, conv::map_image_layout(image.imageLayout)))
                        .collect()
                ),
                pso::DescriptorType::UniformTexelBuffer => pso::DescriptorWrite::UniformTexelBuffer(
                    texel_buffer_views
                        .into_iter()
                        .map(|view| view.deref())
                        .collect()
                ),
                pso::DescriptorType::StorageTexelBuffer => pso::DescriptorWrite::StorageTexelBuffer(
                    texel_buffer_views
                        .into_iter()
                        .map(|view| view.deref())
                        .collect()
                ),
                pso::DescriptorType::UniformBuffer => pso::DescriptorWrite::UniformBuffer(
                    map_buffer_info(buffer_info)
                ),
                pso::DescriptorType::StorageBuffer => pso::DescriptorWrite::StorageBuffer(
                    map_buffer_info(buffer_info)
                ),
                pso::DescriptorType::InputAttachment => pso::DescriptorWrite::InputAttachment(
                    image_info
                        .into_iter()
                        .map(|image| (&*image.imageView, conv::map_image_layout(image.imageLayout)))
                        .collect()
                ),
            };

            pso::DescriptorSetWrite {
                set: &*write.dstSet,
                binding: write.dstBinding as _,
                array_offset: write.dstArrayElement as _,
                write: desc_write,
            }
        })
        .collect::<Vec<_>>();

    gpu.device.update_descriptor_sets(&writes);
}
#[inline]
pub extern "C" fn gfxCreateFramebuffer(
    gpu: VkDevice,
    pCreateInfo: *const VkFramebufferCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pFramebuffer: *mut VkFramebuffer,
) -> VkResult {
    let info = unsafe { &*pCreateInfo };

    let attachments = unsafe {
        slice::from_raw_parts(info.pAttachments, info.attachmentCount as _)
    };
    let attachments = attachments
        .into_iter()
        .map(|attachment| attachment.deref())
        .collect::<Vec<_>>();

    let extent = hal::device::Extent {
        width: info.width,
        height: info.height,
        depth: info.layers,
    };

    let framebuffer = gpu
        .device
        .create_framebuffer(&*info.renderPass, &attachments, extent)
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
    gpu.device.destroy_framebuffer(*framebuffer.unwrap());
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
            unimplemented!()
            /*
            unsafe {
                slice::from_raw_parts(subpass.pResolveAttachments, subpass.colorAttachmentCount as _)
                    .into_iter()
                    .map(map_attachment_ref)
                    .collect()
            }
            */
        };
        let depth_stencil = unsafe {
            subpass
                .pDepthStencilAttachment
                .as_ref()
                .map(|attachment| map_attachment_ref(attachment))
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
        .map(|attachment_ref| {
            pass::SubpassDesc {
                colors: &attachment_ref.color,
                depth_stencil: attachment_ref.depth_stencil.as_ref(),
                inputs: &attachment_ref.input,
                preserves: &attachment_ref.preserve,
            }
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
            assert_eq!(dependency.dependencyFlags, 0); // TODO

            let src_pass = map_subpass_ref(dependency.srcSubpass);
            let dst_pass = map_subpass_ref(dependency.dstSubpass);

            let src_stage = conv::map_pipeline_stage_flags(dependency.srcStageMask);
            let dst_stage = conv::map_pipeline_stage_flags(dependency.dstStageMask);

            // Our portability implementation only supports image access flags atm.
            // Global buffer barriers can't be handled currently.
            let src_access = conv::map_image_acces(dependency.srcAccessMask);
            let dst_access = conv::map_image_acces(dependency.dstAccessMask);

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
    gpu.device.destroy_renderpass(*renderPass.unwrap());
}
#[inline]
pub extern "C" fn gfxGetRenderAreaGranularity(
    device: VkDevice,
    renderPass: VkRenderPass,
    pGranularity: *mut VkExtent2D,
) {
    unimplemented!()
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

    let pool = gpu.device.create_command_pool(family, flags);
    unsafe { *pCommandPool = Handle::new(pool) };
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxDestroyCommandPool(
    gpu: VkDevice,
    commandPool: VkCommandPool,
    _pAllocator: *const VkAllocationCallbacks,
) {
    gpu.device.destroy_command_pool(*commandPool.unwrap());
}

#[inline]
pub extern "C" fn gfxResetCommandPool(
    _gpu: VkDevice,
    mut commandPool: VkCommandPool,
    _flags: VkCommandPoolResetFlags,
) -> VkResult {
    commandPool.reset();
    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxAllocateCommandBuffers(
    _gpu: VkDevice,
    pAllocateInfo: *const VkCommandBufferAllocateInfo,
    pCommandBuffers: *mut VkCommandBuffer,
) -> VkResult {
    let info = unsafe { &mut *(pAllocateInfo as *mut VkCommandBufferAllocateInfo) };
    assert_eq!(
        info.level,
        VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY
    ); //TODO
    let count = info.commandBufferCount as usize;

    let cmd_bufs = info.commandPool.allocate(count);

    let output = unsafe { slice::from_raw_parts_mut(pCommandBuffers, count) };
    for (out, cmd_buf) in output.iter_mut().zip(cmd_bufs) {
        *out = Handle::new(cmd_buf);
    }

    VkResult::VK_SUCCESS
}

#[inline]
pub extern "C" fn gfxFreeCommandBuffers(
    _gpu: VkDevice,
    mut commandPool: VkCommandPool,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    let buffer_slice = unsafe { slice::from_raw_parts(pCommandBuffers, commandBufferCount as _) };
    let buffers = buffer_slice.iter().map(|buffer| *buffer.unwrap()).collect();

    unsafe { commandPool.free(buffers) };
}

#[inline]
pub extern "C" fn gfxBeginCommandBuffer(
    mut commandBuffer: VkCommandBuffer,
    pBeginInfo: *const VkCommandBufferBeginInfo,
) -> VkResult {
    assert_eq!(unsafe { (*pBeginInfo).flags }, 0); // TODO

    commandBuffer.begin();

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxEndCommandBuffer(mut commandBuffer: VkCommandBuffer) -> VkResult {
    commandBuffer.finish();

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxResetCommandBuffer(
    commandBuffer: VkCommandBuffer,
    flags: VkCommandBufferResetFlags,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdBindPipeline(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    pipeline: VkPipeline,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetViewport(
    commandBuffer: VkCommandBuffer,
    firstViewport: u32,
    viewportCount: u32,
    pViewports: *const VkViewport,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetScissor(
    commandBuffer: VkCommandBuffer,
    firstScissor: u32,
    scissorCount: u32,
    pScissors: *const VkRect2D,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetLineWidth(commandBuffer: VkCommandBuffer, lineWidth: f32) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetDepthBias(
    commandBuffer: VkCommandBuffer,
    depthBiasConstantFactor: f32,
    depthBiasClamp: f32,
    depthBiasSlopeFactor: f32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetBlendConstants(
    commandBuffer: VkCommandBuffer,
    blendConstants: *const f32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetDepthBounds(
    commandBuffer: VkCommandBuffer,
    minDepthBounds: f32,
    maxDepthBounds: f32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetStencilCompareMask(
    commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    compareMask: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetStencilWriteMask(
    commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    writeMask: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetStencilReference(
    commandBuffer: VkCommandBuffer,
    faceMask: VkStencilFaceFlags,
    reference: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdBindDescriptorSets(
    commandBuffer: VkCommandBuffer,
    pipelineBindPoint: VkPipelineBindPoint,
    layout: VkPipelineLayout,
    firstSet: u32,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
    dynamicOffsetCount: u32,
    pDynamicOffsets: *const u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdBindIndexBuffer(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    indexType: VkIndexType,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdBindVertexBuffers(
    commandBuffer: VkCommandBuffer,
    firstBinding: u32,
    bindingCount: u32,
    pBuffers: *const VkBuffer,
    pOffsets: *const VkDeviceSize,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdDraw(
    commandBuffer: VkCommandBuffer,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdDrawIndexed(
    commandBuffer: VkCommandBuffer,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    vertexOffset: i32,
    firstInstance: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdDrawIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdDrawIndexedIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
    drawCount: u32,
    stride: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdDispatch(
    commandBuffer: VkCommandBuffer,
    groupCountX: u32,
    groupCountY: u32,
    groupCountZ: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdDispatchIndirect(
    commandBuffer: VkCommandBuffer,
    buffer: VkBuffer,
    offset: VkDeviceSize,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdCopyBuffer(
    commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferCopy,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdCopyImage(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageCopy,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdBlitImage(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageBlit,
    filter: VkFilter,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdCopyBufferToImage(
    commandBuffer: VkCommandBuffer,
    srcBuffer: VkBuffer,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkBufferImageCopy,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdCopyImageToBuffer(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstBuffer: VkBuffer,
    regionCount: u32,
    pRegions: *const VkBufferImageCopy,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdUpdateBuffer(
    commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    dataSize: VkDeviceSize,
    pData: *const ::std::os::raw::c_void,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdFillBuffer(
    commandBuffer: VkCommandBuffer,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    size: VkDeviceSize,
    data: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdClearColorImage(
    commandBuffer: VkCommandBuffer,
    image: VkImage,
    imageLayout: VkImageLayout,
    pColor: *const VkClearColorValue,
    rangeCount: u32,
    pRanges: *const VkImageSubresourceRange,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdClearDepthStencilImage(
    commandBuffer: VkCommandBuffer,
    image: VkImage,
    imageLayout: VkImageLayout,
    pDepthStencil: *const VkClearDepthStencilValue,
    rangeCount: u32,
    pRanges: *const VkImageSubresourceRange,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdClearAttachments(
    commandBuffer: VkCommandBuffer,
    attachmentCount: u32,
    pAttachments: *const VkClearAttachment,
    rectCount: u32,
    pRects: *const VkClearRect,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdResolveImage(
    commandBuffer: VkCommandBuffer,
    srcImage: VkImage,
    srcImageLayout: VkImageLayout,
    dstImage: VkImage,
    dstImageLayout: VkImageLayout,
    regionCount: u32,
    pRegions: *const VkImageResolve,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdResetEvent(
    commandBuffer: VkCommandBuffer,
    event: VkEvent,
    stageMask: VkPipelineStageFlags,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdWaitEvents(
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
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdPipelineBarrier(
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
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdBeginQuery(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    query: u32,
    flags: VkQueryControlFlags,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdEndQuery(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    query: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdResetQueryPool(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdWriteTimestamp(
    commandBuffer: VkCommandBuffer,
    pipelineStage: VkPipelineStageFlagBits,
    queryPool: VkQueryPool,
    query: u32,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdCopyQueryPoolResults(
    commandBuffer: VkCommandBuffer,
    queryPool: VkQueryPool,
    firstQuery: u32,
    queryCount: u32,
    dstBuffer: VkBuffer,
    dstOffset: VkDeviceSize,
    stride: VkDeviceSize,
    flags: VkQueryResultFlags,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdPushConstants(
    commandBuffer: VkCommandBuffer,
    layout: VkPipelineLayout,
    stageFlags: VkShaderStageFlags,
    offset: u32,
    size: u32,
    pValues: *const ::std::os::raw::c_void,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdBeginRenderPass(
    commandBuffer: VkCommandBuffer,
    pRenderPassBegin: *const VkRenderPassBeginInfo,
    contents: VkSubpassContents,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdNextSubpass(commandBuffer: VkCommandBuffer, contents: VkSubpassContents) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdEndRenderPass(commandBuffer: VkCommandBuffer) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdExecuteCommands(
    commandBuffer: VkCommandBuffer,
    commandBufferCount: u32,
    pCommandBuffers: *const VkCommandBuffer,
) {
    unimplemented!()
}

#[inline]
pub extern "C" fn gfxDestroySurfaceKHR(
    _instance: VkInstance,
    surface: VkSurfaceKHR,
    _: *const VkAllocationCallbacks,
) {
    let _ = surface.unwrap(); //TODO
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
        .capabilities_and_formats(&adapter.physical_device)
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
        color_format: conv::map_format(info.imageFormat).unwrap(),
        depth_stencil_format: None,
        image_count: info.minImageCount,
    };
    let (swapchain, backbuffers) = gpu.device
        .create_swapchain(&mut info.surface.clone(), config);

    let images = match backbuffers {
        hal::Backbuffer::Images(images) => images
            .into_iter()
            .map(|image| Handle::new(Image::Image(image)))
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
pub extern "C" fn gfxGetSwapchainImagesKHR(
    device: VkDevice,
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
    commandBuffer: VkCommandBuffer,
    pProcessCommandsInfo: *const VkCmdProcessCommandsInfoNVX,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdReserveSpaceForCommandsNVX(
    commandBuffer: VkCommandBuffer,
    pReserveSpaceInfo: *const VkCmdReserveSpaceForCommandsInfoNVX,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateIndirectCommandsLayoutNVX(
    device: VkDevice,
    pCreateInfo: *const VkIndirectCommandsLayoutCreateInfoNVX,
    pAllocator: *const VkAllocationCallbacks,
    pIndirectCommandsLayout: *mut VkIndirectCommandsLayoutNVX,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyIndirectCommandsLayoutNVX(
    device: VkDevice,
    indirectCommandsLayout: VkIndirectCommandsLayoutNVX,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateObjectTableNVX(
    device: VkDevice,
    pCreateInfo: *const VkObjectTableCreateInfoNVX,
    pAllocator: *const VkAllocationCallbacks,
    pObjectTable: *mut VkObjectTableNVX,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDestroyObjectTableNVX(
    device: VkDevice,
    objectTable: VkObjectTableNVX,
    pAllocator: *const VkAllocationCallbacks,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxRegisterObjectsNVX(
    device: VkDevice,
    objectTable: VkObjectTableNVX,
    objectCount: u32,
    ppObjectTableEntries: *const *const VkObjectTableEntryNVX,
    pObjectIndices: *const u32,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxUnregisterObjectsNVX(
    device: VkDevice,
    objectTable: VkObjectTableNVX,
    objectCount: u32,
    pObjectEntryTypes: *const VkObjectEntryTypeNVX,
    pObjectIndices: *const u32,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceGeneratedCommandsPropertiesNVX(
    physicalDevice: VkPhysicalDevice,
    pFeatures: *mut VkDeviceGeneratedCommandsFeaturesNVX,
    pLimits: *mut VkDeviceGeneratedCommandsLimitsNVX,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetViewportWScalingNV(
    commandBuffer: VkCommandBuffer,
    firstViewport: u32,
    viewportCount: u32,
    pViewportWScalings: *const VkViewportWScalingNV,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxReleaseDisplayEXT(
    physicalDevice: VkPhysicalDevice,
    display: VkDisplayKHR,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetPhysicalDeviceSurfaceCapabilities2EXT(
    physicalDevice: VkPhysicalDevice,
    surface: VkSurfaceKHR,
    pSurfaceCapabilities: *mut VkSurfaceCapabilities2EXT,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxDisplayPowerControlEXT(
    device: VkDevice,
    display: VkDisplayKHR,
    pDisplayPowerInfo: *const VkDisplayPowerInfoEXT,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxRegisterDeviceEventEXT(
    device: VkDevice,
    pDeviceEventInfo: *const VkDeviceEventInfoEXT,
    pAllocator: *const VkAllocationCallbacks,
    pFence: *mut VkFence,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxRegisterDisplayEventEXT(
    device: VkDevice,
    display: VkDisplayKHR,
    pDisplayEventInfo: *const VkDisplayEventInfoEXT,
    pAllocator: *const VkAllocationCallbacks,
    pFence: *mut VkFence,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxGetSwapchainCounterEXT(
    device: VkDevice,
    swapchain: VkSwapchainKHR,
    counter: VkSurfaceCounterFlagBitsEXT,
    pCounterValue: *mut u64,
) -> VkResult {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCmdSetDiscardRectangleEXT(
    commandBuffer: VkCommandBuffer,
    firstDiscardRectangle: u32,
    discardRectangleCount: u32,
    pDiscardRectangles: *const VkRect2D,
) {
    unimplemented!()
}
#[inline]
pub extern "C" fn gfxCreateWin32SurfaceKHR(
    instance: VkInstance,
    pCreateInfos: *const VkWin32SurfaceCreateInfoKHR,
    pAllocator: *const VkAllocationCallbacks,
    pSurface: *mut VkSurfaceKHR,
) -> VkResult {
    #[cfg(all(feature = "vulkan", target_os = "windows"))]
    {
        unsafe {
            assert_eq!((*pCreateInfos).flags, 0);
            assert!(pAllocator.is_null());
            *pSurface = Handle::new(
                instance.create_surface_from_hwnd((*pCreateInfos).hinstance, (*pCreateInfos).hwnd),
            );
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(feature = "dx12")]
    {
        unsafe {
            assert_eq!((*pCreateInfos).flags, 0);
            assert!(pAllocator.is_null());
            *pSurface = Handle::new(instance.create_surface_from_hwnd((*pCreateInfos).hwnd));
            VkResult::VK_SUCCESS
        }
    }
    #[cfg(not(target_os = "windows"))]
    unreachable!()
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
    let sync = if !semaphore.is_null() {
        FrameSync::Semaphore(&*semaphore)
    } else {
        FrameSync::Fence(&*fence)
    };

    let frame = swapchain.raw.acquire_frame(sync);
    unsafe { *pImageIndex = frame.id() as _; }

    VkResult::VK_SUCCESS
}
#[inline]
pub extern "C" fn gfxQueuePresentKHR(
    queue: VkQueue,
    pPresentInfo: *const VkPresentInfoKHR,
) -> VkResult {
    unimplemented!()
}
