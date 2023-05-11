use super::*;

use std::fmt::Debug;

/// Parameters required by [`render()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderParams {
    /// The background color of the fill (if required).
    pub bg_color: PixelType,
    /// The color of the stroke.
    pub stroke_color: PixelType,
    /// The width of the stroke.
    pub stroke_width: u32,

    /// (reserved)
    ///
    /// 自定义笔刷的随机数种子。
    pub seed: u64,
}

/// Reconstruct the styled image in-place from the canvas from [`analyze()`].
///
/// # 🚧 Panics 🚧
///
/// ***WARNING***: The image and canvas must be exactly the same as those passed to [`analyze()`]!
///
/// Quadim has not done the work of splitting [`analyze()`] and [`render()`], no promises are made here.
///
/// Don't mess around unless you're sure you know exactly what the function does.
pub fn render(
    img: &mut ImageType,
    canvas: CanvasView,
    mut brush: Box<dyn Brush>,
    ge_params: GenericParams,
    re_params: RenderParams,
    time_elapsed: f32,
) -> Result<(), RenderError> {
    let GenericParams {
        slicing_ratio,
        max_depth,
    } = ge_params;

    if brush.need_background() {
        img.pixels_mut().for_each(|p| *p = re_params.bg_color);
    }

    for Tile {
        start_at,
        area_size,
        real_max_depth: _,
    } in util::div_grid(img.dimensions(), slicing_ratio, max_depth)
    {
        go_depth(
            img,
            canvas,
            start_at,
            area_size,
            re_params,
            time_elapsed,
            None,
            &mut brush,
        )?;
    }
    Ok(())
}

fn go_depth(
    img: &mut ImageType,
    canvas: CanvasView,
    start_at: (u32, u32),
    area_size: (u32, u32),
    re_params: RenderParams,
    time_elapsed: f32,
    now_depth: Option<u8>,
    brush: &mut Box<dyn Brush>,
) -> Result<(), RenderError> {
    let (img_w, _) = img.dimensions();
    let (sx, sy) = start_at;

    let now_depth = now_depth.unwrap_or(1);

    let (d, c) = canvas[util::pos(img_w, sx, sy)];
    if now_depth < d {
        for (start_at, area_size) in util::div_quad(start_at, area_size) {
            go_depth(
                img,
                canvas,
                start_at,
                area_size,
                re_params,
                time_elapsed,
                Some(now_depth + 1),
                brush,
            )?;
        }
    } else {
        brush.paint(
            img,
            re_params,
            start_at,
            area_size,
            time_elapsed,
            now_depth,
            PixelType::from(Into::<[DepthType; CHANNEL_COUNT]>::into(c)),
        );
    }

    Ok(())
}

#[non_exhaustive]
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {}

/// Interface for custom brushes.
pub trait Brush: Debug + Send + Sync {
    fn paint(
        &self,
        img: &mut ImageType,
        params: RenderParams,
        start_at: (u32, u32),
        area_size: (u32, u32),
        time_elapsed: f32,
        now_depth: u8,
        color: PixelType,
    );

    fn need_background(&self) -> bool;
}

/// Built-in brush kinds.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ClassicBrush {
    Rect,
    Circle,
    Cross,
    YrAdd,
    YrMul,
}

