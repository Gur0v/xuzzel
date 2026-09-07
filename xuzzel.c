/*
 * xuzzel - X11 application launcher and dmenu
 *
 * Drawing, keyboard handling and the event-loop structure derive from dmenu
 * 5.4 (see LICENSE).  XDG application, configuration and matching code is
 * original.  Fuzzel names are accepted for configuration compatibility;
 * this program contains no fuzzel source code.
 */
#define _DEFAULT_SOURCE
#define _POSIX_C_SOURCE 200809L
#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <getopt.h>
#include <locale.h>
#include <limits.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>
#include <wchar.h>
#include <wctype.h>

#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/keysym.h>
#include <X11/Xft/Xft.h>
#include <cairo/cairo-xlib.h>
#ifdef XINERAMA
#include <X11/extensions/Xinerama.h>
#endif

#include "drw.h"
#include "icon.h"
#include "util.h"

#define VERSION "1.15.0-x11.3"
#define TEXTSZ 4096
#define TEXTW(s) (drw_fontset_getwidth(drw, (s)))

enum { SchemeNorm, SchemePrompt, SchemeInput, SchemeMatch, SchemeSel,
       SchemeSelMatch, SchemeCounter, SchemeBorder, SchemeLast };
enum MatchMode { MATCH_EXACT, MATCH_FZF, MATCH_FUZZY };

struct config {
    char *font, *prompt, *placeholder, *terminal, *launch_prefix;
    char *cache, *output, *anchor, *icon_theme, *fields;
    char *colors[SchemeLast][2];
    int lines, width, tabs, hpad, vpad, inner_pad, border_width, icon_size;
    int line_height, letter_spacing, monitor;
    int x_margin, y_margin, select_index;
    unsigned fuzzy_min, fuzzy_discrepancy, fuzzy_distance;
    bool dmenu, password, password_char, icons, bold, minimal_lines;
        bool dmenu0;
    bool hide_prompt, hide_before_typing, counter, no_sort, auto_select;
    bool no_mouse, index, log_level_none;
    enum MatchMode match_mode;
};

struct item {
    char *text, *match, *exec, *icon, *desktop_id;
    size_t input_index;
    unsigned history;
    int score;
    unsigned match_class, match_pos, match_chunks;
    bool word_boundary;
};

static struct config cfg;
static struct item *items, **matches;
static size_t item_count, match_count, match_cap, selected, first;
static char text[TEXTSZ];
static size_t cursor;
static Display *dpy;
static int screen, sw, sh, mw, mh, bh;
static Window root, win;
static Drw *drw;
static Clr *scheme[SchemeLast];
static XIC xic;
static Atom utf8, clip, targets;
static char *config_path, *message, *search_text, *select_string;
static char *with_nth, *accept_nth, *match_nth, *nth_delim;
static bool only_match, print_timings, check_config;

static void usage(FILE *f);
static void cleanup(void);

static void *xrealloc(void *p, size_t n)
{
    void *q = realloc(p, n);
    if (!q && n) die("realloc:");
    return q;
}

static char *xstrdup(const char *s)
{
    char *p = strdup(s ? s : "");
    if (!p) die("strdup:");
    return p;
}

static char *trim(char *s)
{
    char *e;
    while (isspace((unsigned char)*s)) s++;
    e = s + strlen(s);
    while (e > s && isspace((unsigned char)e[-1])) *--e = '\0';
    return s;
}

static bool parse_bool(const char *s, bool *v)
{
    if (!strcasecmp(s, "yes") || !strcasecmp(s, "true") || !strcmp(s, "1")) *v = true;
    else if (!strcasecmp(s, "no") || !strcasecmp(s, "false") || !strcmp(s, "0")) *v = false;
    else return false;
    return true;
}

static int parse_int(const char *s, int min, int max, const char *name)
{
    char *end;
    long v;
    errno = 0; v = strtol(s, &end, 10);
    if (errno || *trim(end) || v < min || v > max)
        die("xuzzel: invalid %s: %s", name, s);
    return (int)v;
}

static char *color(const char *s)
{
    char buf[8];
    const char *p = s;
    size_t n;
    if (*p == '#') p++;
    n = strlen(p);
    if (n != 6 && n != 8) die("xuzzel: invalid color: %s", s);
    for (size_t i = 0; i < n; i++) if (!isxdigit((unsigned char)p[i])) die("xuzzel: invalid color: %s", s);
    snprintf(buf, sizeof buf, "#%.6s", p);
    return xstrdup(buf);
}

static void setstr(char **dst, const char *src)
{
    free(*dst); *dst = xstrdup(src);
}

static void setcolor(int s, int which, const char *v)
{
    free(cfg.colors[s][which]); cfg.colors[s][which] = color(v);
}

static void defaults(void)
{
    memset(&cfg, 0, sizeof cfg);
    cfg.font = xstrdup("monospace:size=10"); cfg.prompt = xstrdup("> ");
    cfg.placeholder = xstrdup(""); cfg.terminal = xstrdup("x-terminal-emulator -e");
    cfg.anchor = xstrdup("center"); cfg.icon_theme = xstrdup("hicolor");
    cfg.fields = xstrdup("filename,name,generic");
    nth_delim = xstrdup("\t");
    cfg.lines = 15; cfg.width = 30; cfg.tabs = 8; cfg.hpad = 40; cfg.vpad = 8;
    cfg.icon_size = 0;
    cfg.monitor = -1; cfg.select_index = -1; cfg.icons = true;
    cfg.fuzzy_min = 3; cfg.fuzzy_discrepancy = 2; cfg.fuzzy_distance = 1;
    cfg.match_mode = MATCH_FZF;
    const char *fg = "#657b83", *bg = "#fdf6e3";
    for (int i = 0; i < SchemeLast; i++) {
        cfg.colors[i][0] = xstrdup(fg); cfg.colors[i][1] = xstrdup(bg);
    }
    setcolor(SchemePrompt, 0, "#586e75"); setcolor(SchemeMatch, 0, "#cb4b16");
    setcolor(SchemeSel, 0, "#586e75"); setcolor(SchemeSel, 1, "#eee8d5");
    setcolor(SchemeSelMatch, 0, "#cb4b16"); setcolor(SchemeSelMatch, 1, "#eee8d5");
    setcolor(SchemeCounter, 0, "#93a1a1"); setcolor(SchemeBorder, 0, "#002b36");
}

