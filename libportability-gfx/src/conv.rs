use hal::{buffer, command, error, format, image, memory, pass, pso, window};
use hal::{PatchSize, Primitive};

use std::mem;

use super::*;

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

pub fn image_format_properties_from_hal(_properties: format::Properties) -> VkImageFormatProperties {
    //TODO
    VkImageFormatProperties {
        maxExtent: VkExtent3D {
            width: 1<<12,
            height: 1<<12,
            depth: 1<<8,
        },
        maxMipLevels: 1,
        maxArrayLayers: 1,
        sampleCounts: 1,
        maxResourceSize: 1<<24,
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
    use VkComponentSwizzle::*;

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

pub fn map_subresource_layers(subresource: VkImageSubresourceLayers) -> image::SubresourceLayers {
    image::SubresourceLayers {
        aspects: map_aspect(subresource.aspectMask),
        level: subresource.mipLevel as _,
        layers: subresource.baseArrayLayer as _
            ..(subresource.baseArrayLayer + subresource.layerCount) as _,
    }
}

pub fn map_subresource_range(subresource: VkImageSubresourceRange) -> image::SubresourceRange {
    image::SubresourceRange {
        aspects: map_aspect(subresource.aspectMask),
        levels: subresource.baseMipLevel as _
            ..(subresource.baseMipLevel + subresource.levelCount) as _,
        layers: subresource.baseArrayLayer as _
            ..(subresource.baseArrayLayer + subresource.layerCount) as _,
    }
}

fn map_aspect(aspects: VkImageAspectFlags) -> format::Aspects {
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

pub fn map_image_kind(
    ty: VkImageType,
    extent: VkExtent3D,
    array_layers: image::Layer,
    samples: VkSampleCountFlagBits,
) -> image::Kind {
    debug_assert_ne!(array_layers, 0);
    match ty {
        VkImageType::VK_IMAGE_TYPE_1D => image::Kind::D1(extent.width as _, array_layers),
        VkImageType::VK_IMAGE_TYPE_2D => image::Kind::D2(extent.width as _, extent.height as _, array_layers, samples as _),
        VkImageType::VK_IMAGE_TYPE_3D => image::Kind::D3(extent.width as _, extent.height as _, extent.depth as _),
        _ => unreachable!()
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
        _ => unreachable!()
    }
}

pub fn map_image_layout(layout: VkImageLayout) -> image::ImageLayout {
    use hal::image::ImageLayout::*;
    match layout {
        VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED => Undefined,
        VkImageLayout::VK_IMAGE_LAYOUT_GENERAL => General,
        VkImageLayout::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL => ColorAttachmentOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL => DepthStencilAttachmentOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL => DepthStencilReadOnlyOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL => ShaderReadOnlyOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL => TransferSrcOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL => TransferDstOptimal,
        VkImageLayout::VK_IMAGE_LAYOUT_PREINITIALIZED => Preinitialized,
        VkImageLayout::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR => Present,
        _ => panic!("Unexpected image layout: {:?}", layout),
    }
}

pub fn map_image_usage(usage: VkImageUsageFlags) -> image::Usage {
    let mut flags = image::Usage::empty();

    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_TRANSFER_SRC_BIT as u32 != 0 {
        flags |= image::Usage::TRANSFER_SRC;
    }
    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_TRANSFER_DST_BIT as u32 != 0 {
        flags |= image::Usage::TRANSFER_DST;
    }
    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_SAMPLED_BIT as u32 != 0 {
        flags |= image::Usage::SAMPLED;
    }
    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_STORAGE_BIT as u32 != 0 {
        flags |= image::Usage::STORAGE;
    }
    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as u32 != 0 {
        flags |= image::Usage::COLOR_ATTACHMENT;
    }
    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT as u32 != 0 {
        flags |= image::Usage::DEPTH_STENCIL_ATTACHMENT;
    }
    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_TRANSIENT_ATTACHMENT_BIT as u32 != 0 {
        unimplemented!()
    }
    if usage & VkImageUsageFlagBits::VK_IMAGE_USAGE_INPUT_ATTACHMENT_BIT as u32 != 0 {
        unimplemented!()
    }

    flags
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
        mask |= buffer::Access::CONSTANT_BUFFER_READ;
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

    match ty {
        VK_DESCRIPTOR_TYPE_SAMPLER => pso::DescriptorType::Sampler,
        VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE => pso::DescriptorType::SampledImage,
        VK_DESCRIPTOR_TYPE_STORAGE_IMAGE => pso::DescriptorType::StorageImage,
        VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER => pso::DescriptorType::UniformTexelBuffer,
        VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER => pso::DescriptorType::StorageTexelBuffer,
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER => pso::DescriptorType::UniformBuffer,
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER => pso::DescriptorType::StorageBuffer,
        VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT => pso::DescriptorType::InputAttachment,

        VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER |
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC |
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC => unimplemented!(),
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
    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_ALL_GRAPHICS as u32 != 0 {
        flags |= pso::ShaderStageFlags::GRAPHICS;
    }
    if stages & VkShaderStageFlagBits::VK_SHADER_STAGE_ALL as u32 != 0 {
        flags |= pso::ShaderStageFlags::ALL;
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

pub fn map_err_device_creation(err: error::DeviceCreationError) -> VkResult {
    use hal::error::DeviceCreationError::*;

    match err {
        OutOfHostMemory => VkResult::VK_ERROR_OUT_OF_HOST_MEMORY,
        OutOfDeviceMemory => VkResult::VK_ERROR_OUT_OF_DEVICE_MEMORY,
        InitializationFailed => VkResult::VK_ERROR_INITIALIZATION_FAILED,
        MissingExtension => VkResult::VK_ERROR_EXTENSION_NOT_PRESENT,
        MissingFeature => VkResult::VK_ERROR_FEATURE_NOT_PRESENT,
        TooManyObjects => VkResult::VK_ERROR_TOO_MANY_OBJECTS,
        DeviceLost => VkResult::VK_ERROR_DEVICE_LOST,
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
        VkSubpassContents::VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS =>
            command::SubpassContents::SecondaryBuffers,

        _ => panic!("Unexpected subpass contents: {:?}", contents),
    }
}

pub fn map_polygon_mode(mode: VkPolygonMode, line_width: f32) -> pso::PolygonMode {
    match mode {
        VkPolygonMode::VK_POLYGON_MODE_FILL => pso::PolygonMode::Fill,
        VkPolygonMode::VK_POLYGON_MODE_LINE => pso::PolygonMode::Line(line_width),
        VkPolygonMode::VK_POLYGON_MODE_POINT => pso::PolygonMode::Point,
        _ => panic!("Unexpected polygon mode: {:?}", mode),
    }
}

pub fn map_cull_face(cull: VkCullModeFlags) -> Option<pso::CullFace> {
    use super::VkCullModeFlagBits::*;

    if cull == VK_CULL_MODE_NONE as _ { None }
    else if cull == VK_CULL_MODE_FRONT_BIT as _ { Some(pso::CullFace::Front) }
    else if cull == VK_CULL_MODE_BACK_BIT as _ { Some(pso::CullFace::Back) }
    else if cull == VK_CULL_MODE_FRONT_AND_BACK as _ { unimplemented!() } // TODO: can we support it?
    else { panic!("Unexpected cull mode: {:?}", cull) }
}

pub fn map_front_face(face: VkFrontFace) -> pso::FrontFace {
    match face {
        VkFrontFace::VK_FRONT_FACE_COUNTER_CLOCKWISE => pso::FrontFace::CounterClockwise,
        VkFrontFace::VK_FRONT_FACE_CLOCKWISE => pso::FrontFace::Clockwise,
        _ => panic!("Unexpected front face: {:?}", face),
    }
}

pub fn map_primitive_topology(topology: VkPrimitiveTopology, patch_size: PatchSize) -> hal::Primitive {
    use super::VkPrimitiveTopology::*;

    match topology {
        VK_PRIMITIVE_TOPOLOGY_POINT_LIST => Primitive::PointList,
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST => Primitive::LineList,
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP => Primitive::LineStrip,
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST => Primitive::TriangleList,
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP => Primitive::TriangleStrip,
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN =>
            panic!("`VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN` not supported in portability"),
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY => Primitive::LineListAdjacency,
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY => Primitive::LineStripAdjacency,
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY => Primitive::TriangleListAdjacency,
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY => Primitive::TriangleStripAdjacency,
        VK_PRIMITIVE_TOPOLOGY_PATCH_LIST => Primitive::PatchList(patch_size),
        _ => panic!("Unexpected primitive topology: {:?}", topology),
    }
}

pub fn map_compare_op(op: VkCompareOp) -> pso::Comparison {
    use super::VkCompareOp::*;

    match op {
        VK_COMPARE_OP_NEVER => pso::Comparison::Never,
        VK_COMPARE_OP_LESS => pso::Comparison::Less,
        VK_COMPARE_OP_EQUAL => pso::Comparison::Equal,
        VK_COMPARE_OP_LESS_OR_EQUAL => pso::Comparison::LessEqual,
        VK_COMPARE_OP_GREATER => pso::Comparison::Greater,
        VK_COMPARE_OP_NOT_EQUAL => pso::Comparison::NotEqual,
        VK_COMPARE_OP_GREATER_OR_EQUAL => pso::Comparison::GreaterEqual,
        VK_COMPARE_OP_ALWAYS => pso::Comparison::Always,
        _ => panic!("Unexpected compare op: {:?}", op),
    }
}

pub fn map_logic_op(op: VkLogicOp) -> pso::LogicOp {
    unimplemented!()
}

pub fn map_stencil_op(op: VkStencilOp) -> pso::StencilOp {
    unimplemented!()
}

pub fn map_color_components(mask: VkColorComponentFlags) -> pso::ColorMask {
    // Vulkan and HAL flags are equal
    unsafe { mem::transmute(mask as u8) }
}

pub fn map_blend_op(
    blend_op: VkBlendOp, src_factor: VkBlendFactor, dst_factor: VkBlendFactor,
) -> pso::BlendOp {
    unimplemented!()
}

#[inline]
pub fn map_cmd_buffer_usage(flags: VkCommandBufferUsageFlags) -> command::CommandBufferFlags {
    // Vulkan and HAL flags are equal
    unsafe { mem::transmute(flags as u16) }
}

pub fn map_filter(filter: VkFilter) -> image::Filter {
    match filter {
        VkFilter::VK_FILTER_NEAREST => image::Filter::Nearest,
        VkFilter::VK_FILTER_LINEAR => image::Filter::Linear,
        _ => panic!("Unsupported filter {:?}", filter)
    }
}

pub fn map_mipmap_filter(mode: VkSamplerMipmapMode) -> image::Filter {
    match mode {
        VkSamplerMipmapMode::VK_SAMPLER_MIPMAP_MODE_NEAREST => image::Filter::Nearest,
        VkSamplerMipmapMode::VK_SAMPLER_MIPMAP_MODE_LINEAR => image::Filter::Linear,
        _ => panic!("Unsupported mipmap mode {:?}", mode)
    }
}

pub fn map_wrap_mode(mode: VkSamplerAddressMode) -> image::WrapMode {
    use super::VkSamplerAddressMode::*;
    match mode {
        VK_SAMPLER_ADDRESS_MODE_REPEAT => image::WrapMode::Tile,
        VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT => image::WrapMode::Mirror,
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE => image::WrapMode::Clamp,
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER => image::WrapMode::Border,
        other => {
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

pub fn map_viewport(vp: &VkViewport) -> pso::Viewport {
    pso::Viewport {
        rect: pso::Rect {
            x: vp.x as _,
            y: vp.y as _,
            w: vp.width as _,
            h: vp.height as _,
        },
        depth: vp.minDepth .. vp.maxDepth,
    }
}
