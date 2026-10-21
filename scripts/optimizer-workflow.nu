#!/usr/bin/env nu

const out_root = "out/subgraph-cross-optimize"
const graphs = [
    {
        name: "cluster_1",
        path: "/home/kmert/projects/layout-parent/layout/inputs/cluster_1.dot",
    },
    {
        name: "cluster_2",
        path: "/home/kmert/projects/layout-parent/layout/inputs/cluster_2.dot",
    },
]

def render-layout [input: string, output: string, use_nix: bool] {
    if $use_nix {
        nix develop -c cargo run --bin layout -- $input -o $output
    } else {
        cargo run --bin layout -- $input -o $output
    }
}

def render-graphviz [input: string, output: string, use_nix: bool] {
    if $use_nix {
        nix develop -c dot -Tsvg $input -o $output
    } else {
        dot -Tsvg $input -o $output
    }
}

def ensure-viewer-link [] {
    ln -sfn ../out scripts/out
}

def print-output [path: string] {
    print $"wrote ($path)"
}

def "main baseline" [
    --nix
] {
    let output_dir = $"($out_root)/baseline"
    mkdir $output_dir
    ensure-viewer-link

    for graph in $graphs {
        let layout_svg = $"($output_dir)/($graph.name)-layout.svg"
        let graphviz_svg = $"($output_dir)/($graph.name)-graphviz.svg"

        render-layout $graph.path $layout_svg $nix
        print-output $layout_svg

        render-graphviz $graph.path $graphviz_svg $nix
        print-output $graphviz_svg
    }
}

def "main variant" [
    iteration: int
    --nix
] {
    if $iteration < 1 {
        error make {
            msg: "iteration must be greater than or equal to 1"
        }
    }

    let output_dir = $"($out_root)/variant_($iteration)"
    mkdir $output_dir
    ensure-viewer-link

    for graph in $graphs {
        let layout_svg = $"($output_dir)/($graph.name)-layout.svg"
        render-layout $graph.path $layout_svg $nix
        print-output $layout_svg
    }
}

def "main all" [
    iteration: int
    --nix
] {
    main baseline --nix=$nix
    main variant $iteration --nix=$nix
}

def "main list" [] {
    if not ($out_root | path exists) {
        print $"no outputs found under ($out_root)"
        return
    }

    ls -a $out_root
}

def "main serve" [
    --port: int = 8000
] {
    ensure-viewer-link
    print $"Serving http://localhost:($port)/"
    python3 -m http.server $port --directory scripts
}

def main [] {
    print "optimizer workflow"
    print ""
    print "subcommands:"
    print "  baseline [--nix]        render baseline layout and Graphviz SVGs"
    print "  variant <i> [--nix]     render layout SVGs into variant_i"
    print "  all <i> [--nix]         render baseline and variant_i"
    print "  list                    list generated output directories"
    print "  serve [--port <port>]   serve scripts/ for SVG comparison"
    print ""
    print "examples:"
    print "  nu scripts/optimizer-workflow.nu baseline --nix"
    print "  nu scripts/optimizer-workflow.nu variant 1 --nix"
    print "  nu scripts/optimizer-workflow.nu serve --port 8000"
}