static bool apply_key(const char *section, const char *key, const char *v, bool strict)
{
    bool b;
#define STR(k,f) if (!strcmp(key,k)) { setstr(&cfg.f,v); return true; }
#define INT(k,f,lo,hi) if (!strcmp(key,k)) { cfg.f=parse_int(v,lo,hi,k); return true; }
    if (!strcmp(key,"gamma-correct") || !strcmp(key,"gamma-correct-blending") ||
        !strcmp(key,"message-mode"))
        die("xuzzel: unsupported configuration key: %s",key);
    if (!strcmp(section,"colors")) {
        int sc = !strcmp(key,"background") ? SchemeNorm :
            !strcmp(key,"text") ? SchemeNorm : !strcmp(key,"prompt") ? SchemePrompt :
            !strcmp(key,"placeholder") ? SchemeInput : !strcmp(key,"input") ? SchemeInput :
            !strcmp(key,"match") ? SchemeMatch : !strcmp(key,"selection") ? SchemeSel :
            !strcmp(key,"selection-text") ? SchemeSel : !strcmp(key,"selection-match") ? SchemeSelMatch :
            !strcmp(key,"counter") ? SchemeCounter : !strcmp(key,"border") ? SchemeBorder : -1;
        if (sc >= 0) {
            int which = !strcmp(key,"background") || !strcmp(key,"selection") ? 1 : 0;
            if (!strcmp(key,"background")) {
                setcolor(SchemeNorm,1,v);
                setcolor(SchemePrompt,1,v);
                setcolor(SchemeInput,1,v);
                setcolor(SchemeMatch,1,v);
                setcolor(SchemeCounter,1,v);
                setcolor(SchemeBorder,1,v);
            } else {
                setcolor(sc,which,v);
            }
            if (!strcmp(key,"selection")) setcolor(SchemeSelMatch,1,v);
            return true;
        }
    }
        if (!strcmp(section,"border")) {
            if (!strcmp(key,"width")) { cfg.border_width=parse_int(v,0,1000,key); return true; }
            if (!strcmp(key,"radius")) return true;
        }
    STR("font",font) STR("prompt",prompt) STR("placeholder",placeholder)
    STR("terminal",terminal) STR("launch-prefix",launch_prefix) STR("output",output)
    STR("anchor",anchor) STR("icon-theme",icon_theme) STR("fields",fields)
    INT("icon-size",icon_size,0,4096)
    if (!strcmp(key,"message")) { setstr(&message,v); return true; }
    INT("lines",lines,0,100000) INT("width",width,1,100000) INT("tabs",tabs,1,64)
    INT("horizontal-pad",hpad,0,100000) INT("vertical-pad",vpad,0,100000)
    INT("inner-pad",inner_pad,0,10000) INT("border-width",border_width,0,1000)
    INT("line-height",line_height,0,10000)
    INT("letter-spacing",letter_spacing,-100,1000) INT("x-margin",x_margin,0,100000)
    INT("y-margin",y_margin,0,100000)
    if (!strcmp(key,"icons-enabled")) { if(!parse_bool(v,&b)) goto badbool; cfg.icons=b; return true; }
    if (!strcmp(key,"use-bold")) { if(!parse_bool(v,&b)) goto badbool; cfg.bold=b; return true; }
    if (!strcmp(key,"minimal-lines")) { if(!parse_bool(v,&b)) goto badbool; cfg.minimal_lines=b; return true; }
    if (!strcmp(key,"hide-prompt")) { if(!parse_bool(v,&b)) goto badbool; cfg.hide_prompt=b; return true; }
    if (!strcmp(key,"hide-before-typing")) { if(!parse_bool(v,&b)) goto badbool; cfg.hide_before_typing=b; return true; }
    if (!strcmp(key,"match-counter") || !strcmp(key,"counter")) { if(!parse_bool(v,&b)) goto badbool; cfg.counter=b; return true; }
    if (!strcmp(key,"no-sort")) { if(!parse_bool(v,&b)) goto badbool; cfg.no_sort=b; return true; }
    if (!strcmp(key,"sort-result")) { if(!parse_bool(v,&b)) goto badbool; cfg.no_sort=!b; return true; }
    if (!strcmp(key,"enable-mouse")) { if(!parse_bool(v,&b)) goto badbool; cfg.no_mouse=!b; return true; }
    if (!strcmp(key,"auto-select")) { if(!parse_bool(v,&b)) goto badbool; cfg.auto_select=b; return true; }
    if (!strcmp(key,"show-actions") || !strcmp(key,"filter-desktop")) {
        if (!parse_bool(v,&b)) die("xuzzel: %s expects yes/no",key);
        return true;
    }
    if (!strcmp(key,"match-mode")) {
        if (!strcmp(v,"exact")) cfg.match_mode=MATCH_EXACT;
        else if (!strcmp(v,"fzf")) cfg.match_mode=MATCH_FZF;
        else if (!strcmp(v,"fuzzy")) cfg.match_mode=MATCH_FUZZY;
        else die("xuzzel: invalid match-mode: %s",v);
        return true;
    }
    INT("fuzzy-min-length",fuzzy_min,0,100000)
    INT("fuzzy-max-length-discrepancy",fuzzy_discrepancy,0,100000)
    INT("fuzzy-max-distance",fuzzy_distance,0,100000)
    if (!strcmp(key,"background")) { setcolor(SchemeNorm,1,v); setcolor(SchemeInput,1,v); return true; }
    if (!strcmp(key,"text")) { setcolor(SchemeNorm,0,v); return true; }
    if (!strcmp(key,"prompt")) { setstr(&cfg.prompt,v); return true; }
    if (!strcmp(key,"prompt-color")) { setcolor(SchemePrompt,0,v); return true; }
    if (!strcmp(key,"placeholder-color")) { setcolor(SchemeInput,0,v); return true; }
    if (!strcmp(key,"input-color")) { setcolor(SchemeInput,0,v); return true; }
    if (!strcmp(key,"match")) { setcolor(SchemeMatch,0,v); return true; }
    if (!strcmp(key,"selection")) { setcolor(SchemeSel,1,v); setcolor(SchemeSelMatch,1,v); return true; }
    if (!strcmp(key,"selection-text")) { setcolor(SchemeSel,0,v); return true; }
    if (!strcmp(key,"selection-match")) { setcolor(SchemeSelMatch,0,v); return true; }
    if (!strcmp(key,"counter")) { setcolor(SchemeCounter,0,v); return true; }
    if (!strcmp(key,"border")) { setcolor(SchemeBorder,0,v); return true; }
    if (!strcmp(key,"message")) { setcolor(SchemePrompt,0,v); return true; }
    /* Parsed but X11-inapplicable or not yet rendered. */
    const char *known[] = {"namespace","dpi-aware","scaling-filter","layer","exit-on-keyboard-focus-loss",
        "image-size-ratio","icon-theme","match-fields","match-counter","match-workers",
        "delayed-filter-ms","delayed-filter-limit","render-workers","selection-radius",
        "list-executables-in-path",
        "show-actions","filter-desktop","password-character","default-mode","include",NULL};
    for (int i=0; known[i]; i++) if (!strcmp(key,known[i])) return true;
    if (strict) fprintf(stderr,"xuzzel: warning: unknown key [%s] %s\n",section,key);
    return false;
badbool:
    die("xuzzel: %s expects yes/no",key);
    return false;
#undef STR
#undef INT
}

static void read_config_file(const char *path, bool required)
{
    FILE *f = fopen(path,"r"); char *line=NULL; size_t cap=0; unsigned n=0; char section[64]="main";
    if (!f) { if (required) die("xuzzel: cannot open %s:",path); return; }
    while (getline(&line,&cap,f)>=0) {
        char *s=trim(line), *eq; n++;
        if (!*s || *s=='#' || *s==';') continue;
        if (*s=='[') { char *e=strchr(s,']'); if (!e) die("%s:%u: invalid section",path,n); *e='\0'; snprintf(section,sizeof section,"%s",s+1); continue; }
        if (!(eq=strchr(s,'='))) die("%s:%u: expected key=value",path,n);
        *eq++='\0';
        char *value=trim(eq);size_t value_len=strlen(value);
        if(value_len>=2&&value[0]=='"'&&value[value_len-1]=='"'){value[value_len-1]='\0';value++;}
        apply_key(section,trim(s),value,true);
    }
    if (ferror(f)) die("xuzzel: read %s:",path);
    free(line); fclose(f);
}

