def --env main [
    ...args: string  # Capture any additional arguments
] {
    mut only_html = false
    mut failures = []
    for arg in $args {
        if ($arg | str contains --ignore-case 'only-html') {
            $only_html = true
        }
    }

    mkdir out/original
    mkdir out/layout

    # Process each .dot file in inputs directory
    for file in (ls inputs/*.dot | get name) {
        # html flag
        if (($file | str contains --ignore-case 'html') == false) and $only_html {
            continue
        }

        let stem = ($file | path parse | get stem)
        let original_svg = $"out/original/($stem).svg"
        let layout_svg = $"out/layout/($stem).svg"

        let original_result = (dot -Tsvg $file -o $original_svg | complete)
        if $original_result.exit_code != 0 {
            $failures = ($failures | append $"original ($file): ($original_result.stderr)")
            continue
        }

        let layout_result = (cargo run --bin layout $file -o $layout_svg | complete)
        if $layout_result.exit_code != 0 {
            $failures = ($failures | append $"layout ($file): ($layout_result.stderr)")
        }
    }

    if (($failures | length) > 0) {
        print "Some files failed to render:"
        for failure in $failures {
            print $"- ($failure)"
        }
    }

    print "Wrote SVG comparison files under out/original and out/layout."
    print "View them at /scripts/svg_compare.html when serving the repository root."
}
