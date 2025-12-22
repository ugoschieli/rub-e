#!/usr/bin/env python3
"""
Generate a detailed 3D scene with a floor with relief, randomly colored cubes, and the cat model.
"""

import random
import math
import os

def generate_floor(width=100, depth=100, y_base=0):
    """Generate a detailed floor with small relief variations using sine waves."""
    cubes = []
    
    # Adjust frequency based on size for consistent look
    freq_scale = 100.0 / width  # Scale frequencies to maintain similar patterns
    
    for x in range(-width//2, width//2):
        for z in range(-depth//2, depth//2):
            # Create relief using multiple sine waves for more interesting terrain
            # Scaled frequencies for larger scenes
            relief = math.sin(x * 0.2 * freq_scale) * 0.7 + math.cos(z * 0.2 * freq_scale) * 0.7
            relief += math.sin(x * 0.05 * freq_scale + z * 0.05 * freq_scale) * 1.5
            relief += math.sin(x * 0.15 * freq_scale) * math.cos(z * 0.15 * freq_scale) * 0.5
            y = y_base + int(relief * 1.5)  # Height variation
            
            # Floor color - greenish with some variation based on height
            height_factor = (y - y_base) / 5.0  # Normalize height
            r = 0.25 + random.uniform(-0.05, 0.05) + height_factor * 0.1
            g = 0.55 + random.uniform(-0.05, 0.05) + height_factor * 0.15
            b = 0.25 + random.uniform(-0.05, 0.05)
            
            # Clamp colors
            r = max(0.0, min(1.0, r))
            g = max(0.0, min(1.0, g))
            b = max(0.0, min(1.0, b))
            
            cubes.append((x, y, z, r, g, b))
    
    return cubes

def generate_random_cubes(num_cubes=150, floor_width=80, floor_depth=80):
    """Generate randomly colored cubes and structures on the floor."""
    cubes = []
    
    for i in range(num_cubes):
        # Random position on the floor
        x = random.randint(-floor_width//2 + 10, floor_width//2 - 10)
        z = random.randint(-floor_depth//2 + 10, floor_depth//2 - 10)
        
        # Calculate floor height at this position (matching the relief formula)
        relief = math.sin(x * 0.2) * 0.7 + math.cos(z * 0.2) * 0.7
        relief += math.sin(x * 0.05 + z * 0.05) * 1.5
        relief += math.sin(x * 0.15) * math.cos(z * 0.15) * 0.5
        floor_y = int(relief * 1.5)
        
        # Random height for cube stacks (1-8 cubes tall)
        height = random.randint(1, 8)
        
        # Random structure type
        structure_type = random.choice(['single', 'pyramid', 'tower', 'platform'])
        
        # Random bright color
        color_choice = random.randint(0, 11)
        if color_choice == 0:  # Red
            r, g, b = 1.0, 0.2, 0.2
        elif color_choice == 1:  # Blue
            r, g, b = 0.2, 0.2, 1.0
        elif color_choice == 2:  # Yellow
            r, g, b = 1.0, 1.0, 0.2
        elif color_choice == 3:  # Purple
            r, g, b = 0.8, 0.2, 0.8
        elif color_choice == 4:  # Orange
            r, g, b = 1.0, 0.6, 0.2
        elif color_choice == 5:  # Cyan
            r, g, b = 0.2, 1.0, 1.0
        elif color_choice == 6:  # Pink
            r, g, b = 1.0, 0.4, 0.7
        elif color_choice == 7:  # Lime
            r, g, b = 0.6, 1.0, 0.2
        elif color_choice == 8:  # White
            r, g, b = 1.0, 1.0, 1.0
        elif color_choice == 9:  # Magenta
            r, g, b = 1.0, 0.0, 1.0
        elif color_choice == 10:  # Teal
            r, g, b = 0.2, 0.8, 0.7
        else:  # Gold
            r, g, b = 1.0, 0.8, 0.2
        
        # Add slight random variation
        r = min(1.0, max(0.0, r + random.uniform(-0.1, 0.1)))
        g = min(1.0, max(0.0, g + random.uniform(-0.1, 0.1)))
        b = min(1.0, max(0.0, b + random.uniform(-0.1, 0.1)))
        
        # Create structure based on type
        if structure_type == 'single':
            # Simple stack
            for h in range(height):
                y = floor_y + h + 1
                cubes.append((x, y, z, r, g, b))
        
        elif structure_type == 'pyramid':
            # Pyramid structure
            base_size = min(3, height // 2 + 1)
            for level in range(height):
                size = max(1, base_size - level // 2)
                for dx in range(-size + 1, size):
                    for dz in range(-size + 1, size):
                        y = floor_y + level + 1
                        cubes.append((x + dx, y, z + dz, r, g, b))
        
        elif structure_type == 'tower':
            # 2x2 tower
            for h in range(height):
                y = floor_y + h + 1
                for dx in [0, 1]:
                    for dz in [0, 1]:
                        cubes.append((x + dx, y, z + dz, r, g, b))
        
        else:  # platform
            # Flat platform
            platform_height = random.randint(2, 4)
            platform_size = random.randint(2, 4)
            for dx in range(platform_size):
                for dz in range(platform_size):
                    for h in range(platform_height):
                        y = floor_y + h + 1
                        cubes.append((x + dx, y, z + dz, r, g, b))
    
    return cubes

def fill_gaps(cubes):
    """Fill single-cube gaps between cubes along axes."""
    if not cubes:
        return []

    print("      -> Filling gaps in model...")
    cubes_dict = {(c[0], c[1], c[2]): (c[3], c[4], c[5]) for c in cubes}
    original_cube_count = len(cubes_dict)
    
    new_cubes_to_add = {}

    for (x, y, z), color in cubes_dict.items():
        # Check in positive directions for neighbors at distance 2
        for dx, dy, dz in [(1, 0, 0), (0, 1, 0), (0, 0, 1)]:
            gap_mid_coord = (x + dx, y + dy, z + dz)
            neighbor_coord = (x + 2 * dx, y + 2 * dy, z + 2 * dz)

            # If there's a neighbor at distance 2 and no cube in between
            if neighbor_coord in cubes_dict and gap_mid_coord not in cubes_dict:
                # Check if we already planned to add this cube
                if gap_mid_coord not in new_cubes_to_add:
                    # Interpolate color
                    neighbor_color = cubes_dict[neighbor_coord]
                    new_color = (
                        (color[0] + neighbor_color[0]) / 2,
                        (color[1] + neighbor_color[1]) / 2,
                        (color[2] + neighbor_color[2]) / 2
                    )
                    new_cubes_to_add[gap_mid_coord] = new_color

    cubes_dict.update(new_cubes_to_add)
    
    filled_cubes_list = [(x, y, z, r, g, b) for (x, y, z), (r, g, b) in cubes_dict.items()]
    
    newly_added_count = len(filled_cubes_list) - original_cube_count
    if newly_added_count > 0:
        print(f"      -> Added {newly_added_count} cubes to fill gaps.")
    
    return filled_cubes_list


def make_solid(cubes):
    """Fills the interior of a model layer by layer to make it solid."""
    if not cubes:
        return []

    print("      -> Making model solid by filling interior gaps...")
    original_cube_count = len(cubes)

    # Use a dictionary for quick lookups of cubes and their colors
    cubes_dict = {(c[0], c[1], c[2]): (c[3], c[4], c[5]) for c in cubes}

    # Group cubes by their y-coordinate
    cubes_by_y = {}
    for x, y, z, r, g, b in cubes:
        if y not in cubes_by_y:
            cubes_by_y[y] = []
        cubes_by_y[y].append((x, z))

    solid_cubes_to_add = {}

    for y, layer_coords in cubes_by_y.items():
        if len(layer_coords) < 2:
            continue

        # Group coordinates by x and z to find ranges to fill
        coords_by_x = {}
        coords_by_z = {}
        for x, z in layer_coords:
            if x not in coords_by_x:
                coords_by_x[x] = []
            coords_by_x[x].append(z)
            if z not in coords_by_z:
                coords_by_z[z] = []
            coords_by_z[z].append(x)

        # Fill along z-axis for each x-column
        for x, z_values in coords_by_x.items():
            if len(z_values) > 1:
                min_z, max_z = min(z_values), max(z_values)
                for z_fill in range(min_z + 1, max_z):
                    if (x, y, z_fill) not in cubes_dict and (x, y, z_fill) not in solid_cubes_to_add:
                        # Find neighbors for color interpolation
                        z_before = max(z for z in z_values if z < z_fill)
                        z_after = min(z for z in z_values if z > z_fill)

                        color_before = cubes_dict.get((x, y, z_before))
                        color_after = cubes_dict.get((x, y, z_after))

                        if color_before and color_after:
                            ratio = (z_fill - z_before) / (z_after - z_before)
                            r = color_before[0] * (1 - ratio) + color_after[0] * ratio
                            g = color_before[1] * (1 - ratio) + color_after[1] * ratio
                            b = color_before[2] * (1 - ratio) + color_after[2] * ratio
                            solid_cubes_to_add[(x, y, z_fill)] = (r, g, b)

        # Fill along x-axis for each z-row
        for z, x_values in coords_by_z.items():
            if len(x_values) > 1:
                min_x, max_x = min(x_values), max(x_values)
                for x_fill in range(min_x + 1, max_x):
                    if (x_fill, y, z) not in cubes_dict and (x_fill, y, z) not in solid_cubes_to_add:
                        # Find neighbors for color interpolation
                        x_before = max(x for x in x_values if x < x_fill)
                        x_after = min(x for x in x_values if x > x_fill)

                        color_before = cubes_dict.get((x_before, y, z))
                        color_after = cubes_dict.get((x_after, y, z))

                        if color_before and color_after:
                            ratio = (x_fill - x_before) / (x_after - x_before)
                            r = color_before[0] * (1 - ratio) + color_after[0] * ratio
                            g = color_before[1] * (1 - ratio) + color_after[1] * ratio
                            b = color_before[2] * (1 - ratio) + color_after[2] * ratio
                            solid_cubes_to_add[(x_fill, y, z)] = (r, g, b)

    cubes_dict.update(solid_cubes_to_add)
    solid_cubes_list = [(x, y, z, r, g, b) for (x, y, z), (r, g, b) in cubes_dict.items()]

    newly_added_count = len(solid_cubes_list) - original_cube_count
    if newly_added_count > 0:
        print(f"      -> Added {newly_added_count} cubes to make model solid.")

    return solid_cubes_list


def generate_new_dog():
    """Generate a new, very detailed dog model in a sitting pose."""
    cubes = []

    def add_cube(x, y, z, r, g, b):
        """Add a single cube to the model, rounding coordinates."""
        cubes.append((int(round(x)), int(round(y)), int(round(z)), r, g, b))

    def add_sphere(cx, cy, cz, radius, r, g, b, density=1.0):
        """Add a spherical volume of cubes."""
        for x in range(int(cx - radius), int(cx + radius + 1)):
            for y in range(int(cy - radius), int(cy + radius + 1)):
                for z in range(int(cz - radius), int(cz + radius + 1)):
                    dist_sq = (x - cx)**2 + (y - cy)**2 + (z - cz)**2
                    if dist_sq <= radius**2:
                        if dist_sq > (radius-1.5)**2:
                            if hash(f"{x},{y},{z}") % 100 / 100.0 < density:
                                add_cube(x, y, z, r, g, b)
                        else:
                            add_cube(x, y, z, r, g, b)

    def add_ellipsoid(cx, cy, cz, rx, ry, rz, r, g, b):
        """Add an ellipsoidal volume of cubes."""
        for x in range(int(cx - rx), int(cx + rx + 1)):
            for y in range(int(cy - ry), int(cy + ry + 1)):
                for z in range(int(cz - rz), int(cz + rz + 1)):
                    if ((x-cx)/rx)**2 + ((y-cy)/ry)**2 + ((z-cz)/rz)**2 <= 1:
                        add_cube(x, y, z, r, g, b)

    # Colors
    brown = (0.6, 0.3, 0.1)
    dark_brown = (0.4, 0.2, 0.05)
    light_brown = (0.8, 0.5, 0.2)
    black = (0.1, 0.1, 0.1)

    # BODY (sitting pose)
    add_ellipsoid(0, 6, 0, 5, 8, 7, *brown) # Main body

    # LEGS (sitting)
    # Front legs
    add_ellipsoid(-3, 4, 5, 1.5, 3, 1.5, *dark_brown)
    add_ellipsoid(3, 4, 5, 1.5, 3, 1.5, *dark_brown)
    # Paws
    add_sphere(-3, 1, 5, 2, *light_brown)
    add_sphere(3, 1, 5, 2, *light_brown)
    
    # Back legs (haunches)
    add_ellipsoid(-5, 4, -2, 3, 4, 3, *brown)
    add_ellipsoid(5, 4, -2, 3, 4, 3, *brown)

    # HEAD
    add_sphere(0, 15, 2, 4.5, *brown)

    # SNOUT
    add_ellipsoid(0, 14, 6, 2, 1.5, 3, *light_brown)
    
    # NOSE
    add_sphere(0, 15, 8.5, 0.8, *black)

    # EARS (floppy)
    add_ellipsoid(-4, 17, 2, 1, 3, 2, *dark_brown)
    add_ellipsoid(4, 17, 2, 1, 3, 2, *dark_brown)

    # EYES
    add_sphere(-2, 16, 5, 0.8, *black)
    add_sphere(2, 16, 5, 0.8, *black)

    # TAIL (curved up)
    for i in range(12):
        x = 0
        y = 2 + i * 0.5
        z = -7 - i * 0.7
        r = 1.4 - i * 0.08
        add_sphere(x, y, z, r, *dark_brown)

    # Remove duplicates caused by rounding
    seen = set()
    unique_cubes = []
    for cube in cubes:
        coords = cube[:3]
        if coords not in seen:
            unique_cubes.append(cube)
            seen.add(coords)
            
    print(f"      -> Generated a new detailed dog model with {len(unique_cubes)} cubes.")
    return unique_cubes


def load_cat_model():
    """Load the cat model from file."""
    cat_path = "examples/models/cat.model"
    if not os.path.exists(cat_path):
        print(f"Warning: Cat model not found at {cat_path}")
        return []
    
    cubes = []
    with open(cat_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith('#'):
                parts = line.split()
                if len(parts) >= 3:
                    try:
                        x = int(round(float(parts[0])))
                        y = int(round(float(parts[1])))
                        z = int(round(float(parts[2])))
                        if len(parts) >= 6:
                            r = float(parts[3])
                            g = float(parts[4])
                            b = float(parts[5])
                        else:
                            r, g, b = 1.0, 1.0, 1.0
                        cubes.append((x, y, z, r, g, b))
                    except ValueError:
                        continue
    return cubes

def place_model_in_scene(model_cubes, position=(0, 0, 0), scale=1.0):
    """Place the model at a specific position in the scene, filling gaps from scaling."""
    
    offset_x, offset_y, offset_z = position
    int_scale = int(round(scale))

    if int_scale <= 1:
        # Original behavior for scale <= 1
        placed_cubes = []
        for x, y, z, r, g, b in model_cubes:
            new_x = x * scale + offset_x
            new_y = y * scale + offset_y
            new_z = z * scale + offset_z
            placed_cubes.append((new_x, new_y, new_z, r, g, b))
        return placed_cubes

    print(f"      -> Scaling model by {int_scale}x, filling gaps by interpolation...")
    
    source_cubes_dict = {(c[0], c[1], c[2]): (c[3], c[4], c[5]) for c in model_cubes}
    
    placed_cubes_dict = {}

    for (x, y, z), (r, g, b) in source_cubes_dict.items():
        # Add the cube itself
        px = int(round(x * scale)) + offset_x
        py = int(round(y * scale)) + offset_y
        pz = int(round(z * scale)) + offset_z
        placed_cubes_dict[(px, py, pz)] = (r, g, b)

        # Check for neighbors in positive directions and draw lines
        for dx, dy, dz in [(1, 0, 0), (0, 1, 0), (0, 0, 1)]:
            neighbor_coord = (x + dx, y + dy, z + dz)
            if neighbor_coord in source_cubes_dict:
                # Neighbor exists, draw a line of cubes
                neighbor_color = source_cubes_dict[neighbor_coord]
                for i in range(1, int_scale):
                    # Interpolate position
                    line_x = int(round(x * scale + i * dx)) + offset_x
                    line_y = int(round(y * scale + i * dy)) + offset_y
                    line_z = int(round(z * scale + i * dz)) + offset_z
                    
                    # Interpolate color
                    ratio = i / int_scale
                    line_r = r * (1 - ratio) + neighbor_color[0] * ratio
                    line_g = g * (1 - ratio) + neighbor_color[1] * ratio
                    line_b = b * (1 - ratio) + neighbor_color[2] * ratio

                    placed_cubes_dict[(line_x, line_y, line_z)] = (line_r, line_g, line_b)

    placed_cubes = [(x, y, z, r, g, b) for (x, y, z), (r, g, b) in placed_cubes_dict.items()]
    return placed_cubes

def write_model(filename, floor_cubes, structure_cubes, model_cubes):
    """Write cubes to a model file."""
    total = len(floor_cubes) + len(structure_cubes) + len(model_cubes)
    
    with open(filename, 'w') as f:
        f.write("# Generated scene with floor and animal model\n")
        f.write(f"# Total cubes: {total}\n")
        f.write(f"# Floor: {len(floor_cubes)}, Model: {len(model_cubes)}\n\n")
        
        f.write("# Floor with relief\n")
        for cube in floor_cubes:
            x, y, z, r, g, b = cube
            f.write(f"{x} {y} {z} {r:.3f} {g:.3f} {b:.3f}\n")
        
        if structure_cubes:
            f.write("\n# Random colored structures\n")
            for cube in structure_cubes:
                x, y, z, r, g, b = cube
                f.write(f"{x} {y} {z} {r:.3f} {g:.3f} {b:.3f}\n")
        
        if model_cubes:
            f.write("\n# Animal model\n")
            for cube in model_cubes:
                x, y, z, r, g, b = cube
                f.write(f"{x} {y} {z} {r:.3f} {g:.3f} {b:.3f}\n")

def main():
    import sys
    
    # Check if we should generate the huge scene
    if len(sys.argv) > 1 and sys.argv[1] == "--huge":
        print("=" * 60)
        print("GENERATING HUGE SCENE - 10x RESOLUTION")
        print("=" * 60)
        
        print("\n[1/3] Generating huge floor (1000x1000)...")
        print("      This will take 1-2 minutes...")
        floor = generate_floor(width=1000, depth=1000, y_base=0)
        print(f"      Generated {len(floor)} floor cubes")
        
        print("\n[2/3] Generating and placing new dog model (5x resolution)...")
        dog_cubes = generate_new_dog()
        if dog_cubes:
            # The new dog model is already quite dense, so we may not need fill_gaps
            print("      -> Densifying dog model before scaling...")
            dog_cubes = fill_gaps(dog_cubes)
            dog_cubes = make_solid(dog_cubes)

            # Place dog at the center of the scene with 5x scale
            dog_x, dog_z = 0, 0  # Place dog at center
            # Calculate floor height at dog position
            relief = math.sin(dog_x * 0.2) * 0.7 + math.cos(dog_z * 0.2) * 0.7
            relief += math.sin(dog_x * 0.05 + dog_z * 0.05) * 1.5
            relief += math.sin(dog_x * 0.15) * math.cos(dog_z * 0.15) * 0.5
            floor_y = int(relief * 1.5)

            placed_dog = place_cat_in_scene(dog_cubes, position=(dog_x, floor_y + 1, dog_z), scale=5.0)
            print(f"      Placed 5x scaled dog model with {len(placed_dog)} cubes at ({dog_x}, {floor_y + 1}, {dog_z})")
        else:
            placed_dog = []
            print("      Skipping dog model (not found)")
        
        output_file = "examples/models/scene_too_big.model"
        total_cubes = len(floor) + len(placed_dog)
        print(f"\n[3/3] Writing {total_cubes} cubes to {output_file}...")
        write_model(output_file, floor, [], placed_dog)  # Empty list for structures
        
        print("\n" + "=" * 60)
        print("HUGE SCENE GENERATION COMPLETE!")
        print("=" * 60)
        print(f"\nStatistics:")
        print(f"  • Floor cubes: {len(floor)}")
        print(f"  • Dog cubes: {len(placed_dog)} (5x resolution)")
        print(f"  • Total cubes: {total_cubes}")
        print(f"\nTo view the scene:")
        print(f"  cargo run --release --example hello {output_file}")
        print("=" * 60)
    else:
        print("=" * 60)
        print("GENERATING SCENE - FLOOR AND CAT (2x resolution)")
        print("=" * 60)
        
        print("\n[1/3] Generating floor (200x200)...")
        floor = generate_floor(width=200, depth=200, y_base=0)
        print(f"      Generated {len(floor)} floor cubes")
        
        print("\n[2/3] Loading and placing cat model...")
        cat_cubes = load_cat_model()
        if cat_cubes:
            cat_cubes = fill_gaps(cat_cubes)
            # Place cat at the center of the scene
            cat_x, cat_z = 0, 0  # Place cat at center
            # Calculate floor height at cat position
            relief = math.sin(cat_x * 0.2) * 0.7 + math.cos(cat_z * 0.2) * 0.7
            relief += math.sin(cat_x * 0.05 + cat_z * 0.05) * 1.5
            relief += math.sin(cat_x * 0.15) * math.cos(cat_z * 0.15) * 0.5
            floor_y = int(relief * 1.5)
            
            placed_cat = place_cat_in_scene(cat_cubes, position=(cat_x, floor_y + 1, cat_z), scale=1.0)
            print(f"      Placed cat model with {len(placed_cat)} cubes at ({cat_x}, {floor_y + 1}, {cat_z})")
        else:
            placed_cat = []
            print("      Skipping cat model (not found)")
        
        output_file = "examples/models/scene.model"
        total_cubes = len(floor) + len(placed_cat)
        print(f"\n[3/3] Writing {total_cubes} cubes to {output_file}...")
        write_model(output_file, floor, [], placed_cat)  # Empty list for structures
        
        print("\n" + "=" * 60)
        print("SCENE GENERATION COMPLETE!")
        print("=" * 60)
        print(f"\nStatistics:")
        print(f"  • Floor cubes: {len(floor)}")
        print(f"  • Cat cubes: {len(placed_cat)}")
        print(f"  • Total cubes: {total_cubes}")
        print(f"\nTo view the scene:")
        print(f"  cargo run --release --example hello {output_file}")
        print("=" * 60)

if __name__ == "__main__":
    main()
