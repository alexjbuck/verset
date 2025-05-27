#!/usr/bin/env python3
"""
Build platform-specific wheels for verset with bundled binaries.

This script creates wheels for different platforms, each containing the
appropriate pre-compiled binary.
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path
import tempfile

# Platform configurations
PLATFORMS = [
    {
        "wheel_platform": "macosx_10_9_x86_64",
        "binary_source": "../../target/x86_64-apple-darwin/release/verset",
        "binary_name": "verset",
    },
    {
        "wheel_platform": "macosx_11_0_arm64",
        "binary_source": "../../target/aarch64-apple-darwin/release/verset",
        "binary_name": "verset",
    },
    {
        "wheel_platform": "manylinux_2_17_x86_64.manylinux2014_x86_64",
        "binary_source": "../../target/x86_64-unknown-linux-gnu/release/verset",
        "binary_name": "verset",
    },
    {
        "wheel_platform": "manylinux_2_17_aarch64.manylinux2014_aarch64",
        "binary_source": "../../target/aarch64-unknown-linux-gnu/release/verset",
        "binary_name": "verset",
    },
    {
        "wheel_platform": "win_amd64",
        "binary_source": "../../target/x86_64-pc-windows-msvc/release/verset.exe",
        "binary_name": "verset.exe",
    },
]


def build_wheel(platform_config, output_dir):
    """Build a wheel for a specific platform."""
    platform = platform_config["wheel_platform"]
    binary_source = Path(platform_config["binary_source"])
    binary_name = platform_config["binary_name"]
    
    print(f"\nBuilding wheel for {platform}...")
    
    # Check if the binary exists
    if not binary_source.exists():
        print(f"  ⚠️  Binary not found at {binary_source}")
        print(f"     Skipping {platform}")
        return False
    
    # Create a temporary directory for this build
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        
        # Copy the package files
        package_dir = temp_path / "verset_build"
        shutil.copytree(".", package_dir, ignore=shutil.ignore_patterns(
            "__pycache__", "*.pyc", "build", "dist", "*.egg-info",
            "bin", "build_wheels.py", ".git"
        ))
        
        # Create bin directory and copy the binary
        bin_dir = package_dir / "bin"
        bin_dir.mkdir(exist_ok=True)
        
        dest_binary = bin_dir / binary_name
        shutil.copy2(binary_source, dest_binary)
        
        # Make sure it's executable (on Unix)
        if binary_name == "verset":
            os.chmod(dest_binary, 0o755)
        
        # Build the wheel
        cmd = [
            sys.executable, "-m", "pip", "wheel",
            "--no-deps",
            "--wheel-dir", str(output_dir),
            str(package_dir)
        ]
        
        # For platform-specific wheels, we need to use specific build options
        if platform != "any":
            # We'll need to rename the wheel after building
            pass
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"  ❌ Build failed:")
            print(result.stderr)
            return False
        
        # Find the built wheel and rename it with the correct platform tag
        built_wheels = list(output_dir.glob("verset-*.whl"))
        if built_wheels:
            wheel = built_wheels[-1]  # Get the most recent wheel
            
            # Parse the wheel filename
            name_parts = wheel.name.split("-")
            if len(name_parts) >= 5:
                # Reconstruct with the correct platform tag
                new_name = f"{name_parts[0]}-{name_parts[1]}-py3-none-{platform}.whl"
                new_path = output_dir / new_name
                
                # Remove any existing wheel with the same name
                if new_path.exists():
                    new_path.unlink()
                
                wheel.rename(new_path)
                print(f"  ✅ Built: {new_name}")
                return True
        
        print(f"  ❌ No wheel file found after build")
        return False


def main():
    """Build all platform-specific wheels."""
    print("Building verset wheels with bundled binaries...")
    
    # Create output directory
    output_dir = Path("dist")
    output_dir.mkdir(exist_ok=True)
    
    # Track successes
    successful_builds = []
    failed_builds = []
    
    # Build each platform wheel
    for platform_config in PLATFORMS:
        if build_wheel(platform_config, output_dir):
            successful_builds.append(platform_config["wheel_platform"])
        else:
            failed_builds.append(platform_config["wheel_platform"])
    
    # Summary
    print("\n" + "="*60)
    print("Build Summary:")
    print(f"  ✅ Successful: {len(successful_builds)}")
    for platform in successful_builds:
        print(f"     - {platform}")
    
    if failed_builds:
        print(f"  ❌ Failed: {len(failed_builds)}")
        for platform in failed_builds:
            print(f"     - {platform}")
    
    print(f"\nWheels are in: {output_dir.absolute()}")
    
    if successful_builds:
        print("\nTo upload to PyPI:")
        print("  python -m twine upload dist/*.whl")


if __name__ == "__main__":
    main() 