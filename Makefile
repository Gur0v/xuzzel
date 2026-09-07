# xuzzel - X11 launcher, derived from dmenu 5.4
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man
CC ?= cc
PKG_CONFIG ?= pkg-config
PKGS = x11 xft fontconfig xinerama cairo-xlib libpng
CPPFLAGS += -D_XOPEN_SOURCE=700 -DXINERAMA $(shell $(PKG_CONFIG) --cflags $(PKGS))
CFLAGS ?= -O3 -flto -fno-plt -ffunction-sections -fdata-sections -DNDEBUG
CFLAGS += -std=c99 -Wall -Wextra -Wpedantic -Werror \
	-Wformat=2 -Wformat-security -Wstrict-prototypes -Wmissing-prototypes \
	-Wshadow -Wundef -Wpointer-arith -Wcast-align -Wwrite-strings -Wvla \
	-Wdate-time -Wnull-dereference -Wduplicated-cond -Wduplicated-branches \
	-Wlogical-op
LDFLAGS += -flto -Wl,--as-needed -Wl,--gc-sections
LDLIBS += $(shell $(PKG_CONFIG) --libs $(PKGS)) -lm
OBJ = xuzzel.o icon.o nanosvg_impl.o drw.o util.o

all: xuzzel

xuzzel: $(OBJ)
	$(CC) $(LDFLAGS) -o $@ $(OBJ) $(LDLIBS)

xuzzel.o: xuzzel.c drw.h icon.h util.h
icon.o: icon.c icon.h util.h nanosvg_vendor.h third_party/nanosvg/nanosvg.h third_party/nanosvg/nanosvgrast.h
nanosvg_impl.o: nanosvg_impl.c third_party/nanosvg/nanosvg.h third_party/nanosvg/nanosvgrast.h
	$(CC) $(CPPFLAGS) -O3 -flto -ffunction-sections -fdata-sections -std=c99 -w -c -o $@ nanosvg_impl.c
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
