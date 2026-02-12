#!/usr/bin/env python3
"""Main entry point for verset."""

import os
import sys
import subprocess
import platform
import sysconfig
from pathlib import Path

def find_verset_binary():
    """Find the verset binary in the package installation."""
    binary_name = "verset.exe" if platform.system() == "Windows" else "verset"
    
    # First, try to find it in the scripts directory (where it should be installed)
    scripts_dir = Path(sysconfig.get_path('scripts'))
    scripts_binary = scripts_dir / binary_name
    if scripts_binary.exists() and os.access(scripts_binary, os.X_OK):
        return str(scripts_binary)
    
    # Also check common locations in case of non-standard installations
    paths = [
        # Check relative to this file (for development)
        Path(__file__).parent.parent / "bin" / binary_name,
        # Check in PATH
        *[Path(p) / binary_name for p in os.environ.get("PATH", "").split(os.pathsep)],
        # Common locations
        Path.home() / ".local" / "bin" / binary_name,
        Path("/usr/local/bin") / binary_name,
        Path("/usr/bin") / binary_name,
    ]
    
    for path in paths:
        if path.exists() and os.access(path, os.X_OK):
            return str(path)
    
    return None

def main():
    """Run verset with the provided arguments."""
    binary = find_verset_binary()
    
    if not binary:
        print("Error: verset binary not found!", file=sys.stderr)
        print("This may indicate an incomplete installation.", file=sys.stderr)
        print("Please try reinstalling the package.", file=sys.stderr)
        sys.exit(1)
    
    # Pass through all arguments
    try:
        result = subprocess.run([binary] + sys.argv[1:], check=False)
        sys.exit(result.returncode)
    except KeyboardInterrupt:
        sys.exit(130)
    except Exception as e:
        print(f"Error running verset: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main() 