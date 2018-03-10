class Sheesy < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./pkg/brew/sheesy.rb.in'
  version '3.0.0'
  desc "share secrets within teams to avoid plain-text secrets from day one"
  homepage "https://github.com/Byron/share-secrets-safely"
  depends_on "gnupg"

  if OS.mac?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Darwin-x86_64.tar.gz"
      sha256 "deff5ea32512a0d3dcf57d1cfaf024520affb686f41039005d2015fa74086fb1"
  elsif OS.linux?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-linux-musl-x86_64.tar.gz"
      sha256 "48ba16ec656db005da37962c15282ab76ccae94be63ee1d3f5c610b1fb3bbec9"
  end

  def install
    bin.install "sy"
  end
end
