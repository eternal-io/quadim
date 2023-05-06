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
        _time_elapsed: f32,
        _now_depth: u8,
        color: PixelType,
    ) {
        use imageproc::{drawing::*, rect::Rect};

        let (sx, sy) = (sx as i32, sy as i32);
        let with_stroke = stroke_width > 0;

        if *self == ClassicBrush::Rect {
            let rect = Rect::at(sx, sy).of_size(w, h);

            draw_filled_rect_mut(img, rect, color);

            if with_stroke {
                draw_filled_rect_mut(
                    img,
                    Rect::at(sx, sy).of_size(w, stroke_width.min(h)),
                    stroke_color,
                );
                draw_filled_rect_mut(
                    img,
                    Rect::at(sx, sy).of_size(stroke_width.min(w), h),
                    stroke_color,
                );
            }
        } else {
            let (rw, rh) = (w as i32 >> 1, h as i32 >> 1);
            match self {
                ClassicBrush::Circle => {
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
                ClassicBrush::Rect => unreachable!(),
            }
        }
    }

    fn need_background(&self) -> bool {
        *self != Self::Rect
    }
}
