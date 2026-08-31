# TODO

## HTML Label Support

### Handle bad image paths without panicking

Current state:

- Image dimensions are loaded during HTML parsing and propagated as builder
  errors, so missing or invalid image paths no longer panic during rendering.

Required changes:

- Add a test for a missing `SRC` file that asserts an error path or placeholder behavior, not a panic.

### Make image hrefs portable

Current state:

- `layout/src/backends/svg.rs::draw_image` emits the `SRC` value directly as `href="..."`.
- Example: rendering to `/tmp/html_img.svg` produces `href="docs/sample.png"`, which is no longer relative to the output file.

Required changes:

- Decide the policy: preserve input path, absolutize paths, copy referenced files next to the SVG, or embed image bytes as data URIs.
- If copying or absolutizing, pass input/output path context from `src/bin/layout.rs` into rendering. The current `RenderBackend::draw_image` only receives the source string.
- If embedding, extend `get_image_size`/image utilities to read bytes and MIME type, then make `SVGWriter::draw_image` emit `data:image/png;base64,...`.
- Add tests for output written outside the repo.

### Improve malformed HTML error handling

Current state:

- `GraphBuilder::try_get()` returns builder errors, and the CLI uses that path.
- HTML parse errors are propagated from graph, node, and edge labels.
- DOT parser token advancement errors are returned instead of panicking.
- Malformed HTML tests cover unclosed tags, invalid table structure, and invalid
  image inputs.

Required changes:

- Audit `panic!` calls in `layout/src/gv/html.rs` and keep only true internal invariant failures.
