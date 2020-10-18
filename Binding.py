# Binding.py
# python binding to use reasoner from python
# requires python 3.x

from queue import Queue, Empty

class Binding(object):
    # /param pathToNar path to the NAR, ex: "/home/r0b3/dev/rust/20mlish6"
    def __init__(self, pathToNar):
        import subprocess
        from threading import Thread

        # universal_newlines because we want text output
        self.proc = subprocess.Popen(["cargo", "run", "--release", "it"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, universal_newlines=True, cwd=pathToNar)

        # we need queue for reading
        # see https://stackoverflow.com/a/4896288/388614
        def enqueueOutput(out, queue):
            while True:
                line = out.readline()
                #print("r"+str(line), flush=True)
                queue.put(line)

            #for line in iter(out.readline, b''):
            #    print("r"+str(line), flush=True)
            #    queue.put(line)
            out.close()
        
        self.queue = Queue()
        t = Thread(target=enqueueOutput, args=(self.proc.stdout, self.queue))
        t.daemon = True # thread dies with the program
        t.start()

    # input
    def i(self, text):
        self.proc.stdin.write(text+"\n")
        self.proc.stdin.flush()
    
    # procedural step0
    def ps0(self):
        self.i("!ps0s")

    # procedural step1
    def ps1(self):
        self.i("!ps1s")
    
    def s(self):
        self.i("!s")
    
    # try to read, returns None if nothing was returned
    def tryRead(self):
        # read line without blocking
        try:
            line = self.queue.get_nowait()
        except Empty:
            return None
        else:
            return line
