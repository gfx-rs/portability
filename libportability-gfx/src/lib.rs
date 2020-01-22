#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)] //TEMP: buggy Rustc FFI analysis
#![cfg_attr(feature = "nightly", feature(core_intrinsics))]

#[cfg(feature = "gfx-backend-dx11")]
extern crate gfx_backend_dx11 as back;
#[cfg(feature = "gfx-backend-dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(not(any(
    feature = "gfx-backend-dx12",
    feature = "gfx-backend-dx11",
    feature = "gfx-backend-metal",
    feature = "gfx-backend-vulkan",
    feature = "gfx-backend-gl",
)))]
extern crate gfx_backend_empty as back;
#[cfg(feature = "gfx-backend-gl")]
extern crate gfx_backend_gl as back;
#[cfg(feature = "gfx-backend-metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "gfx-backend-vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;
extern crate smallvec;

extern crate copyless;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[cfg(feature = "env_logger")]
extern crate env_logger;
#[cfg(feature = "nightly")]
extern crate gfx_auxil;
#[cfg(feature = "renderdoc")]
extern crate renderdoc;

mod conv;
mod handle;
mod impls;

use smallvec::SmallVec;

use back::Backend as B;
use handle::{DispatchHandle, Handle};

use std::collections::HashMap;
use std::slice;

pub use impls::*;

// Vulkan objects
pub type VkInstance = Handle<RawInstance>;
pub type VkPhysicalDevice = Handle<hal::adapter::Adapter<B>>;
pub type VkDevice = DispatchHandle<Gpu<B>>;
pub type VkQueue = DispatchHandle<<B as hal::Backend>::CommandQueue>;
pub type VkCommandPool = Handle<CommandPool<B>>;
pub type VkCommandBuffer = DispatchHandle<<B as hal::Backend>::CommandBuffer>;
pub type VkDeviceMemory = Handle<<B as hal::Backend>::Memory>;
pub type VkDescriptorSetLayout = Handle<<B as hal::Backend>::DescriptorSetLayout>;
pub type VkPipelineLayout = Handle<<B as hal::Backend>::PipelineLayout>;
pub type VkDescriptorPool = Handle<DescriptorPool<B>>;
pub type VkDescriptorSet = Handle<<B as hal::Backend>::DescriptorSet>;
pub type VkSampler = Handle<<B as hal::Backend>::Sampler>;
pub type VkBufferView = Handle<<B as hal::Backend>::BufferView>;
pub type VkShaderModule = Handle<<B as hal::Backend>::ShaderModule>;
pub type VkImage = Handle<Image<B>>;
pub type VkImageView = Handle<<B as hal::Backend>::ImageView>;
pub type VkBuffer = Handle<<B as hal::Backend>::Buffer>;
pub type VkSemaphore = Handle<<B as hal::Backend>::Semaphore>;
pub type VkEvent = Handle<<B as hal::Backend>::Event>;
pub type VkFence = Handle<<B as hal::Backend>::Fence>;
pub type VkRenderPass = Handle<<B as hal::Backend>::RenderPass>;
pub type VkFramebuffer = Handle<<B as hal::Backend>::Framebuffer>;
pub type VkPipeline = Handle<Pipeline<B>>;
pub type VkPipelineCache = Handle<<B as hal::Backend>::PipelineCache>;
pub type VkQueryPool = Handle<<B as hal::Backend>::QueryPool>;

pub type QueueFamilyIndex = u32;

pub struct RawInstance {
    pub backend: back::Instance,
    pub adapters: Vec<VkPhysicalDevice>,
    pub enabled_extensions: Vec<String>,
}

pub struct Gpu<B: hal::Backend> {
    device: B::Device,
    queues: HashMap<QueueFamilyIndex, Vec<VkQueue>>,
    enabled_extensions: Vec<String>,
    #[cfg(feature = "renderdoc")]
    renderdoc: renderdoc::RenderDoc<renderdoc::V110>,
    #[cfg(feature = "renderdoc")]
    capturing: *mut (),
}

pub struct DescriptorPool<B: hal::Backend> {
    raw: B::DescriptorPool,
    temp_sets: SmallVec<[B::DescriptorSet; 1]>,
    set_handles: Option<Vec<VkDescriptorSet>>,
}

pub enum Pipeline<B: hal::Backend> {
    Graphics(B::GraphicsPipeline),
    Compute(B::ComputePipeline),
}

pub struct Image<B: hal::Backend> {
    raw: B::Image,
    mip_levels: u32,
    array_layers: u32,
}

impl<B: hal::Backend> Image<B> {
    fn map_subresource(&self, subresource: VkImageSubresource) -> hal::image::Subresource {
        hal::image::Subresource {
            aspects: conv::map_aspect(subresource.aspectMask),
            level: subresource.mipLevel as _,
            layer: subresource.arrayLayer as _,
        }
    }

    fn map_subresource_layers(
        &self,
        subresource: VkImageSubresourceLayers,
    ) -> hal::image::SubresourceLayers {
        let layer_end = if subresource.layerCount == VK_REMAINING_ARRAY_LAYERS as _ {
            self.array_layers
        } else {
            subresource.baseArrayLayer + subresource.layerCount
        };
        hal::image::SubresourceLayers {
            aspects: conv::map_aspect(subresource.aspectMask),
            level: subresource.mipLevel as _,
            layers: subresource.baseArrayLayer as _..layer_end as _,
        }
    }

    fn map_subresource_range(
        &self,
        subresource: VkImageSubresourceRange,
    ) -> hal::image::SubresourceRange {
        let level_end = if subresource.levelCount == VK_REMAINING_MIP_LEVELS as _ {
            self.mip_levels
        } else {
            subresource.baseMipLevel + subresource.levelCount
        };
        let layer_end = if subresource.layerCount == VK_REMAINING_ARRAY_LAYERS as _ {
            self.array_layers
        } else {
            subresource.baseArrayLayer + subresource.layerCount
        };
        hal::image::SubresourceRange {
            aspects: conv::map_aspect(subresource.aspectMask),
            levels: subresource.baseMipLevel as _..level_end as _,
            layers: subresource.baseArrayLayer as _..layer_end as _,
        }
    }
}

pub struct CommandPool<B: hal::Backend> {
    pool: B::CommandPool,
    buffers: Vec<VkCommandBuffer>,
}

//NOTE: all *KHR types have to be pure `Handle` things for compatibility with
//`VK_DEFINE_NON_DISPATCHABLE_HANDLE` used in `vulkan.h`
pub type VkSurfaceKHR = Handle<<B as hal::Backend>::Surface>;
pub type VkSwapchainKHR = Handle<Swapchain>;

pub struct Swapchain {
    // this can become None if it was used as the "old_swapchain"
    raw: Option<<B as hal::Backend>::Swapchain>,
    images: Vec<VkImage>,
}

/* automatically generated by rust-bindgen */

pub const VULKAN_H_: ::std::os::raw::c_uint = 1;
pub const VK_VERSION_1_0: ::std::os::raw::c_uint = 1;
pub const _STDINT_H: ::std::os::raw::c_uint = 1;
pub const _FEATURES_H: ::std::os::raw::c_uint = 1;
pub const _DEFAULT_SOURCE: ::std::os::raw::c_uint = 1;
pub const __USE_ISOC11: ::std::os::raw::c_uint = 1;
pub const __USE_ISOC99: ::std::os::raw::c_uint = 1;
pub const __USE_ISOC95: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX_IMPLICITLY: ::std::os::raw::c_uint = 1;
pub const _POSIX_SOURCE: ::std::os::raw::c_uint = 1;
pub const _POSIX_C_SOURCE: ::std::os::raw::c_uint = 200809;
pub const __USE_POSIX: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX2: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX199309: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX199506: ::std::os::raw::c_uint = 1;
pub const __USE_XOPEN2K: ::std::os::raw::c_uint = 1;
pub const __USE_XOPEN2K8: ::std::os::raw::c_uint = 1;
pub const _ATFILE_SOURCE: ::std::os::raw::c_uint = 1;
pub const __USE_MISC: ::std::os::raw::c_uint = 1;
pub const __USE_ATFILE: ::std::os::raw::c_uint = 1;
pub const __USE_FORTIFY_LEVEL: ::std::os::raw::c_uint = 0;
pub const _STDC_PREDEF_H: ::std::os::raw::c_uint = 1;
pub const __STDC_IEC_559__: ::std::os::raw::c_uint = 1;
pub const __STDC_IEC_559_COMPLEX__: ::std::os::raw::c_uint = 1;
pub const __STDC_ISO_10646__: ::std::os::raw::c_uint = 201605;
pub const __STDC_NO_THREADS__: ::std::os::raw::c_uint = 1;
pub const __GNU_LIBRARY__: ::std::os::raw::c_uint = 6;
pub const __GLIBC__: ::std::os::raw::c_uint = 2;
pub const __GLIBC_MINOR__: ::std::os::raw::c_uint = 25;
pub const _SYS_CDEFS_H: ::std::os::raw::c_uint = 1;
pub const __glibc_c99_flexarr_available: ::std::os::raw::c_uint = 1;
pub const __WORDSIZE: ::std::os::raw::c_uint = 64;
pub const __WORDSIZE_TIME64_COMPAT32: ::std::os::raw::c_uint = 1;
pub const __SYSCALL_WORDSIZE: ::std::os::raw::c_uint = 64;
pub const __GLIBC_USE_LIB_EXT2: ::std::os::raw::c_uint = 0;
pub const __GLIBC_USE_IEC_60559_BFP_EXT: ::std::os::raw::c_uint = 0;
pub const __GLIBC_USE_IEC_60559_FUNCS_EXT: ::std::os::raw::c_uint = 0;
pub const _BITS_TYPES_H: ::std::os::raw::c_uint = 1;
pub const _BITS_TYPESIZES_H: ::std::os::raw::c_uint = 1;
pub const __OFF_T_MATCHES_OFF64_T: ::std::os::raw::c_uint = 1;
pub const __INO_T_MATCHES_INO64_T: ::std::os::raw::c_uint = 1;
pub const __RLIM_T_MATCHES_RLIM64_T: ::std::os::raw::c_uint = 1;
pub const __FD_SETSIZE: ::std::os::raw::c_uint = 1024;
pub const _BITS_WCHAR_H: ::std::os::raw::c_uint = 1;
pub const INT8_MIN: ::std::os::raw::c_int = -128;
pub const INT16_MIN: ::std::os::raw::c_int = -32768;
pub const INT32_MIN: ::std::os::raw::c_int = -2147483648;
pub const INT8_MAX: ::std::os::raw::c_uint = 127;
pub const INT16_MAX: ::std::os::raw::c_uint = 32767;
pub const INT32_MAX: ::std::os::raw::c_uint = 2147483647;
pub const UINT8_MAX: ::std::os::raw::c_uint = 255;
pub const UINT16_MAX: ::std::os::raw::c_uint = 65535;
pub const UINT32_MAX: ::std::os::raw::c_uint = 4294967295;
pub const INT_LEAST8_MIN: ::std::os::raw::c_int = -128;
pub const INT_LEAST16_MIN: ::std::os::raw::c_int = -32768;
pub const INT_LEAST32_MIN: ::std::os::raw::c_int = -2147483648;
pub const INT_LEAST8_MAX: ::std::os::raw::c_uint = 127;
pub const INT_LEAST16_MAX: ::std::os::raw::c_uint = 32767;
pub const INT_LEAST32_MAX: ::std::os::raw::c_uint = 2147483647;
pub const UINT_LEAST8_MAX: ::std::os::raw::c_uint = 255;
pub const UINT_LEAST16_MAX: ::std::os::raw::c_uint = 65535;
pub const UINT_LEAST32_MAX: ::std::os::raw::c_uint = 4294967295;
pub const INT_FAST8_MIN: ::std::os::raw::c_int = -128;
pub const INT_FAST16_MIN: ::std::os::raw::c_longlong = -9223372036854775808;
pub const INT_FAST32_MIN: ::std::os::raw::c_longlong = -9223372036854775808;
pub const INT_FAST8_MAX: ::std::os::raw::c_uint = 127;
pub const INT_FAST16_MAX: ::std::os::raw::c_ulonglong = 9223372036854775807;
pub const INT_FAST32_MAX: ::std::os::raw::c_ulonglong = 9223372036854775807;
pub const UINT_FAST8_MAX: ::std::os::raw::c_uint = 255;
pub const UINT_FAST16_MAX: ::std::os::raw::c_int = -1;
pub const UINT_FAST32_MAX: ::std::os::raw::c_int = -1;
pub const INTPTR_MIN: ::std::os::raw::c_longlong = -9223372036854775808;
pub const INTPTR_MAX: ::std::os::raw::c_ulonglong = 9223372036854775807;
pub const UINTPTR_MAX: ::std::os::raw::c_int = -1;
pub const PTRDIFF_MIN: ::std::os::raw::c_longlong = -9223372036854775808;
pub const PTRDIFF_MAX: ::std::os::raw::c_ulonglong = 9223372036854775807;
pub const SIG_ATOMIC_MIN: ::std::os::raw::c_int = -2147483648;
pub const SIG_ATOMIC_MAX: ::std::os::raw::c_uint = 2147483647;
pub const SIZE_MAX: ::std::os::raw::c_int = -1;
pub const WINT_MIN: ::std::os::raw::c_uint = 0;
pub const WINT_MAX: ::std::os::raw::c_uint = 4294967295;
pub const VK_HEADER_VERSION: ::std::os::raw::c_uint = 42;
pub const VK_NULL_HANDLE: ::std::os::raw::c_uint = 0;
pub const VK_LOD_CLAMP_NONE: f64 = 1000.;
pub const VK_REMAINING_MIP_LEVELS: ::std::os::raw::c_int = -1;
pub const VK_REMAINING_ARRAY_LAYERS: ::std::os::raw::c_int = -1;
pub const VK_WHOLE_SIZE: ::std::os::raw::c_int = -1;
pub const VK_ATTACHMENT_UNUSED: ::std::os::raw::c_int = -1;
pub const VK_TRUE: ::std::os::raw::c_uint = 1;
pub const VK_FALSE: ::std::os::raw::c_uint = 0;
pub const VK_QUEUE_FAMILY_IGNORED: ::std::os::raw::c_int = -1;
pub const VK_SUBPASS_EXTERNAL: ::std::os::raw::c_int = -1;
pub const VK_MAX_PHYSICAL_DEVICE_NAME_SIZE: ::std::os::raw::c_uint = 256;
pub const VK_UUID_SIZE: ::std::os::raw::c_uint = 16;
pub const VK_MAX_MEMORY_TYPES: ::std::os::raw::c_uint = 32;
pub const VK_MAX_MEMORY_HEAPS: ::std::os::raw::c_uint = 16;
pub const VK_MAX_EXTENSION_NAME_SIZE: ::std::os::raw::c_uint = 256;
pub const VK_MAX_DESCRIPTION_SIZE: ::std::os::raw::c_uint = 256;
pub const VK_KHR_surface: ::std::os::raw::c_uint = 1;
pub const VK_KHR_SURFACE_SPEC_VERSION: ::std::os::raw::c_uint = 25;
pub const VK_KHR_SURFACE_EXTENSION_NAME: &'static [u8; 15usize] = b"VK_KHR_surface\x00";
pub const VK_KHR_WIN32_SURFACE_SPEC_VERSION: ::std::os::raw::c_uint = 6;
pub const VK_MVK_MACOS_SURFACE_SPEC_VERSION: ::std::os::raw::c_uint = 2;
pub const VK_KHR_WIN32_SURFACE_EXTENSION_NAME: &'static [u8; 21usize] = b"VK_KHR_win32_surface\x00";
pub const VK_MVK_MACOS_SURFACE_EXTENSION_NAME: &'static [u8; 21usize] = b"VK_MVK_macos_surface\x00";
pub const VK_EXT_METAL_SURFACE_EXTENSION_NAME: &'static [u8; 21usize] = b"VK_EXT_metal_surface\x00";
pub const VK_EXT_METAL_SURFACE_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHR_swapchain: ::std::os::raw::c_uint = 1;
pub const VK_KHR_SWAPCHAIN_SPEC_VERSION: ::std::os::raw::c_uint = 68;
pub const VK_KHR_SWAPCHAIN_EXTENSION_NAME: &'static [u8; 17usize] = b"VK_KHR_swapchain\x00";
pub const VK_KHR_display: ::std::os::raw::c_uint = 1;
pub const VK_KHR_DISPLAY_SPEC_VERSION: ::std::os::raw::c_uint = 21;
pub const VK_KHR_DISPLAY_EXTENSION_NAME: &'static [u8; 15usize] = b"VK_KHR_display\x00";
pub const VK_KHR_display_swapchain: ::std::os::raw::c_uint = 1;
pub const VK_KHR_DISPLAY_SWAPCHAIN_SPEC_VERSION: ::std::os::raw::c_uint = 9;
pub const VK_KHR_DISPLAY_SWAPCHAIN_EXTENSION_NAME: &'static [u8; 25usize] =
    b"VK_KHR_display_swapchain\x00";
pub const VK_KHR_sampler_mirror_clamp_to_edge: ::std::os::raw::c_uint = 1;
pub const VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_EXTENSION_NAME: &'static [u8; 36usize] =
    b"VK_KHR_sampler_mirror_clamp_to_edge\x00";
pub const VK_KHR_get_physical_device_properties2: ::std::os::raw::c_uint = 1;
pub const VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME: &'static [u8; 39usize] =
    b"VK_KHR_get_physical_device_properties2\x00";
pub const VK_KHR_shader_draw_parameters: ::std::os::raw::c_uint = 1;
pub const VK_KHR_SHADER_DRAW_PARAMETERS_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHR_SHADER_DRAW_PARAMETERS_EXTENSION_NAME: &'static [u8; 30usize] =
    b"VK_KHR_shader_draw_parameters\x00";
pub const VK_KHR_maintenance1: ::std::os::raw::c_uint = 1;
pub const VK_KHR_MAINTENANCE1_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHR_MAINTENANCE1_EXTENSION_NAME: &'static [u8; 20usize] = b"VK_KHR_maintenance1\x00";
pub const VK_KHR_push_descriptor: ::std::os::raw::c_uint = 1;
pub const VK_KHR_PUSH_DESCRIPTOR_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHR_PUSH_DESCRIPTOR_EXTENSION_NAME: &'static [u8; 23usize] =
    b"VK_KHR_push_descriptor\x00";
pub const VK_KHR_descriptor_update_template: ::std::os::raw::c_uint = 1;
pub const VK_KHR_DESCRIPTOR_UPDATE_TEMPLATE_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHR_DESCRIPTOR_UPDATE_TEMPLATE_EXTENSION_NAME: &'static [u8; 34usize] =
    b"VK_KHR_descriptor_update_template\x00";
pub const VK_EXT_debug_report: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DEBUG_REPORT_SPEC_VERSION: ::std::os::raw::c_uint = 5;
pub const VK_EXT_DEBUG_REPORT_EXTENSION_NAME: &'static [u8; 20usize] = b"VK_EXT_debug_report\x00";
pub const VK_NV_glsl_shader: ::std::os::raw::c_uint = 1;
pub const VK_NV_GLSL_SHADER_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_GLSL_SHADER_EXTENSION_NAME: &'static [u8; 18usize] = b"VK_NV_glsl_shader\x00";
pub const VK_IMG_filter_cubic: ::std::os::raw::c_uint = 1;
pub const VK_IMG_FILTER_CUBIC_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_IMG_FILTER_CUBIC_EXTENSION_NAME: &'static [u8; 20usize] = b"VK_IMG_filter_cubic\x00";
pub const VK_AMD_rasterization_order: ::std::os::raw::c_uint = 1;
pub const VK_AMD_RASTERIZATION_ORDER_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_RASTERIZATION_ORDER_EXTENSION_NAME: &'static [u8; 27usize] =
    b"VK_AMD_rasterization_order\x00";
pub const VK_AMD_shader_trinary_minmax: ::std::os::raw::c_uint = 1;
pub const VK_AMD_SHADER_TRINARY_MINMAX_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_SHADER_TRINARY_MINMAX_EXTENSION_NAME: &'static [u8; 29usize] =
    b"VK_AMD_shader_trinary_minmax\x00";
pub const VK_AMD_shader_explicit_vertex_parameter: ::std::os::raw::c_uint = 1;
pub const VK_AMD_SHADER_EXPLICIT_VERTEX_PARAMETER_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_SHADER_EXPLICIT_VERTEX_PARAMETER_EXTENSION_NAME: &'static [u8; 40usize] =
    b"VK_AMD_shader_explicit_vertex_parameter\x00";
pub const VK_EXT_debug_marker: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DEBUG_MARKER_SPEC_VERSION: ::std::os::raw::c_uint = 4;
pub const VK_EXT_DEBUG_MARKER_EXTENSION_NAME: &'static [u8; 20usize] = b"VK_EXT_debug_marker\x00";
pub const VK_AMD_gcn_shader: ::std::os::raw::c_uint = 1;
pub const VK_AMD_GCN_SHADER_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_GCN_SHADER_EXTENSION_NAME: &'static [u8; 18usize] = b"VK_AMD_gcn_shader\x00";
pub const VK_NV_dedicated_allocation: ::std::os::raw::c_uint = 1;
pub const VK_NV_DEDICATED_ALLOCATION_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_DEDICATED_ALLOCATION_EXTENSION_NAME: &'static [u8; 27usize] =
    b"VK_NV_dedicated_allocation\x00";
pub const VK_AMD_draw_indirect_count: ::std::os::raw::c_uint = 1;
pub const VK_AMD_DRAW_INDIRECT_COUNT_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_DRAW_INDIRECT_COUNT_EXTENSION_NAME: &'static [u8; 27usize] =
    b"VK_AMD_draw_indirect_count\x00";
pub const VK_AMD_negative_viewport_height: ::std::os::raw::c_uint = 1;
pub const VK_AMD_NEGATIVE_VIEWPORT_HEIGHT_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_NEGATIVE_VIEWPORT_HEIGHT_EXTENSION_NAME: &'static [u8; 32usize] =
    b"VK_AMD_negative_viewport_height\x00";
pub const VK_AMD_gpu_shader_half_float: ::std::os::raw::c_uint = 1;
pub const VK_AMD_GPU_SHADER_HALF_FLOAT_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_GPU_SHADER_HALF_FLOAT_EXTENSION_NAME: &'static [u8; 29usize] =
    b"VK_AMD_gpu_shader_half_float\x00";
pub const VK_AMD_shader_ballot: ::std::os::raw::c_uint = 1;
pub const VK_AMD_SHADER_BALLOT_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_AMD_SHADER_BALLOT_EXTENSION_NAME: &'static [u8; 21usize] = b"VK_AMD_shader_ballot\x00";
pub const VK_KHX_multiview: ::std::os::raw::c_uint = 1;
pub const VK_KHX_MULTIVIEW_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_MULTIVIEW_EXTENSION_NAME: &'static [u8; 17usize] = b"VK_KHX_multiview\x00";
pub const VK_IMG_format_pvrtc: ::std::os::raw::c_uint = 1;
pub const VK_IMG_FORMAT_PVRTC_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_IMG_FORMAT_PVRTC_EXTENSION_NAME: &'static [u8; 20usize] = b"VK_IMG_format_pvrtc\x00";
pub const VK_NV_external_memory_capabilities: ::std::os::raw::c_uint = 1;
pub const VK_NV_EXTERNAL_MEMORY_CAPABILITIES_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION_NAME: &'static [u8; 35usize] =
    b"VK_NV_external_memory_capabilities\x00";
pub const VK_NV_external_memory: ::std::os::raw::c_uint = 1;
pub const VK_NV_EXTERNAL_MEMORY_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_EXTERNAL_MEMORY_EXTENSION_NAME: &'static [u8; 22usize] =
    b"VK_NV_external_memory\x00";
pub const VK_KHX_device_group: ::std::os::raw::c_uint = 1;
pub const VK_MAX_DEVICE_GROUP_SIZE_KHX: ::std::os::raw::c_uint = 32;
pub const VK_KHX_DEVICE_GROUP_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_DEVICE_GROUP_EXTENSION_NAME: &'static [u8; 20usize] = b"VK_KHX_device_group\x00";
pub const VK_EXT_validation_flags: ::std::os::raw::c_uint = 1;
pub const VK_EXT_VALIDATION_FLAGS_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXT_VALIDATION_FLAGS_EXTENSION_NAME: &'static [u8; 24usize] =
    b"VK_EXT_validation_flags\x00";
pub const VK_EXT_shader_subgroup_ballot: ::std::os::raw::c_uint = 1;
pub const VK_EXT_SHADER_SUBGROUP_BALLOT_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXT_SHADER_SUBGROUP_BALLOT_EXTENSION_NAME: &'static [u8; 30usize] =
    b"VK_EXT_shader_subgroup_ballot\x00";
pub const VK_EXT_shader_subgroup_vote: ::std::os::raw::c_uint = 1;
pub const VK_EXT_SHADER_SUBGROUP_VOTE_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXT_SHADER_SUBGROUP_VOTE_EXTENSION_NAME: &'static [u8; 28usize] =
    b"VK_EXT_shader_subgroup_vote\x00";
pub const VK_KHX_device_group_creation: ::std::os::raw::c_uint = 1;
pub const VK_KHX_DEVICE_GROUP_CREATION_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_DEVICE_GROUP_CREATION_EXTENSION_NAME: &'static [u8; 29usize] =
    b"VK_KHX_device_group_creation\x00";
pub const VK_KHX_external_memory_capabilities: ::std::os::raw::c_uint = 1;
pub const VK_LUID_SIZE_KHX: ::std::os::raw::c_uint = 8;
pub const VK_KHX_EXTERNAL_MEMORY_CAPABILITIES_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION_NAME: &'static [u8; 36usize] =
    b"VK_KHX_external_memory_capabilities\x00";
pub const VK_KHX_external_memory: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_MEMORY_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_MEMORY_EXTENSION_NAME: &'static [u8; 23usize] =
    b"VK_KHX_external_memory\x00";
pub const VK_QUEUE_FAMILY_EXTERNAL_KHX: ::std::os::raw::c_int = -2;
pub const VK_KHX_external_memory_fd: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_MEMORY_FD_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_MEMORY_FD_EXTENSION_NAME: &'static [u8; 26usize] =
    b"VK_KHX_external_memory_fd\x00";
pub const VK_KHX_external_semaphore_capabilities: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_SEMAPHORE_CAPABILITIES_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_SEMAPHORE_CAPABILITIES_EXTENSION_NAME: &'static [u8; 39usize] =
    b"VK_KHX_external_semaphore_capabilities\x00";
pub const VK_KHX_external_semaphore: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_SEMAPHORE_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_SEMAPHORE_EXTENSION_NAME: &'static [u8; 26usize] =
    b"VK_KHX_external_semaphore\x00";
pub const VK_KHX_external_semaphore_fd: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_SEMAPHORE_FD_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_KHX_EXTERNAL_SEMAPHORE_FD_EXTENSION_NAME: &'static [u8; 29usize] =
    b"VK_KHX_external_semaphore_fd\x00";
pub const VK_NVX_device_generated_commands: ::std::os::raw::c_uint = 1;
pub const VK_NVX_DEVICE_GENERATED_COMMANDS_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NVX_DEVICE_GENERATED_COMMANDS_EXTENSION_NAME: &'static [u8; 33usize] =
    b"VK_NVX_device_generated_commands\x00";
pub const VK_NV_clip_space_w_scaling: ::std::os::raw::c_uint = 1;
pub const VK_NV_CLIP_SPACE_W_SCALING_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_CLIP_SPACE_W_SCALING_EXTENSION_NAME: &'static [u8; 27usize] =
    b"VK_NV_clip_space_w_scaling\x00";
pub const VK_EXT_direct_mode_display: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DIRECT_MODE_DISPLAY_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DIRECT_MODE_DISPLAY_EXTENSION_NAME: &'static [u8; 27usize] =
    b"VK_EXT_direct_mode_display\x00";
pub const VK_EXT_display_surface_counter: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DISPLAY_SURFACE_COUNTER_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DISPLAY_SURFACE_COUNTER_EXTENSION_NAME: &'static [u8; 31usize] =
    b"VK_EXT_display_surface_counter\x00";
pub const VK_EXT_display_control: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DISPLAY_CONTROL_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DISPLAY_CONTROL_EXTENSION_NAME: &'static [u8; 23usize] =
    b"VK_EXT_display_control\x00";
pub const VK_NV_sample_mask_override_coverage: ::std::os::raw::c_uint = 1;
pub const VK_NV_SAMPLE_MASK_OVERRIDE_COVERAGE_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_SAMPLE_MASK_OVERRIDE_COVERAGE_EXTENSION_NAME: &'static [u8; 36usize] =
    b"VK_NV_sample_mask_override_coverage\x00";
pub const VK_NV_geometry_shader_passthrough: ::std::os::raw::c_uint = 1;
pub const VK_NV_GEOMETRY_SHADER_PASSTHROUGH_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_GEOMETRY_SHADER_PASSTHROUGH_EXTENSION_NAME: &'static [u8; 34usize] =
    b"VK_NV_geometry_shader_passthrough\x00";
pub const VK_NV_viewport_array2: ::std::os::raw::c_uint = 1;
pub const VK_NV_VIEWPORT_ARRAY2_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_VIEWPORT_ARRAY2_EXTENSION_NAME: &'static [u8; 22usize] =
    b"VK_NV_viewport_array2\x00";
pub const VK_NVX_multiview_per_view_attributes: ::std::os::raw::c_uint = 1;
pub const VK_NVX_MULTIVIEW_PER_VIEW_ATTRIBUTES_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NVX_MULTIVIEW_PER_VIEW_ATTRIBUTES_EXTENSION_NAME: &'static [u8; 37usize] =
    b"VK_NVX_multiview_per_view_attributes\x00";
pub const VK_NV_viewport_swizzle: ::std::os::raw::c_uint = 1;
pub const VK_NV_VIEWPORT_SWIZZLE_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_NV_VIEWPORT_SWIZZLE_EXTENSION_NAME: &'static [u8; 23usize] =
    b"VK_NV_viewport_swizzle\x00";
pub const VK_EXT_discard_rectangles: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DISCARD_RECTANGLES_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXT_DISCARD_RECTANGLES_EXTENSION_NAME: &'static [u8; 26usize] =
    b"VK_EXT_discard_rectangles\x00";
pub const VK_EXTX_portability_subset: ::std::os::raw::c_uint = 1;
pub const VK_EXTX_PORTABILITY_SUBSET_SPEC_VERSION: ::std::os::raw::c_uint = 1;
pub const VK_EXTX_PORTABILITY_SUBSET_EXTENSION_NAME: &'static [u8; 27usize] =
    b"VK_EXTX_portability_subset\x00";

