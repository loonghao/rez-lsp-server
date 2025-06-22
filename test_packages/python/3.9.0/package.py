name = "python"
version = "3.9.0"
description = "Python programming language interpreter"
authors = ["Python Software Foundation"]

tools = [
    "python",
    "pip",
]

def commands():
    env.PATH.prepend("{root}/bin")
    env.PYTHONPATH.prepend("{root}/lib/python3.9/site-packages")
