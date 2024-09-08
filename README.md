# RustGrapher

A library to simulate and visualize a [force directed graph](https://en.wikipedia.org/wiki/Force-directed_graph_drawing) in rust.
![plot](./example_images/example.gif)

The initial goal of this project was to render a graph of all nodes in wikipedia now it has transformed into a library for visualizing graphs.

Currently `RustGrapher` doesn't utilize the GPU for it's calculations but it's planned for future updates.

The library only supports 2D graphs. (For now)

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
- `Space` - Start/Pause simulation
- `Scroll Wheel` - Zoom in or out
- `W`, `A`, `S` and `D` - to move the camera

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
