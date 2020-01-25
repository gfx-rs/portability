use crate::hal::{buffer, command, device, format, image, memory, pass, pso, query, window};
use crate::hal::{pso::PatchSize, pso::Primitive, Features, IndexType, Limits};

use std::mem;

use super::*;

pub fn limits_from_hal(limits: Limits) -> VkPhysicalDeviceLimits {
    let viewport_size = limits.max_viewport_dimensions[0].max(limits.max_viewport_dimensions[1]);
    VkPhysicalDeviceLimits {
        maxImageDimension1D: limits.max_image_1d_size,
        maxImageDimension2D: limits.max_image_2d_size,
        maxImageDimension3D: limits.max_image_3d_size,
        maxImageDimensionCube: limits.max_image_cube_size,
        maxFramebufferWidth: limits.max_framebuffer_extent.width,
        maxFramebufferHeight: limits.max_framebuffer_extent.height,
        maxTexelBufferElements: limits.max_texel_elements as _,
        maxTessellationPatchSize: limits.max_patch_size as _,
        maxPushConstantsSize: limits.max_push_constants_size as _,
        maxViewports: limits.max_viewports as _,
        maxViewportDimensions: limits.max_viewport_dimensions,
        maxBoundDescriptorSets: limits.max_bound_descriptor_sets as _,
        maxPerStageDescriptorUniformBuffers: limits.max_per_stage_descriptor_uniform_buffers as _,
        maxDescriptorSetUniformBuffers: limits.max_descriptor_set_uniform_buffers as _,
        maxFragmentInputComponents: limits.max_fragment_input_components as _,
        maxFramebufferLayers: limits.max_framebuffer_layers as _,
        maxMemoryAllocationCount: limits.max_memory_allocation_count as _,
        maxUniformBufferRange: limits.max_uniform_buffer_range as _,
        // Warning: spec violation
        // "The x/y rectangle of the viewport must lie entirely within the current attachment size."
        viewportBoundsRange: [0.0, viewport_size as f32],
        maxVertexInputAttributes: limits.max_vertex_input_attributes as _,
        maxVertexInputBindings: limits.max_vertex_input_bindings as _,
        maxVertexInputAttributeOffset: limits.max_vertex_input_attribute_offset as _,
        maxVertexInputBindingStride: limits.max_vertex_input_binding_stride as _,
        maxVertexOutputComponents: limits.max_vertex_output_components as _,
        maxComputeWorkGroupCount: limits.max_compute_work_group_count,
        maxComputeWorkGroupSize: limits.max_compute_work_group_size,
        bufferImageGranularity: limits.buffer_image_granularity,
        minTexelBufferOffsetAlignment: limits.min_texel_buffer_offset_alignment,
        minUniformBufferOffsetAlignment: limits.min_uniform_buffer_offset_alignment,
        minStorageBufferOffsetAlignment: limits.min_storage_buffer_offset_alignment,
        framebufferColorSampleCounts: limits.framebuffer_color_sample_counts as _,
        framebufferDepthSampleCounts: limits.framebuffer_depth_sample_counts as _,
        framebufferStencilSampleCounts: limits.framebuffer_stencil_sample_counts as _,
        maxColorAttachments: limits.max_color_attachments as _,
        nonCoherentAtomSize: limits.non_coherent_atom_size as _,
        maxSamplerAnisotropy: limits.max_sampler_anisotropy,
        optimalBufferCopyOffsetAlignment: limits.optimal_buffer_copy_offset_alignment,
        optimalBufferCopyRowPitchAlignment: limits.optimal_buffer_copy_pitch_alignment,
        maxPerStageDescriptorSampledImages: limits.max_per_stage_descriptor_sampled_images as _,
        maxPerStageDescriptorSamplers: limits.max_per_stage_descriptor_samplers as _,
        maxDescriptorSetSampledImages: limits.max_descriptor_set_sampled_images as _,
        maxDescriptorSetSamplers: limits.max_descriptor_set_samplers as _,
        ..unsafe { mem::zeroed() } //TODO
    }
}

