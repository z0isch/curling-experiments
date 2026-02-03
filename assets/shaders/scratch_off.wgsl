// Scratch-off reveal shader
// Creates the effect of scratching away a top layer to reveal underneath color

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ScratchOffMaterial {
    top_color: vec4<f32>,
    reveal_color: vec4<f32>,
    progress: f32,
};

@group(2) @binding(0)
var<uniform> material: ScratchOffMaterial;

// Simple hash function for noise
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

// Fractal brownian motion for more organic scratches
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(pos * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value;
}

// Scratch line pattern - creates directional scratches
fn scratch_pattern(uv: vec2<f32>, progress: f32) -> f32 {
    var scratch = 0.0;
    
    // Multiple layers of scratches at different angles and scales
    let angles = array<f32, 6>(0.0, 0.5, 1.0, 1.5, 2.0, 2.5);
    let scales = array<f32, 6>(8.0, 12.0, 6.0, 15.0, 10.0, 7.0);
    
    for (var i = 0; i < 6; i++) {
        let angle = angles[i];
        let scale = scales[i];
        
        // Rotate UV for this scratch layer
        let c = cos(angle);
        let s = sin(angle);
        let rotated = vec2<f32>(
            uv.x * c - uv.y * s,
            uv.x * s + uv.y * c
        );
        
        // Create elongated scratch marks
        let scratch_uv = vec2<f32>(rotated.x * scale, rotated.y * scale * 0.3);
        
        // Use noise to create scratch pattern
        let n = noise(scratch_uv + vec2<f32>(f32(i) * 17.3, f32(i) * 23.7));
        
        // Threshold based on progress - more scratches as progress increases
        // Different thresholds for each layer create staggered reveal
        let layer_threshold = 1.0 - progress + f32(i) * 0.08;
        
        if (n > layer_threshold) {
            // Add scratch intensity with some variation
            let intensity = smoothstep(layer_threshold, layer_threshold + 0.1, n);
            scratch = max(scratch, intensity);
        }
    }
    
    return scratch;
}

// Creates rough, organic edges for scratches
fn scratch_edge_roughness(uv: vec2<f32>, progress: f32) -> f32 {
    let edge_noise = fbm(uv * 20.0);
    return edge_noise * 0.15 * progress;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    
    // Get the scratch pattern
    let scratch = scratch_pattern(uv, material.progress);
    
    // Add some edge roughness to the scratches
    let roughness = scratch_edge_roughness(uv, material.progress);
    
    // Combine scratch pattern with roughness
    let reveal_amount = clamp(scratch + roughness, 0.0, 1.0);
    
    // Mix colors based on reveal amount
    let final_color = mix(material.top_color, material.reveal_color, reveal_amount);
    
    return final_color;
}

