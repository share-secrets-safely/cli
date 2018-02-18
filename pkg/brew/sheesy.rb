class Sheesy < Formula
  version '2.0.0'
  desc "share secrets within teams to avoid plain-text secrets from day one"
  homepage "https://github.com/Byron/share-secrets-safely"
  depends_on "gnupg"

  if OS.mac?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-Darwin-x86_64.tar.gz"
      sha256 "2d109dea5ce2eaba8b34b8b87341e7c077dd6c569b13459fc4fceeae2bc6fb90"
  elsif OS.linux?
      url "https://github.com/Byron/share-secrets-safely/releases/download/#{version}/sy-cli-linux-musl-x86_64.tar.gz"
      sha256 "29009fbc7dbe75580d013abfde74e079a46c371a7dd140f418562104f406b473"
  end

  def install
    bin.install "sy"
  end
end
