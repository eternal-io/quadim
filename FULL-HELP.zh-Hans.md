# `quadim --help`

```
迄今为止最快的图像四叉树风格化实现，拥有上百FPS的速度并且能够避免丑陋的长方形。

Usage: quadim.exe [OPTIONS] <IMAGE_OR_DIR>

Arguments:
  <IMAGE_OR_DIR>
          要处理的图像，或者是要处理一个目录里的所有图像。（不包括子目录）

          注意，目前不允许从Stdio读写。

Options:
  -o, --output <IMAGE_OR_DIR>
          手动指定，或是留空来自动生成（基于时间命名的）目标文件。

          注意，目前不允许从Stdio读写。

  -r, --ratio <SLICING_RATIO>
          指示如何将图像划分为子块以避免丑陋的长方形。

          [默认值: 1:1]

  -d, --depth <MAX_DEPTH>
          四叉树的最大深度（这项参数不会导致时间复杂度爆炸（￣︶￣）↗　）

          [默认值: 8]

  -Y, --thres-ay <THRES_AY>
          Alpha 和 Luma 通道上的阈值。

          - 当前默认值是对标准差设置的。
          - 值越大，丢失的细节越多（图像也就越小）。
          - 这个参数对性能的影响微弱。

          [默认值: 20]

  -C, --thres-cbcr <THRES_CBCR>
          其它两个色度通道（Cb/Cr）上的阈值。注意！合并测试时顺序执行的！也就是说只有通过了 Alpha 和 Luma 测试之后才会轮到它！

          - 当前默认值是对标准差设置的。
          - 值越大，丢失的细节越多（图像也就越小）。
          - 这个参数对性能的影响微弱。

          [默认值: 2]

      --merge <MERGE_METHOD>
          指定合并测试所用的算法。

          “st-dev”指的是“标准差”。这个方法通常能产生更“抽象”的图像。

          “range”指的是“极差”。可能更快更详细，但是产生的图像也会稍大。

          [默认值: st-dev]
          [允许的值: st-dev, range]

  -s, --shape <BRUSH>
          指定画笔形状。

          [默认值: rect]
          [允许的值: rect, circle, cross]

  -B, --bg-color <BG_COLOR>
          指定背景颜色（如果需要）。

          [默认值: white]

  -S, --stroke-color <STROKE_COLOR>
          指定描边颜色。（只有描边粗细大于零时有效）

          [默认值: black]

  -W, --stroke-width <STROKE_WIDTH>
          指定描边粗细。

          [默认值: 0]

  -P, --parallel <PARALLELISM>
          指定线程数。默认值是CPU的逻辑核心数。

      --buffer <BUFFER_SIZE>
          指定缓存大小。如果遇到了 `ImageTooLarge` 之类的错误，请尝试增大这个值。

          [默认值: 33177600]

      --errors <MAX_ERRORS>
          错误计数。当有这么多错误发生时，立即提前终止处理。

          [默认值: 5]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