static void load_config(void)
{
    if (config_path) { if (strcmp(config_path,"/dev/null")) read_config_file(config_path,true); return; }
    const char *home=getenv("HOME"), *xdg=getenv("XDG_CONFIG_HOME"); char p[4096];
    const char *dirs[] = {"fuzzel/fuzzel.ini","xuzzel/xuzzel.ini",NULL};
    for (int i=0; dirs[i]; i++) {
        if (xdg && *xdg) snprintf(p,sizeof p,"%s/%s",xdg,dirs[i]);
        else if (home) snprintf(p,sizeof p,"%s/.config/%s",home,dirs[i]); else continue;
        if (!access(p,R_OK)) { read_config_file(p,false); return; }
    }
    const char *sys=getenv("XDG_CONFIG_DIRS"); if (!sys) sys="/etc/xdg";
    char *copy=xstrdup(sys), *save=NULL;
    for (char *d=strtok_r(copy,":",&save); d; d=strtok_r(NULL,":",&save))
        for(int i=0;dirs[i];i++){snprintf(p,sizeof p,"%s/%s",d,dirs[i]);if(!access(p,R_OK)){read_config_file(p,false);free(copy);return;}}
    free(copy);
}

static char *field_select(const char *s, const char *spec)
{
    if (!spec || !*spec) return xstrdup(s);
    char *out=xstrdup(""), *copy=xstrdup(s), *tok, *save=NULL, *sp=xstrdup(spec), *r, *ss=NULL;
    size_t len=0; int fields=0; char **v=NULL;
    for(tok=strtok_r(copy,nth_delim,&save);tok;tok=strtok_r(NULL,nth_delim,&save)){v=xrealloc(v,sizeof(*v)*(fields+1));v[fields++]=tok;}
    for(r=strtok_r(sp,",",&ss);r;r=strtok_r(NULL,",",&ss)){int a=atoi(r),b=a;char *dash=strchr(r,'-');if(dash)b=atoi(dash+1);if(a<0)a=fields+a+1;if(b<0)b=fields+b+1;if(!a)a=1;if(!b)b=fields;for(int i=a;i<=b&&i<=fields;i++)if(i>0){size_t z=strlen(v[i-1]);out=xrealloc(out,len+z+2);if(len)out[len++]=' ';memcpy(out+len,v[i-1],z+1);len+=z;}}
    free(v);free(sp);free(copy);return out;
}

static void add_item(const char *display,const char *match,const char *exec,const char *icon,const char *id)
{
    items=xrealloc(items,(item_count+1)*sizeof *items);
    items[item_count]=(struct item){.text=xstrdup(display),.match=xstrdup(match?match:display),.exec=exec?xstrdup(exec):NULL,.icon=icon?xstrdup(icon):NULL,.desktop_id=id?xstrdup(id):NULL,.input_index=item_count}; item_count++;
}

static void read_stdin_items(void)
{
    char *line=NULL;size_t cap=0;ssize_t n;
    int delim=cfg.dmenu0?'\0':'\n';
    while((n=getdelim(&line,&cap,delim,stdin))>=0){while(n&& (line[n-1]==delim||line[n-1]=='\r'))line[--n]='\0';char *d=with_nth?field_select(line,with_nth):xstrdup(line);char *m=match_nth?field_select(line,match_nth):xstrdup(d);add_item(d,m,line,NULL,NULL);free(d);free(m);} free(line);
}

static char *desktop_exec(const char *src)
{
    char *out=ecalloc(strlen(src)+1,1);size_t j=0;bool quote=false;
    for(size_t i=0;src[i];i++){if(src[i]=='"')quote=!quote;if(src[i]=='%'&&src[i+1]){char c=src[++i];if(c=='%')out[j++]='%';else if(strchr("fFuUdDnNickvm",c)){}else{}continue;}out[j++]=src[i];}
    while(j&&isspace((unsigned char)out[j-1]))j--;
    out[j]='\0';
    (void)quote;
    return out;
}

static void parse_desktop(const char *path,const char *id)
{
    FILE *f=fopen(path,"r");char *line=NULL,*name=NULL,*generic=NULL,*exec=NULL,*icon=NULL,*comment=NULL;size_t cap=0;bool entry=false,nodisplay=false,terminal=false;
    if(!f)return;
    while(getline(&line,&cap,f)>=0){char *s=trim(line),*eq;if(*s=='['){entry=!strcmp(s,"[Desktop Entry]");continue;}if(!entry||!(eq=strchr(s,'=')))continue;*eq++='\0';
        if(!strcmp(s,"Name")&&!name)name=xstrdup(eq);else if(!strcmp(s,"GenericName")&&!generic)generic=xstrdup(eq);else if(!strcmp(s,"Comment")&&!comment)comment=xstrdup(eq);else if(!strcmp(s,"Exec"))setstr(&exec,eq);else if(!strcmp(s,"Icon"))setstr(&icon,eq);else if(!strcmp(s,"NoDisplay")||!strcmp(s,"Hidden")){bool b;if(parse_bool(eq,&b)&&b)nodisplay=true;}else if(!strcmp(s,"Terminal")){bool b;if(parse_bool(eq,&b))terminal=b;}}
    fclose(f);free(line);
    if(name&&exec&&!nodisplay){char *clean=desktop_exec(exec),*cmd;if(terminal){size_t z=strlen(cfg.terminal)+strlen(clean)+2;cmd=ecalloc(z,1);snprintf(cmd,z,"%s %s",cfg.terminal,clean);free(clean);}else cmd=clean;size_t z=strlen(name)+(generic?strlen(generic):0)+(comment?strlen(comment):0)+strlen(id)+4;char *m=ecalloc(z,1);snprintf(m,z,"%s %s %s %s",name,generic?generic:"",comment?comment:"",id);add_item(name,m,cmd,icon,id);free(m);free(cmd);}
    free(name);free(generic);free(exec);free(icon);free(comment);
}

static bool have_id(const char *id){for(size_t i=0;i<item_count;i++)if(items[i].desktop_id&&!strcmp(items[i].desktop_id,id))return true;return false;}
static void scan_apps(const char *base,const char *rel)
{
    char dirpath[4096];snprintf(dirpath,sizeof dirpath,"%s/applications%s%s",base,*rel?"/":"",rel);DIR *d=opendir(dirpath);if(!d)return;struct dirent *e;
    while((e=readdir(d))){if(e->d_name[0]=='.')continue;char nr[2048];snprintf(nr,sizeof nr,"%s%s%s",rel,*rel?"/":"",e->d_name);char full[4096];snprintf(full,sizeof full,"%s/applications/%s",base,nr);struct stat st;if(stat(full,&st))continue;if(S_ISDIR(st.st_mode))scan_apps(base,nr);else{size_t l=strlen(nr);if(l>8&&!strcmp(nr+l-8,".desktop")){char id[2048];snprintf(id,sizeof id,"%s",nr);for(char*p=id;*p;p++)if(*p=='/')*p='-';if(!have_id(id))parse_desktop(full,id);}}}closedir(d);
}

static void read_apps(void)
{
    const char *home=getenv("HOME"),*data=getenv("XDG_DATA_HOME");char p[4096];if(data&&*data)scan_apps(data,"");else if(home){snprintf(p,sizeof p,"%s/.local/share",home);scan_apps(p,"");}
    const char *dirs=getenv("XDG_DATA_DIRS");if(!dirs)dirs="/usr/local/share:/usr/share";char *c=xstrdup(dirs),*save=NULL;for(char*d=strtok_r(c,":",&save);d;d=strtok_r(NULL,":",&save))scan_apps(d,"");free(c);
}

