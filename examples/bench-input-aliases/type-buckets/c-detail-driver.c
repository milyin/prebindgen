// Reuse the exact original ABI calls, checksum checks and alias-rejection probes.
#define main previous_driver_main
#include "driver.c"
#undef main
static uintptr_t address_at(const void *data,size_t index){uintptr_t v;memcpy(&v,(const char*)data+index*sizeof(void*),sizeof(v));return v;}
static int pointer_order(const void *a,const void*b){uintptr_t x=address_at(a,0),y=address_at(b,0);return (x>y)-(x<y);}
static void swap_at(void *data,size_t a,size_t b){unsigned char tmp[sizeof(void*)];char *p=data;memcpy(tmp,p+a*sizeof(void*),sizeof tmp);memcpy(p+a*sizeof(void*),p+b*sizeof(void*),sizeof tmp);memcpy(p+b*sizeof(void*),tmp,sizeof tmp);}
static void reorder(void *rows,size_t n,const char *layout){
 if(strcmp(layout,"creation")==0)return;
 if(strcmp(layout,"shuffled")){qsort(rows,n,sizeof(void*),pointer_order);}
 if(!strcmp(layout,"reverse"))for(size_t i=0;i<n/2;++i)swap_at(rows,i,n-1-i);
 else if(!strcmp(layout,"shuffled"))for(size_t i=n;i>1;--i)swap_at(rows,i-1,next(i));
 else if(!strcmp(layout,"near"))for(size_t i=0;i<n/100;++i)swap_at(rows,next(n),next(n));
 else assert(!strcmp(layout,"sorted"));
}
static struct fixture prepare_detail(void *dl,size_t n,const char*allocator,const char*layout,unsigned forkno){
 struct fixture f={.n=n};rng=42+forkno*177;
 for(size_t i=0;i<4;++i)f.len[i]=n/4+(i<n%4);
 f.a=calloc(f.len[0]+1,sizeof(*f.a));f.b=calloc(f.len[1]+1,sizeof(*f.b));f.c=calloc(f.len[2]+1,sizeof(*f.c));f.d=calloc(f.len[3]+1,sizeof(*f.d));assert(f.a&&f.b&&f.c&&f.d);
 buckets_a **filler=NULL;
 if(!strcmp(allocator,"fragmented")){
  filler=calloc(n*2,sizeof(*filler));assert(filler);for(size_t i=0;i<n*2;++i)filler[i]=LOOKUP(dl,buckets_a_new)(-1);
  for(size_t i=n*2;i>1;--i){size_t j=next(i);buckets_a*p=filler[i-1];filler[i-1]=filler[j];filler[j]=p;}
  for(size_t i=0;i<n;++i)LOOKUP(dl,buckets_drop_a)(filler[i]);
 }
 if(!strcmp(allocator,"by_type")){
  for(size_t i=0;i<f.len[0];++i)f.a[i]=LOOKUP(dl,buckets_a_new)(i+1);
  for(size_t i=0;i<f.len[1];++i)f.b[i]=LOOKUP(dl,buckets_b_new)(i+1);
  for(size_t i=0;i<f.len[2];++i)f.c[i]=LOOKUP(dl,buckets_c_new)(i+1);
  for(size_t i=0;i<f.len[3];++i)f.d[i]=LOOKUP(dl,buckets_d_new)(i+1);
 }else {
  assert(!strcmp(allocator,"interleaved")||!strcmp(allocator,"fragmented"));
  for(size_t i=0;i<f.len[0];++i){f.a[i]=LOOKUP(dl,buckets_a_new)(i+1);if(i<f.len[1])f.b[i]=LOOKUP(dl,buckets_b_new)(i+1);if(i<f.len[2])f.c[i]=LOOKUP(dl,buckets_c_new)(i+1);if(i<f.len[3])f.d[i]=LOOKUP(dl,buckets_d_new)(i+1);}
 }
 f.token=LOOKUP(dl,buckets_token_new)(7);
 if(filler){for(size_t i=n;i<n*2;++i)LOOKUP(dl,buckets_drop_a)(filler[i]);free(filler);}
 void *rows[]={f.a,f.b,f.c,f.d};uintptr_t lo[4],hi[4];size_t global_down=0;uintptr_t previous=0;bool first=true;
 for(size_t g=0;g<4;++g){
  f.sum+=f.len[g]*(f.len[g]+1)/2;reorder(rows[g],f.len[g],layout);size_t down=0;lo[g]=UINTPTR_MAX;hi[g]=0;
  for(size_t i=0;i<f.len[g];++i){uintptr_t a=address_at(rows[g],i);if(a<lo[g])lo[g]=a;if(a>hi[g])hi[g]=a;if(i&&address_at(rows[g],i-1)>a)down++;if(!first&&previous>a)global_down++;first=false;previous=a;}
  printf("INPUT,%zu,%zu,%zu,%"PRIuPTR",%"PRIuPTR"\n",g,f.len[g],down,lo[g],hi[g]);
 }
 size_t overlaps=0;for(int a=0;a<4;++a)for(int b=a+1;b<4;++b)if(f.len[a]&&f.len[b]&&lo[a]<=hi[b]&&lo[b]<=hi[a])overlaps++;
 printf("SHAPE,%zu,%zu\n",global_down,overlaps);return f;
}
static void release_detail(void*dl,struct fixture*f){
 for(size_t i=0;i<f->len[0];++i)LOOKUP(dl,buckets_drop_a)(f->a[i]);
 for(size_t i=0;i<f->len[1];++i)LOOKUP(dl,buckets_drop_b)(f->b[i]);
 for(size_t i=0;i<f->len[2];++i)LOOKUP(dl,buckets_drop_c)(f->c[i]);
 for(size_t i=0;i<f->len[3];++i)LOOKUP(dl,buckets_drop_d)(f->d[i]);
 LOOKUP(dl,buckets_drop_token)(f->token);free(f->a);free(f->b);free(f->c);free(f->d);assert(LOOKUP(dl,buckets_live)()==0);
}
int main(int argc,char**argv){
 assert(argc==15);int kind=atoi(argv[1]);size_t n=strtoull(argv[2],0,10);const char*layout=argv[3];const char*allocator=argv[4];int batches=atoi(argv[5]);uint64_t target=strtoull(argv[6],0,10)*1000000;int forkno=atoi(argv[7]);assert(!strcmp(argv[8],"ab")||!strcmp(argv[8],"aa"));
 struct lib libs[6];
 for(int i=0;i<6;++i){void*dl=dlopen(argv[9+i],RTLD_NOW|RTLD_LOCAL);if(!dl){puts(dlerror());abort();}libs[i]=(struct lib){dl,LOOKUP(dl,buckets_mut_one),LOOKUP(dl,buckets_mixed_mut),LOOKUP(dl,buckets_mixed_shared)};assert(libs[i].one&&libs[i].mut&&libs[i].shared);
  for(int j=0;j<i;++j){assert(libs[i].mut!=libs[j].mut);}
  bool(*enabled)(void)=dlsym(dl,"bench_counter_enabled");assert(enabled&&!enabled());
  Dl_info info;assert(dladdr(libs[i].mut,&info));assert(!strcmp(info.dli_fname,argv[9+i]));printf("LIBRARY,%d,%s,%p\n",i,info.dli_fname,(void*)libs[i].mut);
 }
 if(n==0){for(int i=0;i<6;++i)probes(libs[i].dl,0);return 0;}
 struct fixture f=prepare_detail(libs[0].dl,n,allocator,layout,(unsigned)forkno);size_t it=10000/(n+1)+1;
 for(int r=0;r<4;++r){uint64_t t=now();run(libs,&f,kind,it);double scale=(double)target/(now()-t);if(scale<.25)scale=.25;if(scale>8)scale=8;it=(size_t)(it*scale);if(!it)it=1;}
 for(int i=0;i<6;++i)for(int w=0;w<3;++w)run(&libs[i],&f,kind,it);
 for(int batch=0;batch<batches;++batch)for(int step=0;step<6;++step){int v=(step+batch+forkno)%6;if((batch+forkno)&1)v=5-v;uint64_t t=now();run(&libs[v],&f,kind,it);printf("TIME,%d,%d,%zu,%"PRIu64"\n",v,batch,it,now()-t);}
 release_detail(libs[0].dl,&f);return 0;
}
