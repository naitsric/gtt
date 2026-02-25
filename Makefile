.PHONY: release

release:
	@if [ -z "$(version)" ]; then \
		echo "Error: version is required."; \
		echo "Usage: make release version=v0.1.2"; \
		exit 1; \
	fi
	@echo "Creating and pushing tag $(version)..."
	git tag $(version)
	git push origin $(version)
	@echo "Done! GitHub Actions will now build and publish $(version)"

.PHONY: install
install:
	@echo "Fetching latest release version..."
	@LATEST_URL=$$(curl -sI https://github.com/naitsric/gtt/releases/latest | grep -i "location:" | awk '{print $$2}' | tr -d '\r'); \
	VERSION=$$(basename $$LATEST_URL); \
	if [ -z "$$VERSION" ]; then \
		echo "Error: Could not determine latest version."; \
		exit 1; \
	fi; \
	echo "Downloading version $$VERSION..."; \
	curl -L "https://github.com/naitsric/gtt/releases/download/$$VERSION/gtt-$$VERSION-x86_64-unknown-linux-gnu.tar.gz" | tar xz; \
	echo "Installing to /usr/local/bin/ (requires sudo)..."; \
	sudo mv gtt /usr/local/bin/; \
	echo "Done! gtt $$VERSION installed successfully."
