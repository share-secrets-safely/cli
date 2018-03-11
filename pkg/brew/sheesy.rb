class Sheesy < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./pkg/brew/sheesy.rb.in'
  version '3.1.0'
  desc "share secrets within teams to avoid plain-text secrets from day one"
  homepage "https://github.com/Byron/share-secrets-safely"
  depends_on "gnupg"

  if OS.mac?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Darwin-x86_64.tar.gz"
      sha256 "f3fd98555908be0c629fac009872a9ba935984bbd66a62f929a5022486142796"
  elsif OS.linux?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-linux-musl-x86_64.tar.gz"
      sha256 "f04c8153cd157a1ad8437afc1209238cbf5d9341ea9b7a9d4dc476759d479b5d"
  end

  def install
    bin.install "sy"
  end
end
