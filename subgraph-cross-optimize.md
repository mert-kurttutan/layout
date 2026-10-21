# Subgraph Cross Optimization Task

## Goal

Try different variations of the layout and edge crossing optimization algorithms
to improve inter-subgraph edge routing and crossing behavior.

The main criterion is whether edges between subgraphs/clusters become more
optimized, especially on the selected cluster test graphs. If the rendered
results get better and become close enough to the original Graphviz output, the
task is considered achieved.

## Evaluation Graphs

Use these DOT files as the primary evaluation inputs:

- `/home/kmert/projects/layout-parent/layout/inputs/cluster_1.dot`
- `/home/kmert/projects/layout-parent/layout/inputs/cluster_2.dot`

Use `/home/kmert/projects/layout-parent/layout/docs/sanders/tr-A03-96.mathjax.html`
as the source of truth for the relevant Sander-style clustered graph layout
ideas. Treat it as conceptual guidance when designing optimization algorithm
changes, but do not copy the paper literally into the implementation.

## Required Environment

Run the work inside the Nix development shell defined by this repository:

```bash
nix develop
```

Use the repository `flake.nix` as the source of the development environment.

Most of the rendering workflow is automated by:

```bash
nu scripts/optimizer-workflow.nu
```

The script can either be run from inside `nix develop`, or with `--nix` on the
rendering subcommands to run the underlying tools through `nix develop -c`.

## Baseline Rendering Workflow

Before making algorithm changes, compile/render both evaluation graphs with:

1. The local `layout` tool.
2. The original Graphviz `dot` tool.

Use clear output names so the baseline files are easy to compare later.

Example commands:

```bash
nu scripts/optimizer-workflow.nu baseline --nix
```

## Iteration Workflow

For each algorithm variation:

1. Change the layout and/or edge crossing optimization code.
2. Rebuild/render both evaluation graphs with the modified `layout` tool.
3. Save the outputs under `out/subgraph-cross-optimize/variant_$i/`,
   where `$i` is the iteration number.
4. Compare the variant SVGs against both:
   - the previous local `layout` result
   - the Graphviz `dot` result

Example variant output commands for iteration `1`:

```bash
nu scripts/optimizer-workflow.nu variant 1 --nix
```

The expected folder structure for generated variant SVGs is:

```txt
out/subgraph-cross-optimize/
  baseline/
    cluster_1-layout.svg
    cluster_1-graphviz.svg
    cluster_2-layout.svg
    cluster_2-graphviz.svg
  variant_1/
    cluster_1-layout.svg
    cluster_2-layout.svg
  variant_2/
    cluster_1-layout.svg
    cluster_2-layout.svg
  variant_$i/
    cluster_1-layout.svg
    cluster_2-layout.svg
```

## Success Criteria

The task is successful when:

- inter-subgraph edge crossings are visibly reduced or better organized on
  `cluster_1.dot` and `cluster_2.dot`;
- cluster/subgraph structure remains visually coherent;
- the output is close enough to the original Graphviz `dot` rendering for these
  graphs;
- the change does not obviously regress general graph layout behavior.

## Sander Method Gap Analysis

Reading `docs/sanders/tr-A03-96.mathjax.html` against the current implementation
shows that the existing code appears to implement only part of the compound
crossing reduction method.

The current code does:

- barycenter ordering with `node_order_wp_layerwise` and
  `node_order_ws_layerwise`;
- subgraph grouping by sorted traversal in `subgraph_order_by_p_layerwise`;
- complete-average-position ordering in `subgraph_order_by_p`;
- horizontal and vertical border node insertion after ordering.

The paper describes an additional major phase that is not fully represented in
the current implementation:

```txt
Calculate subgraph ordering graph OG.
Sort OG topologically and break cycles if necessary.
Then do sorted traversal using lambda_O,i.
```

This matters for `cluster_1.dot` because the issue is not only ordinary edge
crossing. The current local layout places the top-level sibling clusters roughly
as:

```txt
cluster_0, cluster_2, cluster_1
```

Graphviz places them more like:

```txt
cluster_0, cluster_1, cluster_2
```

The current `subgraph_order_by_p()` is only a heuristic based on average
positions. It does not build the paper's explicit "is left of" subgraph ordering
graph from adjacent layer relationships, and it does not topologically enforce
sibling-cluster order globally. This is likely one reason inter-subgraph edges
remain messy.

A second likely gap is dummy-node ownership for inter-subgraph long edges. In
`layout/src/topo/layout.rs`, long-edge connectors are created through
`add_connector_node`. The Sander paper says dummy nodes for an edge crossing
cluster boundaries should be assigned at least to the common containing
subgraph, or according to an explicit inside/outside routing strategy. If dummy
nodes are assigned to the wrong cluster, border crossings and routing
constraints become worse.

The next serious optimization variants should target:

1. Implementing a subgraph ordering graph for sibling clusters, then reordering
   layers from its topological order.
2. Fixing inter-subgraph dummy connector ownership to use the lowest common
   containing subgraph.
3. Comparing `cluster_1.dot` and `cluster_2.dot` against the Graphviz output
   after each variant.

Increasing the existing Sander iteration count alone is not enough. A trial with
more Sander sweeps produced byte-identical SVGs to the baseline for both
cluster evaluation graphs.

## Notes

- Keep every generated SVG from each variant under
  `out/subgraph-cross-optimize/`.
- Use `variant_$i` for variant directories, where `$i` is the numeric iteration
  count, for example `variant_1`, `variant_2`, and `variant_3`.
- Prefer small, comparable changes so it is clear which algorithm variation
  improves or worsens the result.
