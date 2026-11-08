#define _GNU_SOURCE
#include "bindings.h"
#include <assert.h>
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>
#include <time.h>
#include <sys/wait.h>
#include <sys/resource.h>
#include <unistd.h>
#include <signal.h>
static volatile uint64_t sink;
static uint64_t now(void){struct timespec t;assert(!clock_gettime(CLOCK_MONOTONIC,&t));return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;}
struct lib {void *dl; __typeof__(&buckets_mut_one) one;__typeof__(&buckets_mixed_mut) mut;__typeof__(&buckets_mixed_shared) shared;};
static uint64_t rng=42;
static size_t next(size_t n){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;return rng%n;}
struct fixture{size_t n,len[4];buckets_a **a;buckets_b **b;buckets_c **c;buckets_d **d;buckets_token *token;uint64_t sum;};
static uint64_t run(struct lib *l,struct fixture*f,int kind,size_t count){uint64_t sum=0;
 for(size_t i=0;i<count;++i) {
  if(kind==0)sum+=l->one((buckets_mut_one_p0_w1_rows){(void*)f->a,f->len[0]});
  if(kind==1)sum+=l->mut((buckets_mixed_mut_p0_w1_rows){(void*)f->a,f->len[0]},(buckets_mixed_mut_p1_w1_rows){(void*)f->b,f->len[1]},(buckets_mixed_mut_p2_w1_rows){(void*)f->c,f->len[2]},(buckets_mixed_mut_p3_w1_rows){(void*)f->d,f->len[3]});
  if(kind==2)sum+=l->shared((buckets_mixed_shared_p0_w1_rows){(void*)f->a,f->len[0]},(buckets_mixed_shared_p1_w1_rows){(void*)f->b,f->len[1]},(buckets_mixed_shared_p2_w1_rows){(void*)f->c,f->len[2]},(buckets_mixed_shared_p3_w1_rows){(void*)f->d,f->len[3]},f->token);
 }
 assert(sum==count*(f->sum+(kind==2?7:0)));sink=sum;return sum;
}
#define LOOKUP(lib,name) ((__typeof__(&name))dlsym(lib,#name))
static void probes(void *dl,int variant){
 buckets_owner *owner=LOOKUP(dl,buckets_owner_new)();const buckets_child *child=LOOKUP(dl,buckets_owner_child)(owner);
 assert((void*)owner==(void*)child);
 bool(*probe)(uint32_t,uintptr_t)=dlsym(dl,"buckets_probe");assert(probe);
 assert(!probe(0,(uintptr_t)owner));assert(probe(1,(uintptr_t)owner));assert(!probe(2,(uintptr_t)owner));
 // Only invoke the checked overlap export, shared unchanged by every prototype.
 pid_t pid=fork();assert(pid>=0);
 if(pid==0){struct rlimit r={0,0};setrlimit(RLIMIT_CORE,&r);assert(freopen("/dev/null","w",stderr));LOOKUP(dl,buckets_overlap)(owner,child);_exit(0);}
 int status;assert(waitpid(pid,&status,0)==pid);assert(WIFSIGNALED(status)&&WTERMSIG(status)==SIGABRT);
 LOOKUP(dl,buckets_drop_owner)(owner);
 buckets_a *a=LOOKUP(dl,buckets_a_new)(1);buckets_mut_one_p0_w1_row repeated[2]={{a},{a}};
 pid=fork();assert(pid>=0);
 if(pid==0){struct rlimit r={0,0};setrlimit(RLIMIT_CORE,&r);assert(freopen("/dev/null","w",stderr));LOOKUP(dl,buckets_mut_one)((buckets_mut_one_p0_w1_rows){repeated,2});_exit(0);}
 assert(waitpid(pid,&status,0)==pid);assert(WIFSIGNALED(status)&&WTERMSIG(status)==SIGABRT);
 buckets_token *token=LOOKUP(dl,buckets_token_new)(7);
 buckets_mixed_shared_p0_w1_row shared[2]={{a},{a}};
 assert(LOOKUP(dl,buckets_mixed_shared)((buckets_mixed_shared_p0_w1_rows){shared,2},(buckets_mixed_shared_p1_w1_rows){0},(buckets_mixed_shared_p2_w1_rows){0},(buckets_mixed_shared_p3_w1_rows){0},token)==9);

 // Check each generated collection group, then cross-type conflicts where supported.
 buckets_b *b=LOOKUP(dl,buckets_b_new)(2);buckets_c *c=LOOKUP(dl,buckets_c_new)(3);buckets_d *d=LOOKUP(dl,buckets_d_new)(4);
 for(int test=0;test<6;++test){
  if(variant==2 && test>=4)continue; // Known-invalid general-C design: local-only type checks.
  pid=fork();assert(pid>=0);
  if(pid==0){
   struct rlimit r={0,0};setrlimit(RLIMIT_CORE,&r);assert(freopen("/dev/null","w",stderr));
   buckets_mixed_mut_p0_w1_row aa[2]={{a},{a}};
   buckets_mixed_mut_p1_w1_row bb[2]={{b},{b}};
   buckets_mixed_mut_p2_w1_row cc[2]={{c},{c}};
   buckets_mixed_mut_p3_w1_row dd[2]={{d},{d}};
   if(test==4)bb[0].v0=(buckets_b*)a;
   if(test==5)LOOKUP(dl,buckets_mixed_shared)((buckets_mixed_shared_p0_w1_rows){shared,2},(buckets_mixed_shared_p1_w1_rows){0},(buckets_mixed_shared_p2_w1_rows){0},(buckets_mixed_shared_p3_w1_rows){0},(buckets_token*)a);
   else LOOKUP(dl,buckets_mixed_mut)((buckets_mixed_mut_p0_w1_rows){aa,test==0?2:1},(buckets_mixed_mut_p1_w1_rows){bb,test==1?2:1},(buckets_mixed_mut_p2_w1_rows){cc,test==2?2:1},(buckets_mixed_mut_p3_w1_rows){dd,test==3?2:1});
   _exit(0);
  }
  assert(waitpid(pid,&status,0)==pid);assert(WIFSIGNALED(status)&&WTERMSIG(status)==SIGABRT);
 }
 LOOKUP(dl,buckets_drop_d)(d);LOOKUP(dl,buckets_drop_c)(c);LOOKUP(dl,buckets_drop_b)(b);
 LOOKUP(dl,buckets_drop_token)(token);LOOKUP(dl,buckets_drop_a)(a);
 printf("PROBES,%d,ok\n",variant);
}
int main(int argc,char**argv){
 assert(argc==11);int kind=atoi(argv[1]);size_t n=strtoull(argv[2],0,10);int shuffled=atoi(argv[3]);int batches=atoi(argv[4]);uint64_t target=strtoull(argv[5],0,10)*1000000;int forkno=atoi(argv[6]);
 struct lib libs[4];
 for(int i=0;i<4;++i){void*dl=dlopen(argv[7+i],RTLD_NOW|RTLD_LOCAL);if(!dl){puts(dlerror());abort();}libs[i]=(struct lib){dl,LOOKUP(dl,buckets_mut_one),LOOKUP(dl,buckets_mixed_mut),LOOKUP(dl,buckets_mixed_shared)};assert(libs[i].one&&libs[i].mut&&libs[i].shared);
  for(int j=0;j<i;++j)assert(libs[j].mut!=libs[i].mut);
  Dl_info info;assert(dladdr(libs[i].mut,&info));assert(!strcmp(info.dli_fname,argv[7+i]));
  printf("LIBRARY,%d,%s,%p\n",i,info.dli_fname,(void*)libs[i].mut);
 }
 if(n==0){for(int i=0;i<4;++i)probes(libs[i].dl,i);return 0;}
 void*dl=libs[0].dl;struct fixture f={.n=n};
 for(int i=0;i<4;++i)f.len[i]=kind==0?(i==0?n:0):n/4+(i<(int)(n%4));
 f.a=calloc(f.len[0]+1,sizeof(*f.a));f.b=calloc(f.len[1]+1,sizeof(*f.b));f.c=calloc(f.len[2]+1,sizeof(*f.c));f.d=calloc(f.len[3]+1,sizeof(*f.d));assert(f.a&&f.b&&f.c&&f.d);
 for(size_t i=0;i<f.len[0];++i){
  f.a[i]=LOOKUP(dl,buckets_a_new)(i+1);f.sum+=i+1;
  if(i<f.len[1]){f.b[i]=LOOKUP(dl,buckets_b_new)(i+1);f.sum+=i+1;}
  if(i<f.len[2]){f.c[i]=LOOKUP(dl,buckets_c_new)(i+1);f.sum+=i+1;}
  if(i<f.len[3]){f.d[i]=LOOKUP(dl,buckets_d_new)(i+1);f.sum+=i+1;}
 }
 f.token=LOOKUP(dl,buckets_token_new)(7);
 if(shuffled){
  #define SHUFFLE(field,idx) for(size_t i=f.len[idx];i>1;--i){size_t j=next(i);__typeof__(*f.field) p=f.field[i-1];f.field[i-1]=f.field[j];f.field[j]=p;}
  SHUFFLE(a,0);SHUFFLE(b,1);SHUFFLE(c,2);SHUFFLE(d,3);
 }
 size_t it=10000/(n+1)+1;
 for(int r=0;r<3;++r){uint64_t t=now();run(libs,&f,kind,it);double scale=(double)target/(now()-t);if(scale<.25)scale=.25;if(scale>8)scale=8;it=(size_t)(it*scale);if(!it)it=1;}
 bool counted=false;for(int k=0;k<4;++k){bool(*enabled)(void)=dlsym(libs[k].dl,"bench_counter_enabled");assert(enabled);counted=enabled();if(counted){void(*start)(void)=dlsym(libs[k].dl,"bench_counter_start");void(*stop)(uint64_t*)=dlsym(libs[k].dl,"bench_counter_stop");uint64_t counts[3];run(&libs[k],&f,kind,1);start();run(&libs[k],&f,kind,10);stop(counts);printf("ALLOC,%d,%.1f,%.1f,%.1f\n",k,counts[0]/10.,counts[1]/10.,counts[2]/10.);}else for(int w=0;w<3;++w)run(&libs[k],&f,kind,it);}
 if(!counted)for(int batch=0;batch<batches;++batch)for(int step=0;step<4;++step){int k=(step+batch+forkno)%4;if((batch+forkno)&1)k=3-k;uint64_t t=now();run(&libs[k],&f,kind,it);printf("TIME,%d,%d,%zu,%"PRIu64"\n",k,batch,it,now()-t);}
 for(size_t i=0;i<f.len[0];++i)LOOKUP(dl,buckets_drop_a)(f.a[i]);
 for(size_t i=0;i<f.len[1];++i)LOOKUP(dl,buckets_drop_b)(f.b[i]);
 for(size_t i=0;i<f.len[2];++i)LOOKUP(dl,buckets_drop_c)(f.c[i]);
 for(size_t i=0;i<f.len[3];++i)LOOKUP(dl,buckets_drop_d)(f.d[i]);
 LOOKUP(dl,buckets_drop_token)(f.token);assert(LOOKUP(dl,buckets_live)()==0);free(f.a);free(f.b);free(f.c);free(f.d);return 0;
}