pub type wchar_t = ::std::os::raw::c_int;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct max_align_t {
    pub __clang_max_align_nonce1: ::std::os::raw::c_longlong,
    pub __bindgen_padding_0: u64,
    pub __clang_max_align_nonce2: f64,
}
impl Clone for max_align_t {
    fn clone(&self) -> Self {
        *self
    }
}
pub type __u_char = ::std::os::raw::c_uchar;
pub type __u_short = ::std::os::raw::c_ushort;
pub type __u_int = ::std::os::raw::c_uint;
pub type __u_long = ::std::os::raw::c_ulong;
pub type __int8_t = ::std::os::raw::c_schar;
pub type __uint8_t = ::std::os::raw::c_uchar;
pub type __int16_t = ::std::os::raw::c_short;
pub type __uint16_t = ::std::os::raw::c_ushort;
pub type __int32_t = ::std::os::raw::c_int;
pub type __uint32_t = ::std::os::raw::c_uint;
pub type __int64_t = ::std::os::raw::c_long;
pub type __uint64_t = ::std::os::raw::c_ulong;
pub type __quad_t = ::std::os::raw::c_long;
pub type __u_quad_t = ::std::os::raw::c_ulong;
pub type __intmax_t = ::std::os::raw::c_long;
pub type __uintmax_t = ::std::os::raw::c_ulong;
pub type __dev_t = ::std::os::raw::c_ulong;
pub type __uid_t = ::std::os::raw::c_uint;
pub type __gid_t = ::std::os::raw::c_uint;
pub type __ino_t = ::std::os::raw::c_ulong;
pub type __ino64_t = ::std::os::raw::c_ulong;
pub type __mode_t = ::std::os::raw::c_uint;
pub type __nlink_t = ::std::os::raw::c_ulong;
pub type __off_t = ::std::os::raw::c_long;
pub type __off64_t = ::std::os::raw::c_long;
pub type __pid_t = ::std::os::raw::c_int;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct __fsid_t {
    pub __val: [::std::os::raw::c_int; 2usize],
}
impl Clone for __fsid_t {
    fn clone(&self) -> Self {
        *self
    }
}
pub type __clock_t = ::std::os::raw::c_long;
pub type __rlim_t = ::std::os::raw::c_ulong;
pub type __rlim64_t = ::std::os::raw::c_ulong;
pub type __id_t = ::std::os::raw::c_uint;
pub type __time_t = ::std::os::raw::c_long;
pub type __useconds_t = ::std::os::raw::c_uint;
pub type __suseconds_t = ::std::os::raw::c_long;
pub type __daddr_t = ::std::os::raw::c_int;
pub type __key_t = ::std::os::raw::c_int;
pub type __clockid_t = ::std::os::raw::c_int;
pub type __timer_t = *mut ::std::os::raw::c_void;
pub type __blksize_t = ::std::os::raw::c_long;
pub type __blkcnt_t = ::std::os::raw::c_long;
pub type __blkcnt64_t = ::std::os::raw::c_long;
pub type __fsblkcnt_t = ::std::os::raw::c_ulong;
pub type __fsblkcnt64_t = ::std::os::raw::c_ulong;
pub type __fsfilcnt_t = ::std::os::raw::c_ulong;
pub type __fsfilcnt64_t = ::std::os::raw::c_ulong;
pub type __fsword_t = ::std::os::raw::c_long;
pub type __ssize_t = ::std::os::raw::c_long;
pub type __syscall_slong_t = ::std::os::raw::c_long;
pub type __syscall_ulong_t = ::std::os::raw::c_ulong;
pub type __loff_t = __off64_t;
pub type __qaddr_t = *mut __quad_t;
pub type __caddr_t = *mut ::std::os::raw::c_char;
pub type __intptr_t = ::std::os::raw::c_long;
pub type __socklen_t = ::std::os::raw::c_uint;
pub type int_least8_t = ::std::os::raw::c_schar;
pub type int_least16_t = ::std::os::raw::c_short;
pub type int_least32_t = ::std::os::raw::c_int;
pub type int_least64_t = ::std::os::raw::c_long;
pub type uint_least8_t = ::std::os::raw::c_uchar;
pub type uint_least16_t = ::std::os::raw::c_ushort;
pub type uint_least32_t = ::std::os::raw::c_uint;
pub type uint_least64_t = ::std::os::raw::c_ulong;
pub type int_fast8_t = ::std::os::raw::c_schar;
pub type int_fast16_t = ::std::os::raw::c_long;
pub type int_fast32_t = ::std::os::raw::c_long;
pub type int_fast64_t = ::std::os::raw::c_long;
pub type uint_fast8_t = ::std::os::raw::c_uchar;
pub type uint_fast16_t = ::std::os::raw::c_ulong;
pub type uint_fast32_t = ::std::os::raw::c_ulong;
pub type uint_fast64_t = ::std::os::raw::c_ulong;
pub type intmax_t = __intmax_t;
pub type uintmax_t = __uintmax_t;
pub type VkFlags = u32;
pub type VkBool32 = u32;
pub type VkDeviceSize = u64;
pub type VkSampleMask = u32;

pub const VkPipelineCacheHeaderVersion_VK_PIPELINE_CACHE_HEADER_VERSION_BEGIN_RANGE:
    VkPipelineCacheHeaderVersion =
    VkPipelineCacheHeaderVersion::VK_PIPELINE_CACHE_HEADER_VERSION_ONE;
pub const VkPipelineCacheHeaderVersion_VK_PIPELINE_CACHE_HEADER_VERSION_END_RANGE:
    VkPipelineCacheHeaderVersion =
    VkPipelineCacheHeaderVersion::VK_PIPELINE_CACHE_HEADER_VERSION_ONE;
pub const VkPipelineCacheHeaderVersion_VK_PIPELINE_CACHE_HEADER_VERSION_RANGE_SIZE:
    VkPipelineCacheHeaderVersion =
    VkPipelineCacheHeaderVersion::VK_PIPELINE_CACHE_HEADER_VERSION_ONE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPipelineCacheHeaderVersion {
    VK_PIPELINE_CACHE_HEADER_VERSION_ONE = 1,
    VK_PIPELINE_CACHE_HEADER_VERSION_MAX_ENUM = 2147483647,
}
pub const VkResult_VK_RESULT_BEGIN_RANGE: VkResult = VkResult::VK_ERROR_FRAGMENTED_POOL;
pub const VkResult_VK_RESULT_END_RANGE: VkResult = VkResult::VK_INCOMPLETE;
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkResult {
    VK_SUCCESS = 0,
    VK_NOT_READY = 1,
    VK_TIMEOUT = 2,
    VK_EVENT_SET = 3,
    VK_EVENT_RESET = 4,
    VK_INCOMPLETE = 5,
    VK_ERROR_OUT_OF_HOST_MEMORY = -1,
    VK_ERROR_OUT_OF_DEVICE_MEMORY = -2,
    VK_ERROR_INITIALIZATION_FAILED = -3,
    VK_ERROR_DEVICE_LOST = -4,
    VK_ERROR_MEMORY_MAP_FAILED = -5,
    VK_ERROR_LAYER_NOT_PRESENT = -6,
    VK_ERROR_EXTENSION_NOT_PRESENT = -7,
    VK_ERROR_FEATURE_NOT_PRESENT = -8,
    VK_ERROR_INCOMPATIBLE_DRIVER = -9,
    VK_ERROR_TOO_MANY_OBJECTS = -10,
    VK_ERROR_FORMAT_NOT_SUPPORTED = -11,
    VK_ERROR_FRAGMENTED_POOL = -12,
    VK_ERROR_SURFACE_LOST_KHR = -1000000000,
    VK_ERROR_NATIVE_WINDOW_IN_USE_KHR = -1000000001,
    VK_SUBOPTIMAL_KHR = 1000001003,
    VK_ERROR_OUT_OF_DATE_KHR = -1000001004,
    VK_ERROR_INCOMPATIBLE_DISPLAY_KHR = -1000003001,
    VK_ERROR_VALIDATION_FAILED_EXT = -1000011001,
    VK_ERROR_INVALID_SHADER_NV = -1000012000,
    VK_ERROR_OUT_OF_POOL_MEMORY_KHR = -1000069000,
    VK_ERROR_INVALID_EXTERNAL_HANDLE_KHX = -1000072003,
    VK_RESULT_RANGE_SIZE = 18,
    VK_RESULT_MAX_ENUM = 2147483647,
}
pub const VkStructureType_VK_STRUCTURE_TYPE_BEGIN_RANGE: VkStructureType =
    VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO;
pub const VkStructureType_VK_STRUCTURE_TYPE_END_RANGE: VkStructureType =
    VkStructureType::VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkStructureType {
    VK_STRUCTURE_TYPE_APPLICATION_INFO = 0,
    VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO = 1,
    VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO = 2,
    VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO = 3,
    VK_STRUCTURE_TYPE_SUBMIT_INFO = 4,
    VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO = 5,
    VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE = 6,
    VK_STRUCTURE_TYPE_BIND_SPARSE_INFO = 7,
    VK_STRUCTURE_TYPE_FENCE_CREATE_INFO = 8,
    VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO = 9,
    VK_STRUCTURE_TYPE_EVENT_CREATE_INFO = 10,
    VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO = 11,
    VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO = 12,
    VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO = 13,
    VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO = 14,
    VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO = 15,
    VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO = 16,
    VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO = 17,
    VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO = 18,
    VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO = 19,
    VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO = 20,
    VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO = 21,
    VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO = 22,
    VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO = 23,
    VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO = 24,
    VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO = 25,
    VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO = 26,
    VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO = 27,
    VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO = 28,
    VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO = 29,
    VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO = 30,
    VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO = 31,
    VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO = 32,
    VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO = 33,
    VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO = 34,
    VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET = 35,
    VK_STRUCTURE_TYPE_COPY_DESCRIPTOR_SET = 36,
    VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO = 37,
    VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO = 38,
    VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO = 39,
    VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO = 40,
    VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_INFO = 41,
    VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO = 42,
    VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO = 43,
    VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER = 44,
    VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER = 45,
    VK_STRUCTURE_TYPE_MEMORY_BARRIER = 46,
    VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO = 47,
    VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO = 48,
    VK_STRUCTURE_TYPE_RANGE_SIZE = 49,
    VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR = 1000001000,
    VK_STRUCTURE_TYPE_PRESENT_INFO_KHR = 1000001001,
    VK_STRUCTURE_TYPE_DISPLAY_MODE_CREATE_INFO_KHR = 1000002000,
    VK_STRUCTURE_TYPE_DISPLAY_SURFACE_CREATE_INFO_KHR = 1000002001,
    VK_STRUCTURE_TYPE_DISPLAY_PRESENT_INFO_KHR = 1000003000,
    VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR = 1000004000,
    VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR = 1000005000,
    VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR = 1000006000,
    VK_STRUCTURE_TYPE_MIR_SURFACE_CREATE_INFO_KHR = 1000007000,
    VK_STRUCTURE_TYPE_ANDROID_SURFACE_CREATE_INFO_KHR = 1000008000,
    VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR = 1000009000,
    VK_STRUCTURE_TYPE_DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT = 1000011000,
    VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD = 1000018000,
    VK_STRUCTURE_TYPE_DEBUG_MARKER_OBJECT_NAME_INFO_EXT = 1000022000,
    VK_STRUCTURE_TYPE_DEBUG_MARKER_OBJECT_TAG_INFO_EXT = 1000022001,
    VK_STRUCTURE_TYPE_DEBUG_MARKER_MARKER_INFO_EXT = 1000022002,
    VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV = 1000026000,
    VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV = 1000026001,
    VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV = 1000026002,
    VK_STRUCTURE_TYPE_RENDER_PASS_MULTIVIEW_CREATE_INFO_KHX = 1000053000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES_KHX = 1000053001,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES_KHX = 1000053002,
    VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV = 1000056000,
    VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO_NV = 1000056001,
    VK_STRUCTURE_TYPE_IMPORT_MEMORY_WIN32_HANDLE_INFO_NV = 1000057000,
    VK_STRUCTURE_TYPE_EXPORT_MEMORY_WIN32_HANDLE_INFO_NV = 1000057001,
    VK_STRUCTURE_TYPE_WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV = 1000058000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2_KHR = 1000059000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2_KHR = 1000059001,
    VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2_KHR = 1000059002,
    VK_STRUCTURE_TYPE_IMAGE_FORMAT_PROPERTIES_2_KHR = 1000059003,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR = 1000059004,
    VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2_KHR = 1000059005,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2_KHR = 1000059006,
    VK_STRUCTURE_TYPE_SPARSE_IMAGE_FORMAT_PROPERTIES_2_KHR = 1000059007,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2_KHR = 1000059008,
    VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_FLAGS_INFO_KHX = 1000060000,
    VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO_KHX = 1000060001,
    VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO_KHX = 1000060002,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_RENDER_PASS_BEGIN_INFO_KHX = 1000060003,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO_KHX = 1000060004,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_SUBMIT_INFO_KHX = 1000060005,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_BIND_SPARSE_INFO_KHX = 1000060006,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_CAPABILITIES_KHX = 1000060007,
    VK_STRUCTURE_TYPE_IMAGE_SWAPCHAIN_CREATE_INFO_KHX = 1000060008,
    VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHX = 1000060009,
    VK_STRUCTURE_TYPE_ACQUIRE_NEXT_IMAGE_INFO_KHX = 1000060010,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_INFO_KHX = 1000060011,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHX = 1000060012,
    VK_STRUCTURE_TYPE_VALIDATION_FLAGS_EXT = 1000061000,
    VK_STRUCTURE_TYPE_VI_SURFACE_CREATE_INFO_NN = 1000062000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_GROUP_PROPERTIES_KHX = 1000070000,
    VK_STRUCTURE_TYPE_DEVICE_GROUP_DEVICE_CREATE_INFO_KHX = 1000070001,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO_KHX = 1000071000,
    VK_STRUCTURE_TYPE_EXTERNAL_IMAGE_FORMAT_PROPERTIES_KHX = 1000071001,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO_KHX = 1000071002,
    VK_STRUCTURE_TYPE_EXTERNAL_BUFFER_PROPERTIES_KHX = 1000071003,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES_KHX = 1000071004,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2_KHX = 1000071005,
    VK_STRUCTURE_TYPE_IMAGE_FORMAT_PROPERTIES_2_KHX = 1000071006,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHX = 1000071007,
    VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_BUFFER_CREATE_INFO_KHX = 1000072000,
    VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO_KHX = 1000072001,
    VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO_KHX = 1000072002,
    VK_STRUCTURE_TYPE_IMPORT_MEMORY_WIN32_HANDLE_INFO_KHX = 1000073000,
    VK_STRUCTURE_TYPE_EXPORT_MEMORY_WIN32_HANDLE_INFO_KHX = 1000073001,
    VK_STRUCTURE_TYPE_MEMORY_WIN32_HANDLE_PROPERTIES_KHX = 1000073002,
    VK_STRUCTURE_TYPE_IMPORT_MEMORY_FD_INFO_KHX = 1000074000,
    VK_STRUCTURE_TYPE_MEMORY_FD_PROPERTIES_KHX = 1000074001,
    VK_STRUCTURE_TYPE_WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHX = 1000075000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO_KHX = 1000076000,
    VK_STRUCTURE_TYPE_EXTERNAL_SEMAPHORE_PROPERTIES_KHX = 1000076001,
    VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_CREATE_INFO_KHX = 1000077000,
    VK_STRUCTURE_TYPE_IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHX = 1000078000,
    VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHX = 1000078001,
    VK_STRUCTURE_TYPE_D3D12_FENCE_SUBMIT_INFO_KHX = 1000078002,
    VK_STRUCTURE_TYPE_IMPORT_SEMAPHORE_FD_INFO_KHX = 1000079000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR = 1000080000,
    VK_STRUCTURE_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO_KHR = 1000085000,
    VK_STRUCTURE_TYPE_OBJECT_TABLE_CREATE_INFO_NVX = 1000086000,
    VK_STRUCTURE_TYPE_INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX = 1000086001,
    VK_STRUCTURE_TYPE_CMD_PROCESS_COMMANDS_INFO_NVX = 1000086002,
    VK_STRUCTURE_TYPE_CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX = 1000086003,
    VK_STRUCTURE_TYPE_DEVICE_GENERATED_COMMANDS_LIMITS_NVX = 1000086004,
    VK_STRUCTURE_TYPE_DEVICE_GENERATED_COMMANDS_FEATURES_NVX = 1000086005,
    VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV = 1000087000,
    VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES2_EXT = 1000090000,
    VK_STRUCTURE_TYPE_DISPLAY_POWER_INFO_EXT = 1000091000,
    VK_STRUCTURE_TYPE_DEVICE_EVENT_INFO_EXT = 1000091001,
    VK_STRUCTURE_TYPE_DISPLAY_EVENT_INFO_EXT = 1000091002,
    VK_STRUCTURE_TYPE_SWAPCHAIN_COUNTER_CREATE_INFO_EXT = 1000091003,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX = 1000097000,
    VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV = 1000098000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT = 1000099000,
    VK_STRUCTURE_TYPE_PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT = 1000099001,
    VK_STRUCTURE_TYPE_IOS_SURFACE_CREATE_INFO_MVK = 1000122000,
    VK_STRUCTURE_TYPE_MACOS_SURFACE_CREATE_INFO_MVK = 1000123000,
    VK_STRUCTURE_TYPE_METAL_SURFACE_CREATE_INFO_EXT = 1000248000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PORTABILITY_SUBSET_FEATURES_EXTX = 1000163000,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PORTABILITY_SUBSET_PROPERTIES_EXTX = 1000163001,
    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_VIEW_SUPPORT_EXTX = 100163002,
    VK_STRUCTURE_TYPE_MAX_ENUM = 2147483647,
}
pub const VkSystemAllocationScope_VK_SYSTEM_ALLOCATION_SCOPE_BEGIN_RANGE: VkSystemAllocationScope =
    VkSystemAllocationScope::VK_SYSTEM_ALLOCATION_SCOPE_COMMAND;
pub const VkSystemAllocationScope_VK_SYSTEM_ALLOCATION_SCOPE_END_RANGE: VkSystemAllocationScope =
    VkSystemAllocationScope::VK_SYSTEM_ALLOCATION_SCOPE_INSTANCE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSystemAllocationScope {
    VK_SYSTEM_ALLOCATION_SCOPE_COMMAND = 0,
    VK_SYSTEM_ALLOCATION_SCOPE_OBJECT = 1,
    VK_SYSTEM_ALLOCATION_SCOPE_CACHE = 2,
    VK_SYSTEM_ALLOCATION_SCOPE_DEVICE = 3,
    VK_SYSTEM_ALLOCATION_SCOPE_INSTANCE = 4,
    VK_SYSTEM_ALLOCATION_SCOPE_RANGE_SIZE = 5,
    VK_SYSTEM_ALLOCATION_SCOPE_MAX_ENUM = 2147483647,
}
pub const VkInternalAllocationType_VK_INTERNAL_ALLOCATION_TYPE_BEGIN_RANGE:
    VkInternalAllocationType = VkInternalAllocationType::VK_INTERNAL_ALLOCATION_TYPE_EXECUTABLE;
pub const VkInternalAllocationType_VK_INTERNAL_ALLOCATION_TYPE_END_RANGE: VkInternalAllocationType =
    VkInternalAllocationType::VK_INTERNAL_ALLOCATION_TYPE_EXECUTABLE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkInternalAllocationType {
    VK_INTERNAL_ALLOCATION_TYPE_EXECUTABLE = 0,
    VK_INTERNAL_ALLOCATION_TYPE_RANGE_SIZE = 1,
    VK_INTERNAL_ALLOCATION_TYPE_MAX_ENUM = 2147483647,
}
pub const VkFormat_VK_FORMAT_BEGIN_RANGE: VkFormat = VkFormat::VK_FORMAT_UNDEFINED;
pub const VkFormat_VK_FORMAT_END_RANGE: VkFormat = VkFormat::VK_FORMAT_ASTC_12x12_SRGB_BLOCK;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkFormat {
    VK_FORMAT_UNDEFINED = 0,
    VK_FORMAT_R4G4_UNORM_PACK8 = 1,
    VK_FORMAT_R4G4B4A4_UNORM_PACK16 = 2,
    VK_FORMAT_B4G4R4A4_UNORM_PACK16 = 3,
    VK_FORMAT_R5G6B5_UNORM_PACK16 = 4,
    VK_FORMAT_B5G6R5_UNORM_PACK16 = 5,
    VK_FORMAT_R5G5B5A1_UNORM_PACK16 = 6,
    VK_FORMAT_B5G5R5A1_UNORM_PACK16 = 7,
    VK_FORMAT_A1R5G5B5_UNORM_PACK16 = 8,
    VK_FORMAT_R8_UNORM = 9,
    VK_FORMAT_R8_SNORM = 10,
    VK_FORMAT_R8_USCALED = 11,
    VK_FORMAT_R8_SSCALED = 12,
    VK_FORMAT_R8_UINT = 13,
    VK_FORMAT_R8_SINT = 14,
    VK_FORMAT_R8_SRGB = 15,
    VK_FORMAT_R8G8_UNORM = 16,
    VK_FORMAT_R8G8_SNORM = 17,
    VK_FORMAT_R8G8_USCALED = 18,
    VK_FORMAT_R8G8_SSCALED = 19,
    VK_FORMAT_R8G8_UINT = 20,
    VK_FORMAT_R8G8_SINT = 21,
    VK_FORMAT_R8G8_SRGB = 22,
    VK_FORMAT_R8G8B8_UNORM = 23,
    VK_FORMAT_R8G8B8_SNORM = 24,
    VK_FORMAT_R8G8B8_USCALED = 25,
    VK_FORMAT_R8G8B8_SSCALED = 26,
    VK_FORMAT_R8G8B8_UINT = 27,
    VK_FORMAT_R8G8B8_SINT = 28,
    VK_FORMAT_R8G8B8_SRGB = 29,
    VK_FORMAT_B8G8R8_UNORM = 30,
    VK_FORMAT_B8G8R8_SNORM = 31,
    VK_FORMAT_B8G8R8_USCALED = 32,
    VK_FORMAT_B8G8R8_SSCALED = 33,
    VK_FORMAT_B8G8R8_UINT = 34,
    VK_FORMAT_B8G8R8_SINT = 35,
    VK_FORMAT_B8G8R8_SRGB = 36,
    VK_FORMAT_R8G8B8A8_UNORM = 37,
    VK_FORMAT_R8G8B8A8_SNORM = 38,
    VK_FORMAT_R8G8B8A8_USCALED = 39,
    VK_FORMAT_R8G8B8A8_SSCALED = 40,
    VK_FORMAT_R8G8B8A8_UINT = 41,
    VK_FORMAT_R8G8B8A8_SINT = 42,
    VK_FORMAT_R8G8B8A8_SRGB = 43,
    VK_FORMAT_B8G8R8A8_UNORM = 44,
    VK_FORMAT_B8G8R8A8_SNORM = 45,
    VK_FORMAT_B8G8R8A8_USCALED = 46,
    VK_FORMAT_B8G8R8A8_SSCALED = 47,
    VK_FORMAT_B8G8R8A8_UINT = 48,
    VK_FORMAT_B8G8R8A8_SINT = 49,
    VK_FORMAT_B8G8R8A8_SRGB = 50,
    VK_FORMAT_A8B8G8R8_UNORM_PACK32 = 51,
    VK_FORMAT_A8B8G8R8_SNORM_PACK32 = 52,
    VK_FORMAT_A8B8G8R8_USCALED_PACK32 = 53,
    VK_FORMAT_A8B8G8R8_SSCALED_PACK32 = 54,
    VK_FORMAT_A8B8G8R8_UINT_PACK32 = 55,
    VK_FORMAT_A8B8G8R8_SINT_PACK32 = 56,
    VK_FORMAT_A8B8G8R8_SRGB_PACK32 = 57,
    VK_FORMAT_A2R10G10B10_UNORM_PACK32 = 58,
    VK_FORMAT_A2R10G10B10_SNORM_PACK32 = 59,
    VK_FORMAT_A2R10G10B10_USCALED_PACK32 = 60,
    VK_FORMAT_A2R10G10B10_SSCALED_PACK32 = 61,
    VK_FORMAT_A2R10G10B10_UINT_PACK32 = 62,
    VK_FORMAT_A2R10G10B10_SINT_PACK32 = 63,
    VK_FORMAT_A2B10G10R10_UNORM_PACK32 = 64,
    VK_FORMAT_A2B10G10R10_SNORM_PACK32 = 65,
    VK_FORMAT_A2B10G10R10_USCALED_PACK32 = 66,
    VK_FORMAT_A2B10G10R10_SSCALED_PACK32 = 67,
    VK_FORMAT_A2B10G10R10_UINT_PACK32 = 68,
    VK_FORMAT_A2B10G10R10_SINT_PACK32 = 69,
    VK_FORMAT_R16_UNORM = 70,
    VK_FORMAT_R16_SNORM = 71,
    VK_FORMAT_R16_USCALED = 72,
    VK_FORMAT_R16_SSCALED = 73,
    VK_FORMAT_R16_UINT = 74,
    VK_FORMAT_R16_SINT = 75,
    VK_FORMAT_R16_SFLOAT = 76,
    VK_FORMAT_R16G16_UNORM = 77,
    VK_FORMAT_R16G16_SNORM = 78,
    VK_FORMAT_R16G16_USCALED = 79,
    VK_FORMAT_R16G16_SSCALED = 80,
    VK_FORMAT_R16G16_UINT = 81,
    VK_FORMAT_R16G16_SINT = 82,
    VK_FORMAT_R16G16_SFLOAT = 83,
    VK_FORMAT_R16G16B16_UNORM = 84,
    VK_FORMAT_R16G16B16_SNORM = 85,
    VK_FORMAT_R16G16B16_USCALED = 86,
    VK_FORMAT_R16G16B16_SSCALED = 87,
    VK_FORMAT_R16G16B16_UINT = 88,
    VK_FORMAT_R16G16B16_SINT = 89,
    VK_FORMAT_R16G16B16_SFLOAT = 90,
    VK_FORMAT_R16G16B16A16_UNORM = 91,
    VK_FORMAT_R16G16B16A16_SNORM = 92,
    VK_FORMAT_R16G16B16A16_USCALED = 93,
    VK_FORMAT_R16G16B16A16_SSCALED = 94,
    VK_FORMAT_R16G16B16A16_UINT = 95,
    VK_FORMAT_R16G16B16A16_SINT = 96,
    VK_FORMAT_R16G16B16A16_SFLOAT = 97,
    VK_FORMAT_R32_UINT = 98,
    VK_FORMAT_R32_SINT = 99,
    VK_FORMAT_R32_SFLOAT = 100,
    VK_FORMAT_R32G32_UINT = 101,
    VK_FORMAT_R32G32_SINT = 102,
    VK_FORMAT_R32G32_SFLOAT = 103,
    VK_FORMAT_R32G32B32_UINT = 104,
    VK_FORMAT_R32G32B32_SINT = 105,
    VK_FORMAT_R32G32B32_SFLOAT = 106,
    VK_FORMAT_R32G32B32A32_UINT = 107,
    VK_FORMAT_R32G32B32A32_SINT = 108,
    VK_FORMAT_R32G32B32A32_SFLOAT = 109,
    VK_FORMAT_R64_UINT = 110,
    VK_FORMAT_R64_SINT = 111,
    VK_FORMAT_R64_SFLOAT = 112,
    VK_FORMAT_R64G64_UINT = 113,
    VK_FORMAT_R64G64_SINT = 114,
    VK_FORMAT_R64G64_SFLOAT = 115,
    VK_FORMAT_R64G64B64_UINT = 116,
    VK_FORMAT_R64G64B64_SINT = 117,
    VK_FORMAT_R64G64B64_SFLOAT = 118,
    VK_FORMAT_R64G64B64A64_UINT = 119,
    VK_FORMAT_R64G64B64A64_SINT = 120,
    VK_FORMAT_R64G64B64A64_SFLOAT = 121,
    VK_FORMAT_B10G11R11_UFLOAT_PACK32 = 122,
    VK_FORMAT_E5B9G9R9_UFLOAT_PACK32 = 123,
    VK_FORMAT_D16_UNORM = 124,
    VK_FORMAT_X8_D24_UNORM_PACK32 = 125,
    VK_FORMAT_D32_SFLOAT = 126,
    VK_FORMAT_S8_UINT = 127,
    VK_FORMAT_D16_UNORM_S8_UINT = 128,
    VK_FORMAT_D24_UNORM_S8_UINT = 129,
    VK_FORMAT_D32_SFLOAT_S8_UINT = 130,
    VK_FORMAT_BC1_RGB_UNORM_BLOCK = 131,
    VK_FORMAT_BC1_RGB_SRGB_BLOCK = 132,
    VK_FORMAT_BC1_RGBA_UNORM_BLOCK = 133,
    VK_FORMAT_BC1_RGBA_SRGB_BLOCK = 134,
    VK_FORMAT_BC2_UNORM_BLOCK = 135,
    VK_FORMAT_BC2_SRGB_BLOCK = 136,
    VK_FORMAT_BC3_UNORM_BLOCK = 137,
    VK_FORMAT_BC3_SRGB_BLOCK = 138,
    VK_FORMAT_BC4_UNORM_BLOCK = 139,
    VK_FORMAT_BC4_SNORM_BLOCK = 140,
    VK_FORMAT_BC5_UNORM_BLOCK = 141,
    VK_FORMAT_BC5_SNORM_BLOCK = 142,
    VK_FORMAT_BC6H_UFLOAT_BLOCK = 143,
    VK_FORMAT_BC6H_SFLOAT_BLOCK = 144,
    VK_FORMAT_BC7_UNORM_BLOCK = 145,
    VK_FORMAT_BC7_SRGB_BLOCK = 146,
    VK_FORMAT_ETC2_R8G8B8_UNORM_BLOCK = 147,
    VK_FORMAT_ETC2_R8G8B8_SRGB_BLOCK = 148,
    VK_FORMAT_ETC2_R8G8B8A1_UNORM_BLOCK = 149,
    VK_FORMAT_ETC2_R8G8B8A1_SRGB_BLOCK = 150,
    VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK = 151,
    VK_FORMAT_ETC2_R8G8B8A8_SRGB_BLOCK = 152,
    VK_FORMAT_EAC_R11_UNORM_BLOCK = 153,
    VK_FORMAT_EAC_R11_SNORM_BLOCK = 154,
    VK_FORMAT_EAC_R11G11_UNORM_BLOCK = 155,
    VK_FORMAT_EAC_R11G11_SNORM_BLOCK = 156,
    VK_FORMAT_ASTC_4x4_UNORM_BLOCK = 157,
    VK_FORMAT_ASTC_4x4_SRGB_BLOCK = 158,
    VK_FORMAT_ASTC_5x4_UNORM_BLOCK = 159,
    VK_FORMAT_ASTC_5x4_SRGB_BLOCK = 160,
    VK_FORMAT_ASTC_5x5_UNORM_BLOCK = 161,
    VK_FORMAT_ASTC_5x5_SRGB_BLOCK = 162,
    VK_FORMAT_ASTC_6x5_UNORM_BLOCK = 163,
    VK_FORMAT_ASTC_6x5_SRGB_BLOCK = 164,
    VK_FORMAT_ASTC_6x6_UNORM_BLOCK = 165,
    VK_FORMAT_ASTC_6x6_SRGB_BLOCK = 166,
    VK_FORMAT_ASTC_8x5_UNORM_BLOCK = 167,
    VK_FORMAT_ASTC_8x5_SRGB_BLOCK = 168,
    VK_FORMAT_ASTC_8x6_UNORM_BLOCK = 169,
    VK_FORMAT_ASTC_8x6_SRGB_BLOCK = 170,
    VK_FORMAT_ASTC_8x8_UNORM_BLOCK = 171,
    VK_FORMAT_ASTC_8x8_SRGB_BLOCK = 172,
    VK_FORMAT_ASTC_10x5_UNORM_BLOCK = 173,
    VK_FORMAT_ASTC_10x5_SRGB_BLOCK = 174,
    VK_FORMAT_ASTC_10x6_UNORM_BLOCK = 175,
    VK_FORMAT_ASTC_10x6_SRGB_BLOCK = 176,
    VK_FORMAT_ASTC_10x8_UNORM_BLOCK = 177,
    VK_FORMAT_ASTC_10x8_SRGB_BLOCK = 178,
    VK_FORMAT_ASTC_10x10_UNORM_BLOCK = 179,
    VK_FORMAT_ASTC_10x10_SRGB_BLOCK = 180,
    VK_FORMAT_ASTC_12x10_UNORM_BLOCK = 181,
    VK_FORMAT_ASTC_12x10_SRGB_BLOCK = 182,
    VK_FORMAT_ASTC_12x12_UNORM_BLOCK = 183,
    VK_FORMAT_ASTC_12x12_SRGB_BLOCK = 184,
    VK_FORMAT_PVRTC1_2BPP_UNORM_BLOCK_IMG = 1000054000,
    VK_FORMAT_PVRTC1_4BPP_UNORM_BLOCK_IMG = 1000054001,
    VK_FORMAT_PVRTC2_2BPP_UNORM_BLOCK_IMG = 1000054002,
    VK_FORMAT_PVRTC2_4BPP_UNORM_BLOCK_IMG = 1000054003,
    VK_FORMAT_PVRTC1_2BPP_SRGB_BLOCK_IMG = 1000054004,
    VK_FORMAT_PVRTC1_4BPP_SRGB_BLOCK_IMG = 1000054005,
    VK_FORMAT_PVRTC2_2BPP_SRGB_BLOCK_IMG = 1000054006,
    VK_FORMAT_PVRTC2_4BPP_SRGB_BLOCK_IMG = 1000054007,
    VK_FORMAT_RANGE_SIZE = 185,
    VK_FORMAT_MAX_ENUM = 2147483647,
}
pub const VkImageType_VK_IMAGE_TYPE_BEGIN_RANGE: VkImageType = VkImageType::VK_IMAGE_TYPE_1D;
pub const VkImageType_VK_IMAGE_TYPE_END_RANGE: VkImageType = VkImageType::VK_IMAGE_TYPE_3D;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkImageType {
    VK_IMAGE_TYPE_1D = 0,
    VK_IMAGE_TYPE_2D = 1,
    VK_IMAGE_TYPE_3D = 2,
    VK_IMAGE_TYPE_RANGE_SIZE = 3,
    VK_IMAGE_TYPE_MAX_ENUM = 2147483647,
}
pub const VkImageTiling_VK_IMAGE_TILING_BEGIN_RANGE: VkImageTiling =
    VkImageTiling::VK_IMAGE_TILING_OPTIMAL;
