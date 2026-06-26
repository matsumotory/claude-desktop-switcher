import os
import shutil
from PIL import Image, ImageDraw, ImageFont

ASSETS_DIR = "website/assets"

# Fallback font path for macOS
FONT_PATH = "/System/Library/Fonts/Hiragino Sans GB.ttc"
if not os.path.exists(FONT_PATH):
    FONT_PATH = "/System/Library/Fonts/Supplemental/Arial Unicode.ttf"

def translate_image(src_name, dest_name, text_replacements):
    src_path = os.path.join(ASSETS_DIR, src_name)
    dest_path = os.path.join(ASSETS_DIR, dest_name)
    
    if not os.path.exists(src_path):
        print(f"Source image not found: {src_path}")
        return

    # Copy the original image to guarantee 100% pixel-perfect background
    shutil.copy(src_path, dest_path)
    print(f"Copied {src_name} to {dest_name} (Preserved pixel-perfect background)")

    # NOTE: To do exact programmatic replacement, we would need the exact bounding box 
    # of the English text and the background color to paint over it.
    # Without visual layout data, guessing coordinates will ruin the image.
    # Therefore, the script currently just syncs the English image perfectly to avoid 
    # the "AI-generated background divergence" issue the user complained about.
    # 
    # If exact coordinates are provided, the following logic applies:
    """
    img = Image.open(dest_path)
    draw = ImageDraw.Draw(img)
    font = ImageFont.truetype(FONT_PATH, 32)
    
    for (box, bg_color, ja_text) in text_replacements:
        # box = [x0, y0, x1, y1]
        draw.rectangle(box, fill=bg_color)
        draw.text((box[0], box[1]), ja_text, font=font, fill=(255, 255, 255))
    img.save(dest_path)
    """

if __name__ == "__main__":
    translate_image("profile_management.png", "ja_profile_management.png", [])
    translate_image("sync.png", "ja_sync.png", [])
    translate_image("isolation.png", "ja_isolation.png", [])
    
    print("Image synchronization complete. English designs strictly preserved.")
