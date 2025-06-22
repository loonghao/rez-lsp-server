# Test Rez package definition file
# This file is used to test the LSP server functionality

name = "test_package"
version = "1.0.0"
description = "A test package for Rez LSP server"

authors = ["Test Author"]

requires = [
    "python-3.7+",
    "maya-2020+",
    "houdini-18.5+<19",
]

tools = [
    "test_tool",
]

def commands():
    env.PYTHONPATH.append("{root}/python")
    env.PATH.append("{root}/bin")
    env.TEST_PACKAGE_ROOT = "{root}"
