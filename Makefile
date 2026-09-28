# Convenience targets. Every target runs the pinned generator through scripts/course.
.PHONY: build validate check-offline package test browser-test clean

build:          ## validate the course source and generate dist/
	scripts/course build

validate:       ## validate the course source (add ONO=../ono-sendai to check snippets upstream)
	scripts/course validate $(if $(ONO),--ono $(ONO))

check-offline:  ## check dist/ for external resources and broken links
	scripts/course check-offline

package: build  ## build release archives into release/
	scripts/course package

test:           ## generator unit, integration and golden tests
	cargo test --locked -p ono-course

browser-test: build ## Playwright tests against dist/ (network blocked)
	npx playwright test

clean:
	rm -rf dist release test-results playwright-report