pub fn features_from_hal(features: Features) -> VkPhysicalDeviceFeatures {
    VkPhysicalDeviceFeatures {
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
        shaderTessellationAndGeometryPointSize: features
            .contains(Features::SHADER_TESSELLATION_AND_GEOMETRY_POINT_SIZE)
            as _,
        shaderImageGatherExtended: features.contains(Features::SHADER_IMAGE_GATHER_EXTENDED) as _,
        shaderStorageImageExtendedFormats: features
            .contains(Features::SHADER_STORAGE_IMAGE_EXTENDED_FORMATS)
            as _,
        shaderStorageImageMultisample: features.contains(Features::SHADER_STORAGE_IMAGE_MULTISAMPLE)
            as _,
        shaderStorageImageReadWithoutFormat: features
            .contains(Features::SHADER_STORAGE_IMAGE_READ_WITHOUT_FORMAT)
            as _,
        shaderStorageImageWriteWithoutFormat: features
            .contains(Features::SHADER_STORAGE_IMAGE_WRITE_WITHOUT_FORMAT)
            as _,
        shaderUniformBufferArrayDynamicIndexing: features
            .contains(Features::SHADER_UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING)
            as _,
        shaderSampledImageArrayDynamicIndexing: features
            .contains(Features::SHADER_SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING)
            as _,
        shaderStorageBufferArrayDynamicIndexing: features
            .contains(Features::SHADER_STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING)
            as _,
        shaderStorageImageArrayDynamicIndexing: features
            .contains(Features::SHADER_STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING)
            as _,
        shaderClipDistance: features.contains(Features::SHADER_CLIP_DISTANCE) as _,
        shaderCullDistance: features.contains(Features::SHADER_CULL_DISTANCE) as _,
        shaderFloat64: features.contains(Features::SHADER_FLOAT64) as _,
        shaderInt64: features.contains(Features::SHADER_INT64) as _,
        shaderInt16: features.contains(Features::SHADER_INT16) as _,
        shaderResourceResidency: features.contains(Features::SHADER_RESOURCE_RESIDENCY) as _,
        shaderResourceMinLod: features.contains(Features::SHADER_RESOURCE_MIN_LOD) as _,
        sparseBinding: features.contains(Features::SPARSE_BINDING) as _,
        sparseResidencyBuffer: features.contains(Features::SPARSE_RESIDENCY_BUFFER) as _,
        sparseResidencyImage2D: features.contains(Features::SPARSE_RESIDENCY_IMAGE_2D) as _,
        sparseResidencyImage3D: features.contains(Features::SPARSE_RESIDENCY_IMAGE_3D) as _,
        sparseResidency2Samples: features.contains(Features::SPARSE_RESIDENCY_2_SAMPLES) as _,
        sparseResidency4Samples: features.contains(Features::SPARSE_RESIDENCY_4_SAMPLES) as _,
        sparseResidency8Samples: features.contains(Features::SPARSE_RESIDENCY_8_SAMPLES) as _,
        sparseResidency16Samples: features.contains(Features::SPARSE_RESIDENCY_16_SAMPLES) as _,
        sparseResidencyAliased: features.contains(Features::SPARSE_RESIDENCY_ALIASED) as _,
        variableMultisampleRate: features.contains(Features::VARIABLE_MULTISAMPLE_RATE) as _,
        inheritedQueries: features.contains(Features::INHERITED_QUERIES) as _,
    }
}

#[inline]
pub fn format_from_hal(format: format::Format) -> VkFormat {
    // HAL formats have the same numeric representation as Vulkan formats
    unsafe { mem::transmute(format) }
}

pub fn format_properties_from_hal(properties: format::Properties) -> VkFormatProperties {
    VkFormatProperties {
        linearTilingFeatures: image_features_from_hal(properties.linear_tiling),
        optimalTilingFeatures: image_features_from_hal(properties.optimal_tiling),
        bufferFeatures: buffer_features_from_hal(properties.buffer_features),
    }
}

pub fn image_format_properties_from_hal(
    properties: image::FormatProperties,
) -> VkImageFormatProperties {
    VkImageFormatProperties {
        maxExtent: extent3d_from_hal(properties.max_extent),
        maxMipLevels: properties.max_levels as _,
        maxArrayLayers: properties.max_layers as _,
        sampleCounts: properties.sample_count_mask as _,
        maxResourceSize: properties.max_resource_size as _,
    }
}

fn image_features_from_hal(features: format::ImageFeature) -> VkFormatFeatureFlags {
    let mut flags = 0;

    if features.contains(format::ImageFeature::SAMPLED) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT as u32;
    }
    if features.contains(format::ImageFeature::STORAGE) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT as u32;
    }
    if features.contains(format::ImageFeature::STORAGE_ATOMIC) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_IMAGE_ATOMIC_BIT as u32;
    }
    if features.contains(format::ImageFeature::COLOR_ATTACHMENT) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT as u32;
    }
    if features.contains(format::ImageFeature::COLOR_ATTACHMENT_BLEND) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT as u32;
    }
    if features.contains(format::ImageFeature::DEPTH_STENCIL_ATTACHMENT) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT as u32;
    }
    if features.contains(format::ImageFeature::BLIT_SRC) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_BLIT_SRC_BIT as u32;
    }
    if features.contains(format::ImageFeature::BLIT_DST) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_BLIT_DST_BIT as u32;
    }
    if features.contains(format::ImageFeature::SAMPLED_LINEAR) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT as u32;
    }

    flags
}

