use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_image::Image;
use bevy_reflect::prelude::*;
use bevy_render::render_asset::RenderAssetUsages;
use derive_more::derive::{Display, Error, From};
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::shadow::{DecodeError, Decoder, Lightmap};

/// A plugin for loading lightmaps into a [`LightmapAsset`].
pub struct LightmapAssetPlugin;

impl Plugin for LightmapAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LightmapAsset>()
            .init_asset_loader::<LightmapAssetLoader>()
            .register_asset_reflect::<LightmapAsset>();
    }
}

/// An asset that represents a lightmap.
#[derive(Asset, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(not(feature = "bevy_reflect"), derive(TypePath))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct LightmapAsset {
    source: Lightmap,
    pub texture: Handle<Image>,
}

/// An asset loader for loading lightmaps into a [`LightmapAsset`].
#[derive(Clone, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct LightmapAssetLoader;

/// Settings for the [`LightmapAssetLoader`].
#[derive(Clone, Copy, Default, Deserialize, Reflect, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Default, Deserialize, Serialize)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct LightmapAssetLoaderSettings {
    /// Optional settings for transforming the lightmap image. If not provided,
    /// the default settings are used.
    pub transform_image_settings: Option<TransformLightmapImageSettings>,
}

/// The orientation of the image.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Hash, Deserialize, PartialEq, Serialize)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub enum Orientation {
    /// Rotate by 90 degrees clockwise.
    Rotate90,
    /// Rotate by 180 degrees. Can be performed in-place.
    Rotate180,
    /// Rotate by 270 degrees clockwise. Equivalent to rotating by 90 degrees
    /// counter-clockwise.
    Rotate270,
}

/// Settings for transforming the lightmap image.
#[derive(Clone, Copy, Deserialize, Reflect, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Default, Deserialize, Serialize)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct TransformLightmapImageSettings {
    /// The orientation of the image. If not provided, the image is not rotated.
    pub orientation: Option<Orientation>,
    /// The contrast factor to apply to the image. If not provided, the image is
    /// not modified.
    pub contrast_factor: Option<f32>,
    /// The spread factor to apply to the image. If not provided, the image is
    /// not modified.
    pub spread_factor: Option<f32>,
    /// The sigma value for the Gaussian blur filter. If not provided, the image
    /// is not blurred.
    pub gaussian_blur_sigma: Option<f32>,
}

impl Default for TransformLightmapImageSettings {
    fn default() -> Self {
        Self {
            orientation: Some(Orientation::Rotate270),
            contrast_factor: None,
            spread_factor: None,
            gaussian_blur_sigma: None,
        }
    }
}

/// Possible errors that can be produced by [`LightmapAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum LightmapAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// An error caused when the asset path cannot be determined.
    #[display("could not determine file path of asset")]
    IndeterminateFilePath,
    /// A [DecodeError] error.
    #[display("could not decode lightmap: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for LightmapAssetLoader {
    type Asset = LightmapAsset;
    type Settings = LightmapAssetLoaderSettings;
    type Error = LightmapAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = std::io::Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let lightmap = decoder.decode()?;

        let image = transform_image(
            settings.transform_image_settings.unwrap_or_default(),
            lightmap.image(),
        );

        let texture = load_context.labeled_asset_scope("texture".to_string(), |_| {
            Image::from_dynamic(
                image.clone(),
                true,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
        });

        Ok(LightmapAsset {
            source: lightmap,
            texture,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["SHD", "shd"]
    }
}

fn transform_image(settings: TransformLightmapImageSettings, img: DynamicImage) -> DynamicImage {
    let img = match settings.orientation {
        Some(Orientation::Rotate90) => {
            let img = img.rotate90(); // rotate 90 degrees clockwise
            DynamicImage::ImageRgba8(img.to_rgba8())
        }
        Some(Orientation::Rotate180) => {
            let img = img.rotate180(); // rotate 180 degrees
            DynamicImage::ImageRgba8(img.to_rgba8())
        }
        Some(Orientation::Rotate270) => {
            let img = img.rotate270(); // rotate 90 degrees counter-clockwise
            DynamicImage::ImageRgba8(img.to_rgba8())
        }
        None => img,
    };

    // Convert to RgbaImage for pixel manipulation.
    let mut rgba_img = img.to_rgba8();

    // Increase the contrast of the image.
    if let Some(contrast_factor) = settings.contrast_factor {
        for pixel in rgba_img.pixels_mut() {
            // Only iterate over the RGB channels.
            for channel in &mut pixel.0[0..3] {
                let new_value = 128.0 + contrast_factor * (*channel as f32 - 128.0);
                *channel = new_value.clamp(0.0, 255.0) as u8;
            }
        }
    }

    // "Spread" the shadows even further.
    if let Some(spread_factor) = settings.spread_factor {
        for y in 0..rgba_img.height() {
            for x in 0..rgba_img.width() {
                let pixel = rgba_img.get_pixel(x, y);
                let mut new_pixel = *pixel;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx < 0
                            || ny < 0
                            || nx >= rgba_img.width() as i32
                            || ny >= rgba_img.height() as i32
                        {
                            continue;
                        }

                        let neighbor = rgba_img.get_pixel(nx as u32, ny as u32);
                        for i in 0..3 {
                            let new_value = pixel.0[i] as f32
                                + spread_factor * (neighbor.0[i] as f32 - pixel.0[i] as f32);
                            new_pixel.0[i] = new_value.clamp(0.0, 255.0) as u8;
                        }
                    }
                }

                rgba_img.put_pixel(x, y, new_pixel);
            }
        }
    }

    let img = DynamicImage::ImageRgba8(rgba_img);

    // Convert so we can blur image.
    if let Some(gaussian_blur_sigma) = settings.gaussian_blur_sigma {
        let img = img.blur(gaussian_blur_sigma); // apply a blur filter

        return DynamicImage::ImageRgba8(img.to_rgba8());
    }

    DynamicImage::ImageRgba8(img.to_rgba8())
}
