#ifndef XUZZEL_ICON_H
#define XUZZEL_ICON_H

#include <cairo/cairo.h>

void icon_init(const char *theme);
cairo_surface_t *icon_load(const char *name, int size);
void icon_cleanup(void);
int icon_probe(const char *theme, const char *name, int size);

#endif
