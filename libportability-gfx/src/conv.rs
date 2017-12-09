use super::*;
use hal::{format, image, window};

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
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT as _;
    }
    if features.contains(format::ImageFeature::STORAGE) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT as _;
    }
    if features.contains(format::ImageFeature::STORAGE_ATOMIC) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_IMAGE_ATOMIC_BIT as _;
    }
    if features.contains(format::ImageFeature::COLOR_ATTACHMENT) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT as _;
    }
    if features.contains(format::ImageFeature::COLOR_ATTACHMENT_BLEND) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT as _;
    }
    if features.contains(format::ImageFeature::DEPTH_STENCIL_ATTACHMENT) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT as _;
    }
    if features.contains(format::ImageFeature::BLIT_SRC) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_BLIT_SRC_BIT as _;
    }
    if features.contains(format::ImageFeature::BLIT_DST) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_BLIT_DST_BIT as _;
    }
    if features.contains(format::ImageFeature::SAMPLED_LINEAR) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT as _;
    }

    flags
}

fn buffer_features_from_hal(features: format::BufferFeature) -> VkFormatFeatureFlags {
    let mut flags = 0;

    if features.contains(format::BufferFeature::UNIFORM_TEXEL) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT as _;
    }
    if features.contains(format::BufferFeature::STORAGE_TEXEL) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_BIT as _;
    }
    if features.contains(format::BufferFeature::STORAGE_TEXEL_ATOMIC) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_ATOMIC_BIT as _;
    }
    if features.contains(format::BufferFeature::VERTEX) {
        flags |= VkFormatFeatureFlagBits::VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT as _;
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
