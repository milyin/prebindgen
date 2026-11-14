/* Fixed result cleanup runs through generated C callbacks. */
struct fixed_cleanup_state {int mode, calls, drops, releases;};
static perftest_fixed_cleanup_call_callback_return fixed_cleanup_run(void *context) {
    struct fixed_cleanup_state *state=context;
    ++state->calls;
    perftest_owned_resource *first=perftest_collection_resource_new(7);
    perftest_owned_resource *second=state->mode==4 ? NULL : perftest_collection_resource_new(9);
    static const uint8_t text[]="abc";
    return (perftest_fixed_cleanup_call_callback_return){first,
        state->mode==5 ? 'p' : 'x',
        (perftest_buffer){text,3},second};
}
static void fixed_cleanup_drop(void *context) {++((struct fixed_cleanup_state *)context)->drops;}
static void fixed_cleanup_release(void *context,const perftest_fixed_cleanup_call_callback_return *result) {
    (void)result; ++((struct fixed_cleanup_state *)context)->releases;
}
static void check_fixed_callback_cleanup(void) {
    const int modes[]={0,4,5};
    for(size_t m=0;m<sizeof(modes)/sizeof(*modes);++m) {
        int mode=modes[m];
        struct fixed_cleanup_state state={.mode=mode};
        size_t before=perftest_owned_resource_drops(), consumed=99;
        int64_t result=12345; char *error=NULL;
        bool ok=perftest_fixed_cleanup_call((perftest_fixed_cleanup_call_callback){
            .run=fixed_cleanup_run,.context=&state,.drop=fixed_cleanup_drop,.release_result=fixed_cleanup_release
        },&result,&consumed,&error);
        assert(state.calls==1 && state.drops==1 && state.releases==1 && consumed==1);
        assert(perftest_owned_resource_drops()==before+(mode==4 ? 1u : 2u));
        if(mode==0 || mode==4) {assert(ok && error==NULL && result==(mode==0 ? 19 : 10));}
        else {assert(!ok && error!=NULL && result==12345);perftest_free_error(error);}
    }
    puts("C fixed callback cleanup PASS: optional owners, constructor panic and result release");
}
