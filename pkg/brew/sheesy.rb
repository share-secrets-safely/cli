class Sheesy < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./pkg/brew/sheesy.rb.in'
  version '3.3.0'
  desc "share secrets within teams to avoid plain-text secrets from day one"
  homepage "https://github.com/Byron/share-secrets-safely"
  depends_on "gnupg"

  if OS.mac?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Darwin-x86_64.tar.gz"
      sha256 "91496c1e2d839b056cf1706a3eb441a77ad50fd3170eb0c46f54325acd0fcad2"
  elsif OS.linux?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Linux-x86_64.tar.gz"
      sha256 "07131855276fc68e663bb4f1092b1c2fe1c51c1c2c574aa0d52172b404cac4e8"
  end

  def install
    bin.install "sy"
  end
end
