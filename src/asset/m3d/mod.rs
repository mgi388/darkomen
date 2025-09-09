pub mod mesh;

use std::{
    io::Cursor,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, AssetPath, LoadContext};
use bevy_ecs::prelude::*;
use bevy_image::{
    prelude::*, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor,
};
use bevy_pbr::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::{prelude::*, render_asset::RenderAssetUsages};
use derive_more::{Display, Error, From};
use dyn_clone::DynClone;
use image::Rgba;
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::m3d::*;

use mesh::*;

pub const EXTENSIONS: &[&str; 4] = &["M3D", "m3d", "M3X", "m3x"];

pub struct M3dAssetPlugin<
    #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
    #[cfg(not(feature = "debug"))] MaterialT: Material,
> {
    _phantom: PhantomData<MaterialT>,

    material_loader: Box<dyn MaterialLoader<MaterialT> + Send + Sync>,

    textures_path: PathBuf,
    low_resolution_textures_path: PathBuf,
}

impl<
        #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
        #[cfg(not(feature = "debug"))] MaterialT: Material,
    > M3dAssetPlugin<MaterialT>
{
    pub fn new(
        material_loader: Box<dyn MaterialLoader<MaterialT> + Send + Sync>,
        textures_path: PathBuf,
        low_resolution_textures_path: PathBuf,
    ) -> Self {
        Self {
            _phantom: PhantomData,
            material_loader,
            textures_path,
            low_resolution_textures_path,
        }
    }
}

impl<
        #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
        #[cfg(not(feature = "debug"))] MaterialT: Material,
    > Plugin for M3dAssetPlugin<MaterialT>
{
    fn build(&self, app: &mut App) {
        app.insert_resource(M3dAssetLoaderSettings::<MaterialT>::new(
            self.textures_path.clone(),
            self.low_resolution_textures_path.clone(),
        ))
        .register_type::<M3dAssetLoaderSettings<MaterialT>>()
        .init_asset::<M3dAsset<MaterialT>>()
        .preregister_asset_loader::<M3dAssetLoader<MaterialT>>(EXTENSIONS)
        .register_asset_reflect::<M3dAsset<MaterialT>>();
    }

    fn finish(&self, app: &mut App) {
        let settings = app.world().resource::<M3dAssetLoaderSettings<MaterialT>>();

        app.register_asset_loader(M3dAssetLoader::<MaterialT>::new(
            settings.clone(),
            dyn_clone::clone_box(&*self.material_loader),
        ));
    }
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(Default))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct M3dTextureDescriptor {
    pub transparent: bool,
    pub color_keyed: bool,
    pub animated: bool,
}

pub trait MaterialLoader<
    #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
    #[cfg(not(feature = "debug"))] MaterialT: Material,
>: DynClone
{
    fn load(
        &self,
        load_context: &mut LoadContext,
        transparent: bool,
        texture_handles: Vec<Handle<Image>>,
        texture_descriptors: Vec<M3dTextureDescriptor>,
        object_index: usize,
        object: &Object,
    ) -> Handle<MaterialT>;
}

#[derive(Asset, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(not(feature = "bevy_reflect"), derive(TypePath))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct M3dAsset<
    #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
    #[cfg(not(feature = "debug"))] MaterialT: Material,
> {
    pub meshes: Vec<M3dMesh<MaterialT>>,
    pub animated: bool,
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(not(feature = "bevy_reflect"), derive(TypePath))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct M3dMesh<
    #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
    #[cfg(not(feature = "debug"))] MaterialT: Material,
> {
    /// Topology to be rendered.
    pub mesh: Handle<Mesh>,

    /// Material to be used.
    pub material: Handle<MaterialT>,

    pub source_object: Object,
}

impl<
        #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
        #[cfg(not(feature = "debug"))] MaterialT: Material,
    > M3dMesh<MaterialT>
{
    pub fn name(&self) -> &str {
        &self.source_object.name
    }
}

pub struct M3dAssetLoader<
    #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
    #[cfg(not(feature = "debug"))] MaterialT: Material,
> {
    _phantom: PhantomData<MaterialT>,

    default_settings: M3dAssetLoaderSettings<MaterialT>,
    material_loader: Box<dyn MaterialLoader<MaterialT> + Send + Sync>,
}

#[derive(Clone, Deserialize, Resource, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(Resource))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct M3dAssetLoaderSettings<
    #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
    #[cfg(not(feature = "debug"))] MaterialT: Material,
> {
    #[reflect(ignore)]
    _phantom: PhantomData<MaterialT>,

    pub use_low_resolution_textures: bool,

    /// Path to the standard resolution textures. Relative to the directory of
    /// the M3D file.
    pub textures_path: PathBuf,
    /// Path to the low resolution textures. Relative to the directory of the
    /// M3D file.
    pub low_resolution_textures_path: PathBuf,
}

impl<
        #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
        #[cfg(not(feature = "debug"))] MaterialT: Material,
    > Default for M3dAssetLoaderSettings<MaterialT>
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,

            use_low_resolution_textures: false,
            textures_path: PathBuf::new(),
            low_resolution_textures_path: PathBuf::new(),
        }
    }
}

