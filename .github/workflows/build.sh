#!/bin/bash

set -euo pipefail

if [[ "$_BUILD_BRANCH" == "refs/heads/main" || "$_BUILD_BRANCH" == "refs/tags/canary" ]]; then
  export _IS_BUILD_CANARY="true"
  export _IS_GITHUB_RELEASE="true"
elif [[ "$_BUILD_BRANCH" == refs/tags/* ]]; then
  _BUILD_VERSION="${_BUILD_VERSION%-*}-0"
  export _BUILD_VERSION
  export _IS_GITHUB_RELEASE="true"
fi
export _RELEASE_VERSION="v${_BUILD_VERSION}"

echo "--------------------------------------------------"
echo "RELEASE VERSION: $_RELEASE_VERSION"
echo "--------------------------------------------------"

echo "_BUILD_VERSION=${_BUILD_VERSION}" >> "${GITHUB_ENV}"
echo "_RELEASE_VERSION=${_RELEASE_VERSION}" >> "${GITHUB_ENV}"
echo "_IS_BUILD_CANARY=${_IS_BUILD_CANARY}" >> "${GITHUB_ENV}"
echo "_IS_GITHUB_RELEASE=${_IS_GITHUB_RELEASE}" >> "${GITHUB_ENV}"

# Update the version in Cargo.toml
sed -i "0,/^version = .*/s//version = \"${_BUILD_VERSION}\"/" Cargo.toml

# Start the build
cargo build --bin launcher --release
cargo build --release

# Start the packaging
mkdir -p .dist/pkg
cp -R target/release/launcher .dist/pkg/
cp -R target/release/${_RELEASE_NAME} .dist/pkg/
7z a "./.dist/${_RELEASE_NAME}-${_RELEASE_VERSION}.zip" "./.dist/pkg/*"

rm -rf .dist/pkg