pub const VkImageTiling_VK_IMAGE_TILING_END_RANGE: VkImageTiling =
    VkImageTiling::VK_IMAGE_TILING_LINEAR;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkImageTiling {
    VK_IMAGE_TILING_OPTIMAL = 0,
    VK_IMAGE_TILING_LINEAR = 1,
    VK_IMAGE_TILING_RANGE_SIZE = 2,
    VK_IMAGE_TILING_MAX_ENUM = 2147483647,
}
pub const VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_BEGIN_RANGE: VkPhysicalDeviceType =
    VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_OTHER;
pub const VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_END_RANGE: VkPhysicalDeviceType =
    VkPhysicalDeviceType::VK_PHYSICAL_DEVICE_TYPE_CPU;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPhysicalDeviceType {
    VK_PHYSICAL_DEVICE_TYPE_OTHER = 0,
    VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU = 1,
    VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU = 2,
    VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU = 3,
    VK_PHYSICAL_DEVICE_TYPE_CPU = 4,
    VK_PHYSICAL_DEVICE_TYPE_RANGE_SIZE = 5,
    VK_PHYSICAL_DEVICE_TYPE_MAX_ENUM = 2147483647,
}
pub const VkQueryType_VK_QUERY_TYPE_BEGIN_RANGE: VkQueryType = VkQueryType::VK_QUERY_TYPE_OCCLUSION;
pub const VkQueryType_VK_QUERY_TYPE_END_RANGE: VkQueryType = VkQueryType::VK_QUERY_TYPE_TIMESTAMP;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkQueryType {
    VK_QUERY_TYPE_OCCLUSION = 0,
    VK_QUERY_TYPE_PIPELINE_STATISTICS = 1,
    VK_QUERY_TYPE_TIMESTAMP = 2,
    VK_QUERY_TYPE_RANGE_SIZE = 3,
    VK_QUERY_TYPE_MAX_ENUM = 2147483647,
}
pub const VkSharingMode_VK_SHARING_MODE_BEGIN_RANGE: VkSharingMode =
    VkSharingMode::VK_SHARING_MODE_EXCLUSIVE;
pub const VkSharingMode_VK_SHARING_MODE_END_RANGE: VkSharingMode =
    VkSharingMode::VK_SHARING_MODE_CONCURRENT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSharingMode {
    VK_SHARING_MODE_EXCLUSIVE = 0,
    VK_SHARING_MODE_CONCURRENT = 1,
    VK_SHARING_MODE_RANGE_SIZE = 2,
    VK_SHARING_MODE_MAX_ENUM = 2147483647,
}
pub const VkImageLayout_VK_IMAGE_LAYOUT_BEGIN_RANGE: VkImageLayout =
    VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED;
pub const VkImageLayout_VK_IMAGE_LAYOUT_END_RANGE: VkImageLayout =
    VkImageLayout::VK_IMAGE_LAYOUT_PREINITIALIZED;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkImageLayout {
    VK_IMAGE_LAYOUT_UNDEFINED = 0,
    VK_IMAGE_LAYOUT_GENERAL = 1,
    VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL = 2,
    VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL = 3,
    VK_IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL = 4,
    VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL = 5,
    VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL = 6,
    VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL = 7,
    VK_IMAGE_LAYOUT_PREINITIALIZED = 8,
    VK_IMAGE_LAYOUT_PRESENT_SRC_KHR = 1000001002,
    VK_IMAGE_LAYOUT_RANGE_SIZE = 9,
    VK_IMAGE_LAYOUT_MAX_ENUM = 2147483647,
}
pub const VkImageViewType_VK_IMAGE_VIEW_TYPE_BEGIN_RANGE: VkImageViewType =
    VkImageViewType::VK_IMAGE_VIEW_TYPE_1D;
pub const VkImageViewType_VK_IMAGE_VIEW_TYPE_END_RANGE: VkImageViewType =
    VkImageViewType::VK_IMAGE_VIEW_TYPE_CUBE_ARRAY;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkImageViewType {
    VK_IMAGE_VIEW_TYPE_1D = 0,
    VK_IMAGE_VIEW_TYPE_2D = 1,
    VK_IMAGE_VIEW_TYPE_3D = 2,
    VK_IMAGE_VIEW_TYPE_CUBE = 3,
    VK_IMAGE_VIEW_TYPE_1D_ARRAY = 4,
    VK_IMAGE_VIEW_TYPE_2D_ARRAY = 5,
    VK_IMAGE_VIEW_TYPE_CUBE_ARRAY = 6,
    VK_IMAGE_VIEW_TYPE_RANGE_SIZE = 7,
    VK_IMAGE_VIEW_TYPE_MAX_ENUM = 2147483647,
}
pub const VkComponentSwizzle_VK_COMPONENT_SWIZZLE_BEGIN_RANGE: VkComponentSwizzle =
    VkComponentSwizzle::VK_COMPONENT_SWIZZLE_IDENTITY;
pub const VkComponentSwizzle_VK_COMPONENT_SWIZZLE_END_RANGE: VkComponentSwizzle =
    VkComponentSwizzle::VK_COMPONENT_SWIZZLE_A;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkComponentSwizzle {
    VK_COMPONENT_SWIZZLE_IDENTITY = 0,
    VK_COMPONENT_SWIZZLE_ZERO = 1,
    VK_COMPONENT_SWIZZLE_ONE = 2,
    VK_COMPONENT_SWIZZLE_R = 3,
    VK_COMPONENT_SWIZZLE_G = 4,
    VK_COMPONENT_SWIZZLE_B = 5,
    VK_COMPONENT_SWIZZLE_A = 6,
    VK_COMPONENT_SWIZZLE_RANGE_SIZE = 7,
    VK_COMPONENT_SWIZZLE_MAX_ENUM = 2147483647,
}
pub const VkVertexInputRate_VK_VERTEX_INPUT_RATE_BEGIN_RANGE: VkVertexInputRate =
    VkVertexInputRate::VK_VERTEX_INPUT_RATE_VERTEX;
pub const VkVertexInputRate_VK_VERTEX_INPUT_RATE_END_RANGE: VkVertexInputRate =
    VkVertexInputRate::VK_VERTEX_INPUT_RATE_INSTANCE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkVertexInputRate {
    VK_VERTEX_INPUT_RATE_VERTEX = 0,
    VK_VERTEX_INPUT_RATE_INSTANCE = 1,
    VK_VERTEX_INPUT_RATE_RANGE_SIZE = 2,
    VK_VERTEX_INPUT_RATE_MAX_ENUM = 2147483647,
}
pub const VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_BEGIN_RANGE: VkPrimitiveTopology =
    VkPrimitiveTopology::VK_PRIMITIVE_TOPOLOGY_POINT_LIST;
pub const VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_END_RANGE: VkPrimitiveTopology =
    VkPrimitiveTopology::VK_PRIMITIVE_TOPOLOGY_PATCH_LIST;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPrimitiveTopology {
    VK_PRIMITIVE_TOPOLOGY_POINT_LIST = 0,
    VK_PRIMITIVE_TOPOLOGY_LINE_LIST = 1,
    VK_PRIMITIVE_TOPOLOGY_LINE_STRIP = 2,
    VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST = 3,
    VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP = 4,
    VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN = 5,
    VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY = 6,
    VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY = 7,
    VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY = 8,
    VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY = 9,
    VK_PRIMITIVE_TOPOLOGY_PATCH_LIST = 10,
    VK_PRIMITIVE_TOPOLOGY_RANGE_SIZE = 11,
    VK_PRIMITIVE_TOPOLOGY_MAX_ENUM = 2147483647,
}
pub const VkPolygonMode_VK_POLYGON_MODE_BEGIN_RANGE: VkPolygonMode =
    VkPolygonMode::VK_POLYGON_MODE_FILL;
pub const VkPolygonMode_VK_POLYGON_MODE_END_RANGE: VkPolygonMode =
    VkPolygonMode::VK_POLYGON_MODE_POINT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPolygonMode {
    VK_POLYGON_MODE_FILL = 0,
    VK_POLYGON_MODE_LINE = 1,
    VK_POLYGON_MODE_POINT = 2,
    VK_POLYGON_MODE_RANGE_SIZE = 3,
    VK_POLYGON_MODE_MAX_ENUM = 2147483647,
}
pub const VkFrontFace_VK_FRONT_FACE_BEGIN_RANGE: VkFrontFace =
    VkFrontFace::VK_FRONT_FACE_COUNTER_CLOCKWISE;
pub const VkFrontFace_VK_FRONT_FACE_END_RANGE: VkFrontFace = VkFrontFace::VK_FRONT_FACE_CLOCKWISE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkFrontFace {
    VK_FRONT_FACE_COUNTER_CLOCKWISE = 0,
    VK_FRONT_FACE_CLOCKWISE = 1,
    VK_FRONT_FACE_RANGE_SIZE = 2,
    VK_FRONT_FACE_MAX_ENUM = 2147483647,
}
pub const VkCompareOp_VK_COMPARE_OP_BEGIN_RANGE: VkCompareOp = VkCompareOp::VK_COMPARE_OP_NEVER;
pub const VkCompareOp_VK_COMPARE_OP_END_RANGE: VkCompareOp = VkCompareOp::VK_COMPARE_OP_ALWAYS;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCompareOp {
    VK_COMPARE_OP_NEVER = 0,
    VK_COMPARE_OP_LESS = 1,
    VK_COMPARE_OP_EQUAL = 2,
    VK_COMPARE_OP_LESS_OR_EQUAL = 3,
    VK_COMPARE_OP_GREATER = 4,
    VK_COMPARE_OP_NOT_EQUAL = 5,
    VK_COMPARE_OP_GREATER_OR_EQUAL = 6,
    VK_COMPARE_OP_ALWAYS = 7,
    VK_COMPARE_OP_RANGE_SIZE = 8,
    VK_COMPARE_OP_MAX_ENUM = 2147483647,
}
pub const VkStencilOp_VK_STENCIL_OP_BEGIN_RANGE: VkStencilOp = VkStencilOp::VK_STENCIL_OP_KEEP;
pub const VkStencilOp_VK_STENCIL_OP_END_RANGE: VkStencilOp =
    VkStencilOp::VK_STENCIL_OP_DECREMENT_AND_WRAP;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkStencilOp {
    VK_STENCIL_OP_KEEP = 0,
    VK_STENCIL_OP_ZERO = 1,
    VK_STENCIL_OP_REPLACE = 2,
    VK_STENCIL_OP_INCREMENT_AND_CLAMP = 3,
    VK_STENCIL_OP_DECREMENT_AND_CLAMP = 4,
    VK_STENCIL_OP_INVERT = 5,
    VK_STENCIL_OP_INCREMENT_AND_WRAP = 6,
    VK_STENCIL_OP_DECREMENT_AND_WRAP = 7,
    VK_STENCIL_OP_RANGE_SIZE = 8,
    VK_STENCIL_OP_MAX_ENUM = 2147483647,
}
pub const VkLogicOp_VK_LOGIC_OP_BEGIN_RANGE: VkLogicOp = VkLogicOp::VK_LOGIC_OP_CLEAR;
pub const VkLogicOp_VK_LOGIC_OP_END_RANGE: VkLogicOp = VkLogicOp::VK_LOGIC_OP_SET;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkLogicOp {
    VK_LOGIC_OP_CLEAR = 0,
    VK_LOGIC_OP_AND = 1,
    VK_LOGIC_OP_AND_REVERSE = 2,
    VK_LOGIC_OP_COPY = 3,
    VK_LOGIC_OP_AND_INVERTED = 4,
    VK_LOGIC_OP_NO_OP = 5,
    VK_LOGIC_OP_XOR = 6,
    VK_LOGIC_OP_OR = 7,
    VK_LOGIC_OP_NOR = 8,
    VK_LOGIC_OP_EQUIVALENT = 9,
    VK_LOGIC_OP_INVERT = 10,
    VK_LOGIC_OP_OR_REVERSE = 11,
    VK_LOGIC_OP_COPY_INVERTED = 12,
    VK_LOGIC_OP_OR_INVERTED = 13,
    VK_LOGIC_OP_NAND = 14,
    VK_LOGIC_OP_SET = 15,
    VK_LOGIC_OP_RANGE_SIZE = 16,
    VK_LOGIC_OP_MAX_ENUM = 2147483647,
}
pub const VkBlendFactor_VK_BLEND_FACTOR_BEGIN_RANGE: VkBlendFactor =
    VkBlendFactor::VK_BLEND_FACTOR_ZERO;
pub const VkBlendFactor_VK_BLEND_FACTOR_END_RANGE: VkBlendFactor =
    VkBlendFactor::VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkBlendFactor {
    VK_BLEND_FACTOR_ZERO = 0,
    VK_BLEND_FACTOR_ONE = 1,
    VK_BLEND_FACTOR_SRC_COLOR = 2,
    VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR = 3,
    VK_BLEND_FACTOR_DST_COLOR = 4,
    VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR = 5,
    VK_BLEND_FACTOR_SRC_ALPHA = 6,
    VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA = 7,
    VK_BLEND_FACTOR_DST_ALPHA = 8,
    VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA = 9,
    VK_BLEND_FACTOR_CONSTANT_COLOR = 10,
    VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR = 11,
    VK_BLEND_FACTOR_CONSTANT_ALPHA = 12,
    VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA = 13,
    VK_BLEND_FACTOR_SRC_ALPHA_SATURATE = 14,
    VK_BLEND_FACTOR_SRC1_COLOR = 15,
    VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR = 16,
    VK_BLEND_FACTOR_SRC1_ALPHA = 17,
    VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA = 18,
    VK_BLEND_FACTOR_RANGE_SIZE = 19,
    VK_BLEND_FACTOR_MAX_ENUM = 2147483647,
}
pub const VkBlendOp_VK_BLEND_OP_BEGIN_RANGE: VkBlendOp = VkBlendOp::VK_BLEND_OP_ADD;
pub const VkBlendOp_VK_BLEND_OP_END_RANGE: VkBlendOp = VkBlendOp::VK_BLEND_OP_MAX;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkBlendOp {
    VK_BLEND_OP_ADD = 0,
    VK_BLEND_OP_SUBTRACT = 1,
    VK_BLEND_OP_REVERSE_SUBTRACT = 2,
    VK_BLEND_OP_MIN = 3,
    VK_BLEND_OP_MAX = 4,
    VK_BLEND_OP_RANGE_SIZE = 5,
    VK_BLEND_OP_MAX_ENUM = 2147483647,
}
pub const VkDynamicState_VK_DYNAMIC_STATE_BEGIN_RANGE: VkDynamicState =
    VkDynamicState::VK_DYNAMIC_STATE_VIEWPORT;
pub const VkDynamicState_VK_DYNAMIC_STATE_END_RANGE: VkDynamicState =
    VkDynamicState::VK_DYNAMIC_STATE_STENCIL_REFERENCE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDynamicState {
    VK_DYNAMIC_STATE_VIEWPORT = 0,
    VK_DYNAMIC_STATE_SCISSOR = 1,
    VK_DYNAMIC_STATE_LINE_WIDTH = 2,
    VK_DYNAMIC_STATE_DEPTH_BIAS = 3,
    VK_DYNAMIC_STATE_BLEND_CONSTANTS = 4,
    VK_DYNAMIC_STATE_DEPTH_BOUNDS = 5,
    VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK = 6,
    VK_DYNAMIC_STATE_STENCIL_WRITE_MASK = 7,
    VK_DYNAMIC_STATE_STENCIL_REFERENCE = 8,
    VK_DYNAMIC_STATE_VIEWPORT_W_SCALING_NV = 1000087000,
    VK_DYNAMIC_STATE_DISCARD_RECTANGLE_EXT = 1000099000,
    VK_DYNAMIC_STATE_RANGE_SIZE = 9,
    VK_DYNAMIC_STATE_MAX_ENUM = 2147483647,
}
pub const VkFilter_VK_FILTER_BEGIN_RANGE: VkFilter = VkFilter::VK_FILTER_NEAREST;
pub const VkFilter_VK_FILTER_END_RANGE: VkFilter = VkFilter::VK_FILTER_LINEAR;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkFilter {
    VK_FILTER_NEAREST = 0,
    VK_FILTER_LINEAR = 1,
    VK_FILTER_CUBIC_IMG = 1000015000,
    VK_FILTER_RANGE_SIZE = 2,
    VK_FILTER_MAX_ENUM = 2147483647,
}
pub const VkSamplerMipmapMode_VK_SAMPLER_MIPMAP_MODE_BEGIN_RANGE: VkSamplerMipmapMode =
    VkSamplerMipmapMode::VK_SAMPLER_MIPMAP_MODE_NEAREST;
pub const VkSamplerMipmapMode_VK_SAMPLER_MIPMAP_MODE_END_RANGE: VkSamplerMipmapMode =
    VkSamplerMipmapMode::VK_SAMPLER_MIPMAP_MODE_LINEAR;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSamplerMipmapMode {
    VK_SAMPLER_MIPMAP_MODE_NEAREST = 0,
    VK_SAMPLER_MIPMAP_MODE_LINEAR = 1,
    VK_SAMPLER_MIPMAP_MODE_RANGE_SIZE = 2,
    VK_SAMPLER_MIPMAP_MODE_MAX_ENUM = 2147483647,
}
pub const VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_BEGIN_RANGE: VkSamplerAddressMode =
    VkSamplerAddressMode::VK_SAMPLER_ADDRESS_MODE_REPEAT;
pub const VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_END_RANGE: VkSamplerAddressMode =
    VkSamplerAddressMode::VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER;
pub const VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_RANGE_SIZE: VkSamplerAddressMode =
    VkSamplerAddressMode::VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSamplerAddressMode {
    VK_SAMPLER_ADDRESS_MODE_REPEAT = 0,
    VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT = 1,
    VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE = 2,
    VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER = 3,
    VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE = 4,
    VK_SAMPLER_ADDRESS_MODE_MAX_ENUM = 2147483647,
}
pub const VkBorderColor_VK_BORDER_COLOR_BEGIN_RANGE: VkBorderColor =
    VkBorderColor::VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK;
pub const VkBorderColor_VK_BORDER_COLOR_END_RANGE: VkBorderColor =
    VkBorderColor::VK_BORDER_COLOR_INT_OPAQUE_WHITE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkBorderColor {
    VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK = 0,
    VK_BORDER_COLOR_INT_TRANSPARENT_BLACK = 1,
    VK_BORDER_COLOR_FLOAT_OPAQUE_BLACK = 2,
    VK_BORDER_COLOR_INT_OPAQUE_BLACK = 3,
    VK_BORDER_COLOR_FLOAT_OPAQUE_WHITE = 4,
    VK_BORDER_COLOR_INT_OPAQUE_WHITE = 5,
    VK_BORDER_COLOR_RANGE_SIZE = 6,
    VK_BORDER_COLOR_MAX_ENUM = 2147483647,
}
pub const VkDescriptorType_VK_DESCRIPTOR_TYPE_BEGIN_RANGE: VkDescriptorType =
    VkDescriptorType::VK_DESCRIPTOR_TYPE_SAMPLER;
pub const VkDescriptorType_VK_DESCRIPTOR_TYPE_END_RANGE: VkDescriptorType =
    VkDescriptorType::VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDescriptorType {
    VK_DESCRIPTOR_TYPE_SAMPLER = 0,
    VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER = 1,
    VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE = 2,
    VK_DESCRIPTOR_TYPE_STORAGE_IMAGE = 3,
    VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER = 4,
    VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER = 5,
    VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER = 6,
    VK_DESCRIPTOR_TYPE_STORAGE_BUFFER = 7,
    VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC = 8,
    VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC = 9,
    VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT = 10,
    VK_DESCRIPTOR_TYPE_RANGE_SIZE = 11,
    VK_DESCRIPTOR_TYPE_MAX_ENUM = 2147483647,
}
pub const VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_BEGIN_RANGE: VkAttachmentLoadOp =
    VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_LOAD;
pub const VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_END_RANGE: VkAttachmentLoadOp =
    VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_DONT_CARE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkAttachmentLoadOp {
    VK_ATTACHMENT_LOAD_OP_LOAD = 0,
    VK_ATTACHMENT_LOAD_OP_CLEAR = 1,
    VK_ATTACHMENT_LOAD_OP_DONT_CARE = 2,
    VK_ATTACHMENT_LOAD_OP_RANGE_SIZE = 3,
    VK_ATTACHMENT_LOAD_OP_MAX_ENUM = 2147483647,
}
pub const VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_BEGIN_RANGE: VkAttachmentStoreOp =
    VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE;
pub const VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_END_RANGE: VkAttachmentStoreOp =
    VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkAttachmentStoreOp {
    VK_ATTACHMENT_STORE_OP_STORE = 0,
    VK_ATTACHMENT_STORE_OP_DONT_CARE = 1,
    VK_ATTACHMENT_STORE_OP_RANGE_SIZE = 2,
    VK_ATTACHMENT_STORE_OP_MAX_ENUM = 2147483647,
}
pub const VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_BEGIN_RANGE: VkPipelineBindPoint =
    VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS;
pub const VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_END_RANGE: VkPipelineBindPoint =
    VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_COMPUTE;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPipelineBindPoint {
    VK_PIPELINE_BIND_POINT_GRAPHICS = 0,
    VK_PIPELINE_BIND_POINT_COMPUTE = 1,
    VK_PIPELINE_BIND_POINT_RANGE_SIZE = 2,
    VK_PIPELINE_BIND_POINT_MAX_ENUM = 2147483647,
}
pub const VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_BEGIN_RANGE: VkCommandBufferLevel =
    VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY;
pub const VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_END_RANGE: VkCommandBufferLevel =
    VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_SECONDARY;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCommandBufferLevel {
    VK_COMMAND_BUFFER_LEVEL_PRIMARY = 0,
    VK_COMMAND_BUFFER_LEVEL_SECONDARY = 1,
    VK_COMMAND_BUFFER_LEVEL_RANGE_SIZE = 2,
    VK_COMMAND_BUFFER_LEVEL_MAX_ENUM = 2147483647,
}
pub const VkIndexType_VK_INDEX_TYPE_BEGIN_RANGE: VkIndexType = VkIndexType::VK_INDEX_TYPE_UINT16;
pub const VkIndexType_VK_INDEX_TYPE_END_RANGE: VkIndexType = VkIndexType::VK_INDEX_TYPE_UINT32;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkIndexType {
    VK_INDEX_TYPE_UINT16 = 0,
    VK_INDEX_TYPE_UINT32 = 1,
    VK_INDEX_TYPE_RANGE_SIZE = 2,
    VK_INDEX_TYPE_MAX_ENUM = 2147483647,
}
pub const VkSubpassContents_VK_SUBPASS_CONTENTS_BEGIN_RANGE: VkSubpassContents =
    VkSubpassContents::VK_SUBPASS_CONTENTS_INLINE;
