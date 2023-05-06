# Quadim

![](https://img.shields.io/crates/v/quadim)
![](https://img.shields.io/crates/d/quadim)
![](https://img.shields.io/crates/l/quadim)
![](https://img.shields.io/docsrs/quadim)
![](https://img.shields.io/github/stars/eternal-io/quadim?style=social)

迄今为止最快的图像四叉树风格化实现，拥有上百FPS的速度并且能够避免丑陋的长方形。

## 安装

1. 安装 [rustup](https://www.rust-lang.org/zh-CN/tools/install)。
2. 安装 [MSVC](https://visualstudio.microsoft.com/zh-hans/visual-cpp-build-tools/)。
3. 打开命令行，运行：**`cargo install quadim -F build-bin`**，听着散热器的嗡嗡声，几分钟就好了。

### 为什么不直接提供二进制文件？

- 高情商：为了防止滥用
- 低情商：能力有限 (￣▽￣)"

## 用法 & 展示

0. 不用担心弄乱你的 workspace！*（感谢我的另一个项目 [src-dst-clarifier](https:github.com/eternal-io/src-dst-clarifier)）*

    ``` shell
    > quadim example.jpg    # 将生成类似“example.jpg-A01123-0456-0789.png”名称的文件

    > quadim ./frames       # 将生成类似“frame-A01123-0456-0789”名称的目录
    ```

1. 然后！我！懒得写教程了！自己捣鼓玩吧～ ╰(￣ω￣ｏ)

    （参看：[完整指令列表](./FULL-HELP.zh-Hans.md)）

### `quadim ./img/4~3/ -o ./img/out-4~3/ --ratio 4:3 --stroke-width 2`

<table style="table-layout:fixed;width:100%"><tr>
    <td><img src="./img/4~3/cloud-wandering.jpg" /></td>
    <td><img src="./img/out-4~3/cloud-wandering.png" /></td>
    <td><img src="./img/4~3/parallel-flare.jpg" /></td>
    <td><img src="./img/out-4~3/parallel-flare.png" /></td>
</tr></table>

### `quadim ./img/18~9/ -o ./img/out-18~9/ --ratio 18:9 --depth 3 --stroke-width 30`

<table style="table-layout:fixed;width:100%"><tr>
    <td><img src="./img/18~9/dash-over-night.jpg" /></td>
    <td><img src="./img/out-18~9/dash-over-night.png" /></td>
    <td><img src="./img/18~9/transiting.jpg" /></td>
    <td><img src="./img/out-18~9/transiting.png" /></td>
</tr></table>

### `quadim ./img/3~4/ -o ./img/out-3~4/ --ratio 3:4 --depth 6 --shape circle --bg-color transparent`

<table style="table-layout:fixed;width:100%"><tr>
    <td><img src="./img/3~4/falling-rainbow.jpg" /></td>
    <td><img src="./img/out-3~4/falling-rainbow.png" /></td>
    <td><img src="./img/3~4/initial-caps.jpg" /></td>
    <td><img src="./img/out-3~4/initial-caps.png" /></td>
</tr></table>

<sub>（我拥有这些相片的版权，请不要滥用哦（づ￣3￣）づ╭❤～）</sub>

## 二次开发？在我自己的程序里调用？

当然可以！所有东西都已经包装好了。不过需要注意的是，**目前并不提供稳定性保证**。*（因为我不知道该怎么保证 (○｀ 3′○)）*

[文档](https://docs.rs/quadim)……凑合着看看吧。注意`analyze()`和`render()`这俩函数，它们既是一切。

## 特性列表

- 多线程！迄今为止最快的实现。
- 以 RGBA-8 格式处理图像。
- 合并测试在 YCbCr 而非 RGB 空间。
- 由于没有抗锯齿，因此在提供`--shape rect --border-width N`，`(N > 0)`渲染参数时，实际上只会绘制左侧和上侧的边框。在指定较大的`border_width`以及突兀的`border_color`时会更加明显。
- 对于颜色参数：你可以传入`DarkSlateGray`、`hsla(168, 100%, 50%, 1)`，等等所有CSS里能写的颜色。*（感谢 [csscolorparser](https://github.com/mazznoer/csscolorparser-rs)）*

## 画饼

- 🔥 允许自定义笔刷，比如随时间旋转的十字、随音乐律动的光点、在HSL颜色空间过滤特定颜色！等等。
- 更友好的CLI：允许一次传入多张图片，以及自动探测最合适的切片比例。
- 把分析和渲染完全分离，允许直接存取四叉树二进制格式……

## Quadim 的原理？

0. 使用 [clap](https://github.com/clap-rs/clap) 解析命令行输入；使用 [src-dst-clarifier](https:github.com/eternal-io/src-dst-clarifier) 来处理源到目标文件的映射；使用 [threadpool](https://github.com/rust-threadpool/rust-threadpool) 进行并行处理。

1. Analyze 阶段

    1. 根据`GenericParams::slicing_ratio`将图像划分成一个个子块。通常需要选择正确的比例来让子块保持正方形，比如`-r 16:9`。

    2. 对每个子块，根据`GenericParams::max_depth`深度优先地遍历四叉树。（存在一行代码限制了真正的最大深度，以保证子图像的边长始终大于零像素）

    3. 尝试合并所有子块。

        能合并的情况有两种：

        1. 抵达了最大深度，则这块区域的所有像素始终被合并，计算平均值并缓存。
        2. 检查自身的四个子块**左上角**的四个像素，计算它们的标准差或是极差（根据`AnalyzeParams::merge_method`）并与`AnalyzeParams::thres_`进行比较，若认为波动程度小则允许合并。然后再计算平均值并缓存。

        不能合并的情况有两种：

        3. 情况二的反相。
        4. 子块的子块的子块……不能合并。

    4. 额外的数据结构`[CanvasPixel]`缓存“颜色平均值”和“该不该合并”。

        （像素颜色平均值储存在相当于子块**左上角**的位置，通过右移可以很容易地寻址。`[CanvasPixel]`本身是一维的，但被抽象成与图像相同的大小，这就是为什么图像的像素数不能大于缓存的长度。）

    5. 遍历完成后，四叉树信息已经被完整记录在缓存中了。由于没有任何数据被重复计算，因此 Quadim 十分高效。

2. Render 阶段

    这回是广度优先遍历四叉树了。简单来说就是把“颜色”从四叉树中取出并用“画笔”（trait `Brush`）画到原图上。

    由于新图的大小保证与原图相同，因此可以在原始图像缓冲区上就地操作，不需要额外的内存分配，这是 Quadim 保持高效的另一个要素。

## 已知问题

- 在绘制较大的圆/椭圆时，它会被裁掉一部分。（详见 [image-rs/imageproc#519](https://github.com/image-rs/imageproc/issues/519)）

## 附录：`exit code`含义

0. 成功
1. 部分错误
2. 错误过多，提前终止
3. 致命错误，在处理任何图像前就已经退出；一般是传错参数了
