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
static uint64_t now(clockid_t clock) { struct timespec t; assert(clock_gettime(clock,&t)==0); return (uint64_t)t.tv_sec*1000000000+(uint64_t)t.tv_nsec; }
static volatile uint64_t sink;
static uint8_t large_text[4096], large_bytes[4096];
struct fixed_pair {bench_resource *first, *second;};
struct batch {void *rows; size_t n, iterations; int kind;};
static struct batch prepare(int kind,size_t n,size_t iterations) {
    if(kind==5)return (struct batch){NULL,n,iterations,kind};
    if(kind>=11 && kind<=13) {
        size_t count=kind==12 ? 1 : iterations;
        struct fixed_pair *rows=malloc(sizeof(*rows)*count);assert(rows);
        for(size_t j=0;j<count;++j)rows[j]=(struct fixed_pair){bench_resource_new(0),bench_resource_new(1)};
        return (struct batch){rows,1,iterations,kind};
    }
    size_t stride=sizeof(bench_take_handles_p0_w1_row);
    if(kind==1)stride=sizeof(bench_take_records_p0_w1_row);
    if(kind==2)stride=sizeof(bench_take_wide_p0_w1_row);
    if(kind==3 || kind==7)stride=sizeof(bench_take_text_p0_w1_row);
    if(kind==4)stride=sizeof(bench_take_numbers_p0_w1_row);
    if(kind==6 || kind==8)stride=sizeof(bench_take_bytes_p0_w1_row);
    if(kind==9 || kind==10)stride=sizeof(bench_take_buffers_p0_w1_row);
    if(kind==15)stride=sizeof(bench_take_tagged_p0_w1_row);
    if(kind==16)stride=sizeof(bench_take_tagged_buffers_p0_w1_row);
    void *rows=n ? malloc(stride*n*iterations) : NULL; assert(!n || rows);
    for(size_t j=0;j<iterations;++j)for(size_t i=0;i<n;++i) {
        size_t k=j*n+i;
        switch(kind) {
        case 14:
        case 0: ((bench_take_handles_p0_w1_row *)rows)[k]=(bench_take_handles_p0_w1_row){bench_resource_new(i)}; break;
        case 1: ((bench_take_records_p0_w1_row *)rows)[k]=(bench_take_records_p0_w1_row){bench_resource_new(i),i,bench_resource_new(i+1)}; break;
        case 2: ((bench_take_wide_p0_w1_row *)rows)[k]=(bench_take_wide_p0_w1_row){bench_resource_new(i),i,i,i,i,i,i,i}; break;
        case 3: ((bench_take_text_p0_w1_row *)rows)[k]=(bench_take_text_p0_w1_row){bench_resource_new(i),{(const uint8_t *)"abc",3}}; break;
        case 4: ((bench_take_numbers_p0_w1_row *)rows)[k]=(bench_take_numbers_p0_w1_row){i}; break;
        case 6: ((bench_take_bytes_p0_w1_row *)rows)[k]=(bench_take_bytes_p0_w1_row){bench_resource_new(i),{(const uint8_t *)"abc",3}}; break;
        case 7: ((bench_take_text_p0_w1_row *)rows)[k]=(bench_take_text_p0_w1_row){bench_resource_new(i),{large_text,sizeof(large_text)}}; break;
        case 8: ((bench_take_bytes_p0_w1_row *)rows)[k]=(bench_take_bytes_p0_w1_row){bench_resource_new(i),{large_bytes,sizeof(large_bytes)}}; break;
        case 9: ((bench_take_buffers_p0_w1_row *)rows)[k]=(bench_take_buffers_p0_w1_row){bench_resource_new(i),{(const uint8_t *)"abc",3},{(const uint8_t *)"abc",3}}; break;
        case 10: ((bench_take_buffers_p0_w1_row *)rows)[k]=(bench_take_buffers_p0_w1_row){bench_resource_new(i),{large_text,sizeof(large_text)},{large_bytes,sizeof(large_bytes)}}; break;
        case 15: ((bench_take_tagged_p0_w1_row *)rows)[k]=(bench_take_tagged_p0_w1_row){bench_resource_new(i),i%3==0 ? bench_input_tag_Low : i%3==1 ? bench_input_tag_Normal : bench_input_tag_High}; break;
        case 16: ((bench_take_tagged_buffers_p0_w1_row *)rows)[k]=(bench_take_tagged_buffers_p0_w1_row){bench_resource_new(i),i%3==0 ? bench_input_tag_Low : i%3==1 ? bench_input_tag_Normal : bench_input_tag_High,{(const uint8_t *)"abc",3},{(const uint8_t *)"abc",3}}; break;
        default: break;
        }
    }
    return (struct batch){rows,n,iterations,kind};
}
struct callback_input {struct batch batch; size_t next;};
static bench_callback_fixed_callback_return callback_fixed_run(void *context) {
    struct callback_input *input=context;
    struct fixed_pair pair=((struct fixed_pair *)input->batch.rows)[input->next++];
    return (bench_callback_fixed_callback_return){pair.first,0,pair.second};
}
static bench_callback_many_callback_return callback_many_run(void *context) {
    struct callback_input *input=context;
    size_t k=input->next++*input->batch.n;
    return (bench_callback_many_callback_return){input->batch.rows ? (bench_callback_many_callback_return_row *)input->batch.rows+k : NULL,input->batch.n};
}
static void callback_many_release(void *context,const bench_callback_many_callback_return *value) {(void)context;(void)value;}
static uint64_t run(struct batch b) {
    struct callback_input input={b,0};
    uint64_t total=0;
    for(size_t j=0;j<b.iterations;++j) {
        size_t k=j*b.n;
        switch(b.kind) {
        case 11: { struct fixed_pair pair=((struct fixed_pair *)b.rows)[j]; total+=bench_fixed_move(pair.first,pair.second); break; }
        case 12: { struct fixed_pair pair=*(struct fixed_pair *)b.rows; total+=bench_fixed_borrow(pair.first,pair.second); break; }
        case 13: total+=bench_callback_fixed((bench_callback_fixed_callback){.run=callback_fixed_run,.context=&input}); break;
        case 14: total+=bench_callback_many((bench_callback_many_callback){.run=callback_many_run,.context=&input,.release_result=callback_many_release}); break;
        case 0: total+=bench_take_handles((bench_take_handles_p0_w1_rows){b.rows ? (bench_take_handles_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 1: total+=bench_take_records((bench_take_records_p0_w1_rows){b.rows ? (bench_take_records_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 2: total+=bench_take_wide((bench_take_wide_p0_w1_rows){b.rows ? (bench_take_wide_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 3: total+=bench_take_text((bench_take_text_p0_w1_rows){b.rows ? (bench_take_text_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 4: total+=bench_take_numbers((bench_take_numbers_p0_w1_rows){b.rows ? (bench_take_numbers_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 6: case 8: total+=bench_take_bytes((bench_take_bytes_p0_w1_rows){b.rows ? (bench_take_bytes_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 7: total+=bench_take_text((bench_take_text_p0_w1_rows){b.rows ? (bench_take_text_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 9: case 10: total+=bench_take_buffers((bench_take_buffers_p0_w1_rows){b.rows ? (bench_take_buffers_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 15: total+=bench_take_tagged((bench_take_tagged_p0_w1_rows){b.rows ? (bench_take_tagged_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        case 16: total+=bench_take_tagged_buffers((bench_take_tagged_buffers_p0_w1_rows){b.rows ? (bench_take_tagged_buffers_p0_w1_row *)b.rows+k : NULL,b.n}); break;
        default: total+=bench_control(b.n); break;
        }
    }
    sink=total;return total;
}
struct score {uint64_t wall,cpu,sum;};
static struct score measure(int kind,size_t n,size_t iterations,bool counts,uint64_t *allocations) {
    struct batch b=prepare(kind,n,iterations);
    if(counts)bench_counter_start();
    uint64_t cpu=now(CLOCK_THREAD_CPUTIME_ID),wall=now(CLOCK_MONOTONIC);
    uint64_t sum=run(b);
    wall=now(CLOCK_MONOTONIC)-wall;cpu=now(CLOCK_THREAD_CPUTIME_ID)-cpu;
    if(counts)bench_counter_stop(allocations);
    if(kind==12) {struct fixed_pair pair=*(struct fixed_pair *)b.rows;bench_drop_resource(pair.first);bench_drop_resource(pair.second);}
    free(b.rows);return (struct score){wall,cpu,sum};
}
int main(int argc,char **argv) {
    assert(argc==6);bool counts=strcmp(argv[1],"count")==0;assert(counts || strcmp(argv[1],"time")==0);assert(bench_counter_enabled()==counts);
    const char *names[]={"handles","records","wide","text","numbers","control","bytes","text_large","bytes_large","buffers","buffers_large","fixed_move","fixed_borrow","callback_fixed","callback_many","tagged","tagged_buffers"};int kind=-1;
    for(int i=0;i<17;++i)if(strcmp(argv[2],names[i])==0)kind=i;
    // 1365 Euro signs plus one ASCII byte: valid UTF-8 exercises the scan as
    // well as copying. All source payload storage is prepared outside timing.
    for(size_t i=0;i+2<sizeof(large_text);i+=3) {large_text[i]=0xe2;large_text[i+1]=0x82;large_text[i+2]=0xac;}
    large_text[sizeof(large_text)-1]='x';memset(large_bytes,0x5a,sizeof(large_bytes));
    size_t n=strtoull(argv[3],NULL,10);int batches=atoi(argv[4]);uint64_t target=(uint64_t)atoi(argv[5])*1000000;
    assert(kind>=0 && n<=100000 && batches>0 && target>0);
    uint64_t expected=(uint64_t)n*(n ? n-1 : 0)/2;
    if(kind>=11 && kind<=13)expected=1;
    if(kind==1)expected=3*expected+n;
    if(kind==2)expected*=8;
    if(kind==3)expected+=3*n;
    if(kind==5)expected=n;
    if(kind==6)expected+=(3+'a'+'c')*n;
    if(kind==7)expected+=sizeof(large_text)*n;
    if(kind==8)expected+=(sizeof(large_bytes)+2*0x5a)*n;
    if(kind==9)expected+=(3+3+'a'+'c')*n;
    if(kind==10)expected+=(sizeof(large_text)+sizeof(large_bytes)+2*0x5a)*n;
    if(kind==15 || kind==16)for(size_t i=0;i<n;++i)expected+=i%3+1;
    if(kind==16)expected+=(3+3+'a'+'c')*n;
    Dl_info library;assert(dladdr((void *)bench_control,&library));printf("LIBRARY,%s\n",library.dli_fname);
    if(counts) {
        for(int j=0;j<10;++j)assert(measure(kind,n,1,false,NULL).sum==expected);
        for(int j=0;j<batches;++j) {uint64_t a[3];assert(measure(kind,n,1,true,a).sum==expected);printf("COUNT,%d,%" PRIu64 ",%" PRIu64 ",%" PRIu64 "\n",j,a[0],a[1],a[2]);}
    } else {
        size_t iterations=1,max=n ? 262144/n : 1048576; if(!max)max=1;
        struct score value;
        do {value=measure(kind,n,iterations,false,NULL);assert(value.sum==expected*iterations);if(value.wall>=target || iterations>=max)break;iterations=iterations>max/2 ? max : iterations*2;}while(1);
        for(int j=0;j<3;++j)assert(measure(kind,n,iterations,false,NULL).sum==expected*iterations);
        for(int j=0;j<batches;++j) {value=measure(kind,n,iterations,false,NULL);assert(value.sum==expected*iterations);printf("TIME,%d,%zu,%" PRIu64 ",%" PRIu64 ",%" PRIu64 "\n",j,iterations,value.wall,value.cpu,value.sum);}
    }
}
