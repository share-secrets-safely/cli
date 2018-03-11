class Sheesy < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./pkg/brew/sheesy.rb.in'
  version '3.1.0'
  desc "share secrets within teams to avoid plain-text secrets from day one"
  homepage "https://github.com/Byron/share-secrets-safely"
  depends_on "gnupg"

  if OS.mac?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Darwin-x86_64.tar.gz"
      sha256 "43f43b16c672de95a1991cb29b3083247f540446ef8f828f663a8ebe3fadfaab"
  elsif OS.linux?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-linux-musl-x86_64.tar.gz"
      sha256 "ed0c81da14f323b4a8397d9813a9bb0a48c42e079ff4c4f7c8ef90e7392ec86c"
  end

  def install
    bin.install "sy"
  end
end
