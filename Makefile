SHELL=/bin/bash
EXE=target/debug/sy
DOCS_IMAGE=sheesy_docs:latest
RELEASE_MUSL_EXE=target/x86_64-unknown-linux-musl/release/sy
RELEASE_EXE=target/release/sy
MUSL_EXE=target/x86_64-unknown-linux-musl/debug/sy
SYV_MUSL_EXE=target/x86_64-unknown-linux-musl/debug/syv
MUSL_IMAGE=clux/muslrust:stable
MY_MUSL_IMAGE=sheesy_musl:stable
MY_LINUX_RUN_IMAGE=alpine_with_gpg2:stable
CARGO_CACHE_ARGS=-v $$PWD/.docker-cargo-cache:/usr/local/cargo/registry
DOCKER_ARGS=docker run -v $$PWD/.docker-cargo-cache:/root/.cargo/registry -v "$$PWD:/volume" --rm
MUSL_DOCKER_ARGS=$(DOCKER_ARGS) $(MY_MUSL_IMAGE)
HOST_DEPLOYABLE=$(shell echo sy-cli-$$(uname -s)-$$(uname -m).tar.gz)
LINUX_DEPLOYABLE=sy-cli-Linux-x86_64.tar.gz
DOCKER_DOCS_ARGS_NO_CMD=$(DOCKER_ARGS) -e EXE_PATH=$(MUSL_EXE) -w /volume $(DOCS_IMAGE)
DOCKER_DOCS_ARGS=$(DOCKER_DOCS_ARGS_NO_CMD) termbook build ./doc
CAST=getting-started.cast
MUSL_RELEASE_ARGS=--release --target=x86_64-unknown-linux-musl
ROOT=$(dir $(realpath $(firstword $(MAKEFILE_LIST))))

help:
	$(info Available Targets)
	$(info - Testing -----------------------------------------------------------------------------------------------------)
	$(info lint-scripts            | Run journey tests using a pre-built linux binary)
	$(info journey-tests           | Run all journey tests using a pre-built binary)
	$(info stateful-journey-tests  | Run only stateful journeys in docker)
	$(info stateless-journey-tests | Run only stateless journey)
	$(info - Documentation -----------------------------------------------------------------------------------------------)
	$(info docs                    | build documentation with termbook)
	$(info watch-docs              | continuously rebuild docs when changes happen. Needs `watchexec`)
	$(info asciinema-no-upload     | record the getting-started chapter to a file)
	$(info asciinema-upload        | upload the recorded cast after review)
	$(info - Deployment  -------------------------------------------------------------------------------------------------)
	$(info tag-release            | Create a new release commit using the version in VERSION file)
	$(info deployable-linux       | Archive usable for any more recent linux system)
	$(info deployable-host        | Archive usable on your host)
	$(info deployment             | All archives, for host and linux system)
	$(info update-homebrew        | Update homebrew after both deployables have been generated)
	$(info update-getting-started | Update the getting-started repository to the latest version)
	$(info - Docker ------------------------------------------------------------------------------------------------------)
	$(info build-linux-musl       | Build the binary via a docker based musl container)
	$(info build-musl-image       | Build our musl build image)
	$(info interactive-linux-musl | The interactive version of the above (MUSL))
	$(info ---------------------------------------------------------------------------------------------------------------)

always:

build-docs-image: build-linux-run-image
	docker build -t $(DOCS_IMAGE) - < etc/docker/Dockerfile.alpine-docs

docs: build-docs-image $(MUSL_EXE)
	$(DOCKER_DOCS_ARGS)

docs-no-deps:
	$(DOCKER_DOCS_ARGS) '3.*'

watch-docs: build-docs-image $(MUSL_EXE)
	watchexec -w doc $(MAKE) docs-no-deps

$(CAST):
	rm -f $@
	asciinema rec \
		--title 'Getting Started with Sheesy (https://share-secrets-safely.github.io/cli)' \
		--idle-time-limit 1 \
		-c '$(DOCKER_DOCS_ARGS_NO_CMD) sh -c "termbook play doc 1.1. 2>/dev/null"' \
		$@

asciinema-no-upload: build-docs-image $(MUSL_EXE) $(CAST)

asciinema-upload: build-docs-image $(MUSL_EXE) $(CAST)
	asciinema upload $(CAST)

