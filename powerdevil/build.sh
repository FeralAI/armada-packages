#!/usr/bin/bash

set -euxo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
PACKAGE_DIR="${PWD}"

source ./BASE.env
source ../toolchain.env

REST="${SRPM#powerdevil-}"
POWERDEVIL_VER="${REST%%-*}"
POWERDEVIL_REL="${REST#*-}"
POWERDEVIL_REL="${POWERDEVIL_REL%.fc*}"
DIST=".fc44.armada"

rm -rf out
mkdir -p out

podman run --rm \
  --volume "${PACKAGE_DIR}:/work:Z" \
  --workdir /work \
  --platform linux/aarch64 \
  --env SRPM="${SRPM}" \
  --env POWERDEVIL_VER="${POWERDEVIL_VER}" \
  --env POWERDEVIL_REL="${POWERDEVIL_REL}" \
  --env DIST="${DIST}" \
  "${BUILDER_IMAGE}" \
  bash -euxo pipefail -c '
    export HOME=/tmp
    dnf -y install rpm-build rpmdevtools koji "dnf-command(builddep)"
    rpmdev-setuptree
    cat >/etc/rpm/macros.armada <<EOF
%_buildhost armada-builder
%packager Armada
%vendor Armada
EOF

    cd /tmp
    koji download-build --arch=src "${SRPM}"
    rpm -i "${SRPM}.src.rpm"
    SPEC="$HOME/rpmbuild/SPECS/powerdevil.spec"

    sed -i "s/^Release:.*/Release: ${POWERDEVIL_REL}%{?dist}/" "$SPEC"

    cp /work/patches/*.patch "$HOME/rpmbuild/SOURCES/"
    LAST=$(grep -nE "^(Patch|Source)[0-9]*:" "$SPEC" | tail -1 | cut -d: -f1)
    [ -n "$LAST" ] || { echo "ERROR: no Source/Patch line to anchor on"; exit 1; }
    sed -i "${LAST}a Patch9001: 0001-armada-keep-internal-displays-visible.patch" "$SPEC"

    grep -qE "^[[:space:]]*%autosetup" "$SPEC" \
        || { echo "ERROR: powerdevil.spec does not auto-apply patches; adjust build.sh"; exit 1; }

    dnf -y builddep "$SPEC"
    rpmbuild -bb --define "dist ${DIST}" "$SPEC"

    cp "$HOME"/rpmbuild/RPMS/*/powerdevil-"${POWERDEVIL_VER}-${POWERDEVIL_REL}${DIST}".*.rpm /work/out/
'

echo "built: ${PACKAGE_DIR}/out"
