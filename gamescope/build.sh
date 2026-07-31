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
  --env ARMADA_MARCH="${ARMADA_MARCH}" \
  "${BUILDER_IMAGE}" \
  bash -euxo pipefail -c '
    source /etc/os-release
    dnf install -y --nogpgcheck --repofrompath "terra,https://repos.fyralabs.com/terra${VERSION_ID}" terra-release
    dnf -y install --skip-unavailable \
        anda
    cat >/etc/rpm/macros.armada <<EOF
%_buildhost armada-builder
%packager Armada
%vendor Armada
EOF
    cd /tmp
    git clone https://github.com/terrapkg/packages.git
    cd packages
    git checkout ${COMMIT}

    git apply /work/patches/0000-add-patches-to-spec.patch

    PKG=anda/games/terra-gamescope

    sed -i "/^Release:/s/%?dist/%{?dist}.armada/" ${PKG}/terra-gamescope.spec
    sed -i "/^%build$/i %global build_cflags %{build_cflags} ${ARMADA_MARCH}" ${PKG}/terra-gamescope.spec
    sed -i "/^%build$/i %global build_cxxflags %{build_cxxflags} ${ARMADA_MARCH}" ${PKG}/terra-gamescope.spec
    cp /work/patches/*.patch ${PKG}/

    dnf builddep -y ${PKG}/terra-gamescope.spec
    anda build --rpm-builder=rpmbuild ${PKG}/pkg
    cp /tmp/packages/anda-build/rpm/rpms/*.rpm /work/out/
'

echo "built: ${PACKAGE_DIR}/out"
