// https://www.a1k0n.net/2011/07/20/donut-math.html

use minifb::{Scale, Window, WindowOptions};
use std::f32::consts::PI;

fn xy(width: usize, x: usize, y: usize) -> usize {
    y * width + x
}

const GRADIENT: [u32; 12] = [
    // 0x421e0f, // Brown 3
    0x19071a, // Dark violett
    0x09012f, // Darkest blue
    0x040449, // Blue 5
    0x000764, // Blue 4
    // 0x0c2c8a, // Blue 3
    // 0x1852b1, // Blue 2
    0x397dd1, // Blue 1
    0x86b5e5, // Blue 0
    0xd3ecf8, // Lightest blue
    0xf1e9bf, // Lightest yellow
    0xf8c95f, // Light yellow
    0xffaa00, // Dirty yellow
    0xcc8000, // Brown 0
    // 0x995700, // Brown 1
    0x6a3403, // Brown 2
];

const SCREEN_WIDTH: usize = 100;
const SCREEN_HEIGHT: usize = 100;

const THETA_SPACING: f32 = 0.007;
const PHI_SPACING: f32 = 0.002;

// Inner torus radius
const R1: f32 = 1.0;

// Outer torus radius
const R2: f32 = 2.0;

// The distance of the donut from the viewer
const K2: f32 = 5.0;

// Calculate K1 based on screen size: The maximum x-distance occurs roughly at the edge of the
// torus, which is at x=R1+R2, z=0.
//
// We want that to be displaced 3/8ths of the width of the screen, which is 3/4th of the way from
// the center to the side of the screen.
//
// screen_width * 3/8 = K1 * (R1 + R2) / (K2 + 0)
// screen_width * K2 *3 / (8 * (R1 + R2)) = K1
const K1: f32 = SCREEN_WIDTH as f32 * K2 * 3.0 / (8.0 * (R1 + R2));

fn render_frame(a: f32, b: f32, output: &mut [u32], output_xy: fn(usize, usize) -> usize) {
    // Precompute sines and cosines of a and b
    let cos_a: f32 = a.cos();
    let sin_a: f32 = a.sin();
    let cos_b: f32 = b.cos();
    let sin_b: f32 = b.sin();

    let mut zbuffer = vec![0.0; SCREEN_WIDTH * SCREEN_HEIGHT];
    let zbuffer_xy = |x, y| xy(SCREEN_WIDTH, x, y);

    // Theta goes around the cross-sectional circle of a torus
    let mut theta = 0.0;
    while theta < 2.0 * PI {
        // Precompute sines and cosines of theta
        let costheta = theta.cos();
        let sintheta = theta.sin();

        // Phi goes around the center of revolution of a torus
        let mut phi = 0.0;
        while phi < 2.0 * PI {
            // Precompute sines and cosines of phi
            let cosphi = phi.cos();
            let sinphi = phi.sin();

            // The x,y coordinate of the circle, before revolving (factored out of the above
            // equations)
            let circlex = R2 + R1 * costheta;
            let circley = R1 * sintheta;

            // Final 3D (x,y,z) coordinate after rotations, directly from our math above
            let x = circlex * (cos_b * cosphi + sin_a * sin_b * sinphi) - circley * cos_a * sin_b;
            let y = circlex * (sin_b * cosphi - sin_a * cos_b * sinphi) + circley * cos_a * cos_b;
            let z = K2 + cos_a * circlex * sinphi + circley * sin_a;
            let ooz = 1.0 / z; // "one over z"

            // The x and y projection. Note that y is negated here, because y goes up in 3D space
            // but down on 2D displays.
            let xp = (SCREEN_WIDTH as f32 / 2.0 + K1 * ooz * x) as usize;
            let yp = (SCREEN_HEIGHT as f32 / 2.0 - K1 * ooz * y) as usize;

            // Calculate luminance. Ugly, but correct.
            let l = cosphi * costheta * sin_b - cos_a * costheta * sinphi - sin_a * sintheta
                + cos_b * (cos_a * sintheta - costheta * sin_a * sinphi);

            // l ranges from -sqrt(2) to +sqrt(2). If it's < 0, the surface is pointing away from
            // us, so we won't bother trying to plot it.
            if l > 0.0 {
                // Test against the z-buffer. larger 1/z means the pixel is closer to the viewer
                // than what's already plotted.

                if ooz > zbuffer[zbuffer_xy(xp, yp)] as f32 {
                    zbuffer[zbuffer_xy(xp, yp)] = ooz;

                    // Convert the luminance_index into the range 0..11 (8 * sqrt(2) = 11.3)
                    let luminance_index = l * 8.0;

                    // Now we lookup the color corresponding to the luminance and plot it in
                    // our output:
                    output[output_xy(xp, yp)] = GRADIENT[luminance_index as usize];
                }
            }

            phi += PHI_SPACING;
        }

        theta += THETA_SPACING;
    }
}

fn main() {
    let mut window = Window::new(
        "Donut",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60fps
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut output = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];
    let output_xy = |x, y| xy(SCREEN_WIDTH, x, y);

    let mut a = 0.0;
    let mut b = 0.0;

    while window.is_open() {
        render_frame(a, b, &mut output, output_xy);

        a += 0.007;
        b += 0.003;

        window
            .update_with_buffer(&output, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();

        output = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];
    }
}
