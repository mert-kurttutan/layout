# TODO

## HTML Label Support

- Handle bad image paths without panicking. Current image sizing uses unwraps, so missing `SRC` files can crash rendering.
- Make image hrefs portable when SVGs are written outside the repo. Options include absolute paths, copied assets, or embedded data URIs.
- Render HTML edge labels. The parser stores HTML `label=<...>` values on edges, but builder currently only renders plain string edge labels.
- Render `headlabel` and `taillabel` for edges.
- Decode HTML entities such as `&amp;`, `&lt;`, `&gt;`, and `&quot;` in HTML-like labels.
- Render or intentionally document currently parsed-but-unused HTML attributes: `href`, `id`, `target`, `tooltip`, `sides`, `fixedsize`, and `gradientangle`.
- Add graph and cluster label rendering for HTML labels.
- Improve malformed HTML error handling. Avoid builder-side `parse_html_string(...).unwrap()` panics and return actionable errors.
