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
#define ELEMENT(rows,i) ((rows).data[i])
#define FREE_RESULT(rows) ((void)(rows))
#define FREE_CALLBACK(rows) ((void)(rows))
#else
#define ELEMENT(rows,i) ((rows).data[i].v0)
#define FREE_RESULT(rows) bench_borrowed_values_ret_w0_rows_free(rows)
#define FREE_CALLBACK(rows) CALLBACK_FREE(rows)
#endif
struct visit {size_t n,calls; uint64_t value;};
static void callback(void *context, CALLBACK_CARRIER rows){
    struct visit *v=context;assert(rows.len==v->n);
    if(rows.len){assert(ELEMENT(rows,0)==7);assert(ELEMENT(rows,rows.len-1)==(int64_t)rows.len+6);v->value+=(uint64_t)ELEMENT(rows,rows.len-1);}
    v->calls++; FREE_CALLBACK(rows);
}
static uint64_t invoke(int kind,bench_borrowed_numbers *owner,size_t n){
    if(kind==3) return bench_control(n);
    if(kind==2){struct visit v={.n=n};bench_borrowed_emit(owner,(CALLBACK_TYPE){.run=callback,.context=&v},32);assert(v.calls==32);return v.value;}
    RETURN_CARRIER rows=bench_borrowed_values(owner);assert(rows.len==n);
    uint64_t value=0;
    if(kind==1){for(size_t i=0;i<n;i++)value+=(uint64_t)ELEMENT(rows,i);assert(value==(uint64_t)n*(n+13)/2);}
    else if(n){assert(ELEMENT(rows,0)==7);value=(uint64_t)ELEMENT(rows,n-1);assert(value==n+6);}
    FREE_RESULT(rows);return value;
}
static uint64_t sample(int kind,bench_borrowed_numbers *owner,size_t n,size_t iterations,bool count,uint64_t counts[3]){
    if(count){bench_counter_start();}
    uint64_t start=now(),value=0;
    for(size_t i=0;i<iterations;i++)value+=invoke(kind,owner,n);
    uint64_t elapsed=now()-start;if(count){bench_counter_stop(counts);}
    sink=value;return elapsed;
}
int main(int argc,char **argv){
    assert(argc==7);bool count=!strcmp(argv[1],"count");assert(count||!strcmp(argv[1],"time"));assert(bench_counter_enabled()==count);
    const char *names[]={"view","read","callback","control"};int kind=-1;for(int i=0;i<4;i++)if(!strcmp(argv[2],names[i]))kind=i;assert(kind>=0);
    size_t n=strtoull(argv[3],NULL,10),batches=strtoull(argv[4],NULL,10),target=strtoull(argv[5],NULL,10)*1000000;assert(!strcmp(argv[6],"verify"));
    Dl_info loaded;assert(dladdr((void*)bench_borrowed_values,&loaded));printf("LIBRARY,%s\n",loaded.dli_fname);
    bench_borrowed_numbers *owner=bench_borrowed_numbers_new(n);size_t it=count?1:64;uint64_t counts[3];
    if(!count)for(int i=0;i<3;i++){uint64_t elapsed=sample(kind,owner,n,it,false,counts);double scale=(double)target/(elapsed?elapsed:1);if(scale<.25)scale=.25;if(scale>16)scale=16;it=(size_t)(it*scale);if(!it)it=1;if(it>10000000)it=10000000;}
    if(!count)for(int i=0;i<3;i++)sample(kind,owner,n,it,false,counts);
    for(size_t i=0;i<batches;i++){uint64_t elapsed=sample(kind,owner,n,it,count,counts);if(count)printf("COUNT,%zu,%"PRIu64",%"PRIu64",%"PRIu64"\n",i,counts[0],counts[1],counts[2]);else printf("TIME,%zu,%zu,%"PRIu64"\n",i,it,elapsed);}
    bench_drop_borrowed_numbers(owner);return 0;
}
