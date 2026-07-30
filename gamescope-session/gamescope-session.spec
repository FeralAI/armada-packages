%define debug_package %nil

# Adapted from Terra's gamescope-session package:
# https://github.com/terrapkg/packages/blob/a2d2f5b2b72ba72d83dd6d98b8a3fc40e1be2486/anda/games/gamescope-session/gamescope-session.spec
%global commit 0000000000000000000000000000000000000000
%global source_date_epoch_from_changelog 0

Name:           gamescope-session
# Overwritten from BASE.env by build.sh.
Version:        0
Release:        4%{?dist}.armada
Summary:        Gamescope-based user session

License:        MIT
URL:            https://github.com/OpenGamingCollective/gamescope-session
Source0:        %{url}/archive/%{commit}/%{name}-%{commit}.tar.gz
Patch1:         0001-armada-disable-session-xtrace.patch
Patch2:         0002-armada-use-no-argument-rotation-shader.patch
Patch3:         0003-armada-extend-startup-socket-timeout.patch
Patch4:         0004-armada-add-independent-itm-target.patch

BuildArch:      noarch
BuildRequires:  systemd-rpm-macros

Requires:       gamescope
Recommends:     switcheroo-control

%description
Common launcher and systemd units for gamescope-based user sessions. The Armada
package carries handheld-specific launcher integration needed by its gamescope
build, slow removable-storage boot path, and HDR-capable internal panels.

%prep
%autosetup -n %{name}-%{commit} -p1

%build

%install
install -Dpm0755 -t "%{buildroot}%{_bindir}/" ".%{_bindir}/export-gpu"
install -Dpm0755 -t "%{buildroot}%{_bindir}/" ".%{_bindir}/gamescope-session-plus"
install -Dpm0644 -t "%{buildroot}%{_userunitdir}/" ".%{_userunitdir}/gamescope-session-plus@.service"
install -Dpm0644 -t "%{buildroot}%{_userunitdir}/" ".%{_userunitdir}/gamescope-session.target"
install -Dpm0644 -t "%{buildroot}%{_datadir}/gamescope-session-plus/" ".%{_datadir}/gamescope-session-plus/device-quirks"
install -Dpm0755 -t "%{buildroot}%{_datadir}/gamescope-session-plus/" ".%{_datadir}/gamescope-session-plus/gamescope-session-plus"
install -Dpm0644 -t "%{buildroot}%{_datadir}/gamescope/scripts/50-custom/" ".%{_datadir}/gamescope/scripts/50-custom/50-disable-explicit-sync.lua"

%check
launcher="usr/share/gamescope-session-plus/gamescope-session-plus"
bash -n "${launcher}"
! grep -qx 'set -x' "${launcher}"
grep -Fqx '	if [ "$USE_ROTATION_SHADER" = "1" ] && gamescope_has_option "--use-rotation-shader"; then' "${launcher}"
grep -Fqx '		USE_ROTATION_SHADER_OPTION="--use-rotation-shader"' "${launcher}"
grep -Fqx '	if [ -n "${GAMESCOPE_HDR_ITM_TARGET_NITS:-}" ] && gamescope_has_option "--hdr-itm-target-nits"; then' "${launcher}"
grep -Fqx '		HDR_OPTIONS="$HDR_OPTIONS --hdr-itm-target-nits $GAMESCOPE_HDR_ITM_TARGET_NITS"' "${launcher}"
grep -Fqx 'if read -r -t 15 response_x_display response_wl_display <>"$socket"; then' "${launcher}"

%files
%doc README.md
%license LICENSE
%{_bindir}/export-gpu
%{_bindir}/gamescope-session-plus
%{_datadir}/gamescope-session-plus/device-quirks
%{_datadir}/gamescope-session-plus/gamescope-session-plus
%{_datadir}/gamescope/scripts/50-custom/50-disable-explicit-sync.lua
%{_userunitdir}/gamescope-session-plus@.service
%{_userunitdir}/gamescope-session.target

%changelog
