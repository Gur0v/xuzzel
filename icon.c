#define _DEFAULT_SOURCE
#define _POSIX_C_SOURCE 200809L
#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <limits.h>
#include <png.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#include "nanosvg_vendor.h"

#include "icon.h"
#include "util.h"

struct directory {
    char *name;
    int size, min, max, threshold;
    enum { FIXED, SCALABLE, THRESHOLD } type;
    struct directory *next;
};
struct theme {
    char *name;
    char **inherits;
    size_t inherit_count;
    struct directory *dirs;
    struct theme *next;
};
struct cached_icon {
    char *name;
    int size;
    cairo_surface_t *surface;
    struct cached_icon *next;
};

static char *theme_name;
static struct theme *themes;
static struct cached_icon *cache;

static char *xstrdup(const char *s){char *p=strdup(s);if(!p)die("strdup:");return p;}
static void *xrealloc(void *p,size_t n){p=realloc(p,n);if(!p)die("realloc:");return p;}

static char *trim(char *s)
{
    while (isspace((unsigned char)*s)) s++;
    char *e=s+strlen(s); while(e>s&&isspace((unsigned char)e[-1]))*--e='\0';
    return s;
}

static void add_string(char ***array, size_t *count, const char *value)
{
    char *copy=xstrdup(value), *save=NULL;
    for(char *s=strtok_r(copy,",",&save);s;s=strtok_r(NULL,",",&save)){
        s=trim(s); if(!*s)continue;
        *array=xrealloc(*array,(*count+1)*sizeof **array);
        (*array)[(*count)++]=xstrdup(s);
    }
    free(copy);
}

static char **data_dirs(size_t *count)
{
    char **dirs=NULL; *count=0;
    const char *home=getenv("HOME"), *xdg=getenv("XDG_DATA_HOME");
    if(xdg&&*xdg){dirs=xrealloc(dirs,sizeof *dirs);dirs[(*count)++]=xstrdup(xdg);}
    else if(home&&*home){size_t n=strlen(home)+15;dirs=xrealloc(dirs,sizeof *dirs);dirs[(*count)]=ecalloc(n,1);snprintf(dirs[(*count)++],n,"%s/.local/share",home);}
    const char *env=getenv("XDG_DATA_DIRS");if(!env||!*env)env="/usr/local/share:/usr/share";
    char *copy=xstrdup(env),*save=NULL;
    for(char *s=strtok_r(copy,":",&save);s;s=strtok_r(NULL,":",&save))if(*s){dirs=xrealloc(dirs,(*count+1)*sizeof *dirs);dirs[(*count)++]=xstrdup(s);}
    free(copy);return dirs;
}

static struct theme *find_theme(const char *name)
{
    for(struct theme *t=themes;t;t=t->next)if(!strcmp(t->name,name))return t;
    return NULL;
}

static void parse_theme_file(struct theme *t, const char *path)
{
    FILE *f=fopen(path,"r");if(!f)return;
    char *line=NULL,*dirs=NULL;size_t cap=0;struct directory *cur=NULL;
    while(getline(&line,&cap,f)>=0){
        char *s=trim(line);if(!*s||*s=='#'||*s==';')continue;
        if(*s=='['){char *e=strchr(s,']');if(!e)continue;*e='\0';cur=NULL;
            if(strcmp(s+1,"Icon Theme")){cur=ecalloc(1,sizeof *cur);cur->name=xstrdup(s+1);cur->type=THRESHOLD;cur->threshold=2;cur->next=t->dirs;t->dirs=cur;}continue;}
        char *eq=strchr(s,'=');if(!eq)continue;*eq++='\0';s=trim(s);eq=trim(eq);
        if(cur){if(!strcmp(s,"Size"))cur->size=atoi(eq);else if(!strcmp(s,"MinSize"))cur->min=atoi(eq);else if(!strcmp(s,"MaxSize"))cur->max=atoi(eq);else if(!strcmp(s,"Threshold"))cur->threshold=atoi(eq);else if(!strcmp(s,"Type")){if(!strcasecmp(eq,"Fixed"))cur->type=FIXED;else if(!strcasecmp(eq,"Scalable"))cur->type=SCALABLE;else cur->type=THRESHOLD;}}
        else if(!strcmp(s,"Directories")){free(dirs);dirs=xstrdup(eq);}else if(!strcmp(s,"Inherits"))add_string(&t->inherits,&t->inherit_count,eq);
    }
    free(line);fclose(f);
    if(dirs){char *save=NULL;for(char *d=strtok_r(dirs,",",&save);d;d=strtok_r(NULL,",",&save)){d=trim(d);for(struct directory *p=t->dirs;p;p=p->next)if(!strcmp(p->name,d)){p->size=p->size?p->size:48;p->min=p->min?p->min:p->size;p->max=p->max?p->max:p->size;break;}}free(dirs);}
}

