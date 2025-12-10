// Triplanar PBR terrain shader with normal mapping
// Fragment-only shader that uses Bevy's standard vertex outputs

#import bevy_pbr::forward_io::VertexOutput

// Triplanar uniforms - matches TriplanarUniforms struct in Rust
struct TriplanarUniforms {
    base_color: vec4<f32>,     // Base color tint
    tex_scale: f32,            // World units per texture tile (lower = higher res)
    blend_sharpness: f32,      // Controls blend falloff (higher = sharper)
    normal_intensity: f32,     // Normal map strength
    _padding: f32,             // Padding for alignment
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> uniforms: TriplanarUniforms;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var tex_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var normal_texture: texture_2d<f32>;

// Compute tiled UV from world coordinates
fn compute_uv(world_coord: vec2<f32>) -> vec2<f32> {
    let tex_scale = uniforms.tex_scale;
    let scaled = world_coord / tex_scale;
    return fract(scaled);
}

// Calculate triplanar blend weights from world normal
fn triplanar_weights(world_normal: vec3<f32>) -> vec3<f32> {
    let sharpness = uniforms.blend_sharpness;
    var weights = pow(abs(world_normal), vec3(sharpness));
    let weight_sum = weights.x + weights.y + weights.z;
    weights = weights / max(weight_sum, 0.001);
    return weights;
}

// Unpack normal from texture (0-1 range to -1 to 1 range)
fn unpack_normal(sampled: vec3<f32>) -> vec3<f32> {
    return normalize(sampled * 2.0 - 1.0);
}

// Reorient tangent-space normal to world space for a triplanar projection
fn reorient_normal(tangent_normal: vec3<f32>, world_normal: vec3<f32>, axis: i32) -> vec3<f32> {
    var n = tangent_normal;
    let intensity = uniforms.normal_intensity;
    n = vec3(n.xy * intensity, n.z);
    n = normalize(n);
    
    var world_n: vec3<f32>;
    if (axis == 0) {
        // X-facing: right vector is +Z, up is +Y
        world_n = vec3(n.z * sign(world_normal.x), n.y, n.x);
    } else if (axis == 1) {
        // Y-facing (ground): right is +X, up is +Z
        world_n = vec3(n.x, n.z * sign(world_normal.y), n.y);
    } else {
        // Z-facing: right is +X, up is +Y  
        world_n = vec3(n.x, n.y, n.z * sign(world_normal.z));
    }
    
    return normalize(world_n);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_position.xyz;
    let world_normal = normalize(in.world_normal);

    // Calculate blend weights based on normal
    let weights = triplanar_weights(world_normal);

    // Compute UVs for each projection plane
    let uv_yz = compute_uv(world_pos.yz);  // X-facing 
    let uv_xz = compute_uv(world_pos.xz);  // Y-facing (ground)
    let uv_xy = compute_uv(world_pos.xy);  // Z-facing

    // Sample albedo from all 3 projections (using shared sampler)
    let albedo_x = textureSample(color_texture, tex_sampler, uv_yz);
    let albedo_y = textureSample(color_texture, tex_sampler, uv_xz);
    let albedo_z = textureSample(color_texture, tex_sampler, uv_xy);
    
    // Blend albedo
    var albedo = albedo_x * weights.x + albedo_y * weights.y + albedo_z * weights.z;
    albedo = albedo * uniforms.base_color;

    // Sample normal maps from all 3 projections
    let normal_x_raw = textureSample(normal_texture, tex_sampler, uv_yz).rgb;
    let normal_y_raw = textureSample(normal_texture, tex_sampler, uv_xz).rgb;
    let normal_z_raw = textureSample(normal_texture, tex_sampler, uv_xy).rgb;
    
    // Unpack and reorient normals to world space
    let normal_x = reorient_normal(unpack_normal(normal_x_raw), world_normal, 0);
    let normal_y = reorient_normal(unpack_normal(normal_y_raw), world_normal, 1);
    let normal_z = reorient_normal(unpack_normal(normal_z_raw), world_normal, 2);
    
    // Blend world-space normals using weights
    var blended_normal = normal_x * weights.x + normal_y * weights.y + normal_z * weights.z;
    blended_normal = normalize(blended_normal);

    // PBR-style lighting
    let light_dir = normalize(vec3(0.4, 0.8, 0.3));  // Sun direction
    let view_dir = normalize(-in.world_position.xyz);  // Approximate view direction
    let half_dir = normalize(light_dir + view_dir);
    
    // Diffuse (Lambert)
    let ndotl = max(dot(blended_normal, light_dir), 0.0);
    let ambient = 0.35;
    let diffuse = ndotl * 0.65;
    
    // Specular (subtle fixed roughness)
    let ndoth = max(dot(blended_normal, half_dir), 0.0);
    let specular = pow(ndoth, 32.0) * 0.15;
    
    // Combine lighting
    let lit_color = albedo.rgb * (ambient + diffuse) + vec3(specular);

    return vec4(lit_color, albedo.a);
}