static char *cache_path(void)
{
    if(cfg.cache)return xstrdup(cfg.cache);
    const char *x=getenv("XDG_CACHE_HOME"),*h=getenv("HOME");char p[4096];if(x&&*x)snprintf(p,sizeof p,"%s/xuzzel",x);else if(h)snprintf(p,sizeof p,"%s/.cache/xuzzel",h);else return NULL;mkdir(p,0700);size_t n=strlen(p)+16;char*r=ecalloc(n,1);snprintf(r,n,"%s/history",p);return r;
}
static void read_history(void){if(cfg.dmenu&&!cfg.cache)return;char*p=cache_path();if(!p)return;FILE*f=fopen(p,"r");free(p);if(!f)return;char*line=NULL;size_t cap=0;unsigned rank=1000000;while(getline(&line,&cap,f)>=0){char*s=trim(line);for(size_t i=0;i<item_count;i++)if(items[i].desktop_id&&!strcmp(items[i].desktop_id,s)){items[i].history=rank--;break;}}free(line);fclose(f);}
static void write_history(struct item *it){if(!it||!it->desktop_id)return;char*p=cache_path();if(!p)return;FILE*f=fopen(p,"w");if(f){fprintf(f,"%s\n",it->desktop_id);for(size_t i=0;i<item_count;i++)if(items[i].desktop_id&&strcmp(items[i].desktop_id,it->desktop_id)&&items[i].history)fprintf(f,"%s\n",items[i].desktop_id);fclose(f);}free(p);}

static int fzf_score(const char *hay,const char *needle)
{
    if(!*needle)return 0;
    int score=0,last=-2;size_t j=0;for(int i=0;hay[i]&&needle[j];i++)if(tolower((unsigned char)hay[i])==tolower((unsigned char)needle[j])){score+=10;if(i==last+1)score+=15;if(i==0||strchr(" /_-.",hay[i-1]))score+=20;score-=i/8;last=i;j++;}return needle[j]?-100000:score;
}
static int exact_score(const char*h,const char*n){if(!*n)return 0;char *hl=xstrdup(h),*nl=xstrdup(n);for(char*p=hl;*p;p++)*p=tolower((unsigned char)*p);for(char*p=nl;*p;p++)*p=tolower((unsigned char)*p);char*q=strstr(hl,nl);int r=q?1000-(int)(q-hl):-100000;free(hl);free(nl);return r;}

static wchar_t *lower_wide(const char *s,size_t *len)
{
    size_t cap=strlen(s)+1,n=0;wchar_t *out=ecalloc(cap,sizeof *out);mbstate_t st={0};
    while(*s){wchar_t wc;size_t z=mbrtowc(&wc,s,MB_CUR_MAX,&st);if(z==(size_t)-1||z==(size_t)-2){memset(&st,0,sizeof st);wc=(unsigned char)*s;z=1;}else if(z==0)break;out[n++]=towlower(wc);s+=z;}
    out[n]=L'\0';*len=n;return out;
}
static unsigned levenshtein(const wchar_t *a,size_t alen,const wchar_t *b,size_t blen)
{
    unsigned *prev=ecalloc(blen+1,sizeof *prev),*cur=ecalloc(blen+1,sizeof *cur);
    for(size_t j=0;j<=blen;j++)prev[j]=(unsigned)j;
    for(size_t i=1;i<=alen;i++){cur[0]=(unsigned)i;for(size_t j=1;j<=blen;j++){unsigned del=prev[j]+1,ins=cur[j-1]+1,sub=prev[j-1]+(a[i-1]!=b[j-1]);cur[j]=del<ins?del:ins;if(sub<cur[j])cur[j]=sub;}unsigned *tmp=prev;prev=cur;cur=tmp;}
    unsigned ret=prev[blen];free(prev);free(cur);return ret;
}
static bool wide_boundary(const wchar_t *s,size_t pos){return pos==0||s[pos-1]==L' '||s[pos-1]==L'/'||s[pos-1]==L'_'||s[pos-1]==L'-'||s[pos-1]==L'.';}
static bool fuzzy_token(const wchar_t *hay,size_t hlen,const wchar_t *tok,size_t tlen,unsigned *klass,unsigned *distance,size_t *pos)
{
    if(tlen==0){*klass=0;*distance=0;*pos=0;return true;}
    for(size_t i=0;i+tlen<=hlen;i++)if(!wmemcmp(hay+i,tok,tlen)){*klass=0;*distance=0;*pos=i;return true;}
    if(tlen<cfg.fuzzy_min||hlen<tlen)return false;
    unsigned best=UINT_MAX;size_t bestpos=0,bestlen=0;
    size_t minlen=tlen>cfg.fuzzy_discrepancy?tlen-cfg.fuzzy_discrepancy:1,maxlen=tlen+cfg.fuzzy_discrepancy;if(maxlen>hlen)maxlen=hlen;
    for(size_t n=minlen;n<=maxlen;n++)for(size_t i=0;i+n<=hlen;i++){unsigned d=levenshtein(hay+i,n,tok,tlen);if(d<best||(d==best&&(i<bestpos||(i==bestpos&&n<bestlen)))){best=d;bestpos=i;bestlen=n;}}
    if(best>cfg.fuzzy_distance)return false;
    *klass=1;*distance=best;*pos=bestpos;return true;
}
static int fuzzy_score(struct item *item,const char *query)
{
    size_t hlen,qlen;wchar_t *hay=lower_wide(item->match,&hlen),*copy=lower_wide(query,&qlen),*p=copy;
    unsigned klass=0,total=0,chunks=0;size_t first_pos=SIZE_MAX;bool boundary=false;
    while(*p){while(*p==L' ')p++;if(!*p)break;wchar_t *end=p;while(*end&&*end!=L' ')end++;wchar_t save=*end;*end=L'\0';unsigned k,d;size_t pos;
        if(!fuzzy_token(hay,hlen,p,(size_t)(end-p),&k,&d,&pos)){free(hay);free(copy);return -100000;}if(k>klass)klass=k;total+=d;chunks++;if(pos<first_pos){first_pos=pos;boundary=wide_boundary(hay,pos);}*end=save;p=end;}
    item->match_class=klass;item->match_pos=first_pos==SIZE_MAX?0:(unsigned)first_pos;item->match_chunks=chunks;item->word_boundary=boundary;free(hay);free(copy);return klass?-(int)total:(int)qlen;
}
static int cmp_match(const void*a,const void*b){const struct item*x=*(const struct item*const*)a,*y=*(const struct item*const*)b;if(!cfg.no_sort){if(cfg.match_mode==MATCH_FUZZY){if(x->match_class!=y->match_class)return x->match_class<y->match_class?-1:1;if(x->score!=y->score)return x->score>y->score?-1:1;if(x->word_boundary!=y->word_boundary)return x->word_boundary?-1:1;if(x->match_chunks!=y->match_chunks)return x->match_chunks<y->match_chunks?-1:1;if(x->history!=y->history)return x->history>y->history?-1:1;if(x->match_pos!=y->match_pos)return x->match_pos<y->match_pos?-1:1;}else{if(x->score!=y->score)return y->score-x->score;if(x->history!=y->history)return y->history>x->history?1:-1;}}return x->input_index>y->input_index?1:x->input_index<y->input_index?-1:0;}
static void match_items(void)
{
    match_count=0;for(size_t i=0;i<item_count;i++){int s=cfg.match_mode==MATCH_EXACT?exact_score(items[i].match,text):cfg.match_mode==MATCH_FUZZY?fuzzy_score(&items[i],text):fzf_score(items[i].match,text);if(s>-100000){items[i].score=s;if(match_count==match_cap){match_cap=match_cap?match_cap*2:256;matches=xrealloc(matches,match_cap*sizeof *matches);}matches[match_count++]=&items[i];}}
    qsort(matches,match_count,sizeof *matches,cmp_match);selected=0;first=0;
    if(select_string)for(size_t i=0;i<match_count;i++)if(!strcmp(matches[i]->text,select_string)){selected=i;break;}else{}
    else if(cfg.select_index>=0&&(size_t)cfg.select_index<match_count)selected=(size_t)cfg.select_index;
}

