use std::path::PathBuf;

use clap::Parser;

use quadim::*;

// 我只能把文档注释复制来复制去！！ ＞︿＜
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
struct Args {
    /* ----- 路径 ----- */
    /// The image to process, or all images in a directory to process.
    ///
    /// Note that IO from Stdio is currently not supported.
    #[arg(required = true, value_name = "IMAGE_OR_DIR")]
    src: PathBuf,
    /// Leave blank to automatically create a time-based named DST, or specify manually.
    ///
    /// Note that IO from Stdio is currently not supported.
    #[arg(short = 'o', long = "output", value_name = "IMAGE_OR_DIR")]
    dst: Option<PathBuf>,

    /* ----- 通用参数 ----- */
    /// Specifies how to slice the image into sub-blocks.
    ///
    /// ...to avoid ugly non-squares.
    #[arg(short = 'r', long = "ratio", value_parser = Self::parse_ratio, default_value = "1:1")]
    slicing_ratio: (u8, u8),
    /// The maximum depth of the quadtree.
    ///
    /// (Don't worry about the performance being affected by too large a value ;)
    #[arg(short = 'd', long = "depth", value_parser = clap::value_parser!(u8).range(1..), default_value_t = 8)]
    max_depth: u8,

    /* ----- 分析参数 ----- */
    /// Thresholding on Alpha and Luma channels.
    ///
    /// - The default parameters are for the standard deviation.
    ///
    /// - The larger the value, the more details are lost.
    ///
    /// - This parameter will only have a weak impact on performance.
    #[arg(short = 'Y', long, default_value_t = 20.)]
    thres_ay: f32,
    /// Thresholding on the other two Chrominance channels. Notice! The tests are performed sequentially!
    ///
    /// That is, the Chrominances tests will only come after the Alpha and Luma tests (if they have passed).
    ///
    /// - The default parameters are for the standard deviation.
    ///
    /// - The larger the value, the more details are lost.
    ///
    /// - This parameter will only have a weak impact on performance.
    #[arg(short = 'C', long, default_value_t = 2.)]
    thres_cbcr: f32,

    /// Specifies the algorithm to use for merging tests.
    ///
    /// "st-dev" means "standard deviation". This method usually produces a more "abstract" result.
    ///
    /// "range" may be faster and more detailed, but at the same time produce slightly larger images.
    #[arg(long = "merge", default_value = "st-dev")]
    merge_method: MergeMethod,

    /* ----- 渲染参数 ----- */
    /// Specifies the shape used to depict each node on the quadtree.
    #[arg(short = 's', long = "shape", default_value = "rect")]
    brush: ClassicBrush,

    /// The background color of the fill (if required).
    #[arg(short = 'B', long, value_parser = Self::parse_color, default_value = "white")]
    bg_color: PixelType,
    /// The color of the stroke.
    ///
    /// Only possible when "--stroke-width N" where N greater than zero.
    #[arg(short = 'S', long, value_parser = Self::parse_color, default_value = "black")]
    stroke_color: PixelType,
    /// The width of the stroke.
    #[arg(short = 'W', long, default_value_t = 0)]
    stroke_width: u32,

    /// (reserved)
    ///
    /// 自定义笔刷的随机数种子。
    #[arg(hide = true, long, default_value_t = 0)]
    seed: u64,

    /// (reserved)
    ///
    /// 让自定义笔刷可以基于时间变化。
    #[arg(hide = true, long = "fps", value_parser = Self::parse_framerate, default_value_t = 25.)]
    framerate: f32,

    /* ----- 杂项 ----- */
    /// Specifies the number of threads to use. The default is the number of CPU logical cores.
    #[arg(short = 'P', long = "parallel")]
    parallelism: Option<usize>,
    /// The size of the buffer.
    ///
    /// If there is an error of `ImageTooLarge`, try to increase this value.
    #[arg(long = "buffer", default_value_t = 7680 * 4320)]
    buffer_size: usize,
    /// Error count, when this many errors have occurred, Quadim will terminate early.
    #[arg(long = "errors", value_parser = Self::parse_errth, default_value_t = 5)]
    max_errors: usize,
}

impl Args {
    fn parse_ratio(s: &str) -> Result<(u8, u8), &'static str> {
        const MSG: &str =
            "the format of `ratio` must be `W:H` where W and H are both positive integers";
        let p = s.find(":").ok_or(MSG)?;
        let op = |_| MSG;
        Ok((
            s[..p].parse::<u8>().map_err(op)?,
            s[p + 1..].parse::<u8>().map_err(op)?,
        ))
    }

    fn parse_color(s: &str) -> Result<PixelType, csscolorparser::ParseColorError> {
        Ok(PixelType::from(csscolorparser::parse(s)?.to_rgba8()))
    }

    fn parse_framerate(s: &str) -> Result<f32, &'static str> {
        const MSG: &str = "`framerate` must be a float greater than zero";
        s.parse::<f32>().ok().filter(|&f| f > 0.).ok_or(MSG)
    }

    fn parse_errth(s: &str) -> Result<usize, std::num::ParseIntError> {
        Ok(match usize::from_str_radix(s, 10)? {
            0 => usize::MAX,
            n => n,
        })
    }

    fn to_params(&self) -> (GenericParams, AnalyzeParams, RenderParams, Box<dyn Brush>) {
        (
            GenericParams {
                slicing_ratio: self.slicing_ratio,
                max_depth: self.max_depth,
            },
            AnalyzeParams {
                thres_ay: self.thres_ay,
                thres_cbcr: self.thres_cbcr,
                merge_method: self.merge_method,
            },
            RenderParams {
                bg_color: self.bg_color,
                stroke_color: self.stroke_color,
                stroke_width: self.stroke_width,
                seed: self.seed,
            },
            Box::new(self.brush),
        )
    }
}

