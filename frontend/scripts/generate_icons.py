"""Tạo icon PNG từ hình chữ D của public/favicon.svg. Cần Pillow khi chạy lại."""

from pathlib import Path
from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1] / "public"
GREEN = "#087f76"


def curve(start, control_a, control_b, end):
    points = []
    for index in range(1, 33):
        t = index / 32
        u = 1 - t
        points.append(
            (
                u**3 * start[0] + 3 * u**2 * t * control_a[0] + 3 * u * t**2 * control_b[0] + t**3 * end[0],
                u**3 * start[1] + 3 * u**2 * t * control_a[1] + 3 * u * t**2 * control_b[1] + t**3 * end[1],
            )
        )
    return points


OUTER = [(18, 19), (32, 19)] + curve((32, 19), (42, 19), (48, 24), (48, 32)) + curve((48, 32), (48, 40), (42, 45), (32, 45)) + [(18, 45)]
INNER = [(28, 27), (28, 37), (32, 37)] + curve((32, 37), (37, 37), (40, 35), (40, 32)) + curve((40, 32), (40, 29), (37, 27), (32, 27))


def render(size, filename):
    large = size * 4
    image = Image.new("RGB", (large, large), GREEN)
    draw = ImageDraw.Draw(image)

    def scale(points):
        return [((x - 32) * 1.2 + 32) * large / 64 for x, _ in points], [((y - 32) * 1.2 + 32) * large / 64 for _, y in points]

    for points, color in ((OUTER, "white"), (INNER, GREEN)):
        xs, ys = scale(points)
        draw.polygon(list(zip(xs, ys)), fill=color)

    image.resize((size, size), Image.Resampling.LANCZOS).save(ROOT / filename, optimize=True)


render(180, "apple-touch-icon.png")
render(192, "icon-192.png")
render(512, "icon-512.png")
