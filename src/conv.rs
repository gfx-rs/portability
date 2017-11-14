use {VkExtent2D, VkFormat};

use hal::format;
use hal::window;

pub fn format_from_hal(format: format::Format) -> VkFormat {
    use VkFormat::*;
    use hal::format::ChannelType::*;
    use hal::format::SurfaceType::*;

    match format.0 {
        R8_G8_B8_A8 => match format.1 {
            Unorm => VK_FORMAT_R8G8B8A8_UNORM,
            Srgb => VK_FORMAT_R8G8B8A8_SRGB,
            _ => unimplemented!()
        },
        B8_G8_R8_A8 => match format.1 {
            Unorm => VK_FORMAT_B8G8R8A8_UNORM,
            Srgb => VK_FORMAT_B8G8R8A8_SRGB,
            _ => unimplemented!()
        },
        _ => {
            println!("\tformat {:?}", format);
            unimplemented!()
        }
    }
}

pub fn extent2d_from_hal(extent: window::Extent2d) -> VkExtent2D {
    VkExtent2D {
        width: extent.width,
        height: extent.height,
    }
}
