document.getElementById('quadim-form').onsubmit = async function(e) {
    e.preventDefault();

    const formData = new FormData(e.target);
    const fileInput = document.querySelector('input[name="image_file"]');
    const file = fileInput.files[0];

    // 显示原图
    const originalImg = document.getElementById('original-image');
    originalImg.src = URL.createObjectURL(file);
    originalImg.style.display = 'block';

    const response = await fetch('/run', {
        method: 'POST',
        body: formData
    });
    const result = await response.json();
    const resultDiv = document.getElementById('result');
    const processedImg = document.getElementById('processed-image');

    if (result.success) {
        resultDiv.innerText = "✅ 处理成功！";
        processedImg.src = result.output_url + '?t=' + Date.now(); // 防止缓存
        processedImg.style.display = 'block';
    } else {
        resultDiv.innerText = "❌ 错误：" + result.error;
        processedImg.style.display = 'none';
    }
};

// 风格预设逻辑
document.getElementById('style-select').onchange = function () {
    const style = this.value;
    const form = document.getElementById('quadim-form');

    const presets = {
        'circle-trans': {
            ratio: '1:1',
            depth: '5',
            shape: 'circle',
            bg_color: 'transparent',
            stroke_color: '#444444',
            stroke_width: '1'
        },
        'bw-block': {
            ratio: '1:1',
            depth: '6',
            shape: 'rect',
            bg_color: 'white',
            stroke_color: 'black',
            stroke_width: '2'
        },
        'color-thick': {
            ratio: '3:2',
            depth: '4',
            shape: 'rect',
            bg_color: '#ffffff',
            stroke_color: '#ff0055',
            stroke_width: '10'
        },
        'low-depth': {
            ratio: '16:9',
            depth: '2',
            shape: 'cross',
            bg_color: '',
            stroke_color: '#0033aa',
            stroke_width: '3'
        }
    };

    if (style in presets) {
        const p = presets[style];
        form.ratio.value = p.ratio;
        form.depth.value = p.depth;
        form.shape.value = p.shape;
        form.bg_color.value = p.bg_color;
        form.stroke_color.value = p.stroke_color;
        form.stroke_width.value = p.stroke_width;
    }
};