static size_t nextrune(int inc){ssize_t n=(ssize_t)cursor+inc;while(n>0&&n<(ssize_t)sizeof text&&(text[n]&0xc0)==0x80)n+=inc;return (size_t)n;}
static void insert(const char *str,ssize_t n){if(strlen(text)+n>=sizeof text-1)n=(ssize_t)sizeof text-strlen(text)-1;if(n>0){memmove(text+cursor+n,text+cursor,strlen(text+cursor)+1);memcpy(text+cursor,str,n);cursor+=n;}else if(cursor>0){memmove(text+cursor+n,text+cursor,strlen(text+cursor)+1);cursor+=n;}match_items();}

static void draw_highlight(const char *s,int x,int y,int w,bool sel)
{
    drw_setscheme(drw,scheme[sel?SchemeSel:SchemeNorm]);drw_text(drw,x,y,w,bh,0,s,0);
    if(!*text)return;
    char *low=xstrdup(s),*needle=xstrdup(text);for(char*p=low;*p;p++)*p=tolower((unsigned char)*p);for(char*p=needle;*p;p++)*p=tolower((unsigned char)*p);char *p=strstr(low,needle);if(p){size_t pre=(size_t)(p-low),n=strlen(needle);char save=((char*)s)[pre];char *prefix=xstrdup(s);prefix[pre]='\0';int px=x+(int)drw_fontset_getwidth(drw,prefix);free(prefix);char *hit=xstrdup(s+pre);hit[n]='\0';int pw=(int)drw_fontset_getwidth(drw,hit);drw_setscheme(drw,scheme[sel?SchemeSelMatch:SchemeMatch]);drw_text(drw,px,y,pw,bh,0,hit,0);(void)save;free(hit);}free(low);free(needle);
}
static void draw_icon(cairo_surface_t *icon,int x,int y,int rowh)
{
    int ih=cairo_image_surface_get_height(icon);
    cairo_surface_t *target=cairo_xlib_surface_create(dpy,drw->drawable,DefaultVisual(dpy,screen),mw,mh);
    cairo_t *cr=cairo_create(target);cairo_set_source_surface(cr,icon,x,y+(rowh-ih)/2);cairo_paint(cr);
    cairo_destroy(cr);cairo_surface_destroy(target);
}
static void drawmenu(void)
{
    int x=cfg.hpad,y=cfg.vpad,w=mw-2*cfg.hpad,promptw=0;
    drw_setscheme(drw,scheme[SchemeNorm]);drw_rect(drw,0,0,mw,mh,1,1);
    if(message){drw_setscheme(drw,scheme[SchemePrompt]);drw_text(drw,x,y,w,bh,0,message,0);y+=bh;}
    if(!cfg.hide_prompt&&cfg.prompt&&*cfg.prompt){promptw=TEXTW(cfg.prompt);drw_setscheme(drw,scheme[SchemePrompt]);drw_text(drw,x,y,promptw,bh,0,cfg.prompt,0);}
    char shown[TEXTSZ];const char *input=text;if(cfg.password&&*text){size_t n=strlen(text),j=0;const char *bullet=cfg.password_char?"*":"•";shown[0]='\0';while(j+strlen(bullet)<sizeof shown&&n--){strcat(shown,bullet);j+=strlen(bullet);}input=shown;}else if(!*text&&cfg.placeholder)input=cfg.placeholder;
    drw_setscheme(drw,scheme[SchemeInput]);drw_text(drw,x+promptw,y,w-promptw,bh,0,input,0);y+=bh;
    if(cfg.lines>0&&!(cfg.hide_before_typing&&!*text))y+=cfg.inner_pad;
    if(!(cfg.hide_before_typing&&!*text)){size_t shown_n=MIN((size_t)cfg.lines,match_count-first);for(size_t k=0;k<shown_n;k++){size_t i=first+k;bool sel=i==selected;cairo_surface_t *icon=!cfg.dmenu&&cfg.icons?icon_load(matches[i]->icon,cfg.icon_size?cfg.icon_size:MAX(1,bh-4)):NULL;int iw=icon?cairo_image_surface_get_width(icon):0;if(sel){drw_setscheme(drw,scheme[SchemeSel]);drw_rect(drw,x,y,w,bh,1,1);}draw_highlight(matches[i]->text,x+(icon?iw+6:0),y,w-(icon?iw+6:0),sel);if(icon)draw_icon(icon,x,y,bh);y+=bh;}}
    if(cfg.counter){char b[64];snprintf(b,sizeof b,"%zu/%zu",match_count,item_count);int cw=TEXTW(b);drw_setscheme(drw,scheme[SchemeCounter]);drw_text(drw,mw-cfg.hpad-cw,cfg.vpad,cw,bh,0,b,0);}
    drw_map(drw,win,0,0,mw,mh);
}

static void paste(void){XConvertSelection(dpy,clip,utf8,utf8,win,CurrentTime);}

static void launch_command(const char *command)
{
    size_t n=strlen(command)+(cfg.launch_prefix?strlen(cfg.launch_prefix):0)+2;
    char *cmd=ecalloc(n,1);
    snprintf(cmd,n,"%s%s%s",cfg.launch_prefix?cfg.launch_prefix:"",cfg.launch_prefix?" ":"",command);
    pid_t child=fork();
    if(child==0){setsid();execl("/bin/sh","sh","-c",cmd,(char*)NULL);_exit(127);}
    free(cmd);
}

static bool command_word_exists(const char *input)
{
    while(isspace((unsigned char)*input))input++;
    size_t len=strcspn(input," \t\r\n");
    if(!len)return false;
    char *word=ecalloc(len+1,1);memcpy(word,input,len);
    static const char *builtins[]={"cd","alias","bg","break","command","continue","eval","exec","exit","export","fg","getopts","hash","jobs","pwd","read","readonly","return","set","shift","test","times","trap","type","ulimit","umask","unalias","unset","wait",NULL};
    for(size_t i=0;builtins[i];i++)if(!strcmp(word,builtins[i])){free(word);return true;}
    bool found=false;
    if(strchr(word,'/'))found=!access(word,X_OK);
    else{
        const char *path=getenv("PATH");char *copy=xstrdup(path&&*path?path:"/usr/local/bin:/usr/bin:/bin"),*save=NULL;
        for(char *dir=strtok_r(copy,":",&save);dir&&!found;dir=strtok_r(NULL,":",&save)){char candidate[PATH_MAX];snprintf(candidate,sizeof candidate,"%s/%s",*dir?dir:".",word);found=!access(candidate,X_OK);}
        free(copy);
    }
    free(word);return found;
}

static struct item *exact_launcher_item(const char *input)
{
    if(!input||!*input)return NULL;
    for(size_t i=0;i<match_count;i++)
        if(matches[i]->desktop_id&&!strcasecmp(matches[i]->text,input))
            return matches[i];
    return NULL;
}

