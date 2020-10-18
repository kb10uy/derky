use super::{
    Error as ObjError, FaceIndexPair, Group, Material, MaterialProperty, Object, WavefrontObj,
};
use crate::AnyResult;

use std::{
    collections::HashMap,
    error::Error,
    io::{prelude::*, BufReader},
    num::NonZeroUsize,
    path::PathBuf,
    str::FromStr,
};

use log::warn;
use ultraviolet::{Vec2, Vec3};

/// Wavefron OBJ の マテリアルの値を表す。
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialProperty {
    /// `Ni` などの小数値
    Float(f32),

    /// `illum` などの整数値
    Integer(u32),

    /// `Kd` などのVec3 値
    Vector(Vec3),

    /// `map_Kd` などのパス情報
    Path(Box<Path>),
}

/// .mtl ファイルで定義されるマテリアル情報を表す。
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    name: String,
    properties: HashMap<String, MaterialProperty>,
}

#[allow(dead_code)]
impl Material {
    /// マテリアル名を返す。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `Ka` の値を返す。
    pub fn ambient_color(&self) -> Option<Vec3> {
        match self.properties.get("Ka") {
            Some(MaterialProperty::Vector(v)) => Some(*v),
            _ => None,
        }
    }

    /// `Kd` の値を返す。
    pub fn diffuse_color(&self) -> Option<Vec3> {
        match self.properties.get("Kd") {
            Some(MaterialProperty::Vector(v)) => Some(*v),
            _ => None,
        }
    }

    /// `Ns` の値を返す。
    pub fn specular_intensity(&self) -> Option<f32> {
        match self.properties.get("Ns") {
            Some(MaterialProperty::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// `illum` の値を返す。
    pub fn illumination(&self) -> Option<u32> {
        match self.properties.get("illum") {
            Some(MaterialProperty::Integer(v)) => Some(*v),
            _ => None,
        }
    }

    /// `map_Kd` の値を返す。
    pub fn diffuse_map(&self) -> Option<&Path> {
        match self.properties.get("map_Kd") {
            Some(MaterialProperty::Path(v)) => Some(v),
            _ => None,
        }
    }

    /// マテリアルの値を取得する。
    pub fn get(&self, key: &str) -> Option<&MaterialProperty> {
        self.properties.get(key)
    }
}

#[derive(Debug, Default)]
struct MtlBuffer {
    name: String,
    properties: HashMap<String, MaterialProperty>,
    complete_materials: Vec<Material>,
}

impl MtlBuffer {
    fn commit_material(&mut self) {
        if self.properties.len() > 0 {
            let group = Material {
                name: self.name.clone(),
                properties: self.properties.clone(),
            };
            self.complete_materials.push(group);
        }
        self.properties.clear();
    }
}

/// .mtl ファイルをパースする。
fn parse_mtl(mtl_buffer: &mut MtlBuffer, reader: impl Read) -> AnyResult<()> {
    let mut reader = BufReader::new(reader);

    let mut line_buffer = String::with_capacity(1024);
    loop {
        line_buffer.clear();
        let read_size = reader.read_line(&mut line_buffer)?;
        if read_size == 0 {
            break;
        }

        let trimmed = line_buffer.trim();
        if trimmed == "" || trimmed.starts_with('#') {
            continue;
        }

        let mut elements = line_buffer.trim().split_whitespace();
        let keyword = elements
            .next()
            .expect("Each line should have at least one element");
        let data: Vec<&str> = elements.collect();

        process_mtl_line(mtl_buffer, keyword, &data)?;
    }

    Ok(())
}

fn process_mtl_line(mtl_buffer: &mut MtlBuffer, keyword: &str, data: &[&str]) -> AnyResult<()> {
    match keyword {
        "newmtl" => {
            mtl_buffer.commit_material();
            mtl_buffer.name = data.get(0).unwrap_or(&"").to_string();
        }
        "illum" => {
            let value = take_single(data)?;
            mtl_buffer
                .properties
                .insert("illum".to_owned(), MaterialProperty::Integer(value));
        }
        k if k.starts_with("K") => {
            let value = take_vec3(data)?;
            mtl_buffer
                .properties
                .insert(k.to_owned(), MaterialProperty::Vector(value));
        }
        k if k.starts_with("N") => {
            let value = take_single(data)?;
            mtl_buffer
                .properties
                .insert(k.to_owned(), MaterialProperty::Float(value));
        }
        k if k.starts_with("map_") => {
            let value = PathBuf::from_str(&data.get(0).unwrap_or(&"").replace("\\\\", "\\"))?;
            mtl_buffer.properties.insert(
                k.to_owned(),
                MaterialProperty::Path(value.into_boxed_path()),
            );
        }
        _ => {
            warn!("Unsupported MTL keyword: {}", keyword);
        }
    }
    Ok(())
}
