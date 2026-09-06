import sys
import subprocess

zig_path = r"C:\Users\nn\Desktop\code\stl-grinder\zig_ext\zig-windows-x86_64-0.13.0\zig.exe"
cmd = [zig_path, "dlltool"] + sys.argv[1:]
sys.exit(subprocess.call(cmd))