pub const VkSubpassContents_VK_SUBPASS_CONTENTS_END_RANGE: VkSubpassContents =
    VkSubpassContents::VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSubpassContents {
    VK_SUBPASS_CONTENTS_INLINE = 0,
    VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS = 1,
    VK_SUBPASS_CONTENTS_RANGE_SIZE = 2,
    VK_SUBPASS_CONTENTS_MAX_ENUM = 2147483647,
}
pub type VkInstanceCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkFormatFeatureFlagBits {
    VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT = 1,
    VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT = 2,
    VK_FORMAT_FEATURE_STORAGE_IMAGE_ATOMIC_BIT = 4,
    VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT = 8,
    VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_BIT = 16,
    VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_ATOMIC_BIT = 32,
    VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT = 64,
    VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT = 128,
    VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT = 256,
    VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT = 512,
    VK_FORMAT_FEATURE_BLIT_SRC_BIT = 1024,
    VK_FORMAT_FEATURE_BLIT_DST_BIT = 2048,
    VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT = 4096,
    VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_CUBIC_BIT_IMG = 8192,
    VK_FORMAT_FEATURE_TRANSFER_SRC_BIT_KHR = 16384,
    VK_FORMAT_FEATURE_TRANSFER_DST_BIT_KHR = 32768,
    VK_FORMAT_FEATURE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkFormatFeatureFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkImageUsageFlagBits {
    VK_IMAGE_USAGE_TRANSFER_SRC_BIT = 1,
    VK_IMAGE_USAGE_TRANSFER_DST_BIT = 2,
    VK_IMAGE_USAGE_SAMPLED_BIT = 4,
    VK_IMAGE_USAGE_STORAGE_BIT = 8,
    VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT = 16,
    VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT = 32,
    VK_IMAGE_USAGE_TRANSIENT_ATTACHMENT_BIT = 64,
    VK_IMAGE_USAGE_INPUT_ATTACHMENT_BIT = 128,
    VK_IMAGE_USAGE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkImageUsageFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkImageCreateFlagBits {
    VK_IMAGE_CREATE_SPARSE_BINDING_BIT = 1,
    VK_IMAGE_CREATE_SPARSE_RESIDENCY_BIT = 2,
    VK_IMAGE_CREATE_SPARSE_ALIASED_BIT = 4,
    VK_IMAGE_CREATE_MUTABLE_FORMAT_BIT = 8,
    VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT = 16,
    VK_IMAGE_CREATE_BIND_SFR_BIT_KHX = 64,
    VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT_KHR = 32,
    VK_IMAGE_CREATE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkImageCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSampleCountFlagBits {
    VK_SAMPLE_COUNT_1_BIT = 1,
    VK_SAMPLE_COUNT_2_BIT = 2,
    VK_SAMPLE_COUNT_4_BIT = 4,
    VK_SAMPLE_COUNT_8_BIT = 8,
    VK_SAMPLE_COUNT_16_BIT = 16,
    VK_SAMPLE_COUNT_32_BIT = 32,
    VK_SAMPLE_COUNT_64_BIT = 64,
    VK_SAMPLE_COUNT_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkSampleCountFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkQueueFlagBits {
    VK_QUEUE_GRAPHICS_BIT = 1,
    VK_QUEUE_COMPUTE_BIT = 2,
    VK_QUEUE_TRANSFER_BIT = 4,
    VK_QUEUE_SPARSE_BINDING_BIT = 8,
    VK_QUEUE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkQueueFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkMemoryPropertyFlagBits {
    VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT = 1,
    VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT = 2,
    VK_MEMORY_PROPERTY_HOST_COHERENT_BIT = 4,
    VK_MEMORY_PROPERTY_HOST_CACHED_BIT = 8,
    VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT = 16,
    VK_MEMORY_PROPERTY_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkMemoryPropertyFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkMemoryHeapFlagBits {
    VK_MEMORY_HEAP_DEVICE_LOCAL_BIT = 1,
    VK_MEMORY_HEAP_MULTI_INSTANCE_BIT_KHX = 2,
    VK_MEMORY_HEAP_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkMemoryHeapFlags = VkFlags;
pub type VkDeviceCreateFlags = VkFlags;
pub type VkDeviceQueueCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPipelineStageFlagBits {
    VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT = 1,
    VK_PIPELINE_STAGE_DRAW_INDIRECT_BIT = 2,
    VK_PIPELINE_STAGE_VERTEX_INPUT_BIT = 4,
    VK_PIPELINE_STAGE_VERTEX_SHADER_BIT = 8,
    VK_PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT = 16,
    VK_PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT = 32,
    VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT = 64,
    VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT = 128,
    VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT = 256,
    VK_PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT = 512,
    VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT = 1024,
    VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT = 2048,
    VK_PIPELINE_STAGE_TRANSFER_BIT = 4096,
    VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT = 8192,
    VK_PIPELINE_STAGE_HOST_BIT = 16384,
    VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT = 32768,
    VK_PIPELINE_STAGE_ALL_COMMANDS_BIT = 65536,
    VK_PIPELINE_STAGE_COMMAND_PROCESS_BIT_NVX = 131072,
    VK_PIPELINE_STAGE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkPipelineStageFlags = VkFlags;
pub type VkMemoryMapFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkImageAspectFlagBits {
    VK_IMAGE_ASPECT_COLOR_BIT = 1,
    VK_IMAGE_ASPECT_DEPTH_BIT = 2,
    VK_IMAGE_ASPECT_STENCIL_BIT = 4,
    VK_IMAGE_ASPECT_METADATA_BIT = 8,
    VK_IMAGE_ASPECT_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkImageAspectFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSparseImageFormatFlagBits {
    VK_SPARSE_IMAGE_FORMAT_SINGLE_MIPTAIL_BIT = 1,
    VK_SPARSE_IMAGE_FORMAT_ALIGNED_MIP_SIZE_BIT = 2,
    VK_SPARSE_IMAGE_FORMAT_NONSTANDARD_BLOCK_SIZE_BIT = 4,
    VK_SPARSE_IMAGE_FORMAT_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkSparseImageFormatFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSparseMemoryBindFlagBits {
    VK_SPARSE_MEMORY_BIND_METADATA_BIT = 1,
    VK_SPARSE_MEMORY_BIND_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkSparseMemoryBindFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkFenceCreateFlagBits {
    VK_FENCE_CREATE_SIGNALED_BIT = 1,
    VK_FENCE_CREATE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkFenceCreateFlags = VkFlags;
pub type VkSemaphoreCreateFlags = VkFlags;
pub type VkEventCreateFlags = VkFlags;
pub type VkQueryPoolCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkQueryPipelineStatisticFlagBits {
    VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_VERTICES_BIT = 1,
    VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_PRIMITIVES_BIT = 2,
    VK_QUERY_PIPELINE_STATISTIC_VERTEX_SHADER_INVOCATIONS_BIT = 4,
    VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_INVOCATIONS_BIT = 8,
    VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_PRIMITIVES_BIT = 16,
    VK_QUERY_PIPELINE_STATISTIC_CLIPPING_INVOCATIONS_BIT = 32,
    VK_QUERY_PIPELINE_STATISTIC_CLIPPING_PRIMITIVES_BIT = 64,
    VK_QUERY_PIPELINE_STATISTIC_FRAGMENT_SHADER_INVOCATIONS_BIT = 128,
    VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_CONTROL_SHADER_PATCHES_BIT = 256,
    VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT = 512,
    VK_QUERY_PIPELINE_STATISTIC_COMPUTE_SHADER_INVOCATIONS_BIT = 1024,
    VK_QUERY_PIPELINE_STATISTIC_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkQueryPipelineStatisticFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkQueryResultFlagBits {
    VK_QUERY_RESULT_64_BIT = 1,
    VK_QUERY_RESULT_WAIT_BIT = 2,
    VK_QUERY_RESULT_WITH_AVAILABILITY_BIT = 4,
    VK_QUERY_RESULT_PARTIAL_BIT = 8,
    VK_QUERY_RESULT_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkQueryResultFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkBufferCreateFlagBits {
    VK_BUFFER_CREATE_SPARSE_BINDING_BIT = 1,
    VK_BUFFER_CREATE_SPARSE_RESIDENCY_BIT = 2,
    VK_BUFFER_CREATE_SPARSE_ALIASED_BIT = 4,
    VK_BUFFER_CREATE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkBufferCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkBufferUsageFlagBits {
    VK_BUFFER_USAGE_TRANSFER_SRC_BIT = 1,
    VK_BUFFER_USAGE_TRANSFER_DST_BIT = 2,
    VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT = 4,
    VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT = 8,
    VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT = 16,
    VK_BUFFER_USAGE_STORAGE_BUFFER_BIT = 32,
    VK_BUFFER_USAGE_INDEX_BUFFER_BIT = 64,
    VK_BUFFER_USAGE_VERTEX_BUFFER_BIT = 128,
    VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT = 256,
    VK_BUFFER_USAGE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkBufferUsageFlags = VkFlags;
pub type VkBufferViewCreateFlags = VkFlags;
pub type VkImageViewCreateFlags = VkFlags;
pub type VkShaderModuleCreateFlags = VkFlags;
pub type VkPipelineCacheCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPipelineCreateFlagBits {
    VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT = 1,
    VK_PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT = 2,
    VK_PIPELINE_CREATE_DERIVATIVE_BIT = 4,
    VK_PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHX = 8,
    VK_PIPELINE_CREATE_DISPATCH_BASE_KHX = 16,
    VK_PIPELINE_CREATE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkPipelineCreateFlags = VkFlags;
pub type VkPipelineShaderStageCreateFlags = VkFlags;
pub const VkShaderStageFlagBits_VK_SHADER_STAGE_FLAG_BITS_MAX_ENUM: VkShaderStageFlagBits =
    VkShaderStageFlagBits::VK_SHADER_STAGE_ALL;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkShaderStageFlagBits {
    VK_SHADER_STAGE_VERTEX_BIT = 1,
    VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT = 2,
    VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT = 4,
    VK_SHADER_STAGE_GEOMETRY_BIT = 8,
    VK_SHADER_STAGE_FRAGMENT_BIT = 16,
    VK_SHADER_STAGE_COMPUTE_BIT = 32,
    VK_SHADER_STAGE_ALL_GRAPHICS = 31,
    VK_SHADER_STAGE_ALL = 2147483647,
}
pub type VkPipelineVertexInputStateCreateFlags = VkFlags;
pub type VkPipelineInputAssemblyStateCreateFlags = VkFlags;
pub type VkPipelineTessellationStateCreateFlags = VkFlags;
pub type VkPipelineViewportStateCreateFlags = VkFlags;
pub type VkPipelineRasterizationStateCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCullModeFlagBits {
    VK_CULL_MODE_NONE = 0,
    VK_CULL_MODE_FRONT_BIT = 1,
    VK_CULL_MODE_BACK_BIT = 2,
    VK_CULL_MODE_FRONT_AND_BACK = 3,
    VK_CULL_MODE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkCullModeFlags = VkFlags;
pub type VkPipelineMultisampleStateCreateFlags = VkFlags;
pub type VkPipelineDepthStencilStateCreateFlags = VkFlags;
pub type VkPipelineColorBlendStateCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkColorComponentFlagBits {
    VK_COLOR_COMPONENT_R_BIT = 1,
    VK_COLOR_COMPONENT_G_BIT = 2,
    VK_COLOR_COMPONENT_B_BIT = 4,
    VK_COLOR_COMPONENT_A_BIT = 8,
    VK_COLOR_COMPONENT_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkColorComponentFlags = VkFlags;
pub type VkPipelineDynamicStateCreateFlags = VkFlags;
pub type VkPipelineLayoutCreateFlags = VkFlags;
pub type VkShaderStageFlags = VkFlags;
pub type VkSamplerCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDescriptorSetLayoutCreateFlagBits {
    VK_DESCRIPTOR_SET_LAYOUT_CREATE_PUSH_DESCRIPTOR_BIT_KHR = 1,
    VK_DESCRIPTOR_SET_LAYOUT_CREATE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkDescriptorSetLayoutCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDescriptorPoolCreateFlagBits {
    VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT = 1,
    VK_DESCRIPTOR_POOL_CREATE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkDescriptorPoolCreateFlags = VkFlags;
pub type VkDescriptorPoolResetFlags = VkFlags;
pub type VkFramebufferCreateFlags = VkFlags;
pub type VkRenderPassCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkAttachmentDescriptionFlagBits {
    VK_ATTACHMENT_DESCRIPTION_MAY_ALIAS_BIT = 1,
    VK_ATTACHMENT_DESCRIPTION_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkAttachmentDescriptionFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSubpassDescriptionFlagBits {
    VK_SUBPASS_DESCRIPTION_PER_VIEW_ATTRIBUTES_BIT_NVX = 1,
    VK_SUBPASS_DESCRIPTION_PER_VIEW_POSITION_X_ONLY_BIT_NVX = 2,
    VK_SUBPASS_DESCRIPTION_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkSubpassDescriptionFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkAccessFlagBits {
    VK_ACCESS_INDIRECT_COMMAND_READ_BIT = 1,
    VK_ACCESS_INDEX_READ_BIT = 2,
    VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT = 4,
    VK_ACCESS_UNIFORM_READ_BIT = 8,
    VK_ACCESS_INPUT_ATTACHMENT_READ_BIT = 16,
    VK_ACCESS_SHADER_READ_BIT = 32,
    VK_ACCESS_SHADER_WRITE_BIT = 64,
    VK_ACCESS_COLOR_ATTACHMENT_READ_BIT = 128,
    VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT = 256,
    VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT = 512,
    VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT = 1024,
    VK_ACCESS_TRANSFER_READ_BIT = 2048,
    VK_ACCESS_TRANSFER_WRITE_BIT = 4096,
    VK_ACCESS_HOST_READ_BIT = 8192,
    VK_ACCESS_HOST_WRITE_BIT = 16384,
    VK_ACCESS_MEMORY_READ_BIT = 32768,
    VK_ACCESS_MEMORY_WRITE_BIT = 65536,
    VK_ACCESS_COMMAND_PROCESS_READ_BIT_NVX = 131072,
    VK_ACCESS_COMMAND_PROCESS_WRITE_BIT_NVX = 262144,
    VK_ACCESS_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkAccessFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDependencyFlagBits {
    VK_DEPENDENCY_BY_REGION_BIT = 1,
    VK_DEPENDENCY_VIEW_LOCAL_BIT_KHX = 2,
    VK_DEPENDENCY_DEVICE_GROUP_BIT_KHX = 4,
    VK_DEPENDENCY_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkDependencyFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCommandPoolCreateFlagBits {
    VK_COMMAND_POOL_CREATE_TRANSIENT_BIT = 1,
    VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT = 2,
    VK_COMMAND_POOL_CREATE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkCommandPoolCreateFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCommandPoolResetFlagBits {
    VK_COMMAND_POOL_RESET_RELEASE_RESOURCES_BIT = 1,
    VK_COMMAND_POOL_RESET_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkCommandPoolResetFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCommandBufferUsageFlagBits {
    VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT = 1,
    VK_COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT = 2,
    VK_COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT = 4,
    VK_COMMAND_BUFFER_USAGE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkCommandBufferUsageFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkQueryControlFlagBits {
    VK_QUERY_CONTROL_PRECISE_BIT = 1,
    VK_QUERY_CONTROL_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkQueryControlFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCommandBufferResetFlagBits {
    VK_COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT = 1,
    VK_COMMAND_BUFFER_RESET_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkCommandBufferResetFlags = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkStencilFaceFlagBits {
    VK_STENCIL_FACE_FRONT_BIT = 1,
    VK_STENCIL_FACE_BACK_BIT = 2,
    VK_STENCIL_FRONT_AND_BACK = 3,
    VK_STENCIL_FACE_FLAG_BITS_MAX_ENUM = 2147483647,
}
pub type VkStencilFaceFlags = VkFlags;
pub type VkMetalSurfaceCreateFlagsEXT = VkFlags;

pub type PFN_vkAllocationFunction = ::std::option::Option<
    unsafe extern "C" fn(
        pUserData: *mut ::std::os::raw::c_void,
        size: usize,
        alignment: usize,
        allocationScope: VkSystemAllocationScope,
    ) -> *mut ::std::os::raw::c_void,
>;
pub type PFN_vkReallocationFunction = ::std::option::Option<
    unsafe extern "C" fn(
        pUserData: *mut ::std::os::raw::c_void,
        pOriginal: *mut ::std::os::raw::c_void,
        size: usize,
        alignment: usize,
        allocationScope: VkSystemAllocationScope,
    ) -> *mut ::std::os::raw::c_void,
>;
pub type PFN_vkFreeFunction = ::std::option::Option<
    unsafe extern "C" fn(
        pUserData: *mut ::std::os::raw::c_void,
        pMemory: *mut ::std::os::raw::c_void,
    ),
>;
pub type PFN_vkInternalAllocationNotification = ::std::option::Option<
    unsafe extern "C" fn(
        pUserData: *mut ::std::os::raw::c_void,
        size: usize,
        allocationType: VkInternalAllocationType,
        allocationScope: VkSystemAllocationScope,
    ),
>;
pub type PFN_vkInternalFreeNotification = ::std::option::Option<
    unsafe extern "C" fn(
        pUserData: *mut ::std::os::raw::c_void,
        size: usize,
        allocationType: VkInternalAllocationType,
        allocationScope: VkSystemAllocationScope,
    ),
>;
pub type PFN_vkVoidFunction = ::std::option::Option<unsafe extern "C" fn()>;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkApplicationInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub pApplicationName: *const ::std::os::raw::c_char,
    pub applicationVersion: u32,
    pub pEngineName: *const ::std::os::raw::c_char,
    pub engineVersion: u32,
    pub apiVersion: u32,
}
impl Clone for VkApplicationInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkInstanceCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkInstanceCreateFlags,
    pub pApplicationInfo: *const VkApplicationInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const *const ::std::os::raw::c_char,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const *const ::std::os::raw::c_char,
}
impl Clone for VkInstanceCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkAllocationCallbacks {
    pub pUserData: *mut ::std::os::raw::c_void,
    pub pfnAllocation: PFN_vkAllocationFunction,
    pub pfnReallocation: PFN_vkReallocationFunction,
    pub pfnFree: PFN_vkFreeFunction,
    pub pfnInternalAllocation: PFN_vkInternalAllocationNotification,
    pub pfnInternalFree: PFN_vkInternalFreeNotification,
}
impl Clone for VkAllocationCallbacks {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceFeatures {
    pub robustBufferAccess: VkBool32,
    pub fullDrawIndexUint32: VkBool32,
    pub imageCubeArray: VkBool32,
    pub independentBlend: VkBool32,
    pub geometryShader: VkBool32,
    pub tessellationShader: VkBool32,
    pub sampleRateShading: VkBool32,
    pub dualSrcBlend: VkBool32,
    pub logicOp: VkBool32,
    pub multiDrawIndirect: VkBool32,
    pub drawIndirectFirstInstance: VkBool32,
    pub depthClamp: VkBool32,
    pub depthBiasClamp: VkBool32,
    pub fillModeNonSolid: VkBool32,
    pub depthBounds: VkBool32,
    pub wideLines: VkBool32,
    pub largePoints: VkBool32,
    pub alphaToOne: VkBool32,
    pub multiViewport: VkBool32,
    pub samplerAnisotropy: VkBool32,
    pub textureCompressionETC2: VkBool32,
    pub textureCompressionASTC_LDR: VkBool32,
    pub textureCompressionBC: VkBool32,
    pub occlusionQueryPrecise: VkBool32,
    pub pipelineStatisticsQuery: VkBool32,
    pub vertexPipelineStoresAndAtomics: VkBool32,
    pub fragmentStoresAndAtomics: VkBool32,
    pub shaderTessellationAndGeometryPointSize: VkBool32,
    pub shaderImageGatherExtended: VkBool32,
    pub shaderStorageImageExtendedFormats: VkBool32,
    pub shaderStorageImageMultisample: VkBool32,
    pub shaderStorageImageReadWithoutFormat: VkBool32,
    pub shaderStorageImageWriteWithoutFormat: VkBool32,
    pub shaderUniformBufferArrayDynamicIndexing: VkBool32,
    pub shaderSampledImageArrayDynamicIndexing: VkBool32,
    pub shaderStorageBufferArrayDynamicIndexing: VkBool32,
    pub shaderStorageImageArrayDynamicIndexing: VkBool32,
    pub shaderClipDistance: VkBool32,
    pub shaderCullDistance: VkBool32,
    pub shaderFloat64: VkBool32,
    pub shaderInt64: VkBool32,
    pub shaderInt16: VkBool32,
    pub shaderResourceResidency: VkBool32,
    pub shaderResourceMinLod: VkBool32,
    pub sparseBinding: VkBool32,
    pub sparseResidencyBuffer: VkBool32,
    pub sparseResidencyImage2D: VkBool32,
    pub sparseResidencyImage3D: VkBool32,
    pub sparseResidency2Samples: VkBool32,
    pub sparseResidency4Samples: VkBool32,
    pub sparseResidency8Samples: VkBool32,
    pub sparseResidency16Samples: VkBool32,
    pub sparseResidencyAliased: VkBool32,
    pub variableMultisampleRate: VkBool32,
    pub inheritedQueries: VkBool32,
}
impl Clone for VkPhysicalDeviceFeatures {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkFormatProperties {
    pub linearTilingFeatures: VkFormatFeatureFlags,
    pub optimalTilingFeatures: VkFormatFeatureFlags,
    pub bufferFeatures: VkFormatFeatureFlags,
}
impl Clone for VkFormatProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExtent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}
impl Clone for VkExtent3D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageFormatProperties {
    pub maxExtent: VkExtent3D,
    pub maxMipLevels: u32,
    pub maxArrayLayers: u32,
    pub sampleCounts: VkSampleCountFlags,
    pub maxResourceSize: VkDeviceSize,
}
impl Clone for VkImageFormatProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceLimits {
    pub maxImageDimension1D: u32,
    pub maxImageDimension2D: u32,
    pub maxImageDimension3D: u32,
    pub maxImageDimensionCube: u32,
    pub maxImageArrayLayers: u32,
    pub maxTexelBufferElements: u32,
    pub maxUniformBufferRange: u32,
    pub maxStorageBufferRange: u32,
    pub maxPushConstantsSize: u32,
    pub maxMemoryAllocationCount: u32,
    pub maxSamplerAllocationCount: u32,
    pub bufferImageGranularity: VkDeviceSize,
    pub sparseAddressSpaceSize: VkDeviceSize,
    pub maxBoundDescriptorSets: u32,
    pub maxPerStageDescriptorSamplers: u32,
    pub maxPerStageDescriptorUniformBuffers: u32,
    pub maxPerStageDescriptorStorageBuffers: u32,
    pub maxPerStageDescriptorSampledImages: u32,
    pub maxPerStageDescriptorStorageImages: u32,
    pub maxPerStageDescriptorInputAttachments: u32,
    pub maxPerStageResources: u32,
    pub maxDescriptorSetSamplers: u32,
    pub maxDescriptorSetUniformBuffers: u32,
    pub maxDescriptorSetUniformBuffersDynamic: u32,
    pub maxDescriptorSetStorageBuffers: u32,
    pub maxDescriptorSetStorageBuffersDynamic: u32,
    pub maxDescriptorSetSampledImages: u32,
    pub maxDescriptorSetStorageImages: u32,
    pub maxDescriptorSetInputAttachments: u32,
    pub maxVertexInputAttributes: u32,
    pub maxVertexInputBindings: u32,
    pub maxVertexInputAttributeOffset: u32,
    pub maxVertexInputBindingStride: u32,
    pub maxVertexOutputComponents: u32,
    pub maxTessellationGenerationLevel: u32,
    pub maxTessellationPatchSize: u32,
    pub maxTessellationControlPerVertexInputComponents: u32,
    pub maxTessellationControlPerVertexOutputComponents: u32,
    pub maxTessellationControlPerPatchOutputComponents: u32,
    pub maxTessellationControlTotalOutputComponents: u32,
    pub maxTessellationEvaluationInputComponents: u32,
    pub maxTessellationEvaluationOutputComponents: u32,
    pub maxGeometryShaderInvocations: u32,
    pub maxGeometryInputComponents: u32,
    pub maxGeometryOutputComponents: u32,
    pub maxGeometryOutputVertices: u32,
    pub maxGeometryTotalOutputComponents: u32,
    pub maxFragmentInputComponents: u32,
    pub maxFragmentOutputAttachments: u32,
    pub maxFragmentDualSrcAttachments: u32,
    pub maxFragmentCombinedOutputResources: u32,
    pub maxComputeSharedMemorySize: u32,
    pub maxComputeWorkGroupCount: [u32; 3usize],
    pub maxComputeWorkGroupInvocations: u32,
    pub maxComputeWorkGroupSize: [u32; 3usize],
    pub subPixelPrecisionBits: u32,
    pub subTexelPrecisionBits: u32,
    pub mipmapPrecisionBits: u32,
    pub maxDrawIndexedIndexValue: u32,
    pub maxDrawIndirectCount: u32,
    pub maxSamplerLodBias: f32,
    pub maxSamplerAnisotropy: f32,
    pub maxViewports: u32,
    pub maxViewportDimensions: [u32; 2usize],
    pub viewportBoundsRange: [f32; 2usize],
    pub viewportSubPixelBits: u32,
    pub minMemoryMapAlignment: usize,
    pub minTexelBufferOffsetAlignment: VkDeviceSize,
    pub minUniformBufferOffsetAlignment: VkDeviceSize,
    pub minStorageBufferOffsetAlignment: VkDeviceSize,
    pub minTexelOffset: i32,
    pub maxTexelOffset: u32,
    pub minTexelGatherOffset: i32,
    pub maxTexelGatherOffset: u32,
    pub minInterpolationOffset: f32,
    pub maxInterpolationOffset: f32,
    pub subPixelInterpolationOffsetBits: u32,
    pub maxFramebufferWidth: u32,
    pub maxFramebufferHeight: u32,
    pub maxFramebufferLayers: u32,
    pub framebufferColorSampleCounts: VkSampleCountFlags,
    pub framebufferDepthSampleCounts: VkSampleCountFlags,
    pub framebufferStencilSampleCounts: VkSampleCountFlags,
    pub framebufferNoAttachmentsSampleCounts: VkSampleCountFlags,
    pub maxColorAttachments: u32,
    pub sampledImageColorSampleCounts: VkSampleCountFlags,
    pub sampledImageIntegerSampleCounts: VkSampleCountFlags,
    pub sampledImageDepthSampleCounts: VkSampleCountFlags,
    pub sampledImageStencilSampleCounts: VkSampleCountFlags,
    pub storageImageSampleCounts: VkSampleCountFlags,
    pub maxSampleMaskWords: u32,
    pub timestampComputeAndGraphics: VkBool32,
    pub timestampPeriod: f32,
    pub maxClipDistances: u32,
    pub maxCullDistances: u32,
    pub maxCombinedClipAndCullDistances: u32,
    pub discreteQueuePriorities: u32,
    pub pointSizeRange: [f32; 2usize],
    pub lineWidthRange: [f32; 2usize],
    pub pointSizeGranularity: f32,
    pub lineWidthGranularity: f32,
    pub strictLines: VkBool32,
    pub standardSampleLocations: VkBool32,
    pub optimalBufferCopyOffsetAlignment: VkDeviceSize,
    pub optimalBufferCopyRowPitchAlignment: VkDeviceSize,
    pub nonCoherentAtomSize: VkDeviceSize,
}
impl Clone for VkPhysicalDeviceLimits {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceSparseProperties {
    pub residencyStandard2DBlockShape: VkBool32,
    pub residencyStandard2DMultisampleBlockShape: VkBool32,
    pub residencyStandard3DBlockShape: VkBool32,
    pub residencyAlignedMipSize: VkBool32,
    pub residencyNonResidentStrict: VkBool32,
}
impl Clone for VkPhysicalDeviceSparseProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct VkPhysicalDeviceProperties {
    pub apiVersion: u32,
    pub driverVersion: u32,
    pub vendorID: u32,
    pub deviceID: u32,
    pub deviceType: VkPhysicalDeviceType,
    pub deviceName: [::std::os::raw::c_char; 256usize],
    pub pipelineCacheUUID: [u8; 16usize],
    pub limits: VkPhysicalDeviceLimits,
    pub sparseProperties: VkPhysicalDeviceSparseProperties,
}
impl Clone for VkPhysicalDeviceProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkQueueFamilyProperties {
    pub queueFlags: VkQueueFlags,
    pub queueCount: u32,
    pub timestampValidBits: u32,
    pub minImageTransferGranularity: VkExtent3D,
}
impl Clone for VkQueueFamilyProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMemoryType {
    pub propertyFlags: VkMemoryPropertyFlags,
    pub heapIndex: u32,
}
impl Clone for VkMemoryType {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMemoryHeap {
    pub size: VkDeviceSize,
    pub flags: VkMemoryHeapFlags,
}
impl Clone for VkMemoryHeap {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceMemoryProperties {
    pub memoryTypeCount: u32,
    pub memoryTypes: [VkMemoryType; 32usize],
    pub memoryHeapCount: u32,
    pub memoryHeaps: [VkMemoryHeap; 16usize],
}
impl Clone for VkPhysicalDeviceMemoryProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceQueueCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDeviceQueueCreateFlags,
    pub queueFamilyIndex: u32,
    pub queueCount: u32,
    pub pQueuePriorities: *const f32,
}
impl Clone for VkDeviceQueueCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDeviceCreateFlags,
    pub queueCreateInfoCount: u32,
    pub pQueueCreateInfos: *const VkDeviceQueueCreateInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const *const ::std::os::raw::c_char,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const *const ::std::os::raw::c_char,
    pub pEnabledFeatures: *const VkPhysicalDeviceFeatures,
}
impl Clone for VkDeviceCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct VkExtensionProperties {
    pub extensionName: [::std::os::raw::c_char; 256usize],
    pub specVersion: u32,
}
impl Clone for VkExtensionProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct VkLayerProperties {
    pub layerName: [::std::os::raw::c_char; 256usize],
    pub specVersion: u32,
    pub implementationVersion: u32,
    pub description: [::std::os::raw::c_char; 256usize],
}
impl Clone for VkLayerProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSubmitInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub waitSemaphoreCount: u32,
    pub pWaitSemaphores: *const VkSemaphore,
    pub pWaitDstStageMask: *const VkPipelineStageFlags,
    pub commandBufferCount: u32,
    pub pCommandBuffers: *const VkCommandBuffer,
    pub signalSemaphoreCount: u32,
    pub pSignalSemaphores: *const VkSemaphore,
}
impl Clone for VkSubmitInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMemoryAllocateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub allocationSize: VkDeviceSize,
    pub memoryTypeIndex: u32,
}
impl Clone for VkMemoryAllocateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMappedMemoryRange {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub memory: VkDeviceMemory,
    pub offset: VkDeviceSize,
    pub size: VkDeviceSize,
}
impl Clone for VkMappedMemoryRange {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMemoryRequirements {
    pub size: VkDeviceSize,
    pub alignment: VkDeviceSize,
    pub memoryTypeBits: u32,
}
impl Clone for VkMemoryRequirements {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseImageFormatProperties {
    pub aspectMask: VkImageAspectFlags,
    pub imageGranularity: VkExtent3D,
    pub flags: VkSparseImageFormatFlags,
}
impl Clone for VkSparseImageFormatProperties {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseImageMemoryRequirements {
    pub formatProperties: VkSparseImageFormatProperties,
    pub imageMipTailFirstLod: u32,
    pub imageMipTailSize: VkDeviceSize,
    pub imageMipTailOffset: VkDeviceSize,
    pub imageMipTailStride: VkDeviceSize,
}
impl Clone for VkSparseImageMemoryRequirements {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseMemoryBind {
    pub resourceOffset: VkDeviceSize,
    pub size: VkDeviceSize,
    pub memory: VkDeviceMemory,
    pub memoryOffset: VkDeviceSize,
    pub flags: VkSparseMemoryBindFlags,
}
impl Clone for VkSparseMemoryBind {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseBufferMemoryBindInfo {
    pub buffer: VkBuffer,
    pub bindCount: u32,
    pub pBinds: *const VkSparseMemoryBind,
}
impl Clone for VkSparseBufferMemoryBindInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseImageOpaqueMemoryBindInfo {
    pub image: VkImage,
    pub bindCount: u32,
    pub pBinds: *const VkSparseMemoryBind,
}
impl Clone for VkSparseImageOpaqueMemoryBindInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageSubresource {
    pub aspectMask: VkImageAspectFlags,
    pub mipLevel: u32,
    pub arrayLayer: u32,
}
impl Clone for VkImageSubresource {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkOffset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Clone for VkOffset3D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseImageMemoryBind {
    pub subresource: VkImageSubresource,
    pub offset: VkOffset3D,
    pub extent: VkExtent3D,
    pub memory: VkDeviceMemory,
    pub memoryOffset: VkDeviceSize,
    pub flags: VkSparseMemoryBindFlags,
}
impl Clone for VkSparseImageMemoryBind {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseImageMemoryBindInfo {
    pub image: VkImage,
    pub bindCount: u32,
    pub pBinds: *const VkSparseImageMemoryBind,
}
impl Clone for VkSparseImageMemoryBindInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBindSparseInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub waitSemaphoreCount: u32,
    pub pWaitSemaphores: *const VkSemaphore,
    pub bufferBindCount: u32,
    pub pBufferBinds: *const VkSparseBufferMemoryBindInfo,
    pub imageOpaqueBindCount: u32,
    pub pImageOpaqueBinds: *const VkSparseImageOpaqueMemoryBindInfo,
    pub imageBindCount: u32,
    pub pImageBinds: *const VkSparseImageMemoryBindInfo,
    pub signalSemaphoreCount: u32,
    pub pSignalSemaphores: *const VkSemaphore,
}
impl Clone for VkBindSparseInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkFenceCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkFenceCreateFlags,
}
impl Clone for VkFenceCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSemaphoreCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkSemaphoreCreateFlags,
}
impl Clone for VkSemaphoreCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkEventCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkEventCreateFlags,
}
impl Clone for VkEventCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkQueryPoolCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkQueryPoolCreateFlags,
    pub queryType: VkQueryType,
    pub queryCount: u32,
    pub pipelineStatistics: VkQueryPipelineStatisticFlags,
}
impl Clone for VkQueryPoolCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBufferCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkBufferCreateFlags,
    pub size: VkDeviceSize,
    pub usage: VkBufferUsageFlags,
    pub sharingMode: VkSharingMode,
    pub queueFamilyIndexCount: u32,
    pub pQueueFamilyIndices: *const u32,
}
impl Clone for VkBufferCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBufferViewCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkBufferViewCreateFlags,
    pub buffer: VkBuffer,
    pub format: VkFormat,
    pub offset: VkDeviceSize,
    pub range: VkDeviceSize,
}
impl Clone for VkBufferViewCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkImageCreateFlags,
    pub imageType: VkImageType,
    pub format: VkFormat,
    pub extent: VkExtent3D,
    pub mipLevels: u32,
    pub arrayLayers: u32,
    pub samples: VkSampleCountFlagBits,
    pub tiling: VkImageTiling,
    pub usage: VkImageUsageFlags,
    pub sharingMode: VkSharingMode,
    pub queueFamilyIndexCount: u32,
    pub pQueueFamilyIndices: *const u32,
    pub initialLayout: VkImageLayout,
}
impl Clone for VkImageCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSubresourceLayout {
    pub offset: VkDeviceSize,
    pub size: VkDeviceSize,
    pub rowPitch: VkDeviceSize,
    pub arrayPitch: VkDeviceSize,
    pub depthPitch: VkDeviceSize,
}
impl Clone for VkSubresourceLayout {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy, PartialEq)]
pub struct VkComponentMapping {
    pub r: VkComponentSwizzle,
    pub g: VkComponentSwizzle,
    pub b: VkComponentSwizzle,
    pub a: VkComponentSwizzle,
}
impl Clone for VkComponentMapping {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageSubresourceRange {
    pub aspectMask: VkImageAspectFlags,
    pub baseMipLevel: u32,
    pub levelCount: u32,
    pub baseArrayLayer: u32,
    pub layerCount: u32,
}
impl Clone for VkImageSubresourceRange {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageViewCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkImageViewCreateFlags,
    pub image: VkImage,
    pub viewType: VkImageViewType,
    pub format: VkFormat,
    pub components: VkComponentMapping,
    pub subresourceRange: VkImageSubresourceRange,
}
impl Clone for VkImageViewCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkShaderModuleCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkShaderModuleCreateFlags,
    pub codeSize: usize,
    pub pCode: *const u32,
}
impl Clone for VkShaderModuleCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineCacheCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineCacheCreateFlags,
    pub initialDataSize: usize,
    pub pInitialData: *const ::std::os::raw::c_void,
}
impl Clone for VkPipelineCacheCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSpecializationMapEntry {
    pub constantID: u32,
    pub offset: u32,
    pub size: usize,
}
impl Clone for VkSpecializationMapEntry {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSpecializationInfo {
    pub mapEntryCount: u32,
    pub pMapEntries: *const VkSpecializationMapEntry,
    pub dataSize: usize,
    pub pData: *const ::std::os::raw::c_void,
}
impl Clone for VkSpecializationInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineShaderStageCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineShaderStageCreateFlags,
    pub stage: VkShaderStageFlagBits,
    pub module: VkShaderModule,
    pub pName: *const ::std::os::raw::c_char,
    pub pSpecializationInfo: *const VkSpecializationInfo,
}
impl Clone for VkPipelineShaderStageCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkVertexInputBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub inputRate: VkVertexInputRate,
}
impl Clone for VkVertexInputBindingDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkVertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: VkFormat,
    pub offset: u32,
}
impl Clone for VkVertexInputAttributeDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineVertexInputStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineVertexInputStateCreateFlags,
    pub vertexBindingDescriptionCount: u32,
    pub pVertexBindingDescriptions: *const VkVertexInputBindingDescription,
    pub vertexAttributeDescriptionCount: u32,
    pub pVertexAttributeDescriptions: *const VkVertexInputAttributeDescription,
}
impl Clone for VkPipelineVertexInputStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineInputAssemblyStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineInputAssemblyStateCreateFlags,
    pub topology: VkPrimitiveTopology,
    pub primitiveRestartEnable: VkBool32,
}
impl Clone for VkPipelineInputAssemblyStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineTessellationStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineTessellationStateCreateFlags,
    pub patchControlPoints: u32,
}
impl Clone for VkPipelineTessellationStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub minDepth: f32,
    pub maxDepth: f32,
}
impl Clone for VkViewport {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkOffset2D {
    pub x: i32,
    pub y: i32,
}
impl Clone for VkOffset2D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExtent2D {
    pub width: u32,
    pub height: u32,
}
impl Clone for VkExtent2D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkRect2D {
    pub offset: VkOffset2D,
    pub extent: VkExtent2D,
}
impl Clone for VkRect2D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineViewportStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineViewportStateCreateFlags,
    pub viewportCount: u32,
    pub pViewports: *const VkViewport,
    pub scissorCount: u32,
    pub pScissors: *const VkRect2D,
}
impl Clone for VkPipelineViewportStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineRasterizationStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineRasterizationStateCreateFlags,
    pub depthClampEnable: VkBool32,
    pub rasterizerDiscardEnable: VkBool32,
    pub polygonMode: VkPolygonMode,
    pub cullMode: VkCullModeFlags,
    pub frontFace: VkFrontFace,
    pub depthBiasEnable: VkBool32,
    pub depthBiasConstantFactor: f32,
    pub depthBiasClamp: f32,
    pub depthBiasSlopeFactor: f32,
    pub lineWidth: f32,
}
impl Clone for VkPipelineRasterizationStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineMultisampleStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineMultisampleStateCreateFlags,
    pub rasterizationSamples: VkSampleCountFlagBits,
    pub sampleShadingEnable: VkBool32,
    pub minSampleShading: f32,
    pub pSampleMask: *const VkSampleMask,
    pub alphaToCoverageEnable: VkBool32,
    pub alphaToOneEnable: VkBool32,
}
impl Clone for VkPipelineMultisampleStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkStencilOpState {
    pub failOp: VkStencilOp,
    pub passOp: VkStencilOp,
    pub depthFailOp: VkStencilOp,
    pub compareOp: VkCompareOp,
    pub compareMask: u32,
    pub writeMask: u32,
    pub reference: u32,
}
impl Clone for VkStencilOpState {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineDepthStencilStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineDepthStencilStateCreateFlags,
    pub depthTestEnable: VkBool32,
    pub depthWriteEnable: VkBool32,
    pub depthCompareOp: VkCompareOp,
    pub depthBoundsTestEnable: VkBool32,
    pub stencilTestEnable: VkBool32,
    pub front: VkStencilOpState,
    pub back: VkStencilOpState,
    pub minDepthBounds: f32,
    pub maxDepthBounds: f32,
}
impl Clone for VkPipelineDepthStencilStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineColorBlendAttachmentState {
    pub blendEnable: VkBool32,
    pub srcColorBlendFactor: VkBlendFactor,
    pub dstColorBlendFactor: VkBlendFactor,
    pub colorBlendOp: VkBlendOp,
    pub srcAlphaBlendFactor: VkBlendFactor,
    pub dstAlphaBlendFactor: VkBlendFactor,
    pub alphaBlendOp: VkBlendOp,
    pub colorWriteMask: VkColorComponentFlags,
}
impl Clone for VkPipelineColorBlendAttachmentState {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineColorBlendStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineColorBlendStateCreateFlags,
    pub logicOpEnable: VkBool32,
    pub logicOp: VkLogicOp,
    pub attachmentCount: u32,
    pub pAttachments: *const VkPipelineColorBlendAttachmentState,
    pub blendConstants: [f32; 4usize],
}
impl Clone for VkPipelineColorBlendStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineDynamicStateCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineDynamicStateCreateFlags,
    pub dynamicStateCount: u32,
    pub pDynamicStates: *const VkDynamicState,
}
impl Clone for VkPipelineDynamicStateCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkGraphicsPipelineCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineCreateFlags,
    pub stageCount: u32,
    pub pStages: *const VkPipelineShaderStageCreateInfo,
    pub pVertexInputState: *const VkPipelineVertexInputStateCreateInfo,
    pub pInputAssemblyState: *const VkPipelineInputAssemblyStateCreateInfo,
    pub pTessellationState: *const VkPipelineTessellationStateCreateInfo,
    pub pViewportState: *const VkPipelineViewportStateCreateInfo,
    pub pRasterizationState: *const VkPipelineRasterizationStateCreateInfo,
    pub pMultisampleState: *const VkPipelineMultisampleStateCreateInfo,
    pub pDepthStencilState: *const VkPipelineDepthStencilStateCreateInfo,
    pub pColorBlendState: *const VkPipelineColorBlendStateCreateInfo,
    pub pDynamicState: *const VkPipelineDynamicStateCreateInfo,
    pub layout: VkPipelineLayout,
    pub renderPass: VkRenderPass,
    pub subpass: u32,
    pub basePipelineHandle: VkPipeline,
    pub basePipelineIndex: i32,
}
impl Clone for VkGraphicsPipelineCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkComputePipelineCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineCreateFlags,
    pub stage: VkPipelineShaderStageCreateInfo,
    pub layout: VkPipelineLayout,
    pub basePipelineHandle: VkPipeline,
    pub basePipelineIndex: i32,
}
impl Clone for VkComputePipelineCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPushConstantRange {
    pub stageFlags: VkShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}
impl Clone for VkPushConstantRange {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineLayoutCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineLayoutCreateFlags,
    pub setLayoutCount: u32,
    pub pSetLayouts: *const VkDescriptorSetLayout,
    pub pushConstantRangeCount: u32,
    pub pPushConstantRanges: *const VkPushConstantRange,
}
impl Clone for VkPipelineLayoutCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSamplerCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkSamplerCreateFlags,
    pub magFilter: VkFilter,
    pub minFilter: VkFilter,
    pub mipmapMode: VkSamplerMipmapMode,
    pub addressModeU: VkSamplerAddressMode,
    pub addressModeV: VkSamplerAddressMode,
    pub addressModeW: VkSamplerAddressMode,
    pub mipLodBias: f32,
    pub anisotropyEnable: VkBool32,
    pub maxAnisotropy: f32,
    pub compareEnable: VkBool32,
    pub compareOp: VkCompareOp,
    pub minLod: f32,
    pub maxLod: f32,
    pub borderColor: VkBorderColor,
    pub unnormalizedCoordinates: VkBool32,
}
impl Clone for VkSamplerCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptorType: VkDescriptorType,
    pub descriptorCount: u32,
    pub stageFlags: VkShaderStageFlags,
    pub pImmutableSamplers: *const VkSampler,
}
impl Clone for VkDescriptorSetLayoutBinding {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorSetLayoutCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDescriptorSetLayoutCreateFlags,
    pub bindingCount: u32,
    pub pBindings: *const VkDescriptorSetLayoutBinding,
}
impl Clone for VkDescriptorSetLayoutCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorPoolSize {
    pub type_: VkDescriptorType,
    pub descriptorCount: u32,
}
impl Clone for VkDescriptorPoolSize {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorPoolCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDescriptorPoolCreateFlags,
    pub maxSets: u32,
    pub poolSizeCount: u32,
    pub pPoolSizes: *const VkDescriptorPoolSize,
}
impl Clone for VkDescriptorPoolCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorSetAllocateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub descriptorPool: VkDescriptorPool,
    pub descriptorSetCount: u32,
    pub pSetLayouts: *const VkDescriptorSetLayout,
}
impl Clone for VkDescriptorSetAllocateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorImageInfo {
    pub sampler: VkSampler,
    pub imageView: VkImageView,
    pub imageLayout: VkImageLayout,
}
impl Clone for VkDescriptorImageInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorBufferInfo {
    pub buffer: VkBuffer,
    pub offset: VkDeviceSize,
    pub range: VkDeviceSize,
}
impl Clone for VkDescriptorBufferInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkWriteDescriptorSet {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub dstSet: VkDescriptorSet,
    pub dstBinding: u32,
    pub dstArrayElement: u32,
    pub descriptorCount: u32,
    pub descriptorType: VkDescriptorType,
    pub pImageInfo: *const VkDescriptorImageInfo,
    pub pBufferInfo: *const VkDescriptorBufferInfo,
    pub pTexelBufferView: *const VkBufferView,
}
impl Clone for VkWriteDescriptorSet {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkCopyDescriptorSet {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub srcSet: VkDescriptorSet,
    pub srcBinding: u32,
    pub srcArrayElement: u32,
    pub dstSet: VkDescriptorSet,
    pub dstBinding: u32,
    pub dstArrayElement: u32,
    pub descriptorCount: u32,
}
impl Clone for VkCopyDescriptorSet {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkFramebufferCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkFramebufferCreateFlags,
    pub renderPass: VkRenderPass,
    pub attachmentCount: u32,
    pub pAttachments: *const VkImageView,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}
impl Clone for VkFramebufferCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkAttachmentDescription {
    pub flags: VkAttachmentDescriptionFlags,
    pub format: VkFormat,
    pub samples: VkSampleCountFlagBits,
    pub loadOp: VkAttachmentLoadOp,
    pub storeOp: VkAttachmentStoreOp,
    pub stencilLoadOp: VkAttachmentLoadOp,
    pub stencilStoreOp: VkAttachmentStoreOp,
    pub initialLayout: VkImageLayout,
    pub finalLayout: VkImageLayout,
}
impl Clone for VkAttachmentDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkAttachmentReference {
    pub attachment: u32,
    pub layout: VkImageLayout,
}
impl Clone for VkAttachmentReference {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSubpassDescription {
    pub flags: VkSubpassDescriptionFlags,
    pub pipelineBindPoint: VkPipelineBindPoint,
    pub inputAttachmentCount: u32,
    pub pInputAttachments: *const VkAttachmentReference,
    pub colorAttachmentCount: u32,
    pub pColorAttachments: *const VkAttachmentReference,
    pub pResolveAttachments: *const VkAttachmentReference,
    pub pDepthStencilAttachment: *const VkAttachmentReference,
    pub preserveAttachmentCount: u32,
    pub pPreserveAttachments: *const u32,
}
impl Clone for VkSubpassDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSubpassDependency {
    pub srcSubpass: u32,
    pub dstSubpass: u32,
    pub srcStageMask: VkPipelineStageFlags,
    pub dstStageMask: VkPipelineStageFlags,
    pub srcAccessMask: VkAccessFlags,
    pub dstAccessMask: VkAccessFlags,
    pub dependencyFlags: VkDependencyFlags,
}
impl Clone for VkSubpassDependency {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkRenderPassCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkRenderPassCreateFlags,
    pub attachmentCount: u32,
    pub pAttachments: *const VkAttachmentDescription,
    pub subpassCount: u32,
    pub pSubpasses: *const VkSubpassDescription,
    pub dependencyCount: u32,
    pub pDependencies: *const VkSubpassDependency,
}
impl Clone for VkRenderPassCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkCommandPoolCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkCommandPoolCreateFlags,
    pub queueFamilyIndex: u32,
}
impl Clone for VkCommandPoolCreateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkCommandBufferAllocateInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub commandPool: VkCommandPool,
    pub level: VkCommandBufferLevel,
    pub commandBufferCount: u32,
}
impl Clone for VkCommandBufferAllocateInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkCommandBufferInheritanceInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub renderPass: VkRenderPass,
    pub subpass: u32,
    pub framebuffer: VkFramebuffer,
    pub occlusionQueryEnable: VkBool32,
    pub queryFlags: VkQueryControlFlags,
    pub pipelineStatistics: VkQueryPipelineStatisticFlags,
}
impl Clone for VkCommandBufferInheritanceInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkCommandBufferBeginInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkCommandBufferUsageFlags,
    pub pInheritanceInfo: *const VkCommandBufferInheritanceInfo,
}
impl Clone for VkCommandBufferBeginInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBufferCopy {
    pub srcOffset: VkDeviceSize,
    pub dstOffset: VkDeviceSize,
    pub size: VkDeviceSize,
}
impl Clone for VkBufferCopy {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageSubresourceLayers {
    pub aspectMask: VkImageAspectFlags,
    pub mipLevel: u32,
    pub baseArrayLayer: u32,
    pub layerCount: u32,
}
impl Clone for VkImageSubresourceLayers {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageCopy {
    pub srcSubresource: VkImageSubresourceLayers,
    pub srcOffset: VkOffset3D,
    pub dstSubresource: VkImageSubresourceLayers,
    pub dstOffset: VkOffset3D,
    pub extent: VkExtent3D,
}
impl Clone for VkImageCopy {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageBlit {
    pub srcSubresource: VkImageSubresourceLayers,
    pub srcOffsets: [VkOffset3D; 2usize],
    pub dstSubresource: VkImageSubresourceLayers,
    pub dstOffsets: [VkOffset3D; 2usize],
}
impl Clone for VkImageBlit {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBufferImageCopy {
    pub bufferOffset: VkDeviceSize,
    pub bufferRowLength: u32,
    pub bufferImageHeight: u32,
    pub imageSubresource: VkImageSubresourceLayers,
    pub imageOffset: VkOffset3D,
    pub imageExtent: VkExtent3D,
}
impl Clone for VkBufferImageCopy {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub union VkClearColorValue {
    pub float32: [f32; 4usize],
    pub int32: [i32; 4usize],
    pub uint32: [u32; 4usize],
    _bindgen_union_align: [u32; 4usize],
}
impl Clone for VkClearColorValue {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32,
}
impl Clone for VkClearDepthStencilValue {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub union VkClearValue {
    pub color: VkClearColorValue,
    pub depthStencil: VkClearDepthStencilValue,
    _bindgen_union_align: [u32; 4usize],
}
impl Clone for VkClearValue {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct VkClearAttachment {
    pub aspectMask: VkImageAspectFlags,
    pub colorAttachment: u32,
    pub clearValue: VkClearValue,
}
impl Clone for VkClearAttachment {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkClearRect {
    pub rect: VkRect2D,
    pub baseArrayLayer: u32,
    pub layerCount: u32,
}
impl Clone for VkClearRect {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageResolve {
    pub srcSubresource: VkImageSubresourceLayers,
    pub srcOffset: VkOffset3D,
    pub dstSubresource: VkImageSubresourceLayers,
    pub dstOffset: VkOffset3D,
    pub extent: VkExtent3D,
}
impl Clone for VkImageResolve {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMemoryBarrier {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub srcAccessMask: VkAccessFlags,
    pub dstAccessMask: VkAccessFlags,
}
impl Clone for VkMemoryBarrier {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBufferMemoryBarrier {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub srcAccessMask: VkAccessFlags,
    pub dstAccessMask: VkAccessFlags,
    pub srcQueueFamilyIndex: u32,
    pub dstQueueFamilyIndex: u32,
    pub buffer: VkBuffer,
    pub offset: VkDeviceSize,
    pub size: VkDeviceSize,
}
impl Clone for VkBufferMemoryBarrier {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageMemoryBarrier {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub srcAccessMask: VkAccessFlags,
    pub dstAccessMask: VkAccessFlags,
    pub oldLayout: VkImageLayout,
    pub newLayout: VkImageLayout,
    pub srcQueueFamilyIndex: u32,
    pub dstQueueFamilyIndex: u32,
    pub image: VkImage,
    pub subresourceRange: VkImageSubresourceRange,
}
impl Clone for VkImageMemoryBarrier {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkRenderPassBeginInfo {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub renderPass: VkRenderPass,
    pub framebuffer: VkFramebuffer,
    pub renderArea: VkRect2D,
    pub clearValueCount: u32,
    pub pClearValues: *const VkClearValue,
}
impl Clone for VkRenderPassBeginInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDispatchIndirectCommand {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl Clone for VkDispatchIndirectCommand {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDrawIndexedIndirectCommand {
    pub indexCount: u32,
    pub instanceCount: u32,
    pub firstIndex: u32,
    pub vertexOffset: i32,
    pub firstInstance: u32,
}
impl Clone for VkDrawIndexedIndirectCommand {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDrawIndirectCommand {
    pub vertexCount: u32,
    pub instanceCount: u32,
    pub firstVertex: u32,
    pub firstInstance: u32,
}
impl Clone for VkDrawIndirectCommand {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetPhysicalDeviceFeatures = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pFeatures: *mut VkPhysicalDeviceFeatures,
    ),
>;
pub type PFN_vkGetPhysicalDeviceFormatProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        format: VkFormat,
        pFormatProperties: *mut VkFormatProperties,
    ),
>;
pub type PFN_vkGetPhysicalDeviceImageFormatProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        format: VkFormat,
        type_: VkImageType,
        tiling: VkImageTiling,
        usage: VkImageUsageFlags,
        flags: VkImageCreateFlags,
        pImageFormatProperties: *mut VkImageFormatProperties,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pProperties: *mut VkPhysicalDeviceProperties,
    ),
>;
pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pQueueFamilyPropertyCount: *mut u32,
        pQueueFamilyProperties: *mut VkQueueFamilyProperties,
    ),
>;
pub type PFN_vkGetPhysicalDeviceMemoryProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties,
    ),
>;
pub type PFN_vkGetInstanceProcAddr = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pName: *const ::std::os::raw::c_char,
    ) -> PFN_vkVoidFunction,
>;
pub type PFN_vkGetDeviceProcAddr = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pName: *const ::std::os::raw::c_char,
    ) -> PFN_vkVoidFunction,
>;
pub type PFN_vkCreateDevice = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pCreateInfo: *const VkDeviceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pDevice: *mut VkDevice,
    ) -> VkResult,
>;
pub type PFN_vkDestroyDevice = ::std::option::Option<
    unsafe extern "C" fn(device: VkDevice, pAllocator: *const VkAllocationCallbacks),
>;
pub type PFN_vkEnumerateInstanceExtensionProperties = ::std::option::Option<
    unsafe extern "C" fn(
        pLayerName: *const ::std::os::raw::c_char,
        pPropertyCount: *mut u32,
        pProperties: *mut VkExtensionProperties,
    ) -> VkResult,
>;
pub type PFN_vkEnumerateDeviceExtensionProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pLayerName: *const ::std::os::raw::c_char,
        pPropertyCount: *mut u32,
        pProperties: *mut VkExtensionProperties,
    ) -> VkResult,
>;
pub type PFN_vkEnumerateInstanceLayerProperties = ::std::option::Option<
    unsafe extern "C" fn(pPropertyCount: *mut u32, pProperties: *mut VkLayerProperties) -> VkResult,
>;
pub type PFN_vkEnumerateDeviceLayerProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pPropertyCount: *mut u32,
        pProperties: *mut VkLayerProperties,
    ) -> VkResult,
>;
pub type PFN_vkGetDeviceQueue = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        queueFamilyIndex: u32,
        queueIndex: u32,
        pQueue: *mut VkQueue,
    ),
>;
pub type PFN_vkQueueSubmit = ::std::option::Option<
    unsafe extern "C" fn(
        queue: VkQueue,
        submitCount: u32,
        pSubmits: *const VkSubmitInfo,
        fence: VkFence,
    ) -> VkResult,
>;
pub type PFN_vkQueueWaitIdle =
    ::std::option::Option<unsafe extern "C" fn(queue: VkQueue) -> VkResult>;
pub type PFN_vkDeviceWaitIdle =
    ::std::option::Option<unsafe extern "C" fn(device: VkDevice) -> VkResult>;
pub type PFN_vkAllocateMemory = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pAllocateInfo: *const VkMemoryAllocateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pMemory: *mut VkDeviceMemory,
    ) -> VkResult,
>;
pub type PFN_vkFreeMemory = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        memory: VkDeviceMemory,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkMapMemory = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        memory: VkDeviceMemory,
        offset: VkDeviceSize,
        size: VkDeviceSize,
        flags: VkMemoryMapFlags,
        ppData: *mut *mut ::std::os::raw::c_void,
    ) -> VkResult,
>;
pub type PFN_vkUnmapMemory =
    ::std::option::Option<unsafe extern "C" fn(device: VkDevice, memory: VkDeviceMemory)>;
pub type PFN_vkFlushMappedMemoryRanges = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        memoryRangeCount: u32,
        pMemoryRanges: *const VkMappedMemoryRange,
    ) -> VkResult,
>;
pub type PFN_vkInvalidateMappedMemoryRanges = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        memoryRangeCount: u32,
        pMemoryRanges: *const VkMappedMemoryRange,
    ) -> VkResult,
>;
pub type PFN_vkGetDeviceMemoryCommitment = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        memory: VkDeviceMemory,
        pCommittedMemoryInBytes: *mut VkDeviceSize,
    ),
>;
pub type PFN_vkBindBufferMemory = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        buffer: VkBuffer,
        memory: VkDeviceMemory,
        memoryOffset: VkDeviceSize,
    ) -> VkResult,
>;
pub type PFN_vkBindImageMemory = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        image: VkImage,
        memory: VkDeviceMemory,
        memoryOffset: VkDeviceSize,
    ) -> VkResult,
>;
pub type PFN_vkGetBufferMemoryRequirements = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        buffer: VkBuffer,
        pMemoryRequirements: *mut VkMemoryRequirements,
    ),
>;
pub type PFN_vkGetImageMemoryRequirements = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        image: VkImage,
        pMemoryRequirements: *mut VkMemoryRequirements,
    ),
>;
pub type PFN_vkGetImageSparseMemoryRequirements = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        image: VkImage,
        pSparseMemoryRequirementCount: *mut u32,
        pSparseMemoryRequirements: *mut VkSparseImageMemoryRequirements,
    ),
>;
pub type PFN_vkGetPhysicalDeviceSparseImageFormatProperties = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        format: VkFormat,
        type_: VkImageType,
        samples: VkSampleCountFlagBits,
        usage: VkImageUsageFlags,
        tiling: VkImageTiling,
        pPropertyCount: *mut u32,
        pProperties: *mut VkSparseImageFormatProperties,
    ),
>;
pub type PFN_vkQueueBindSparse = ::std::option::Option<
    unsafe extern "C" fn(
        queue: VkQueue,
        bindInfoCount: u32,
        pBindInfo: *const VkBindSparseInfo,
        fence: VkFence,
    ) -> VkResult,
>;
pub type PFN_vkCreateFence = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkFenceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pFence: *mut VkFence,
    ) -> VkResult,
>;
pub type PFN_vkDestroyFence = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        fence: VkFence,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkResetFences = ::std::option::Option<
    unsafe extern "C" fn(device: VkDevice, fenceCount: u32, pFences: *const VkFence) -> VkResult,
>;
pub type PFN_vkGetFenceStatus =
    ::std::option::Option<unsafe extern "C" fn(device: VkDevice, fence: VkFence) -> VkResult>;
pub type PFN_vkWaitForFences = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        fenceCount: u32,
        pFences: *const VkFence,
        waitAll: VkBool32,
        timeout: u64,
    ) -> VkResult,
>;
pub type PFN_vkCreateSemaphore = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkSemaphoreCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pSemaphore: *mut VkSemaphore,
    ) -> VkResult,
>;
pub type PFN_vkDestroySemaphore = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        semaphore: VkSemaphore,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateEvent = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkEventCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pEvent: *mut VkEvent,
    ) -> VkResult,