impl Brush for ClassicBrush {
    fn paint(
        &self,
        img: &mut ImageType,
        RenderParams {
            bg_color: _,
            stroke_color,
            stroke_width,
            seed: _,
        }: RenderParams,
        (sx, sy): (u32, u32),
        (w, h): (u32, u32),
        time_elapsed: f32,
        now_depth: u8,
        color: PixelType,
    ) {
        use imageproc::{drawing::*, rect::Rect};

        let (sx, sy) = (sx as i32, sy as i32);
        let with_stroke = stroke_width > 0;

        fn draw_rect_inner_stroke(
            img: &mut ImageType,
            (sx, sy): (i32, i32),
            (w, h): (u32, u32),
            stroke_width: u32,
            color: PixelType,
        ) {
            if stroke_width >= w.min(h) {
                draw_filled_rect_mut(img, Rect::at(sx, sy).of_size(w, h), color);
                return;
            }
            let se = stroke_width >> 1;
            let ss = se + (stroke_width & 1);

            /*
            ,-----------= sx
            | ,---------= sx + ss
            | |     ,---= sx + w - se
            | |     | ,-= sx + w
            v v     v v
            ,-+--1--+-. = sy
            |-|-----|-| = sy + ss
            |2|     |4|
            |-|-----|-| = sy + h - se
            `-+--3--+-' = sy + h
            */

            draw_filled_rect_mut(img, Rect::at(sx, sy).of_size(w, ss), color);
            draw_filled_rect_mut(img, Rect::at(sx, sy).of_size(ss, h), color);
            if se > 0 {
                draw_filled_rect_mut(
                    img,
                    Rect::at(sx, sy + h as i32 - se as i32).of_size(w, se),
                    color,
                );
                draw_filled_rect_mut(
                    img,
                    Rect::at(sx + w as i32 - se as i32, sy).of_size(se, h),
                    color,
                );
            }
        }

        match self {
            ClassicBrush::Rect => {
                draw_filled_rect_mut(img, Rect::at(sx, sy).of_size(w, h), color);
                if with_stroke {
                    draw_rect_inner_stroke(img, (sx, sy), (w, h), stroke_width, stroke_color);
                }
            }
            ClassicBrush::Circle => {
                let (rw, rh) = (w as i32 >> 1, h as i32 >> 1);
                let center = (sx + rw, sy + rh);
                draw_filled_ellipse_mut(img, center, rw, rh, color);
                if with_stroke {
                    draw_hollow_ellipse_mut(img, center, rw, rh, stroke_color);
                }
            }
            ClassicBrush::Cross => {
                let stroke_width = stroke_width.max(1);
                let (cx, cy) = (
                    sx + (w.saturating_sub(stroke_width) >> 1) as i32,
                    sy + (h.saturating_sub(stroke_width) >> 1) as i32,
                );
                draw_filled_rect_mut(img, Rect::at(sx, cy).of_size(w, stroke_width), color);
                draw_filled_rect_mut(img, Rect::at(cx, sy).of_size(stroke_width, h), color);
            }
            ClassicBrush::YrAdd => {
                let hue = (sx + sy) as f64 / 8. + 360. * time_elapsed.fract() as f64;
                let lit = 0.1
                    + 0.13
                        * match now_depth {
                            n @ 1..=4 => n - 1,
                            _ => 6,
                        } as f64;
                let color: PixelType = csscolorparser::Color::from_hsla(hue, 0.8, lit, 1.0)
                    .to_rgba8()
                    .into();
                draw_rect_inner_stroke(img, (sx, sy), (w, h), stroke_width.max(1), color);
            }
            ClassicBrush::YrMul => match now_depth {
                1..=2 => draw_filled_rect_mut(img, Rect::at(sx, sy).of_size(w, h), color),
                n => {
                    let hue =
                        sx.saturating_mul(sy) as f64 / 20. + 360. * time_elapsed.fract() as f64;
                    let lit = 0.1
                        + 0.12
                            * match n {
                                3 => 2,
                                n => n,
                            } as f64;
                    let color: PixelType = csscolorparser::Color::from_hsla(hue, 0.7, lit, 1.0)
                        .to_rgba8()
                        .into();
                    draw_filled_rect_mut(img, Rect::at(sx, sy).of_size(w, h), color);
                }
            },
        }
    }

    fn need_background(&self) -> bool {
        match self {
            ClassicBrush::Rect => false,
            ClassicBrush::Circle => true,
            ClassicBrush::Cross => true,
            ClassicBrush::YrAdd => true,
            ClassicBrush::YrMul => false,
        }
    }
}
