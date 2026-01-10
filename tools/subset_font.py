#!/usr/bin/env python3
import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


RUST_STRING_RE = re.compile(r'"([^"\\]*(?:\\.[^"\\]*)*)"')
RUST_RAW_STRING_RE = re.compile(r'r(#*)"(.*?)"\1', re.DOTALL)
GENERIC_QUOTED_RE = re.compile(r'["\']([^"\']+)["\']')


def extract_chars_from_text(text: str) -> set[str]:
    chars: set[str] = set()
    # Keep common escapes from contributing characters
    text = (
        text.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\"", '"')
        .replace("\\\\", "\\")
    )
    for ch in text:
        if ch == "\0":
            continue
        chars.add(ch)
    return chars


def extract_chars_from_roots(roots: list[Path]) -> set[str]:
    chars: set[str] = set()
    exts = {".rs", ".html", ".css", ".js", ".ts"}

    for root in roots:
        if root.is_file():
            paths = [root]
        else:
            paths = [p for p in root.rglob("*") if p.suffix in exts]

        for path in paths:
            try:
                content = path.read_text(encoding="utf-8")
            except Exception:
                continue

            if path.suffix == ".rs":
                for s in RUST_STRING_RE.findall(content):
                    chars |= extract_chars_from_text(s)
                for _, s in RUST_RAW_STRING_RE.findall(content):
                    chars |= extract_chars_from_text(s)
                continue

            # Generic quoted strings in web assets
            for s in GENERIC_QUOTED_RE.findall(content):
                chars |= extract_chars_from_text(s)

            # Visible-ish text for HTML (between tags) - best effort
            if path.suffix == ".html":
                for s in re.findall(r">([^<]+)<", content):
                    chars |= extract_chars_from_text(s.strip())

    # Always include basic ASCII (menu shortcuts, numbers, etc.)
    for code in range(0x20, 0x7F):
        chars.add(chr(code))

    # Common CJK punctuation used in UI
    for ch in "，。！？：；（）【】《》“”‘’、·—…+-×/":
        chars.add(ch)

    return chars


def fonttools_available(python: str) -> bool:
    try:
        subprocess.check_call(
            [python, "-c", "import fontTools; import fontTools.subset"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return True
    except Exception:
        return False


def main() -> int:
    ap = argparse.ArgumentParser(description="Subset a font based on UI strings used in the repo.")
    ap.add_argument("--input", required=True, help="Path to input font (full font).")
    ap.add_argument("--output", required=True, help="Path to output font (subset).")
    ap.add_argument(
        "--roots",
        action="append",
        default=None,
        help="Root directory/file to scan for strings (repeatable). Default: src and web.",
    )
    ap.add_argument(
        "--ui-src",
        default=None,
        help="(Deprecated) Alias of --roots for backward compatibility.",
    )
    ap.add_argument("--python", default=sys.executable, help="Python executable to run fontTools with.")
    args = ap.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    roots: list[Path] = []
    if args.roots:
        roots.extend(Path(r) for r in args.roots)
    if args.ui_src:
        roots.append(Path(args.ui_src))
    if not roots:
        roots = [Path("src"), Path("web")]

    if not input_path.is_file() or input_path.stat().st_size <= 0:
        print(f"[subset_font] missing input font: {input_path}", file=sys.stderr)
        return 2

    output_path.parent.mkdir(parents=True, exist_ok=True)

    chars = extract_chars_from_roots(roots)
    text = "".join(sorted(chars))
    print(f"[subset_font] extracted {len(chars)} unique chars from: {', '.join(str(r) for r in roots)}")

    if not fonttools_available(args.python):
        print("[subset_font] fontTools not available; copying full font as-is")
        shutil.copyfile(input_path, output_path)
        return 0

    with tempfile.TemporaryDirectory() as td:
        text_file = Path(td) / "glyphs.txt"
        text_file.write_text(text, encoding="utf-8")

        tmp_out = Path(td) / output_path.name

        cmd = [
            args.python,
            "-m",
            "fontTools.subset",
            str(input_path),
            f"--output-file={tmp_out}",
            f"--text-file={text_file}",
            # Keep subset lean; UI doesn't rely on advanced OpenType features.
            "--layout-features=",
            "--notdef-glyph",
            "--notdef-outline",
            "--recommended-glyphs",
            "--no-hinting",
            "--retain-gids",
        ]

        try:
            subprocess.check_call(cmd)
        except subprocess.CalledProcessError as e:
            print(f"[subset_font] fontTools.subset failed ({e}); copying full font as-is")
            shutil.copyfile(input_path, output_path)
            return 0

        if not tmp_out.is_file() or tmp_out.stat().st_size <= 0:
            print("[subset_font] subset output missing; copying full font as-is")
            shutil.copyfile(input_path, output_path)
            return 0

        shutil.copyfile(tmp_out, output_path)

    print(f"[subset_font] wrote subset font: {output_path} ({output_path.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
