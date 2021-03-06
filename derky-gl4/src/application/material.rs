//! マテリアル内容を記述するモジュール。

use glium::Texture2d;
use ultraviolet::Vec4;

/// マテリアル定義
#[derive(Debug)]
#[allow(dead_code)]
pub enum Material {
    /// Lambert モデル
    Diffuse { albedo: Texture2d, color: Vec4 },

    /// Phong モデル
    Specular {
        albedo: Texture2d,
        color: Vec4,
        intensity: f32,
    },

    /// Unlit (単純なテクスチャマッピング)
    Unlit { albedo: Texture2d },
}
