mod decoder;
mod packbits;
mod zeroruns;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use image::DynamicImage;

pub use decoder::{DecodeError, Decoder};
pub(crate) use packbits::PackBitsReader;
pub(crate) use zeroruns::ZeroRunsReader;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct SpriteSheet {
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub texture: DynamicImage,
    pub atlas_layout: AtlasLayout,
    pub frames: Vec<Frame>,
}

/// Provides information about how the sprite sheet is laid out.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct AtlasLayout {
    pub tile_size: (u16, u16),
    pub columns: usize,
    pub rows: usize,
    pub padding: Option<(u16, u16)>,
    pub offset: Option<(u16, u16)>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Frame {
    pub frame_type: FrameType,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

/// Provides information about how to interpret a frame image.
#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum FrameType {
    /// Indicates the frame is a repeat of a previous frame.
    FrameTypeRepeat,
    // Indicates the frame should be flipped horizontally.
    FrameTypeFlipHorizontally,
    // Indicates the frame should be flipped vertically.
    FrameTypeFlipVertically,
    // Indicates the frame should be flipped horizontally and vertically.
    FrameTypeFlipHorizontallyAndVertically,
    // Indicates a normal frame.
    FrameTypeNormal,
    // Indicates the frame is empty.
    // There is no frame or palette data associated with the frame.
    // The frame's width and height are 0.
    FrameTypeEmpty,
}

impl From<u8> for FrameType {
    fn from(value: u8) -> Self {
        match value {
            0 => FrameType::FrameTypeRepeat,
            1 => FrameType::FrameTypeFlipHorizontally,
            2 => FrameType::FrameTypeFlipVertically,
            3 => FrameType::FrameTypeFlipHorizontallyAndVertically,
            4 => FrameType::FrameTypeNormal,
            5 => FrameType::FrameTypeEmpty,
            _ => panic!("invalid frame type"),
        }
    }
}
