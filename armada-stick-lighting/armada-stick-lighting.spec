%global debug_package %{nil}
%global source_date_epoch_from_changelog 0

Name:           armada-stick-lighting
# Keep this in sync with Cargo.toml.
Version:        0.1.0
Release:        1%{?dist}.armada
Summary:        Stick lighting controller for Armada
License:        GPL-3.0-or-later
URL:            https://github.com/armada-os/armada-packages

Source0:        armada-stick-lighting.tar.gz

BuildRequires:  cargo
BuildRequires:  rust

%description
%{name} controls handheld stick LEDs.

%prep
%autosetup -n work

%build
cargo build --release --locked

%install
install -Dpm 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%files
%doc README.md
%{_bindir}/%{name}

%changelog
