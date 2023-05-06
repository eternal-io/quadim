#![doc = include_str!("../CRATES.IO-README.md")]

use image::{GenericImageView, ImageBuffer, Rgba};
use nalgebra::{ArrayStorage, Const, Matrix};
use thiserror::Error;

mod analyze;
mod render;
mod util;

use util::Tile;

pub use self::{analyze::*, render::*};

pub type DepthType = u8;
pub type PixelType = Rgba<DepthType>;
pub type ImageType = ImageBuffer<PixelType, Vec<DepthType>>;
pub type CanvasPixel = (u8, SampleType);
pub type CanvasView<'a> = &'a [CanvasPixel];
pub type CanvasViewMut<'a> = &'a mut [CanvasPixel];

const CHANNEL_COUNT: usize = <PixelType as image::Pixel>::CHANNEL_COUNT as usize;

// 格式是 RGBA，用于存储
pub type SampleType =
    Matrix<DepthType, Const<CHANNEL_COUNT>, Const<1>, ArrayStorage<DepthType, CHANNEL_COUNT, 1>>;
// 可能是 RGBA，也可能是 A-YCbCr，用于计算
pub type SampleAltType =
    Matrix<f32, Const<CHANNEL_COUNT>, Const<1>, ArrayStorage<f32, CHANNEL_COUNT, 1>>;

#[allow(non_snake_case, non_upper_case_globals)]
mod FOUR {
    pub const usize: usize = 4;
    pub const f32: f32 = 4.;
}

/// Parameters required by both [`analyze()`] and [`render()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenericParams {
    /// Specifies how to slice the image into sub-blocks.
    pub slicing_ratio: (u8, u8),
    /// The maximum depth of the quadtree.
    ///
    /// (Don't worry about the performance being affected by too large a value ;)
    pub max_depth: u8,
}