static struct theme *load_theme(const char *name)
{
    struct theme *old=find_theme(name);if(old)return old;
    struct theme *t=ecalloc(1,sizeof *t);t->name=xstrdup(name);t->next=themes;themes=t;
    size_t n=0;char **dirs=data_dirs(&n);char path[PATH_MAX];
    for(size_t i=0;i<n;i++){snprintf(path,sizeof path,"%s/icons/%s/index.theme",dirs[i],name);parse_theme_file(t,path);free(dirs[i]);}free(dirs);
    return t;
}

static int dir_distance(const struct directory *d,int size)
{
    int lo=d->size,hi=d->size;
    if(d->type==SCALABLE){lo=d->min;hi=d->max;}else if(d->type==THRESHOLD){lo=d->size-d->threshold;hi=d->size+d->threshold;}
    return size<lo?lo-size:size>hi?size-hi:0;
}

static bool regular_readable(const char *path)
{
    struct stat st;return !stat(path,&st)&&S_ISREG(st.st_mode)&&!access(path,R_OK);
}

static char *try_icon(const char *base,const char *sub,const char *name)
{
    static const char *exts[]={"",".png",".svg",NULL};char path[PATH_MAX];
    size_t len=strlen(name);bool has_ext=len>4&&(!strcasecmp(name+len-4,".png")||!strcasecmp(name+len-4,".svg"));
    for(int i=0;exts[i];i++){if(has_ext&&i)break;snprintf(path,sizeof path,"%s/%s%s%s%s",base,sub&&*sub?sub:"",sub&&*sub?"/":"",name,exts[i]);if(regular_readable(path))return xstrdup(path);}
    return NULL;
}

static char *lookup_theme(const char *name,int size,const char *theme,int depth)
{
    if(depth>16)
        return NULL;
    struct theme *t=load_theme(theme);
    size_t n=0;
    char **bases=data_dirs(&n);
    char *best=NULL;
    int bestdist=INT_MAX;
    for(struct directory *d=t->dirs;d;d=d->next){
        int dist=dir_distance(d,size);
        for(size_t i=0;i<n;i++){
            char root[PATH_MAX];
            snprintf(root,sizeof root,"%s/icons/%s",bases[i],theme);
            char *p=try_icon(root,d->name,name);
            if(p&&dist<bestdist){
                free(best);
                best=p;
                bestdist=dist;
            }else{
                free(p);
            }
        }
    }
    for(size_t i=0;i<n;i++)
        free(bases[i]);
    free(bases);
    if(best)
        return best;
    for(size_t i=0;i<t->inherit_count;i++){
        best=lookup_theme(name,size,t->inherits[i],depth+1);
        if(best)
            return best;
    }
    return NULL;
}

static char *resolve_icon(const char *name,int size)
{
    if(!name||!*name)
        return NULL;
    if(name[0]=='/')
        return regular_readable(name)?xstrdup(name):NULL;
    char *p=lookup_theme(name,size,theme_name,0);if(!p&&strcmp(theme_name,"hicolor"))p=lookup_theme(name,size,"hicolor",0);if(p)return p;
    size_t n=0;char **dirs=data_dirs(&n);for(size_t i=0;i<n&&!p;i++){char base[PATH_MAX];snprintf(base,sizeof base,"%s/pixmaps",dirs[i]);p=try_icon(base,NULL,name);}
    for(size_t i=0;i<n;i++)
        free(dirs[i]);
    free(dirs);
    return p;
}