fn buffer_features_from_hal(features: format::BufferFeature) -> VkFormatFeatureFlags {
    let mut flags = 0;

    if features.contains(format::BufferFeature::UNIFORM_TEXEL) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT as u32;
    }
    if features.contains(format::BufferFeature::STORAGE_TEXEL) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_BIT as u32;
    }
    if features.contains(format::BufferFeature::STORAGE_TEXEL_ATOMIC) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_ATOMIC_BIT as u32;
    }
    if features.contains(format::BufferFeature::VERTEX) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT as u32;
    }

    flags
}

pub fn map_format(format: VkFormat) -> Option<format::Format> {
    if format == VkFormat::VK_FORMAT_UNDEFINED {
        None
    } else if (format as usize) < format::NUM_FORMATS {
        // HAL formats have the same numeric representation as Vulkan formats
        Some(unsafe { mem::transmute(format) })
    } else {
        unimplemented!("Unknown format {:?}", format);
    }
}

pub fn extent2d_from_hal(extent: window::Extent2D) -> VkExtent2D {
    VkExtent2D {
        width: extent.width,
        height: extent.height,
    }
}

pub fn map_extent2d(extent: VkExtent2D) -> window::Extent2D {
    window::Extent2D {
        width: extent.width,
        height: extent.height,
    }
}

pub fn extent3d_from_hal(extent: image::Extent) -> VkExtent3D {
    VkExtent3D {
        width: extent.width,
        height: extent.height,
        depth: extent.depth,
    }
}

pub fn map_swizzle(components: VkComponentMapping) -> format::Swizzle {
    format::Swizzle(
        map_swizzle_component(components.r, format::Component::R),
        map_swizzle_component(components.g, format::Component::G),
        map_swizzle_component(components.b, format::Component::B),
        map_swizzle_component(components.a, format::Component::A),
    )
}

fn map_swizzle_component(
    component: VkComponentSwizzle,
    identity: format::Component,
) -> format::Component {
    use crate::VkComponentSwizzle::*;

    match component {
        VK_COMPONENT_SWIZZLE_IDENTITY => identity,
        VK_COMPONENT_SWIZZLE_ZERO => format::Component::Zero,
        VK_COMPONENT_SWIZZLE_ONE => format::Component::One,
        VK_COMPONENT_SWIZZLE_R => format::Component::R,
        VK_COMPONENT_SWIZZLE_G => format::Component::G,
        VK_COMPONENT_SWIZZLE_B => format::Component::B,
        VK_COMPONENT_SWIZZLE_A => format::Component::A,
        _ => panic!("Unsupported swizzle component: {:?}", component),
    }
}

pub fn map_aspect(aspects: VkImageAspectFlags) -> format::Aspects {
    let mut flags = format::Aspects::empty();
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_COLOR_BIT as u32 != 0 {
        flags |= format::Aspects::COLOR;
    }
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_DEPTH_BIT as u32 != 0 {
        flags |= format::Aspects::DEPTH;
    }
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_STENCIL_BIT as u32 != 0 {
        flags |= format::Aspects::DEPTH;
    }
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_METADATA_BIT as u32 != 0 {
        unimplemented!()
    }
    flags
}

pub fn map_image_create_flags(flags: VkImageCreateFlags) -> image::ViewCapabilities {
    image::ViewCapabilities::from_bits_truncate(flags as u32)
}

pub fn map_image_kind(
    ty: VkImageType,
    extent: VkExtent3D,
    array_layers: image::Layer,
    samples: VkSampleCountFlagBits,
) -> image::Kind {
    debug_assert_ne!(array_layers, 0);
    match ty {
        VkImageType::VK_IMAGE_TYPE_1D => image::Kind::D1(extent.width as _, array_layers),
        VkImageType::VK_IMAGE_TYPE_2D => image::Kind::D2(
            extent.width as _,
            extent.height as _,
            array_layers,
            samples as _,
        ),
        VkImageType::VK_IMAGE_TYPE_3D => {
            image::Kind::D3(extent.width as _, extent.height as _, extent.depth as _)
        }
        _ => unreachable!(),
    }
}

