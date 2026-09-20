"""离线验证下载脚本：HTTP 错误、校验失败和错误程序均不得替换旧文件。"""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get('FQDT_TEST_BINARY', ROOT / 'target/debug/fqdt')).resolve()


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='fqdt 下载 ')
        self.dir = Path(self.tmp.name)
        self.bin = self.dir / 'bin'
        self.bin.mkdir()
        self.dest = self.dir / '安装 目录' / 'fqdt'
        self.dest.parent.mkdir()
        self.dest.write_text('existing program')
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ['PATH'])
        self.stub('uname', 'import platform\nprint("Darwin" if sys.argv[1] == "-s" else platform.machine())')

    def tearDown(self):
        self.tmp.cleanup()

    def stub(self, name, body):
        path = self.bin / name
        path.write_text('#!' + sys.executable + '\nimport sys, os, shutil\nfrom pathlib import Path\n' + body + '\n')
        path.chmod(0o755)

    def run_download(self):
        return subprocess.run(['sh', str(ROOT / 'dl.sh'), str(self.dest)], env=self.env,
                              capture_output=True, text=True, timeout=15)

    def assert_preserved(self, result):
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertEqual(self.dest.read_text(), 'existing program')
        self.assertEqual(list(self.dest.parent.glob('.fqdt-download.*')), [])

    def prepare_payload(self, payload, checksum=None):
        source = self.dir / 'payload'
        source.write_bytes(payload)
        self.env['TEST_PAYLOAD'] = str(source)
        self.env['TEST_SHA'] = checksum or hashlib.sha256(payload).hexdigest()
        self.stub('curl', '''out = Path(sys.argv[sys.argv.index('-o') + 1])
if any(arg.endswith('.sha256') for arg in sys.argv):
    out.write_text(os.environ['TEST_SHA'] + '  payload\\n')
else:
    shutil.copyfile(os.environ['TEST_PAYLOAD'], out)''')

    def test_http_error_preserves_existing(self):
        self.stub('curl', 'sys.exit(22)')
        self.assert_preserved(self.run_download())

    def test_wrong_checksum_preserves_existing(self):
        self.prepare_payload(b'Not Found', '0' * 64)
        self.assert_preserved(self.run_download())

    def test_error_page_with_valid_checksum_is_rejected(self):
        self.prepare_payload(b'Not Found')
        self.assert_preserved(self.run_download())

    def test_missing_checksum_preserves_existing(self):
        self.stub('curl', '''if any(arg.endswith('.sha256') for arg in sys.argv):
    sys.exit(22)
Path(sys.argv[sys.argv.index('-o') + 1]).write_bytes(b'payload')''')
        self.assert_preserved(self.run_download())

    def test_native_binary_installs(self):
        self.assertTrue(BINARY.exists(), '先运行 cargo test')
        self.prepare_payload(BINARY.read_bytes())
        result = self.run_download()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.dest.read_bytes(), BINARY.read_bytes())
        self.assertTrue(os.access(self.dest, os.X_OK))
        self.assertEqual(list(self.dest.parent.glob('.fqdt-download.*')), [])

    def test_wrong_architecture_is_rejected(self):
        self.prepare_payload(BINARY.read_bytes())
        self.stub('uname', 'import platform\nprint("Darwin" if sys.argv[1] == "-s" else ("x86_64" if platform.machine() == "arm64" else "arm64"))')
        self.assert_preserved(self.run_download())


if __name__ == '__main__':
    unittest.main()
