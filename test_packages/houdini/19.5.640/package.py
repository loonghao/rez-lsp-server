name = "houdini"
version = "19.5.640"
description = "SideFX Houdini 3D animation and VFX software"
authors = ["Side Effects Software Inc."]

requires = [
    "python-3.9+",
]

tools = [
    "houdini",
    "hython",
    "hbatch",
]

def commands():
    env.PATH.prepend("{root}/bin")
    env.HFS = "{root}"
    env.HOUDINI_PATH.prepend("{root}/houdini")
    env.PYTHONPATH.prepend("{root}/houdini/python3.9libs")
