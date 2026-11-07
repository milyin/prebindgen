#define _GNU_SOURCE
#ifdef NDEBUG
#undef NDEBUG
#endif
#include "bindings.h"
#include <assert.h>
#include <dlfcn.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
extern bool bench_counter_enabled(void);
extern void bench_counter_start(void);
extern void bench_counter_stop(uint64_t *out);
static volatile uint64_t sink;
static uint64_t now(void) {struct timespec t;assert(!clock_gettime(CLOCK_MONOTONIC,&t));return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;}
static int address_order(const void *a,const void *b) {
    uintptr_t x=(uintptr_t)*(bench_resource *const *)a,y=(uintptr_t)*(bench_resource *const *)b;
    return (x>y)-(x<y);
}
static void reorder(bench_resource **values,size_t n,const char *order) {
    qsort(values,n,sizeof(*values),address_order);
    if(!strcmp(order,"descending"))for(size_t i=0;i<n/2;++i){bench_resource *t=values[i];values[i]=values[n-1-i];values[n-1-i]=t;}
    if(!strcmp(order,"shuffled")){
        uint64_t state=20261107;
        for(size_t i=n;i>1;--i){state^=state<<13;state^=state>>7;state^=state<<17;size_t j=state%i;bench_resource *t=values[i-1];values[i-1]=values[j];values[j]=t;}
    }
}
struct fixture {void *leaf,*rows;size_t n,groups,iterations;bench_resource **handles,*token;int kind;};
typedef __typeof__(*(((bench_history_callback_callback_return_row *)0)->v0.data)) callback_leaf;
#define FILL(LEAF,ROW) do { \
    LEAF *leaf=calloc((n? n:1)*iterations,sizeof(*leaf)); \
    ROW *rows=calloc((groups?groups:1)*iterations,sizeof(*rows));assert(leaf&&rows); \
    for(size_t j=0;j<iterations;++j){ \
        for(size_t i=0;i<n;++i)leaf[j*n+i]=(LEAF){handles[j*n+i]}; \
        for(size_t i=0;i<groups;++i){size_t start=i*chunk,len=n-start;if(len>chunk)len=chunk;rows[j*groups+i]=(ROW){{leaf+j*n+start,len}};} \
    } \
    result.leaf=leaf;result.rows=rows; \
}while(0)
static struct fixture prepare(int kind,size_t n,size_t iterations,const char *order) {
    bool callback=kind>=5;
    if(!callback)iterations=1;
    bench_resource **handles=calloc((n?n:1)*iterations,sizeof(*handles));assert(handles);
    for(size_t j=0;j<iterations;++j){for(size_t i=0;i<n;++i)handles[j*n+i]=bench_resource_new(i);reorder(handles+j*n,n,order);}
    size_t chunk=(kind==2 ? 8 : (kind==6 ? (n?n:1) : 1)),groups=(n+chunk-1)/chunk;
    struct fixture result={.n=n,.groups=groups,.iterations=iterations,.handles=handles,.kind=kind};
    switch(kind){
    case 0: {bench_history_flat_p0_w1_row *rows=calloc(n?n:1,sizeof(*rows));assert(rows);for(size_t i=0;i<n;++i)rows[i]=(bench_history_flat_p0_w1_row){handles[i]};result.leaf=rows;break;}
    case 1:case 2:FILL(bench_history_nested_p0_w5_row,bench_history_nested_p0_w1_row);break;
    case 3:FILL(bench_history_shared_p0_w5_row,bench_history_shared_p0_w1_row);result.token=bench_resource_new(7);break;
    case 4:FILL(bench_history_readonly_p0_w5_row,bench_history_readonly_p0_w1_row);break;
    default:FILL(callback_leaf,bench_history_callback_callback_return_row);break;
    }
    return result;
}
struct callback_state {struct fixture *input;size_t next;};
static bench_history_callback_callback_return callback_run(void *context) {
    struct callback_state *s=context;struct fixture *f=s->input;
    return (bench_history_callback_callback_return){(bench_history_callback_callback_return_row *)f->rows+s->next++*f->groups,f->groups};
}
static void callback_release(void *context,const bench_history_callback_callback_return *rows) {(void)context;(void)rows;}
static uint64_t invoke(struct fixture *f,size_t iterations) {
    uint64_t sum=0;struct callback_state state={f,0};
    for(size_t i=0;i<iterations;++i)switch(f->kind){
    case 0:sum+=bench_history_flat((bench_history_flat_p0_w1_rows){f->leaf,f->n});break;
    case 1:case 2:sum+=bench_history_nested((bench_history_nested_p0_w1_rows){f->rows,f->groups});break;
    case 3:sum+=bench_history_shared((bench_history_shared_p0_w1_rows){f->rows,f->groups},f->token);break;
    case 4:sum+=bench_history_readonly((bench_history_readonly_p0_w1_rows){f->rows,f->groups});break;
    default:sum+=bench_history_callback((bench_history_callback_callback){.run=callback_run,.context=&state,.release_result=callback_release});break;
    }
    assert(sum==iterations*(f->n*(f->n-1)/2+(f->kind==3?7:0)));sink=sum;return sum;
}
static void release(struct fixture *f) {
    if(f->kind<5)for(size_t i=0;i<f->n;++i)bench_drop_resource(f->handles[i]);
    if(f->token)bench_drop_resource(f->token);
    free(f->handles);free(f->rows);free(f->leaf);
}
static uint64_t sample(struct fixture *borrowed,int kind,size_t n,size_t iterations,const char *order,bool counts,uint64_t *allocation) {
    struct fixture owned;
    struct fixture *f=borrowed;
    if(kind>=5){owned=prepare(kind,n,iterations,order);f=&owned;}
    if(counts)bench_counter_start();
    uint64_t begin=now();invoke(f,iterations);uint64_t elapsed=now()-begin;
    if(counts)bench_counter_stop(allocation);
    if(kind>=5)release(f);
    return elapsed;
}
int main(int argc,char **argv) {
    // Checked-only child-process probes deliberately fail after a large prefix.
    // Never run malformed frames against an unchecked build.
    if(argc==3 && !strcmp(argv[1],"reject")) {
        int kind=!strcmp(argv[2],"flat")?0:!strcmp(argv[2],"nested")?1:!strcmp(argv[2],"groups8")?2:!strcmp(argv[2],"shared")?3:!strcmp(argv[2],"callback")?5:6;
        struct fixture f=prepare(kind,256,1,"ascending");
        size_t at=200;
        switch(kind){
        case 0:((bench_history_flat_p0_w1_row *)f.leaf)[at].v0=f.handles[0];break;
        case 1:case 2:((bench_history_nested_p0_w5_row *)f.leaf)[at].v0=f.handles[0];break;
        case 3:((bench_history_shared_p0_w5_row *)f.leaf)[at].v0=f.token;break;
        default:((callback_leaf *)f.leaf)[at].v0=f.handles[0];break;
        }
        invoke(&f,1); // A checked binding must abort with its alias diagnostic.
        return 90;
    }
    assert(argc==8);
    bool counts=!strcmp(argv[1],"count");assert(counts||!strcmp(argv[1],"time"));assert(bench_counter_enabled()==counts);
    const char *names[]={"flat","nested","groups8","shared","readonly","callback","callback_single"};int kind=-1;
    for(int i=0;i<7;++i)if(!strcmp(argv[2],names[i]))kind=i;
    assert(kind>=0);size_t n=strtoull(argv[3],NULL,10);const char *order=argv[4];size_t batches=strtoull(argv[5],NULL,10),target=strtoull(argv[6],NULL,10)*1000000;
    assert(!strcmp(argv[7],"verify"));
    Dl_info loaded;assert(dladdr((void *)bench_history_nested,&loaded));printf("LIBRARY,%s\n",loaded.dli_fname);
    struct fixture borrowed={0};if(kind<5)borrowed=prepare(kind,n,1,order);
    size_t limit=kind>=5?65536/(n+1):1000000;if(!limit)limit=1;
    size_t iterations=counts?1:16384/(n+1);if(!iterations)iterations=1;if(iterations>limit)iterations=limit;
    uint64_t values[3];
    if(!counts)for(int i=0;i<3;++i){uint64_t elapsed=sample(&borrowed,kind,n,iterations,order,false,values);double scale=(double)target/(elapsed?elapsed:1);if(scale<.25)scale=.25;if(scale>8)scale=8;iterations=(size_t)(iterations*scale);if(!iterations)iterations=1;if(iterations>limit)iterations=limit;}
    for(int i=0;i<3;++i)sample(&borrowed,kind,n,iterations,order,false,values);
    for(size_t batch=0;batch<batches;++batch){uint64_t elapsed=sample(&borrowed,kind,n,iterations,order,counts,values);if(counts)printf("COUNT,%zu,%"PRIu64",%"PRIu64",%"PRIu64"\n",batch,values[0],values[1],values[2]);else printf("TIME,%zu,%zu,%"PRIu64"\n",batch,iterations,elapsed);}
    if(kind<5)release(&borrowed);
    return 0;
}
