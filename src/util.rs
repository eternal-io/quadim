use super::*;

pub fn pos(img_w: u32, sx: u32, sy: u32) -> usize {
    (img_w * sy + sx) as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    pub start_at: (u32, u32),
    pub area_size: (u32, u32),
    pub real_max_depth: u8,
}

/// "И" order.
pub fn div_grid<'a>(
    (img_w, img_h): (u32, u32),
    (ratio_w, ratio_h): (u8, u8),
    max_depth: u8,
) -> Vec<Tile> {
    fn lancet(full_length: u32, now_step: u8, denom: u8) -> (u32, u32) {
        let start = (full_length as f32 * now_step as f32 / denom as f32) as u32;
        let end = (full_length as f32 * (now_step as f32 + 1.) / denom as f32) as u32;
        (start, end - start)
    }

    let mut tiles = Vec::with_capacity(ratio_w as usize * ratio_h as usize);

    for step_y in 0..ratio_h {
        let (sy, h) = lancet(img_h, step_y, ratio_h);
        if h == 0 {
            continue;
        }
        for step_x in 0..ratio_w {
            let (sx, w) = lancet(img_w, step_x, ratio_w);
            if w == 0 {
                continue;
            }

            tiles.push(Tile {
                start_at: (sx, sy),
                area_size: (w, h),
                real_max_depth: max_depth.min(w.ilog2() as u8).min(h.ilog2() as u8),
            })
        }
    }

    tiles
}

/// "Z" order.
pub fn div_quad(
    start_at: (u32, u32),
    area_size: (u32, u32),
) -> std::array::IntoIter<((u32, u32), (u32, u32)), { FOUR::usize }> {
    let (sx, sy) = start_at;
    let (aw, ah) = area_size;

    let w0 = aw >> 1;
    let h0 = ah >> 1;
    let w1 = w0 + (aw & 1);
    let h1 = h0 + (ah & 1);

    let x0 = sx;
    let x1 = sx + w0;
    let y0 = sy;
    let y1 = sy + h0;

    [
        ((x0, y0), (w0, h0)),
        ((x1, y0), (w1, h0)),
        ((x0, y1), (w0, h1)),
        ((x1, y1), (w1, h1)),
    ]
    .into_iter()
}
