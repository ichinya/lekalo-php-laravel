#!/usr/bin/env bash
# Runs the lekalo-php-laravel suite under every current PHP version
# through the official docker CLI images (lekalo-php#1: the framework
# package works with all current PHP versions wherever the language
# adapter does). The suite resolves the language kernel at
# ../lekalo-php/src, so the PARENT directory rides the mount.
#
#   scripts/version-matrix.sh
#
# LEKALO_DOCKER overrides the docker command. PHP 8.6 rides its `-rc`
# image until GA lands in the official library.
set -euo pipefail
cd "$(dirname "$0")/.."
parent="$(cd .. && pwd)"

VERSIONS=(8.0 8.1 8.2 8.3 8.4 8.5)
if docker image inspect php:8.6-cli >/dev/null 2>&1; then
  VERSIONS+=(8.6)
else
  VERSIONS+=("8.6-rc")
fi

DOCKER=(docker)
if ! docker info >/dev/null 2>&1; then
  DOCKER=(sudo docker)
fi
fail=0
for version in "${VERSIONS[@]}"; do
  tag="php:${version}-cli"
  if output="$("${DOCKER[@]}" run --rm -v "$parent:/work" -w "/work/$(basename "$PWD")" "$tag" php -n tests/laravel.php 2>&1)"; then
    echo "php ${version} tests/laravel.php: $(echo "$output" | tail -1)"
  else
    echo "php ${version} tests/laravel.php: FAILED"
    echo "$output" | tail -5
    fail=1
  fi
done
exit $fail
