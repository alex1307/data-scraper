PROJECT_NAME=scraper
TARGET_DIR=target/release
RELEASE_DIR=release
CONFIG_DIR=config
DEST_CONFIG_DIR=/Users/$(USER)/Software/$(RELEASE_DIR)/Rust/$(PROJECT_NAME)/config
DEST_RELEASE_DIR=/Users/$(USER)/Software/$(RELEASE_DIR)/Rust/$(PROJECT_NAME)
.PHONY: all build release copy_config clean test

all: build copy-config clean

build:
	cargo build --release --target-dir $(DEST_RELEASE_DIR)
	cp $(DEST_RELEASE_DIR)/release/listing $(DEST_RELEASE_DIR)/
	cp $(DEST_RELEASE_DIR)/release/details $(DEST_RELEASE_DIR)/
	cp $(DEST_RELEASE_DIR)/release/metadata $(DEST_RELEASE_DIR)/
	strip $(DEST_RELEASE_DIR)/listing
	strip $(DEST_RELEASE_DIR)/details
	strip $(DEST_RELEASE_DIR)/metadata

copy-config:
	mkdir -p $(DEST_CONFIG_DIR)
	cp -r $(CONFIG_DIR)/* $(DEST_CONFIG_DIR)/

test:
	cargo test

clean:
	rm -rf $(DEST_RELEASE_DIR)/release
