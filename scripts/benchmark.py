#!/usr/bin/env python3
"""Reproducible single-file and Rayon batch benchmark for Konvert."""

import argparse
import os
import shutil
import statistics
import subprocess
import tempfile
import time
from pathlib import Path


def find_konvert_binary(explicit: str | None) -> str:
    if explicit:
        return explicit
    repo_root = Path(__file__).resolve().parent.parent
    for candidate in (
        repo_root / "target" / "release" / "konvert",
        repo_root / "target" / "debug" / "konvert",
    ):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    which_bin = shutil.which("konvert")
    if which_bin:
        return which_bin
    raise SystemExit("Error: konvert binary not found. Run `cargo build --release` first.")


def fmt_bytes(num_bytes: int) -> str:
    if num_bytes >= 1024 * 1024:
        return f"{num_bytes / (1024 * 1024):.1f} MB"
    if num_bytes >= 1024:
        return f"{num_bytes / 1024:.1f} KB"
    return f"{num_bytes} B"


def fmt_ms(seconds: float) -> str:
    if seconds < 1.0:
        return f"{seconds * 1000.0:.0f} ms"
    return f"{seconds:.2f} s"


def time_cmd(cmd: list[str], stdin_bytes: bytes | None = None, runs: int = 3) -> float:
    samples: list[float] = []
    for _ in range(runs):
        t0 = time.perf_counter()
        subprocess.run(
            cmd,
            input=stdin_bytes,
            stdout=subprocess.DEVNULL if stdin_bytes is not None else subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
        )
        samples.append(time.perf_counter() - t0)
    return statistics.median(samples)


