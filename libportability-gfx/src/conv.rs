use super::*;
use hal::{self, format, image, memory, window};

pub fn format_from_hal(format: format::Format) -> VkFormat {
    use VkFormat::*;
    use hal::format::ChannelType::*;
    use hal::format::SurfaceType::*;

    match format.0 {
        R5_G6_B5 => match format.1 {
            Unorm => VK_FORMAT_R5G6B5_UNORM_PACK16,
            _ => unreachable!(),
        },
        R4_G4_B4_A4 => match format.1 {
            Unorm => VK_FORMAT_R4G4B4A4_UNORM_PACK16,
            _ => unreachable!(),
        },
        R8_G8_B8_A8 => match format.1 {
            Unorm => VK_FORMAT_R8G8B8A8_UNORM,
            Inorm => VK_FORMAT_R8G8B8A8_SNORM,
            Srgb => VK_FORMAT_R8G8B8A8_SRGB,
            _ => panic!("format {:?}", format),
        },
        B8_G8_R8_A8 => match format.1 {
            Unorm => VK_FORMAT_B8G8R8A8_UNORM,
            Inorm => VK_FORMAT_B8G8R8A8_SNORM,
            Srgb => VK_FORMAT_B8G8R8A8_SRGB,
            _ => panic!("format {:?}", format),
        },
        R16_G16_B16_A16 => match format.1 {
            Unorm => VK_FORMAT_R16G16B16A16_UNORM,
            Inorm => VK_FORMAT_R16G16B16A16_SNORM,
            Float => VK_FORMAT_R16G16B16A16_SFLOAT,
            _ => panic!("format {:?}", format),
        },
        _ => {
            panic!("format {:?}", format);
        }
    }
}

pub fn format_properties_from_hal(properties: format::Properties) -> VkFormatProperties {
    VkFormatProperties {
        linearTilingFeatures: image_features_from_hal(properties.linear_tiling),
        optimalTilingFeatures: image_features_from_hal(properties.optimal_tiling),
        bufferFeatures: buffer_features_from_hal(properties.buffer_features),
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

pub fn map_format(format: VkFormat) -> format::Format {
    use VkFormat::*;
    use hal::format::ChannelType::*;
    use hal::format::SurfaceType::*;

    let (sf, cf) = match format {
        VK_FORMAT_B8G8R8A8_UNORM => (B8_G8_R8_A8, Unorm),
        VK_FORMAT_D16_UNORM => (D16, Unorm),
        _ => {
            panic!("format {:?}", format);
        }
    };

    format::Format(sf, cf)
}

pub fn extent2d_from_hal(extent: window::Extent2d) -> VkExtent2D {
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

pub fn map_subresource_range(subresource: VkImageSubresourceRange) -> image::SubresourceRange {
    image::SubresourceRange {
        aspects: map_aspect(subresource.aspectMask),
        levels: subresource.baseMipLevel as _ .. (subresource.baseMipLevel+subresource.levelCount) as _,
        layers: subresource.baseArrayLayer as _ .. (subresource.baseArrayLayer+subresource.layerCount) as _,
    }
}

fn map_aspect(aspects: VkImageAspectFlags) -> image::AspectFlags {
    let mut flags = image::AspectFlags::empty();
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_COLOR_BIT as u32 != 0 {
        flags |= image::AspectFlags::COLOR;
    }
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_DEPTH_BIT as u32 != 0 {
        flags |= image::AspectFlags::DEPTH;
    }
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_STENCIL_BIT as u32 != 0 {
        flags |= image::AspectFlags::DEPTH;
    }
    if aspects & VkImageAspectFlagBits::VK_IMAGE_ASPECT_METADATA_BIT as u32 != 0 {
        unimplemented!()
    }
    flags
}

pub fn map_image_kind(
    ty: VkImageType,
    flags: VkImageCreateFlags,
    extent: VkExtent3D,
    array_layers: u32,
    samples: VkSampleCountFlagBits,
) -> image::Kind {
    debug_assert_ne!(array_layers, 0);
    let is_cube = flags & VkImageCreateFlagBits::VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT as u32 != 0;
    assert!(!is_cube || array_layers % 6 == 0);

    match ty {
        VkImageType::VK_IMAGE_TYPE_1D => {
            image::Kind::D1(extent.width as _)
        }
        VkImageType::VK_IMAGE_TYPE_1D => {
            image::Kind::D1Array(extent.width as _, array_layers as _)
        }
        VkImageType::VK_IMAGE_TYPE_2D if array_layers == 1 => {
            image::Kind::D2(extent.width as _, extent.height as _, map_aa_mode(samples))
        }
        VkImageType::VK_IMAGE_TYPE_2D if is_cube && array_layers == 6 => {
            image::Kind::Cube(extent.width as _)
        }
        VkImageType::VK_IMAGE_TYPE_2D if is_cube => {
            image::Kind::CubeArray(extent.width as _, (array_layers / 6) as _)
        }
        VkImageType::VK_IMAGE_TYPE_2D => {
            image::Kind::D2Array(
                extent.width as _,
                extent.height as _,
                array_layers as _,
                map_aa_mode(samples),
            )
        }
        VkImageType::VK_IMAGE_TYPE_3D => {
            image::Kind::D3(extent.width as _, extent.height as _, extent.depth as _)
        }
        _ => unimplemented!(),
    }
}

fn map_aa_mode(samples: VkSampleCountFlagBits) -> image::AaMode {
    use VkSampleCountFlagBits::*;

    match samples {
        VK_SAMPLE_COUNT_1_BIT => image::AaMode::Single,
        _ => image::AaMode::Multi(samples as _),
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
