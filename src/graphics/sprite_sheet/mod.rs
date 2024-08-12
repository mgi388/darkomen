mod decoder;
mod packbits;
mod zeroruns;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use image::DynamicImage;
use serde::Serialize;

pub use decoder::{DecodeError, Decoder};
pub(crate) use packbits::PackBitsReader;
pub(crate) use zeroruns::ZeroRunsReader;

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct SpriteSheet {
    #[serde(skip)]
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub textures: Vec<DynamicImage>,
    pub texture_descriptors: Vec<TextureDescriptor>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct TextureDescriptor {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}
