BINARY := target/release/educational-game-rs

.PHONY: release
release: $(BINARY) fix-dylibs

$(BINARY): $(shell find src -name '*.rs') Cargo.toml Cargo.lock
	cargo build --release

.PHONY: fix-dylibs
fix-dylibs: $(BINARY)
	@# Rewrite any Nix store dylib paths to their macOS system equivalents.
	@otool -L $(BINARY) \
		| grep '/nix/store/' \
		| awk '{print $$1}' \
		| while read nix_path; do \
			lib_name=$$(basename "$$nix_path"); \
			system_path="/usr/lib/$$lib_name"; \
			echo "Patching $$nix_path -> $$system_path"; \
			install_name_tool -change "$$nix_path" "$$system_path" $(BINARY); \
		done
