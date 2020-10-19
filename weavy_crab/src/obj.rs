use ultraviolet::{Vec2, Vec3};

/// Represents an index pair in face definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceIndexPair(usize, Option<usize>, Option<usize>);

/// Represents an object in OBJ file.
#[derive(Debug, Clone)]
pub struct Object {
    name: Option<Box<str>>,
    groups: Box<[Group]>,
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
    name: Option<Box<str>>,
    vertices: Box<[Vec3]>,
    texture_uvs: Box<[Vec2]>,
    normals: Box<[Vec3]>,
    face_index_pairs: Box<[Box<[FaceIndexPair]>]>,
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
    pub fn face_index_pairs(&self) -> &[Box<[FaceIndexPair]>] {
        &self.face_index_pairs
    }

    /// Iterates all faces in this group.
    pub fn faces(&self) -> GroupFaces {
        GroupFaces(self, 0)
    }
}

/// The iterator adaptor for faces in `Group`.
/// It returns another iterator which iterates vertices in each face.
#[derive(Debug)]
pub struct GroupFaces<'a>(&'a Group, usize);

impl<'a> Iterator for GroupFaces<'a> {
    type Item = FaceVertices<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 < self.0.face_index_pairs.len() {
            let result = FaceVertices(self.0, &self.0.face_index_pairs[self.1], 0);
            self.1 += 1;
            Some(result)
        } else {
            None
        }
    }
}

/// The iterator adapter for vertices in each face.
#[derive(Debug)]
pub struct FaceVertices<'a>(&'a Group, &'a [FaceIndexPair], usize);

impl<'a> Iterator for FaceVertices<'a> {
    type Item = (Vec3, Option<Vec2>, Option<Vec3>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.2 < self.1.len() {
            let index_pair = &self.1[self.2];
            let result = (
                self.0.vertices[index_pair.0],
                index_pair.1.map(|i| self.0.texture_uvs[i]),
                index_pair.2.map(|i| self.0.normals[i]),
            );
            self.2 += 1;
            Some(result)
        } else {
            None
        }
    }
}