pub fn map_view_kind(ty: VkImageViewType) -> image::ViewKind {
    match ty {
        VkImageViewType::VK_IMAGE_VIEW_TYPE_1D => image::ViewKind::D1,
        VkImageViewType::VK_IMAGE_VIEW_TYPE_1D_ARRAY => image::ViewKind::D1Array,
        VkImageViewType::VK_IMAGE_VIEW_TYPE_2D => image::ViewKind::D2,
        VkImageViewType::VK_IMAGE_VIEW_TYPE_2D_ARRAY => image::ViewKind::D2Array,
        VkImageViewType::VK_IMAGE_VIEW_TYPE_3D => image::ViewKind::D3,
        VkImageViewType::VK_IMAGE_VIEW_TYPE_CUBE => image::ViewKind::Cube,
        VkImageViewType::VK_IMAGE_VIEW_TYPE_CUBE_ARRAY => image::ViewKind::CubeArray,
        _ => unreachable!(),
    }
}

pub fn map_image_layout(layout: VkImageLayout) -> image::Layout {
    use crate::hal::image::Layout::*;
    match layout {
        VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED => Undefined,
        VkImageLayout::VK_IMAGE_LAYOUT_GENERAL => General,
        VkImageLayout::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL => ColorAttachmentOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
            DepthStencilAttachmentOptimal
        }
        VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL => {
            DepthStencilReadOnlyOptimal
        }
        VkImageLayout::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL => ShaderReadOnlyOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL => TransferSrcOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL => TransferDstOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_PREINITIALIZED => Preinitialized,
        VkImageLayout::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR => Present,
        _ => panic!("Unexpected image layout: {:?}", layout),
    }
}

pub fn map_image_usage(usage: VkImageUsageFlags) -> image::Usage {
    image::Usage::from_bits_truncate(usage as u32)
}

pub fn map_image_usage_from_hal(usage: image::Usage) -> VkImageUsageFlags {
    usage.bits()
}

