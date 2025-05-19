from flask import Flask, request, render_template, jsonify
import subprocess

app = Flask(__name__)

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/run', methods=['POST'])
def run_quadim():
    image_path = request.form['image_path']
    output_dir = request.form['output_dir']
    ratio = request.form['ratio']
    depth = request.form['depth']

    cmd = ['quadim', image_path, '-o', output_dir, '--ratio', ratio, '--depth', depth]
    try:
        subprocess.run(cmd, check=True)
        return jsonify({'success': True})
    except subprocess.CalledProcessError as e:
        return jsonify({'success': False, 'error': str(e)})

if __name__ == '__main__':
    app.run(debug=True)
