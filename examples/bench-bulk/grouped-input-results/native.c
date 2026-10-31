#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "bindings.h"
#ifdef PREVIOUS
 typedef bool receipt_t;
#else
 typedef size_t receipt_t;
#endif
static volatile int64_t sink;
static uint64_t nanos(void) {
    struct timespec t; assert(clock_gettime(CLOCK_MONOTONIC, &t)==0);
    return (uint64_t)t.tv_sec*1000000000ULL+(uint64_t)t.tv_nsec;
}
static uint64_t sample(int kind, size_t n, size_t iterations) {
    int64_t expected=(int64_t)(n*100+n*(n ? n-1 : 0)/2);
    perftest_collection_input_custom_p0_w1_row *values=n ? malloc(n*sizeof(*values)) : NULL;
    for(size_t i=0;i<n;i++) values[i].v0=(int64_t)i+100;
    perftest_collection_try_construct_all_p0_w1_row *owners=NULL;
    if(kind==2 && n) {
        owners=malloc(n*iterations*sizeof(*owners)); assert(owners);
        for(size_t j=0;j<iterations;j++) for(size_t i=0;i<n;i++) owners[j*n+i].v0=perftest_collection_resource_new((int64_t)i+100);
    }
    uint64_t start=nanos(); int64_t total=0;
    for(size_t j=0;j<iterations;j++) {
        int64_t out;
        if(kind==0) out=perftest_collection_input_plain((perftest_collection_input_plain_p0_w1_rows){(void*)values,n},7)-7;
        else if(kind==1) out=perftest_collection_input_custom((perftest_collection_input_custom_p0_w1_rows){values,n},7)-7;
        else {
            receipt_t receipt=0; char *error=NULL; out=-1;
            assert(perftest_collection_try_construct_all((perftest_collection_try_construct_all_p0_w1_rows){owners ? owners+j*n : NULL,n},&out,&receipt,&error));
#ifdef PREVIOUS
            assert(receipt);
#else
            assert(receipt==n);
#endif
            assert(!error);
        }
        assert(out==expected); total+=out;
    }
    uint64_t elapsed=nanos()-start; sink=total;
    free(values); free(owners); return elapsed;
}
int main(int argc,char **argv) {
    assert(argc==4); int kind=atoi(argv[1]); size_t n=strtoull(argv[2],NULL,10); int trial=atoi(argv[3]);
    size_t max=kind==2 && n ? 1048576/n : 5000000;
    size_t iterations=10000/(n+1); if(!iterations) iterations=1; if(iterations>max) iterations=max;
    for(int k=0;k<3;k++) {
        uint64_t time=sample(kind,n,iterations);
        double scale=20000000.0/(double)time; if(scale>8) scale=8; if(scale<0.25) scale=0.25;
        iterations=(size_t)((double)iterations*scale); if(!iterations) iterations=1; if(iterations>max) iterations=max;
    }
    for(int k=0;k<3;k++) (void)sample(kind,n,iterations);
    for(int batch=0;batch<9;batch++) {
        uint64_t elapsed=sample(kind,n,iterations);
        printf("NATIVE,%d,%zu,%d,%d,%zu,%" PRIu64 "\n",kind,n,trial,batch,iterations,elapsed);
    }
    return 0;
}
