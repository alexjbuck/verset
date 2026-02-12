class Verset < Formula
  desc "Universal changeset management for all languages"
  homepage "https://github.com/alexjbuck/verset"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/alexjbuck/verset/releases/download/v0.1.0/verset-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_AARCH64_DARWIN"
    else
      url "https://github.com/alexjbuck/verset/releases/download/v0.1.0/verset-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64_DARWIN"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/alexjbuck/verset/releases/download/v0.1.0/verset-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_AARCH64_LINUX"
    else
      url "https://github.com/alexjbuck/verset/releases/download/v0.1.0/verset-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64_LINUX"
    end
  end

  def install
    bin.install "verset"
  end

  test do
    system "#{bin}/verset", "--version"
  end
end 