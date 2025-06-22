name = "maya"
version = "2024.0"
description = "Autodesk Maya 3D animation and modeling software"
authors = ["Autodesk Inc."]

requires = [
    "python-3.9+",
]

tools = [
    "maya",
    "mayabatch",
    "mayapy",
]

def commands():
    env.PATH.prepend("{root}/bin")
    env.MAYA_LOCATION = "{root}"
    env.PYTHONPATH.prepend("{root}/lib/python3.9/site-packages")
