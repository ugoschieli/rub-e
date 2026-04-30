#!/usr/bin/env python3
"""
Generate a completely new 1000x1000 colored scene with procedural terrain with some relief.
"""

import random
import math
import os

def generate_terrain(width=1000, depth=1000, y_base=0):
    """Generate a completely new procedural terrain with more dramatic relief and a different color palette."""
    cubes = []
    
    # Use different frequencies for a new look
    freq_x1 = random.uniform(0.08, 0.12)
    freq_z1 = random.uniform(0.08, 0.12)
    freq_x2 = random.uniform(0.02, 0.04)
    freq_z2 = random.uniform(0.02, 0.04)
    freq_xz = random.uniform(0.05, 0.07)
    
    # Introduce random offsets to shift the whole pattern
    offset_x = random.uniform(0, 1000)
    offset_z = random.uniform(0, 1000)

    for x in range(-width//2, width//2):
        for z in range(-depth//2, depth//2):
            # More dramatic relief with different combination of waves
            n_x = (x + offset_x)
            n_z = (z + offset_z)

            relief1 = math.sin(n_x * freq_x1) * math.cos(n_z * freq_z1) * 2.5
            relief2 = math.sin(n_x * freq_x2) * math.cos(n_z * freq_z2) * 4.0
            relief3 = math.sin(n_x * freq_xz + n_z * freq_xz) * 2.0
            
            total_relief = relief1 + relief2 + relief3
            y = y_base + int(total_relief * 2.0)  # More height variation
            
            # Sand yellow — uniform across all ground cubes
            r, g, b = 0.85, 0.75, 0.45
            
            cubes.append((x, y, z, r, g, b))
    
    return cubes

def generate_trees(terrain_cubes, num_trees=200):
    """Generate trees to place on the terrain."""
    print("      -> Generating trees...")
    trees_cubes = []
    
    # Create a height map for quick lookup
    height_map = {(c[0], c[2]): c[1] for c in terrain_cubes}
    
    # Get terrain boundaries
    min_x = min(c[0] for c in terrain_cubes)
    max_x = max(c[0] for c in terrain_cubes)
    min_z = min(c[2] for c in terrain_cubes)
    max_z = max(c[2] for c in terrain_cubes)
    
    # Tree colors
    trunk_color = (0.4, 0.2, 0.1) # dark brown
    leaves_color = (0.1, 0.5, 0.1) # dark green

    for _ in range(num_trees):
        # Choose a random spot
        x = random.randint(min_x, max_x)
        z = random.randint(min_z, max_z)
        
        # Get ground level
        y = height_map.get((x, z))
        
        if y is None:
            continue
            
        # Trees grow on "ground" level, not high snowy peaks
        # Let's say y is between 2 and 10
        if not (2 <= y <= 10):
            continue

        # Generate a tree
        tree_height = random.randint(5, 15)
        
        # Trunk
        for i in range(tree_height):
            trees_cubes.append((x, y + i + 1, z, *trunk_color))
            
        # Leaves (a cone)
        leaves_height = random.randint(4, 8)
        base_radius = leaves_height / 2
        
        for i in range(leaves_height):
            radius = base_radius * (1 - (i / leaves_height))
            leaf_y = y + tree_height + i + 1
            
            # Create a circular layer of leaves
            for dx in range(-int(radius), int(radius) + 1):
                for dz in range(-int(radius), int(radius) + 1):
                    if dx**2 + dz**2 <= radius**2:
                        # Add some randomness to make it look more natural
                        if random.random() < 0.8:
                           trees_cubes.append((x + dx, leaf_y, z + dz, *leaves_color))

    print(f"      -> Generated {num_trees} trees with {len(trees_cubes)} cubes.")
    return trees_cubes

def generate_structures(terrain_cubes, num_structures=50):
    """Generate simple structures to place on the terrain."""
    print("      -> Generating structures...")
    structure_cubes = []
    
    height_map = {(c[0], c[2]): c[1] for c in terrain_cubes}
    
    min_x = min(c[0] for c in terrain_cubes)
    max_x = max(c[0] for c in terrain_cubes)
    min_z = min(c[2] for c in terrain_cubes)
    max_z = max(c[2] for c in terrain_cubes)

    for _ in range(num_structures):
        x = random.randint(min_x, max_x)
        z = random.randint(min_z, max_z)
        
        y = height_map.get((x, z))
        
        if y is None:
            continue

        # Structures can appear at various altitudes, but avoid water if y < 0
        if y < 0:
            continue

        structure_type = random.choice(['pyramid', 'sphere', 'crystal_cluster'])

        if structure_type == 'pyramid':
            base_size = random.randint(3, 7)
            height = random.randint(base_size, base_size * 2)
            color = (0.6, 0.6, 0.6) # Stone color
            for i in range(height):
                size = max(1, base_size - i // 2)
                for dx in range(-size + 1, size):
                    for dz in range(-size + 1, size):
                        # add some random missing blocks for ruined look
                        if random.random() < 0.9:
                            structure_cubes.append((x + dx, y + i + 1, z + dz, *color))

        elif structure_type == 'sphere':
            radius = random.randint(4, 8)
            y_offset = random.randint(radius, radius * 3) # Make it float
            color = (0.9, 0.9, 0.9) # White/marble
            for sx in range(x - radius, x + radius):
                for sy in range(y + y_offset - radius, y + y_offset + radius):
                    for sz in range(z - radius, z + radius):
                        if (sx-x)**2 + (sy-(y+y_offset))**2 + (sz-z)**2 <= radius**2:
                           if random.random() < 0.9: # ruined look
                                structure_cubes.append((sx, sy, sz, *color))
        
        elif structure_type == 'crystal_cluster':
            num_crystals = random.randint(3, 8)
            cluster_radius = 5
            color = (0.3, 0.8, 0.9) # Icy blue/crystal
            for _ in range(num_crystals):
                cx = x + random.randint(-cluster_radius, cluster_radius)
                cz = z + random.randint(-cluster_radius, cluster_radius)
                cy = height_map.get((cx, cz))
                if cy is None:
                    continue
                
                crystal_height = random.randint(4, 12)
                base_size = random.uniform(0.5, 2.5)
                for i in range(crystal_height):
                    # Make it pointy
                    size = max(0, base_size - i * 0.2)
                    for dx in range(-int(size), int(size)+1):
                        for dz in range(-int(size), int(size)+1):
                            if dx**2 + dz**2 <= size**2:
                                structure_cubes.append((cx+dx, cy+i+1, cz+dz, *color))


    print(f"      -> Generated {num_structures} structures with {len(structure_cubes)} cubes.")
    return structure_cubes

def find_castle_location(terrain_cubes, castle_footprint, search_radius=100):
    """Find a large, flat area near the center to build a castle."""
    print("      -> Finding a suitable location for a castle near the center...")
    height_map = {(c[0], c[2]): c[1] for c in terrain_cubes}
    
    best_location = None
    min_variance = float('inf')

    # Search for a good spot around the center
    for _ in range(500): # Check 500 random locations in the search radius
        x = random.randint(-search_radius, search_radius)
        z = random.randint(-search_radius, search_radius)

        # check bounds to avoid castle being partially off map
        if not (-500 + castle_footprint < x < 500 - castle_footprint and 
                -500 + castle_footprint < z < 500 - castle_footprint):
            continue

        area_heights = []
        possible = True
        for dx in range(-castle_footprint//2, castle_footprint//2):
            for dz in range(-castle_footprint//2, castle_footprint//2):
                y = height_map.get((x + dx, z + dz))
                if y is None or not (0 <= y <= 15): # Must be on land, not too high
                    possible = False
                    break
                area_heights.append(y)
            if not possible:
                break
        
        if not possible or len(area_heights) < (castle_footprint**2) * 0.8:
            continue

        mean = sum(area_heights) / len(area_heights)
        variance = sum((h - mean)**2 for h in area_heights) / len(area_heights)

        if variance < min_variance:
            min_variance = variance
            base_y = int(mean)
            best_location = (x, base_y, z)
            # If we find a very good spot, don't search anymore
            if min_variance < 1.0:
                break

    if best_location:
        print(f"      -> Found a castle location with variance {min_variance:.2f} at {best_location}")
        return best_location
    else:
        print("      -> Could not find a suitable location for a castle near the center.")
        return None

def generate_castle(terrain_cubes):
    """Generates a castle on a suitable flat area."""
    castle_cubes = []
    
    # Castle dimensions
    footprint = 50 # a 50x50 area
    tower_radius = 6
    tower_height = 25
    wall_height = 15
    keep_size = 12
    keep_height = 40

    location = find_castle_location(terrain_cubes, footprint)
    if location is None:
        return [] # No suitable place found

    cx, cy, cz = location
    color = (0.5, 0.5, 0.55) # Gray stone

    def build_cylinder(x, y, z, radius, height, color):
        """Builds a cylinder of cubes."""
        for i in range(height):
            for dx in range(-radius, radius+1):
                for dz in range(-radius, radius+1):
                    if dx**2 + dz**2 <= radius**2:
                        castle_cubes.append((x+dx, y+i, z+dz, *color))

    def build_wall(x1, z1, x2, z2, y_base, height, color):
        """Builds a wall between two points."""
        # This is a simple line-drawing algorithm (bresenham or similar could be used for non-axis-aligned)
        # For simplicity, I'll assume axis-aligned walls
        if x1 == x2: # Vertical wall
            for z in range(min(z1, z2), max(z1, z2) + 1):
                for i in range(height):
                    # Add thickness
                    castle_cubes.append((x1-1, y_base+i, z, *color))
                    castle_cubes.append((x1, y_base+i, z, *color))
                    castle_cubes.append((x1+1, y_base+i, z, *color))
        elif z1 == z2: # Horizontal wall
            for x in range(min(x1, x2), max(x1, x2) + 1):
                for i in range(height):
                    castle_cubes.append((x, y_base+i, z1-1, *color))
                    castle_cubes.append((x, y_base+i, z1, *color))
                    castle_cubes.append((x, y_base+i, z1+1, *color))

    # Corner towers
    c = footprint // 2
    towers = [
        (cx - c, cz - c),
        (cx + c, cz - c),
        (cx + c, cz + c),
        (cx - c, cz + c)
    ]
    for tx, tz in towers:
        build_cylinder(tx, cy, tz, tower_radius, tower_height, color)

    # Walls
    build_wall(towers[0][0], towers[0][1], towers[1][0], towers[1][1], cy, wall_height, color)
    build_wall(towers[1][0], towers[1][1], towers[2][0], towers[2][1], cy, wall_height, color)
    build_wall(towers[2][0], towers[2][1], towers[3][0], towers[3][1], cy, wall_height, color)
    build_wall(towers[3][0], towers[3][1], towers[0][0], towers[0][1], cy, wall_height, color)

    # Main Keep (a square building in the middle)
    for dx in range(-keep_size, keep_size+1):
        for dz in range(-keep_size, keep_size+1):
            for i in range(keep_height):
                 castle_cubes.append((cx+dx, cy+i, cz+dz, *color))
    
    print(f"      -> Generated a castle with {len(castle_cubes)} cubes.")
    return castle_cubes






def write_world_model(filename, terrain_cubes, life_cubes):
    """Write cubes to a model file."""
    total_cubes = len(terrain_cubes) + len(life_cubes)
    with open(filename, 'w') as f:
        f.write(f"# Generated new world with life\n")
        f.write(f"# Total cubes: {total_cubes}\n\n")
        
        f.write("# Terrain\n")
        for cube in terrain_cubes:
            x, y, z, r, g, b = cube
            f.write(f"{x} {y} {z} {r:.3f} {g:.3f} {b:.3f}\n")
            
        if life_cubes:
            f.write("\n# Life (trees, etc.)\n")
            for cube in life_cubes:
                x, y, z, r, g, b = cube
                f.write(f"{x} {y} {z} {r:.3f} {g:.3f} {b:.3f}\n")

def main():
    print("=" * 60)
    print("GENERATING COMPLETELY NEW 1000x1000 SCENE WITH LIFE AND STRUCTURES")
    print("=" * 60)
    
    print("\n[1/4] Generating new procedural terrain (1000x1000)...")
    print("      This may take 1-2 minutes...")
    terrain = generate_terrain(width=1000, depth=1000, y_base=0)
    print(f"      Generated {len(terrain)} terrain cubes")

    print("\n[2/4] Adding life to the scene...")
    life_cubes = generate_trees(terrain, num_trees=500)
    
    print("\n[3/4] Adding structures to the scene...")
    structure_cubes = generate_structures(terrain, num_structures=50)
    
    print("\n[4/4] Adding a castle to the scene...")
    castle_cubes = generate_castle(terrain)

    all_structures = structure_cubes + castle_cubes
    all_life_and_structures = life_cubes + all_structures

    output_file = "examples/models/new_world_with_centered_castle.model"
    total_cubes = len(terrain) + len(all_life_and_structures)
    print(f"\nWriting {total_cubes} cubes to {output_file}...")
    write_world_model(output_file, terrain, all_life_and_structures)
    
    print("\n" + "=" * 60)
    print("NEW WORLD GENERATION COMPLETE!")
    print("=" * 60)
    print(f"\nTo view the scene:")
    print(f"  cargo run --release --example hello {output_file}")
    print("=" * 60)

if __name__ == "__main__":
    main()
