all: build

build:
	cargo build --release

install:
	install -D target/release/raid-notifier -m755 "$(DESTDIR)/usr/bin/raid-notifier"
	install -D raid-notifier.service -m644 "$(DESTDIR)/usr/lib/systemd/system"
