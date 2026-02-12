"""Verset - Universal changeset management for all languages."""

import importlib.metadata

try:
    # Read the version from the installed package metadata
    __version__ = importlib.metadata.version("verset")
except importlib.metadata.PackageNotFoundError:
    # This block is executed if the package is not installed,
    # e.g., when running directly from source without installation.
    # You can set a fallback version, or attempt to read from pyproject.toml,
    # or simply set it to a placeholder.
    __version__ = "0.0.0.dev0"  # Placeholder for development environments