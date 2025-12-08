# Grass Rendering Fix

## Issue
Grass patches were being spawned but were not visible in the world despite:
- Grass material and shader being properly configured
- Grass entities being created with correct components
- Mesh instances being generated

## Root Cause Analysis

### Problem 1: Incorrect Normal Vector Calculation
The primary issue was in the `collect_grass_instances` function in `src/vegetation/mod.rs`. The function was calculating surface normals by transforming vertices to world space first, then computing the cross product:

```rust
// INCORRECT APPROACH
let v0 = transform.transform_point(Vec3::from(positions[tri[0] as usize]));
let v1 = transform.transform_point(Vec3::from(positions[tri[1] as usize]));
let v2 = transform.transform_point(Vec3::from(positions[tri[2] as usize]));

let normal = (v1 - v0).cross(v2 - v0);
let normal_dir = normal.normalize();
```

This caused all normals to point downward (`Vec3(0.0, -1.0, 0.0)`) instead of upward, causing the filter `if normal_dir.y <= 0.25` to reject all potential grass spawn surfaces.

### Problem 2: Incorrect Mesh Handle Access
The code was attempting to access `Mesh3d` incorrectly:

```rust
// WRONG
let Some(chunk_source_mesh) = meshes.get(chunk_mesh) else { ... };
```

`Mesh3d` is a tuple struct containing `Handle<Mesh>`, requiring:

```rust
// CORRECT
let Some(chunk_source_mesh) = meshes.get(&chunk_mesh.0) else { ... };
```

## Solution

### Fix 1: Use Mesh's Stored Normals
Instead of recalculating normals from transformed vertices, use the mesh's pre-calculated normal attributes and transform them properly:

```rust
// Get stored normals from mesh
let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
    Some(VertexAttributeValues::Float32x3(values)) => values,
    _ => {
        warn!("Mesh has no NORMAL attribute");
        return Vec::new();
    }
};

// Use the stored normal and transform it correctly
let normal_local = Vec3::from(normals[tri[0] as usize]);
let normal_world = transform.rotation * normal_local; // Rotation only, not translation
let normal_dir = normal_world.normalize();
```

**Why this works:**
- Voxel chunk meshes store normals as `[0.0, 1.0, 0.0]` for top faces
- Transforming normals requires only rotation, not full point transformation
- This preserves the correct normal direction regardless of chunk position

### Fix 2: Access Mesh Handle Correctly
```rust
let Some(chunk_source_mesh) = meshes.get(&chunk_mesh.0) else {
    continue;
};
```

### Fix 3: Add ViewVisibility Component
Added `ViewVisibility::default()` to spawned grass entities for proper rendering pipeline integration:

```rust
commands.spawn((
    Mesh3d(mesh_handle),
    MeshMaterial3d(material_handle),
    Transform::IDENTITY,
    GlobalTransform::IDENTITY,
    Visibility::Visible,
    InheritedVisibility::VISIBLE,
    ViewVisibility::default(),  // Added
));
```

## Configuration Changes

### Grass Density Settings
Set reasonable production values:
- **Density**: 20 blades per square unit (down from 50 during testing)
- **Max count**: 2000 instances per chunk (down from 5000 during testing)

```rust
let instances = collect_grass_instances(chunk_source_mesh, transform, 20, 2000);
```

### Shader Configuration
Confirmed shader path is correct in `src/vegetation/grass_material.rs`:

```rust
impl Material for GrassMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/grass.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/grass.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }
}
```

Shader location: `assets/shaders/grass.wgsl`

## Debug Process

### Diagnostic Logging Added (Now Removed)
During debugging, comprehensive logging was added to track:
1. Number of chunks being processed
2. Grass instances generated per chunk
3. Triangle rejection reasons (area too small, wrong normal direction)
4. Successful grass patch spawning

Example debug output that confirmed the fix:
```
INFO voxel_builder::vegetation: Chunk IVec3(6, 1, 8) generated 2000 grass instances
INFO voxel_builder::vegetation: Spawning grass patch for chunk IVec3(6, 1, 8) at world origin
```

### Key Debug Findings
Before fix:
```
Triangle rejected: normal_dir = Vec3(0.0, -1.0, 0.0), y = -1
Grass filtering: 0 triangles too small, 540 wrong normal, 0 accepted
Chunk IVec3(7, 1, 7) generated 0 grass instances
```

After fix:
```
Triangle rejected: normal_dir = Vec3(0.0, 0.0, 1.0), y = 0  // Side faces (correct)
Chunk IVec3(6, 1, 8) generated 2000 grass instances         // Success!
```

## System Architecture

### Grass Spawning Pipeline
1. **Setup** (`setup_grass_patch_assets`): Creates blade mesh template and material variations
2. **Attachment** (`attach_procedural_grass_to_chunks`): Runs per frame in Update schedule
   - Queries chunks without `ChunkGrassAttached` marker
   - Skips water surfaces
   - Collects grass instances from upward-facing triangles
   - Builds combined mesh for all instances
   - Spawns single grass entity at world origin per chunk
3. **Animation** (`update_grass_time`): Updates time uniform for wind animation

### Grass Instance Collection Logic
```rust
fn collect_grass_instances(
    mesh: &Mesh,
    transform: &Transform,
    density: u32,
    max_count: usize,
) -> Vec<GrassInstance>
```

**Filters:**
- Triangle area must be > 0.0001 (eliminates degenerate triangles)
- Normal Y component must be > 0.25 (only upward-facing surfaces ~75° from vertical)

**Instance placement:**
- Uses barycentric coordinates for even distribution within triangles
- Blade count per triangle: `(density as f32 * area).ceil()`
- Deterministic random placement via `simple_hash` function

## Result
✅ Grass now renders correctly on all upward-facing chunk surfaces  
✅ Proper wind animation via WGSL shader  
✅ Multiple color variations for visual diversity  
✅ Efficient batched rendering (one mesh per chunk)

## Related Files
- `src/vegetation/mod.rs` - Main grass spawning logic
- `src/vegetation/grass_material.rs` - Material definition and shader reference
- `assets/shaders/grass.wgsl` - Vertex and fragment shaders with wind animation
- `src/voxel/meshing.rs` - Chunk mesh generation with normals

## Future Improvements
- Consider LOD system for distant grass
- Optimize instance count based on camera distance
- Add grass type variations based on biome/height
- Implement grass culling for chunks outside view frustum
