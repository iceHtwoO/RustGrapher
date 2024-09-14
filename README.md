# RustGrapher

A library to simulate and visualize a 2D [force directed graph](https://en.wikipedia.org/wiki/Force-directed_graph_drawing) in rust.
![plot](./example_images/example.gif)

## Features

- Render PetGraphs in 2D
- Physics based positioning via a Force Directed Graph
- Drag Nodes to a new position

## Algorithms

### Barnes–Hut

Barnes-Hut is a approximation algorithm for n-body simulation.

The force directed graph calculates for each node the repulsive force to other nodes which will lead to a complexity of $O(n^2)$.

Using a k-d Tree(Quadtree) the Barnes-Hut algorithm groups far away nodes and only calculates the repulsive force once.This results in a complexity of $O(nlogn)$.

## Performance

> [!TIP]
> Run the project with `--release` for the best performance(~10x).

On a Ryzen 7 3700X the library can calculate 2000 simulation steps per second at 1000 Nodes. (Using 16 Physics threads)

## Controls

- `return` - Centers the camera on the average poisson of all nodes.
- `space` - Start/Pause simulation
- `scroll wheel` - Zoom in or out
- `W`, `A`, `S` and `D` - to move the camera
- `Click` and `drag` - move nodes

## Usage

```rust
    // Build a PetGraph
    let mut rng = rand::thread_rng();
    let graph: petgraph::Graph<(), (), Directed> =
        petgraph_gen::barabasi_albert_graph(&mut rng, 1000, 1, None);

    // Configure the simulator
    let simulator = SimulatorBuilder::new()
        .delta_time(0.01)
        .freeze_threshold(-1.0)
        .build(graph.into());

    // Start the renderer
    let renderer = Renderer::new(simulator);
    renderer.create_window();
```

## Examples

- [Basic Barabasi Albert Graph](examples/basic.rs)
- [Section of Wikipedia Graph](examples/wikipedia.rs)
- [Headless Graph](examples/headless.rs)
