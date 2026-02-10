from __future__ import annotations

import os
import sys

from zerv._find_zerv import find_zerv_bin

if __name__ == "__main__":
    zerv = find_zerv_bin()
    if sys.platform == "win32":
        import subprocess

        completed_process = subprocess.run([zerv, *sys.argv[1:]])
        sys.exit(completed_process.returncode)
    else:
        os.execvp(zerv, [zerv, *sys.argv[1:]])
