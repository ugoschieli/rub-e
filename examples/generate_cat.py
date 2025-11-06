#!/usr/bin/env python3
"""Generate a detailed cat model with thousands of cubes"""

def write_cube(f, x, y, z, r, g, b):
    """Write a single cube to the model file"""
    f.write(f"{x} {y} {z} {r} {g} {b}\n")

def write_block(f, x_range, y_range, z_range, r, g, b):
    """Write a rectangular block of cubes"""
    for x in x_range:
        for y in y_range:
            for z in z_range:
                write_cube(f, x, y, z, r, g, b)

def write_sphere(f, cx, cy, cz, radius, r, g, b):
    """Write a spherical volume of cubes"""
    for x in range(cx - radius, cx + radius + 1):
        for y in range(cy - radius, cy + radius + 1):
            for z in range(cz - radius, cz + radius + 1):
                # Check if point is within sphere
                dist_sq = (x - cx)**2 + (y - cy)**2 + (z - cz)**2
                if dist_sq <= radius**2:
                    write_cube(f, x, y, z, r, g, b)

with open('examples/models/cat.model', 'w') as f:
    f.write("# Detailed Cat Model - Generated with thousands of cubes\n")
    f.write("# Format: x y z r g b (integer positions, float colors)\n\n")

    # Orange color for body
    orange = (1.0, 0.6, 0.2)
    dark_orange = (0.8, 0.4, 0.1)
    light_orange = (1.0, 0.8, 0.6)
    brown = (0.7, 0.4, 0.1)
    green = (0.2, 1.0, 0.2)
    dark_green = (0.1, 0.5, 0.1)
    pink = (1.0, 0.5, 0.7)
    white = (0.9, 0.9, 0.9)
    black = (0.1, 0.1, 0.1)

    # HEAD - Large spherical head
    f.write("# Head (spherical, orange)\n")
    write_sphere(f, 0, 10, -2, 4, *orange)

    # EARS - Triangular pointed ears
    f.write("\n# Left Ear\n")
    write_block(f, range(-5, -2), range(12, 16), range(-3, 0), *dark_orange)
    f.write("\n# Right Ear\n")
    write_block(f, range(3, 6), range(12, 16), range(-3, 0), *dark_orange)

    # EYES - Green with black pupils
    f.write("\n# Left Eye\n")
    write_sphere(f, -3, 11, 1, 1, *green)
    write_cube(f, -3, 11, 2, *black)  # Pupil
    f.write("\n# Right Eye\n")
    write_sphere(f, 3, 11, 1, 1, *green)
    write_cube(f, 3, 11, 2, *black)  # Pupil

    # NOSE - Pink triangular nose
    f.write("\n# Nose\n")
    write_block(f, range(-1, 2), range(9, 11), range(2, 4), *pink)

    # WHISKERS - Thin white lines
    f.write("\n# Whiskers\n")
    for i in range(5, 10):
        write_cube(f, -i, 10, 2, *white)
        write_cube(f, i, 10, 2, *white)

    # NECK - Connecting head to body
    f.write("\n# Neck\n")
    write_block(f, range(-3, 4), range(6, 10), range(-5, -2), *orange)

    # BODY - Large rectangular torso
    f.write("\n# Body Core\n")
    write_block(f, range(-4, 5), range(4, 8), range(-15, -4), *orange)

    # CHEST - Light colored chest
    f.write("\n# Chest\n")
    write_block(f, range(-2, 3), range(4, 7), range(-6, -4), *light_orange)

    # BELLY - Light colored belly
    f.write("\n# Belly\n")
    write_block(f, range(-3, 4), range(2, 5), range(-13, -6), *light_orange)

    # STRIPES - Darker stripes along back
    f.write("\n# Back Stripes\n")
    for z in [-7, -9, -11]:
        write_block(f, range(-2, 3), range(7, 8), range(z, z+1), *dark_orange)

    # FRONT LEFT LEG
    f.write("\n# Front Left Leg\n")
    write_block(f, range(-4, -2), range(0, 5), range(-6, -4), *brown)
    # Paw
    write_block(f, range(-5, -1), range(0, 1), range(-7, -3), *dark_orange)

    # FRONT RIGHT LEG
    f.write("\n# Front Right Leg\n")
    write_block(f, range(3, 5), range(0, 5), range(-6, -4), *brown)
    # Paw
    write_block(f, range(2, 6), range(0, 1), range(-7, -3), *dark_orange)

    # BACK LEFT LEG
    f.write("\n# Back Left Leg\n")
    write_block(f, range(-4, -2), range(0, 5), range(-14, -12), *brown)
    # Paw
    write_block(f, range(-5, -1), range(0, 1), range(-15, -11), *dark_orange)

    # BACK RIGHT LEG
    f.write("\n# Back Right Leg\n")
    write_block(f, range(3, 5), range(0, 5), range(-14, -12), *brown)
    # Paw
    write_block(f, range(2, 6), range(0, 1), range(-15, -11), *dark_orange)

    # TAIL - Curved upward tail
    f.write("\n# Tail\n")
    tail_segments = [
        (0, 5, -16, 1.0, 0.6, 0.2),
        (0, 6, -17, 1.0, 0.65, 0.25),
        (0, 7, -18, 1.0, 0.7, 0.3),
        (0, 8, -19, 1.0, 0.75, 0.35),
        (0, 9, -20, 1.0, 0.8, 0.4),
        (0, 10, -21, 1.0, 0.85, 0.45),
        (0, 11, -22, 1.0, 0.9, 0.5),
    ]
    for cx, cy, cz, r, g, b in tail_segments:
        write_sphere(f, cx, cy, cz, 1, r, g, b)

print("Generated detailed cat model with thousands of cubes!")
