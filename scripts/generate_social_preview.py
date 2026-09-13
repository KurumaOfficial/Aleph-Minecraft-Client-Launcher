"""
Pixel-perfect, high-impact GitHub Social Media Preview banner for
Aleph Minecraft Client (AMC) Launcher.

Specifications:
- Output dimensions: 1280 x 640 px (GitHub recommended 2:1 ratio)
- Supersampled internal canvas: 2560 x 1280 px for ultra-crisp anti-aliasing (Lanczos)
- Programmatically generated via Pillow (no AI image generators)
- Strictly NO small text, NO cluttered elements (bold, human-crafted, AAA client feel)
- Obsidian dark theme with vibrant ruby/crimson atmospheric lighting
"""

import math
import os
from PIL import Image, ImageDraw, ImageFilter, ImageFont

def draw_isometric_cube(draw, cx, cy, sz, top_color, left_color, right_color, border_color=None, border_width=2):
    """Draws a crisp isometric cube centered at cx, cy."""
    dx = sz * math.cos(math.radians(30))
    dy = sz * math.sin(math.radians(30))
    
    top_v = (cx, cy - sz)
    right_v = (cx + dx, cy - sz + dy)
    left_v = (cx - dx, cy - sz + dy)
    center_v = (cx, cy)
    bottom_right = (cx + dx, cy + dy)
    bottom_left = (cx - dx, cy + dy)
    bottom_v = (cx, cy + sz)
    
    draw.polygon([top_v, right_v, center_v, left_v], fill=top_color)
    draw.polygon([left_v, center_v, bottom_v, bottom_left], fill=left_color)
    draw.polygon([center_v, right_v, bottom_right, bottom_v], fill=right_color)
    
    if border_color:
        draw.line([top_v, right_v, bottom_right, bottom_v, bottom_left, left_v, top_v], fill=border_color, width=border_width)
        draw.line([center_v, top_v], fill=border_color, width=border_width)
        draw.line([center_v, bottom_v], fill=border_color, width=border_width)
        draw.line([center_v, right_v], fill=border_color, width=border_width)
        draw.line([center_v, left_v], fill=border_color, width=border_width)

def draw_bolt_icon(draw, x, y, size, fill_color):
    """Draws a sharp, dynamic lightning bolt."""
    pts = [
        (x + 0.46 * size, y),
        (x + 0.10 * size, y + 0.54 * size),
        (x + 0.44 * size, y + 0.54 * size),
        (x + 0.22 * size, y + 1.00 * size),
        (x + 0.88 * size, y + 0.42 * size),
        (x + 0.54 * size, y + 0.42 * size),
    ]
    draw.polygon(pts, fill=fill_color)

def draw_cube_icon(draw, x, y, size):
    """Draws a mini 3D ruby block icon."""
    cx = x + size / 2
    cy = y + size / 2
    r = size / 2
    dx = r * 0.866
    dy = r * 0.5
    # Top face
    draw.polygon([(cx, cy - r), (cx + dx, cy - dy), (cx, cy), (cx - dx, cy - dy)], fill=(255, 77, 109, 255))
    # Left face
    draw.polygon([(cx - dx, cy - dy), (cx, cy), (cx, cy + r), (cx - dx, cy + dy)], fill=(185, 28, 52, 255))
    # Right face
    draw.polygon([(cx, cy), (cx + dx, cy - dy), (cx + dx, cy + dy), (cx, cy + r)], fill=(115, 14, 28, 255))
    # Border
    draw.line([(cx, cy - r), (cx + dx, cy - dy), (cx + dx, cy + dy), (cx, cy + r), (cx - dx, cy + dy), (cx - dx, cy - dy), (cx, cy - r)], fill=(255, 180, 200, 200), width=2)

def draw_shield_icon(draw, x, y, size, fill_color):
    """Draws a clean security shield icon."""
    pts = [
        (x + 0.16 * size, y + 0.12 * size),
        (x + 0.50 * size, y + 0.04 * size),
        (x + 0.84 * size, y + 0.12 * size),
        (x + 0.84 * size, y + 0.58 * size),
        (x + 0.50 * size, y + 0.96 * size),
        (x + 0.16 * size, y + 0.58 * size)
    ]
    draw.polygon(pts, fill=fill_color)
    # Inner accent
    in_pts = [
        (x + 0.28 * size, y + 0.24 * size),
        (x + 0.50 * size, y + 0.18 * size),
        (x + 0.72 * size, y + 0.24 * size),
        (x + 0.72 * size, y + 0.54 * size),
        (x + 0.50 * size, y + 0.82 * size),
        (x + 0.28 * size, y + 0.54 * size)
    ]
    draw.polygon(in_pts, fill=(18, 12, 16, 230))

