#define EI_PREFIX perftest
#include "enum_inputs_case.h"
#undef EI_PREFIX
static void check_enum_inputs(void) {
    perftest_check_enum_inputs();
    owner_packet packets[]={owner_inline_packet_new(),owner_inline_packet_new()};
    owner_enum_inputs_inline_p0_w1_row rows[]={{&packets[0],-3},{&packets[1],17}};
    owner_enum_inputs_reset();
    size_t before=owner_inline_packet_drops(),consumed=99;
    int64_t result=12345;char *error=NULL;
    assert(owner_enum_inputs_inline((owner_enum_inputs_inline_p0_w1_rows){rows,2},&result,&consumed,&error));
    assert(consumed==2 && result==8206 && error==NULL && owner_enum_inputs_constructors()==2);
    assert(owner_inline_packet_drops()==before+2);
    for(size_t i=0;i<2;++i)owner_drop_packet(&packets[i]);
    puts("C enum inputs PASS: sparse tags, constructors, source failures and inactive fields");
}