def main() -> None:
    parser = argparse.ArgumentParser(description="Benchmark Konvert vs bare CLI tools.")
    parser.add_argument("--bin", help="Path to konvert binary")
    parser.add_argument("--count", type=int, default=50, help="Batch file count (default: 50)")
    parser.add_argument(
        "--jobs",
        type=int,
        default=min(os.cpu_count() or 4, 8),
        help="Parallel Rayon jobs (default: min(cores, 8))",
    )
    parser.add_argument("--size", default="3840x2160", help="Test PNG resolution (default: 3840x2160)")
    args = parser.parse_args()

    konvert = find_konvert_binary(args.bin)
    if not shutil.which("magick"):
        raise SystemExit("Error: ImageMagick (`magick`) is required to synthesize reference image.")

    with tempfile.TemporaryDirectory(prefix="konvert_bench_") as tmp:
        work = Path(tmp)
        ref_png = work / "ref_4k.png"
        subprocess.run(
            [
                "magick",
                "-size",
                args.size,
                "plasma:fractal",
                "-depth",
                "8",
                str(ref_png),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        ref_bytes = ref_png.stat().st_size

        batch_dir = work / "batch"
        batch_dir.mkdir()
        batch_files: list[str] = []
        for idx in range(args.count):
            dst = batch_dir / f"frame_{idx:03d}.png"
            shutil.copyfile(ref_png, dst)
            batch_files.append(str(dst))

        print(f"Binary : {konvert}")
        print(f"Input  : {args.size} PNG ({fmt_bytes(ref_bytes)}), Batch x{args.count}, -j {args.jobs}")
        print()
        print("| Operation | Mode | Output Size | Disk Saved | Bare Tool (Single) | Konvert (Single) | Konvert (Rayon -j {} x{}) |".format(args.jobs, args.count))
        print("|---|---|---|---|---|---|---|")

        # 1. JPEG XL lossless (if cjxl is installed)
        if shutil.which("cjxl"):
            bare_jxl = work / "bare.jxl"
            bare_t = time_cmd(["cjxl", str(ref_png), str(bare_jxl), "-d", "0"])
            kv_single_out = work / "single_jxl"
            kv_single_out.mkdir()
            kv_t = time_cmd(
                [konvert, "--no-notify", "-f", "-l", "-t", "jxl", "-o", str(kv_single_out), str(ref_png)]
            )
            jxl_size = (kv_single_out / "ref_4k_jxl_lossless.jxl").stat().st_size
            kv_batch_out = work / "batch_jxl"
            kv_batch_out.mkdir()
            batch_t = time_cmd(
                [konvert, "--no-notify", "-f", "-l", "-t", "jxl", "-j", str(args.jobs), "-o", str(kv_batch_out), *batch_files],
                runs=1,
            )
            saved = (1.0 - (jxl_size / ref_bytes)) * 100.0
            speedup = (bare_t * args.count) / batch_t
            print(
                f"| JPEG XL (.jxl) | --lossless | {fmt_bytes(jxl_size)} | -{saved:.1f}% | {fmt_ms(bare_t)} (cjxl) | {fmt_ms(kv_t)} | {fmt_ms(batch_t)} (~{speedup:.1f}x vs serial cjxl) |"
            )

        # 2. WebP q=85 (ImageMagick backend)
        bare_webp = work / "bare.webp"
        bare_webp_t = time_cmd(["magick", str(ref_png), "-auto-orient", "-quality", "85", str(bare_webp)])
        kv_webp_single = work / "single_webp"
        kv_webp_single.mkdir()
        kv_webp_t = time_cmd(
            [konvert, "--no-notify", "-f", "-q", "85", "-t", "webp", "-o", str(kv_webp_single), str(ref_png)]
        )
        webp_size = (kv_webp_single / "ref_4k.webp").stat().st_size
        kv_webp_batch = work / "batch_webp"
        kv_webp_batch.mkdir()
        batch_webp_t = time_cmd(
            [konvert, "--no-notify", "-f", "-q", "85", "-t", "webp", "-j", str(args.jobs), "-o", str(kv_webp_batch), *batch_files],
            runs=1,
        )
        webp_saved = (1.0 - (webp_size / ref_bytes)) * 100.0
        webp_speedup = (bare_webp_t * args.count) / batch_webp_t
        print(
            f"| WebP (.webp) | -q 85 | {fmt_bytes(webp_size)} | -{webp_saved:.1f}% | {fmt_ms(bare_webp_t)} (magick) | {fmt_ms(kv_webp_t)} | {fmt_ms(batch_webp_t)} (~{webp_speedup:.1f}x vs serial magick) |"
        )

        # 3. JPEG q=90 (image-rs JpegEncoder backend in Konvert)
        kv_jpg_single = work / "single_jpg"
        kv_jpg_single.mkdir()
        kv_jpg_t = time_cmd(
            [konvert, "--no-notify", "-f", "-q", "90", "-t", "jpg", "-o", str(kv_jpg_single), str(ref_png)]
        )
        jpg_size = (kv_jpg_single / "ref_4k.jpg").stat().st_size
        bare_jpg = work / "bare.jpg"
        bare_jpg_t = time_cmd(["magick", str(ref_png), "-quality", "90", str(bare_jpg)])
        kv_jpg_batch = work / "batch_jpg"
        kv_jpg_batch.mkdir()
        batch_jpg_t = time_cmd(
            [konvert, "--no-notify", "-f", "-q", "90", "-t", "jpg", "-j", str(args.jobs), "-o", str(kv_jpg_batch), *batch_files],
            runs=1,
        )
        jpg_saved = (1.0 - (jpg_size / ref_bytes)) * 100.0
        jpg_speedup = (kv_jpg_t * args.count) / batch_jpg_t
        print(
            f"| JPEG (.jpg) | -q 90 (image-rs) | {fmt_bytes(jpg_size)} | -{jpg_saved:.1f}% | {fmt_ms(bare_jpg_t)} (magick) | {fmt_ms(kv_jpg_t)} (image-rs) | {fmt_ms(batch_jpg_t)} (~{jpg_speedup:.1f}x vs serial) |"
        )

        # 4. Stdin -> Stdout pipe latency on 1080p PNG
        ref_1080 = work / "ref_1080.png"
        subprocess.run(
            ["magick", str(ref_png), "-resize", "1920x1080!", str(ref_1080)],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        pipe_in = ref_1080.read_bytes()
        pipe_t = time_cmd([konvert, "--no-notify", "-t", "webp", "-"], stdin_bytes=pipe_in)
        print(f"| Unix Pipe (-) | Stdin -> Stdout (1080p) | Stream | - | - | {fmt_ms(pipe_t)} latency | - |")


if __name__ == "__main__":
    main()
