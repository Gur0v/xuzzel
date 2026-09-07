# xuzzel - X11 launcher, derived from dmenu 5.4
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man
CC ?= cc
PKG_CONFIG ?= pkg-config
PKGS = x11 xft fontconfig xinerama
CPPFLAGS += -D_XOPEN_SOURCE=700 -DXINERAMA $(shell $(PKG_CONFIG) --cflags $(PKGS))
CFLAGS ?= -O2
CFLAGS += -std=c99 -Wall -Wextra -Wpedantic
LDLIBS += $(shell $(PKG_CONFIG) --libs $(PKGS))
OBJ = xuzzel.o drw.o util.o

all: xuzzel

xuzzel: $(OBJ)
	$(CC) $(LDFLAGS) -o $@ $(OBJ) $(LDLIBS)

xuzzel.o: xuzzel.c drw.h util.h
drw.o: drw.c drw.h util.h
util.o: util.c util.h

check: xuzzel
	./tests/parity.sh

sanitize:
	$(MAKE) clean
	@if printf 'int main(void){return 0;}\n' | $(CC) -x c - -o .sanitize-check -fsanitize=address,undefined >/dev/null 2>&1; then \
		rm -f .sanitize-check; \
		$(MAKE) CFLAGS='-O1 -g -std=c99 -Wall -Wextra -Wpedantic -fsanitize=address,undefined -fno-omit-frame-pointer' LDFLAGS='-fsanitize=address,undefined' && ./tests/parity.sh; \
	else \
		rm -f .sanitize-check; \
		echo 'SKIP: compiler sanitizer runtimes unavailable'; \
	fi

install: xuzzel
	install -Dm755 xuzzel $(DESTDIR)$(BINDIR)/xuzzel
	install -Dm644 xuzzel.1 $(DESTDIR)$(MANDIR)/man1/xuzzel.1
	install -Dm644 xuzzel.ini.5 $(DESTDIR)$(MANDIR)/man5/xuzzel.ini.5
	install -Dm644 xuzzel.ini $(DESTDIR)$(PREFIX)/share/doc/xuzzel/xuzzel.ini
	install -Dm644 contrib/xuzzel.desktop $(DESTDIR)$(PREFIX)/share/applications/xuzzel.desktop

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/xuzzel $(DESTDIR)$(MANDIR)/man1/xuzzel.1 \
		$(DESTDIR)$(MANDIR)/man5/xuzzel.ini.5 $(DESTDIR)$(PREFIX)/share/doc/xuzzel/xuzzel.ini \
		$(DESTDIR)$(PREFIX)/share/applications/xuzzel.desktop

clean:
	rm -f xuzzel $(OBJ)

.PHONY: all check sanitize install uninstall clean
