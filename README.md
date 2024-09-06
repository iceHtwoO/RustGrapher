# RustGrapher

A library to simulate and visualize a [force directed graph](https://en.wikipedia.org/wiki/Force-directed_graph_drawing) in rust.
![plot](./example_images/example.gif)

> [!NOTE]
> Project is Work In Progress

The initial goal of this project was to render a graph of all nodes in wikipedia now it has transformed into a library for visualizing graphs.

Currently `RustGrapher` doesn't utilize the GPU for it's calculations but it's planned for future updates.

The library only supports 2D graphs. (For now)

## Algorithms

### Barnes–Hut

Barnes-Hut is a approximation algorithm for n-body simulation.

The force directed graph calculates for each node the repulsive force to other nodes which will lead to a complexity of $O(n^2)$.

Using a k-d Tree(Quadtree) the Barnes-Hut algorithm groups far away nodes and only calculates the repulsive force once.This results in a complexity of $O(nlogn)$.

## Performance

On a Ryzen 7 3700X the library can calculate 2000 simulation steps per second at 1000 Nodes. (Using 16 Physics threads)
