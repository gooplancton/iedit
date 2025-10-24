class Iedit < Formula
  desc "Minimal text editor that opens alongside the scrollback buffer"
  homepage "https://github.com/gooplancton/iedit"
  version "0.1.2"
  
  MACOS_X86_URL = "https://github.com/gooplancton/iedit/releases/download/v0.1.2/iedit-macos-x86_64"
  MACOS_X86_SHA = "8a0fbee5d6ce16788c7e037eacb93e2a3d65767da7e619bc4d6ddd59df18a92c"

  MACOS_ARM_URL = "https://github.com/gooplancton/iedit/releases/download/v0.1.2/iedit-macos-arm64"
  MACOS_ARM_SHA = "2077eb7d88290a96d8404d156cdee834bc9822a312a7229da66f75f619e348d1"

  LINUX_URL = "https://github.com/gooplancton/iedit/releases/download/v0.1.2/iedit-linux"
  LINUX_SHA = "0703d50711063f8df9f3e99fe0e51fc1d13d43a9b08afb19ce98cfacce6e08e7"
  
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