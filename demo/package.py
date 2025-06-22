# -*- coding: utf-8 -*-
"""
Demo Rez package for testing LSP commands functionality.
"""

name = "demo_package"

version = "1.0.0"

description = "A demo package for testing Rez LSP extension commands"

authors = ["Rez LSP Team"]

requires = [
    "python-3.7+",
    "cmake-3.16+",
]

build_requires = [
    "gcc-9+",
    "make",
]

private_build_requires = [
    "pytest-6+",
    "coverage",
]

variants = [
    ["platform-linux", "arch-x86_64"],
    ["platform-windows", "arch-AMD64"],
]

tools = [
    "demo_tool",
]

def commands():
    """
    Define environment commands for this package.
    """
    import os
    
    # Add package tools to PATH
    env.PATH.prepend("{this.root}/bin")
    
    # Set package-specific environment variables
    env.DEMO_PACKAGE_ROOT = "{this.root}"
    env.DEMO_PACKAGE_VERSION = "{this.version}"
    
    # Add Python path for package modules
    env.PYTHONPATH.prepend("{this.root}/python")

# Package metadata
uuid = "12345678-1234-5678-9abc-123456789abc"

# Build configuration
build_command = "python {root}/build.py {install}"

# Test configuration  
tests = {
    "unit_tests": "python -m pytest tests/unit",
    "integration_tests": "python -m pytest tests/integration",
}

# Documentation
help = [
    ["Demo Package", "This is a demo package for testing Rez LSP functionality"],
    ["Usage", "rez env demo_package -- demo_tool"],
    ["Testing", "Use the LSP commands to test server functionality"],
]
