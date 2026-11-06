/* Exercise the same C-to-Rust rows under both alias-check policies. */
#define EI_PREFIX perftest
#define EI_CHECKED 1
#include "enum_inputs_case.h"
#undef EI_CHECKED
#undef EI_PREFIX
#define EI_PREFIX fast
#define EI_CHECKED 0
#include "enum_inputs_case.h"
#undef EI_CHECKED
#undef EI_PREFIX
static void check_enum_inputs(void) {
    perftest_check_enum_inputs();
    fast_check_enum_inputs();
    /* Inline-owner rows retain the preparation path, with enum decoding still
       deferred until construction. A bad later tag consumes the current owner. */
    for(int mode=0;mode<3;++mode) {
        owner_packet packets[]={owner_inline_packet_new(),owner_inline_packet_new()};
        owner_enum_inputs_inline_p0_w1_row rows[]={{&packets[0],-3},{&packets[1],17}};
        if(mode)rows[mode-1].v1=42;
        owner_enum_inputs_reset();
        size_t before=owner_inline_packet_drops(),consumed=99;
        int64_t result=12345;char *error=NULL;
        bool ok=owner_enum_inputs_inline((owner_enum_inputs_inline_p0_w1_rows){rows,2},&result,&consumed,&error);
        assert(consumed==(mode==1 ? 1u : 2u));
        assert(owner_enum_inputs_constructors()==(mode==0 ? 2u : mode==1 ? 0u : 1u));
        assert(owner_inline_packet_drops()==before+consumed);
        if(mode==0)assert(ok && result==8206 && error==NULL);
        else {assert(!ok && result==12345 && error!=NULL);owner_free_error(error);}
        if(mode==1)assert(owner_inline_child_id(owner_inline_packet_ref(&packets[1]))==42);
        owner_drop_packet(&packets[0]);owner_drop_packet(&packets[1]);
        assert(owner_inline_packet_drops()==before+2);
    }
    puts("C enum input replay PASS: sparse tags, optional/selector borrows, aliases and failure cleanup");
}
