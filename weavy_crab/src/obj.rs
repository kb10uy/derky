use ultraviolet::{Vec2, Vec3};

/// Represents an index pair in face definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceIndexPair(pub usize, pub Option<usize>, pub Option<usize>);

/// Represents an object in OBJ file.
#[derive(Debug, Clone)]
pub struct Object {
    pub(crate) name: Option<Box<str>>,
    pub(crate) groups: Box<[Group]>,
}

impl Object {
    /// The name of this object.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The groups which this object has.
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    /// Take owned `Group`s.
    pub fn into_groups(self) -> Box<[Group]> {
        self.groups
    }
}

/// Represents a group of object.
#[derive(Debug, Clone)]
pub struct Group {
    pub(crate) name: Option<Box<str>>,
    pub(crate) vertices: Box<[Vec3]>,
    pub(crate) texture_uvs: Box<[Vec2]>,
    pub(crate) normals: Box<[Vec3]>,
    pub(crate) face_index_pairs: Box<[(Box<[FaceIndexPair]>, Option<usize>)]>,
}

impl Group {
    /// The name of this group.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The vertex definitions.
    pub fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    /// The material UV definitions.
    pub fn texture_uvs(&self) -> &[Vec2] {
        &self.texture_uvs
    }

    /// The normal definitions (normalized).
    pub fn normals(&self) -> &[Vec3] {
        &self.normals
    }

    /// The slice of face index pairs.
    /// Each element corresponds to face, and its elements are face index pairs.
    pub fn face_index_pairs(&self) -> &[(Box<[FaceIndexPair]>, Option<usize>)] {
        &self.face_index_pairs
    }

    /// Iterates all faces in this group.
    pub fn faces(&self) -> GroupFaces {
        GroupFaces {
            source_group: self,
            current_index: 0,
        }
    }
}

/// The iterator adaptor for faces in `Group`.
/// It returns another iterator which iterates vertices in each face.
#[derive(Debug)]
pub struct GroupFaces<'a> {
    source_group: &'a Group,
    current_index: usize,
}

impl<'a> Iterator for GroupFaces<'a> {
    type Item = (FaceVertices<'a>, Option<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.source_group.face_index_pairs.len() {
            let (vertices, material) = &self.source_group.face_index_pairs[self.current_index];

            let result = FaceVertices {
                source_group: self.source_group,
                source_pairs: vertices,
                current_index: 0,
            };
            self.current_index += 1;
            Some((result, *material))
        } else {
            None
        }
    }
}

/// The iterator adapter for vertices in each face.
#[derive(Debug)]
pub struct FaceVertices<'a> {
    source_group: &'a Group,
    source_pairs: &'a [FaceIndexPair],
    current_index: usize,
}

impl<'a> Iterator for FaceVertices<'a> {
    type Item = (Vec3, Option<Vec2>, Option<Vec3>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.source_pairs.len() {
            let index_pair = &self.source_pairs[self.current_index];
            let result = (
                self.source_group.vertices[index_pair.0],
                index_pair.1.map(|i| self.source_group.texture_uvs[i]),
                index_pair.2.map(|i| self.source_group.normals[i]),
            );
            self.current_index += 1;
            Some(result)
        } else {
            None
        }
    }
}
