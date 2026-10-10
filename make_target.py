from pathlib import Path
from PIL import Image, ImageDraw
import numpy as np
from scipy.ndimage import sobel
import struct

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT/'reference.png'
OUT = ROOT
im = Image.open(SOURCE).convert('RGB').crop((0, 0, 532, 560))
a = np.asarray(im).astype(np.float32)
bg = np.array([250, 251, 226], dtype=np.float32)
dist = np.sqrt(((a-bg)**2 * np.array([0.3, 0.45, 0.25])).sum(axis=2))
lum = (a * np.array([0.299, 0.587, 0.114])).sum(axis=2)
edge = np.hypot(sobel(lum, axis=0), sobel(lum, axis=1))
weights = np.clip((dist - 13) / 75, 0, 1) ** 0.75
weights += 0.35 * np.clip(edge / 180, 0, 1)
weights = np.clip(weights, 0, 1.3)
weights[:8] = 0

rng = np.random.default_rng(73921)
n = 110_000
flat = weights.ravel()
sample = rng.choice(flat.size, size=n, replace=True, p=flat / flat.sum())
ys, xs = np.divmod(sample, a.shape[1])
colors = a[ys, xs].astype(np.uint8)
sample_im = Image.fromarray(colors.reshape(1, n, 3), 'RGB')
q = sample_im.quantize(colors=48, method=Image.Quantize.MEDIANCUT)
indices = np.asarray(q, dtype=np.uint8).reshape(-1)
palette = np.asarray(q.getpalette()[:144], dtype=np.uint8).reshape(48, 3)

with (OUT/'src'/'target.bin').open('wb') as f:
    f.write(struct.pack('<I', n))
    f.write(palette.tobytes())
    for x, y, i in zip(xs, ys, indices):
        f.write(struct.pack('<HHB', int(x), int(y), int(i)))

# Design check only: final video uses these sampled records inside fframes.
preview = Image.new('RGB', (810, 870), (250, 250, 240))
d = ImageDraw.Draw(preview)
order = rng.permutation(n)
for j in order:
    px = int(xs[j] * 810 / 532)
    py = int(ys[j] * 870 / 560)
    color = tuple(int(v) for v in palette[indices[j]])
    d.ellipse((px,py,px+2,py+2), fill=color)
preview.save(OUT/'target-preview.png')
print('target particles', n, 'weighted coverage', float((weights>0).mean()))
