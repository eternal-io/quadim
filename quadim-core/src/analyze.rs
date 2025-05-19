use super::*;

/// Parameters required by [`analyze()`].
///
/// Note that tests are performed sequentially.
/// That is, the Chrominances tests will only come after the Alpha and Luma tests (if they have passed).
///
/// - The attempt of quadtree image sub-block merging is carried out in Alpha-YCbCr color space.
/// - The larger the threshold (`thres_`), the more details are lost.
/// - Threshold parameters will only have a weak impact on performance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnalyzeParams {
    pub thres_ay: f32,
    pub thres_cbcr: f32,

    pub merge_method: MergeMethod,
}

/// Specifies the algorithm to use for merging tests.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum MergeMethod {
    /// Range (statistics)
    Range,
    /// Standard deviation
    StDev,
}

impl MergeMethod {
    /// ITU-R BT.709
    fn rgba_to_aycbcr(rgba: SampleType) -> SampleAltType {
        use nalgebra::Matrix3;
        #[rustfmt::skip]
        const TRANS: Matrix3<f32> = Matrix3::new(
            0.2126,     0.7152,     0.0722,
           -0.1146,    -0.3854,     0.5,
            0.5,       -0.4542,    -0.0458,
        );

        let ycbcr = TRANS * rgba.xyz().cast();
        SampleAltType::new(rgba.w as f32, ycbcr.x, ycbcr.y, ycbcr.z)
    }

    fn is_fluctuated(
        colors: [SampleType; FOUR::usize],
        AnalyzeParams {
            thres_ay,
            thres_cbcr,
            merge_method,
        }: AnalyzeParams,
    ) -> bool {
        let chall = [thres_ay, thres_ay, thres_cbcr, thres_cbcr];

        let aycbcrs = Matrix::<
            f32,
            Const<CHANNEL_COUNT>,
            Const<{ FOUR::usize }>,
            ArrayStorage<f32, CHANNEL_COUNT, { FOUR::usize }>,
        >::from_columns(
            colors
                .iter()
                .map(|c| Self::rgba_to_aycbcr(*c))
                .collect::<Vec<_>>()
                .as_slice(),
        );

        match merge_method {
            MergeMethod::Range => chall
                .iter()
                .zip(
                    aycbcrs
                        .row_iter()
                        .map(|ch| ch.max() as f32 - ch.min() as f32),
                )
                .any(|(&chall, range)| range > chall),

            MergeMethod::StDev => chall
                .iter()
                .zip(aycbcrs.column_variance().as_slice().iter())
                .any(|(chall, &var)| var > chall * chall),
        }
    }
}

/// Perform quadtree analysis for a image and store the result in a canvas.
pub fn analyze(
    img: &ImageType,
    canvas: CanvasViewMut,
    ge_params: GenericParams,
    an_params: AnalyzeParams,
) -> Result<(), AnalyzeError> {
    let GenericParams {
        slicing_ratio,
        max_depth,
    } = ge_params;

    let (img_w, img_h) = img.dimensions();
    if img_w * img_h > canvas.len() as u32 {
        return Err(AnalyzeError::ImageTooLarge);
    }

    util::div_grid((img_w, img_h), slicing_ratio, max_depth)
        .into_iter()
        .for_each(
            |Tile {
                 start_at,
                 area_size,
                 real_max_depth,
             }| {
                go_depth(
                    img,
                    canvas,
                    start_at,
                    area_size,
                    an_params,
                    real_max_depth,
                    None,
                );
            },
        );

    Ok(())
}

fn go_depth(
    img: &ImageType,
    canvas: CanvasViewMut,
    start_at: (u32, u32),
    area_size: (u32, u32),
    an_params: AnalyzeParams,
    max_depth: u8,
    now_depth: Option<u8>,
) -> Option<SampleType> {
    let (img_w, _) = img.dimensions();
    let (sx, sy) = start_at;
    let (w, h) = area_size;

    let now_depth = now_depth.unwrap_or(1);

    let avg_color: SampleType = if now_depth < max_depth {
        let colors: [SampleType; FOUR::usize] = match util::div_quad(start_at, area_size)
            .map(|(start_at, area_size)| {
                go_depth(
                    img,
                    canvas,
                    start_at,
                    area_size,
                    an_params,
                    max_depth,
                    Some(now_depth + 1),
                )
            })
            .fuse()
            .filter_map(std::convert::identity)
            .collect::<Vec<_>>()
            .try_into()
            .ok()
        {
            Some(cs) => cs,
            None => return None,
        };

        match MergeMethod::is_fluctuated(colors, an_params) {
            true => return None,
            false => (colors
                .iter()
                .fold(SampleAltType::zeros(), |acc, c| acc + c.cast())
                / FOUR::f32)
                .try_cast()
                .unwrap(),
        }
    } else {
        (img.view(sx, sy, w, h)
            .pixels()
            .fold(SampleAltType::zeros(), |acc, (_, _, p)| {
                acc + SampleType::from(p.0).cast()
            })
            / (w * h) as f32)
            .try_cast()
            .unwrap()
    };

    let (d, c) = &mut canvas[util::pos(img_w, sx, sy)];
    *d = now_depth;
    *c = avg_color;
    Some(avg_color)
}

#[non_exhaustive]
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzeError {
    #[error("the image has more pixels than the canvas' buffer length")]
    ImageTooLarge,
}
