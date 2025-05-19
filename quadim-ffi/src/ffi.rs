//! C ABI 层：把 quadim 封装成一个简单的 RGBA 流处理函数
use image::ImageBuffer;
use std::slice;
use quadim_core::{
    analyze, render, GenericParams, AnalyzeParams, RenderParams,
    MergeMethod, SampleType, Brush, ClassicBrush, PixelType, ImageType,
};

/// 处理一帧 RGBA u8 数据，in-out 原地处理
#[no_mangle]
pub extern "C" fn quadim_process_rgba_u8(
    data: *mut u8,
    width: u32,
    height: u32,
    buffer_size: usize,
    ratio_w: u8,
    ratio_h: u8,
    max_depth: u8,
    thres_ay: u8,
    thres_cbcr: u8,
    merge_method: u32,
    shape: u32,
) -> i32 {
    // 安全检查
    let pixels = (width as usize) * (height as usize) * 4;
    if buffer_size < (width as usize) * (height as usize) || pixels == 0 {
        return -1;
    }
    unsafe {
        let slice = slice::from_raw_parts_mut(data, pixels);
        let mut img: ImageType =
            ImageBuffer::from_raw(width, height, slice.to_vec()).unwrap();
        let mut canvas = vec![(0u8, SampleType::zeros()); buffer_size];

        // 参数封装
        let gp = GenericParams {
            slicing_ratio: (ratio_w, ratio_h),
            max_depth,
        };
        let ap = AnalyzeParams {
            thres_ay: thres_ay as f32,
            thres_cbcr: thres_cbcr as f32,
            merge_method: if merge_method == 1 {
                MergeMethod::Range
            } else {
                MergeMethod::StDev
            },
        };

        // 分析
        if analyze(&img, &mut canvas, gp, ap).is_err() {
            return -2;
        }

        // 渲染
        let brush: Box<dyn Brush> = {
            let brush_enum: ClassicBrush = (shape as u8).into();
            Box::new(brush_enum)
        };
        let rp = RenderParams {
            bg_color: PixelType::from([255, 255, 255, 255]),
            stroke_color: PixelType::from([0, 0, 0, 255]),
            stroke_width: 1,
            seed: 0,
        };

        // 参数顺序：image, canvas, brush, generic, render params, elapsed time
        if render(&mut img, &canvas, brush, gp, rp, 0.0).is_err() {
            return -3;
        }

        // 写回到原 slice
        let out = img.into_raw();
        slice.copy_from_slice(&out);
    }
    0
}

/// 简化版接口：只传 data/width/height/size，其他都用默认 CLI 参数
#[no_mangle]
pub extern "C" fn quadim_process_rgba_u8_default(
    data: *mut u8,
    width: u32,
    height: u32,
    buffer_size: usize,
) -> i32 {
    // 这里的默认值可以根据你的 CLI  `quadim 1.png` 时使用的参数来填
    const RATIO_W: u8 = 1;
    const RATIO_H: u8 = 1;
    const MAX_DEPTH: u8 = 5;
    const THRES_AY: u8 = 10;
    const THRES_CBCR: u8 = 10;
    const MERGE_METHOD: u32 = 1;
    const SHAPE: u32 = 0;

    quadim_process_rgba_u8(
        data, width, height, buffer_size,
        RATIO_W, RATIO_H, MAX_DEPTH,
        THRES_AY, THRES_CBCR, MERGE_METHOD, SHAPE,
    )
}