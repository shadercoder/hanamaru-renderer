use vector::{Vector3, Vector2};
use scene::{Aabb, Mesh, Intersection};
use camera::Ray;
use config;
use math::det;

#[derive(Debug)]
pub struct BvhNode {
    pub aabb: Aabb,

    // size must be 0 or 2
    // empty means leaf node
    pub children: Vec<Box<BvhNode>>,

    // has faces means leaf node
    pub face_indexes: Vec<usize>,
}

impl BvhNode {
    fn empty() -> BvhNode {
        BvhNode {
            aabb: Aabb {
                left_bottom: Vector3::new(config::INF, config::INF, config::INF),
                right_top: Vector3::new(-config::INF, -config::INF, -config::INF),
            },
            children: vec![],
            face_indexes: vec![],
        }
    }

    fn set_aabb(&mut self, mesh: &Mesh, face_indexes: &Vec<usize>) {
        for face_index in face_indexes {
            let face = &mesh.faces[*face_index];
            let v0 = &mesh.vertexes[face.v0];
            let v1 = &mesh.vertexes[face.v1];
            let v2 = &mesh.vertexes[face.v2];

            self.aabb.left_bottom.x = self.aabb.left_bottom.x.min(v0.x).min(v1.x).min(v2.x);
            self.aabb.left_bottom.y = self.aabb.left_bottom.y.min(v0.y).min(v1.y).min(v2.y);
            self.aabb.left_bottom.z = self.aabb.left_bottom.z.min(v0.z).min(v1.z).min(v2.z);

            self.aabb.right_top.x = self.aabb.right_top.x.max(v0.x).max(v1.x).max(v2.x);
            self.aabb.right_top.y = self.aabb.right_top.y.max(v0.y).max(v1.y).max(v2.y);
            self.aabb.right_top.z = self.aabb.right_top.z.max(v0.z).max(v1.z).max(v2.z);
        }
    }

    fn from_face_indexes(mesh: &Mesh, face_indexes: &mut Vec<usize>) -> BvhNode {
        let mut node = BvhNode::empty();
        node.set_aabb(mesh, face_indexes);

        let mid = face_indexes.len() / 2;
        if mid <= 2 {
            // set leaf node
            node.face_indexes = face_indexes.clone();
        } else {
            // set intermediate node
            let lx = node.aabb.right_top.x - node.aabb.left_bottom.x;
            let ly = node.aabb.right_top.y - node.aabb.left_bottom.y;
            let lz = node.aabb.right_top.z - node.aabb.left_bottom.z;

            if lx > ly && lx > lz {
                face_indexes.sort_by(|a, b| {
                    let a_face = &mesh.faces[*a];
                    let b_face = &mesh.faces[*b];
                    let a_sum = mesh.vertexes[a_face.v0].x + mesh.vertexes[a_face.v1].x + mesh.vertexes[a_face.v2].x;
                    let b_sum = mesh.vertexes[b_face.v0].x + mesh.vertexes[b_face.v1].x + mesh.vertexes[b_face.v2].x;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            } else if ly > lx && ly > lz {
                face_indexes.sort_by(|a, b| {
                    let a_face = &mesh.faces[*a];
                    let b_face = &mesh.faces[*b];
                    let a_sum = mesh.vertexes[a_face.v0].y + mesh.vertexes[a_face.v1].y + mesh.vertexes[a_face.v2].y;
                    let b_sum = mesh.vertexes[b_face.v0].y + mesh.vertexes[b_face.v1].y + mesh.vertexes[b_face.v2].y;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            } else {
                face_indexes.sort_by(|a, b| {
                    let a_face = &mesh.faces[*a];
                    let b_face = &mesh.faces[*b];
                    let a_sum = mesh.vertexes[a_face.v0].z + mesh.vertexes[a_face.v1].z + mesh.vertexes[a_face.v2].z;
                    let b_sum = mesh.vertexes[b_face.v0].z + mesh.vertexes[b_face.v1].z + mesh.vertexes[b_face.v2].z;
                    a_sum.partial_cmp(&b_sum).unwrap()
                });
            }

            let mut left_face_indexes = face_indexes.split_off(mid);
            node.children.push(Box::new(BvhNode::from_face_indexes(mesh, face_indexes)));
            node.children.push(Box::new(BvhNode::from_face_indexes(mesh, &mut left_face_indexes)));
        }

        node
    }

    pub fn from_mesh(mesh: &Mesh) -> BvhNode {
        let mut face_indexes: Vec<usize> = (0..mesh.faces.len()).collect();
        BvhNode::from_face_indexes(mesh, &mut face_indexes)
    }

    pub fn intersect(&self, mesh: &Mesh, ray: &Ray, intersection: &mut Intersection) -> bool {
        if !self.aabb.intersect_ray(ray).0 {
            return false;
        }

        let mut any_hit = false;
        if self.children.is_empty() {
            // leaf node
            for face_index in &self.face_indexes {
                let face = &mesh.faces[*face_index];
                if intersect_polygon(&mesh.vertexes[face.v0], &mesh.vertexes[face.v1], &mesh.vertexes[face.v2], ray, intersection) {
                    any_hit = true;
                }
            }
        } else {
            // intermediate node
            for child in &self.children {
                if child.intersect(mesh, ray, intersection) {
                    any_hit = true;
                }
            }
        }

        any_hit
    }
}

pub fn intersect_polygon(v0: &Vector3, v1: &Vector3, v2: &Vector3, ray: &Ray, intersection: &mut Intersection) -> bool {
    let ray_inv = -ray.direction;
    let edge1 = *v1 - *v0;
    let edge2 = *v2 - *v0;
    let denominator = det(&edge1, &edge2, &ray_inv);
    if denominator == 0.0 { return false; }

    let denominator_inv = denominator.recip();
    let d = ray.origin - *v0;

    let u = det(&d, &edge2, &ray_inv) * denominator_inv;
    if u < 0.0 || u > 1.0 { return false; }

    let v = det(&edge1, &d, &ray_inv) * denominator_inv;
    if v < 0.0 || u + v > 1.0 { return false; };

    let t = det(&edge1, &edge2, &d) * denominator_inv;
    if t < 0.0 || t > intersection.distance { return false; }

    intersection.position = ray.origin + ray.direction * t;
    intersection.normal = edge1.cross(&edge2).normalize();
    intersection.distance = t;
    intersection.uv = Vector2::new(u, v);
    true
}
