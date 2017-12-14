EXE=target/debug/s3
MAKESHELL=$(shell /usr/bin/env bash)

help:
	$(info Available Targets)
	$(info ---------------------------------------------------------------------------------------------------------------)
	$(info journey-tests    | Run journey tests using a pre-built linux binary)
	$(info ---------------------------------------------------------------------------------------------------------------)

$(EXE): $(shell find src -name "*.rs")
	cargo build --all-features

journey-tests: $(EXE)
	tests/journey-test.sh $(EXE)
