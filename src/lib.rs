//! `grayscale`, `sepia` and `invert` filters over every image in scope. Alpha is kept.

use clik_plugin::v1;

#[cfg(target_family = "wasm")]
mod guest {
    use clik_plugin::{Output, Plugin, v1};

    struct ColorFilters;

    impl Plugin for ColorFilters {
        fn run(step: &str, _args: &[String], input: v1::Input) -> Result<Output, String> {
            let op = super::op(step).ok_or_else(|| format!("unknown step: {step}"))?;
            Ok(Output::Filter(super::apply(
                input,
                op,
                clik_plugin::progress,
            )))
        }

        fn render(kind: &str, _input: v1::RenderInput) -> Result<v1::RenderOutput, String> {
            Err(format!("unknown shape kind: {kind}"))
        }
    }

    clik_plugin::export!(ColorFilters);
}

/// A per-pixel operation on one RGBA8 pixel.
type PixelOp = fn(&mut [u8; 4]);

/// The operation for a step name.
fn op(step: &str) -> Option<PixelOp> {
    match step {
        "grayscale" => Some(grayscale),
        "sepia" => Some(sepia),
        "invert" => Some(invert),
        _ => None,
    }
}

/// Runs `op` on every pixel of every image in place, calling `progress` after each image.
/// Images keep their id and size, so clik writes them back over the visible crop.
fn apply(mut input: v1::Input, op: PixelOp, mut progress: impl FnMut(f32)) -> v1::FilterOutput {
    let count = input.images.len().max(1) as f32;
    for (done, image) in input.images.iter_mut().enumerate() {
        image.rgba.as_chunks_mut::<4>().0.iter_mut().for_each(op);
        progress((done + 1) as f32 / count);
    }
    v1::FilterOutput {
        shapes: input.shapes,
        images: input.images,
    }
}

/// Rec. 601 luma.
fn grayscale(px: &mut [u8; 4]) {
    let [r, g, b, _] = px.map(u32::from);
    let y = ((299 * r + 587 * g + 114 * b + 500) / 1000) as u8;
    px[..3].fill(y);
}

/// The classic sepia matrix, clamped to 255.
fn sepia(px: &mut [u8; 4]) {
    let [r, g, b, _] = px.map(f32::from);
    let mix = |kr: f32, kg: f32, kb: f32| (kr * r + kg * g + kb * b).round().min(255.0) as u8;
    px[0] = mix(0.393, 0.769, 0.189);
    px[1] = mix(0.349, 0.686, 0.168);
    px[2] = mix(0.272, 0.534, 0.131);
}

fn invert(px: &mut [u8; 4]) {
    px[..3].iter_mut().for_each(|c| *c = 255 - *c);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clik_plugin::testing;

    fn first_pixel(step: &str, rgba: [u8; 4]) -> Option<[u8; 4]> {
        let out = apply(testing::input_with_image(2, 2, rgba), op(step)?, |_| {});
        let px = out.images.first()?.rgba.get(..4)?;
        Some([px[0], px[1], px[2], px[3]])
    }

    #[test]
    fn grayscale_keeps_alpha() {
        assert_eq!(
            first_pixel("grayscale", [255, 0, 0, 128]),
            Some([76, 76, 76, 128])
        );
        assert_eq!(
            first_pixel("grayscale", [255, 255, 255, 255]),
            Some([255, 255, 255, 255])
        );
    }

    #[test]
    fn sepia_tints_and_clamps() {
        assert_eq!(
            first_pixel("sepia", [100, 100, 100, 7]),
            Some([135, 120, 94, 7])
        );
        assert_eq!(
            first_pixel("sepia", [255, 255, 255, 255]),
            Some([255, 255, 239, 255])
        );
    }

    #[test]
    fn invert_keeps_alpha() {
        assert_eq!(
            first_pixel("invert", [10, 20, 30, 40]),
            Some([245, 235, 225, 40])
        );
    }

    #[test]
    fn keeps_image_id_size_and_shapes() {
        let input = testing::input_with_image(3, 2, [1, 2, 3, 4]);
        let shapes = input.shapes.clone();
        let mut calls = Vec::new();
        let out = apply(input, invert, |f| calls.push(f));
        assert_eq!(out.shapes, shapes);
        assert_eq!(out.images.len(), 1);
        assert_eq!(out.images[0].id, testing::IMAGE_ID);
        assert_eq!((out.images[0].width, out.images[0].height), (3, 2));
        assert_eq!(out.images[0].rgba.len(), 3 * 2 * 4);
        assert_eq!(calls, vec![1.0]);
    }

    #[test]
    fn unknown_step_has_no_op() {
        assert!(op("blur").is_none());
    }
}
