import os
from flask import Flask, request, render_template, jsonify, send_from_directory
from werkzeug.utils import secure_filename
import subprocess

app = Flask(__name__)
UPLOAD_FOLDER = 'uploads'
OUTPUT_FOLDER = 'outputs'
os.makedirs(UPLOAD_FOLDER, exist_ok=True)
os.makedirs(OUTPUT_FOLDER, exist_ok=True)

@app.route('/')
def index():
    return render_template('index.html')
@app.route('/run', methods=['POST'])
def run_quadim():
    image_file = request.files['image_file']
    ratio = request.form['ratio']
    depth = request.form['depth']
    shape = request.form.get('shape', 'rect')
    bg_color = request.form.get('bg_color', 'white')
    stroke_color = request.form.get('stroke_color', 'black')
    stroke_width = request.form.get('stroke_width', '0')

    # 保存上传图像
    filename = secure_filename(image_file.filename)
    input_path = os.path.join(UPLOAD_FOLDER, filename)
    image_file.save(input_path)

    # 输出路径
    output_path = os.path.join(OUTPUT_FOLDER, filename + '-styled.png')
    cmd = ['quadim', input_path, '-o', output_path]

    if ratio:
        cmd.extend(['--ratio', ratio])
    if depth:
        cmd.extend(['--depth', depth])
    if shape:
        cmd.extend(['--shape', shape])
    if bg_color:
        cmd.extend(['--bg-color', bg_color])
    if stroke_color:
        cmd.extend(['--stroke-color', stroke_color])
    if stroke_width and stroke_width != '0':
        cmd.extend(['--stroke-width', stroke_width])

    try:
        subprocess.run(cmd, check=True)
        return jsonify({'success': True, 'output_url': '/output/' + os.path.basename(output_path)})
    except subprocess.CalledProcessError as e:
        return jsonify({'success': False, 'error': str(e)})

@app.route('/output/<filename>')
def output_file(filename):
    return send_from_directory(OUTPUT_FOLDER, filename)
if __name__ == '__main__':
    app.run(debug=True)