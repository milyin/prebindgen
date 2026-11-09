#define II_MIXED owner_inline_inputs_mixed
#define II_PAIR owner_inline_inputs_pair
#define II_CHECK check_inline_inputs_checked
#include "inline_inputs_case.h"
#undef II_MIXED
#undef II_PAIR
#undef II_CHECK
static void check_inline_inputs(void) {
    check_inline_inputs_checked();
    owner_inline_inputs_reset(0,0);
    int64_t result=12345;size_t consumed=99;char *error=NULL;
    assert(owner_inline_inputs_optional(0,(owner_inline_inputs_optional_p0_w3_rows){(void *)1,SIZE_MAX},&result,&consumed,&error));
    assert(result==-1 && consumed==0 && owner_inline_inputs_empty_calls()==0);
    owner_packet packet=owner_inline_packet_new();
    owner_inline_inputs_optional_p0_w3_row optional[]={
        {NULL,(void *)1,(void *)1},{owner_collection_resource_new(10),&packet,owner_collection_resource_new(20)}
    };
    assert(owner_inline_inputs_optional(1,(owner_inline_inputs_optional_p0_w3_rows){optional,2},&result,&consumed,&error));
    assert(result==33 && consumed==3 && owner_inline_inputs_empty_calls()==1);
    owner_drop_packet(&packet);
    owner_packet shared=owner_inline_packet_new(),receiver=owner_inline_packet_new();
    owner_inline_inputs_borrow_p1_w1_row borrowed[]={{&shared},{&shared}};
    uintptr_t value=12345;owner_inline_inputs_reset(0,0);
    assert(owner_inline_inputs_borrow(&receiver,(owner_inline_inputs_borrow_p1_w1_rows){borrowed,2},&value,&error));
    assert(value==9 && owner_inline_inputs_constructor_calls()==2 && owner_inline_inputs_empty_calls()==0);
    owner_drop_packet(&shared);owner_drop_packet(&receiver);
    owner_bytes bytes[3];owner_text texts[3];
    owner_inline_inputs_bytes_p0_w1_row byte_rows[3];owner_inline_inputs_texts_p0_w1_row text_rows[3];
    uintptr_t expected_bytes=0,expected_text=0;
    for(size_t i=0;i<3;++i) {
        owner_buffer_view view;uintptr_t length;
        bytes[i]=owner_inline_bytes_new(&view,&length);texts[i]=owner_inline_text_new();
        expected_bytes^=(uintptr_t)view.data;expected_text^=(uintptr_t)owner_text_view(&texts[i]).data;
        byte_rows[i]=(owner_inline_inputs_bytes_p0_w1_row){&bytes[i]};text_rows[i]=(owner_inline_inputs_texts_p0_w1_row){&texts[i]};
    }
    assert(owner_inline_inputs_bytes((owner_inline_inputs_bytes_p0_w1_rows){byte_rows,3},&value,&consumed,&error));
    assert(value==expected_bytes && consumed==3 && error==NULL);
    assert(owner_inline_inputs_texts((owner_inline_inputs_texts_p0_w1_rows){text_rows,3},&value,&consumed,&error));
    assert(value==expected_text && consumed==3 && error==NULL);
    for(size_t i=0;i<3;++i){assert(owner_bytes_len(&bytes[i])==0 && owner_text_len(&texts[i])==0);owner_drop_bytes(&bytes[i]);owner_drop_text(&texts[i]);}
    puts("C inline input replay PASS: partial replacement panics, borrows, original buffer addresses and suffix cleanup");
}
