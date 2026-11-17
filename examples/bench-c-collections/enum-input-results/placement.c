#define _GNU_SOURCE
#ifdef NDEBUG
#error This diagnostic requires validation assertions
#endif
#include "bindings.h"
#include "placement.h"
#include <assert.h>
#include <dlfcn.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>
#include <limits.h>
typedef uint64_t (*take_fn)(bench_take_numbers_p0_w1_rows);
static volatile uint64_t sink;
static uint64_t now(clockid_t c) {struct timespec t;assert(clock_gettime(c,&t)==0);return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;}
#ifdef PROBE_MALLOC
extern void *__libc_malloc(size_t);
static _Thread_local int probing;
static _Thread_local size_t probe_size,probe_count;
static _Thread_local uintptr_t probe_pointer;
void *malloc(size_t n) {void *p=__libc_malloc(n);if(probing && n==probe_size){probe_pointer=(uintptr_t)p;++probe_count;}return p;}
#endif
struct score {uint64_t wall,cpu,sum;};
/* Diagnostic only: validated x86-64 benchmark libraries, never change their files.
   Copy the identical 152-byte native sum function; repair its one RIP-relative
   deallocation call. Point each library's number-input GOT entry at an executable clone.
   placement.py verifies the exact native function bytes and RELRO slots before
   compiling this diagnostic; unsupported function shapes are rejected. */
static void control_sum(take_fn *f) {
    const char *requested=getenv("SUM_ALIGN");if(!requested)return;
    size_t align=strtoull(requested,NULL,10);assert(align<64 && align%16==0);
    Dl_info info[2];for(int v=0;v<2;++v)assert(dladdr((void *)f[v],&info[v]));
    void **slot[2]={(void **)((char *)info[0].dli_fbase+BEFORE_GOT),(void **)((char *)info[1].dli_fbase+AFTER_GOT)};
    uint8_t *original=*slot[0];assert(original==(uint8_t *)info[0].dli_fbase+BEFORE_NATIVE);
    assert(*slot[1]==(uint8_t *)info[1].dli_fbase+AFTER_NATIVE);
    assert(original[0x8d]==0xff && original[0x8e]==0x15);
    size_t pagesize=(size_t)sysconf(_SC_PAGESIZE);assert(pagesize==4096);
    const char *other=getenv("SUM_ALIGN_OTHER");uint8_t *first=NULL;
    for(int v=0;v<2;++v) {
        if(v==1 && other){align=strtoull(other,NULL,10);assert(align<64 && align%16==0);}
        uint8_t *copy=first;
        if(v==0 || other) {
            uint8_t *page=mmap(NULL,pagesize,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);assert(page!=MAP_FAILED);
            copy=page+align;memcpy(copy,original,0x98);
            int32_t displacement;memcpy(&displacement,original+0x8f,4);
            intptr_t target=(intptr_t)(original+0x93)+displacement;
            intptr_t changed=target-(intptr_t)(copy+0x93);assert(changed>=INT32_MIN && changed<=INT32_MAX);
            displacement=(int32_t)changed;memcpy(copy+0x8f,&displacement,4);
            __builtin___clear_cache((char *)page,(char *)page+pagesize);
            assert(mprotect(page,pagesize,PROT_READ|PROT_EXEC)==0);
            if(v==0)first=copy;
        }
        void *p=(void *)((uintptr_t)slot[v]&~(pagesize-1));assert(mprotect(p,pagesize,PROT_READ|PROT_WRITE)==0);*slot[v]=copy;assert(mprotect(p,pagesize,PROT_READ)==0);
        printf("SUM_CLONE,%d,%zu,%p\n",v,align,(void *)copy);
    }
}
static struct score run(take_fn f,bench_take_numbers_p0_w1_row *rows,size_t n,size_t slots,size_t iterations) {
    uint64_t cpu=now(CLOCK_THREAD_CPUTIME_ID),wall=now(CLOCK_MONOTONIC),sum=0;
    for(size_t j=0;j<iterations;++j)sum+=f((bench_take_numbers_p0_w1_rows){rows+(j%slots)*n,n});
    wall=now(CLOCK_MONOTONIC)-wall;cpu=now(CLOCK_THREAD_CPUTIME_ID)-cpu;sink=sum;
    return (struct score){wall,cpu,sum};
}
int main(int argc,char **argv) {
    assert(argc==7);
    int reverse=atoi(argv[3]),batches=atoi(argv[4]);size_t elements=strtoull(argv[5],NULL,10),selected=strtoull(argv[6],NULL,10);
    void *libs[2];take_fn f[2];
    for(int i=0;i<2;++i){int v=reverse ? 1-i:i;libs[v]=dlopen(argv[v+1],RTLD_NOW|RTLD_LOCAL);if(!libs[v]){fprintf(stderr,"%s\n",dlerror());return 1;}f[v]=(take_fn)dlsym(libs[v],"bench_take_numbers");assert(f[v]);Dl_info info;assert(dladdr((void *)f[v],&info));printf("LIB,%d,%s,%p\n",v,info.dli_fname,(void *)f[v]);}
    control_sum(f);
    const size_t sizes[]={64,512,4096},offsets[]={0,16,32,48,64,128,256,512,1024,2048};
    for(size_t s=0;s<3;++s)for(size_t off=0;off<10;++off)for(int stream=0;stream<2;++stream) {
        size_t n=sizes[s];if(selected && selected!=n)continue;
        const char *selected_offset=getenv("SOURCE_OFFSET");if(selected_offset && strtoull(selected_offset,NULL,10)!=offsets[off])continue;
        const char *selected_stream=getenv("SOURCE_STREAM");if(selected_stream && atoi(selected_stream)!=stream)continue;
        size_t slots=stream ? 262144/n:1,iterations=elements/n;assert(iterations>0);
        void *allocation=NULL;assert(posix_memalign(&allocation,4096,slots*n*sizeof(bench_take_numbers_p0_w1_row)+4096)==0);
        bench_take_numbers_p0_w1_row *rows=(void *)((uint8_t *)allocation+offsets[off]);
        for(size_t i=0;i<slots;++i)for(size_t j=0;j<n;++j)rows[i*n+j].v0=j;
        uint64_t expected=n*(n-1)/2;
        for(int v=0;v<2;++v)for(int j=0;j<3;++j)assert(run(f[v],rows,n,slots,iterations/4+1).sum==expected*(iterations/4+1));
#ifdef PROBE_MALLOC
        for(int v=0;v<2;++v){probe_size=n*sizeof(uint64_t);probe_count=0;probing=1;uint64_t result=f[v]((bench_take_numbers_p0_w1_rows){rows,n});probing=0;assert(result==expected && probe_count==1);printf("PROBE,%d,%zu,%zu,%zu,%p,0x%" PRIxPTR ",%zu\n",v,n,offsets[off],slots,(void *)rows,probe_pointer,probe_count);}
#endif
        for(int b=0;b<batches;++b){struct score score[2];for(int i=0;i<2;++i){int v=(b+reverse)%2 ? 1-i:i;score[v]=run(f[v],rows,n,slots,iterations);assert(score[v].sum==expected*iterations);}printf("PAIR,%zu,%zu,%zu,%d,%zu,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 "\n",n,offsets[off],slots,b,iterations,score[0].wall,score[1].wall,score[0].cpu,score[1].cpu);}
        free(allocation);
    }
    return 0;
}