pub fn map_image_access(access: VkAccessFlags) -> image::Access {
    let mut mask = image::Access::empty();

    if access & VkAccessFlagBits::VK_ACCESS_INPUT_ATTACHMENT_READ_BIT as u32 != 0 {
        mask |= image::Access::INPUT_ATTACHMENT_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_SHADER_READ_BIT as u32 != 0 {
        mask |= image::Access::SHADER_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_SHADER_WRITE_BIT as u32 != 0 {
        mask |= image::Access::SHADER_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_COLOR_ATTACHMENT_READ_BIT as u32 != 0 {
        mask |= image::Access::COLOR_ATTACHMENT_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT as u32 != 0 {
        mask |= image::Access::COLOR_ATTACHMENT_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT as u32 != 0 {
        mask |= image::Access::DEPTH_STENCIL_ATTACHMENT_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT as u32 != 0 {
        mask |= image::Access::DEPTH_STENCIL_ATTACHMENT_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_TRANSFER_READ_BIT as u32 != 0 {
        mask |= image::Access::TRANSFER_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_TRANSFER_WRITE_BIT as u32 != 0 {
        mask |= image::Access::TRANSFER_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_HOST_READ_BIT as u32 != 0 {
        mask |= image::Access::HOST_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_HOST_WRITE_BIT as u32 != 0 {
        mask |= image::Access::HOST_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_MEMORY_READ_BIT as u32 != 0 {
        mask |= image::Access::MEMORY_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_MEMORY_WRITE_BIT as u32 != 0 {
        mask |= image::Access::MEMORY_WRITE;
    }

    mask
}

pub fn map_buffer_usage(usage: VkBufferUsageFlags) -> buffer::Usage {
    let mut flags = buffer::Usage::empty();

    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_TRANSFER_SRC_BIT as u32 != 0 {
        flags |= buffer::Usage::TRANSFER_SRC;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_TRANSFER_DST_BIT as u32 != 0 {
        flags |= buffer::Usage::TRANSFER_DST;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT as u32 != 0 {
        flags |= buffer::Usage::UNIFORM_TEXEL;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT as u32 != 0 {
        flags |= buffer::Usage::STORAGE_TEXEL;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as u32 != 0 {
        flags |= buffer::Usage::UNIFORM;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_STORAGE_BUFFER_BIT as u32 != 0 {
        flags |= buffer::Usage::STORAGE;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_INDEX_BUFFER_BIT as u32 != 0 {
        flags |= buffer::Usage::INDEX;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT as u32 != 0 {
        flags |= buffer::Usage::VERTEX;
    }
    if usage & VkBufferUsageFlagBits::VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT as u32 != 0 {
        flags |= buffer::Usage::INDIRECT;
    }

    flags
}

pub fn map_buffer_access(access: VkAccessFlags) -> buffer::Access {
    let mut mask = buffer::Access::empty();

    if access & VkAccessFlagBits::VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT as u32 != 0 {
        mask |= buffer::Access::VERTEX_BUFFER_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_UNIFORM_READ_BIT as u32 != 0 {
        mask |= buffer::Access::UNIFORM_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_INDIRECT_COMMAND_READ_BIT as u32 != 0 {
        mask |= buffer::Access::INDIRECT_COMMAND_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_SHADER_READ_BIT as u32 != 0 {
        mask |= buffer::Access::SHADER_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_SHADER_WRITE_BIT as u32 != 0 {
        mask |= buffer::Access::SHADER_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_TRANSFER_READ_BIT as u32 != 0 {
        mask |= buffer::Access::TRANSFER_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_TRANSFER_WRITE_BIT as u32 != 0 {
        mask |= buffer::Access::TRANSFER_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_HOST_READ_BIT as u32 != 0 {
        mask |= buffer::Access::HOST_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_HOST_WRITE_BIT as u32 != 0 {
        mask |= buffer::Access::HOST_WRITE;
    }
    if access & VkAccessFlagBits::VK_ACCESS_MEMORY_READ_BIT as u32 != 0 {
        mask |= buffer::Access::MEMORY_READ;
    }
    if access & VkAccessFlagBits::VK_ACCESS_MEMORY_WRITE_BIT as u32 != 0 {
        mask |= buffer::Access::MEMORY_WRITE;
    }

    mask
}

pub fn memory_properties_from_hal(properties: memory::Properties) -> VkMemoryPropertyFlags {
    let mut flags = 0;

    if properties.contains(memory::Properties::DEVICE_LOCAL) {
        flags |= VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as u32;
    }
    if properties.contains(memory::Properties::COHERENT) {
        flags |= VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT as u32;
    }
    if properties.contains(memory::Properties::CPU_VISIBLE) {
        flags |= VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as u32;
    }
    if properties.contains(memory::Properties::CPU_CACHED) {
        flags |= VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_CACHED_BIT as u32;
    }
    if properties.contains(memory::Properties::LAZILY_ALLOCATED) {
        flags |= VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT as u32;
    }

    flags
}

pub fn map_descriptor_type(ty: VkDescriptorType) -> pso::DescriptorType {
    use super::VkDescriptorType::*;

    // TODO(krolli): Determining value of read_only in pso::BufferDescriptorType::Storage. Vulkan storage buffer variants always allow writes.
    match ty {
        VK_DESCRIPTOR_TYPE_SAMPLER => pso::DescriptorType::Sampler,
        VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE => pso::DescriptorType::Image {
            ty: pso::ImageDescriptorType::Sampled {
                with_sampler: false,
            },
        },
        VK_DESCRIPTOR_TYPE_STORAGE_IMAGE => pso::DescriptorType::Image {
            ty: pso::ImageDescriptorType::Storage,
        },
        VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER => pso::DescriptorType::Buffer {
            ty: pso::BufferDescriptorType::Uniform,
            format: pso::BufferDescriptorFormat::Texel,
        },
        VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER => pso::DescriptorType::Buffer {
            ty: pso::BufferDescriptorType::Storage { read_only: false },
            format: pso::BufferDescriptorFormat::Texel,
        },
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER => pso::DescriptorType::Buffer {
            ty: pso::BufferDescriptorType::Uniform,
            format: pso::BufferDescriptorFormat::Structured {
                dynamic_offset: false,
            },
        },
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER => pso::DescriptorType::Buffer {
            ty: pso::BufferDescriptorType::Storage { read_only: false },
            format: pso::BufferDescriptorFormat::Structured {
                dynamic_offset: false,
            },
        },
        VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT => pso::DescriptorType::InputAttachment,
        VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER => pso::DescriptorType::Image {
            ty: pso::ImageDescriptorType::Sampled { with_sampler: true },
        },
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC => pso::DescriptorType::Buffer {
            ty: pso::BufferDescriptorType::Uniform,
            format: pso::BufferDescriptorFormat::Structured {
                dynamic_offset: true,
            },
        },
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC => pso::DescriptorType::Buffer {
            ty: pso::BufferDescriptorType::Storage { read_only: false },
            format: pso::BufferDescriptorFormat::Structured {
                dynamic_offset: true,
            },
        },
        _ => panic!("Unexpected descriptor type: {:?}", ty),
    }
}

pub fn map_stage_flags(stages: VkShaderStageFlags) -> pso::ShaderStageFlags {
    let mut flags = pso::ShaderStageFlags::empty();

    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_VERTEX_BIT as u32 != 0 {
        flags |= pso::ShaderStageFlags::VERTEX;
    }
    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT as u32 != 0 {
        flags |= pso::ShaderStageFlags::HULL;
    }
    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT as u32 != 0 {
        flags |= pso::ShaderStageFlags::DOMAIN;
    }
    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_GEOMETRY_BIT as u32 != 0 {
        flags |= pso::ShaderStageFlags::GEOMETRY;
    }
    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_FRAGMENT_BIT as u32 != 0 {
        flags |= pso::ShaderStageFlags::FRAGMENT;
    }
    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_COMPUTE_BIT as u32 != 0 {
        flags |= pso::ShaderStageFlags::COMPUTE;
    }

    flags
}

pub fn map_pipeline_stage_flags(stages: VkPipelineStageFlags) -> pso::PipelineStage {
    let max_flag = VkPipelineStageFlagBits::VK_PIPELINE_STAGE_HOST_BIT as u32;

    if (stages & !((max_flag << 1) - 1)) == 0 {
        // HAL flags have the same numeric representation as Vulkan flags
        unsafe { mem::transmute(stages) }
    } else {
        // GRAPHICS and ALL are missing
        warn!("Unsupported pipeline stage flags: {:?}", stages);
        pso::PipelineStage::all()
    }
}

pub fn map_dependency_flags(dependencies: VkDependencyFlags) -> memory::Dependencies {
    let max_flag = VkDependencyFlagBits::VK_DEPENDENCY_BY_REGION_BIT as u32;

    if (dependencies & !((max_flag << 1) - 1)) == 0 {
        // HAL flags have the same numeric representation as Vulkan flags
        unsafe { mem::transmute(dependencies) }
    } else {
        // VIEW_LOCAL and DEVICE_GROUP are missing
        warn!("Unsupported dependency flags: {:?}", dependencies);
        memory::Dependencies::all()
    }
}

pub fn map_err_device_creation(err: device::CreationError) -> VkResult {
    use crate::hal::device::OutOfMemory::{Device, Host};
    match err {
        device::CreationError::OutOfMemory(Host) => VkResult::VK_ERROR_OUT_OF_HOST_MEMORY,
        device::CreationError::OutOfMemory(Device) => VkResult::VK_ERROR_OUT_OF_DEVICE_MEMORY,
        device::CreationError::InitializationFailed => VkResult::VK_ERROR_INITIALIZATION_FAILED,
        device::CreationError::MissingExtension => VkResult::VK_ERROR_EXTENSION_NOT_PRESENT,
        device::CreationError::MissingFeature => VkResult::VK_ERROR_FEATURE_NOT_PRESENT,
        device::CreationError::TooManyObjects => VkResult::VK_ERROR_TOO_MANY_OBJECTS,
        device::CreationError::DeviceLost => VkResult::VK_ERROR_DEVICE_LOST,
    }
}

pub fn map_attachment_load_op(op: VkAttachmentLoadOp) -> pass::AttachmentLoadOp {
    match op {
        VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_LOAD => pass::AttachmentLoadOp::Load,
        VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR => pass::AttachmentLoadOp::Clear,
        VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_DONT_CARE => pass::AttachmentLoadOp::DontCare,
        _ => panic!("Unsupported attachment load op: {:?}", op),
    }
}

pub fn map_attachment_store_op(op: VkAttachmentStoreOp) -> pass::AttachmentStoreOp {
    match op {
        VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE => pass::AttachmentStoreOp::Store,
        VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE => pass::AttachmentStoreOp::DontCare,
        _ => panic!("Unsupported attachment store op: {:?}", op),
    }
}

pub fn map_subpass_contents(contents: VkSubpassContents) -> command::SubpassContents {
    match contents {
        VkSubpassContents::VK_SUBPASS_CONTENTS_INLINE => command::SubpassContents::Inline,
        VkSubpassContents::VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS => {
            command::SubpassContents::SecondaryBuffers
        }

        _ => panic!("Unexpected subpass contents: {:?}", contents),
    }
}

pub fn map_stencil_face(face: VkStencilFaceFlags) -> pso::Face {
    match unsafe { mem::transmute(face) } {
        VkStencilFaceFlagBits::VK_STENCIL_FACE_FRONT_BIT => pso::Face::FRONT,
        VkStencilFaceFlagBits::VK_STENCIL_FACE_BACK_BIT => pso::Face::BACK,
        VkStencilFaceFlagBits::VK_STENCIL_FRONT_AND_BACK => pso::Face::all(),
        _ => panic!("Unexpected stencil face: {:?}", face),
    }
}

pub fn map_cull_face(cull: VkCullModeFlags) -> pso::Face {
    match unsafe { mem::transmute(cull) } {
        VkCullModeFlagBits::VK_CULL_MODE_NONE => pso::Face::empty(),
        VkCullModeFlagBits::VK_CULL_MODE_FRONT_BIT => pso::Face::FRONT,
        VkCullModeFlagBits::VK_CULL_MODE_BACK_BIT => pso::Face::BACK,
        VkCullModeFlagBits::VK_CULL_MODE_FRONT_AND_BACK => pso::Face::all(),
        _ => panic!("Unexpected cull face: {:?}", cull),
    }
}

pub fn map_front_face(face: VkFrontFace) -> pso::FrontFace {
    match face {
        VkFrontFace::VK_FRONT_FACE_COUNTER_CLOCKWISE => pso::FrontFace::CounterClockwise,
        VkFrontFace::VK_FRONT_FACE_CLOCKWISE => pso::FrontFace::Clockwise,
        _ => panic!("Unexpected front face: {:?}", face),
    }
}

pub fn map_primitive_topology(
    topology: VkPrimitiveTopology,
    patch_size: PatchSize,
) -> Option<(Primitive, bool)> {
    use super::VkPrimitiveTopology::*;

    Some(match topology {
        VK_PRIMITIVE_TOPOLOGY_POINT_LIST => (Primitive::PointList, false),
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST => (Primitive::LineList, false),
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP => (Primitive::LineStrip, false),
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST => (Primitive::TriangleList, false),
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP => (Primitive::TriangleStrip, false),
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN => return None,
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY => (Primitive::LineList, true),
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY => (Primitive::LineStrip, true),
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY => (Primitive::TriangleList, true),
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY => (Primitive::TriangleStrip, true),
        VK_PRIMITIVE_TOPOLOGY_PATCH_LIST => (Primitive::PatchList(patch_size), false),
        _ => return None,
    })
}

#[inline]
pub fn map_present_mode(present_mode: VkPresentModeKHR) -> window::PresentMode {
    // Vulkan and HAL values are equal
    unsafe { mem::transmute(present_mode) }
}

pub fn map_composite_alpha(
    composite_alpha: VkCompositeAlphaFlagBitsKHR,
) -> window::CompositeAlphaMode {
    if composite_alpha == VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR {
        window::CompositeAlphaMode::OPAQUE
    } else if composite_alpha
        == VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR
    {
        window::CompositeAlphaMode::PREMULTIPLIED
    } else if composite_alpha
        == VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR
    {
        window::CompositeAlphaMode::POSTMULTIPLIED
    } else if composite_alpha == VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR {
        window::CompositeAlphaMode::INHERIT
    } else {
        error!("Unrecognized composite alpha: {:?}", composite_alpha);
        window::CompositeAlphaMode::OPAQUE
    }
}

#[inline]
pub fn map_compare_op(op: VkCompareOp) -> pso::Comparison {
    // Vulkan and HAL values are equal
    unsafe { mem::transmute(op as u8) }
}

#[inline]
pub fn map_logic_op(op: VkLogicOp) -> pso::LogicOp {
    // Vulkan and HAL values are equal
    unsafe { mem::transmute(op as u8) }
}

#[inline]
pub fn map_stencil_op(op: VkStencilOp) -> pso::StencilOp {
    // Vulkan and HAL values are equal
    unsafe { mem::transmute(op as u8) }
}

#[inline]
pub fn map_color_components(mask: VkColorComponentFlags) -> pso::ColorMask {
    // Vulkan and HAL flags are equal
    unsafe { mem::transmute(mask as u8) }
}

#[inline]
fn map_blend_factor(factor: VkBlendFactor) -> pso::Factor {
    // Vulkan and HAL values are equal
    unsafe { mem::transmute(factor as u8) }
}

pub fn map_blend_op(
    blend_op: VkBlendOp,
    src_factor: VkBlendFactor,
    dst_factor: VkBlendFactor,
) -> pso::BlendOp {
    use super::VkBlendOp::*;
    match blend_op {
        VK_BLEND_OP_ADD => pso::BlendOp::Add {
            src: map_blend_factor(src_factor),
            dst: map_blend_factor(dst_factor),
        },
        VK_BLEND_OP_SUBTRACT => pso::BlendOp::Sub {
            src: map_blend_factor(src_factor),
            dst: map_blend_factor(dst_factor),
        },
        VK_BLEND_OP_REVERSE_SUBTRACT => pso::BlendOp::RevSub {
            src: map_blend_factor(src_factor),
            dst: map_blend_factor(dst_factor),
        },
        VK_BLEND_OP_MIN => pso::BlendOp::Min,
        VK_BLEND_OP_MAX => pso::BlendOp::Max,
        _ => panic!("Unexpected blend operation: {:?}", blend_op),
    }
}

#[inline]
pub fn map_cmd_buffer_usage(flags: VkCommandBufferUsageFlags) -> command::CommandBufferFlags {
    // Vulkan and HAL flags are equal
    unsafe { mem::transmute(flags) }
}

pub fn map_filter(filter: VkFilter) -> image::Filter {
    match filter {
        VkFilter::VK_FILTER_NEAREST => image::Filter::Nearest,
        VkFilter::VK_FILTER_LINEAR => image::Filter::Linear,
        _ => panic!("Unsupported filter {:?}", filter),
    }
}

pub fn map_mipmap_filter(mode: VkSamplerMipmapMode) -> image::Filter {
    match mode {
        VkSamplerMipmapMode::VK_SAMPLER_MIPMAP_MODE_NEAREST => image::Filter::Nearest,
        VkSamplerMipmapMode::VK_SAMPLER_MIPMAP_MODE_LINEAR => image::Filter::Linear,
        _ => panic!("Unsupported mipmap mode {:?}", mode),
    }
}

pub fn map_wrap_mode(mode: VkSamplerAddressMode) -> image::WrapMode {
    use super::VkSamplerAddressMode::*;
    match mode {
        VK_SAMPLER_ADDRESS_MODE_REPEAT => image::WrapMode::Tile,
        VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT => image::WrapMode::Mirror,
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE => image::WrapMode::Clamp,
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER => image::WrapMode::Border,
        _ => {
            warn!("Non-covered sampler address mode: {:?}", mode);
            image::WrapMode::Clamp
        }
    }
}

pub fn map_offset(extent: VkOffset3D) -> image::Offset {
    image::Offset {
        x: extent.x,
        y: extent.y,
        z: extent.z,
    }
}

pub fn map_extent(extent: VkExtent3D) -> image::Extent {
    image::Extent {
        width: extent.width,
        height: extent.height,
        depth: extent.depth,
    }
}

pub fn map_rect(rect: &VkRect2D) -> pso::Rect {
    pso::Rect {
        x: rect.offset.x as _,
        y: rect.offset.y as _,
        w: rect.extent.width as _,
        h: rect.extent.height as _,
    }
}

pub fn map_clear_rect(rect: &VkClearRect) -> pso::ClearRect {
    let base = rect.baseArrayLayer as image::Layer;
    pso::ClearRect {
        rect: map_rect(&rect.rect),
        layers: base..base + rect.layerCount as image::Layer,
    }
}

pub fn map_viewport(vp: &VkViewport) -> pso::Viewport {
    pso::Viewport {
        rect: pso::Rect {
            x: vp.x as _,
            y: vp.y as _,
            w: vp.width as _,
            h: vp.height as _,
        },
        depth: vp.minDepth..vp.maxDepth,
    }
}

pub fn map_tiling(tiling: VkImageTiling) -> image::Tiling {
    match tiling {
        VkImageTiling::VK_IMAGE_TILING_OPTIMAL => image::Tiling::Optimal,
        VkImageTiling::VK_IMAGE_TILING_LINEAR => image::Tiling::Linear,
        _ => panic!("Unexpected tiling: {:?}", tiling),
    }
}

pub fn map_index_type(ty: VkIndexType) -> IndexType {
    match ty {
        VkIndexType::VK_INDEX_TYPE_UINT16 => IndexType::U16,
        VkIndexType::VK_INDEX_TYPE_UINT32 => IndexType::U32,
        _ => panic!("Unexpected index type: {:?}", ty),
    }
}

#[inline]
pub fn map_query_type(ty: VkQueryType, statistic: VkQueryPipelineStatisticFlags) -> query::Type {
    match ty {
        VkQueryType::VK_QUERY_TYPE_OCCLUSION => query::Type::Occlusion,
        VkQueryType::VK_QUERY_TYPE_PIPELINE_STATISTICS => {
            query::Type::PipelineStatistics(map_pipeline_statistics(statistic))
        }
        VkQueryType::VK_QUERY_TYPE_TIMESTAMP => query::Type::Timestamp,
        _ => panic!("Unexpected query type: {:?}", ty),
    }
}

#[inline]
pub fn map_query_control(flags: VkQueryControlFlags) -> query::ControlFlags {
    // Vulkan and HAL flags are equal
    query::ControlFlags::from_bits_truncate(flags as u32)
}

#[inline]
pub fn map_query_result(flags: VkQueryResultFlags) -> query::ResultFlags {
    // Vulkan and HAL flags are equal
    query::ResultFlags::from_bits_truncate(flags as u32)
}

#[inline]
pub fn map_pipeline_statistics(flags: VkQueryPipelineStatisticFlags) -> query::PipelineStatistic {
    // Vulkan and HAL flags are equal
    query::PipelineStatistic::from_bits_truncate(flags as u32)
}
