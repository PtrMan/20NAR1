# Binding.py
# python binding to use reasoner from python

class Binding(object):
    # /param pathToNar path to the NAR, ex: "/home/r0b3/dev/rust/20mlish6"
    def __init__(self, pathToNar):
        import subprocess

        # universal_newlines because we want text output
        self.proc = subprocess.Popen(["cargo", "run", "--release", "it"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, universal_newlines=True, cwd=pathToNar)

    # input
    def i(self, text):
        self.proc.stdin.write(text+"\n")
    
    # procedural step0
    def ps0(self):
        self.i("!ps0s")

    # procedural step1
    def ps1(self):
        self.i("!ps1s")
    
    def s(self):
        self.i("!s")

# TODO< add receiving from subprocess >
