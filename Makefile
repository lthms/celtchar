EPUB_TPL   := $(wildcard templates/epub/*)
STATIC_TPL := $(wildcard templates/static/*)
FONTS      := $(wildcard fonts/*)

release:
	@cargo build --release

install: release
	@sudo -k # always ask user password
	@sudo install -dv /usr/local/share/celtchar/templates/epub \
	                  /usr/local/share/celtchar/templates/static \
	                  /usr/local/share/celtchar/fonts
	@sudo install -v ${EPUB_TPL} /usr/local/share/celtchar/templates/epub
	@sudo install -v ${STATIC_TPL} /usr/local/share/celtchar/templates/static
	@sudo install -v ${FONTS} /usr/local/share/celtchar/fonts
	@sudo install -v target/release/celtchar /usr/local/bin/celtchar

uninstall:
	@sudo rm -rfv /usr/local/share/celtchar
	@sudo rm -fv /usr/local/bin/celtchar


.PHONY: install uninstall release
