#!/usr/bin/env python3
"""
docx_to_md.py — Convert one or more .docx files to Markdown, preserving text order.
- Wraps output in a single fenced code block with language inferred from filename.
- Strips the trailing "Version FormulaID ..." suffix from the output filename.
- Requires no internet. Uses python-docx if available; otherwise falls back to parsing DOCX XML.
Usage:
  python docx_to_md.py input1.docx input2.docx -o /path/to/outdir
  python docx_to_md.py --glob "*.docx" -o .
"""
import argparse
import os
import re
import sys
import glob
import zipfile
from datetime import datetime

# Optional dependency; the script will fall back if missing.
try:
    import docx  # type: ignore
    HAS_PYTHON_DOCX = True
except Exception:
    HAS_PYTHON_DOCX = False

def detect_language_from_name(name: str) -> str:
    lower = name.lower()
    if "cargo.toml" in lower or "rust-toolchain.toml" in lower or "config.toml" in lower:
        return "toml"
    if "makefile" in lower:
        return "makefile"
    if ".gitignore" in lower or "gitignore" in lower:
        return "gitignore"
    # Plain fence if we don't recognize a language
    return ""

def sanitize_basename(basename: str) -> str:
    # Remove .docx
    no_ext = re.sub(r'\.docx$', '', basename, flags=re.IGNORECASE)
    # Remove trailing ", Version FormulaID ...)" suffix variants
    no_suffix = re.sub(r',?\s*Version FormulaID.*\)?$', '', no_ext, flags=re.IGNORECASE)
    return no_suffix.strip()

def read_docx_text_python_docx(path: str) -> str:
    """Extract plain text from a .docx using python-docx, preserving paragraph breaks."""
    d = docx.Document(path)  # type: ignore
    parts = []
    # Iterate low-level XML elements to grab text and soft line breaks
    for block in d.element.body.iter():
        tag = block.tag.split('}')[-1]
        if tag == 'p':
            texts = []
            for child in block.iter():
                ctag = child.tag.split('}')[-1]
                if ctag == 't':
                    texts.append(child.text or '')
                elif ctag == 'br':
                    texts.append('\n')
            parts.append(''.join(texts).strip())
        elif tag == 'tbl':
            # Very simple table flatten: collect cell text in order
            for cell in block.iter():
                ctag = cell.tag.split('}')[-1]
                if ctag == 't':
                    parts.append((cell.text or '').strip())
            parts.append('')  # blank line after table
    text = '\n'.join(p for p in parts if p is not None)
    text = re.sub(r'\n{3,}', '\n\n', text).strip()
    return text

def read_docx_text_zipxml(path: str) -> str:
    """Fallback: extract text from DOCX by reading word/document.xml directly."""
    import xml.etree.ElementTree as ET
    with zipfile.ZipFile(path) as z:
        xml_bytes = z.read("word/document.xml")
    root = ET.fromstring(xml_bytes)
    ns = {'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}
    parts = []
    for para in root.findall('.//w:p', ns):
        texts = []
        for node in para.iter():
            tag = node.tag.split('}')[-1]
            if tag == 't':
                texts.append(node.text or '')
            elif tag == 'br':
                texts.append('\n')
        parts.append(''.join(texts).strip())
    text = '\n'.join(parts)
    text = re.sub(r'\n{3,}', '\n\n', text).strip()
    return text

def read_docx_text(path: str) -> str:
    if HAS_PYTHON_DOCX:
        try:
            return read_docx_text_python_docx(path)
        except Exception:
            return read_docx_text_zipxml(path)
    else:
        return read_docx_text_zipxml(path)

def convert_one(input_path: str, outdir: str) -> str:
    base = os.path.basename(input_path)
    lang = detect_language_from_name(base)
    text = read_docx_text(input_path)

    # Build fenced markdown body
    header = f"<!-- Converted from: {base} on {datetime.utcnow().isoformat()}Z -->\n"
    fence = f"```{lang}\n" if lang else "```\n"
    md_body = header + "\n" + fence + text + "\n```\n"

    clean_base = sanitize_basename(base)
    out_name = f"{clean_base}.md"
    out_path = os.path.join(outdir, out_name)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(md_body)
    return out_path

def main(argv=None):
    p = argparse.ArgumentParser(description="Convert .docx files to Markdown with code fences.")
    p.add_argument("inputs", nargs="*", help="Input .docx files")
    p.add_argument("--glob", dest="glob_pattern", default=None, help="Glob pattern for input files, e.g., '*.docx'")
    p.add_argument("-o", "--outdir", default=".", help="Output directory (default: current dir)")
    args = p.parse_args(argv)

    inputs = list(args.inputs)
    if args.glob_pattern:
        inputs.extend(glob.glob(args.glob_pattern))

    # If no inputs specified, abort with hint
    if not inputs:
        p.error("No inputs provided. Specify files or use --glob '*.docx'.")

    # Ensure output dir exists
    os.makedirs(args.outdir, exist_ok=True)

    results = []
    for path in inputs:
        if not os.path.exists(path):
            results.append((path, "MISSING"))
            continue
        if not path.lower().endswith(".docx"):
            results.append((path, "SKIPPED (not .docx)"))
            continue
        try:
            outp = convert_one(path, args.outdir)
            results.append((path, outp))
        except Exception as e:
            results.append((path, f"ERROR: {e}"))

    # Print a simple report
    col_w = max((len(os.path.basename(x[0])) for x in results), default=20)
    print(f"{'Input':<{col_w}}  ->  Output/Status")
    print("-" * (col_w + 18))
    for inp, status in results:
        print(f"{os.path.basename(inp):<{col_w}}  ->  {status}")

if __name__ == "__main__":
    sys.exit(main())
