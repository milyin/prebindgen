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
static uint64_t now(void){struct timespec t;assert(!clock_gettime(CLOCK_MONOTONIC,&t));return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;}
#if DIRECT_VIEW
typedef int64_t input_element;
#define WRITE_INPUT(p,i) ((p)[i]=(int64_t)(i)+7)
#else
typedef INPUT_ELEMENT input_element;
#define WRITE_INPUT(p,i) ((p)[i].v0=(int64_t)(i)+7)
#endif
struct inputs { input_element *rows; char *text; size_t n; };
static uint64_t invoke(int kind,const struct inputs *in){
    size_t n=in->n;
    if(kind==4)return bench_control(n);
    if(kind==2){assert(bench_shared_input_text((bench_buffer){(const uint8_t*)in->text,n})==n);return n;}
    if(kind==3){assert(bench_shared_input_owned((bench_buffer){(const uint8_t*)in->text,n})==n);return n;}
    if(kind==5){assert(n==4);int64_t value=bench_shared_input_array((ARRAY_CARRIER){(const void*)in->rows,n});assert(value==34);return (uint64_t)value;}
    if(kind==1){int64_t value=bench_shared_input_sum((SUM_CARRIER){(const void*)in->rows,n});assert(value==(int64_t)(n*(n+13)/2));return (uint64_t)value;}
    uintptr_t address=bench_shared_input_inspect((INPUT_CARRIER){in->rows,n});
    if(n){assert((address==(uintptr_t)in->rows)==DIRECT_VIEW);}return address;
}
static uint64_t sample(int kind,const struct inputs *in,size_t it,bool count,uint64_t counts[3]){
    if(count){bench_counter_start();}
    uint64_t start=now(),value=0;for(size_t i=0;i<it;i++)value+=invoke(kind,in);
    uint64_t elapsed=now()-start;if(count){bench_counter_stop(counts);}sink=value;return elapsed;
}
int main(int argc,char **argv){
    assert(argc==7);bool count=!strcmp(argv[1],"count");assert(count||!strcmp(argv[1],"time"));assert(bench_counter_enabled()==count);
    const char *names[]={"inspect","sum","text","owned","control","array"};int kind=-1;for(int i=0;i<6;i++)if(!strcmp(argv[2],names[i]))kind=i;assert(kind>=0);
    size_t n=strtoull(argv[3],NULL,10),batches=strtoull(argv[4],NULL,10),target=strtoull(argv[5],NULL,10)*1000000;assert(!strcmp(argv[6],"verify"));
    Dl_info loaded;assert(dladdr((void*)bench_shared_input_inspect,&loaded));printf("LIBRARY,%s\n",loaded.dli_fname);
    struct inputs in={.n=n};in.rows=calloc(n?n:1,sizeof(*in.rows));in.text=malloc(n?n:1);assert(in.rows&&in.text);for(size_t i=0;i<n;i++){WRITE_INPUT(in.rows,i);in.text[i]='a';}
    size_t it=count?1:64;uint64_t counts[3];
    if(!count)for(int i=0;i<3;i++){uint64_t elapsed=sample(kind,&in,it,false,counts);double scale=(double)target/(elapsed?elapsed:1);if(scale<.25)scale=.25;if(scale>16)scale=16;it=(size_t)(it*scale);if(!it)it=1;if(it>10000000)it=10000000;}
    if(!count)for(int i=0;i<3;i++)sample(kind,&in,it,false,counts);
    for(size_t i=0;i<batches;i++){uint64_t elapsed=sample(kind,&in,it,count,counts);if(count)printf("COUNT,%zu,%"PRIu64",%"PRIu64",%"PRIu64"\n",i,counts[0],counts[1],counts[2]);else printf("TIME,%zu,%zu,%"PRIu64"\n",i,it,elapsed);}
    free(in.rows);free(in.text);return 0;
}