impl<
        #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
        #[cfg(not(feature = "debug"))] MaterialT: Material,
    > M3dAssetLoaderSettings<MaterialT>
{
    pub fn new(textures_path: PathBuf, low_resolution_textures_path: PathBuf) -> Self {
        Self {
            _phantom: PhantomData,

            use_low_resolution_textures: false,
            textures_path,
            low_resolution_textures_path,
        }
    }

    pub fn with_low_resolution_textures(mut self, use_low: bool) -> Self {
        self.use_low_resolution_textures = use_low;
        self
    }
}

/// Possible errors that can be produced by [`M3dAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum M3dAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode m3d: {_0}")]
    DecodeError(DecodeError),
    /// A [TextureError](bevy::render::texture) error.
    /// This error is produced when loading textures.
    #[display("could not load texture: {_0}")]
    TextureError(TextureError),
    #[display("could not load texture: {dependency}")]
    LoadTextureError { dependency: AssetPath<'static> },
}

impl<
        #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
        #[cfg(not(feature = "debug"))] MaterialT: Material,
    > AssetLoader for M3dAssetLoader<MaterialT>
{
    type Asset = M3dAsset<MaterialT>;
    type Settings = M3dAssetLoaderSettings<MaterialT>;
    type Error = M3dAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let m3d = decoder.decode()?;

        let textures_path = if settings.use_low_resolution_textures {
            if settings
                .low_resolution_textures_path
                .to_string_lossy()
                .is_empty()
            {
                self.default_settings.low_resolution_textures_path.clone()
            } else {
                settings.low_resolution_textures_path.clone()
            }
        } else if settings.textures_path.to_string_lossy().is_empty() {
            self.default_settings.textures_path.clone()
        } else {
            settings.textures_path.clone()
        };

        self.load_m3d(load_context, textures_path, &m3d).await
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}

impl<
        #[cfg(feature = "debug")] MaterialT: Material + core::fmt::Debug,
        #[cfg(not(feature = "debug"))] MaterialT: Material,
    > M3dAssetLoader<MaterialT>
{
    pub fn new(
        settings: M3dAssetLoaderSettings<MaterialT>,
        material_loader: Box<dyn MaterialLoader<MaterialT> + Send + Sync>,
    ) -> Self {
        Self {
            _phantom: PhantomData,
            default_settings: settings,
            material_loader,
        }
    }

    async fn load_m3d(
        self: &M3dAssetLoader<MaterialT>,
        load_context: &mut LoadContext<'_>,
        textures_path: PathBuf,
        m3d: &M3d,
    ) -> Result<M3dAsset<MaterialT>, M3dAssetLoaderError> {
        let file_path = load_context
            .asset_path()
            .path()
            .to_str()
            .expect("file path should be valid")
            .to_string();

        let _span = info_span!("load_m3d", name = file_path);

        let file_name = load_context
            .asset_path()
            .path()
            .file_name()
            .expect("file name should be valid")
            .to_str()
            .expect("file name should be valid UTF-8")
            .to_string();

        let transparent = is_m3d_transparent(&file_name);
        let animated = is_m3d_animated(&file_name);

        _span.in_scope(|| debug!("Transparent: {}, animated: {}", transparent, animated));

        let (texture_handles, texture_desciptors) =
            load_textures(load_context, textures_path, m3d).await?;

        let mut meshes = Vec::new();
        for (object_index, object) in m3d.objects.iter().enumerate() {
            // Some objects have no faces, so we skip them because there's
            // nothing to render.
            if object.faces.is_empty() {
                debug!("Skipping object with no faces: {}", object.name);
                continue;
            }

            let mut mesh = mesh_from_m3d_object(object);

            let generate_tangents_span = info_span!("generate_tangents", name = file_path);

            generate_tangents_span.in_scope(|| {
                if let Err(err) = mesh.generate_tangents() {
                    warn!("Could not generate tangents: {}", err);
                }
            });

            let object_label = object_label(object);

            let mesh = load_context.add_labeled_asset(object_label, mesh);

            let material = self.material_loader.load(
                load_context,
                transparent,
                texture_handles.clone(),
                texture_desciptors.clone(),
                object_index,
                object,
            );

            meshes.push(M3dMesh::<MaterialT> {
                mesh,
                material,
                source_object: object.clone(),
            });
        }

        Ok(M3dAsset { meshes, animated })
    }
}

