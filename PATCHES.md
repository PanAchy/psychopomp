# Local divergence from upstream

This fork carries changes made for one setup: an Intel UHD laptop running Arch/Hyprland, a local
Kyutai Pocket TTS server for narration, and a preference for cheap renders over motion-blur
fidelity. Upstream is `kitlangton/psychopomp`, MIT licensed.

## What differs

**`media`: a local Pocket TTS backend.** `Backend::Pocket` sits beside ElevenLabs, Fish, and `say`.
`Voice::pocket(name)` posts the line to `POCKET_TTS_URL` (default `http://127.0.0.1:8000/tts`), stores
the WAV, and excludes local speech from every billing surface through one `paid()` predicate.

Two constraints drove the shape. Pocket returns no word timings, and the crate's accurate fallback
runs `uvx mlx-whisper`, which is Apple-only. Pocket therefore times words with `words::estimate`
clamped to the real ffmpeg duration. That is proportional, not measured: `Spoken::at("word")` works
but drifts inside a long line. Fixing it means a Linux aligner, and the aligner name should join the
media key when it lands.

**`render`: one shutter sample by default.** Upstream spends 8 samples per frame, 16 during
entrances. On this hardware that is 3.28 s of render per second of film; one sample is 0.46 s. Full
against one sample changes under 0.4 % of pixels on a text-and-diagram lecture, measured frame by
frame. `--draft` is gone because it is now the default; `--samples N` remains for scenes with whip
pans or explosions.

**`render`: 960×540 was tried and reverted.** Scene coordinates, primitive radii, and the fragment
SDF math all live in framebuffer pixels, so a smaller target crops instead of scaling. A
scale-aware fragment stage is the remaining 4× lever, and it needs a downscale-comparison test
before it can be trusted.

**`scenes`: a Pocket narration probe and an Effect explainer.** Both are evidence, not product. The
probe proves the local backend on a real artifact; the explainer is a lecture scene that renders a
stepped code diff and a dependency diagram, and exists to judge the animation vocabulary.

## Environment this assumes

- `pocket-tts serve --port 8000 --language english_7m_2026-10` running for narration.
- Plans written to `target/`, then copied beside the scene, because media paths resolve relative to
  the plan file.
- FFmpeg with `libx264`, a working wgpu adapter, and no CUDA.
