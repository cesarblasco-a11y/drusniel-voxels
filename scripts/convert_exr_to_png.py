#!/usr/bin/env python3
"""
Convert EXR textures to PNG for Bevy compatibility.
Requires: pip install OpenEXR numpy Pillow
"""

import os
import sys

try:
    import OpenEXR
    import Imath
    import numpy as np
    from PIL import Image
except ImportError:
    print("Installing required packages...")
    os.system(f"{sys.executable} -m pip install OpenEXR numpy Pillow")
    import OpenEXR
    import Imath
    import numpy as np
    from PIL import Image

def exr_to_png(exr_path: str, png_path: str):
    """Convert an EXR file to PNG, preserving data in linear space."""
    exr_file = OpenEXR.InputFile(exr_path)
    header = exr_file.header()
    
    dw = header['dataWindow']
    width = dw.max.x - dw.min.x + 1
    height = dw.max.y - dw.min.y + 1
    
    # Get available channels
    channels = header['channels'].keys()
    print(f"  Channels in {os.path.basename(exr_path)}: {list(channels)}")
    
    pt = Imath.PixelType(Imath.PixelType.FLOAT)
    
    if 'R' in channels and 'G' in channels and 'B' in channels:
        # RGB image (normal maps, etc.)
        r_str = exr_file.channel('R', pt)
        g_str = exr_file.channel('G', pt)
        b_str = exr_file.channel('B', pt)
        
        r = np.frombuffer(r_str, dtype=np.float32).reshape((height, width))
        g = np.frombuffer(g_str, dtype=np.float32).reshape((height, width))
        b = np.frombuffer(b_str, dtype=np.float32).reshape((height, width))
        
        # Clamp and convert to 8-bit
        r = np.clip(r * 0.5 + 0.5, 0, 1)  # Normal maps: -1..1 -> 0..1
        g = np.clip(g * 0.5 + 0.5, 0, 1)
        b = np.clip(b * 0.5 + 0.5, 0, 1)
        
        rgb = np.stack([r, g, b], axis=-1)
        rgb_8bit = (rgb * 255).astype(np.uint8)
        
        img = Image.fromarray(rgb_8bit, mode='RGB')
        
    elif 'Y' in channels:
        # Grayscale (roughness, etc.)
        y_str = exr_file.channel('Y', pt)
        y = np.frombuffer(y_str, dtype=np.float32).reshape((height, width))
        y = np.clip(y, 0, 1)
        y_8bit = (y * 255).astype(np.uint8)
        img = Image.fromarray(y_8bit, mode='L')
        
    else:
        # Try first available channel
        ch = list(channels)[0]
        ch_str = exr_file.channel(ch, pt)
        data = np.frombuffer(ch_str, dtype=np.float32).reshape((height, width))
        data = np.clip(data, 0, 1)
        data_8bit = (data * 255).astype(np.uint8)
        img = Image.fromarray(data_8bit, mode='L')
    
    img.save(png_path)
    print(f"  Converted: {exr_path} -> {png_path}")

def main():
    # Find EXR files in rocky_terrain folder
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    texture_dir = os.path.join(base_dir, "assets", "textures", "rocky_terrain_02_1k")
    
    if not os.path.exists(texture_dir):
        print(f"Directory not found: {texture_dir}")
        return
    
    print(f"Scanning: {texture_dir}")
    
    for filename in os.listdir(texture_dir):
        if filename.endswith('.exr'):
            exr_path = os.path.join(texture_dir, filename)
            png_name = filename.replace('.exr', '.png')
            png_path = os.path.join(texture_dir, png_name)
            
            if os.path.exists(png_path):
                print(f"  Skipping (already exists): {png_name}")
                continue
                
            try:
                exr_to_png(exr_path, png_path)
            except Exception as e:
                print(f"  Error converting {filename}: {e}")
    
    print("Done!")

if __name__ == "__main__":
    main()
