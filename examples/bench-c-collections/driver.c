#define _GNU_SOURCE
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

static uint64_t handles(size_t n) {
    __auto_type rows = bench_make_handles(n);
    assert(rows.len == n && (n != 0 || rows.data == NULL));
    uint64_t sum = 0;
    for (size_t i=0; i<n; ++i) {
        sum += bench_resource_value(rows.data[i].v0);
        bench_drop_resource(rows.data[i].v0);
    }
    bench_make_handles_ret_w0_rows_free(rows);
    return sum;
}
static uint64_t records(size_t n) {
    __auto_type rows = bench_make_records(n);
    assert(rows.len == n && (n != 0 || rows.data == NULL));
    uint64_t sum = 0;
    for (size_t i=0; i<n; ++i) {
        sum += bench_resource_value(rows.data[i].v0) + rows.data[i].v1 + bench_resource_value(rows.data[i].v2);
        bench_drop_resource(rows.data[i].v0);
        bench_drop_resource(rows.data[i].v2);
    }
    bench_make_records_ret_w0_rows_free(rows);
    return sum;
}
#define NESTED_BODY(name, outer_len) \
    __auto_type rows = bench_##name(n); \
    assert(rows.len == (outer_len)); \
    uint64_t sum = 0; \
    for (size_t i=0; i<rows.len; ++i) { \
        __auto_type child = rows.data[i].v0; \
        assert(child.len != 0 || child.data == NULL); \
        for (size_t j=0; j<child.len; ++j) { \
            sum += bench_resource_value(child.data[j].v0); \
            bench_drop_resource(child.data[j].v0); \
        } \
        bench_##name##_ret_w4_rows_free(child); \
    } \
    bench_##name##_ret_w0_rows_free(rows); \
    return sum;
static uint64_t nested(size_t n) { NESTED_BODY(make_nested, 3) }
static uint64_t many(size_t n) { NESTED_BODY(make_many, n) }
static uint64_t numbers(size_t n) {
    __auto_type rows = bench_make_numbers(n);
    assert(rows.len == n && (n != 0 || rows.data == NULL));
    uint64_t sum = 0;
    for (size_t i=0; i<n; ++i) sum += rows.data[i].v0;
    bench_make_numbers_ret_w0_rows_free(rows);
    return sum;
}
static uint64_t control(size_t n) { return bench_control(n); }
static uint64_t now(clockid_t clock) {
    struct timespec t; assert(clock_gettime(clock, &t) == 0);
    return (uint64_t)t.tv_sec * 1000000000 + (uint64_t)t.tv_nsec;
}
typedef uint64_t (*operation)(size_t);
struct sample { uint64_t wall, cpu, sum; };
static struct sample sample(operation op, size_t n, uint64_t iterations) {
    uint64_t sum=0, cpu=now(CLOCK_THREAD_CPUTIME_ID), wall=now(CLOCK_MONOTONIC);
    for (uint64_t i=0; i<iterations; ++i) sum += op(n);
    wall=now(CLOCK_MONOTONIC)-wall; cpu=now(CLOCK_THREAD_CPUTIME_ID)-cpu;
    return (struct sample){wall,cpu,sum};
}
int main(int argc, char **argv) {
    assert(argc == 6);
    bool counts = strcmp(argv[1], "count") == 0;
    assert(counts || strcmp(argv[1], "time") == 0);
    assert(bench_counter_enabled() == counts);
    const char *name=argv[2]; size_t n=strtoull(argv[3],NULL,10);
    int batches=atoi(argv[4]); uint64_t target=(uint64_t)atoi(argv[5])*1000000;
    const char *names[]={"handles","records","nested","many","numbers","control"};
    operation ops[]={handles,records,nested,many,numbers,control}, op=NULL;
    for (size_t i=0;i<sizeof(ops)/sizeof(*ops);++i) if(strcmp(name,names[i])==0) op=ops[i];
    assert(op != NULL && batches > 0 && target > 0);
    uint64_t expected=(uint64_t)n*(n ? n-1 : 0)/2;
    if(op==records)expected=expected*3+n;
    if(op==control)expected=n;
    Dl_info library; assert(dladdr((void *)bench_control,&library));
    printf("LIBRARY,%s\n",library.dli_fname);
    if(counts) {
        for(int i=0;i<100;++i)assert(op(n)==expected);
        for(int i=0;i<batches;++i) {
            uint64_t values[3]; bench_counter_start(); uint64_t sum=op(n); bench_counter_stop(values);
            assert(sum==expected);
            printf("COUNT,%d,%" PRIu64 ",%" PRIu64 ",%" PRIu64 "\n",i,values[0],values[1],values[2]);
        }
    } else {
        uint64_t iterations=1; struct sample measured;
        do { measured=sample(op,n,iterations); assert(measured.sum==expected*iterations);
             if(measured.wall<target)iterations*=2;
        } while(measured.wall<target);
        uint64_t warmup=now(CLOCK_MONOTONIC);
        do { measured=sample(op,n,iterations); assert(measured.sum==expected*iterations);
        } while(now(CLOCK_MONOTONIC)-warmup<200000000);
        for(int i=0;i<batches;++i) {
            measured=sample(op,n,iterations); assert(measured.sum==expected*iterations);
            printf("TIME,%d,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 "\n",
                   i,iterations,measured.wall,measured.cpu,measured.sum);
        }
    }
    return 0;
}
