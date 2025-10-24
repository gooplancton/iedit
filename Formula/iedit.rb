class Iedit < Formula
  desc "Minimal text editor that opens alongside the scrollback buffer"
  homepage "https://github.com/gooplancton/iedit"
  version "0.1.0"
  
  MACOS_X86_URL = "https://github.com/gooplancton/iedit/releases/download/v0.1.0/iedit-macos-x86_64"
  MACOS_X86_SHA = "***"

  MACOS_ARM_URL = "https://github.com/gooplancton/iedit/releases/download/v0.1.0/iedit-macos-arm64"
  MACOS_ARM_SHA = "***"

  LINUX_URL = "https://github.com/gooplancton/iedit/releases/download/v0.1.0/iedit-linux"
  LINUX_SHA = "***"
  
  on_macos do
    if Hardware::CPU.intel?
      url MACOS_X86_URL
      sha256 MACOS_X86_SHA
    else
      url MACOS_ARM_URL
      sha256 MACOS_ARM_SHA
    end
  end
  
  on_linux do
    url LINUX_URL
    sha256 LINUX_SHA
  end
  
  def install
    if OS.mac?
      if Hardware::CPU.intel?
        bin.install "iedit-macos-x86_64" => "iedit"
      else
        bin.install "iedit-macos-arm64" => "iedit"
      end
    else
      bin.install "iedit-linux" => "iedit"
    end
  end
  
  test do
    output = shell_output("#{bin}/iedit --version 2>&1", 0)
    assert_match version.to_s, output
  end
end