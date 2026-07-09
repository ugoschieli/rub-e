import package::ray::Ray;

struct Grid {
    size: vec3<f32>,
}

struct Entry {
    t: f32,
    hit: bool,
}

fn grid_entry(ray: Ray, grid: Grid) -> Entry {
    // Use the classic ray-slab intersection method to determine the ray entry into the voxel grid
    let inv_dir = 1 / ray.dir;
    let t_low = (vec3<f32>(0) - ray.orig) * inv_dir;
    let t_high = (grid.size - ray.orig) * inv_dir;

    let t_close = min(t_low, t_high);
    let t_far = max(t_low, t_high);

    let t_enter = max(max(t_close.x, t_close.y), t_close.z);
    let t_exit = min(min(t_far.x, t_far.y), t_far.z);

    var entry = Entry();
    entry.hit = t_enter <= t_exit;
    entry.t = t_enter;

    return entry;
}
