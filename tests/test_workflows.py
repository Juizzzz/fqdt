"""使用本地模拟接口与模拟 TTS 验证工作流，不依赖第三方小说/语音服务。"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
import zipfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get('FQDT_TEST_BINARY', ROOT / 'target/debug/fqdt')).resolve()


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path.startswith('/catalog'):
            body = {'data': {'lists': [{'item_id': '1', 'title': '第一章 中文 空格'}]}}
        elif self.path.startswith('/detail'):
            body = {'data': {'data': {'book_name': '测试书籍', 'author': '测试作者'}}}
        else:
            body = {'data': {'content': '<p>第一段中文正文。</p><p>第二段 &amp; 内容。</p>'}}
        raw = json.dumps(body).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def log_message(self, *args):
        pass


class WorkflowTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='fqdt 工作流 ')
        self.dir = Path(self.tmp.name)
        self.config = self.dir / '配置'
        self.config.mkdir()
        self.temp = self.dir / '临时文件'
        self.temp.mkdir()
        self.env = dict(os.environ, FQDT_CONFIG_DIR=str(self.config),
                        FQDT_CACHE_DIR=str(self.dir / '缓存'), TMPDIR=str(self.temp))

    def tearDown(self):
        self.tmp.cleanup()

    def run_cli(self, *args):
        return subprocess.run([str(BINARY), *args], cwd=self.dir, env=self.env,
                              capture_output=True, text=True, timeout=20)

    def test_download_txt_and_epub(self):
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        base = 'http://127.0.0.1:' + str(server.server_port)
        (self.config / 'config.ini').write_text(
            f'[api]\ncatalog_url = {base}/catalog?book={{}}\n'
            f'content_url = {base}/content?item={{}}\n'
            f'detail_url = {base}/detail?book={{}}\nbatch_url =\n')
        try:
            for fmt in ('txt', 'epub'):
                out = self.dir / ('下载 ' + fmt)
                result = self.run_cli('download', '123', '-t', fmt, '-o', str(out), '-j', '1')
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue((out / 'info.list').exists(), result.stdout + result.stderr)
                if fmt == 'txt':
                    text = next(out.glob('*.txt')).read_text()
                    self.assertIn('第一段中文正文。', text)
                    self.assertIn('第二段 & 内容。', text)
                else:
                    with zipfile.ZipFile(next(out.glob('*.epub'))) as epub:
                        self.assertIsNone(epub.testzip())
                        combined = b'\n'.join(epub.read(n) for n in epub.namelist())
                        self.assertIn('第一段中文正文。'.encode(), combined)
        finally:
            server.shutdown()
            server.server_close()
            worker.join()

    def prepare_tts(self, fail=False):
        bindir = self.dir / '工具'
        bindir.mkdir()
        tool = bindir / 'edge-tts'
        tool.write_text('#!' + sys.executable + '\n' + '''import os, sys
from pathlib import Path
args = sys.argv[1:]
assert '-t' not in args
assert '--rate=-10%' in args
assert '--pitch=-2Hz' in args
source = Path(args[args.index('--file') + 1])
assert source.read_text() == Path(os.environ['TEST_SOURCE']).read_text()
''' + ('sys.exit(2)\n' if fail else '''out = Path(args[args.index('--write-media') + 1])
out.write_bytes(b'X' * 2048)
'''))
        tool.chmod(0o755)
        self.env['PATH'] = str(bindir) + os.pathsep + os.environ['PATH']
        (self.config / 'config.ini').write_text('[tts]\ntts_rate = -10%\ntts_pitch = -2Hz\n')
        source = self.dir / '中文 长篇.txt'
        source.write_text('测试长篇小说正文。\n' * 20000)
        self.env['TEST_SOURCE'] = str(source)
        return source

    def test_long_tts_uses_file_and_cleans_up(self):
        source = self.prepare_tts()
        result = self.run_cli('tts', str(source))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(source.with_suffix('.mp3').exists(), result.stderr)
        self.assertTrue(source.with_suffix('.lrc').exists())
        self.assertEqual(list(self.temp.iterdir()), [])

    def test_tts_failure_cleans_up(self):
        source = self.prepare_tts(fail=True)
        result = self.run_cli('tts', str(source))
        self.assertIn('edge-tts 未生成有效音频', result.stderr)
        self.assertFalse(source.with_suffix('.mp3').exists())
        self.assertEqual(list(self.temp.iterdir()), [])


if __name__ == '__main__':
    unittest.main()
