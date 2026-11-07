/* Included once for each alias policy. The result storage outlives decoding. */
#define CC_CAT_RAW(a,b) a##b
#define CC_CAT(a,b) CC_CAT_RAW(a,b)
#define CC_NAME(name) CC_CAT(CC_PREFIX,name)
typedef CC_NAME(_cleanup_group_call_callback_return) CC_NAME(_group_result);
typedef __typeof__(((CC_NAME(_group_result) *)0)->v1.data[0]) CC_NAME(_outer_row);
typedef __typeof__(((CC_NAME(_outer_row) *)0)->v0.data[0]) CC_NAME(_owned_row);
static CC_NAME(_group_result) CC_NAME(_group_return)(void *context) {
    struct collection_cleanup_state *state=context; ++state->calls;
    static CC_NAME(_owned_row) rows[2];
    static CC_NAME(_outer_row) groups[2];
    memset(rows,0,sizeof(rows));
    CC_NAME(_owned_resource) *before=CC_NAME(_collection_resource_new)(state->mode==7 ? -7 : 100);
    CC_NAME(_owned_resource) *after=CC_NAME(_collection_resource_new)(200);
    static const uint8_t text[]="abc", invalid[]={0xff};
    if(state->mode!=2 && state->mode!=3) {
        rows[0]=(CC_NAME(_owned_row)){CC_NAME(_collection_resource_new)(7),
            state->mode==1 ? 0xd800 : state->mode==6 ? 'p' : 'x',
            {text,3},state->mode==9 ? NULL : CC_NAME(_collection_resource_new)(9)};
    }
    if(state->mode!=4) {
        CC_NAME(_owned_resource) *first=state->mode==5 ? before : CC_NAME(_collection_resource_new)(11);
        CC_NAME(_owned_resource) *second=state->mode==10 ? first : state->mode==11 ? (void *)1 : CC_NAME(_collection_resource_new)(13);
        rows[1]=(CC_NAME(_owned_row)){first,'x',state->mode==8 ? (CC_NAME(_buffer)){invalid,1} : (CC_NAME(_buffer)){text,3},second};
    }
    groups[0]=(CC_NAME(_outer_row)){{state->mode==2 ? NULL : &rows[0],state->mode==3 ? SIZE_MAX : 1}};
    groups[1]=(CC_NAME(_outer_row)){{state->mode==4 ? NULL : &rows[1],1}};
    return (CC_NAME(_group_result)){before,{groups,2},after};
}
static void CC_NAME(_group_drop)(void *context) {++((struct collection_cleanup_state *)context)->drops;}
static void CC_NAME(_group_release)(void *context,const CC_NAME(_group_result) *value) {
    assert(value->v1.len==2);++((struct collection_cleanup_state *)context)->releases;
}
typedef CC_NAME(_cleanup_optional_call_callback_return) CC_NAME(_optional_result);
typedef __typeof__(((CC_NAME(_optional_result) *)0)->v1.data[0]) CC_NAME(_optional_row);
static CC_NAME(_optional_result) CC_NAME(_optional_return)(void *context) {
    struct collection_cleanup_state *state=context;++state->calls;
    if(state->mode==0) return (CC_NAME(_optional_result)){0,{(void *)1,SIZE_MAX}};
    if(state->mode==1) return (CC_NAME(_optional_result)){1,{NULL,0}};
    if(state->mode==5) return (CC_NAME(_optional_result)){1,{NULL,1}};
    static CC_NAME(_optional_row) rows[3];
    static const uint8_t text[]="abc",invalid[]={0xff};
    rows[0]=(CC_NAME(_optional_row)){NULL,0xd800,{NULL,SIZE_MAX},(void *)1};
    rows[1]=(CC_NAME(_optional_row)){CC_NAME(_collection_resource_new)(7),'x',{text,3},CC_NAME(_collection_resource_new)(9)};
    rows[2]=(CC_NAME(_optional_row)){state->mode==7 ? rows[1].v3 : CC_NAME(_collection_resource_new)(11),
        state->mode==3 ? 0xd800 : state->mode==6 ? 'p' : 'x',
        state->mode==4 ? (CC_NAME(_buffer)){invalid,1} : (CC_NAME(_buffer)){text,3},CC_NAME(_collection_resource_new)(13)};
    return (CC_NAME(_optional_result)){1,{rows,3}};
}
static void CC_NAME(_optional_release)(void *context,const CC_NAME(_optional_result) *value) {
    (void)value;++((struct collection_cleanup_state *)context)->releases;
}
static void CC_NAME(_check_collection_cleanup)(void) {
    for(int mode=0;mode<12;++mode) {
        if(!CC_CHECKED && (mode==5 || mode==10)) continue;
        struct collection_cleanup_state state={.mode=mode};
        size_t drops=CC_NAME(_owned_resource_drops)(),accepted=99;
        int64_t result=12345;char *error=NULL;
        bool ok=CC_NAME(_cleanup_group_call)((CC_NAME(_cleanup_group_call_callback)){
            .run=CC_NAME(_group_return),.context=&state,.drop=CC_NAME(_group_drop),.release_result=CC_NAME(_group_release)
        },&result,&accepted,&error);
        assert(accepted==1 && state.calls==1 && state.drops==1 && state.releases==1);
        size_t expected=(mode==2 || mode==3 || mode==4) ? 4 : (mode==5 || mode==9 || mode==10 || mode==11) ? 5 : 6;
        assert(CC_NAME(_owned_resource_drops)()==drops+expected);
        if(mode==0 || mode==9) {assert(ok && error==NULL && result==(mode==0 ? 346 : 337));}
        else {assert(!ok && error!=NULL && result==12345);CC_NAME(_free_error)(error);}
    }
    for(int mode=0;mode<8;++mode) {
        if(!CC_CHECKED && mode==7) continue;
        struct collection_cleanup_state state={.mode=mode};
        size_t drops=CC_NAME(_owned_resource_drops)(),accepted=99;
        int64_t result=12345;char *error=NULL;
        bool ok=CC_NAME(_cleanup_optional_call)((CC_NAME(_cleanup_optional_call_callback)){
            .run=CC_NAME(_optional_return),.context=&state,.drop=CC_NAME(_group_drop),.release_result=CC_NAME(_optional_release)
        },&result,&accepted,&error);
        assert(accepted==1 && state.calls==1 && state.drops==1 && state.releases==1);
        assert(CC_NAME(_owned_resource_drops)()==drops+((mode<2 || mode==5) ? 0 : mode==7 ? 3 : 4));
        if(mode<3) {assert(ok && error==NULL && result==(mode==2 ? 46 : 0));}
        else {assert(!ok && error!=NULL && result==12345);CC_NAME(_free_error)(error);}
    }
}
#undef CC_NAME
#undef CC_CAT
#undef CC_CAT_RAW
