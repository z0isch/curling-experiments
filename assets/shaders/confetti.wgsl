// Confetti Shader - Rains down from ceiling (screen-space)

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ConfettiMaterial {
    params: vec4<f32>,
};

@group(2) @binding(0) var<uniform> material: ConfettiMaterial;

// Pseudo-random number generator
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn hash3(p: vec2<f32>) -> vec3<f32> {
    let q = vec3<f32>(dot(p, vec2<f32>(127.1, 311.7)),
                      dot(p, vec2<f32>(269.5, 183.3)),
                      dot(p, vec2<f32>(419.2, 371.9)));
    return fract(sin(q) * 43758.5453);
}

// Rotates a point around the origin
fn rotate(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let cs = cos(angle);
    let sn = sin(angle);
    return vec2<f32>(v.x * cs - v.y * sn, v.x * sn + v.y * cs);
}

// 2D Box distance function
fn sdBox(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Read actual framebuffer resolution from material uniform (accounts for HiDPI scaling)
    let resolution = vec2<f32>(material.params.y, material.params.z);
    
    // Normalize gl_FragCoord to 0..1 range for the screen
    let uv = in.position.xy / resolution;
    
    // Convert to aspect-corrected coordinates for shapes
    let aspect = resolution.x / resolution.y;
    let p = vec2<f32>(uv.x * aspect, uv.y);
    
    var final_color = vec4<f32>(0.0);
    let time = material.params.x;
    
    // Grid parameters for particle cells
    let grid_size = 20.0;
    
    // Create multiple layers of confetti for depth/density
    for(var layer = 0; layer < 3; layer++) {
        let layer_offset = f32(layer) * 123.45;
        let scale = 1.0 + f32(layer) * 0.2; // Different sizes per layer
        
        let grid_uv = p * grid_size / scale;
        let cell_id = floor(grid_uv + vec2<f32>(layer_offset, 0.0));
        let cell_uv = fract(grid_uv) - 0.5; // Center 0,0 in cell
        
        // Random values per cell
        let rand = hash3(cell_id + vec2<f32>(layer_offset, 0.0));
        
        // Time offset so they don't all start at once
        let t_offset = rand.x * 5.0;
        let t = time + t_offset;
        
        // Falling motion
        let fall_speed = 0.2 + rand.y * 0.3;
        // Start from above screen (y > 1.0) and fall down
        // We use repeat to make it loop if needed, but for "one-shot" celebration
        // we can just let it fall. Let's make it loop seamlessly for the effect duration.
        let y_pos = 1.2 - fract(t * fall_speed); 
        
        // Horizontal sway
        let sway_speed = 1.0 + rand.z * 2.0;
        let sway_amount = 0.1 + rand.y * 0.1;
        let x_sway = sin(t * sway_speed + rand.x * 6.28) * sway_amount;
        
        // Current particle center in grid-normalized coords is (0,0) + motion
        // But since we are rendering *inside* the cell, we need to adjust the cell_uv 
        // effectively moving the particle within the cell view.
        // Actually, easier approach: Screen-space particles.
        // Let's iterate cells in screen space? No, that's heavy.
        // Better: Tiled approach.
        // We are on a specific pixel. Which cell is it in?
        // We need to account for the falling motion crossing cell boundaries.
        // Standard trick: Offset UV by time before grid division.
    }
    
    // Let's retry the loop approach but purely screen-space based on the previous working shader concept 
    // but optimized and using screen coords.
    
    // Number of particles - higher density 
    let num_particles = 150;
    
    // Loop through particles
    for (var i = 0; i < num_particles; i++) {
        // Random properties
        let seed = vec2<f32>(f32(i) * 12.34, f32(i) * 56.78);
        let rand = hash3(seed);
        
        // Spawn parameters
        let spawn_delay = rand.x * 0.5; // Spawn over 0.5s
        let active_time = time - spawn_delay;
        
        // Don't draw if waiting to spawn
        if (active_time < 0.0) {
            continue;
        }
        
        // Physics
        let fall_speed = 0.3 + rand.y * 0.4;
        let x_start = rand.z * aspect * 1.5 - 0.25; // Random X across screen width + buffer
        
        let sway_freq = 2.0 + rand.x * 3.0;
        let sway_amp = 0.01 + rand.y * 0.02; // Reduced sway
        let sway = sin(active_time * sway_freq) * sway_amp;
        
        // Fix direction: Start from -0.2 (Top) and go down (Positive Y)
        // Assuming Y=0 is Top on this platform based on "Bottom Up" feedback from "1.2 - t"
        let particle_pos = vec2<f32>(
            x_start + sway,
            -0.2 + active_time * fall_speed 
        );
        
        // Cull if far off screen (bottom)
        if (particle_pos.y > 1.2) {
            continue;
        }
        
        // Rotation
        let rot_speed = (rand.z - 0.5) * 5.0;
        let angle = active_time * rot_speed;
        
        let offset = p - particle_pos;
        let rotated_offset = rotate(offset, angle);
        
        // Shape: Rectangle using smaller sizes
        let size = 0.004 + rand.x * 0.006; // Much smaller
        let aspect_ratio = 1.5 + rand.y; 
        let dim = vec2<f32>(size, size * aspect_ratio);
        
        let d = sdBox(rotated_offset, dim);
        
        // Soft edge
        let alpha = 1.0 - smoothstep(0.0, 0.002, d);
        
        if (alpha > 0.01) {
            // Color Palette
             var color = vec3<f32>(1.0);
            let color_rand = rand.x; // Use x as color seed
            
            if (color_rand < 0.14) {
                color = vec3<f32>(1.0, 0.2, 0.3); // Red
            } else if (color_rand < 0.28) {
                color = vec3<f32>(0.2, 0.5, 1.0); // Blue
            } else if (color_rand < 0.42) {
                color = vec3<f32>(1.0, 0.8, 0.1); // Yellow
            } else if (color_rand < 0.56) {
                color = vec3<f32>(0.3, 0.9, 0.5); // Green
            } else if (color_rand < 0.7) {
                color = vec3<f32>(1.0, 0.4, 0.8); // Pink
            } else if (color_rand < 0.84) {
                color = vec3<f32>(0.6, 0.3, 0.9); // Purple
            } else {
                color = vec3<f32>(1.0, 0.6, 0.2); // Orange
            }
            
            // Simple lighting
            let shade = 0.8 + 0.2 * sin(angle * 3.0);
            
            let particle_color = vec4<f32>(color * shade, alpha);
            
            // Front-to-back blending (simple alpha composition)
            final_color = mix(final_color, particle_color, particle_color.a);
            
            // Optimization: Early exit if pixel is opaque? 
            // Since we loop arbitrary order, better to just accumulate.
            // But we can check if alpha is saturated.
            if (final_color.a >= 0.99) {
                break;
            }
        }
    }
    
    if (final_color.a <= 0.01) {
        discard;
    }
    
    return final_color;
}
