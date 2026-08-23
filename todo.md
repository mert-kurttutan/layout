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

### Render `headlabel` and `taillabel`

Current state:

- The parser can store these attributes as plain or HTML `DotString`.
- `GraphBuilder::get_arrow_from_attributes` ignores `headlabel` and `taillabel`.
- `Arrow` only has one central label field.

Required changes:

- Extend `Arrow` with optional head and tail label fields.
- Decide layout semantics: labels should be placed near destination/source ends, not on the midpoint text path.
- Update `render_arrow` or add generated label elements during graph lowering.
- Add tests for plain and HTML `headlabel`/`taillabel`.

### Decode HTML entities

Current state:

- DOT HTML content is stored raw in `DotString::HtmlString`.
- `layout/src/gv/html.rs` treats text like `&amp;` as literal text.
- `SVGWriter::draw_text` escapes output text, so raw `&amp;` can become `&amp;amp;` in SVG.

Required changes:

- Add entity decoding in the HTML parser text path, likely in `HtmlParser::read_html_text`.
- Support at least XML built-ins: `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`.
- Consider numeric entities (`&#123;`, `&#x7b;`) and Graphviz-supported named entities as follow-up scope.
- Add tests that render `A &amp; B` as `A &amp; B` in SVG source, not `A &amp;amp; B`.

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

### Add graph and cluster label rendering

Current state:

- Graph-level `label=<...>` parses as a graph `AttrStmt`.
- `GraphBuilder` records top-level graph attributes in `global_state`, but `VisualGraph` has no graph-label render path.
- Subgraphs/clusters are parsed recursively, but the current builder flattens them and does not preserve cluster boxes/labels.

Required changes:

- Add graph-level metadata to `VisualGraph`, including optional label content and style.
- Render graph labels before or after nodes based on desired z-order.
- Defer cluster HTML labels until subgraph/cluster layout support exists, because cluster boxes need bounds and nesting.
- Add tests for top-level graph labels now; add cluster tests after subgraph support lands.

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