>;
pub type PFN_vkDestroyEvent = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        event: VkEvent,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkGetEventStatus =
    ::std::option::Option<unsafe extern "C" fn(device: VkDevice, event: VkEvent) -> VkResult>;
pub type PFN_vkSetEvent =
    ::std::option::Option<unsafe extern "C" fn(device: VkDevice, event: VkEvent) -> VkResult>;
pub type PFN_vkResetEvent =
    ::std::option::Option<unsafe extern "C" fn(device: VkDevice, event: VkEvent) -> VkResult>;
pub type PFN_vkCreateQueryPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkQueryPoolCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pQueryPool: *mut VkQueryPool,
    ) -> VkResult,
>;
pub type PFN_vkDestroyQueryPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        queryPool: VkQueryPool,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkGetQueryPoolResults = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        queryPool: VkQueryPool,
        firstQuery: u32,
        queryCount: u32,
        dataSize: usize,
        pData: *mut ::std::os::raw::c_void,
        stride: VkDeviceSize,
        flags: VkQueryResultFlags,
    ) -> VkResult,
>;
pub type PFN_vkCreateBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkBufferCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pBuffer: *mut VkBuffer,
    ) -> VkResult,
>;
pub type PFN_vkDestroyBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        buffer: VkBuffer,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateBufferView = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkBufferViewCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pView: *mut VkBufferView,
    ) -> VkResult,
>;
pub type PFN_vkDestroyBufferView = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        bufferView: VkBufferView,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateImage = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkImageCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pImage: *mut VkImage,
    ) -> VkResult,
>;
pub type PFN_vkDestroyImage = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        image: VkImage,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkGetImageSubresourceLayout = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        image: VkImage,
        pSubresource: *const VkImageSubresource,
        pLayout: *mut VkSubresourceLayout,
    ),
>;
pub type PFN_vkCreateImageView = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkImageViewCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pView: *mut VkImageView,
    ) -> VkResult,
>;
pub type PFN_vkDestroyImageView = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        imageView: VkImageView,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateShaderModule = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkShaderModuleCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pShaderModule: *mut VkShaderModule,
    ) -> VkResult,
>;
pub type PFN_vkDestroyShaderModule = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        shaderModule: VkShaderModule,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreatePipelineCache = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkPipelineCacheCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pPipelineCache: *mut VkPipelineCache,
    ) -> VkResult,
>;
pub type PFN_vkDestroyPipelineCache = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pipelineCache: VkPipelineCache,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkGetPipelineCacheData = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pipelineCache: VkPipelineCache,
        pDataSize: *mut usize,
        pData: *mut ::std::os::raw::c_void,
    ) -> VkResult,
>;
pub type PFN_vkMergePipelineCaches = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        dstCache: VkPipelineCache,
        srcCacheCount: u32,
        pSrcCaches: *const VkPipelineCache,
    ) -> VkResult,
>;
pub type PFN_vkCreateGraphicsPipelines = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pipelineCache: VkPipelineCache,
        createInfoCount: u32,
        pCreateInfos: *const VkGraphicsPipelineCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pPipelines: *mut VkPipeline,
    ) -> VkResult,
>;
pub type PFN_vkCreateComputePipelines = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pipelineCache: VkPipelineCache,
        createInfoCount: u32,
        pCreateInfos: *const VkComputePipelineCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pPipelines: *mut VkPipeline,
    ) -> VkResult,
>;
pub type PFN_vkDestroyPipeline = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pipeline: VkPipeline,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreatePipelineLayout = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkPipelineLayoutCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pPipelineLayout: *mut VkPipelineLayout,
    ) -> VkResult,
>;
pub type PFN_vkDestroyPipelineLayout = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pipelineLayout: VkPipelineLayout,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateSampler = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkSamplerCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pSampler: *mut VkSampler,
    ) -> VkResult,
>;
pub type PFN_vkDestroySampler = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        sampler: VkSampler,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateDescriptorSetLayout = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkDescriptorSetLayoutCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pSetLayout: *mut VkDescriptorSetLayout,
    ) -> VkResult,
>;
pub type PFN_vkDestroyDescriptorSetLayout = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        descriptorSetLayout: VkDescriptorSetLayout,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateDescriptorPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkDescriptorPoolCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pDescriptorPool: *mut VkDescriptorPool,
    ) -> VkResult,
>;
pub type PFN_vkDestroyDescriptorPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        descriptorPool: VkDescriptorPool,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkResetDescriptorPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        descriptorPool: VkDescriptorPool,
        flags: VkDescriptorPoolResetFlags,
    ) -> VkResult,
>;
pub type PFN_vkAllocateDescriptorSets = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pAllocateInfo: *const VkDescriptorSetAllocateInfo,
        pDescriptorSets: *mut VkDescriptorSet,
    ) -> VkResult,
>;
pub type PFN_vkFreeDescriptorSets = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        descriptorPool: VkDescriptorPool,
        descriptorSetCount: u32,
        pDescriptorSets: *const VkDescriptorSet,
    ) -> VkResult,
>;
pub type PFN_vkUpdateDescriptorSets = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        descriptorWriteCount: u32,
        pDescriptorWrites: *const VkWriteDescriptorSet,
        descriptorCopyCount: u32,
        pDescriptorCopies: *const VkCopyDescriptorSet,
    ),
>;
pub type PFN_vkCreateFramebuffer = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkFramebufferCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pFramebuffer: *mut VkFramebuffer,
    ) -> VkResult,
>;
pub type PFN_vkDestroyFramebuffer = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        framebuffer: VkFramebuffer,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateRenderPass = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkRenderPassCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pRenderPass: *mut VkRenderPass,
    ) -> VkResult,
>;
pub type PFN_vkDestroyRenderPass = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        renderPass: VkRenderPass,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkGetRenderAreaGranularity = ::std::option::Option<
    unsafe extern "C" fn(device: VkDevice, renderPass: VkRenderPass, pGranularity: *mut VkExtent2D),
>;
pub type PFN_vkCreateCommandPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkCommandPoolCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pCommandPool: *mut VkCommandPool,
    ) -> VkResult,
>;
pub type PFN_vkDestroyCommandPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        commandPool: VkCommandPool,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkResetCommandPool = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        commandPool: VkCommandPool,
        flags: VkCommandPoolResetFlags,
    ) -> VkResult,
>;
pub type PFN_vkAllocateCommandBuffers = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pAllocateInfo: *const VkCommandBufferAllocateInfo,
        pCommandBuffers: *mut VkCommandBuffer,
    ) -> VkResult,
>;
pub type PFN_vkFreeCommandBuffers = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        commandPool: VkCommandPool,
        commandBufferCount: u32,
        pCommandBuffers: *const VkCommandBuffer,
    ),
