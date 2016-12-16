import sys
import subprocess

if len(sys.argv) > 1:
    if sys.argv[1] == "start":
        print("start music")
        subprocess.Popen("rhythmbox \"`find ~/Music/*.mp3 -type f | shuf -n 1`\" &>/dev/null", shell=True)