struct LabeledImage {
    image: Image,
    label: String,
}

async fn load_textures(
    load_context: &mut LoadContext<'_>,
    textures_path: PathBuf,
    m3d: &M3d,
) -> Result<(Vec<Handle<Image>>, Vec<M3dTextureDescriptor>), M3dAssetLoaderError> {
    fn process_loaded_texture(
        load_context: &mut LoadContext,
        handles: &mut Vec<Handle<Image>>,
        texture: LabeledImage,
    ) {
        let handle = load_context.add_labeled_asset(texture.label, texture.image);
        handles.push(handle);
    }

    let mut texture_handles = Vec::new();
    let mut texture_descriptors = Vec::new();

    let textures_path = load_context.path().parent().unwrap().join(textures_path);

    for descriptor in m3d.texture_descriptors.clone() {
        let image = load_image(load_context, &descriptor, &textures_path).await?;
        process_loaded_texture(load_context, &mut texture_handles, image);
        texture_descriptors.push(M3dTextureDescriptor {
            color_keyed: descriptor.is_color_keyed(),
            ..Default::default()
        });
    }

    Ok((texture_handles, texture_descriptors))
}

/// Loads a texture as a bevy [`Image`] and returns it together with its label.
async fn load_image(
    load_context: &mut LoadContext<'_>,
    texture_descriptor: &crate::m3d::M3dTextureDescriptor,
    textures_path: &Path,
) -> Result<LabeledImage, M3dAssetLoaderError> {
    let path = textures_path.join(&texture_descriptor.file_name);

    let loaded = load_context
        .loader()
        .immediate()
        .load::<Image>(path.clone())
        .await
        .map_err(|_| M3dAssetLoaderError::LoadTextureError {
            dependency: path.clone().into(),
        })?;

    let img = loaded.get();

    let mut dyn_img = img
        .clone()
        .try_into_dynamic()
        .map_err(|_| M3dAssetLoaderError::LoadTextureError {
            dependency: path.clone().into(),
        })?
        .into_rgba8();

    for y in 0..dyn_img.height() {
        for x in 0..dyn_img.width() {
            let pixel = dyn_img.get_pixel(x, y);
            // Convert black pixels to transparent.
            // TODO: Can/should we do this in an asset processor?
            if texture_descriptor.is_color_keyed()
                && pixel[0] == 0
                && pixel[1] == 0
                && pixel[2] == 0
            {
                dyn_img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    let mut image = Image::from_dynamic(dyn_img.into(), true, RenderAssetUsages::default());
    image.sampler = ImageSampler::Descriptor(texture_sampler());

    Ok(LabeledImage {
        image,
        label: texture_label(texture_descriptor),
    })
}

fn texture_sampler() -> ImageSamplerDescriptor {
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..Default::default()
    }
}

/// Returns the label for the `texture_descriptor`.
fn texture_label(texture_descriptor: &crate::m3d::M3dTextureDescriptor) -> String {
    format!("Texture{}", texture_descriptor.file_name)
}

/// Returns the label for the `object`.
fn object_label(object: &Object) -> String {
    format!("Object{}", object.name)
}

// TODO: Translucency?
//
// TODO: Use flags?

// TODO: There are models that start with _4 e.g. in B2_01 and B2_05. What does
// this flag mean?
pub fn is_m3d_4(file_name: &str) -> bool {
    file_name.to_ascii_lowercase().starts_with("_4")
}

pub fn is_m3d_transparent(file_name: &str) -> bool {
    file_name.to_ascii_lowercase().starts_with("_7")
}

pub fn is_m3d_animated(file_name: &str) -> bool {
    file_name.to_ascii_lowercase().starts_with("_7")
        || file_name.to_ascii_lowercase().starts_with("_6")
}

pub fn is_m3d_color_keyed(file_name: &str) -> bool {
    file_name.to_ascii_lowercase().starts_with("_k")
}