>;
pub type PFN_vkBeginCommandBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pBeginInfo: *const VkCommandBufferBeginInfo,
    ) -> VkResult,
>;
pub type PFN_vkEndCommandBuffer =
    ::std::option::Option<unsafe extern "C" fn(commandBuffer: VkCommandBuffer) -> VkResult>;
pub type PFN_vkResetCommandBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        flags: VkCommandBufferResetFlags,
    ) -> VkResult,
>;
pub type PFN_vkCmdBindPipeline = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pipelineBindPoint: VkPipelineBindPoint,
        pipeline: VkPipeline,
    ),
>;
pub type PFN_vkCmdSetViewport = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        firstViewport: u32,
        viewportCount: u32,
        pViewports: *const VkViewport,
    ),
>;
pub type PFN_vkCmdSetScissor = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        firstScissor: u32,
        scissorCount: u32,
        pScissors: *const VkRect2D,
    ),
>;
pub type PFN_vkCmdSetLineWidth =
    ::std::option::Option<unsafe extern "C" fn(commandBuffer: VkCommandBuffer, lineWidth: f32)>;
pub type PFN_vkCmdSetDepthBias = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        depthBiasConstantFactor: f32,
        depthBiasClamp: f32,
        depthBiasSlopeFactor: f32,
    ),
>;
pub type PFN_vkCmdSetBlendConstants = ::std::option::Option<
    unsafe extern "C" fn(commandBuffer: VkCommandBuffer, blendConstants: *const f32),
>;
pub type PFN_vkCmdSetDepthBounds = ::std::option::Option<
    unsafe extern "C" fn(commandBuffer: VkCommandBuffer, minDepthBounds: f32, maxDepthBounds: f32),
>;
pub type PFN_vkCmdSetStencilCompareMask = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        faceMask: VkStencilFaceFlags,
        compareMask: u32,
    ),
>;
pub type PFN_vkCmdSetStencilWriteMask = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        faceMask: VkStencilFaceFlags,
        writeMask: u32,
    ),
>;
pub type PFN_vkCmdSetStencilReference = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        faceMask: VkStencilFaceFlags,
        reference: u32,
    ),
>;
pub type PFN_vkCmdBindDescriptorSets = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pipelineBindPoint: VkPipelineBindPoint,
        layout: VkPipelineLayout,
        firstSet: u32,
        descriptorSetCount: u32,
        pDescriptorSets: *const VkDescriptorSet,
        dynamicOffsetCount: u32,
        pDynamicOffsets: *const u32,
    ),
>;
pub type PFN_vkCmdBindIndexBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        indexType: VkIndexType,
    ),
>;
pub type PFN_vkCmdBindVertexBuffers = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        firstBinding: u32,
        bindingCount: u32,
        pBuffers: *const VkBuffer,
        pOffsets: *const VkDeviceSize,
    ),
>;
pub type PFN_vkCmdDraw = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        vertexCount: u32,
        instanceCount: u32,
        firstVertex: u32,
        firstInstance: u32,
    ),
>;
pub type PFN_vkCmdDrawIndexed = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        vertexOffset: i32,
        firstInstance: u32,
    ),
>;
pub type PFN_vkCmdDrawIndirect = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        drawCount: u32,
        stride: u32,
    ),
>;
pub type PFN_vkCmdDrawIndexedIndirect = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        drawCount: u32,
        stride: u32,
    ),
>;
pub type PFN_vkCmdDispatch = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        groupCountX: u32,
        groupCountY: u32,
        groupCountZ: u32,
    ),
>;
pub type PFN_vkCmdDispatchIndirect = ::std::option::Option<
    unsafe extern "C" fn(commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize),
>;
pub type PFN_vkCmdCopyBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        srcBuffer: VkBuffer,
        dstBuffer: VkBuffer,
        regionCount: u32,
        pRegions: *const VkBufferCopy,
    ),
>;
pub type PFN_vkCmdCopyImage = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        srcImage: VkImage,
        srcImageLayout: VkImageLayout,
        dstImage: VkImage,
        dstImageLayout: VkImageLayout,
        regionCount: u32,
        pRegions: *const VkImageCopy,
    ),
>;
pub type PFN_vkCmdBlitImage = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        srcImage: VkImage,
        srcImageLayout: VkImageLayout,
        dstImage: VkImage,
        dstImageLayout: VkImageLayout,
        regionCount: u32,
        pRegions: *const VkImageBlit,
        filter: VkFilter,
    ),
>;
pub type PFN_vkCmdCopyBufferToImage = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        srcBuffer: VkBuffer,
        dstImage: VkImage,
        dstImageLayout: VkImageLayout,
        regionCount: u32,
        pRegions: *const VkBufferImageCopy,
    ),
>;
pub type PFN_vkCmdCopyImageToBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        srcImage: VkImage,
        srcImageLayout: VkImageLayout,
        dstBuffer: VkBuffer,
        regionCount: u32,
        pRegions: *const VkBufferImageCopy,
    ),
>;
pub type PFN_vkCmdUpdateBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        dstBuffer: VkBuffer,
        dstOffset: VkDeviceSize,
        dataSize: VkDeviceSize,
        pData: *const ::std::os::raw::c_void,
    ),
>;
pub type PFN_vkCmdFillBuffer = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        dstBuffer: VkBuffer,
        dstOffset: VkDeviceSize,
        size: VkDeviceSize,
        data: u32,
    ),
>;
pub type PFN_vkCmdClearColorImage = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        image: VkImage,
        imageLayout: VkImageLayout,
        pColor: *const VkClearColorValue,
        rangeCount: u32,
        pRanges: *const VkImageSubresourceRange,
    ),
>;
pub type PFN_vkCmdClearDepthStencilImage = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        image: VkImage,
        imageLayout: VkImageLayout,
        pDepthStencil: *const VkClearDepthStencilValue,
        rangeCount: u32,
        pRanges: *const VkImageSubresourceRange,
    ),
>;
pub type PFN_vkCmdClearAttachments = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        attachmentCount: u32,
        pAttachments: *const VkClearAttachment,
        rectCount: u32,
        pRects: *const VkClearRect,
    ),
>;
pub type PFN_vkCmdResolveImage = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        srcImage: VkImage,
        srcImageLayout: VkImageLayout,
        dstImage: VkImage,
        dstImageLayout: VkImageLayout,
        regionCount: u32,
        pRegions: *const VkImageResolve,
    ),
>;
pub type PFN_vkCmdSetEvent = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        event: VkEvent,
        stageMask: VkPipelineStageFlags,
    ),
>;
pub type PFN_vkCmdResetEvent = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        event: VkEvent,
        stageMask: VkPipelineStageFlags,
    ),
>;
pub type PFN_vkCmdWaitEvents = ::std::option::Option<
    unsafe extern "C" fn(
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
    ),
>;
pub type PFN_vkCmdPipelineBarrier = ::std::option::Option<
    unsafe extern "C" fn(
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
    ),
>;
pub type PFN_vkCmdBeginQuery = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        queryPool: VkQueryPool,
        query: u32,
        flags: VkQueryControlFlags,
    ),
>;
pub type PFN_vkCmdEndQuery = ::std::option::Option<
    unsafe extern "C" fn(commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32),
>;
pub type PFN_vkCmdResetQueryPool = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        queryPool: VkQueryPool,
        firstQuery: u32,
        queryCount: u32,
    ),
>;
pub type PFN_vkCmdWriteTimestamp = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pipelineStage: VkPipelineStageFlagBits,
        queryPool: VkQueryPool,
        query: u32,
    ),
>;
pub type PFN_vkCmdCopyQueryPoolResults = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        queryPool: VkQueryPool,
        firstQuery: u32,
        queryCount: u32,
        dstBuffer: VkBuffer,
        dstOffset: VkDeviceSize,
        stride: VkDeviceSize,
        flags: VkQueryResultFlags,
    ),
>;
pub type PFN_vkCmdPushConstants = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        layout: VkPipelineLayout,
        stageFlags: VkShaderStageFlags,
        offset: u32,
        size: u32,
        pValues: *const ::std::os::raw::c_void,
    ),
>;
pub type PFN_vkCmdBeginRenderPass = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pRenderPassBegin: *const VkRenderPassBeginInfo,
        contents: VkSubpassContents,
    ),
>;
pub type PFN_vkCmdNextSubpass = ::std::option::Option<
    unsafe extern "C" fn(commandBuffer: VkCommandBuffer, contents: VkSubpassContents),
>;
pub type PFN_vkCmdEndRenderPass =
    ::std::option::Option<unsafe extern "C" fn(commandBuffer: VkCommandBuffer)>;
pub type PFN_vkCmdExecuteCommands = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        commandBufferCount: u32,
        pCommandBuffers: *const VkCommandBuffer,
    ),
>;
pub const VkColorSpaceKHR_VK_COLOR_SPACE_BEGIN_RANGE_KHR: VkColorSpaceKHR =
    VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
pub const VkColorSpaceKHR_VK_COLOR_SPACE_END_RANGE_KHR: VkColorSpaceKHR =
    VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkColorSpaceKHR {
    VK_COLOR_SPACE_SRGB_NONLINEAR_KHR = 0,
    VK_COLOR_SPACE_RANGE_SIZE_KHR = 1,
    VK_COLOR_SPACE_MAX_ENUM_KHR = 2147483647,
}
pub const VkPresentModeKHR_VK_PRESENT_MODE_BEGIN_RANGE_KHR: VkPresentModeKHR =
    VkPresentModeKHR::VK_PRESENT_MODE_IMMEDIATE_KHR;
pub const VkPresentModeKHR_VK_PRESENT_MODE_END_RANGE_KHR: VkPresentModeKHR =
    VkPresentModeKHR::VK_PRESENT_MODE_FIFO_RELAXED_KHR;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPresentModeKHR {
    VK_PRESENT_MODE_IMMEDIATE_KHR = 0,
    VK_PRESENT_MODE_MAILBOX_KHR = 1,
    VK_PRESENT_MODE_FIFO_KHR = 2,
    VK_PRESENT_MODE_FIFO_RELAXED_KHR = 3,
    VK_PRESENT_MODE_RANGE_SIZE_KHR = 4,
    VK_PRESENT_MODE_MAX_ENUM_KHR = 2147483647,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSurfaceTransformFlagBitsKHR {
    VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR = 1,
    VK_SURFACE_TRANSFORM_ROTATE_90_BIT_KHR = 2,
    VK_SURFACE_TRANSFORM_ROTATE_180_BIT_KHR = 4,
    VK_SURFACE_TRANSFORM_ROTATE_270_BIT_KHR = 8,
    VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_BIT_KHR = 16,
    VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_90_BIT_KHR = 32,
    VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_180_BIT_KHR = 64,
    VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_270_BIT_KHR = 128,
    VK_SURFACE_TRANSFORM_INHERIT_BIT_KHR = 256,
    VK_SURFACE_TRANSFORM_FLAG_BITS_MAX_ENUM_KHR = 2147483647,
}
pub type VkSurfaceTransformFlagsKHR = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkCompositeAlphaFlagBitsKHR {
    VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR = 1,
    VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR = 2,
    VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR = 4,
    VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR = 8,
    VK_COMPOSITE_ALPHA_FLAG_BITS_MAX_ENUM_KHR = 2147483647,
}
pub type VkCompositeAlphaFlagsKHR = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSurfaceCapabilitiesKHR {
    pub minImageCount: u32,
    pub maxImageCount: u32,
    pub currentExtent: VkExtent2D,
    pub minImageExtent: VkExtent2D,
    pub maxImageExtent: VkExtent2D,
    pub maxImageArrayLayers: u32,
    pub supportedTransforms: VkSurfaceTransformFlagsKHR,
    pub currentTransform: VkSurfaceTransformFlagBitsKHR,
    pub supportedCompositeAlpha: VkCompositeAlphaFlagsKHR,
    pub supportedUsageFlags: VkImageUsageFlags,
}
impl Clone for VkSurfaceCapabilitiesKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSurfaceFormatKHR {
    pub format: VkFormat,
    pub colorSpace: VkColorSpaceKHR,
}
impl Clone for VkSurfaceFormatKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkDestroySurfaceKHR = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        surface: VkSurfaceKHR,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkGetPhysicalDeviceSurfaceSupportKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        queueFamilyIndex: u32,
        surface: VkSurfaceKHR,
        pSupported: *mut VkBool32,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        pSurfaceCapabilities: *mut VkSurfaceCapabilitiesKHR,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceSurfaceFormatsKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        pSurfaceFormatCount: *mut u32,
        pSurfaceFormats: *mut VkSurfaceFormatKHR,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceSurfacePresentModesKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        pPresentModeCount: *mut u32,
        pPresentModes: *mut VkPresentModeKHR,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceSurfaceCapabilities2EXT = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        pSurfaceCapabilities: *mut VkSurfaceCapabilities2EXT,
    ) -> VkResult,
>;
pub type PFN_vkDisplayPowerControlEXT = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        display: VkDisplayKHR,
        pDisplayPowerInfo: *const VkDisplayPowerInfoEXT,
    ) -> VkResult,
>;
pub type PFN_vkRegisterDeviceEventEXT = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pDeviceEventInfo: *const VkDeviceEventInfoEXT,
        pAllocator: *const VkAllocationCallbacks,
        pFence: *mut VkFence,
    ) -> VkResult,
>;
pub type PFN_vkRegisterDisplayEventEXT = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        display: VkDisplayKHR,
        pDisplayEventInfo: *const VkDisplayEventInfoEXT,
        pAllocator: *const VkAllocationCallbacks,
        pFence: *mut VkFence,
    ) -> VkResult,
>;
pub type PFN_vkGetSwapchainCounterEXT = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        swapchain: VkSwapchainKHR,
        counter: VkSurfaceCounterFlagBitsEXT,
        pCounterValue: *mut u64,
    ) -> VkResult,
>;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSwapchainCreateFlagBitsKHR {
    VK_SWAPCHAIN_CREATE_BIND_SFR_BIT_KHX = 1,
    VK_SWAPCHAIN_CREATE_FLAG_BITS_MAX_ENUM_KHR = 2147483647,
}
pub type VkSwapchainCreateFlagsKHR = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSwapchainCreateInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkSwapchainCreateFlagsKHR,
    pub surface: VkSurfaceKHR,
    pub minImageCount: u32,
    pub imageFormat: VkFormat,
    pub imageColorSpace: VkColorSpaceKHR,
    pub imageExtent: VkExtent2D,
    pub imageArrayLayers: u32,
    pub imageUsage: VkImageUsageFlags,
    pub imageSharingMode: VkSharingMode,
    pub queueFamilyIndexCount: u32,
    pub pQueueFamilyIndices: *const u32,
    pub preTransform: VkSurfaceTransformFlagBitsKHR,
    pub compositeAlpha: VkCompositeAlphaFlagBitsKHR,
    pub presentMode: VkPresentModeKHR,
    pub clipped: VkBool32,
    pub oldSwapchain: VkSwapchainKHR,
}
impl Clone for VkSwapchainCreateInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPresentInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub waitSemaphoreCount: u32,
    pub pWaitSemaphores: *const VkSemaphore,
    pub swapchainCount: u32,
    pub pSwapchains: *const VkSwapchainKHR,
    pub pImageIndices: *const u32,
    pub pResults: *mut VkResult,
}
impl Clone for VkPresentInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCreateSwapchainKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkSwapchainCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pSwapchain: *mut VkSwapchainKHR,
    ) -> VkResult,
>;
pub type PFN_vkDestroySwapchainKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        swapchain: VkSwapchainKHR,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkGetSwapchainImagesKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        swapchain: VkSwapchainKHR,
        pSwapchainImageCount: *mut u32,
        pSwapchainImages: *mut VkImage,
    ) -> VkResult,
>;
pub type PFN_vkAcquireNextImageKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        swapchain: VkSwapchainKHR,
        timeout: u64,
        semaphore: VkSemaphore,
        fence: VkFence,
        pImageIndex: *mut u32,
    ) -> VkResult,
>;
pub type PFN_vkQueuePresentKHR = ::std::option::Option<
    unsafe extern "C" fn(queue: VkQueue, pPresentInfo: *const VkPresentInfoKHR) -> VkResult,
>;
extern "C" {
    pub fn vkAcquireNextImageKHR(
        device: VkDevice,
        swapchain: VkSwapchainKHR,
        timeout: u64,
        semaphore: VkSemaphore,
        fence: VkFence,
        pImageIndex: *mut u32,
    ) -> VkResult;
}
extern "C" {
    pub fn vkQueuePresentKHR(queue: VkQueue, pPresentInfo: *const VkPresentInfoKHR) -> VkResult;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VkDisplayKHR_T {
    _unused: [u8; 0],
}
pub type VkDisplayKHR = *mut VkDisplayKHR_T;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VkDisplayModeKHR_T {
    _unused: [u8; 0],
}
pub type VkDisplayModeKHR = *mut VkDisplayModeKHR_T;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDisplayPlaneAlphaFlagBitsKHR {
    VK_DISPLAY_PLANE_ALPHA_OPAQUE_BIT_KHR = 1,
    VK_DISPLAY_PLANE_ALPHA_GLOBAL_BIT_KHR = 2,
    VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_BIT_KHR = 4,
    VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_PREMULTIPLIED_BIT_KHR = 8,
    VK_DISPLAY_PLANE_ALPHA_FLAG_BITS_MAX_ENUM_KHR = 2147483647,
}
pub type VkDisplayPlaneAlphaFlagsKHR = VkFlags;
pub type VkDisplayModeCreateFlagsKHR = VkFlags;
pub type VkDisplaySurfaceCreateFlagsKHR = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayPropertiesKHR {
    pub display: VkDisplayKHR,
    pub displayName: *const ::std::os::raw::c_char,
    pub physicalDimensions: VkExtent2D,
    pub physicalResolution: VkExtent2D,
    pub supportedTransforms: VkSurfaceTransformFlagsKHR,
    pub planeReorderPossible: VkBool32,
    pub persistentContent: VkBool32,
}
impl Clone for VkDisplayPropertiesKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayModeParametersKHR {
    pub visibleRegion: VkExtent2D,
    pub refreshRate: u32,
}
impl Clone for VkDisplayModeParametersKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayModePropertiesKHR {
    pub displayMode: VkDisplayModeKHR,
    pub parameters: VkDisplayModeParametersKHR,
}
impl Clone for VkDisplayModePropertiesKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayModeCreateInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDisplayModeCreateFlagsKHR,
    pub parameters: VkDisplayModeParametersKHR,
}
impl Clone for VkDisplayModeCreateInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayPlaneCapabilitiesKHR {
    pub supportedAlpha: VkDisplayPlaneAlphaFlagsKHR,
    pub minSrcPosition: VkOffset2D,
    pub maxSrcPosition: VkOffset2D,
    pub minSrcExtent: VkExtent2D,
    pub maxSrcExtent: VkExtent2D,
    pub minDstPosition: VkOffset2D,
    pub maxDstPosition: VkOffset2D,
    pub minDstExtent: VkExtent2D,
    pub maxDstExtent: VkExtent2D,
}
impl Clone for VkDisplayPlaneCapabilitiesKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayPlanePropertiesKHR {
    pub currentDisplay: VkDisplayKHR,
    pub currentStackIndex: u32,
}
impl Clone for VkDisplayPlanePropertiesKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplaySurfaceCreateInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDisplaySurfaceCreateFlagsKHR,
    pub displayMode: VkDisplayModeKHR,
    pub planeIndex: u32,
    pub planeStackIndex: u32,
    pub transform: VkSurfaceTransformFlagBitsKHR,
    pub globalAlpha: f32,
    pub alphaMode: VkDisplayPlaneAlphaFlagBitsKHR,
    pub imageExtent: VkExtent2D,
}
impl Clone for VkDisplaySurfaceCreateInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetPhysicalDeviceDisplayPropertiesKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pPropertyCount: *mut u32,
        pProperties: *mut VkDisplayPropertiesKHR,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceDisplayPlanePropertiesKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pPropertyCount: *mut u32,
        pProperties: *mut VkDisplayPlanePropertiesKHR,
    ) -> VkResult,
>;
pub type PFN_vkGetDisplayPlaneSupportedDisplaysKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        planeIndex: u32,
        pDisplayCount: *mut u32,
        pDisplays: *mut VkDisplayKHR,
    ) -> VkResult,
>;
pub type PFN_vkGetDisplayModePropertiesKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        display: VkDisplayKHR,
        pPropertyCount: *mut u32,
        pProperties: *mut VkDisplayModePropertiesKHR,
    ) -> VkResult,
>;
pub type PFN_vkCreateDisplayModeKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        display: VkDisplayKHR,
        pCreateInfo: *const VkDisplayModeCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pMode: *mut VkDisplayModeKHR,
    ) -> VkResult,
>;
pub type PFN_vkGetDisplayPlaneCapabilitiesKHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        mode: VkDisplayModeKHR,
        planeIndex: u32,
        pCapabilities: *mut VkDisplayPlaneCapabilitiesKHR,
    ) -> VkResult,
>;
pub type PFN_vkCreateDisplayPlaneSurfaceKHR = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pCreateInfo: *const VkDisplaySurfaceCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pSurface: *mut VkSurfaceKHR,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkGetPhysicalDeviceDisplayPropertiesKHR(
        physicalDevice: VkPhysicalDevice,
        pPropertyCount: *mut u32,
        pProperties: *mut VkDisplayPropertiesKHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetPhysicalDeviceDisplayPlanePropertiesKHR(
        physicalDevice: VkPhysicalDevice,
        pPropertyCount: *mut u32,
        pProperties: *mut VkDisplayPlanePropertiesKHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetDisplayPlaneSupportedDisplaysKHR(
        physicalDevice: VkPhysicalDevice,
        planeIndex: u32,
        pDisplayCount: *mut u32,
        pDisplays: *mut VkDisplayKHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetDisplayModePropertiesKHR(
        physicalDevice: VkPhysicalDevice,
        display: VkDisplayKHR,
        pPropertyCount: *mut u32,
        pProperties: *mut VkDisplayModePropertiesKHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkCreateDisplayModeKHR(
        physicalDevice: VkPhysicalDevice,
        display: VkDisplayKHR,
        pCreateInfo: *const VkDisplayModeCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pMode: *mut VkDisplayModeKHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetDisplayPlaneCapabilitiesKHR(
        physicalDevice: VkPhysicalDevice,
        mode: VkDisplayModeKHR,
        planeIndex: u32,
        pCapabilities: *mut VkDisplayPlaneCapabilitiesKHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkCreateDisplayPlaneSurfaceKHR(
        instance: VkInstance,
        pCreateInfo: *const VkDisplaySurfaceCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pSurface: *mut VkSurfaceKHR,
    ) -> VkResult;
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayPresentInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub srcRect: VkRect2D,
    pub dstRect: VkRect2D,
    pub persistent: VkBool32,
}
impl Clone for VkDisplayPresentInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCreateSharedSwapchainsKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        swapchainCount: u32,
        pCreateInfos: *const VkSwapchainCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pSwapchains: *mut VkSwapchainKHR,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkCreateSharedSwapchainsKHR(
        device: VkDevice,
        swapchainCount: u32,
        pCreateInfos: *const VkSwapchainCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pSwapchains: *mut VkSwapchainKHR,
    ) -> VkResult;
}
pub type VkWin32SurfaceCreateFlagsKHR = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkWin32SurfaceCreateInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub flags: VkWin32SurfaceCreateFlagsKHR,
    pub hinstance: *mut ::std::os::raw::c_void,
    pub hwnd: *mut ::std::os::raw::c_void,
}
impl Clone for VkWin32SurfaceCreateInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VkXcbSurfaceCreateFlagsKHR = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkXcbSurfaceCreateInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub flags: VkXcbSurfaceCreateFlagsKHR,
    pub connection: *mut ::std::os::raw::c_void,
    pub window: u32,
}
impl Clone for VkXcbSurfaceCreateInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VkMacOSSurfaceCreateFlagsMVK = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMacOSSurfaceCreateInfoMVK {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub flags: VkMacOSSurfaceCreateFlagsMVK,
    pub pView: *mut ::std::os::raw::c_void,
}
impl Clone for VkMacOSSurfaceCreateInfoMVK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceFeatures2KHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub features: VkPhysicalDeviceFeatures,
}
impl Clone for VkPhysicalDeviceFeatures2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct VkPhysicalDeviceProperties2KHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub properties: VkPhysicalDeviceProperties,
}
impl Clone for VkPhysicalDeviceProperties2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkFormatProperties2KHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub formatProperties: VkFormatProperties,
}
impl Clone for VkFormatProperties2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageFormatProperties2KHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub imageFormatProperties: VkImageFormatProperties,
}
impl Clone for VkImageFormatProperties2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceImageFormatInfo2KHR {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub format: VkFormat,
    pub type_: VkImageType,
    pub tiling: VkImageTiling,
    pub usage: VkImageUsageFlags,
    pub flags: VkImageCreateFlags,
}
impl Clone for VkPhysicalDeviceImageFormatInfo2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkQueueFamilyProperties2KHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub queueFamilyProperties: VkQueueFamilyProperties,
}
impl Clone for VkQueueFamilyProperties2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceMemoryProperties2KHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub memoryProperties: VkPhysicalDeviceMemoryProperties,
}
impl Clone for VkPhysicalDeviceMemoryProperties2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSparseImageFormatProperties2KHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub properties: VkSparseImageFormatProperties,
}
impl Clone for VkSparseImageFormatProperties2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceSparseImageFormatInfo2KHR {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub format: VkFormat,
    pub type_: VkImageType,
    pub samples: VkSampleCountFlagBits,
    pub usage: VkImageUsageFlags,
    pub tiling: VkImageTiling,
}
impl Clone for VkPhysicalDeviceSparseImageFormatInfo2KHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetPhysicalDeviceFeatures2KHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pFeatures: *mut VkPhysicalDeviceFeatures2KHR,
    ),
>;
pub type PFN_vkGetPhysicalDeviceProperties2KHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pProperties: *mut VkPhysicalDeviceProperties2KHR,
    ),
>;
pub type PFN_vkGetPhysicalDeviceFormatProperties2KHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        format: VkFormat,
        pFormatProperties: *mut VkFormatProperties2KHR,
    ),
>;
pub type PFN_vkGetPhysicalDeviceImageFormatProperties2KHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pImageFormatInfo: *const VkPhysicalDeviceImageFormatInfo2KHR,
        pImageFormatProperties: *mut VkImageFormatProperties2KHR,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pQueueFamilyPropertyCount: *mut u32,
        pQueueFamilyProperties: *mut VkQueueFamilyProperties2KHR,
    ),
>;
pub type PFN_vkGetPhysicalDeviceMemoryProperties2KHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties2KHR,
    ),
>;
pub type PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pFormatInfo: *const VkPhysicalDeviceSparseImageFormatInfo2KHR,
        pPropertyCount: *mut u32,
        pProperties: *mut VkSparseImageFormatProperties2KHR,
    ),
>;
extern "C" {
    pub fn vkGetPhysicalDeviceFeatures2KHR(
        physicalDevice: VkPhysicalDevice,
        pFeatures: *mut VkPhysicalDeviceFeatures2KHR,
    );
}
extern "C" {
    pub fn vkGetPhysicalDeviceProperties2KHR(
        physicalDevice: VkPhysicalDevice,
        pProperties: *mut VkPhysicalDeviceProperties2KHR,
    );
}
extern "C" {
    pub fn vkGetPhysicalDeviceFormatProperties2KHR(
        physicalDevice: VkPhysicalDevice,
        format: VkFormat,
        pFormatProperties: *mut VkFormatProperties2KHR,
    );
}
extern "C" {
    pub fn vkGetPhysicalDeviceImageFormatProperties2KHR(
        physicalDevice: VkPhysicalDevice,
        pImageFormatInfo: *const VkPhysicalDeviceImageFormatInfo2KHR,
        pImageFormatProperties: *mut VkImageFormatProperties2KHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetPhysicalDeviceQueueFamilyProperties2KHR(
        physicalDevice: VkPhysicalDevice,
        pQueueFamilyPropertyCount: *mut u32,
        pQueueFamilyProperties: *mut VkQueueFamilyProperties2KHR,
    );
}
extern "C" {
    pub fn vkGetPhysicalDeviceMemoryProperties2KHR(
        physicalDevice: VkPhysicalDevice,
        pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties2KHR,
    );
}
extern "C" {
    pub fn vkGetPhysicalDeviceSparseImageFormatProperties2KHR(
        physicalDevice: VkPhysicalDevice,
        pFormatInfo: *const VkPhysicalDeviceSparseImageFormatInfo2KHR,
        pPropertyCount: *mut u32,
        pProperties: *mut VkSparseImageFormatProperties2KHR,
    );
}
pub type VkCommandPoolTrimFlagsKHR = VkFlags;
pub type PFN_vkTrimCommandPoolKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        commandPool: VkCommandPool,
        flags: VkCommandPoolTrimFlagsKHR,
    ),
>;
extern "C" {
    pub fn vkTrimCommandPoolKHR(
        device: VkDevice,
        commandPool: VkCommandPool,
        flags: VkCommandPoolTrimFlagsKHR,
    );
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDevicePushDescriptorPropertiesKHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub maxPushDescriptors: u32,
}
impl Clone for VkPhysicalDevicePushDescriptorPropertiesKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCmdPushDescriptorSetKHR = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pipelineBindPoint: VkPipelineBindPoint,
        layout: VkPipelineLayout,
        set: u32,
        descriptorWriteCount: u32,
        pDescriptorWrites: *const VkWriteDescriptorSet,
    ),
