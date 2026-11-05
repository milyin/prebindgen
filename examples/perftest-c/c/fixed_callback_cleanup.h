/* Fixed result cleanup runs through generated C callbacks with checked aliases. */
struct fixed_cleanup_state {int mode, calls, drops, releases;};
static perftest_fixed_cleanup_call_callback_return fixed_cleanup_run(void *context) {
    struct fixed_cleanup_state *state=context;
    ++state->calls;
    perftest_owned_resource *first=state->mode==3 ? NULL : perftest_collection_resource_new(7);
    perftest_owned_resource *second=state->mode==2 ? first : state->mode==4 ? NULL :
        state->mode==7 ? (void *)1 : perftest_collection_resource_new(9);
    static const uint8_t text[]="abc", invalid[]={0xff};
    return (perftest_fixed_cleanup_call_callback_return){first,
        state->mode==1 ? 0xd800 : state->mode==5 ? 'p' : 'x',
        state->mode==6 ? (perftest_buffer){invalid,1} : (perftest_buffer){text,3},second};
}
static void fixed_cleanup_drop(void *context) {++((struct fixed_cleanup_state *)context)->drops;}
static void fixed_cleanup_release(void *context,const perftest_fixed_cleanup_call_callback_return *result) {
    (void)result; ++((struct fixed_cleanup_state *)context)->releases;
}
static void check_fixed_callback_cleanup(void) {
    for(int mode=0;mode<8;++mode) {
        struct fixed_cleanup_state state={.mode=mode};
        size_t before=perftest_owned_resource_drops(), consumed=99;
        int64_t result=12345; char *error=NULL;
        bool ok=perftest_fixed_cleanup_call((perftest_fixed_cleanup_call_callback){
            .run=fixed_cleanup_run,.context=&state,.drop=fixed_cleanup_drop,.release_result=fixed_cleanup_release
        },&result,&consumed,&error);
        assert(state.calls==1 && state.drops==1 && state.releases==1 && consumed==1);
        assert(perftest_owned_resource_drops()==before+(mode==2 || mode==3 || mode==4 || mode==7 ? 1u : 2u));
        if(mode==0 || mode==4) {assert(ok && error==NULL && result==(mode==0 ? 19 : 10));}
        else {assert(!ok && error!=NULL && result==12345);perftest_free_error(error);}
    }
    puts("C fixed callback cleanup PASS: alias rejection, partial acquisition, invalid values, constructor panic and result release");
}
