/* C owns the row allocation. Rust takes its inline values and pointer handles;
   release_result runs after success/failure cleanup and frees only row storage. */
typedef owner_inline_callback_rows_callback_return_row mixed_callback_row;
typedef struct {mixed_callback_row *rows;size_t len,packets_before,boxes_before,boxes,released,dropped;bool failure;} mixed_callback_context;
static owner_inline_callback_rows_callback_return mixed_callback_run(void *raw) {
    mixed_callback_context *ctx=raw;
    return (owner_inline_callback_rows_callback_return){ctx->rows,ctx->len};
}
static void mixed_callback_release(void *raw,const owner_inline_callback_rows_callback_return *value) {
    mixed_callback_context *ctx=raw;
    assert(value->data==ctx->rows && value->len==ctx->len && ctx->released==0);
    assert(owner_inline_packet_drops()==ctx->packets_before+(ctx->failure ? ctx->len:0));
    assert(owner_owned_resource_drops()==ctx->boxes_before+(ctx->failure ? ctx->boxes:0));
    assert(owner_inline_inputs_empty_calls()==0);
    ++ctx->released;free(ctx->rows);ctx->rows=NULL;
}
static void mixed_callback_drop(void *raw) {mixed_callback_context *ctx=raw;assert(ctx->released==1);++ctx->dropped;}
static mixed_callback_row mixed_callback_make(size_t i) {
    owner_packet packet=owner_inline_packet_new();
    return (mixed_callback_row){owner_collection_resource_new(10+(int64_t)i),packet,owner_collection_resource_new(20+(int64_t)i),'a',owner_inline_packet_bytes_address(&packet),owner_inline_packet_text_address(&packet)};
}
static owner_inline_callback_optional_callback_return mixed_callback_optional_run(void *raw) {
    return *(owner_inline_callback_optional_callback_return *)raw;
}
static void mixed_callback_optional_release(void *raw,const owner_inline_callback_optional_callback_return *value) {
    owner_inline_callback_optional_callback_return *ctx=raw;
    assert(ctx->v0==value->v0 && ctx->v1.data==value->v1.data);
    if(value->v0==1)free(value->v1.data);
}
static void check_mixed_callback_rows(void) {
    const int modes[]={0,1,7,8};
    for(size_t m=0;m<sizeof(modes)/sizeof(*modes);++m) {
        int mode=modes[m];
        mixed_callback_context ctx={.rows=malloc(3*sizeof(mixed_callback_row)),.len=3,.boxes=6,.failure=mode!=0};assert(ctx.rows);
        for(size_t i=0;i<3;++i)ctx.rows[i]=mixed_callback_make(i);
        ctx.packets_before=owner_inline_packet_drops();ctx.boxes_before=owner_owned_resource_drops();
        size_t ctor_panic=mode==1 ? 2 : mode==7 ? 1 : mode==8 ? 3 : 0;
        owner_inline_inputs_reset(0,ctor_panic);
        int64_t result=12345;size_t consumed=99;char *error=NULL;
        owner_inline_callback_rows_callback callback={.run=mixed_callback_run,.context=&ctx,.drop=mixed_callback_drop,.release_result=mixed_callback_release};
        bool ok=owner_inline_callback_rows(callback,&result,&consumed,&error);
        assert(consumed==1 && ctx.released==1 && ctx.dropped==1);
        assert(owner_inline_packet_drops()==ctx.packets_before+ctx.len && owner_owned_resource_drops()==ctx.boxes_before+ctx.boxes);
        assert(owner_inline_inputs_constructor_calls()==(mode==0 ? 3 : ctor_panic));
        assert(owner_inline_inputs_calls()==(mode==0 ? 1u:0u));
        if(mode==0)assert(ok && result==396 && error==NULL);
        else {assert(!ok && result==12345 && error!=NULL);owner_free_error(error);}
    }
    owner_inline_inputs_reset(0,0);
    int64_t result=12345;size_t consumed=99;char *error=NULL;
    owner_inline_callback_optional_callback_return absent={0,{(void *)1,SIZE_MAX}};
    assert(owner_inline_callback_optional((owner_inline_callback_optional_callback){.run=mixed_callback_optional_run,.context=&absent,.release_result=mixed_callback_optional_release},&result,&consumed,&error));
    assert(result==-1 && consumed==1 && error==NULL);
    typedef owner_callback_70db60a6f9dbbb06_callback_return_w3_row optional_row;
    optional_row *rows=malloc(3*sizeof(*rows));assert(rows);memset(rows,0xa5,3*sizeof(*rows));rows[0].v0=NULL;
    for(size_t i=1;i<3;++i){mixed_callback_row row=mixed_callback_make(i);rows[i]=(optional_row){row.v0,row.v1,row.v2,row.v3,row.v4,row.v5};}
    owner_inline_callback_optional_callback_return present={1,{rows,3}};
    size_t packets=owner_inline_packet_drops(),boxes=owner_owned_resource_drops();
    assert(owner_inline_callback_optional((owner_inline_callback_optional_callback){.run=mixed_callback_optional_run,.context=&present,.release_result=mixed_callback_optional_release},&result,&consumed,&error));
    assert(result==266 && consumed==1 && error==NULL);
    assert(owner_inline_inputs_constructor_calls()==2);
    assert(owner_inline_packet_drops()==packets+2 && owner_owned_resource_drops()==boxes+4);
    puts("C mixed callback replay PASS: original payload addresses, constructor failure, inactive fields and cleanup before row release");
}