static cairo_surface_t *load_png(const char *path,int target)
{
    FILE *f=fopen(path,"rb");if(!f)return NULL;png_structp png=png_create_read_struct(PNG_LIBPNG_VER_STRING,NULL,NULL,NULL);png_infop info=png?png_create_info_struct(png):NULL;if(!png||!info){if(png)png_destroy_read_struct(&png,NULL,NULL);fclose(f);return NULL;}if(setjmp(png_jmpbuf(png))){png_destroy_read_struct(&png,&info,NULL);fclose(f);return NULL;}
    png_init_io(png,f);png_read_info(png,info);png_uint_32 w=png_get_image_width(png,info),h=png_get_image_height(png,info);int type=png_get_color_type(png,info),depth=png_get_bit_depth(png,info);if(depth==16)png_set_strip_16(png);if(type==PNG_COLOR_TYPE_PALETTE)png_set_palette_to_rgb(png);if(type==PNG_COLOR_TYPE_GRAY&&depth<8)png_set_expand_gray_1_2_4_to_8(png);if(png_get_valid(png,info,PNG_INFO_tRNS))png_set_tRNS_to_alpha(png);if(type==PNG_COLOR_TYPE_RGB||type==PNG_COLOR_TYPE_GRAY||type==PNG_COLOR_TYPE_PALETTE)png_set_filler(png,0xff,PNG_FILLER_AFTER);if(type==PNG_COLOR_TYPE_GRAY||type==PNG_COLOR_TYPE_GRAY_ALPHA)png_set_gray_to_rgb(png);png_read_update_info(png,info);
    unsigned char *rgba=ecalloc((size_t)w*h,4),**rows=ecalloc(h,sizeof *rows);for(png_uint_32 y=0;y<h;y++)rows[y]=rgba+(size_t)y*w*4;png_read_image(png,rows);free(rows);png_destroy_read_struct(&png,&info,NULL);fclose(f);
    double scale=(double)target/(double)(w>h?w:h);int dw=(int)(w*scale+.5),dh=(int)(h*scale+.5);if(dw<1)dw=1;if(dh<1)dh=1;cairo_surface_t *src=cairo_image_surface_create(CAIRO_FORMAT_ARGB32,(int)w,(int)h);unsigned char *dst=cairo_image_surface_get_data(src);int stride=cairo_image_surface_get_stride(src);for(png_uint_32 y=0;y<h;y++)for(png_uint_32 x=0;x<w;x++){unsigned char *s=rgba+((size_t)y*w+x)*4,*d=dst+y*stride+x*4;unsigned a=s[3];d[0]=(unsigned char)(s[2]*a/255);d[1]=(unsigned char)(s[1]*a/255);d[2]=(unsigned char)(s[0]*a/255);d[3]=(unsigned char)a;}free(rgba);cairo_surface_mark_dirty(src);
    cairo_surface_t *out=cairo_image_surface_create(CAIRO_FORMAT_ARGB32,dw,dh);cairo_t *cr=cairo_create(out);cairo_scale(cr,(double)dw/w,(double)dh/h);cairo_set_source_surface(cr,src,0,0);cairo_pattern_set_filter(cairo_get_source(cr),CAIRO_FILTER_BILINEAR);cairo_paint(cr);cairo_destroy(cr);cairo_surface_destroy(src);return out;
}

static cairo_surface_t *load_svg(const char *path,int target)
{
    NSVGimage *image=nsvgParseFromFile(path,"px",96);if(!image||image->width<=0||image->height<=0){nsvgDelete(image);return NULL;}float scale=(float)target/(image->width>image->height?image->width:image->height);int w=(int)(image->width*scale+.5f),h=(int)(image->height*scale+.5f);if(w<1)w=1;if(h<1)h=1;unsigned char *rgba=ecalloc((size_t)w*h,4);NSVGrasterizer *rast=nsvgCreateRasterizer();if(!rast){free(rgba);nsvgDelete(image);return NULL;}nsvgRasterize(rast,image,0,0,scale,rgba,w,h,w*4);nsvgDeleteRasterizer(rast);nsvgDelete(image);cairo_surface_t *out=cairo_image_surface_create(CAIRO_FORMAT_ARGB32,w,h);unsigned char *dst=cairo_image_surface_get_data(out);int stride=cairo_image_surface_get_stride(out);for(int y=0;y<h;y++)for(int x=0;x<w;x++){unsigned char *s=rgba+((size_t)y*w+x)*4,*d=dst+y*stride+x*4;unsigned a=s[3];d[0]=(unsigned char)(s[2]*a/255);d[1]=(unsigned char)(s[1]*a/255);d[2]=(unsigned char)(s[0]*a/255);d[3]=(unsigned char)a;}free(rgba);cairo_surface_mark_dirty(out);return out;
}

void icon_init(const char *theme){free(theme_name);theme_name=xstrdup(theme&&*theme?theme:"hicolor");}

cairo_surface_t *icon_load(const char *name,int size)
{
    if(!name||size<1)
        return NULL;
    for(struct cached_icon *c=cache;c;c=c->next)
        if(c->size==size&&!strcmp(c->name,name))
            return c->surface;
    char *path=resolve_icon(name,size);cairo_surface_t *surface=NULL;if(path){size_t n=strlen(path);if(n>4&&!strcasecmp(path+n-4,".svg"))surface=load_svg(path,size);else surface=load_png(path,size);free(path);}
    struct cached_icon *c=ecalloc(1,sizeof *c);c->name=xstrdup(name);c->size=size;c->surface=surface;c->next=cache;cache=c;return surface;
}
void icon_cleanup(void)
{
    while(cache){struct cached_icon *n=cache->next;if(cache->surface)cairo_surface_destroy(cache->surface);free(cache->name);free(cache);cache=n;}
    while(themes){struct theme *n=themes->next;while(themes->dirs){struct directory *d=themes->dirs->next;free(themes->dirs->name);free(themes->dirs);themes->dirs=d;}for(size_t i=0;i<themes->inherit_count;i++)free(themes->inherits[i]);free(themes->inherits);free(themes->name);free(themes);themes=n;}free(theme_name);theme_name=NULL;
}

int icon_probe(const char *theme,const char *name,int size)
{
    icon_init(theme);cairo_surface_t *surface=icon_load(name,size);int ok=surface!=NULL;icon_cleanup();return ok;
}
