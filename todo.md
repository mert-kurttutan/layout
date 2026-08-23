# TODO

## HTML Label Support

### Handle bad image paths without panicking

Current state:

- `layout/src/gv/html.rs` calls `get_image_size(...).unwrap()` in `Image::width`, `Image::height`, and `Image::size`.
- `layout/src/core/utils.rs` returns `Result<(u32, u32), Error>` from `get_image_size`, but that error is discarded by the unwraps.
- `layout/src/std_shapes/render.rs` calls `img.size()` while rendering image cells, so a missing or invalid image can panic during render.

Required changes:

- Change `Image::{width,height,size}` to return `Result` or cache a fallible size during HTML parsing/grid construction.
- Propagate image-size failures through `parse_html_string`/`HtmlGrid` construction, or add a documented placeholder-size fallback.
- Update `GraphBuilder::get_shape_from_attributes` so HTML parse/image errors are not hidden behind `unwrap()`.
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

### Render or document parsed-but-unused HTML attributes

Current state:

- `layout/src/gv/html.rs` parses/stores `href`, `id`, `target`, `tooltip`, `sides`, `fixedsize`, and `gradientangle` on table/cell attributes.
- Rendering mostly ignores them in `layout/src/std_shapes/render.rs`.
- SVG primitives already accept raw `properties: Option<String>` in `layout/src/core/format.rs` and `layout/src/backends/svg.rs`.

Required changes:

- For `id`, `href`, `target`, and `tooltip`, build SVG-safe property strings and pass them into `draw_rect`, `draw_text`, or wrapper groups.
- Escape attribute values before inserting them into SVG. Current `properties` strings are raw.
- For `sides`, change table/cell border rendering from one `draw_rect` to selective line drawing.
- For `fixedsize`, make table/cell sizing respect explicit `WIDTH`/`HEIGHT` and clipping/overflow rules.
- For `gradientangle`, decide whether to implement SVG gradients or explicitly document it as unsupported.
- Add per-attribute tests that inspect SVG output.

### Improve malformed HTML error handling

Current state:

- `GraphBuilder::get_shape_from_attributes` calls `parse_html_string(val).unwrap()`.
- Some parser methods in `layout/src/gv/html.rs` use `panic!` for supposedly impossible states.
- `GraphBuilder::get()` currently returns `VisualGraph`, so builder errors cannot be propagated.

Required changes:

- Change `GraphBuilder::get()` to return `Result<VisualGraph, String>`, or add a separate fallible builder path to preserve API compatibility.
- Replace `parse_html_string(...).unwrap()` with error propagation.
- Audit `panic!` calls in `layout/src/gv/html.rs` and keep only true internal invariant failures.
- Update CLI error reporting in `src/bin/layout.rs` to print builder/render errors cleanly.
- Add malformed HTML tests for unclosed tags, invalid table structure, and invalid image inputs.
