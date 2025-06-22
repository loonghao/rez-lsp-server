# Sample Rez Package for Testing LSP Server
# This file demonstrates the features of the Rez LSP Extension

name = "test_package"
version = "1.0.0"
description = "A test package for Rez LSP Server demonstration"

# Authors and contact information
authors = [
    "Developer <dev@example.com>"
]

# Package requirements - try typing here to see completion
requires = [
    "python-3.9+",
    "maya-2023",
    "nuke-13.2"
]

# Build requirements
build_requires = [
    "cmake-3.20+",
    "gcc-9+"
]

# Variants - different configurations
variants = [
    ["python-3.9", "maya-2023"],
    ["python-3.9", "nuke-13.2"]
]

# Tools provided by this package
tools = [
    "test_tool",
    "converter"
]

# Package metadata
uuid = "12345678-1234-5678-9abc-123456789abc"
relocatable = True

def commands():
    """
    Environment setup commands.
    This function sets up the package environment.
    """
    import os
    
    # Add Python modules to PYTHONPATH
    env.PYTHONPATH.append("{root}/python")
    
    # Add tools to PATH
    env.PATH.prepend("{root}/bin")
    
    # Set package-specific environment variables
    env.TEST_PACKAGE_ROOT = "{root}"
    env.TEST_PACKAGE_VERSION = "{version}"

def pre_commands():
    """
    Pre-commands for validation.
    """
    # Validate environment
    if not env.get("MAYA_LOCATION"):
        warning("Maya not found in environment")

# Build function
def build(source_path, build_path, install_path):
    """
    Build function for this package.
    """
    import shutil
    import os
    
    # Copy source files
    if os.path.exists(os.path.join(source_path, "python")):
        shutil.copytree(
            os.path.join(source_path, "python"),
            os.path.join(install_path, "python")
        )
