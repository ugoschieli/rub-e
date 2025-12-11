# ETIB Model Files

This directory contains model files for the ETIB game engine.

## Available Models

- **simple.model** - A simple arrangement of 7 cubes
- **cat.model** - A detailed cat model made of ~1,200 cubes
- **scene.model** - A procedurally generated scene with relief floor and cat model (~11,000 cubes)

## Model Format

Each line represents a cube in one of two formats:
- Position only: `x y z` (color defaults to white: 1.0 1.0 1.0)
- Position and color: `x y z r g b`
- Lines starting with `#` are comments
- Empty lines are ignored

Example:
```
# This is a comment
0.0 0.0 0.0 1.0 0.0 0.0   # Red cube at origin
1.0 0.0 0.0                # White cube (default)
-1.0 1.0 0.0 0.0 0.0 1.0   # Blue cube
```

## Running Models

To run a model with the example program:

```bash
# Simple cubes
cargo run --release --example hello examples/models/simple.model

# Cat model
cargo run --release --example hello examples/models/cat.model

# Generated scene
cargo run --release --example hello examples/models/scene.model
```

## Camera Controls

Once running:
- **WASD** or **Arrow keys**: Move forward/back/strafe left/right
- **Space**: Move up
- **Shift**: Move down
- **Mouse**: Look around (press C to grab cursor, ESC to release)
- **Mouse scroll**: Adjust movement speed

## Generating New Scenes

You can regenerate the scene:

```bash
python3 examples/generate_scene.py
```

The generated scene includes:
- **Floor**: 100x100 cube grid with multi-layered sinusoidal relief (10,000 cubes)
  - Height variations from -4 to +4
  - Green grass-like color with height-based shading
  - Multiple sine wave patterns for natural-looking terrain
  
- **Cat Model**: The detailed cat model placed on the floor (1,232 cubes)
  - Positioned near center front for easy viewing
  - Full cat model with all details from cat.model
  - Automatically placed on the terrain surface

**Total**: ~11,000 cubes per scene

**Camera**: Automatically positioned high and far back (0, 30, 80) with increased view distance and movement speed for comfortable exploration of the scene.

## Generating Huge Scene (1 Million Cubes)

To generate the huge scene with 10x resolution and 10x scaled cat:

```bash
python3 examples/generate_scene.py --huge
```

This creates:
- **scene_too_big.model**: 1,000,000 floor cubes + 1,232 cat cubes (10x scale)
- **Total**: 1,001,232 cubes (24x more than regular scene!)
- **File size**: ~28 MB
- **Purpose**: Stress testing GPU instancing optimization

**Warning**: This scene is very demanding and best run with the GPU instancing optimization enabled.
Expected performance: 30-100+ FPS with instancing (vs <1 FPS without).
