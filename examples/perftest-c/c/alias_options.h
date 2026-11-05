/* Valid unchecked calls plus decoding failure cleanup. Never execute invalid aliases. */
#include "unchecked.h"
struct fast_callback_state { int calls, drops; bool invalid; };
static fast_alias_handler_new_callback_return fast_pair_callback(void *context) {
    struct fast_callback_state *state = context;
    ++state->calls;
    return (fast_alias_handler_new_callback_return){fast_collection_resource_new(7), state->invalid ? NULL : fast_collection_resource_new(9)};
}
static void fast_pair_drop(void *context) { ++((struct fast_callback_state *)context)->drops; }
struct fast_rows_state {int releases;};
static fast_alias_callback_rows_callback_return fast_rows_callback(void *context) {
    (void)context;
    static fast_alias_callback_rows_callback_return_row rows[3];
    rows[0].v0=fast_collection_resource_new(7); rows[1].v0=NULL; rows[2].v0=fast_collection_resource_new(9);
    return (fast_alias_callback_rows_callback_return){rows,3};
}
static void fast_rows_release(void *context,const fast_alias_callback_rows_callback_return *value) {
    assert(value->len==3); ++((struct fast_rows_state *)context)->releases;
}
static void check_alias_options(void) {
    fast_owned_resource *left=fast_collection_resource_new(7), *right=fast_collection_resource_new(9);
    assert(fast_alias_fixed(left,right)==16);
    fast_drop_owned_resource(left); fast_drop_owned_resource(right);
    char *error=NULL; int64_t result=12345; size_t consumed=99;
    const uint8_t bytes[]={2,3}, invalid[]={0xff};
    fast_buffer text={(const uint8_t *)"abc",3}, payload={bytes,2};
    fast_buffer_rows_sum_p0_w1_row rows[3];
    for (int mode=0;mode<5;++mode) {
        size_t drops=fast_owned_resource_drops();
        for (size_t i=0;i<3;++i) rows[i]=(fast_buffer_rows_sum_p0_w1_row){fast_collection_resource_new((int64_t)i+1),text,payload};
        if (mode==1) rows[1].v1=(fast_buffer){invalid,1};
        if (mode==2) rows[1].v2=(fast_buffer){NULL,1};
        if (mode==3) rows[1].v2=(fast_buffer){bytes,SIZE_MAX};
        if (mode==4) { fast_drop_owned_resource(rows[1].v0); ++drops; rows[1].v0=NULL; }
        result=12345;
        bool ok=fast_buffer_rows_sum((fast_buffer_rows_sum_p0_w1_rows){rows,3},&result,&consumed,&error);
        if (!mode) {
            assert(ok && result==30 && consumed==3 && error==NULL);
            assert(fast_owned_resource_drops()==drops+3);
        } else {
            assert(!ok && result==12345 && error!=NULL);
            assert(consumed==(mode==4 ? 1u : 2u));
            assert(fast_owned_resource_drops()==drops+consumed);
            assert(fast_option_resource_id(rows[2].v0)==3);
            fast_drop_owned_resource(rows[2].v0);
            fast_free_error(error); error=NULL;
        }
    }
    assert(fast_buffer_rows_optional(0,(fast_buffer_rows_optional_p0_w3_rows){(void *)1,SIZE_MAX},&result,&consumed,&error));
    assert(result==-1 && consumed==0);
    fast_buffer_rows_optional_p0_w3_row optional[]={
        {NULL,{NULL,SIZE_MAX},{NULL,SIZE_MAX}},
        {fast_collection_resource_new(7),text,payload}
    };
    assert(fast_buffer_rows_optional(1,(fast_buffer_rows_optional_p0_w3_rows){optional,2},&result,&consumed,&error));
    assert(result==15 && consumed==1);
    fast_alias_nested_sum_p0_w5_row inner[]={{fast_collection_resource_new(7)},{fast_collection_resource_new(9)}};
    fast_alias_nested_sum_p0_w1_row outer[]={{{NULL,0}},{{inner,2}},{{NULL,0}}};
    size_t drops=fast_owned_resource_drops();
    assert(fast_alias_nested_sum((fast_alias_nested_sum_p0_w1_rows){outer,3})==16);
    assert(fast_owned_resource_drops()==drops+2);
    /* Enum/borrowed temporary rows previously used prepared closures. */
    fast_buffer_rows_enum_p0_w1_row enums[]={
        {fast_collection_resource_new(1),0,1,text,1,payload},
        {fast_collection_resource_new(2),0,1,{invalid,1},1,payload}
    };
    drops=fast_owned_resource_drops(); result=12345;
    assert(!fast_buffer_rows_enum((fast_buffer_rows_enum_p0_w1_rows){enums,2},&result,&consumed,&error));
    assert(result==12345 && consumed==2 && error!=NULL && fast_owned_resource_drops()==drops+2);
    fast_free_error(error); error=NULL;
    /* This binding locally re-enables checks under the unchecked project default. */
    left=fast_collection_resource_new(7);
    fast_collection_alias_p0_w1_row conflict[]={{left}};
    EXPECT_ABORT(fast_collection_alias(((fast_collection_alias_p0_w1_rows){conflict,1}),left));
    assert(fast_option_resource_id(left)==7); fast_drop_owned_resource(left);
    /* Callback conversion retains the registration policy even when invoked by
       a locally checked wrapper after the registration call has returned. */
    struct fast_callback_state state={0};
    fast_alias_handler *handler=fast_alias_handler_new((fast_alias_handler_new_callback){fast_pair_callback,&state,fast_pair_drop});
    drops=fast_owned_resource_drops();
    assert(fast_alias_handler_call(handler,&result,&error) && result==16 && error==NULL);
    assert(state.calls==1 && !state.drops && fast_owned_resource_drops()==drops+2);
    state.invalid=true; result=12345;
    assert(!fast_alias_handler_call(handler,&result,&error) && result==12345 && error!=NULL);
    fast_free_error(error);
    assert(state.calls==2 && fast_owned_resource_drops()==drops+3);
    fast_drop_alias_handler(handler); assert(state.drops==1);
    struct fast_rows_state row_state={0}; drops=fast_owned_resource_drops(); result=12345; error=NULL;
    assert(!fast_alias_callback_rows((fast_alias_callback_rows_callback){.run=fast_rows_callback,.context=&row_state,.release_result=fast_rows_release},&result,&error));
    assert(result==12345 && error!=NULL && row_state.releases==1);
    /* The prefix was decoded and dropped; the unread suffix was reclaimed by
       callback-result ownership tracking. C releases only the row storage. */
    assert(fast_owned_resource_drops()==drops+2); fast_free_error(error);

    puts("C alias options PASS: unchecked inputs, local checks, retained callbacks and error cleanup");
}
