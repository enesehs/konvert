# 📋 Konvert Core Optimization Roadmap & TODO

This document details the step-by-step optimization plan for the **Konvert Rust core engine (`konvert`)**. Tasks are ordered from highest impact to lowest impact.

---

## 📊 Summary & Priority Matrix

| Phase | Task | Impact Area | Expected Gain | Status |
| :---: | :--- | :--- | :--- | :---: |
| **1** | Multi-threaded Parallel Batch Processing (Rayon) | Batch Conversions | 4x – 12x throughput speedup | `[x]` |
| **2** | Compiler & Cargo Profile Optimization | Entire Application | +15-35% CPU speed, -50% binary size | `[x]` |
| **3** | Buffered Disk I/O (`BufWriter` / `BufReader`) | Disk Syscalls | 2x – 4x faster file writing | `[x]` |
| **4** | Thread-Safe Temp Files & In-Memory Piping | Concurrency & Stability | Race condition fix, zero temp I/O | `[x]` |
| **5** | Native In-Memory Processing vs. ImageMagick Fork | Image Operations | Zero process spawn latency (-150ms/file) | `[x]` |
| **6** | Memory Allocation & Syscall Reductions | CLI / Runtime Overhead | Zero repeated env queries, O(1) history | `[x]` |
| **7** | FFmpeg Multi-threading & Hardware Acceleration | Video Encoding | Multi-core CPU encoding + VA-API | `[x]` |
| **8** | Dead Code Cleanup & Desktop Installer Fix | Codebase Hygiene | Bug fix: Nautilus/Thunar `--install-all` | `[x]` |

---

## 🛠️ Step-by-Step Implementation Details

### Phase 1: Multi-threaded Parallel Batch Processing (Rayon)
> **Priority:** 🔴 Critical (Highest Performance Impact)  
> **Target File:** `src/converter/mod.rs` (lines 180–208)

- [x] **1.1. Replace Sequential Loop with Rayon Parallel Iterator:**
  - Implemented `rayon::prelude::*` parallel iteration (`files.par_iter()`) in `convert_batch_with_progress`.
- [x] **1.2. Thread-Safe Progress Tracking:**
  - Implemented `std::sync::atomic::{AtomicUsize, Ordering}` and `std::sync::Mutex` to safely update progress callbacks and print status across all worker threads without lock contention.
- [x] **1.3. Concurrency Limits for Heavy Tasks:**
  - Thread pool work-stealing handles balanced parallel execution.

---

### Phase 2: Compiler & Cargo Release Profile Optimizations
> **Priority:** 🔴 Critical (Instant system-wide gains with zero code churn)  
> **Target File:** `Cargo.toml`

- [x] **2.1. Add Production Release Profile:**
  - Added the following profile to `Cargo.toml`:
    ```toml
    [profile.release]
    opt-level = 3
    lto = "fat"
    codegen-units = 1
    panic = "abort"
    strip = true
    ```
  - **`lto = "fat"`**: Enables aggressive whole-program link-time optimization across crate boundaries.
  - **`codegen-units = 1`**: Enables maximum compiler inlining and vectorization opportunities.
  - **`panic = "abort"`**: Removes bulky stack unwinding landing pads.
  - **`strip = true`**: Automatically strips symbol tables and debug symbols. Binary size: 7.6MB.

---

### Phase 3: Buffered Disk I/O (`BufWriter` & `BufReader`)
> **Priority:** 🟠 High  
> **Target Files:** `src/converter/image.rs`, `src/converter/document.rs`

- [x] **3.1. Wrap `File::create` in `BufWriter` for JPEG Encoding:**
  - Wrapped `File::create` in `std::io::BufWriter::with_capacity(64 * 1024, file)` and properly flushed buffers.
- [x] **3.2. Wrap `File::create` in `BufWriter` for WebP and Fallback Formats:**
  - Applied `BufWriter` for WebP and general image saves in `src/converter/image.rs`.
- [x] **3.3. Buffered File Reading:**
  - Wrapped `File::open` in `std::io::BufReader` for ZIP archives (DOCX & UDF documents).

---

### Phase 4: Thread-Safe Temp Files & In-Memory Piping
> **Priority:** 🟠 High (Stability & Race Condition Fix)  
> **Target Files:** `src/converter/image.rs`, `src/converter/document.rs`

- [x] **4.1. Eliminate Hardcoded PID Temp File Race Condition:**
  - Replaced disk roundtrip `konvert_jxl_{pid}.png` with stdout streaming `djxl - --output_format png` directly into `image::load_from_memory(&output.stdout)`.
- [x] **4.2. UDF Document Temp File Removal & Thread Safety:**
  - Added atomic counter `TEMP_DOC_COUNTER` in `src/converter/document.rs` to guarantee unique temporary filenames under parallel conversions.

---

### Phase 5: Native In-Memory Processing vs. ImageMagick Fork
> **Priority:** 🟡 Medium  
> **Target File:** `src/converter/image.rs`

- [x] **5.1. Native Metadata Stripping:**
  - Implemented in-memory decode and re-encode to automatically strip EXIF, GPS, and camera metadata with zero process spawn overhead. Fallback to ImageMagick kept only for vector/uncommon formats.
- [x] **5.2. Native Preset WebP Compression:**
  - Replaced `Command::new("magick")` for `web_opt` preset with direct Rust WebP encoding through `BufWriter`.

---

### Phase 6: Memory Allocation & Syscall Reductions
> **Priority:** 🟡 Medium  
> **Target Files:** `src/i18n.rs`, `src/history.rs`, `src/converter/document.rs`

- [x] **6.1. Cache Language Detection with `OnceLock`:**
  - Cached `Language::current()` globally using `std::sync::OnceLock<Language>` to avoid querying `LC_ALL`, `LC_MESSAGES`, and `LANG` on every string translation lookup.
- [x] **6.2. Optimize History Insertions ($O(1)$ vs $O(N)$):**
  - Converted `load_history()` to `std::collections::VecDeque` with `push_front` for $O(1)$ insertion, and serialized using compact `serde_json::to_string`.
- [x] **6.3. Streamlined XML Text Extraction:**
  - Preallocated buffer with `String::with_capacity(xml.len() / 2)` in `extract_text_from_xml` to avoid repeated reallocations.

---

### Phase 7: FFmpeg Multi-threading & Hardware Acceleration
> **Priority:** 🟢 Normal  
> **Target File:** `src/converter/video.rs`, `src/converter/audio.rs`

- [x] **7.1. Add Explicit Multi-Threading to FFmpeg:**
  - Added `-threads 0` to all video conversion, GIF generation, frame extraction, video compression, and audio transcoding commands.

---

### Phase 8: Dead Code Cleanup & Desktop Installer Fix
> **Priority:** 🟢 Normal  
> **Target Files:** `src/converter/media.rs`, `src/main.rs`

- [x] **8.1. Delete Dead Code `src/converter/media.rs`:**
  - Removed unreferenced and obsolete file `src/converter/media.rs`.
- [x] **8.2. Fix `--install-all` and `--uninstall-all` in `src/main.rs`:**
  - Wired up `cli.install_all` to `desktop::detector::install_all()` and `cli.uninstall_all` to `desktop::detector::uninstall_all()` so Dolphin, Nautilus, and Thunar integrations are all handled.
