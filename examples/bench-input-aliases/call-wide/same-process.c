// Reuse the exact same C rows and native handles for every borrowed-input candidate.
#define main history_driver_main
#include "../../bench-c-collections/history-driver.c"
#undef main
struct candidate {void *library;void *function;const char *path;};
static uint64_t call(struct fixture *f,struct candidate *c,size_t iterations) {
    uint64_t sum=0;
    for(size_t i=0;i<iterations;++i)switch(f->kind) {
        case 0:sum+=((__typeof__(&bench_history_flat))c->function)((bench_history_flat_p0_w1_rows){f->leaf,f->n});break;
        case 1:case 2:sum+=((__typeof__(&bench_history_nested))c->function)((bench_history_nested_p0_w1_rows){f->rows,f->groups});break;
        case 3:sum+=((__typeof__(&bench_history_shared))c->function)((bench_history_shared_p0_w1_rows){f->rows,f->groups},f->token);break;
        case 4:sum+=((__typeof__(&bench_history_readonly))c->function)((bench_history_readonly_p0_w1_rows){f->rows,f->groups});break;
        default:abort();
    }
    assert(sum==iterations*(f->n*(f->n-1)/2+(f->kind==3?7:0)));sink=sum;return sum;
}
int main(int argc,char **argv) {
    assert(argc==12); // kind,size,order,batches,target-ms,fork, five libraries
    int kind=atoi(argv[1]);assert(kind>=0&&kind<=4);
    size_t n=strtoull(argv[2],NULL,10),batches=strtoull(argv[4],NULL,10),target=strtoull(argv[5],NULL,10)*1000000;
    unsigned fork=(unsigned)strtoul(argv[6],NULL,10);
    const char *symbols[]={"bench_history_flat","bench_history_nested","bench_history_nested","bench_history_shared","bench_history_readonly"};
    struct candidate c[5];
    for(size_t i=0;i<5;++i) {
        c[i].path=argv[7+i];c[i].library=dlopen(c[i].path,RTLD_NOW|RTLD_LOCAL);assert(c[i].library);
        c[i].function=dlsym(c[i].library,symbols[kind]);assert(c[i].function);
        bool (*counter)(void)=dlsym(c[i].library,"bench_counter_enabled");assert(counter&&!counter());
        Dl_info loaded;assert(dladdr(c[i].function,&loaded));assert(!strcmp(loaded.dli_fname,c[i].path));
        printf("LIBRARY,%zu,%s,%p\n",i,loaded.dli_fname,c[i].function);
    }
    struct fixture f=prepare(kind,n,1,argv[3]);
    size_t iterations=16384/(n+1);if(!iterations)iterations=1;
    for(int round=0;round<3;++round) {
        uint64_t begin=now();call(&f,&c[0],iterations);uint64_t elapsed=now()-begin;
        double scale=(double)target/(elapsed?elapsed:1);if(scale<.25)scale=.25;if(scale>8)scale=8;
        iterations=(size_t)(iterations*scale);if(!iterations)iterations=1;if(iterations>1000000)iterations=1000000;
    }
    // The same iteration count, fixtures and addresses are used by all variants.
    for(size_t i=0;i<5;++i)for(int warm=0;warm<3;++warm)call(&f,&c[i],iterations);
    for(size_t batch=0;batch<batches;++batch)for(size_t step=0;step<5;++step) {
        size_t i=(step+batch+fork)%5;if((batch+fork)&1)i=4-i;
        uint64_t begin=now();call(&f,&c[i],iterations);uint64_t elapsed=now()-begin;
        printf("TIME,%zu,%zu,%zu,%"PRIu64"\n",i,batch,iterations,elapsed);
    }
    release(&f);for(size_t i=0;i<5;++i)assert(!dlclose(c[i].library));return 0;
}
