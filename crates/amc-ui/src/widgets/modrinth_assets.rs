//! Bundled Modrinth artwork used by the app-inspired shell.
//!
//! The instance icons and brand mark are taken from the public Modrinth App
//! source tree (`apps/app-frontend/src/assets/instance-icons` and
//! `apps/app/icons`) so the replica uses the same artwork as the reference
//! instead of hand-drawn stand-ins. Modrinth App is GPL-3.0; see the
//! repository credits in `shells::modrinth`.

use amc_core::types::LoaderType;
use egui::{Color32, TextureHandle, TextureOptions};

const GRASS: &[u8] = include_bytes!("../../../../assets/modrinth/instance-icons/grass-block.png");
const FABRIC: &[u8] = include_bytes!("../../../../assets/modrinth/instance-icons/fabric.png");
const QUILT: &[u8] = include_bytes!("../../../../assets/modrinth/instance-icons/quilt.png");
const FORGE: &[u8] = include_bytes!("../../../../assets/modrinth/instance-icons/forge.png");
const NEOFORGE: &[u8] = include_bytes!("../../../../assets/modrinth/instance-icons/neoforge.png");
const CREEPER: &[u8] = include_bytes!("../../../../assets/modrinth/instance-icons/creeper.png");
const WORDMARK: &[u8] = include_bytes!("../../../../assets/modrinth/branding/wordmark.png");

/// Reference default instance artwork for a loader. Vanilla uses the grass
/// block, loaders use their own art, and OptiFine falls back to the creeper.
fn loader_art(loader: LoaderType) -> (&'static str, &'static [u8]) {
    match loader {
        LoaderType::Vanilla => ("modrinth-art-vanilla", GRASS),
        LoaderType::Fabric => ("modrinth-art-fabric", FABRIC),
        LoaderType::Quilt => ("modrinth-art-quilt", QUILT),
        LoaderType::Forge => ("modrinth-art-forge", FORGE),
        LoaderType::NeoForge => ("modrinth-art-neoforge", NEOFORGE),
        LoaderType::OptiFine => ("modrinth-art-optifine", CREEPER),
    }
}

/// Cached texture for a loader's reference artwork.
pub fn loader_texture(ctx: &egui::Context, loader: LoaderType) -> Option<TextureHandle> {
    let (key, bytes) = loader_art(loader);
    cached_texture(ctx, key, bytes, 256)
}

/// Cached official Modrinth text logo (icon + wordmark) used in the titlebar.
pub fn wordmark_texture(ctx: &egui::Context) -> Option<TextureHandle> {
    cached_texture(ctx, "modrinth-wordmark", WORDMARK, 660)
}

fn cached_texture(
    ctx: &egui::Context,
    key: &'static str,
    bytes: &'static [u8],
    max_side: u32,
) -> Option<TextureHandle> {
    let id = egui::Id::new(key);
    if let Some(tex) = ctx.data(|data| data.get_temp::<TextureHandle>(id)) {
        return Some(tex);
    }
    let image = image::load_from_memory(bytes).ok()?;
    let image = if image.width() > max_side || image.height() > max_side {
        image.thumbnail(max_side, max_side)
    } else {
        image
    };
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    if size[0] == 0 || size[1] == 0 {
        return None;
    }
    let pixels = rgba
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    let texture = ctx.load_texture(
        key,
        egui::ColorImage { size, pixels },
        TextureOptions::LINEAR,
    );
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    Some(texture)
}
