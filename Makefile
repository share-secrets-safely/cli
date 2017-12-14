EXE=target/debug/s3

help:
	$(info Available Targets)
	$(info ---------------------------------------------------------------------------------------------------------------)
	$(info journey-tests    | Run journey tests using a pre-built linux binary)
	$(info ---------------------------------------------------------------------------------------------------------------)

$(EXE): 
	cargo build --all-features

journey-tests: $(EXE)
	tests/journey-test.sh $(EXE)
