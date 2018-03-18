class Sheesy < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./pkg/brew/sheesy.rb.in'
  version '3.2.0'
  desc "share secrets within teams to avoid plain-text secrets from day one"
  homepage "https://github.com/Byron/share-secrets-safely"
  depends_on "gnupg"

  if OS.mac?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Darwin-x86_64.tar.gz"
      sha256 "5009305f84dcfbeac3435eb94a59ff651c15580a0542b423504204f66c942da2"
  elsif OS.linux?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Linux-x86_64.tar.gz"
      sha256 "cde10359d91f6b3af9ed6c4c562c0d875bdbc1e17534955f1f653c6f6f91dd12"
  end

  def install
    bin.install "sy"
  end
end
