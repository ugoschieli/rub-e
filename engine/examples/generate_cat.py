#!/usr/bin/env python3
"""Generate a new, very detailed cat model in a sitting pose."""

import math

def write_cube(f, x, y, z, r, g, b):
    """Write a single cube to the model file."""
    f.write(f"{x:.2f} {y:.2f} {z:.2f} {r:.3f} {g:.3f} {b:.3f}\n")

def write_sphere(f, cx, cy, cz, radius, r, g, b, density=1.0):
    """Write a spherical volume of cubes."""
    # To make it look more organic, we can vary the density
    for x in range(int(cx - radius), int(cx + radius + 1)):
        for y in range(int(cy - radius), int(cy + radius + 1)):
            for z in range(int(cz - radius), int(cz + radius + 1)):
                dist_sq = (x - cx)**2 + (y - cy)**2 + (z - cz)**2
                if dist_sq <= radius**2:
                    # Add some noise to the surface
                    if dist_sq > (radius-1.5)**2:
                        if hash(f"{x},{y},{z}") % 100 / 100.0 < density:
                            write_cube(f, x, y, z, r, g, b)
                    else:
                        write_cube(f, x, y, z, r, g, b)

def write_ellipsoid(f, cx, cy, cz, rx, ry, rz, r, g, b):
    """Write an ellipsoidal volume of cubes."""
    for x in range(int(cx - rx), int(cx + rx + 1)):
        for y in range(int(cy - ry), int(cy + ry + 1)):
            for z in range(int(cz - rz), int(cz + rz + 1)):
                if ((x-cx)/rx)**2 + ((y-cy)/ry)**2 + ((z-cz)/rz)**2 <= 1:
                    write_cube(f, x, y, z, r, g, b)

with open('examples/models/cat.model', 'w') as f:
    f.write("# A new, very detailed cat model in a sitting pose\n")

    # Colors
    orange = (1.0, 0.5, 0.1)
    dark_orange = (0.8, 0.4, 0.0)
    light_orange = (1.0, 0.7, 0.3)
    white = (0.95, 0.95, 0.95)
    black = (0.1, 0.1, 0.1)
    green = (0.1, 0.8, 0.1)
    pink = (1.0, 0.6, 0.7)

    # BODY (sitting pose) - a large ellipsoid
    f.write("\n# Body\n")
    write_ellipsoid(f, 0, 5, 0, 6, 7, 8, *orange)
    
    # Front Paws
    f.write("\n# Front Paws\n")
    write_sphere(f, -3, 1, 6, 2, *dark_orange)
    write_sphere(f, 3, 1, 6, 2, *dark_orange)
    
    # Chest/Belly
    f.write("\n# Chest and Belly\n")
    write_ellipsoid(f, 0, 4, 3, 4, 4, 5, *light_orange)

    # HEAD
    f.write("\n# Head\n")
    write_sphere(f, 0, 14, 2, 5, *orange, density=0.9)

    # EARS (more triangular)
    f.write("\n# Ears\n")
    for i in range(4):
        # Left ear
        write_ellipsoid(f, -4, 18+i, 2, 2-i*0.5, 1, 1, *dark_orange)
        # Right ear
        write_ellipsoid(f, 4, 18+i, 2, 2-i*0.5, 1, 1, *dark_orange)
    
    # Muzzle
    f.write("\n# Muzzle\n")
    write_sphere(f, 0, 13, 6, 2.5, *light_orange)
    
    # Nose
    f.write("\n# Nose\n")
    write_sphere(f, 0, 14, 8, 0.5, *pink)

    # EYES
    f.write("\n# Eyes\n")
    write_sphere(f, -2.5, 15, 6, 1.2, *green)
    write_cube(f, -2.5, 15, 7, *black) # pupil
    write_sphere(f, 2.5, 15, 6, 1.2, *green)
    write_cube(f, 2.5, 15, 7, *black) # pupil

    # Whiskers
    f.write("\n# Whiskers\n")
    for i in range(3, 8):
        write_cube(f, -i, 13, 7, *white)
        write_cube(f, i, 13, 7, *white)
        write_cube(f, -i, 12.5, 7.5, *white)
        write_cube(f, i, 12.5, 7.5, *white)

    # TAIL wrapped around body
    f.write("\n# Tail\n")
    for i in range(20):
        angle = i * (math.pi / 16)
        x = math.cos(angle) * 7
        z = math.sin(angle) * 7 - 7
        y = 1.5 + math.sin(i/5.0)
        r = 1.5 - i * 0.05
        # alternate color for stripes
        color = dark_orange if i % 4 < 2 else orange
        write_sphere(f, x, y, z, r, *color)

print("Generated a new, very detailed cat model!")