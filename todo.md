# TODO

## HTML Label Support

### Image error handling

Current state:

- Image dimensions are loaded during HTML parsing in
  `layout/src/gv/html.rs::Image::from_tag_attr_list`.
- Missing `SRC` attributes return a builder error and are covered by
  `build_html_image_without_src_returns_error`.
- Missing or invalid image files are converted into builder errors with the
  `Could not read HTML image source ...` message, so they no longer panic
  during rendering.

Required changes:

- Add a test for a present `SRC` attribute that points to a missing file. It
  should assert the builder error path, not a panic.

### Make image hrefs portable

Current state:

- `layout/src/backends/svg.rs::draw_image` emits the `SRC` value directly as `href="..."`.
- Example: rendering to `/tmp/html_img.svg` produces `href="docs/sample.png"`, which is no longer relative to the output file.
- `RenderBackend::draw_image` only receives the source string, and the CLI does
  not pass input/output path context into rendering.

Required changes:

- Decide the policy: preserve input path, absolutize paths, copy referenced files next to the SVG, or embed image bytes as data URIs.
- If copying or absolutizing, pass input/output path context from `src/bin/layout.rs` into rendering.
- If embedding, extend `get_image_size`/image utilities to read bytes and MIME type, then make `SVGWriter::draw_image` emit `data:image/png;base64,...`.
- Add tests for output written outside the repo.

### Table grid invariant panic

Current state:

- `layout/src/gv/html.rs` still has one `panic!` in table grid construction
  when a row/column span calculation occupies the same cell twice.
- This panic represents an unrecoverable layout algorithm failure, not an input
  validation error. Malformed HTML should already fail earlier through returned
  builder errors.
