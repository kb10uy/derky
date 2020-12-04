//! Contains general model operations.

use std::{error::Error, fs::read_to_string, io::Cursor, path::Path};

use anyhow::{Context, Result};
use itertools::Itertools;
use weavy_crab::{FaceVertexPair, Material, Parser};

/// Represents a generic model data structure.
pub struct Model<VG, M> {
    vertex_groups: Box<[VG]>,
    material_mapping: Box<[Option<usize>]>,
    materials: Box<[M]>,
}

impl<VG, M> Model<VG, M> {
    /// Loads a Wavefront OBJ file and transform it.
    /// # Parameters
    /// * `filename`: path to file
    /// * `vertex_mapper` a closure that converts faces into `VG`
    /// * `material_mapper` a closure that converts `Material` into `M`
    pub fn load_obj<
        P: AsRef<Path>,
        E: 'static + Send + Sync + Error,
        VM: FnMut(Box<[Box<[FaceVertexPair]>]>) -> Result<VG, E>,
        MM: FnMut(Material) -> Result<M, E>,
    >(
        filename: P,
        vertex_mapper: VM,
        material_mapper: MM,
    ) -> Result<Model<VG, M>> {
        let filename = filename.as_ref();
        let parent_directory = filename
            .parent()
            .context("Parent directory not found")?
            .to_owned();

        let wfobj = {
            let obj_file = read_to_string(filename).context("Failed to read OBJ file")?;
            let mut parser = Parser::new(move |mtllib, _| {
                let mut mtl_path = parent_directory.clone();
                mtl_path.push(mtllib);

                let mtl_file = Cursor::new(read_to_string(mtl_path)?);
                Ok(mtl_file)
            });

            parser.parse(Cursor::new(obj_file), ())?
        };

        let (wf_objects, wf_materials) = wfobj.split();

        let materials = wf_materials
            .into_vec()
            .into_iter()
            .map(material_mapper)
            .collect::<Result<Box<[M]>, E>>()
            .context("Error occured during material mapping")?;

        let mut vertex_groups = vec![];
        let mut material_mapping = vec![];
        let mut vertex_mapper = vertex_mapper;
        for object in wf_objects.into_vec() {
            for group in object.into_groups().into_vec() {
                for (material_index, faces) in &group.faces().group_by(|f| f.1) {
                    let vertex_group =
                        vertex_mapper(faces.map(|(face, _)| face.collect()).collect())?;
                    vertex_groups.push(vertex_group);
                    material_mapping.push(material_index);
                }
            }
        }

        Ok(Model {
            vertex_groups: vertex_groups.into_boxed_slice(),
            material_mapping: material_mapping.into_boxed_slice(),
            materials,
        })
    }

    /// Visits all vertex groups.
    pub fn visit(&self) -> Visit<VG, M> {
        Visit {
            model: self,
            index: 0,
            terminated: false,
        }
    }
}

/// The iterator adaptor for `Model::visit`.
pub struct Visit<'a, VG, M> {
    model: &'a Model<VG, M>,
    index: usize,
    terminated: bool,
}

impl<'a, VG, M> Iterator for Visit<'a, VG, M> {
    type Item = (&'a VG, Option<&'a M>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.terminated {
            return None;
        } else if self.index >= self.model.vertex_groups.len() {
            self.terminated = true;
            return None;
        }

        let pair = (
            &self.model.vertex_groups[self.index],
            self.model.material_mapping[self.index].map(|i| &self.model.materials[i]),
        );
        self.index += 1;
        Some(pair)
    }
}