>;
extern "C" {
    pub fn vkCmdPushDescriptorSetKHR(
        commandBuffer: VkCommandBuffer,
        pipelineBindPoint: VkPipelineBindPoint,
        layout: VkPipelineLayout,
        set: u32,
        descriptorWriteCount: u32,
        pDescriptorWrites: *const VkWriteDescriptorSet,
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VkDescriptorUpdateTemplateKHR_T {
    _unused: [u8; 0],
}
pub type VkDescriptorUpdateTemplateKHR = *mut VkDescriptorUpdateTemplateKHR_T;
pub const VkDescriptorUpdateTemplateTypeKHR_VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_BEGIN_RANGE_KHR:
    VkDescriptorUpdateTemplateTypeKHR =
    VkDescriptorUpdateTemplateTypeKHR::VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_DESCRIPTOR_SET_KHR;
pub const VkDescriptorUpdateTemplateTypeKHR_VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_END_RANGE_KHR:
    VkDescriptorUpdateTemplateTypeKHR =
    VkDescriptorUpdateTemplateTypeKHR::VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_PUSH_DESCRIPTORS_KHR;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDescriptorUpdateTemplateTypeKHR {
    VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_DESCRIPTOR_SET_KHR = 0,
    VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_PUSH_DESCRIPTORS_KHR = 1,
    VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_RANGE_SIZE_KHR = 2,
    VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_MAX_ENUM_KHR = 2147483647,
}
pub type VkDescriptorUpdateTemplateCreateFlagsKHR = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorUpdateTemplateEntryKHR {
    pub dstBinding: u32,
    pub dstArrayElement: u32,
    pub descriptorCount: u32,
    pub descriptorType: VkDescriptorType,
    pub offset: usize,
    pub stride: usize,
}
impl Clone for VkDescriptorUpdateTemplateEntryKHR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDescriptorUpdateTemplateCreateInfoKHR {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub flags: VkDescriptorUpdateTemplateCreateFlagsKHR,
    pub descriptorUpdateEntryCount: u32,
    pub pDescriptorUpdateEntries: *const VkDescriptorUpdateTemplateEntryKHR,
    pub templateType: VkDescriptorUpdateTemplateTypeKHR,
    pub descriptorSetLayout: VkDescriptorSetLayout,
    pub pipelineBindPoint: VkPipelineBindPoint,
    pub pipelineLayout: VkPipelineLayout,
    pub set: u32,
}
impl Clone for VkDescriptorUpdateTemplateCreateInfoKHR {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCreateDescriptorUpdateTemplateKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkDescriptorUpdateTemplateCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pDescriptorUpdateTemplate: *mut VkDescriptorUpdateTemplateKHR,
    ) -> VkResult,
>;
pub type PFN_vkDestroyDescriptorUpdateTemplateKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        descriptorUpdateTemplate: VkDescriptorUpdateTemplateKHR,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkUpdateDescriptorSetWithTemplateKHR = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        descriptorSet: VkDescriptorSet,
        descriptorUpdateTemplate: VkDescriptorUpdateTemplateKHR,
        pData: *const ::std::os::raw::c_void,
    ),
>;
pub type PFN_vkCmdPushDescriptorSetWithTemplateKHR = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        descriptorUpdateTemplate: VkDescriptorUpdateTemplateKHR,
        layout: VkPipelineLayout,
        set: u32,
        pData: *const ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn vkCreateDescriptorUpdateTemplateKHR(
        device: VkDevice,
        pCreateInfo: *const VkDescriptorUpdateTemplateCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pDescriptorUpdateTemplate: *mut VkDescriptorUpdateTemplateKHR,
    ) -> VkResult;
}
extern "C" {
    pub fn vkDestroyDescriptorUpdateTemplateKHR(
        device: VkDevice,
        descriptorUpdateTemplate: VkDescriptorUpdateTemplateKHR,
        pAllocator: *const VkAllocationCallbacks,
    );
}
extern "C" {
    pub fn vkUpdateDescriptorSetWithTemplateKHR(
        device: VkDevice,
        descriptorSet: VkDescriptorSet,
        descriptorUpdateTemplate: VkDescriptorUpdateTemplateKHR,
        pData: *const ::std::os::raw::c_void,
    );
}
extern "C" {
    pub fn vkCmdPushDescriptorSetWithTemplateKHR(
        commandBuffer: VkCommandBuffer,
        descriptorUpdateTemplate: VkDescriptorUpdateTemplateKHR,
        layout: VkPipelineLayout,
        set: u32,
        pData: *const ::std::os::raw::c_void,
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VkDebugReportCallbackEXT_T {
    _unused: [u8; 0],
}
pub type VkDebugReportCallbackEXT = *mut VkDebugReportCallbackEXT_T;
pub const VkDebugReportObjectTypeEXT_VK_DEBUG_REPORT_OBJECT_TYPE_BEGIN_RANGE_EXT:
    VkDebugReportObjectTypeEXT =
    VkDebugReportObjectTypeEXT::VK_DEBUG_REPORT_OBJECT_TYPE_UNKNOWN_EXT;
pub const VkDebugReportObjectTypeEXT_VK_DEBUG_REPORT_OBJECT_TYPE_END_RANGE_EXT:
    VkDebugReportObjectTypeEXT =
    VkDebugReportObjectTypeEXT::VK_DEBUG_REPORT_OBJECT_TYPE_INDIRECT_COMMANDS_LAYOUT_NVX_EXT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDebugReportObjectTypeEXT {
    VK_DEBUG_REPORT_OBJECT_TYPE_UNKNOWN_EXT = 0,
    VK_DEBUG_REPORT_OBJECT_TYPE_INSTANCE_EXT = 1,
    VK_DEBUG_REPORT_OBJECT_TYPE_PHYSICAL_DEVICE_EXT = 2,
    VK_DEBUG_REPORT_OBJECT_TYPE_DEVICE_EXT = 3,
    VK_DEBUG_REPORT_OBJECT_TYPE_QUEUE_EXT = 4,
    VK_DEBUG_REPORT_OBJECT_TYPE_SEMAPHORE_EXT = 5,
    VK_DEBUG_REPORT_OBJECT_TYPE_COMMAND_BUFFER_EXT = 6,
    VK_DEBUG_REPORT_OBJECT_TYPE_FENCE_EXT = 7,
    VK_DEBUG_REPORT_OBJECT_TYPE_DEVICE_MEMORY_EXT = 8,
    VK_DEBUG_REPORT_OBJECT_TYPE_BUFFER_EXT = 9,
    VK_DEBUG_REPORT_OBJECT_TYPE_IMAGE_EXT = 10,
    VK_DEBUG_REPORT_OBJECT_TYPE_EVENT_EXT = 11,
    VK_DEBUG_REPORT_OBJECT_TYPE_QUERY_POOL_EXT = 12,
    VK_DEBUG_REPORT_OBJECT_TYPE_BUFFER_VIEW_EXT = 13,
    VK_DEBUG_REPORT_OBJECT_TYPE_IMAGE_VIEW_EXT = 14,
    VK_DEBUG_REPORT_OBJECT_TYPE_SHADER_MODULE_EXT = 15,
    VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_CACHE_EXT = 16,
    VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_LAYOUT_EXT = 17,
    VK_DEBUG_REPORT_OBJECT_TYPE_RENDER_PASS_EXT = 18,
    VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_EXT = 19,
    VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_SET_LAYOUT_EXT = 20,
    VK_DEBUG_REPORT_OBJECT_TYPE_SAMPLER_EXT = 21,
    VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_POOL_EXT = 22,
    VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_SET_EXT = 23,
    VK_DEBUG_REPORT_OBJECT_TYPE_FRAMEBUFFER_EXT = 24,
    VK_DEBUG_REPORT_OBJECT_TYPE_COMMAND_POOL_EXT = 25,
    VK_DEBUG_REPORT_OBJECT_TYPE_SURFACE_KHR_EXT = 26,
    VK_DEBUG_REPORT_OBJECT_TYPE_SWAPCHAIN_KHR_EXT = 27,
    VK_DEBUG_REPORT_OBJECT_TYPE_DEBUG_REPORT_EXT = 28,
    VK_DEBUG_REPORT_OBJECT_TYPE_DISPLAY_KHR_EXT = 29,
    VK_DEBUG_REPORT_OBJECT_TYPE_DISPLAY_MODE_KHR_EXT = 30,
    VK_DEBUG_REPORT_OBJECT_TYPE_OBJECT_TABLE_NVX_EXT = 31,
    VK_DEBUG_REPORT_OBJECT_TYPE_INDIRECT_COMMANDS_LAYOUT_NVX_EXT = 32,
    VK_DEBUG_REPORT_OBJECT_TYPE_RANGE_SIZE_EXT = 33,
    VK_DEBUG_REPORT_OBJECT_TYPE_MAX_ENUM_EXT = 2147483647,
}
pub const VkDebugReportErrorEXT_VK_DEBUG_REPORT_ERROR_BEGIN_RANGE_EXT: VkDebugReportErrorEXT =
    VkDebugReportErrorEXT::VK_DEBUG_REPORT_ERROR_NONE_EXT;
pub const VkDebugReportErrorEXT_VK_DEBUG_REPORT_ERROR_END_RANGE_EXT: VkDebugReportErrorEXT =
    VkDebugReportErrorEXT::VK_DEBUG_REPORT_ERROR_CALLBACK_REF_EXT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDebugReportErrorEXT {
    VK_DEBUG_REPORT_ERROR_NONE_EXT = 0,
    VK_DEBUG_REPORT_ERROR_CALLBACK_REF_EXT = 1,
    VK_DEBUG_REPORT_ERROR_RANGE_SIZE_EXT = 2,
    VK_DEBUG_REPORT_ERROR_MAX_ENUM_EXT = 2147483647,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDebugReportFlagBitsEXT {
    VK_DEBUG_REPORT_INFORMATION_BIT_EXT = 1,
    VK_DEBUG_REPORT_WARNING_BIT_EXT = 2,
    VK_DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT = 4,
    VK_DEBUG_REPORT_ERROR_BIT_EXT = 8,
    VK_DEBUG_REPORT_DEBUG_BIT_EXT = 16,
    VK_DEBUG_REPORT_FLAG_BITS_MAX_ENUM_EXT = 2147483647,
}
pub type VkDebugReportFlagsEXT = VkFlags;
pub type PFN_vkDebugReportCallbackEXT = ::std::option::Option<
    unsafe extern "C" fn(
        flags: VkDebugReportFlagsEXT,
        objectType: VkDebugReportObjectTypeEXT,
        object: u64,
        location: usize,
        messageCode: i32,
        pLayerPrefix: *const ::std::os::raw::c_char,
        pMessage: *const ::std::os::raw::c_char,
        pUserData: *mut ::std::os::raw::c_void,
    ) -> VkBool32,
>;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDebugReportCallbackCreateInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkDebugReportFlagsEXT,
    pub pfnCallback: PFN_vkDebugReportCallbackEXT,
    pub pUserData: *mut ::std::os::raw::c_void,
}
impl Clone for VkDebugReportCallbackCreateInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCreateDebugReportCallbackEXT = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pCreateInfo: *const VkDebugReportCallbackCreateInfoEXT,
        pAllocator: *const VkAllocationCallbacks,
        pCallback: *mut VkDebugReportCallbackEXT,
    ) -> VkResult,
>;
pub type PFN_vkDestroyDebugReportCallbackEXT = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        callback: VkDebugReportCallbackEXT,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkDebugReportMessageEXT = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        flags: VkDebugReportFlagsEXT,
        objectType: VkDebugReportObjectTypeEXT,
        object: u64,
        location: usize,
        messageCode: i32,
        pLayerPrefix: *const ::std::os::raw::c_char,
        pMessage: *const ::std::os::raw::c_char,
    ),
>;
extern "C" {
    pub fn vkCreateDebugReportCallbackEXT(
        instance: VkInstance,
        pCreateInfo: *const VkDebugReportCallbackCreateInfoEXT,
        pAllocator: *const VkAllocationCallbacks,
        pCallback: *mut VkDebugReportCallbackEXT,
    ) -> VkResult;
}
extern "C" {
    pub fn vkDestroyDebugReportCallbackEXT(
        instance: VkInstance,
        callback: VkDebugReportCallbackEXT,
        pAllocator: *const VkAllocationCallbacks,
    );
}
extern "C" {
    pub fn vkDebugReportMessageEXT(
        instance: VkInstance,
        flags: VkDebugReportFlagsEXT,
        objectType: VkDebugReportObjectTypeEXT,
        object: u64,
        location: usize,
        messageCode: i32,
        pLayerPrefix: *const ::std::os::raw::c_char,
        pMessage: *const ::std::os::raw::c_char,
    );
}
pub const VkRasterizationOrderAMD_VK_RASTERIZATION_ORDER_BEGIN_RANGE_AMD: VkRasterizationOrderAMD =
    VkRasterizationOrderAMD::VK_RASTERIZATION_ORDER_STRICT_AMD;
pub const VkRasterizationOrderAMD_VK_RASTERIZATION_ORDER_END_RANGE_AMD: VkRasterizationOrderAMD =
    VkRasterizationOrderAMD::VK_RASTERIZATION_ORDER_RELAXED_AMD;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkRasterizationOrderAMD {
    VK_RASTERIZATION_ORDER_STRICT_AMD = 0,
    VK_RASTERIZATION_ORDER_RELAXED_AMD = 1,
    VK_RASTERIZATION_ORDER_RANGE_SIZE_AMD = 2,
    VK_RASTERIZATION_ORDER_MAX_ENUM_AMD = 2147483647,
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineRasterizationStateRasterizationOrderAMD {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub rasterizationOrder: VkRasterizationOrderAMD,
}
impl Clone for VkPipelineRasterizationStateRasterizationOrderAMD {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDebugMarkerObjectNameInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub objectType: VkDebugReportObjectTypeEXT,
    pub object: u64,
    pub pObjectName: *const ::std::os::raw::c_char,
}
impl Clone for VkDebugMarkerObjectNameInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDebugMarkerObjectTagInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub objectType: VkDebugReportObjectTypeEXT,
    pub object: u64,
    pub tagName: u64,
    pub tagSize: usize,
    pub pTag: *const ::std::os::raw::c_void,
}
impl Clone for VkDebugMarkerObjectTagInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDebugMarkerMarkerInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub pMarkerName: *const ::std::os::raw::c_char,
    pub color: [f32; 4usize],
}
impl Clone for VkDebugMarkerMarkerInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkDebugMarkerSetObjectTagEXT = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pTagInfo: *mut VkDebugMarkerObjectTagInfoEXT,
    ) -> VkResult,
>;
pub type PFN_vkDebugMarkerSetObjectNameEXT = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pNameInfo: *mut VkDebugMarkerObjectNameInfoEXT,
    ) -> VkResult,
>;
pub type PFN_vkCmdDebugMarkerBeginEXT = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pMarkerInfo: *mut VkDebugMarkerMarkerInfoEXT,
    ),
>;
pub type PFN_vkCmdDebugMarkerEndEXT =
    ::std::option::Option<unsafe extern "C" fn(commandBuffer: VkCommandBuffer)>;
pub type PFN_vkCmdDebugMarkerInsertEXT = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pMarkerInfo: *mut VkDebugMarkerMarkerInfoEXT,
    ),
>;
extern "C" {
    pub fn vkDebugMarkerSetObjectTagEXT(
        device: VkDevice,
        pTagInfo: *mut VkDebugMarkerObjectTagInfoEXT,
    ) -> VkResult;
}
extern "C" {
    pub fn vkDebugMarkerSetObjectNameEXT(
        device: VkDevice,
        pNameInfo: *mut VkDebugMarkerObjectNameInfoEXT,
    ) -> VkResult;
}
extern "C" {
    pub fn vkCmdDebugMarkerBeginEXT(
        commandBuffer: VkCommandBuffer,
        pMarkerInfo: *mut VkDebugMarkerMarkerInfoEXT,
    );
}
extern "C" {
    pub fn vkCmdDebugMarkerEndEXT(commandBuffer: VkCommandBuffer);
}
extern "C" {
    pub fn vkCmdDebugMarkerInsertEXT(
        commandBuffer: VkCommandBuffer,
        pMarkerInfo: *mut VkDebugMarkerMarkerInfoEXT,
    );
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDedicatedAllocationImageCreateInfoNV {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub dedicatedAllocation: VkBool32,
}
impl Clone for VkDedicatedAllocationImageCreateInfoNV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDedicatedAllocationBufferCreateInfoNV {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub dedicatedAllocation: VkBool32,
}
impl Clone for VkDedicatedAllocationBufferCreateInfoNV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDedicatedAllocationMemoryAllocateInfoNV {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub image: VkImage,
    pub buffer: VkBuffer,
}
impl Clone for VkDedicatedAllocationMemoryAllocateInfoNV {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCmdDrawIndirectCountAMD = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        countBuffer: VkBuffer,
        countBufferOffset: VkDeviceSize,
        maxDrawCount: u32,
        stride: u32,
    ),
>;
pub type PFN_vkCmdDrawIndexedIndirectCountAMD = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        countBuffer: VkBuffer,
        countBufferOffset: VkDeviceSize,
        maxDrawCount: u32,
        stride: u32,
    ),
>;
extern "C" {
    pub fn vkCmdDrawIndirectCountAMD(
        commandBuffer: VkCommandBuffer,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        countBuffer: VkBuffer,
        countBufferOffset: VkDeviceSize,
        maxDrawCount: u32,
        stride: u32,
    );
}
extern "C" {
    pub fn vkCmdDrawIndexedIndirectCountAMD(
        commandBuffer: VkCommandBuffer,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        countBuffer: VkBuffer,
        countBufferOffset: VkDeviceSize,
        maxDrawCount: u32,
        stride: u32,
    );
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkRenderPassMultiviewCreateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub subpassCount: u32,
    pub pViewMasks: *const u32,
    pub dependencyCount: u32,
    pub pViewOffsets: *const i32,
    pub correlationMaskCount: u32,
    pub pCorrelationMasks: *const u32,
}
impl Clone for VkRenderPassMultiviewCreateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceMultiviewFeaturesKHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub multiview: VkBool32,
    pub multiviewGeometryShader: VkBool32,
    pub multiviewTessellationShader: VkBool32,
}
impl Clone for VkPhysicalDeviceMultiviewFeaturesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceMultiviewPropertiesKHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub maxMultiviewViewCount: u32,
    pub maxMultiviewInstanceIndex: u32,
}
impl Clone for VkPhysicalDeviceMultiviewPropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkExternalMemoryHandleTypeFlagBitsNV {
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_NV = 1,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_NV = 2,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_IMAGE_BIT_NV = 4,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_IMAGE_KMT_BIT_NV = 8,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_FLAG_BITS_MAX_ENUM_NV = 2147483647,
}
pub type VkExternalMemoryHandleTypeFlagsNV = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkExternalMemoryFeatureFlagBitsNV {
    VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT_NV = 1,
    VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT_NV = 2,
    VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT_NV = 4,
    VK_EXTERNAL_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM_NV = 2147483647,
}
pub type VkExternalMemoryFeatureFlagsNV = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalImageFormatPropertiesNV {
    pub imageFormatProperties: VkImageFormatProperties,
    pub externalMemoryFeatures: VkExternalMemoryFeatureFlagsNV,
    pub exportFromImportedHandleTypes: VkExternalMemoryHandleTypeFlagsNV,
    pub compatibleHandleTypes: VkExternalMemoryHandleTypeFlagsNV,
}
impl Clone for VkExternalImageFormatPropertiesNV {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetPhysicalDeviceExternalImageFormatPropertiesNV = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        format: VkFormat,
        type_: VkImageType,
        tiling: VkImageTiling,
        usage: VkImageUsageFlags,
        flags: VkImageCreateFlags,
        externalHandleType: VkExternalMemoryHandleTypeFlagsNV,
        pExternalImageFormatProperties: *mut VkExternalImageFormatPropertiesNV,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkGetPhysicalDeviceExternalImageFormatPropertiesNV(
        physicalDevice: VkPhysicalDevice,
        format: VkFormat,
        type_: VkImageType,
        tiling: VkImageTiling,
        usage: VkImageUsageFlags,
        flags: VkImageCreateFlags,
        externalHandleType: VkExternalMemoryHandleTypeFlagsNV,
        pExternalImageFormatProperties: *mut VkExternalImageFormatPropertiesNV,
    ) -> VkResult;
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalMemoryImageCreateInfoNV {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleTypes: VkExternalMemoryHandleTypeFlagsNV,
}
impl Clone for VkExternalMemoryImageCreateInfoNV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExportMemoryAllocateInfoNV {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleTypes: VkExternalMemoryHandleTypeFlagsNV,
}
impl Clone for VkExportMemoryAllocateInfoNV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkPeerMemoryFeatureFlagBitsKHX {
    VK_PEER_MEMORY_FEATURE_COPY_SRC_BIT_KHX = 1,
    VK_PEER_MEMORY_FEATURE_COPY_DST_BIT_KHX = 2,
    VK_PEER_MEMORY_FEATURE_GENERIC_SRC_BIT_KHX = 4,
    VK_PEER_MEMORY_FEATURE_GENERIC_DST_BIT_KHX = 8,
    VK_PEER_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM_KHX = 2147483647,
}
pub type VkPeerMemoryFeatureFlagsKHX = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkMemoryAllocateFlagBitsKHX {
    VK_MEMORY_ALLOCATE_DEVICE_MASK_BIT_KHX = 1,
    VK_MEMORY_ALLOCATE_FLAG_BITS_MAX_ENUM_KHX = 2147483647,
}
pub type VkMemoryAllocateFlagsKHX = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDeviceGroupPresentModeFlagBitsKHX {
    VK_DEVICE_GROUP_PRESENT_MODE_LOCAL_BIT_KHX = 1,
    VK_DEVICE_GROUP_PRESENT_MODE_REMOTE_BIT_KHX = 2,
    VK_DEVICE_GROUP_PRESENT_MODE_SUM_BIT_KHX = 4,
    VK_DEVICE_GROUP_PRESENT_MODE_LOCAL_MULTI_DEVICE_BIT_KHX = 8,
    VK_DEVICE_GROUP_PRESENT_MODE_FLAG_BITS_MAX_ENUM_KHX = 2147483647,
}
pub type VkDeviceGroupPresentModeFlagsKHX = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMemoryAllocateFlagsInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkMemoryAllocateFlagsKHX,
    pub deviceMask: u32,
}
impl Clone for VkMemoryAllocateFlagsInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBindBufferMemoryInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub buffer: VkBuffer,
    pub memory: VkDeviceMemory,
    pub memoryOffset: VkDeviceSize,
    pub deviceIndexCount: u32,
    pub pDeviceIndices: *const u32,
}
impl Clone for VkBindBufferMemoryInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBindImageMemoryInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub image: VkImage,
    pub memory: VkDeviceMemory,
    pub memoryOffset: VkDeviceSize,
    pub deviceIndexCount: u32,
    pub pDeviceIndices: *const u32,
    pub SFRRectCount: u32,
    pub pSFRRects: *const VkRect2D,
}
impl Clone for VkBindImageMemoryInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupRenderPassBeginInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub deviceMask: u32,
    pub deviceRenderAreaCount: u32,
    pub pDeviceRenderAreas: *const VkRect2D,
}
impl Clone for VkDeviceGroupRenderPassBeginInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupCommandBufferBeginInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub deviceMask: u32,
}
impl Clone for VkDeviceGroupCommandBufferBeginInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupSubmitInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub waitSemaphoreCount: u32,
    pub pWaitSemaphoreDeviceIndices: *const u32,
    pub commandBufferCount: u32,
    pub pCommandBufferDeviceMasks: *const u32,
    pub signalSemaphoreCount: u32,
    pub pSignalSemaphoreDeviceIndices: *const u32,
}
impl Clone for VkDeviceGroupSubmitInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupBindSparseInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub resourceDeviceIndex: u32,
    pub memoryDeviceIndex: u32,
}
impl Clone for VkDeviceGroupBindSparseInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupPresentCapabilitiesKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub presentMask: [u32; 32usize],
    pub modes: VkDeviceGroupPresentModeFlagsKHX,
}
impl Clone for VkDeviceGroupPresentCapabilitiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageSwapchainCreateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub swapchain: VkSwapchainKHR,
}
impl Clone for VkImageSwapchainCreateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkBindImageMemorySwapchainInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub swapchain: VkSwapchainKHR,
    pub imageIndex: u32,
}
impl Clone for VkBindImageMemorySwapchainInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkAcquireNextImageInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub swapchain: VkSwapchainKHR,
    pub timeout: u64,
    pub semaphore: VkSemaphore,
    pub fence: VkFence,
    pub deviceMask: u32,
}
impl Clone for VkAcquireNextImageInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupPresentInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub swapchainCount: u32,
    pub pDeviceMasks: *const u32,
    pub mode: VkDeviceGroupPresentModeFlagBitsKHX,
}
impl Clone for VkDeviceGroupPresentInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupSwapchainCreateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub modes: VkDeviceGroupPresentModeFlagsKHX,
}
impl Clone for VkDeviceGroupSwapchainCreateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetDeviceGroupPeerMemoryFeaturesKHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        heapIndex: u32,
        localDeviceIndex: u32,
        remoteDeviceIndex: u32,
        pPeerMemoryFeatures: *mut VkPeerMemoryFeatureFlagsKHX,
    ),
>;
pub type PFN_vkBindBufferMemory2KHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        bindInfoCount: u32,
        pBindInfos: *const VkBindBufferMemoryInfoKHX,
    ) -> VkResult,
>;
pub type PFN_vkBindImageMemory2KHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        bindInfoCount: u32,
        pBindInfos: *const VkBindImageMemoryInfoKHX,
    ) -> VkResult,
>;
pub type PFN_vkCmdSetDeviceMaskKHX =
    ::std::option::Option<unsafe extern "C" fn(commandBuffer: VkCommandBuffer, deviceMask: u32)>;
pub type PFN_vkGetDeviceGroupPresentCapabilitiesKHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pDeviceGroupPresentCapabilities: *mut VkDeviceGroupPresentCapabilitiesKHX,
    ) -> VkResult,
>;
pub type PFN_vkGetDeviceGroupSurfacePresentModesKHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        surface: VkSurfaceKHR,
        pModes: *mut VkDeviceGroupPresentModeFlagsKHX,
    ) -> VkResult,
>;
pub type PFN_vkAcquireNextImage2KHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pAcquireInfo: *const VkAcquireNextImageInfoKHX,
        pImageIndex: *mut u32,
    ) -> VkResult,
>;
pub type PFN_vkCmdDispatchBaseKHX = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        baseGroupX: u32,
        baseGroupY: u32,
        baseGroupZ: u32,
        groupCountX: u32,
        groupCountY: u32,
        groupCountZ: u32,
    ),
>;
pub type PFN_vkGetPhysicalDevicePresentRectanglesKHX = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        pRectCount: *mut u32,
        pRects: *mut VkRect2D,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkGetDeviceGroupPeerMemoryFeaturesKHX(
        device: VkDevice,
        heapIndex: u32,
        localDeviceIndex: u32,
        remoteDeviceIndex: u32,
        pPeerMemoryFeatures: *mut VkPeerMemoryFeatureFlagsKHX,
    );
}
extern "C" {
    pub fn vkBindBufferMemory2KHX(
        device: VkDevice,
        bindInfoCount: u32,
        pBindInfos: *const VkBindBufferMemoryInfoKHX,
    ) -> VkResult;
}
extern "C" {
    pub fn vkBindImageMemory2KHX(
        device: VkDevice,
        bindInfoCount: u32,
        pBindInfos: *const VkBindImageMemoryInfoKHX,
    ) -> VkResult;
}
extern "C" {
    pub fn vkCmdSetDeviceMaskKHX(commandBuffer: VkCommandBuffer, deviceMask: u32);
}
extern "C" {
    pub fn vkGetDeviceGroupPresentCapabilitiesKHX(
        device: VkDevice,
        pDeviceGroupPresentCapabilities: *mut VkDeviceGroupPresentCapabilitiesKHX,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetDeviceGroupSurfacePresentModesKHX(
        device: VkDevice,
        surface: VkSurfaceKHR,
        pModes: *mut VkDeviceGroupPresentModeFlagsKHX,
    ) -> VkResult;
}
extern "C" {
    pub fn vkAcquireNextImage2KHX(
        device: VkDevice,
        pAcquireInfo: *const VkAcquireNextImageInfoKHX,
        pImageIndex: *mut u32,
    ) -> VkResult;
}
extern "C" {
    pub fn vkCmdDispatchBaseKHX(
        commandBuffer: VkCommandBuffer,
        baseGroupX: u32,
        baseGroupY: u32,
        baseGroupZ: u32,
        groupCountX: u32,
        groupCountY: u32,
        groupCountZ: u32,
    );
}
extern "C" {
    pub fn vkGetPhysicalDevicePresentRectanglesKHX(
        physicalDevice: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        pRectCount: *mut u32,
        pRects: *mut VkRect2D,
    ) -> VkResult;
}
pub const VkValidationCheckEXT_VK_VALIDATION_CHECK_BEGIN_RANGE_EXT: VkValidationCheckEXT =
    VkValidationCheckEXT::VK_VALIDATION_CHECK_ALL_EXT;
pub const VkValidationCheckEXT_VK_VALIDATION_CHECK_END_RANGE_EXT: VkValidationCheckEXT =
    VkValidationCheckEXT::VK_VALIDATION_CHECK_ALL_EXT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkValidationCheckEXT {
    VK_VALIDATION_CHECK_ALL_EXT = 0,
    VK_VALIDATION_CHECK_RANGE_SIZE_EXT = 1,
    VK_VALIDATION_CHECK_MAX_ENUM_EXT = 2147483647,
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkValidationFlagsEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub disabledValidationCheckCount: u32,
    pub pDisabledValidationChecks: *mut VkValidationCheckEXT,
}
impl Clone for VkValidationFlagsEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceGroupPropertiesKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub physicalDeviceCount: u32,
    pub physicalDevices: [VkPhysicalDevice; 32usize],
    pub subsetAllocation: VkBool32,
}
impl Clone for VkPhysicalDeviceGroupPropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGroupDeviceCreateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub physicalDeviceCount: u32,
    pub pPhysicalDevices: *const VkPhysicalDevice,
}
impl Clone for VkDeviceGroupDeviceCreateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkEnumeratePhysicalDeviceGroupsKHX = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pPhysicalDeviceGroupCount: *mut u32,
        pPhysicalDeviceGroupProperties: *mut VkPhysicalDeviceGroupPropertiesKHX,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkEnumeratePhysicalDeviceGroupsKHX(
        instance: VkInstance,
        pPhysicalDeviceGroupCount: *mut u32,
        pPhysicalDeviceGroupProperties: *mut VkPhysicalDeviceGroupPropertiesKHX,
    ) -> VkResult;
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkExternalMemoryHandleTypeFlagBitsKHX {
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT_KHX = 1,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHX = 2,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHX = 4,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_BIT_KHX = 8,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_KMT_BIT_KHX = 16,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT_KHX = 32,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_RESOURCE_BIT_KHX = 64,
    VK_EXTERNAL_MEMORY_HANDLE_TYPE_FLAG_BITS_MAX_ENUM_KHX = 2147483647,
}
pub type VkExternalMemoryHandleTypeFlagsKHX = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkExternalMemoryFeatureFlagBitsKHX {
    VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT_KHX = 1,
    VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT_KHX = 2,
    VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT_KHX = 4,
    VK_EXTERNAL_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM_KHX = 2147483647,
}
pub type VkExternalMemoryFeatureFlagsKHX = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalMemoryPropertiesKHX {
    pub externalMemoryFeatures: VkExternalMemoryFeatureFlagsKHX,
    pub exportFromImportedHandleTypes: VkExternalMemoryHandleTypeFlagsKHX,
    pub compatibleHandleTypes: VkExternalMemoryHandleTypeFlagsKHX,
}
impl Clone for VkExternalMemoryPropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceExternalImageFormatInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleType: VkExternalMemoryHandleTypeFlagBitsKHX,
}
impl Clone for VkPhysicalDeviceExternalImageFormatInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalImageFormatPropertiesKHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub externalMemoryProperties: VkExternalMemoryPropertiesKHX,
}
impl Clone for VkExternalImageFormatPropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceExternalBufferInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkBufferCreateFlags,
    pub usage: VkBufferUsageFlags,
    pub handleType: VkExternalMemoryHandleTypeFlagBitsKHX,
}
impl Clone for VkPhysicalDeviceExternalBufferInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalBufferPropertiesKHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub externalMemoryProperties: VkExternalMemoryPropertiesKHX,
}
impl Clone for VkExternalBufferPropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceIDPropertiesKHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub deviceUUID: [u8; 16usize],
    pub driverUUID: [u8; 16usize],
    pub deviceLUID: [u8; 8usize],
    pub deviceLUIDValid: VkBool32,
}
impl Clone for VkPhysicalDeviceIDPropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct VkPhysicalDeviceProperties2KHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub properties: VkPhysicalDeviceProperties,
}
impl Clone for VkPhysicalDeviceProperties2KHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImageFormatProperties2KHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub imageFormatProperties: VkImageFormatProperties,
}
impl Clone for VkImageFormatProperties2KHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceImageFormatInfo2KHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub format: VkFormat,
    pub type_: VkImageType,
    pub tiling: VkImageTiling,
    pub usage: VkImageUsageFlags,
    pub flags: VkImageCreateFlags,
}
impl Clone for VkPhysicalDeviceImageFormatInfo2KHX {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetPhysicalDeviceExternalBufferPropertiesKHX = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pExternalBufferInfo: *const VkPhysicalDeviceExternalBufferInfoKHX,
        pExternalBufferProperties: *mut VkExternalBufferPropertiesKHX,
    ),
