"""How the browser checks launch each engine.

Chromium's headless build draws WebGL in software (SwiftShader, about 6
frames a second here), so every step that plays the game crawled. Headless
Chromium cannot use the GPU under WSL (the WebGL context is lost and
screenshots fail), but a windowed Chromium can: ANGLE on OpenGL through
Mesa's d3d12 driver to the Windows GPU, 60 frames a second with drawing and
screenshots intact. Its window goes on an invisible Xvfb display started
here, so nothing shows on screen. Without Xvfb or /dev/dxg (CI) Chromium
runs headless as before; BUILDER_HEADLESS=1 forces that.
"""

import atexit
import os
import shutil
import subprocess
import time

GPU_ARGS = ["--ignore-gpu-blocklist", "--enable-gpu", "--use-angle=gl", "--ozone-platform=x11"]

RENDERER = """() => { const g = document.createElement('canvas').getContext('webgl2');
  if (!g) return 'no WebGL 2';
  const e = g.getExtension('WEBGL_debug_renderer_info');
  return e ? g.getParameter(e.UNMASKED_RENDERER_WEBGL) : g.getParameter(g.RENDERER); }"""

_display = None


def _xvfb():
    """An invisible X display for this run, started once (":N").

    The display is named, not picked with -displayfd: WSL mounts
    /tmp/.X11-unix read-only, so Xvfb only listens on its abstract socket,
    and an X server started on a taken number exits at once."""
    global _display
    if _display is None:
        for n in range(99, 140):
            proc = subprocess.Popen(
                ["Xvfb", f":{n}", "-screen", "0", "1920x1080x24", "-nolisten", "tcp"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            time.sleep(0.5)
            if proc.poll() is None:
                atexit.register(proc.kill)
                _display = f":{n}"
                break
    return _display


def gpu_window():
    """Whether Chromium can draw on the GPU in an invisible window here."""
    forced = os.environ.get("BUILDER_HEADLESS") == "1"
    return os.path.exists("/dev/dxg") and shutil.which("Xvfb") is not None and not forced


def launch(p, engine):
    if engine == "chromium" and gpu_window():
        # the env goes to the browser itself (Playwright's driver started
        # before this); no Wayland, so the window can only go to Xvfb
        env = {k: v for k, v in os.environ.items() if k != "WAYLAND_DISPLAY"}
        env.update(DISPLAY=_xvfb(), GALLIUM_DRIVER="d3d12")
        return p.chromium.launch(channel="chromium", headless=False, args=GPU_ARGS, env=env)
    return getattr(p, engine).launch()


def renderer(page):
    """The WebGL renderer the page got, for the log (GPU or software)."""
    try:
        return page.evaluate(RENDERER)[:80]
    except Exception as e:  # a page without a canvas yet
        return f"unknown ({e.__class__.__name__})"
