use std::io::Cursor;

use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, AsyncReadExt, LoadContext};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_math::UVec2;
use bevy_reflect::prelude::*;
use bevy_render::{prelude::*, render_asset::RenderAssetUsages};
use bevy_sprite::{TextureAtlasBuilder, TextureAtlasBuilderError, TextureAtlasLayout};
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::graphics::sprite_sheet::{DecodeError, Decoder, SpriteSheet};

pub struct SpriteSheetAssetPlugin;

impl Plugin for SpriteSheetAssetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpriteSheetAssetLoaderSettings {
            use_fallback_image: true,
            ..Default::default()
        })
        .register_type::<SpriteSheetAssetLoaderSettings>()
        .init_asset::<SpriteSheetAsset>()
        .init_asset_loader::<SpriteSheetAssetLoader>()
        .register_asset_reflect::<SpriteSheetAsset>()
        .register_type::<SpriteSheetHandle>();
    }
}

#[derive(Asset, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct SpriteSheetAsset {
    source: SpriteSheet,
    pub texture: Handle<Image>,
    pub texture_atlas_layout: Handle<TextureAtlasLayout>,
    pub texture_descriptors: Vec<TextureDescriptor>,
}

impl SpriteSheetAsset {
    /// The number of sprites in the sprite sheet.
    #[inline(always)]
    pub fn sprite_count(&self) -> usize {
        self.source.textures.len()
    }

    /// Try to get a specific texture descriptor by index.
    pub fn try_get_texture_descriptor(
        &self,
        index: usize,
    ) -> Result<&TextureDescriptor, TextureDescriptorNotFoundError> {
        self.texture_descriptors
            .get(index)
            .ok_or(TextureDescriptorNotFoundError { index })
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct TextureDescriptor {
    d: crate::graphics::TextureDescriptor,
    pub width: u32,
    pub height: u32,
    pub x: i16,
    pub y: i16,
}

#[derive(Debug)]
pub struct TextureDescriptorNotFoundError {
    index: usize,
}

impl std::fmt::Display for TextureDescriptorNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "texture descriptor not found at index {}", self.index)
    }
}

impl std::error::Error for TextureDescriptorNotFoundError {}

/// A [`Handle`] to a [`SpriteSheetAsset`] asset.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Eq, PartialEq, Reflect)]
#[reflect(Component, Default)]
pub struct SpriteSheetHandle(pub Handle<SpriteSheetAsset>);

impl From<Handle<SpriteSheetAsset>> for SpriteSheetHandle {
    fn from(handle: Handle<SpriteSheetAsset>) -> Self {
        Self(handle)
    }
}

impl From<SpriteSheetHandle> for AssetId<SpriteSheetAsset> {
    fn from(handle: SpriteSheetHandle) -> Self {
        handle.id()
    }
}

impl From<&SpriteSheetHandle> for AssetId<SpriteSheetAsset> {
    fn from(handle: &SpriteSheetHandle) -> Self {
        handle.id()
    }
}

#[derive(Clone)]
pub struct SpriteSheetAssetLoader {
    default_settings: SpriteSheetAssetLoaderSettings,
}

#[derive(Clone, Default, Deserialize, Reflect, Resource, Serialize)]
#[reflect(Resource)]
pub struct SpriteSheetAssetLoaderSettings {
    #[serde(default)]
    pub use_fallback_image: bool,
    #[serde(default)]
    pub padding: Option<(u16, u16)>,
}

/// Possible errors that can be produced by [`SpriteSheetAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SpriteSheetAssetLoaderError {
    /// An IO error.
    #[error("could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// An error caused when the asset path cannot be determined.
    #[error("could not determine file path of asset")]
    IndeterminateFilePath,
    /// A sprite sheet decoding error.
    #[error("could not decode sprite sheet: {0}")]
    DecodeError(#[from] DecodeError),
    /// A texture atlas builder error.
    #[error("could not build texture atlas: {0}")]
    TextureAtlasBuilderError(TextureAtlasBuilderError),
}

impl From<TextureAtlasBuilderError> for SpriteSheetAssetLoaderError {
    fn from(error: TextureAtlasBuilderError) -> Self {
        SpriteSheetAssetLoaderError::TextureAtlasBuilderError(error)
    }
}

impl AssetLoader for SpriteSheetAssetLoader {
    type Asset = SpriteSheetAsset;
    type Settings = SpriteSheetAssetLoaderSettings;
    type Error = SpriteSheetAssetLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a SpriteSheetAssetLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let sprite_sheet = decoder.decode()?;

        let use_fallback_image = if settings.use_fallback_image {
            true
        } else {
            self.default_settings.use_fallback_image
        };

        let padding = if settings.padding.is_none() {
            self.default_settings.padding
        } else {
            settings.padding
        };

        let textures = sprite_sheet
            .textures
            .iter()
            .enumerate()
            .map(|(i, texture)| {
                // If the sprite sheet texture has no dimensions, check if we
                // should use a fallback image. Some sprite sheet textures seem
                // to be placeholders with no actual image data, so this is not
                // an uncommon scenario.
                let x = if use_fallback_image && texture.dimensions() == (0, 0) {
                    Image::from_dynamic(
                        image::DynamicImage::new_rgba8(1, 1),
                        true,
                        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                    )
                } else {
                    Image::from_dynamic(
                        texture.clone(),
                        true,
                        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                    )
                };
                (
                    load_context
                        .labeled_asset_scope(format!("{}_texture", i).to_string(), |_| x.clone()),
                    x,
                )
            })
            .collect::<Vec<_>>();

        let mut texture_atlas_builder = TextureAtlasBuilder::default();
        texture_atlas_builder.padding(if let Some(padding) = padding {
            UVec2::new(padding.0 as u32, padding.1 as u32)
        } else {
            UVec2::default()
        });

        for (handle, texture) in textures.iter() {
            texture_atlas_builder.add_texture(Some(handle.id()), texture);
        }

        let (texture_atlas_layout, texture) = texture_atlas_builder.build()?;

        let texture_atlas_layout = load_context
            .labeled_asset_scope("texture_atlas_layout".to_string(), |_| texture_atlas_layout);
        let texture = load_context.labeled_asset_scope("texture".to_string(), |_| texture);

        Ok(SpriteSheetAsset {
            source: sprite_sheet.clone(),
            texture,
            texture_atlas_layout,
            texture_descriptors: sprite_sheet
                .texture_descriptors
                .iter()
                .map(|d| TextureDescriptor {
                    d: d.clone(),
                    width: d.width as u32,
                    height: d.height as u32,
                    x: d.x,
                    y: d.y,
                })
                .collect(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["SPR", "spr"]
    }
}

impl FromWorld for SpriteSheetAssetLoader {
    fn from_world(world: &mut World) -> Self {
        let settings = world.resource::<SpriteSheetAssetLoaderSettings>();

        Self {
            default_settings: settings.clone(),
        }
    }
}
