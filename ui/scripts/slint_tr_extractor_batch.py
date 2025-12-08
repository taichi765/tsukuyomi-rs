#!/usr/bin/env python
"""Run `slint-tr-extractor` on every file in a directory.

Usage examples:
  # run on all .slint files in a directory (non-recursive)
  ./scripts/slint_tr_extractor_batch.py ui

  # recursive, match any file
  ./scripts/slint_tr_extractor_batch.py ui -r -e ""

This script is intentionally small and dependency-free.
"""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from typing import Iterable, Optional


def find_files(directory: str, recursive: bool, ext: Optional[str]) -> Iterable[str]:
    directory = os.path.abspath(directory)
    if recursive:
        for root, _, files in os.walk(directory):
            for f in files:
                if ext is None or f.endswith(ext):
                    yield os.path.join(root, f)
    else:
        with os.scandir(directory) as it:
            for entry in it:
                if entry.is_file():
                    if ext is None or entry.name.endswith(ext):
                        yield entry.path


def run_on_files(files: Iterable[str], extractor_cmd: str) -> int:
    failures = 0
    for path in files:
        print(f"Running: {extractor_cmd} {path} -j")
        # run and capture output to show errors inline
        proc = subprocess.run(
            [extractor_cmd, path, "-j"],  # join messages with exsisting file
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        if proc.stdout:
            print(proc.stdout, end="")
        if proc.returncode != 0:
            print(
                f"ERROR: command exited {proc.returncode} for {path}", file=sys.stderr
            )
            if proc.stderr:
                print(proc.stderr, file=sys.stderr)
            failures += 1
    return failures


def main() -> int:
    p = argparse.ArgumentParser(
        description="Run slint-tr-extractor for each file in a directory"
    )
    p.add_argument("directory", help="Directory containing files to process")
    p.add_argument(
        "-r", "--recursive", action="store_true", help="Recurse into subdirectories"
    )
    p.add_argument(
        "-e",
        "--ext",
        default=".slint",
        help="File extension to match (default: .slint). Use empty string to match all files.",
    )
    args = p.parse_args()

    directory = args.directory
    if not os.path.isdir(directory):
        print(f"Error: not a directory: {directory}", file=sys.stderr)
        return 2

    # interpret empty string as no filtering
    ext = args.ext if args.ext != "" else None

    extractor = shutil.which("slint-tr-extractor")
    if extractor is None:
        print(
            "Warning: 'slint-tr-extractor' not found in PATH. Attempting to run anyway.",
            file=sys.stderr,
        )
        extractor = "slint-tr-extractor"

    files = list(find_files(directory, args.recursive, ext))
    if not files:
        print("No matching files found.")
        return 0

    failures = run_on_files(files, extractor)
    if failures:
        print(f"Completed with {failures} failures.", file=sys.stderr)
        return 1
    print("All files processed successfully.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