static void accept(bool input)
{
    struct item *it=match_count?matches[selected]:NULL;if(!input&&only_match&&!it)return;
    const char *out=input||!it?text:(it->exec?it->exec:it->text);
    if(cfg.dmenu){if(cfg.index&&it)printf("%zu\n",it->input_index);else if(it&&accept_nth){char*r=field_select(it->exec?it->exec:it->text,accept_nth);puts(r);free(r);}else puts(out);fflush(stdout);}else{
        struct item *exact=input?NULL:exact_launcher_item(text);
        if(exact)it=exact;
        if((input||(!exact&&(!it||command_word_exists(text))))&&*text){launch_command(text);}
        else if(it){write_history(it);launch_command(it->exec?it->exec:it->text);}
    }cleanup();exit(0);
}
static void keypress(XKeyEvent *ev)
{
    char buf[64];KeySym ksym=NoSymbol;Status status;int len=xic?Xutf8LookupString(xic,ev,buf,sizeof buf-1,&ksym,&status):XLookupString(ev,buf,sizeof buf-1,&ksym,NULL);if(len<0)len=0;buf[len]='\0';
    bool ctrl=ev->state&ControlMask,shift=ev->state&ShiftMask;
    if(ctrl){if(ksym==XK_a)ksym=XK_Home;else if(ksym==XK_e)ksym=XK_End;else if(ksym==XK_b)ksym=XK_Left;else if(ksym==XK_f)ksym=XK_Right;else if(ksym==XK_p)ksym=XK_Up;else if(ksym==XK_n)ksym=XK_Down;else if(ksym==XK_h)ksym=XK_BackSpace;else if(ksym==XK_d)ksym=XK_Delete;else if(ksym==XK_u){insert(NULL,-(ssize_t)cursor);drawmenu();return;}else if(ksym==XK_k){text[cursor]='\0';match_items();drawmenu();return;}else if(ksym==XK_w){size_t old=cursor;while(cursor&&isspace((unsigned char)text[cursor-1]))cursor=nextrune(-1);while(cursor&&!isspace((unsigned char)text[cursor-1]))cursor=nextrune(-1);memmove(text+cursor,text+old,strlen(text+old)+1);match_items();drawmenu();return;}else if(ksym==XK_v){paste();return;}else if(ksym==XK_c){cleanup();exit(1);}}
    switch(ksym){case XK_Escape:cleanup();exit(1);case XK_Return:case XK_KP_Enter:accept(shift);break;case XK_BackSpace:if(cursor)insert(NULL,(ssize_t)nextrune(-1)-(ssize_t)cursor);break;case XK_Delete:if(text[cursor]){size_t n=nextrune(1);memmove(text+cursor,text+n,strlen(text+n)+1);match_items();}break;case XK_Left:if(cursor)cursor=nextrune(-1);break;case XK_Right:if(text[cursor])cursor=nextrune(1);break;case XK_Home:cursor=0;break;case XK_End:cursor=strlen(text);break;case XK_Up:if(selected){selected--;if(selected<first)first=selected;}break;case XK_Down:if(selected+1<match_count){selected++;if(selected>=first+(size_t)cfg.lines)first++;}break;case XK_Page_Up:selected=selected>(size_t)cfg.lines?selected-cfg.lines:0;first=selected;break;case XK_Page_Down:selected=MIN(match_count?match_count-1:0,selected+(size_t)cfg.lines);first=selected;break;case XK_Tab:if(match_count){snprintf(text,sizeof text,"%s",matches[selected]->text);cursor=strlen(text);match_items();}break;default:if(!ctrl&&len&&!(ev->state&Mod1Mask))insert(buf,len);break;}drawmenu();
}
static void buttonpress(XButtonEvent *e){if(cfg.no_mouse)return;if(e->button==Button4){if(selected)selected--;}else if(e->button==Button5){if(selected+1<match_count)selected++;}else if(e->button==Button1){int list_y=cfg.vpad+bh*(1+(message?1:0))+cfg.inner_pad;int row=(e->y-list_y)/bh;if(e->y>=list_y&&row>=0&&first+(size_t)row<match_count){selected=first+(size_t)row;accept(false);}}drawmenu();}

static void setup(void)
{
    XSetWindowAttributes swa;XIM xim;XWindowAttributes wa;
    screen=DefaultScreen(dpy);root=RootWindow(dpy,screen);XGetWindowAttributes(dpy,root,&wa);sw=wa.width;sh=wa.height;
    drw=drw_create(dpy,screen,root,sw,sh);Fnt *fonts=drw_fontset_create(drw,(const char**)&cfg.font,1);if(!fonts)die("xuzzel: cannot load font %s",cfg.font);drw_setfontset(drw,fonts);bh=cfg.line_height?cfg.line_height:(int)drw->fonts->h;if(!cfg.dmenu&&cfg.icons&&cfg.icon_size>0)bh=MAX(bh,cfg.icon_size+4);
    int visible=cfg.lines;if(cfg.minimal_lines)visible=MIN(visible,(int)match_count);int gap=visible>0?cfg.inner_pad:0;mh=2*cfg.vpad+bh*(1+visible+(message?1:0))+gap;
    int char_width=(int)drw_fontset_getwidth(drw,"o")+cfg.letter_spacing;
    if(char_width<0)char_width=0;
    mw=cfg.width*char_width+2*cfg.hpad;mw=MIN(mw,sw-2*(cfg.x_margin+cfg.border_width));mh=MIN(mh,sh-2*(cfg.y_margin+cfg.border_width));
    int outer_w=mw+2*cfg.border_width,outer_h=mh+2*cfg.border_width;
    int x=(sw-outer_w)/2,y=(sh-outer_h)/2;if(strstr(cfg.anchor,"left"))x=cfg.x_margin;if(strstr(cfg.anchor,"right"))x=sw-outer_w-cfg.x_margin;if(strstr(cfg.anchor,"top"))y=cfg.y_margin;if(strstr(cfg.anchor,"bottom"))y=sh-outer_h-cfg.y_margin;
#ifdef XINERAMA
    if(XineramaIsActive(dpy)){int n;XineramaScreenInfo*si=XineramaQueryScreens(dpy,&n);int m=cfg.monitor>=0?MIN(cfg.monitor,n-1):0;if(si){x=si[m].x_org+(si[m].width-outer_w)/2;y=si[m].y_org+(si[m].height-outer_h)/2;if(strstr(cfg.anchor,"left"))x=si[m].x_org+cfg.x_margin;if(strstr(cfg.anchor,"right"))x=si[m].x_org+si[m].width-outer_w-cfg.x_margin;if(strstr(cfg.anchor,"top"))y=si[m].y_org+cfg.y_margin;if(strstr(cfg.anchor,"bottom"))y=si[m].y_org+si[m].height-outer_h-cfg.y_margin;XFree(si);}}
#endif
    for(int i=0;i<SchemeLast;i++)scheme[i]=drw_scm_create(drw,(const char**)cfg.colors[i],2);
    swa.override_redirect=True;swa.background_pixel=scheme[SchemeNorm][ColBg].pixel;swa.border_pixel=scheme[SchemeBorder][ColFg].pixel;
    swa.event_mask=ExposureMask|KeyPressMask|FocusChangeMask;
    if(!cfg.no_mouse)swa.event_mask|=ButtonPressMask;
    win=XCreateWindow(dpy,root,x,y,mw,mh,cfg.border_width,CopyFromParent,CopyFromParent,CopyFromParent,CWOverrideRedirect|CWBackPixel|CWBorderPixel|CWEventMask,&swa);XStoreName(dpy,win,"xuzzel");
    Atom window_type=XInternAtom(dpy,"_NET_WM_WINDOW_TYPE",False);
    Atom utility_type=XInternAtom(dpy,"_NET_WM_WINDOW_TYPE_UTILITY",False);
    XChangeProperty(dpy,win,window_type,XA_ATOM,32,PropModeReplace,
                    (unsigned char*)&utility_type,1);
    struct { unsigned long flags,functions,decorations,input_mode,status; } motif={2,0,0,0,0};
    Atom motif_hints=XInternAtom(dpy,"_MOTIF_WM_HINTS",False);
    XChangeProperty(dpy,win,motif_hints,motif_hints,32,PropModeReplace,
                    (unsigned char*)&motif,5);
    char resource_name[]="xuzzel",resource_class[]="xuzzel";
    XClassHint ch={resource_name,resource_class};XSetClassHint(dpy,win,&ch);XMapRaised(dpy,win);XSetInputFocus(dpy,win,RevertToParent,CurrentTime);
    xim=XOpenIM(dpy,NULL,NULL,NULL);if(xim)xic=XCreateIC(xim,XNInputStyle,XIMPreeditNothing|XIMStatusNothing,XNClientWindow,win,XNFocusWindow,win,NULL);
    utf8=XInternAtom(dpy,"UTF8_STRING",False);clip=XInternAtom(dpy,"CLIPBOARD",False);targets=XInternAtom(dpy,"TARGETS",False);(void)targets;
    int grab = AlreadyGrabbed;
    for (int i = 0; i < 1000 && grab != GrabSuccess; i++) {
        grab = XGrabKeyboard(dpy, root, True, GrabModeAsync, GrabModeAsync,
                             CurrentTime);
        if (grab != GrabSuccess)
            usleep(1000);
    }
    if (grab != GrabSuccess)
        die("xuzzel: cannot grab keyboard");
    if (!cfg.no_mouse) {
        int pointer_grab = XGrabPointer(
            dpy, root, True, ButtonPressMask, GrabModeAsync, GrabModeAsync,
            None, None, CurrentTime);
        if (pointer_grab != GrabSuccess)
            die("xuzzel: cannot grab pointer");
    }
}
static void run(void)
{
    XEvent ev;while(!XNextEvent(dpy,&ev)){if(xic&&XFilterEvent(&ev,win))continue;switch(ev.type){case Expose:if(!ev.xexpose.count)drawmenu();break;case KeyPress:keypress(&ev.xkey);break;case ButtonPress:if(!cfg.no_mouse){if(ev.xbutton.window!=win){cleanup();exit(1);}buttonpress(&ev.xbutton);}break;case SelectionNotify:if(ev.xselection.property){Atom da;int di;unsigned long n,left;unsigned char*p=NULL;if(XGetWindowProperty(dpy,win,utf8,0,TEXTSZ/4,True,utf8,&da,&di,&n,&left,&p)==Success&&p){insert((char*)p,(ssize_t)n);XFree(p);drawmenu();}}break;}}
}
static void cleanup(void){icon_cleanup();if(!dpy)return;if(xic)XDestroyIC(xic);XUngrabPointer(dpy,CurrentTime);XUngrabKeyboard(dpy,CurrentTime);if(win)XDestroyWindow(dpy,win);for(int i=0;i<SchemeLast;i++)free(scheme[i]);if(drw)drw_free(drw);XCloseDisplay(dpy);dpy=NULL;}

