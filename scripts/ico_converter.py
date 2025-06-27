from PIL import Image

source = "..\\assets\\icon\\icon.png"

source = Image.open(source).convert("RGBA")

for size in (16, 32, 64):
    ico = source.resize((size, size), Image.LANCZOS)
    ico.save(f"..\\assets\\icon\\icon{size}.png")