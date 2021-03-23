use std::ffi::OsStr;

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    render::texture::{ImageType, Texture},
    utils::HashMap,
};

mod entity;
pub mod spine;
pub mod sprite;

use entity::*;
use spine::Atlas;
use sprite::{Rotation, Sprite, SpriteShape};

pub struct SpineImpoter;

const EXTENSIONS: &'static [&'static str] = &["spine_json"];
impl AssetLoader for SpineImpoter {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let spine = spine::Spine::parse(bytes)?;

            // // List of loaded sprites
            // let mut sprites: HashMap<String, Handle<Sprite>> = Default::default();

            // Load atlas with the same name
            if let Ok(bytes) = load_context
                .read_asset_bytes(load_context.path().with_extension("spine_atlas"))
                .await
            {
                let atlas = Atlas::parse(&bytes[..])?;

                // Load texture
                let texture_path = load_context.path().with_file_name(&atlas.name);
                let texture_extension = texture_path
                    .extension()
                    .map(OsStr::to_str)
                    .flatten()
                    .unwrap_or("");
                let texture_buffer = load_context.read_asset_bytes(&texture_path).await?;
                let texture = Texture::from_buffer(
                    &texture_buffer[..],
                    ImageType::Extension(texture_extension),
                )?;
                // TODO: Set texture format, filter and repeat attributes
                let texture =
                    load_context.set_labeled_asset(&atlas.name, LoadedAsset::new(texture));

                let mut atlas_size: Vec2 = atlas.size.into();
                atlas_size = atlas_size.recip();

                for region in &atlas.regions {
                    let size: Vec2 = region.size.into();
                    let mut pivot: Vec2 = region.orig.into();
                    let mut size_uv = size;

                    if region.rotate {
                        size_uv = Vec2::new(size_uv.y, size_uv.x);
                        pivot = Vec2::new(pivot.y, pivot.x);
                    }

                    pivot *= size_uv.recip();
                    size_uv *= atlas_size;

                    // ! FIXME: 1 pixel line and row is been trimmed
                    let mut min: Vec2 = region.xy.into();
                    min *= atlas_size;
                    let mut max: Vec2 = min + size_uv;
                    std::mem::swap(&mut min.y, &mut max.y);

                    let sprite = Sprite::with_shape(
                        Some(texture.clone()),
                        SpriteShape::Rect {
                            min,
                            max,
                            rotation: if region.rotate {
                                Rotation::CCW
                            } else {
                                Rotation::None
                            },
                            size,
                            pivot,
                            padding: None,
                        },
                    );
                    //sprite.name = Some(region.name.clone());

                    //let sprite_handle =
                    load_context.set_labeled_asset(&region.name, LoadedAsset::new(sprite));
                }
            } else {
                // TODO: Fallback sprites from the spine `spine.skeleton.images`
                todo!("unpacked sprites")
            }

            let mut world = World::default();
            let world_builder = &mut world.build();

            let entity_root = world_builder.spawn(()).current_entity().unwrap();

            world_builder.with_children(|world_builder| {
                let mut bones_lookup: HashMap<String, Entity> = Default::default();
                // TODO: Bone InheritTransform, length, color and shear
                for bone in &spine.bones {
                    let entity = world_builder
                        .spawn(BoneBundle {
                            parent: Parent(
                                bone.parent
                                    .as_ref()
                                    .and_then(|parent_name| bones_lookup.get(parent_name))
                                    .copied()
                                    .unwrap_or_else(|| entity_root),
                            ),
                            name: Name::new(bone.name.clone()),
                            transform: Transform {
                                translation: Vec3::new(bone.x, bone.y, 0.0),
                                rotation: Quat::from_rotation_z(bone.rotation),
                                scale: Vec3::new(bone.scale_x, bone.scale_y, 1.0),
                            },
                            ..Default::default()
                        })
                        .current_entity()
                        .unwrap();

                    bones_lookup.insert(bone.name.clone(), entity);
                }
            });

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}

#[derive(Default)]
pub struct SpinePlugin;

impl Plugin for SpinePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let _ = app;
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
