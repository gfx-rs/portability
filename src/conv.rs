use {VkExtent2D, VkFormat};

use hal::format;
use hal::window;

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

pub fn hal_from_format(format: VkFormat) -> format::Format {
    use VkFormat::*;
    use hal::format::ChannelType::*;
    use hal::format::SurfaceType::*;

    let (sf, cf) = match format {
        VK_FORMAT_B8G8R8A8_UNORM => (B8_G8_R8_A8, Unorm),
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
