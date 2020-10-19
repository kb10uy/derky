use std::{collections::HashMap, path::Path};

use ultraviolet::Vec3;

/// Represents a single value in material definition.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialProperty {
    /// Float value.
    /// Property name starts with `N`.
    Float(f32),

    /// Integer value.
    Integer(u32),

    /// Vector value.
    /// Property name starts with `K`.
    Vector(Vec3),

    /// Path value.
    /// Property name starts with `map_`.
    Path(Box<Path>),
}

/// Represents a material defined in MTL file.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub(crate) name: Box<str>,
    pub(crate) properties: HashMap<String, MaterialProperty>,
}

impl Material {
    /// The material name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The ambient color, which is defined with `Ka`.
    pub fn ambient_color(&self) -> Option<Vec3> {
        match self.properties.get("Ka") {
            Some(MaterialProperty::Vector(v)) => Some(*v),
            _ => None,
        }
    }

    /// The diffuse color, which is defined with `Kd`.
    pub fn diffuse_color(&self) -> Option<Vec3> {
        match self.properties.get("Kd") {
            Some(MaterialProperty::Vector(v)) => Some(*v),
            _ => None,
        }
    }

    /// The specular color, which is defined with `Ks`.
    pub fn specular_color(&self) -> Option<Vec3> {
        match self.properties.get("Ks") {
            Some(MaterialProperty::Vector(v)) => Some(*v),
            _ => None,
        }
    }

    /// The specular intensity, which is defined with `Ns`.
    pub fn specular_intensity(&self) -> Option<f32> {
        match self.properties.get("Ns") {
            Some(MaterialProperty::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// The illumination type, which is defined with `illum`.
    pub fn illumination(&self) -> Option<u32> {
        match self.properties.get("illum") {
            Some(MaterialProperty::Integer(v)) => Some(*v),
            _ => None,
        }
    }

    /// The diffuse map, which is defined with `map_Kd`.
    pub fn diffuse_map(&self) -> Option<&Path> {
        match self.properties.get("map_Kd") {
            Some(MaterialProperty::Path(v)) => Some(v),
            _ => None,
        }
    }

    /// Returns defined value with specified key.
    pub fn get(&self, key: &str) -> Option<&MaterialProperty> {
        self.properties.get(key)
    }
}