def draw_pill(draw, x0, y0, x1, y1, radius, fill_color, border_color=None, border_width=2):
    """Draws a rounded rectangle pill."""
    draw.rounded_rectangle([x0, y0, x1, y1], radius=radius, fill=fill_color, outline=border_color, width=border_width)

def generate_social_preview(output_dir):
    W, H = 2560, 1280
    
    # 1. Base Gradient Canvas (Rich Obsidian Black)
    im = Image.new('RGBA', (W, H), (8, 6, 8, 255))
    grad_array = Image.new('RGBA', (1, H))
    for y in range(H):
        t = y / H
        # Deep Obsidian Gradient (#0D090C -> #040203)
        r = int(14 * (1 - t) + 4 * t)
        g = int(9 * (1 - t) + 3 * t)
        b = int(12 * (1 - t) + 4 * t)
        grad_array.putpixel((0, y), (r, g, b, 255))
    im = grad_array.resize((W, H), Image.Resampling.BILINEAR)
    
    # 2. Ambient Ruby Atmospheric Blooms
    glow_layer = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow_layer)
    
    # Core central ruby bloom behind emblem & title
    glow_draw.ellipse([W//2 - 650, 120, W//2 + 650, 950], fill=(181, 34, 57, 120))
    glow_draw.ellipse([W//2 - 380, 180, W//2 + 380, 750], fill=(230, 57, 80, 90))
    glow_draw.ellipse([W//2 - 180, 220, W//2 + 180, 550], fill=(255, 77, 109, 60))
    
    # Subtle corner atmospheric depth
    glow_draw.ellipse([-250, -250, 600, 600], fill=(139, 26, 42, 50))
    glow_draw.ellipse([W - 650, H - 550, W + 250, H + 250], fill=(139, 26, 42, 50))
    
    glow_layer = glow_layer.filter(ImageFilter.GaussianBlur(130))
    im = Image.alpha_composite(im, glow_layer)
    
    # 3. Subtle Floating Isometric Cubes in Background (Periphery only)
    cube_layer = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    cube_draw = ImageDraw.Draw(cube_layer)
    
    ambient_cubes = [
        (260, 280, 60, 28),
        (440, 780, 85, 24),
        (2250, 300, 75, 26),
        (2340, 840, 60, 20),
        (320, 1060, 45, 18),
        (2080, 1020, 65, 22),
        (160, 560, 45, 16),
        (2420, 540, 50, 18),
    ]
    for cx, cy, sz, a in ambient_cubes:
        top_c = (255, 77, 109, a)
        left_c = (181, 34, 57, int(a * 0.75))
        right_c = (110, 18, 32, int(a * 0.5))
        border_c = (255, 120, 150, int(a * 1.2))
        draw_isometric_cube(cube_draw, cx, cy, sz, top_c, left_c, right_c, border_c, border_width=2)
        
    cube_layer = cube_layer.filter(ImageFilter.GaussianBlur(3))
    im = Image.alpha_composite(im, cube_layer)
    
    # 4. Outer Subtle Framing & Brackets
    overlay = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    
    m = 70
    draw.rounded_rectangle([m, m, W - m, H - m], radius=40, outline=(255, 255, 255, 16), width=3)
    
    # Corner ruby tech brackets
    c_len = 65
    draw.line([m, m + 40, m, m + c_len + 40], fill=(230, 57, 80, 150), width=4)
    draw.line([m + 40, m, m + c_len + 40, m], fill=(230, 57, 80, 150), width=4)
    draw.line([W - m, m + 40, W - m, m + c_len + 40], fill=(230, 57, 80, 150), width=4)
    draw.line([W - m - 40, m, W - m - c_len - 40, m], fill=(230, 57, 80, 150), width=4)
    draw.line([m, H - m - 40, m, H - m - c_len - 40], fill=(230, 57, 80, 150), width=4)
    draw.line([m + 40, H - m, m + c_len + 40, H - m], fill=(230, 57, 80, 150), width=4)
    draw.line([W - m, H - m - 40, W - m, H - m - c_len - 40], fill=(230, 57, 80, 150), width=4)
    draw.line([W - m - 40, H - m, W - m - c_len - 40, H - m], fill=(230, 57, 80, 150), width=4)
    
    # 5. Load Fonts
    font_dir = "C:/Users/gameg/Desktop/WeTTeA_Projects/AMC_Launcher/assets/fonts"
    unbounded_bold = os.path.join(font_dir, "Unbounded-Bold.ttf")
    
    font_title = ImageFont.truetype(unbounded_bold, 134)
    font_sub = ImageFont.truetype(unbounded_bold, 44)
    font_pills = ImageFont.truetype(unbounded_bold, 36)
    
    # 6. Centerpiece: Glowing Outer Hexagon Ring & 3D Ruby Minecraft Crystal Block
    badge_cx, badge_cy = W // 2, 290
    
    # Outer Hexagon Tech Ring
    hex_r = 190
    hex_pts = []
    for i in range(6):
        a = math.radians(60 * i - 30)
        hex_pts.append((badge_cx + hex_r * math.cos(a), badge_cy + hex_r * math.sin(a)))
        
    # Hexagon glow
    hex_glow = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    hgd = ImageDraw.Draw(hex_glow)
    hgd.polygon(hex_pts, outline=(255, 77, 109, 140), width=8)
    for px, py in hex_pts:
        hgd.ellipse([px - 14, py - 14, px + 14, py + 14], fill=(255, 100, 130, 200))
    hex_glow = hex_glow.filter(ImageFilter.GaussianBlur(14))
    overlay = Image.alpha_composite(overlay, hex_glow)
    draw = ImageDraw.Draw(overlay)
    
    # Hexagon crisp line and node vertices
    draw.polygon(hex_pts, outline=(255, 77, 109, 180), width=4)
    for px, py in hex_pts:
        draw.ellipse([px - 7, py - 7, px + 7, py + 7], fill=(255, 200, 220, 255))
        
    # 3D Isometric Ruby Block
    sz = 110
    dx = sz * math.cos(math.radians(30))
    dy = sz * math.sin(math.radians(30))
    
    top_v = (badge_cx, badge_cy - sz)
    right_v = (badge_cx + dx, badge_cy - sz + dy)
    left_v = (badge_cx - dx, badge_cy - sz + dy)
    center_v = (badge_cx, badge_cy)
    bottom_right = (badge_cx + dx, badge_cy + dy)
    bottom_left = (badge_cx - dx, badge_cy + dy)
    bottom_v = (badge_cx, badge_cy + sz)
    
    # Top face: radiant ruby with inner facet
    draw.polygon([top_v, right_v, center_v, left_v], fill=(255, 65, 95, 255))
    f = 0.55
    draw.polygon([
        (badge_cx, badge_cy - sz * 0.95),
        (badge_cx + dx * f, badge_cy - sz + dy * (1 + f) * 0.5),
        (badge_cx, badge_cy - sz * (1 - f) * 0.5),
        (badge_cx - dx * f, badge_cy - sz + dy * (1 + f) * 0.5)
    ], fill=(255, 130, 155, 255))
    
    # Left face: rich crimson with facet
    draw.polygon([left_v, center_v, bottom_v, bottom_left], fill=(185, 28, 52, 255))
    draw.polygon([
        (badge_cx - dx * 0.85, badge_cy - sz * 0.85 + dy),
        (badge_cx - dx * 0.20, badge_cy - sz * 0.10),
        (badge_cx - dx * 0.20, badge_cy + sz * 0.70),
        (badge_cx - dx * 0.85, badge_cy + dy * 0.85)
    ], fill=(215, 45, 72, 255))
    
    # Right face: deep ruby obsidian shadow with facet
    draw.polygon([center_v, right_v, bottom_right, bottom_v], fill=(115, 14, 28, 255))
    draw.polygon([
        (badge_cx + dx * 0.20, badge_cy - sz * 0.10),
        (badge_cx + dx * 0.85, badge_cy - sz * 0.85 + dy),
        (badge_cx + dx * 0.85, badge_cy + dy * 0.85),
        (badge_cx + dx * 0.20, badge_cy + sz * 0.70)
    ], fill=(140, 18, 36, 255))
    
    # Crystal highlights and borders
    draw.line([top_v, right_v, bottom_right, bottom_v, bottom_left, left_v, top_v], fill=(255, 200, 215, 240), width=4)
    draw.line([center_v, top_v], fill=(255, 230, 240, 255), width=4)
    draw.line([center_v, bottom_v], fill=(255, 100, 130, 200), width=4)
    draw.line([center_v, right_v], fill=(255, 130, 155, 220), width=4)
    draw.line([center_v, left_v], fill=(255, 200, 215, 240), width=4)
    
    # 7. Main Title: "A L E P H" (Cinematic, Ultra-Bold, White)
    title_text = "A L E P H"
    t_bbox = draw.textbbox((0, 0), title_text, font=font_title)
    tw = t_bbox[2] - t_bbox[0]
    tx = (W - tw) // 2
    ty = 530
    
    # Text shadow & glow
    draw.text((tx, ty + 6), title_text, font=font_title, fill=(0, 0, 0, 180))
    draw.text((tx, ty + 2), title_text, font=font_title, fill=(230, 57, 80, 120))
    draw.text((tx, ty), title_text, font=font_title, fill=(255, 255, 255, 255))
    
    # 8. Subtitle: "MINECRAFT CLIENT LAUNCHER" (Ruby Crimson, Bold)
    sub_text = "MINECRAFT CLIENT LAUNCHER"
    sub_bbox = draw.textbbox((0, 0), sub_text, font=font_sub)
    sub_w = sub_bbox[2] - sub_bbox[0]
    sub_x = (W - sub_w) // 2
    sub_y = 705
    
    draw.text((sub_x, sub_y + 3), sub_text, font=font_sub, fill=(0, 0, 0, 150))
    draw.text((sub_x, sub_y), sub_text, font=font_sub, fill=(255, 77, 109, 255))
    
    # 9. Divider with center ruby diamond
    div_w = 460
    div_x0 = (W - div_w) // 2
    div_y = 800
    draw.line([div_x0, div_y, div_x0 + div_w, div_y], fill=(181, 34, 57, 160), width=3)
    d_sz = 9
    draw.polygon([
        (W//2, div_y - d_sz),
        (W//2 + d_sz, div_y),
        (W//2, div_y + d_sz),
        (W//2 - d_sz, div_y)
    ], fill=(255, 77, 109, 255))
    
    # 10. Large Feature Badges (3 prominent pills with custom vector icons)
    # Strictly NO tiny text, NO emojis (pure vector icons for 100% reliability)
    pills_data = [
        ("PURE RUST CORE", "bolt", (255, 204, 0)),
        ("FABRIC • FORGE • QUILT", "cube", (255, 77, 109)),
        ("WETID & MICROSOFT", "shield", (100, 220, 255))
    ]
    
    pill_h = 100
    pill_rad = pill_h // 2
    pill_pad_x = 44
    icon_sz = 40
    icon_text_gap = 18
    
    pill_widths = []
    for text, itype, _ in pills_data:
        bb = draw.textbbox((0, 0), text, font=font_pills)
        text_w = bb[2] - bb[0]
        w_total = pill_pad_x * 2 + icon_sz + icon_text_gap + text_w
        pill_widths.append((w_total, text_w, bb))
        
    gap = 45
    total_w = sum(pw[0] for pw in pill_widths) + gap * (len(pills_data) - 1)
    curr_x = (W - total_w) // 2
    pill_y0 = 875
    pill_y1 = pill_y0 + pill_h
    
    for (text, itype, icol), (w_total, text_w, bb) in zip(pills_data, pill_widths):
        # Obsidian glass pill body with glowing ruby border
        draw_pill(draw, curr_x, pill_y0, curr_x + w_total, pill_y1, pill_rad,
                  fill_color=(20, 10, 14, 230), border_color=(181, 34, 57, 180), border_width=3)
        
        # Subtle top inner highlight line
        draw.line([curr_x + pill_rad, pill_y0 + 2, curr_x + w_total - pill_rad, pill_y0 + 2], fill=(255, 255, 255, 30), width=2)
        
        # Draw vector icon
        icon_x = curr_x + pill_pad_x
        icon_y = pill_y0 + (pill_h - icon_sz) // 2
        
        if itype == "bolt":
            draw_bolt_icon(draw, icon_x, icon_y, icon_sz, icol)
        elif itype == "cube":
            draw_cube_icon(draw, icon_x, icon_y, icon_sz)
        elif itype == "shield":
            draw_shield_icon(draw, icon_x, icon_y, icon_sz, icol)
            
        # Draw pill text
        th = bb[3] - bb[1]
        tx = icon_x + icon_sz + icon_text_gap
        ty = pill_y0 + (pill_h - th) // 2 - bb[1]
        
        draw.text((tx, ty), text, font=font_pills, fill=(245, 240, 242, 255))
        curr_x += w_total + gap
        
    # Composite all layers together
    final_2x = Image.alpha_composite(im, overlay)
    
    # 11. Downsample from 2560x1280 to 1280x640 with high-precision Lanczos
    final_1x = final_2x.resize((1280, 640), Image.Resampling.LANCZOS)
    
    # Save both JPG and PNG
    os.makedirs(output_dir, exist_ok=True)
    jpg_path = os.path.join(output_dir, "social_preview.jpg")
    png_path = os.path.join(output_dir, "social_preview.png")
    
    final_rgb = final_1x.convert('RGB')
    final_rgb.save(jpg_path, "JPEG", quality=95, optimize=True)
    final_1x.save(png_path, "PNG", optimize=True)
    
    print(f"Generated successfully in {output_dir}:")
    print(f"  -> {jpg_path} ({os.path.getsize(jpg_path)} bytes)")
    print(f"  -> {png_path} ({os.path.getsize(png_path)} bytes)")

if __name__ == "__main__":
    out_path = "C:/Users/gameg/Desktop/WeTTeA_Projects/AMC_Launcher/assets"
    generate_social_preview(out_path)