$(EXE): always
	cargo build --bin=sy --all-features

all-release-host-binaries: always
	cargo build --release --bin=sy --all-features
	cargo build --release --bin=syv --features=vault
	cargo build --release --bin=syp --features=process
	cargo build --release --bin=sye --features=extract
	cargo build --release --bin=sys --features=substitute
	
$(MUSL_EXE): build-linux-musl
	
$(SYV_MUSL_EXE): build-linux-musl-syv

$(RELEASE_MUSL_EXE): release-linux-musl

$(LINUX_DEPLOYABLE): build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo/registry -v "$$PWD:/volume" \
					--rm $(MY_MUSL_IMAGE) \
					/bin/bash -c \
					'cargo build --bin=sy --all-features $(MUSL_RELEASE_ARGS) && \
					cargo build --bin=syv --features=vault $(MUSL_RELEASE_ARGS) && \
					cargo build --bin=syp --features=process $(MUSL_RELEASE_ARGS) && \
					cargo build --bin=sye --features=extract $(MUSL_RELEASE_ARGS) && \
					cargo build --bin=sys --features=substitute $(MUSL_RELEASE_ARGS) && \
					(cd "$(dir $(RELEASE_MUSL_EXE))" && \
					 strip --strip-all `../../../bin/find-executables.sh .`)'
	bin/package.sh "$(dir $(RELEASE_MUSL_EXE))" "$@"
					
deployable-linux: $(LINUX_DEPLOYABLE)

$(HOST_DEPLOYABLE): all-release-host-binaries
	(cd "$(dir $(RELEASE_EXE))" && strip `$(ROOT)/bin/find-executables.sh .`)
	bin/package.sh "$(dir $(RELEASE_EXE))" "$@"

deployable-host: $(HOST_DEPLOYABLE)

update-homebrew:
	git clone https://github.com/share-secrets-safely/homebrew-cli
	set -ex; ./bin/update-homebrew-formula.sh $$(cat ../VERSION) ./etc/brew/sheesy.rb.in ./homebrew-cli/sheesy.rb
	cd homebrew-cli && git commit -am "update formula" && git push origin master
	rm -Rf homebrew-cli/

update-getting-started:
	git clone https://github.com/share-secrets-safely/getting-started
	cd getting-started && echo "VERSION=$$(cat ../VERSION)" > .version && git commit -am "update version" && git push origin master
	rm -Rf getting-started

tag-release: bin/tag-release.sh release.md VERSION
	bin/tag-release.sh $$(cat VERSION) release.md

stateful-journey-tests: $(SYV_MUSL_EXE) build-linux-run-image
	tests/stateful-journey-test.sh $< $(MY_LINUX_RUN_IMAGE)

stateless-journey-tests: $(EXE)
	tests/stateless-journey-test.sh $<

journey-tests: stateful-journey-tests stateless-journey-tests

deployment: lint-scripts journey-tests
	$(MAKE) deployable-host
	$(MAKE) deployable-linux

build-linux-run-image:
	docker build -t $(MY_LINUX_RUN_IMAGE) - < etc/docker/Dockerfile.alpine-gpg2

build-musl-image:
	docker build -t $(MY_MUSL_IMAGE) - < etc/docker/Dockerfile.musl-build

build-linux-musl: build-musl-image
	$(MUSL_DOCKER_ARGS) cargo build --bin=sy --all-features --target=x86_64-unknown-linux-musl

build-linux-musl-syv: build-musl-image
	$(MUSL_DOCKER_ARGS) cargo build --bin=syv --features=vault --target=x86_64-unknown-linux-musl
	
release-linux-musl: build-musl-image
	docker run -v $$PWD/.docker-cargo-cache:/root/.cargo -v "$$PWD:/volume" --rm $(MY_MUSL_IMAGE) cargo build --bin=sy --all-features --target=x86_64-unknown-linux-musl --release

interactive-linux-musl: build-musl-image
	$(DOCKER_ARGS) -it $(MY_MUSL_IMAGE)

lint-scripts:
	find . -not \( -name 'included-*' -or -path '*target/*' -or -path "*cargo*" \) -name '*.sh' -type f | while read -r sf; do shellcheck -x "$$sf"; done