/* Values follow fuzzel 1.15.0. Compatibility-only flags are intentionally accepted. */
    enum { O_CONFIG=256,O_CHECK,O_CACHE,O_OVERRIDE,O_BOLD,O_ICON_THEME,O_ICON_SIZE,O_PASSWORD,O_XMARGIN,O_YMARGIN,O_SELECT,O_SELECT_INDEX,O_TABS,O_PROMPT_COLOR,O_PLACEHOLDER_COLOR,O_INPUT_COLOR,O_COUNTER_COLOR,O_SELECTION_RADIUS,O_SHOW_ACTIONS,O_MATCH_MODE,O_NO_SORT,O_COUNTER,O_FILTER_DESKTOP,O_FUZZY_MIN,O_FUZZY_DISC,O_FUZZY_DIST,O_LINE_HEIGHT,O_LETTER_SPACING,O_LAYER,O_EXIT_FOCUS,O_LAUNCH_PREFIX,O_DMENU,O_DMENU0,O_INDEX,O_LOG_LEVEL,O_LIST_EXEC,O_NTH,O_WITH_NTH,O_ACCEPT_NTH,O_DELIM,O_ONLY_MATCH,O_AUTO_SELECT,O_MESSAGE,O_MESSAGE_MODE,O_NO_MOUSE,O_HIDE,O_HIDE_PROMPT,O_MINIMAL,O_DELAY_MS,O_DELAY_LIMIT,O_SEARCH,O_CACHE_ONLY,O_NO_CACHE,O_TIMINGS,O_ANCHOR};
static const struct option opts[]={
{"prompt",1,0,'p'},{"icon-size",1,0,O_ICON_SIZE},
{"config",1,0,O_CONFIG},{"check-config",0,0,O_CHECK},{"namespace",1,0,'n'},{"cache",1,0,O_CACHE},{"override",1,0,O_OVERRIDE},{"output",1,0,'o'},{"font",1,0,'f'},{"use-bold",0,0,O_BOLD},{"dpi-aware",1,0,'D'},{"gamma-correct",0,0,0},{"icon-theme",1,0,O_ICON_THEME},{"no-icons",0,0,'I'},{"hide-before-typing",0,0,O_HIDE},{"fields",1,0,'F'},{"password",2,0,O_PASSWORD},{"anchor",1,0,'a'},{"x-margin",1,0,O_XMARGIN},{"y-margin",1,0,O_YMARGIN},{"select",1,0,O_SELECT},{"select-index",1,0,O_SELECT_INDEX},{"lines",1,0,'l'},{"minimal-lines",0,0,O_MINIMAL},{"hide-prompt",0,0,O_HIDE_PROMPT},{"width",1,0,'w'},{"tabs",1,0,O_TABS},{"horizontal-pad",1,0,'x'},{"vertical-pad",1,0,'y'},{"inner-pad",1,0,'P'},{"background-color",1,0,'b'},{"text-color",1,0,'t'},{"message-color",1,0,0},{"prompt-color",1,0,O_PROMPT_COLOR},{"placeholder-color",1,0,O_PLACEHOLDER_COLOR},{"input-color",1,0,O_INPUT_COLOR},{"match-color",1,0,'m'},{"selection-color",1,0,'s'},{"selection-text-color",1,0,'S'},{"selection-match-color",1,0,'M'},{"selection-radius",1,0,O_SELECTION_RADIUS},{"counter-color",1,0,O_COUNTER_COLOR},{"border-width",1,0,'B'},{"border-color",1,0,'C'},{"show-actions",0,0,O_SHOW_ACTIONS},{"match-mode",1,0,O_MATCH_MODE},{"no-sort",0,0,O_NO_SORT},{"counter",0,0,O_COUNTER},{"filter-desktop",2,0,O_FILTER_DESKTOP},{"fuzzy-min-length",1,0,O_FUZZY_MIN},{"fuzzy-max-length-discrepancy",1,0,O_FUZZY_DISC},{"fuzzy-max-distance",1,0,O_FUZZY_DIST},{"line-height",1,0,O_LINE_HEIGHT},{"letter-spacing",1,0,O_LETTER_SPACING},{"layer",1,0,O_LAYER},{"exit-on-keyboard-focus-loss",1,0,O_EXIT_FOCUS},{"launch-prefix",1,0,O_LAUNCH_PREFIX},{"dmenu",0,0,'d'},{"dmenu0",0,0,O_DMENU0},{"index",0,0,O_INDEX},{"log-level",1,0,O_LOG_LEVEL},{"list-executables-in-path",1,0,O_LIST_EXEC},{"dmenu-match-nth",1,0,O_NTH},{"dmenu-with-nth",1,0,O_WITH_NTH},{"dmenu-accept-nth",1,0,O_ACCEPT_NTH},{"dmenu-nth-delimiter",1,0,O_DELIM},{"dmenu-only-match",0,0,O_ONLY_MATCH},{"auto-select",0,0,O_AUTO_SELECT},{"dmenu-message",1,0,O_MESSAGE},{"dmenu-message-mode",1,0,O_MESSAGE_MODE},{"no-mouse",0,0,O_NO_MOUSE},{"search",1,0,O_SEARCH},{"print-timings",0,0,O_TIMINGS},{"version",0,0,'v'},{"help",0,0,'h'},{0,0,0,0}};