fn main() {
    use std::{
        error::Error,
        io::{self, Write},
        process::exit,
        sync::{mpsc, Arc},
        time::Instant,
    };

    use object_pool::Pool;
    use threadpool::ThreadPool;

    use src_dst_clarifier::*;

    fn err_cast(e: Box<dyn Error>) -> String {
        if let Some(e) = e.downcast_ref::<image::ImageError>() {
            return format!("(ImageError) {e}");
        } else if let Some(e) = e.downcast_ref::<std::io::Error>() {
            let k = e.kind();
            return format!("({k:?}) {e}");
        } else {
            return format!("({e:?}) {e}");
        }
    }

    let args = Args::parse();

    let mut sdpairs = match || -> Result<SrcDstPairs, Box<dyn Error>> {
        let ps = SrcDstConfig {
            allow_from_stdin: false,
            allow_to_stdout: false,
            auto_tnamed_dst_file: true,
            auto_tnamed_dst_dir: true,
            default_extension: "png".into(),
            allow_inplace: false,
        }
        .parse(&args.src, args.dst.as_ref())??;
        ps.create_tnamed_dir()?;
        Ok(ps)
    }() {
        Ok(ps) => ps,
        Err(e) => {
            eprintln!("FATAL: {}.", err_cast(e.into()));
            exit(3)
        }
    };

    let (tx, rx) = mpsc::channel::<Result<(), Box<dyn Error + Send + Sync>>>();

    let num_threads = match sdpairs.is_batch() {
        false => 1,
        true => match args.parallelism {
            Some(n) => n,
            None => num_cpus::get(),
        },
    };

    let thread_pool = ThreadPool::new(num_threads);
    let canvas_pool = Arc::new(Pool::<Box<[CanvasPixel]>>::new(num_threads, || {
        vec![(0u8, SampleType::zeros()); *&args.buffer_size].into_boxed_slice()
    }));

    fn worker(
        tx: mpsc::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
        (src, dst): (Src, Dst),
        canvas_pool: Arc<Pool<Box<[CanvasPixel]>>>,
        (ge_params, an_params, re_params, brush): (
            GenericParams,
            AnalyzeParams,
            RenderParams,
            Box<dyn Brush>,
        ),
        time_elapsed: f32,
    ) {
        tx.send((|| {
            let src = match src {
                Src::File(p) => p,
                Src::Stdin => unreachable!(),
            };
            let mut dst = match dst {
                Dst::File(p) => p,
                Dst::Stdout => unreachable!(),
            };

            let mut img = Into::<ImageType>::into(image::open(src)?.into_rgba8());

            let mut canvas = canvas_pool.try_pull().unwrap();

            analyze(&img, &mut canvas, ge_params, an_params)?;
            render(&mut img, &canvas, brush, ge_params, re_params, time_elapsed)?;

            dst.set_extension("png");
            img.save(dst)?;

            Ok(())
        })())
        .unwrap();
    }

    let t_started = Instant::now();

    let fps = args.framerate;
    let mut tot = 0f32;

    let err_max = args.max_errors;
    let mut err_ctr = 0usize;
    let mut succ_ctr = 0usize;
    let mut milestone = 1usize;
    const MILESTONE_INTERVAL: usize = 500;

    let mut done = false;

    let mut stderr = io::stderr();
    let mut exit_code = 0i32;

    loop {
        while thread_pool.queued_count() > 0 {}

        match sdpairs.next() {
            Some(sdpair) => {
                let tx = tx.clone();
                let canvas_pool = canvas_pool.clone();
                let params = args.to_params();
                let time_elapsed = tot / fps;
                thread_pool.execute(move || worker(tx, sdpair, canvas_pool, params, time_elapsed));
            }
            None => {
                thread_pool.join();
                done = true;
            }
        }

        tot += 1.;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                Ok(_) => {
                    succ_ctr += 1;
                    eprint!(".");
                }
                Err(e) => {
                    err_ctr += 1;
                    exit_code = 1;
                    eprintln!("\nERROR ({err_ctr}/{err_max}): {}.", err_cast(e));
                }
            }
        }

        if succ_ctr > milestone * MILESTONE_INTERVAL {
            eprintln!(
                "\nINFO: {} images have been processed.",
                milestone * MILESTONE_INTERVAL
            );
            milestone += 1;
        }

        if err_ctr >= err_max {
            exit_code = 2;
            eprintln!("\nFATAL: Too many errors ({err_max}/{err_max}).");
            break;
        }

        if done {
            break;
        }

        stderr.flush().ok();
    }

    let t_finished = Instant::now();
    let t_used = (t_finished - t_started).as_secs_f32();
    eprintln!(
        "\n{} image(s) processed in {:.2}s, average {:.1} fps.",
        succ_ctr,
        t_used,
        succ_ctr as f32 / t_used,
    );

    exit(exit_code)
}
