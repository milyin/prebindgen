#include "owners.h"
_Static_assert(sizeof(owner_bytes)==3*sizeof(uintptr_t),"native Vec carrier layout");
_Static_assert(_Alignof(owner_bytes)==_Alignof(uintptr_t),"native Vec carrier alignment");
static owner_bytes return_inline_bytes(void *context,owner_bytes bytes) {
    (void)context;
    assert(owner_bytes_len(&bytes)==3);
    return bytes; /* Move ownership back to Rust, without a C allocation. */
}
static owner_packet return_inline_packet(void *context,owner_packet packet) {
    (void)context;
    assert(owner_inline_child_id(owner_inline_packet_ref(&packet))==42);
    return packet;
}
static owner_inline_return_catch_callback_return return_bad_pair(void *context) {
    (void)context;return (owner_inline_return_catch_callback_return){owner_inline_packet_new(),0xd800};
}
static owner_inline_rows_catch_callback_return return_bad_rows(void *context) {
    (void)context;
    static owner_inline_rows_catch_callback_return_row rows[2];
    rows[0]=(owner_inline_rows_catch_callback_return_row){owner_inline_packet_new(),'a'};
    rows[1]=(owner_inline_rows_catch_callback_return_row){owner_inline_packet_new(),0xd800};
    return (owner_inline_rows_catch_callback_return){rows,2};
}
static void check_inline_owners(void) {
    owner_buffer_view view={0}; uintptr_t len=0;
    owner_bytes bytes=owner_inline_bytes_new(&view,&len);
    assert(len==3 && view.len==3 && memcmp(view.data,"a\0b",3)==0);
    assert(view.data==owner_bytes_view(&bytes).data);
    assert(owner_inline_bytes_capacity(&bytes)==4096);
    size_t accepted=1;uintptr_t output=123;char *error=NULL;
    assert(!owner_inline_bytes_alias(&bytes,&bytes,&output,&accepted,&error));
    assert(accepted==1 && output==123 && error!=NULL);
    owner_free_error(error);
    assert(owner_bytes_len(&bytes)==0);
    owner_drop_bytes(&bytes);
    bytes=owner_inline_bytes_new(&view,&len);
    assert(owner_inline_bytes_take(&bytes)==4096);
    assert(owner_bytes_len(&bytes)==0);
    owner_drop_bytes(&bytes);owner_drop_bytes(&bytes);
    owner_text text=owner_inline_text_new();
    assert(owner_text_len(&text)==strlen("héllo"));
    assert(owner_inline_text_capacity(&text)==4096);
    assert(owner_inline_text_take(&text)==4096);
    owner_drop_text(&text);owner_drop_text(&text);
    owner_boxed_numbers *boxed=owner_inline_boxed_numbers_new();
    assert(owner_inline_boxed_numbers_len(boxed)==2);
    assert(owner_inline_boxed_numbers_take(boxed)==16);
    owner_numbers numbers=owner_inline_numbers_new();
    assert(owner_numbers_len(&numbers)==2);
    assert(*owner_numbers_get(&numbers,1)==9);
    assert(owner_numbers_get(&numbers,2)==NULL);
    assert(owner_inline_numbers_capacity(&numbers)==4096);
    assert(owner_inline_numbers_take(&numbers)==16);
    owner_drop_numbers(&numbers);
    uintptr_t before=owner_inline_packet_drops();
    owner_packet packet=owner_inline_packet_new();
    assert(owner_inline_child_id(owner_inline_packet_ref(&packet))==42);
    assert(owner_inline_packet_take(&packet)==4096);
    owner_drop_packet(&packet);owner_drop_packet(&packet);
    assert(owner_inline_packet_drops()==before+1);
    assert(owner_inline_packet_optional(0,&packet)==0);owner_drop_packet(&packet);
    assert(owner_inline_packet_optional(1,&packet)==1);owner_drop_packet(&packet);
    assert(owner_inline_packet_drops()==before+2);
    assert(owner_inline_packet_callback((owner_inline_packet_callback_callback){.run=return_inline_packet})==4096);
    assert(owner_inline_packet_drops()==before+3);
    assert(owner_inline_bytes_callback((owner_inline_bytes_callback_callback){.run=return_inline_bytes})==4096);
    memset(&packet,42,sizeof(packet));owner_packet unchanged=packet;
    error=NULL;
    assert(!owner_inline_packet_result(&packet,&len,&error));
    assert(error!=NULL && memcmp(&packet,&unchanged,sizeof(packet))==0);
    owner_free_error(error);
    assert(owner_inline_packet_drops()==before+4);
    assert(owner_inline_return_catch((owner_inline_return_catch_callback){.run=return_bad_pair}));
    assert(owner_inline_packet_drops()==before+5);
    assert(owner_inline_rows_catch((owner_inline_rows_catch_callback){.run=return_bad_rows}));
    assert(owner_inline_packet_drops()==before+7);
    owner_packet inputs[2]={owner_inline_packet_new(),owner_inline_packet_new()};
    owner_inline_packets_take_p0_w1_row input_rows[]={{&inputs[0]},{&inputs[1]}};
    assert(owner_inline_packets_take((owner_inline_packets_take_p0_w1_rows){input_rows,2})==6);
    owner_drop_packet(&inputs[0]);owner_drop_packet(&inputs[1]);
    assert(owner_inline_packet_drops()==before+9);
    owner_inline_packets_rows_ret_w3_rows output_rows={0};
    error=NULL;
    assert(!owner_inline_packets_rows(&output_rows,&len,&error));
    assert(error!=NULL);owner_free_error(error);
    assert(owner_inline_packet_drops()==before+11);
    puts("C inline owners PASS: retained allocation/capacity, views, typed drop, moves, alias rejection, callbacks and rollback");
}
