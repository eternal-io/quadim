# `quadim --help`

```
Fastest image quadtree stylization implementation to date, capable of hundreds of fps and avoiding ugly non-squares.

Usage: quadim.exe [OPTIONS] <IMAGE_OR_DIR>

Arguments:
  <IMAGE_OR_DIR>
          The image to process, or all images in a directory to process.

          Note that IO from Stdio is currently not supported.

Options:
  -o, --output <IMAGE_OR_DIR>
          Leave blank to automatically create a time-based named DST, or specify manually.

          Note that IO from Stdio is currently not supported.

  -r, --ratio <SLICING_RATIO>
          Specifies how to slice the image into sub-blocks.

          ...to avoid ugly non-squares.

          [default: 1:1]

  -d, --depth <MAX_DEPTH>
          The maximum depth of the quadtree.

          (Don't worry about the performance being affected by too large a value ;)

          [default: 8]

  -Y, --thres-ay <THRES_AY>
          Thresholding on Alpha and Luma channels.

          - The default parameters are for the standard deviation.

          - The larger the value, the more details are lost.

          - This parameter will only have a weak impact on performance.

          [default: 20]

  -C, --thres-cbcr <THRES_CBCR>
          Thresholding on the other two Chrominance channels. Notice! The tests are performed sequentially!

          That is, the Chrominances tests will only come after the Alpha and Luma tests (if they have passed).

          - The default parameters are for the standard deviation.

          - The larger the value, the more details are lost.

          - This parameter will only have a weak impact on performance.

          [default: 2]

      --merge <MERGE_METHOD>
          Specifies the algorithm to use for merging tests.

          "st-dev" means "standard deviation". This method usually produces a more "abstract" result.

          "range" may be faster and more detailed, but at the same time produce slightly larger images.

          [default: st-dev]

          Possible values:
          - range:  Range (statistics)
          - st-dev: Standard deviation

  -s, --shape <BRUSH>
          Specifies the shape used to depict each node on the quadtree

          [default: rect]
          [possible values: rect, circle, cross, yr-add, yr-mul]

  -B, --bg-color <BG_COLOR>
          The background color of the fill (if required)

          [default: white]

  -S, --stroke-color <STROKE_COLOR>
          The color of the stroke.

          Only possible when "--stroke-width N" where N greater than zero.

          [default: black]

  -W, --stroke-width <STROKE_WIDTH>
          The width of the stroke

          [default: 0]

      --fps <FRAMERATE>
          Make your brushes change over time!

          [default: 30]

  -P, --parallel <PARALLELISM>
          Specifies the number of threads to use. The default is the number of CPU logical cores

      --buffer <BUFFER_SIZE>
          The size of the buffer. 7680×4320 for single process and 1920×1080 for batch process.

          If there is an error of `ImageTooLarge`, try to increase this value.

      --errors <MAX_ERRORS>
          Error count, when this many errors have occurred, Quadim will terminate early

          [default: 5]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
