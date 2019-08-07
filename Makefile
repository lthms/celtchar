TEMPLATES := $(wildcard templates/*)
FONTS     := $(wildcard fonts/*)

release:
	@cargo build --release

install: release
	@sudo -k # always ask user password
	@sudo install -dv /usr/local/share/celtchar/templates \
	                  /usr/local/share/celtchar/fonts
	@sudo install -v ${TEMPLATES} /usr/local/share/celtchar/templates
	@sudo install -v ${FONTS} /usr/local/share/celtchar/fonts
	@sudo install -v target/release/celtchar /usr/local/bin/celtchar

uninstall:
	@sudo rm -rfv /usr/local/share/celtchar
	@sudo rm -fv /usr/local/bin/celtchar


.PHONY: install uninstall release
