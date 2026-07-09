struct Ray {
    // The ray origin point
    orig: vec3<f32>,
    // The ray direction vector
    dir: vec3<f32>,
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.orig + t * ray.dir;
}
