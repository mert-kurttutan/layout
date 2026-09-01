#!/bin/bash

set -u

ONLY_HTML=false
SERVE=true
PORT=8000

for arg in "$@"; do
  case "${arg,,}" in
    only-html | --only-html)
      ONLY_HTML=true
      ;;
    --no-serve)
      SERVE=false
      ;;
    --port=*)
      PORT="${arg#*=}"
      ;;
  esac
done

mkdir -p out/original out/layout
ln -sfn ../out scripts/out

FAILURES=()

for file in ./inputs/*.dot; do
  if [[ "$ONLY_HTML" == true && "$file" != *html* ]]; then
    continue
  fi

  stem=$(basename "$file" .dot)
  original_svg="out/original/$stem.svg"
  layout_svg="out/layout/$stem.svg"

  if ! original_error=$(dot -Tsvg "$file" -o "$original_svg" 2>&1); then
    FAILURES+=("original $file: $original_error")
    continue
  fi

  if ! layout_error=$(cargo run --bin layout "$file" -o "$layout_svg" 2>&1); then
    FAILURES+=("layout $file: $layout_error")
  fi
done

if [[ "${#FAILURES[@]}" -gt 0 ]]; then
  echo "Some files failed to render:"
  for failure in "${FAILURES[@]}"; do
    echo "- $failure"
  done
fi

echo "Wrote SVG comparison files under out/original and out/layout."

if [[ "$SERVE" == true ]]; then
  echo "Serving http://localhost:$PORT/"
  python3 -m http.server "$PORT" --directory scripts
else
  echo "View them at / when serving the scripts directory."
fi
