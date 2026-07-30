#!/usr/bin/bash

set -euxo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
PACKAGE_DIR="${PWD}"

source ./BASE.env
source ../toolchain.env

rm -rf out
mkdir -p out

podman run --rm \
  --volume "${PACKAGE_DIR}:/work:Z" \
  --workdir /work \
  --platform linux/aarch64 \
  --env COMMIT="${COMMIT}" \
  --env VERSION="${VERSION}" \
  "${BUILDER_IMAGE}" \
  bash -euxo pipefail -c '
    export HOME=/tmp
    dnf -y install rpm-build rpmdevtools spectool "dnf-command(builddep)"
    rpmdev-setuptree
    cat >/etc/rpm/macros.armada <<EOF
%_buildhost armada-builder
%packager Armada
%vendor Armada
EOF

    cp /work/gamescope-session.spec "$HOME/rpmbuild/SPECS/"
    sed -i "s/^%global commit .*/%global commit ${COMMIT}/" "$HOME/rpmbuild/SPECS/gamescope-session.spec"
    sed -i "s/^Version:.*/Version:        ${VERSION}/" "$HOME/rpmbuild/SPECS/gamescope-session.spec"
    cp /work/patches/*.patch "$HOME/rpmbuild/SOURCES/"

    spectool -g -R "$HOME/rpmbuild/SPECS/gamescope-session.spec"
    dnf -y builddep "$HOME/rpmbuild/SPECS/gamescope-session.spec"
    rpmbuild -bb "$HOME/rpmbuild/SPECS/gamescope-session.spec"

    cp "$HOME"/rpmbuild/RPMS/noarch/gamescope-session-*.rpm /work/out/
'

echo "built: ${PACKAGE_DIR}/out"
