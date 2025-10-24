class Iedit < Formula
  desc "Minimal text editor that opens alongside the scrollback buffer"
  homepage "https://github.com/gooplancton/iedit"
  # Set the version to match your GitHub release tag (without the leading 'v').
  # e.g. if your release is 'v0.1.0' set version "0.1.0"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/gooplancton/iedit/releases/download/v#{version}/iedit-macos-x86_64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_x86_64_TAR_GZ"
    else
      url "https://github.com/gooplancton/iedit/releases/download/v#{version}/iedit-macos-arm64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_arm64_TAR_GZ"
    end

    def install
      # The release tarball is expected to contain a single executable named `iedit`.
      bin.install "iedit"
    end
  end

  on_linux do
    # For Linux/Homebrew on Linux we build from source by default.
    url "https://github.com/gooplancton/iedit/archive/refs/tags/v#{version}.tar.gz"
    sha256 "REPLACE_WITH_SOURCE_TARBALL_SHA256"

    depends_on "rust" => :build

    def install
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    # Basic smoke test: --version or help should exit successfully.
    # Adjust if `iedit` uses a different flag for version output.
    output = shell_output("#{bin}/iedit --version 2>&1", 0)
    assert_match version.to_s, output
  end
end