static void parse_cli(int argc,char **argv,bool late)
{
    int c;optind=1;opterr=0;while((c=getopt_long(argc,argv,":n:o:f:D:IF:ia:l:w:x:y:p:P:b:t:m:s:S:M:B:C:TdRvh",opts,NULL))!=-1){
        if(late&&c==O_ICON_SIZE){cfg.icon_size=parse_int(optarg,0,4096,"icon-size");continue;}
        if(!late){if(c==O_CONFIG)setstr(&config_path,optarg);else if(c==O_CHECK)check_config=true;else if(c=='h'){usage(stdout);exit(0);}else if(c==O_MESSAGE_MODE){die("xuzzel: unsupported option: --dmenu-message-mode");}else if(c=='v'){puts("xuzzel " VERSION);exit(0);}continue;}
        switch(c){case O_CONFIG:case O_CHECK:break;case 'n':break;case O_CACHE:setstr(&cfg.cache,optarg);break;case O_OVERRIDE:{char*x=xstrdup(optarg),*eq=strchr(x,'=');if(!eq)die("xuzzel: --override expects [section.]key=value");*eq++='\0';char*dot=strchr(x,'.');if(dot){*dot++='\0';apply_key(x,dot,eq,true);}else apply_key("main",x,eq,true);free(x);break;}case 'o':setstr(&cfg.output,optarg);if(isdigit((unsigned char)*optarg))cfg.monitor=atoi(optarg);break;case 'f':setstr(&cfg.font,optarg);break;case O_BOLD:cfg.bold=true;break;case 'D':break;case O_ICON_THEME:setstr(&cfg.icon_theme,optarg);break;case 'I':cfg.icons=false;break;case O_HIDE:cfg.hide_before_typing=true;break;case 'F':setstr(&cfg.fields,optarg);break;case O_PASSWORD:cfg.password=true;if(optarg&&*optarg)cfg.password_char=true;break;case 'a':setstr(&cfg.anchor,optarg);break;case O_XMARGIN:cfg.x_margin=parse_int(optarg,0,100000,"x-margin");break;case O_YMARGIN:cfg.y_margin=parse_int(optarg,0,100000,"y-margin");break;case O_SELECT:setstr(&select_string,optarg);break;case O_SELECT_INDEX:cfg.select_index=parse_int(optarg,0,100000000,"select-index");break;case 'l':cfg.lines=parse_int(optarg,0,100000,"lines");break;case O_MINIMAL:cfg.minimal_lines=true;break;case O_HIDE_PROMPT:cfg.hide_prompt=true;break;case 'w':cfg.width=parse_int(optarg,1,100000,"width");break;case O_TABS:cfg.tabs=parse_int(optarg,1,64,"tabs");break;case 'x':cfg.hpad=parse_int(optarg,0,100000,"horizontal-pad");break;case 'y':cfg.vpad=parse_int(optarg,0,100000,"vertical-pad");break;case 'P':cfg.inner_pad=parse_int(optarg,0,10000,"inner-pad");break;case 'b':setcolor(SchemeNorm,1,optarg);setcolor(SchemeInput,1,optarg);break;case 't':setcolor(SchemeNorm,0,optarg);break;case 'p':setstr(&cfg.prompt,optarg);break;case O_PROMPT_COLOR:setcolor(SchemePrompt,0,optarg);break;case O_PLACEHOLDER_COLOR:setcolor(SchemeInput,0,optarg);break;case O_INPUT_COLOR:setcolor(SchemeInput,0,optarg);break;case 'm':setcolor(SchemeMatch,0,optarg);break;case 's':setcolor(SchemeSel,1,optarg);break;case 'S':setcolor(SchemeSel,0,optarg);break;case 'M':setcolor(SchemeSelMatch,0,optarg);break;case O_COUNTER_COLOR:setcolor(SchemeCounter,0,optarg);break;case 'B':cfg.border_width=parse_int(optarg,0,1000,"border-width");break;case 'C':setcolor(SchemeBorder,0,optarg);break;case O_MATCH_MODE:apply_key("main","match-mode",optarg,true);break;case O_NO_SORT:cfg.no_sort=true;break;case O_COUNTER:cfg.counter=true;break;case O_FUZZY_MIN:cfg.fuzzy_min=parse_int(optarg,0,100000,"fuzzy-min-length");break;case O_FUZZY_DISC:cfg.fuzzy_discrepancy=parse_int(optarg,0,100000,"fuzzy-max-length-discrepancy");break;case O_FUZZY_DIST:cfg.fuzzy_distance=parse_int(optarg,0,100000,"fuzzy-max-distance");break;case O_LINE_HEIGHT:cfg.line_height=parse_int(optarg,0,10000,"line-height");break;case O_LETTER_SPACING:cfg.letter_spacing=parse_int(optarg,-100,1000,"letter-spacing");break;case O_LAUNCH_PREFIX:setstr(&cfg.launch_prefix,optarg);break;case 'd':cfg.dmenu=true;break;case O_DMENU0:cfg.dmenu=true;cfg.dmenu0=true;break;case O_INDEX:cfg.index=true;break;case O_NTH:setstr(&match_nth,optarg);break;case O_WITH_NTH:setstr(&with_nth,optarg);break;case O_ACCEPT_NTH:setstr(&accept_nth,optarg);break;case O_DELIM:setstr(&nth_delim,optarg);break;case O_ONLY_MATCH:only_match=true;break;case O_AUTO_SELECT:cfg.auto_select=true;break;case O_MESSAGE:setstr(&message,optarg);break;case O_NO_MOUSE:cfg.no_mouse=true;break;case O_SEARCH:setstr(&search_text,optarg);break;case O_TIMINGS:print_timings=true;break;case 'i':break;case 'T':break;case 'R':break;case O_LOG_LEVEL:cfg.log_level_none=!strcmp(optarg,"none");break;case ':':die("xuzzel: option requires an argument: %s",argv[optind-1]);case '?':die("xuzzel: unknown option: %s",argv[optind-1]);default:break;}
    }if(optind<argc)die("xuzzel: unexpected positional argument: %s",argv[optind]);
}
static void usage(FILE*f){fprintf(f,"usage: xuzzel [OPTIONS]\nX11 application launcher and dmenu compatible with fuzzel 1.15 option names.\n  -d, --dmenu                 read entries from stdin and print selection\n  -f, --font=FONT             fontconfig font\n  -p, --prompt=TEXT           prompt\n  -l, --lines=N               visible result lines\n  -w, --width=N               width in characters\n      --icon-size=PIXELS      icon size; 0 uses row height\n      --config=PATH           configuration file\n      --check-config          validate configuration and exit\n      --match-mode=MODE       exact, fzf, or fuzzy\n      --password[=CHAR]       obscure input\n  -h, --help                  show help\n  -v, --version               show version\nSee xuzzel(1) and xuzzel.ini(5).\n");}
int main(int argc,char **argv)
{
    if(argc==5&&!strcmp(argv[1],"--icon-probe"))return icon_probe(argv[2],argv[3],parse_int(argv[4],1,4096,"icon size"))?0:1;
    setlocale(LC_CTYPE,"");defaults();parse_cli(argc,argv,false);load_config();parse_cli(argc,argv,true);if(check_config)return 0;
    if(search_text){snprintf(text,sizeof text,"%s",search_text);cursor=strlen(text);}if(cfg.dmenu)read_stdin_items();else read_apps();read_history();match_items();
    if(cfg.auto_select&&match_count==1)accept(false);
    if (!(dpy = XOpenDisplay(NULL)))
        die("xuzzel: cannot open display");
    icon_init(cfg.icon_theme);
    setup();
    drawmenu();
    run();
    return 1;
}
