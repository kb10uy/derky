//! マテリアル内容を記述するモジュール。

use glium::Texture2d;
use ultraviolet::Vec4;

/// マテリアル定義
#[derive(Debug)]
pub enum Material<'t> {
    /// Lambert モデル
    Diffuse { albedo: &'t Texture2d, color: Vec4 },

    /// Phong モデル
    Specular {
        albedo: &'t Texture2d,
        color: Vec4,
        intensity: f32,
    },

    /// Unlit (単純なテクスチャマッピング)
    Unlit { albedo: &'t Texture2d },
}