>;
pub type PFN_vkGetPhysicalDeviceProperties2KHX = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pProperties: *mut VkPhysicalDeviceProperties2KHX,
    ),
>;
pub type PFN_vkGetPhysicalDeviceImageFormatProperties2KHX = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pImageFormatInfo: *const VkPhysicalDeviceImageFormatInfo2KHX,
        pImageFormatProperties: *mut VkImageFormatProperties2KHX,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkGetPhysicalDeviceExternalBufferPropertiesKHX(
        physicalDevice: VkPhysicalDevice,
        pExternalBufferInfo: *const VkPhysicalDeviceExternalBufferInfoKHX,
        pExternalBufferProperties: *mut VkExternalBufferPropertiesKHX,
    );
}
extern "C" {
    pub fn vkGetPhysicalDeviceProperties2KHX(
        physicalDevice: VkPhysicalDevice,
        pProperties: *mut VkPhysicalDeviceProperties2KHX,
    );
}
extern "C" {
    pub fn vkGetPhysicalDeviceImageFormatProperties2KHX(
        physicalDevice: VkPhysicalDevice,
        pImageFormatInfo: *const VkPhysicalDeviceImageFormatInfo2KHX,
        pImageFormatProperties: *mut VkImageFormatProperties2KHX,
    ) -> VkResult;
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalMemoryImageCreateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleTypes: VkExternalMemoryHandleTypeFlagsKHX,
}
impl Clone for VkExternalMemoryImageCreateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalMemoryBufferCreateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleTypes: VkExternalMemoryHandleTypeFlagsKHX,
}
impl Clone for VkExternalMemoryBufferCreateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExportMemoryAllocateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleTypes: VkExternalMemoryHandleTypeFlagsKHX,
}
impl Clone for VkExportMemoryAllocateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImportMemoryFdInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleType: VkExternalMemoryHandleTypeFlagBitsKHX,
    pub fd: ::std::os::raw::c_int,
}
impl Clone for VkImportMemoryFdInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMemoryFdPropertiesKHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub memoryTypeBits: u32,
}
impl Clone for VkMemoryFdPropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetMemoryFdKHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        memory: VkDeviceMemory,
        handleType: VkExternalMemoryHandleTypeFlagBitsKHX,
        pFd: *mut ::std::os::raw::c_int,
    ) -> VkResult,
>;
pub type PFN_vkGetMemoryFdPropertiesKHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        handleType: VkExternalMemoryHandleTypeFlagBitsKHX,
        fd: ::std::os::raw::c_int,
        pMemoryFdProperties: *mut VkMemoryFdPropertiesKHX,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkGetMemoryFdKHX(
        device: VkDevice,
        memory: VkDeviceMemory,
        handleType: VkExternalMemoryHandleTypeFlagBitsKHX,
        pFd: *mut ::std::os::raw::c_int,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetMemoryFdPropertiesKHX(
        device: VkDevice,
        handleType: VkExternalMemoryHandleTypeFlagBitsKHX,
        fd: ::std::os::raw::c_int,
        pMemoryFdProperties: *mut VkMemoryFdPropertiesKHX,
    ) -> VkResult;
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkExternalSemaphoreHandleTypeFlagBitsKHX {
    VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD_BIT_KHX = 1,
    VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHX = 2,
    VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHX = 4,
    VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_D3D12_FENCE_BIT_KHX = 8,
    VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_FENCE_FD_BIT_KHX = 16,
    VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_FLAG_BITS_MAX_ENUM_KHX = 2147483647,
}
pub type VkExternalSemaphoreHandleTypeFlagsKHX = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkExternalSemaphoreFeatureFlagBitsKHX {
    VK_EXTERNAL_SEMAPHORE_FEATURE_EXPORTABLE_BIT_KHX = 1,
    VK_EXTERNAL_SEMAPHORE_FEATURE_IMPORTABLE_BIT_KHX = 2,
    VK_EXTERNAL_SEMAPHORE_FEATURE_FLAG_BITS_MAX_ENUM_KHX = 2147483647,
}
pub type VkExternalSemaphoreFeatureFlagsKHX = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceExternalSemaphoreInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleType: VkExternalSemaphoreHandleTypeFlagBitsKHX,
}
impl Clone for VkPhysicalDeviceExternalSemaphoreInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExternalSemaphorePropertiesKHX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub exportFromImportedHandleTypes: VkExternalSemaphoreHandleTypeFlagsKHX,
    pub compatibleHandleTypes: VkExternalSemaphoreHandleTypeFlagsKHX,
    pub externalSemaphoreFeatures: VkExternalSemaphoreFeatureFlagsKHX,
}
impl Clone for VkExternalSemaphorePropertiesKHX {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkGetPhysicalDeviceExternalSemaphorePropertiesKHX = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pExternalSemaphoreInfo: *const VkPhysicalDeviceExternalSemaphoreInfoKHX,
        pExternalSemaphoreProperties: *mut VkExternalSemaphorePropertiesKHX,
    ),
>;
extern "C" {
    pub fn vkGetPhysicalDeviceExternalSemaphorePropertiesKHX(
        physicalDevice: VkPhysicalDevice,
        pExternalSemaphoreInfo: *const VkPhysicalDeviceExternalSemaphoreInfoKHX,
        pExternalSemaphoreProperties: *mut VkExternalSemaphorePropertiesKHX,
    );
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkExportSemaphoreCreateInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub handleTypes: VkExternalSemaphoreHandleTypeFlagsKHX,
}
impl Clone for VkExportSemaphoreCreateInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkImportSemaphoreFdInfoKHX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub semaphore: VkSemaphore,
    pub handleType: VkExternalSemaphoreHandleTypeFlagBitsKHX,
    pub fd: ::std::os::raw::c_int,
}
impl Clone for VkImportSemaphoreFdInfoKHX {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkImportSemaphoreFdKHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pImportSemaphoreFdInfo: *const VkImportSemaphoreFdInfoKHX,
    ) -> VkResult,
>;
pub type PFN_vkGetSemaphoreFdKHX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        semaphore: VkSemaphore,
        handleType: VkExternalSemaphoreHandleTypeFlagBitsKHX,
        pFd: *mut ::std::os::raw::c_int,
    ) -> VkResult,
>;
extern "C" {
    pub fn vkImportSemaphoreFdKHX(
        device: VkDevice,
        pImportSemaphoreFdInfo: *const VkImportSemaphoreFdInfoKHX,
    ) -> VkResult;
}
extern "C" {
    pub fn vkGetSemaphoreFdKHX(
        device: VkDevice,
        semaphore: VkSemaphore,
        handleType: VkExternalSemaphoreHandleTypeFlagBitsKHX,
        pFd: *mut ::std::os::raw::c_int,
    ) -> VkResult;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VkObjectTableNVX_T {
    _unused: [u8; 0],
}
pub type VkObjectTableNVX = *mut VkObjectTableNVX_T;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VkIndirectCommandsLayoutNVX_T {
    _unused: [u8; 0],
}
pub type VkIndirectCommandsLayoutNVX = *mut VkIndirectCommandsLayoutNVX_T;
pub const VkIndirectCommandsTokenTypeNVX_VK_INDIRECT_COMMANDS_TOKEN_TYPE_BEGIN_RANGE_NVX:
    VkIndirectCommandsTokenTypeNVX =
    VkIndirectCommandsTokenTypeNVX::VK_INDIRECT_COMMANDS_TOKEN_PIPELINE_NVX;
pub const VkIndirectCommandsTokenTypeNVX_VK_INDIRECT_COMMANDS_TOKEN_TYPE_END_RANGE_NVX:
    VkIndirectCommandsTokenTypeNVX =
    VkIndirectCommandsTokenTypeNVX::VK_INDIRECT_COMMANDS_TOKEN_DISPATCH_NVX;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkIndirectCommandsTokenTypeNVX {
    VK_INDIRECT_COMMANDS_TOKEN_PIPELINE_NVX = 0,
    VK_INDIRECT_COMMANDS_TOKEN_DESCRIPTOR_SET_NVX = 1,
    VK_INDIRECT_COMMANDS_TOKEN_INDEX_BUFFER_NVX = 2,
    VK_INDIRECT_COMMANDS_TOKEN_VERTEX_BUFFER_NVX = 3,
    VK_INDIRECT_COMMANDS_TOKEN_PUSH_CONSTANT_NVX = 4,
    VK_INDIRECT_COMMANDS_TOKEN_DRAW_INDEXED_NVX = 5,
    VK_INDIRECT_COMMANDS_TOKEN_DRAW_NVX = 6,
    VK_INDIRECT_COMMANDS_TOKEN_DISPATCH_NVX = 7,
    VK_INDIRECT_COMMANDS_TOKEN_TYPE_RANGE_SIZE_NVX = 8,
    VK_INDIRECT_COMMANDS_TOKEN_TYPE_MAX_ENUM_NVX = 2147483647,
}
pub const VkObjectEntryTypeNVX_VK_OBJECT_ENTRY_TYPE_BEGIN_RANGE_NVX: VkObjectEntryTypeNVX =
    VkObjectEntryTypeNVX::VK_OBJECT_ENTRY_DESCRIPTOR_SET_NVX;
pub const VkObjectEntryTypeNVX_VK_OBJECT_ENTRY_TYPE_END_RANGE_NVX: VkObjectEntryTypeNVX =
    VkObjectEntryTypeNVX::VK_OBJECT_ENTRY_PUSH_CONSTANT_NVX;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkObjectEntryTypeNVX {
    VK_OBJECT_ENTRY_DESCRIPTOR_SET_NVX = 0,
    VK_OBJECT_ENTRY_PIPELINE_NVX = 1,
    VK_OBJECT_ENTRY_INDEX_BUFFER_NVX = 2,
    VK_OBJECT_ENTRY_VERTEX_BUFFER_NVX = 3,
    VK_OBJECT_ENTRY_PUSH_CONSTANT_NVX = 4,
    VK_OBJECT_ENTRY_TYPE_RANGE_SIZE_NVX = 5,
    VK_OBJECT_ENTRY_TYPE_MAX_ENUM_NVX = 2147483647,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkIndirectCommandsLayoutUsageFlagBitsNVX {
    VK_INDIRECT_COMMANDS_LAYOUT_USAGE_UNORDERED_SEQUENCES_BIT_NVX = 1,
    VK_INDIRECT_COMMANDS_LAYOUT_USAGE_SPARSE_SEQUENCES_BIT_NVX = 2,
    VK_INDIRECT_COMMANDS_LAYOUT_USAGE_EMPTY_EXECUTIONS_BIT_NVX = 4,
    VK_INDIRECT_COMMANDS_LAYOUT_USAGE_INDEXED_SEQUENCES_BIT_NVX = 8,
    VK_INDIRECT_COMMANDS_LAYOUT_USAGE_FLAG_BITS_MAX_ENUM_NVX = 2147483647,
}
pub type VkIndirectCommandsLayoutUsageFlagsNVX = VkFlags;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkObjectEntryUsageFlagBitsNVX {
    VK_OBJECT_ENTRY_USAGE_GRAPHICS_BIT_NVX = 1,
    VK_OBJECT_ENTRY_USAGE_COMPUTE_BIT_NVX = 2,
    VK_OBJECT_ENTRY_USAGE_FLAG_BITS_MAX_ENUM_NVX = 2147483647,
}
pub type VkObjectEntryUsageFlagsNVX = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGeneratedCommandsFeaturesNVX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub computeBindingPointSupport: VkBool32,
}
impl Clone for VkDeviceGeneratedCommandsFeaturesNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceGeneratedCommandsLimitsNVX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub maxIndirectCommandsLayoutTokenCount: u32,
    pub maxObjectEntryCounts: u32,
    pub minSequenceCountBufferOffsetAlignment: u32,
    pub minSequenceIndexBufferOffsetAlignment: u32,
    pub minCommandsTokenBufferOffsetAlignment: u32,
}
impl Clone for VkDeviceGeneratedCommandsLimitsNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkIndirectCommandsTokenNVX {
    pub tokenType: VkIndirectCommandsTokenTypeNVX,
    pub buffer: VkBuffer,
    pub offset: VkDeviceSize,
}
impl Clone for VkIndirectCommandsTokenNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkIndirectCommandsLayoutTokenNVX {
    pub tokenType: VkIndirectCommandsTokenTypeNVX,
    pub bindingUnit: u32,
    pub dynamicCount: u32,
    pub divisor: u32,
}
impl Clone for VkIndirectCommandsLayoutTokenNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkIndirectCommandsLayoutCreateInfoNVX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub pipelineBindPoint: VkPipelineBindPoint,
    pub flags: VkIndirectCommandsLayoutUsageFlagsNVX,
    pub tokenCount: u32,
    pub pTokens: *const VkIndirectCommandsLayoutTokenNVX,
}
impl Clone for VkIndirectCommandsLayoutCreateInfoNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkCmdProcessCommandsInfoNVX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub objectTable: VkObjectTableNVX,
    pub indirectCommandsLayout: VkIndirectCommandsLayoutNVX,
    pub indirectCommandsTokenCount: u32,
    pub pIndirectCommandsTokens: *const VkIndirectCommandsTokenNVX,
    pub maxSequencesCount: u32,
    pub targetCommandBuffer: VkCommandBuffer,
    pub sequencesCountBuffer: VkBuffer,
    pub sequencesCountOffset: VkDeviceSize,
    pub sequencesIndexBuffer: VkBuffer,
    pub sequencesIndexOffset: VkDeviceSize,
}
impl Clone for VkCmdProcessCommandsInfoNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkCmdReserveSpaceForCommandsInfoNVX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub objectTable: VkObjectTableNVX,
    pub indirectCommandsLayout: VkIndirectCommandsLayoutNVX,
    pub maxSequencesCount: u32,
}
impl Clone for VkCmdReserveSpaceForCommandsInfoNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkObjectTableCreateInfoNVX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub objectCount: u32,
    pub pObjectEntryTypes: *const VkObjectEntryTypeNVX,
    pub pObjectEntryCounts: *const u32,
    pub pObjectEntryUsageFlags: *const VkObjectEntryUsageFlagsNVX,
    pub maxUniformBuffersPerDescriptor: u32,
    pub maxStorageBuffersPerDescriptor: u32,
    pub maxStorageImagesPerDescriptor: u32,
    pub maxSampledImagesPerDescriptor: u32,
    pub maxPipelineLayouts: u32,
}
impl Clone for VkObjectTableCreateInfoNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkObjectTableEntryNVX {
    pub type_: VkObjectEntryTypeNVX,
    pub flags: VkObjectEntryUsageFlagsNVX,
}
impl Clone for VkObjectTableEntryNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkObjectTablePipelineEntryNVX {
    pub type_: VkObjectEntryTypeNVX,
    pub flags: VkObjectEntryUsageFlagsNVX,
    pub pipeline: VkPipeline,
}
impl Clone for VkObjectTablePipelineEntryNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkObjectTableDescriptorSetEntryNVX {
    pub type_: VkObjectEntryTypeNVX,
    pub flags: VkObjectEntryUsageFlagsNVX,
    pub pipelineLayout: VkPipelineLayout,
    pub descriptorSet: VkDescriptorSet,
}
impl Clone for VkObjectTableDescriptorSetEntryNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkObjectTableVertexBufferEntryNVX {
    pub type_: VkObjectEntryTypeNVX,
    pub flags: VkObjectEntryUsageFlagsNVX,
    pub buffer: VkBuffer,
}
impl Clone for VkObjectTableVertexBufferEntryNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkObjectTableIndexBufferEntryNVX {
    pub type_: VkObjectEntryTypeNVX,
    pub flags: VkObjectEntryUsageFlagsNVX,
    pub buffer: VkBuffer,
    pub indexType: VkIndexType,
}
impl Clone for VkObjectTableIndexBufferEntryNVX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkObjectTablePushConstantEntryNVX {
    pub type_: VkObjectEntryTypeNVX,
    pub flags: VkObjectEntryUsageFlagsNVX,
    pub pipelineLayout: VkPipelineLayout,
    pub stageFlags: VkShaderStageFlags,
}
impl Clone for VkObjectTablePushConstantEntryNVX {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCmdProcessCommandsNVX = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pProcessCommandsInfo: *const VkCmdProcessCommandsInfoNVX,
    ),
>;
pub type PFN_vkCmdReserveSpaceForCommandsNVX = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        pReserveSpaceInfo: *const VkCmdReserveSpaceForCommandsInfoNVX,
    ),
>;
pub type PFN_vkCreateIndirectCommandsLayoutNVX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkIndirectCommandsLayoutCreateInfoNVX,
        pAllocator: *const VkAllocationCallbacks,
        pIndirectCommandsLayout: *mut VkIndirectCommandsLayoutNVX,
    ) -> VkResult,
>;
pub type PFN_vkDestroyIndirectCommandsLayoutNVX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        indirectCommandsLayout: VkIndirectCommandsLayoutNVX,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkCreateObjectTableNVX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        pCreateInfo: *const VkObjectTableCreateInfoNVX,
        pAllocator: *const VkAllocationCallbacks,
        pObjectTable: *mut VkObjectTableNVX,
    ) -> VkResult,
>;
pub type PFN_vkDestroyObjectTableNVX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        objectTable: VkObjectTableNVX,
        pAllocator: *const VkAllocationCallbacks,
    ),
>;
pub type PFN_vkRegisterObjectsNVX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        objectTable: VkObjectTableNVX,
        objectCount: u32,
        ppObjectTableEntries: *const *const VkObjectTableEntryNVX,
        pObjectIndices: *const u32,
    ) -> VkResult,
>;
pub type PFN_vkUnregisterObjectsNVX = ::std::option::Option<
    unsafe extern "C" fn(
        device: VkDevice,
        objectTable: VkObjectTableNVX,
        objectCount: u32,
        pObjectEntryTypes: *const VkObjectEntryTypeNVX,
        pObjectIndices: *const u32,
    ) -> VkResult,
>;
pub type PFN_vkGetPhysicalDeviceGeneratedCommandsPropertiesNVX = ::std::option::Option<
    unsafe extern "C" fn(
        physicalDevice: VkPhysicalDevice,
        pFeatures: *mut VkDeviceGeneratedCommandsFeaturesNVX,
        pLimits: *mut VkDeviceGeneratedCommandsLimitsNVX,
    ),
>;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkViewportWScalingNV {
    pub xcoeff: f32,
    pub ycoeff: f32,
}
impl Clone for VkViewportWScalingNV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineViewportWScalingStateCreateInfoNV {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub viewportWScalingEnable: VkBool32,
    pub viewportCount: u32,
    pub pViewportWScalings: *const VkViewportWScalingNV,
}
impl Clone for VkPipelineViewportWScalingStateCreateInfoNV {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCmdSetViewportWScalingNV = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        firstViewport: u32,
        viewportCount: u32,
        pViewportWScalings: *const VkViewportWScalingNV,
    ),
>;

pub type PFN_vkReleaseDisplayEXT = ::std::option::Option<
    unsafe extern "C" fn(physicalDevice: VkPhysicalDevice, display: VkDisplayKHR) -> VkResult,
>;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkSurfaceCounterFlagBitsEXT {
    VK_SURFACE_COUNTER_VBLANK_EXT = 1,
    VK_SURFACE_COUNTER_FLAG_BITS_MAX_ENUM_EXT = 2147483647,
}
pub type VkSurfaceCounterFlagsEXT = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSurfaceCapabilities2EXT {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub minImageCount: u32,
    pub maxImageCount: u32,
    pub currentExtent: VkExtent2D,
    pub minImageExtent: VkExtent2D,
    pub maxImageExtent: VkExtent2D,
    pub maxImageArrayLayers: u32,
    pub supportedTransforms: VkSurfaceTransformFlagsKHR,
    pub currentTransform: VkSurfaceTransformFlagBitsKHR,
    pub supportedCompositeAlpha: VkCompositeAlphaFlagsKHR,
    pub supportedUsageFlags: VkImageUsageFlags,
    pub supportedSurfaceCounters: VkSurfaceCounterFlagsEXT,
}
impl Clone for VkSurfaceCapabilities2EXT {
    fn clone(&self) -> Self {
        *self
    }
}
pub const VkDisplayPowerStateEXT_VK_DISPLAY_POWER_STATE_BEGIN_RANGE_EXT: VkDisplayPowerStateEXT =
    VkDisplayPowerStateEXT::VK_DISPLAY_POWER_STATE_OFF_EXT;
pub const VkDisplayPowerStateEXT_VK_DISPLAY_POWER_STATE_END_RANGE_EXT: VkDisplayPowerStateEXT =
    VkDisplayPowerStateEXT::VK_DISPLAY_POWER_STATE_ON_EXT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDisplayPowerStateEXT {
    VK_DISPLAY_POWER_STATE_OFF_EXT = 0,
    VK_DISPLAY_POWER_STATE_SUSPEND_EXT = 1,
    VK_DISPLAY_POWER_STATE_ON_EXT = 2,
    VK_DISPLAY_POWER_STATE_RANGE_SIZE_EXT = 3,
    VK_DISPLAY_POWER_STATE_MAX_ENUM_EXT = 2147483647,
}
pub const VkDeviceEventTypeEXT_VK_DEVICE_EVENT_TYPE_BEGIN_RANGE_EXT: VkDeviceEventTypeEXT =
    VkDeviceEventTypeEXT::VK_DEVICE_EVENT_TYPE_DISPLAY_HOTPLUG_EXT;
pub const VkDeviceEventTypeEXT_VK_DEVICE_EVENT_TYPE_END_RANGE_EXT: VkDeviceEventTypeEXT =
    VkDeviceEventTypeEXT::VK_DEVICE_EVENT_TYPE_DISPLAY_HOTPLUG_EXT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDeviceEventTypeEXT {
    VK_DEVICE_EVENT_TYPE_DISPLAY_HOTPLUG_EXT = 0,
    VK_DEVICE_EVENT_TYPE_RANGE_SIZE_EXT = 1,
    VK_DEVICE_EVENT_TYPE_MAX_ENUM_EXT = 2147483647,
}
pub const VkDisplayEventTypeEXT_VK_DISPLAY_EVENT_TYPE_BEGIN_RANGE_EXT: VkDisplayEventTypeEXT =
    VkDisplayEventTypeEXT::VK_DISPLAY_EVENT_TYPE_FIRST_PIXEL_OUT_EXT;
pub const VkDisplayEventTypeEXT_VK_DISPLAY_EVENT_TYPE_END_RANGE_EXT: VkDisplayEventTypeEXT =
    VkDisplayEventTypeEXT::VK_DISPLAY_EVENT_TYPE_FIRST_PIXEL_OUT_EXT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDisplayEventTypeEXT {
    VK_DISPLAY_EVENT_TYPE_FIRST_PIXEL_OUT_EXT = 0,
    VK_DISPLAY_EVENT_TYPE_RANGE_SIZE_EXT = 1,
    VK_DISPLAY_EVENT_TYPE_MAX_ENUM_EXT = 2147483647,
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayPowerInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub powerState: VkDisplayPowerStateEXT,
}
impl Clone for VkDisplayPowerInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDeviceEventInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub deviceEvent: VkDeviceEventTypeEXT,
}
impl Clone for VkDeviceEventInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkDisplayEventInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub displayEvent: VkDisplayEventTypeEXT,
}
impl Clone for VkDisplayEventInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkSwapchainCounterCreateInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub surfaceCounters: VkSurfaceCounterFlagsEXT,
}
impl Clone for VkSwapchainCounterCreateInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceMultiviewPerViewAttributesPropertiesNVX {
    pub sType: VkStructureType,
    pub pNext: *mut ::std::os::raw::c_void,
    pub perViewPositionAllComponents: VkBool32,
}
impl Clone for VkPhysicalDeviceMultiviewPerViewAttributesPropertiesNVX {
    fn clone(&self) -> Self {
        *self
    }
}
pub const VkViewportCoordinateSwizzleNV_VK_VIEWPORT_COORDINATE_SWIZZLE_BEGIN_RANGE_NV:
    VkViewportCoordinateSwizzleNV =
    VkViewportCoordinateSwizzleNV::VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_X_NV;
pub const VkViewportCoordinateSwizzleNV_VK_VIEWPORT_COORDINATE_SWIZZLE_END_RANGE_NV:
    VkViewportCoordinateSwizzleNV =
    VkViewportCoordinateSwizzleNV::VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_W_NV;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkViewportCoordinateSwizzleNV {
    VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_X_NV = 0,
    VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_X_NV = 1,
    VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_Y_NV = 2,
    VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_Y_NV = 3,
    VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_Z_NV = 4,
    VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_Z_NV = 5,
    VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_W_NV = 6,
    VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_W_NV = 7,
    VK_VIEWPORT_COORDINATE_SWIZZLE_RANGE_SIZE_NV = 8,
    VK_VIEWPORT_COORDINATE_SWIZZLE_MAX_ENUM_NV = 2147483647,
}
pub type VkPipelineViewportSwizzleStateCreateFlagsNV = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkViewportSwizzleNV {
    pub x: VkViewportCoordinateSwizzleNV,
    pub y: VkViewportCoordinateSwizzleNV,
    pub z: VkViewportCoordinateSwizzleNV,
    pub w: VkViewportCoordinateSwizzleNV,
}
impl Clone for VkViewportSwizzleNV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineViewportSwizzleStateCreateInfoNV {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineViewportSwizzleStateCreateFlagsNV,
    pub viewportCount: u32,
    pub pViewportSwizzles: *const VkViewportSwizzleNV,
}
impl Clone for VkPipelineViewportSwizzleStateCreateInfoNV {
    fn clone(&self) -> Self {
        *self
    }
}
pub const VkDiscardRectangleModeEXT_VK_DISCARD_RECTANGLE_MODE_BEGIN_RANGE_EXT:
    VkDiscardRectangleModeEXT = VkDiscardRectangleModeEXT::VK_DISCARD_RECTANGLE_MODE_INCLUSIVE_EXT;
pub const VkDiscardRectangleModeEXT_VK_DISCARD_RECTANGLE_MODE_END_RANGE_EXT:
    VkDiscardRectangleModeEXT = VkDiscardRectangleModeEXT::VK_DISCARD_RECTANGLE_MODE_EXCLUSIVE_EXT;
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VkDiscardRectangleModeEXT {
    VK_DISCARD_RECTANGLE_MODE_INCLUSIVE_EXT = 0,
    VK_DISCARD_RECTANGLE_MODE_EXCLUSIVE_EXT = 1,
    VK_DISCARD_RECTANGLE_MODE_RANGE_SIZE_EXT = 2,
    VK_DISCARD_RECTANGLE_MODE_MAX_ENUM_EXT = 2147483647,
}
pub type VkPipelineDiscardRectangleStateCreateFlagsEXT = VkFlags;
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceDiscardRectanglePropertiesEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub maxDiscardRectangles: u32,
}
impl Clone for VkPhysicalDeviceDiscardRectanglePropertiesEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPipelineDiscardRectangleStateCreateInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkPipelineDiscardRectangleStateCreateFlagsEXT,
    pub discardRectangleMode: VkDiscardRectangleModeEXT,
    pub discardRectangleCount: u32,
    pub pDiscardRectangles: *const VkRect2D,
}
impl Clone for VkPipelineDiscardRectangleStateCreateInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PFN_vkCmdSetDiscardRectangleEXT = ::std::option::Option<
    unsafe extern "C" fn(
        commandBuffer: VkCommandBuffer,
        firstDiscardRectangle: u32,
        discardRectangleCount: u32,
        pDiscardRectangles: *const VkRect2D,
    ),
>;

pub type PFN_vkCreateInstance = ::std::option::Option<
    unsafe extern "C" fn(
        pCreateInfo: *const VkInstanceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pInstance: *mut VkInstance,
    ) -> VkResult,
>;

pub type PFN_vkEnumeratePhysicalDevices = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pPhysicalDeviceCount: *mut u32,
        pPhysicalDevices: *mut VkPhysicalDevice,
    ) -> VkResult,
>;

pub type PFN_vkDestroyInstance = ::std::option::Option<
    unsafe extern "C" fn(instance: VkInstance, pAllocator: *const VkAllocationCallbacks),
>;

pub type PFN_vkCreateWin32SurfaceKHR = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pCreateInfo: *const VkWin32SurfaceCreateInfoKHR,
        pAllocator: *const VkAllocationCallbacks,
        pSurface: *mut VkSurfaceKHR,
    ) -> VkResult,
>;

pub type PFN_vkCreateMetalSurfaceEXT = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pCreateInfo: *const VkMetalSurfaceCreateInfoEXT,
        pAllocator: *const VkAllocationCallbacks,
        pSurface: *mut VkSurfaceKHR,
    ) -> VkResult,
>;

pub type PFN_vkCreateMacOSSurfaceMVK = ::std::option::Option<
    unsafe extern "C" fn(
        instance: VkInstance,
        pCreateInfo: *const VkMacOSSurfaceCreateInfoMVK,
        pAllocator: *const VkAllocationCallbacks,
        pSurface: *mut VkSurfaceKHR,
    ) -> VkResult,
>;

pub type PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR = ::std::option::Option<
    unsafe extern "C" fn(physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32) -> VkBool32,
>;

#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDevicePortabilitySubsetFeaturesEXTX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub triangleFans: VkBool32,
    pub separateStencilMaskRef: VkBool32,
    pub events: VkBool32,
    pub standardImageViews: VkBool32,
    pub samplerMipLodBias: VkBool32,
}
impl Clone for VkPhysicalDevicePortabilitySubsetFeaturesEXTX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDevicePortabilitySubsetPropertiesEXTX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub minVertexInputBindingStrideAlignment: u32,
}
impl Clone for VkPhysicalDevicePortabilitySubsetPropertiesEXTX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkPhysicalDeviceImageViewSupportEXTX {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkImageViewCreateFlags,
    pub viewType: VkImageViewType,
    pub format: VkFormat,
    pub components: VkComponentMapping,
    pub aspectMask: VkImageAspectFlags,
}
impl Clone for VkPhysicalDeviceImageViewSupportEXTX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct VkMetalSurfaceCreateInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const ::std::os::raw::c_void,
    pub flags: VkMetalSurfaceCreateFlagsEXT,
    pub pLayer: *const ::std::os::raw::c_void,
}
impl Clone for VkMetalSurfaceCreateInfoEXT {
    fn clone(&self) -> Self {
        *self
    }
}
