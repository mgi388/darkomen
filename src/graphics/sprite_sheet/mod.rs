mod decoder;
mod packbits;
mod zeroruns;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use image::DynamicImage;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

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

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum FrameType {
    /// Indicates the frame is a repeat of a previous frame.
    Repeat = 0,
    /// Indicates the frame should be flipped along the x axis.
    FlipX = 1,
    /// Indicates the frame should be flipped along the y axis.
    FlipY = 2,
    /// Indicates the frame should be flipped along the x and y axes.
    FlipXY = 3,
    /// Indicates a normal frame.
    #[default]
    Normal = 4,
    /// Indicates the frame is empty. There is no frame or palette data
    /// associated with the frame. The frame's width and height are 0.
    Empty = 5,
}
