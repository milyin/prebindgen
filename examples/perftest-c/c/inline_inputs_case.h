/* Inline-owner moves preserve replacement and constructor failure receipts. */
#define II_CAT_RAW(a,b) a##b
#define II_CAT(a,b) II_CAT_RAW(a,b)
typedef II_CAT(II_MIXED,_p0_w1_row) II_CAT(II_CHECK,_mixed_row);
typedef II_CAT(II_PAIR,_p0_w1_row) II_CAT(II_CHECK,_pair_row);
static void II_CHECK(void) {
    for(int mode=0;mode<7;++mode) {
        owner_packet packets[3];owner_resource *first[3],*last[3];
        II_CAT(II_CHECK,_mixed_row) rows[3];
        for(size_t i=0;i<3;++i) {
            packets[i]=owner_inline_packet_new();first[i]=owner_collection_resource_new(10+(int64_t)i);last[i]=owner_collection_resource_new(20+(int64_t)i);
            rows[i]=(II_CAT(II_CHECK,_mixed_row)){first[i],&packets[i],last[i]};
        }
        size_t fail_empty=mode>=1 && mode<=3 ? (size_t)mode:0, fail_ctor=mode>=4 && mode<=6 ? (size_t)mode-3:0;
        owner_inline_inputs_reset(fail_empty,fail_ctor);
        size_t before_packets=owner_inline_packet_drops(),before_boxes=owner_owned_resource_drops(),consumed=99;
        int64_t result=12345;char *error=NULL;
        bool ok=II_MIXED((II_CAT(II_MIXED,_p0_w1_rows)){rows,3},&result,&consumed,&error);
        size_t expected=mode==0 ? 9 : fail_empty ? 3*(fail_empty-1)+1 : fail_ctor ? 3*fail_ctor : 0;
        assert(consumed==expected);
        size_t packet_count=0,box_count=0;
        for(size_t i=0;i<consumed;++i){if(i%3==1)++packet_count;else ++box_count;}
        assert(owner_inline_packet_drops()==before_packets+packet_count);
        assert(owner_owned_resource_drops()==before_boxes+box_count);
        assert(owner_inline_inputs_empty_calls()==(mode==0 ? 3 : fail_empty ? fail_empty : fail_ctor ? fail_ctor : 0));
        assert(owner_inline_inputs_constructor_calls()==(mode==0 ? 3 : fail_empty ? fail_empty-1 : fail_ctor ? fail_ctor : 0));
        assert(owner_inline_inputs_calls()==(mode==0 ? 1u:0u));
        if(mode==0)assert(ok && error==NULL && result==105);
        else {assert(!ok && error!=NULL && result==12345);owner_free_error(error);}
        /* Empty factories are also used by typed C drops. Disable injection
           before releasing the caller-owned suffix and empty consumed slots. */
        owner_inline_inputs_reset(0,0);
        for(size_t i=0;i<3;++i) {
            if(3*i>=consumed){assert(owner_owned_resource_id(first[i])==10+(int64_t)i);owner_drop_resource(first[i]);}
            if(3*i+1>=consumed)assert(owner_inline_child_id(owner_inline_packet_ref(&packets[i]))==42);
            else assert(owner_inline_packet_len(&packets[i])==0);
            if(3*i+2>=consumed){assert(owner_owned_resource_id(last[i])==20+(int64_t)i);owner_drop_resource(last[i]);}
            owner_drop_packet(&packets[i]);
        }
        assert(owner_inline_packet_drops()==before_packets+3 && owner_owned_resource_drops()==before_boxes+6);
    }
    /* A failing replacement must not count the current owner, even when
       earlier fields in this row have already transferred. */
    for(size_t fail=0;fail<=4;++fail) {
        owner_packet packets[4];for(size_t i=0;i<4;++i)packets[i]=owner_inline_packet_new();
        II_CAT(II_CHECK,_pair_row) rows[]={{&packets[0],&packets[1]},{&packets[2],&packets[3]}};
        owner_inline_inputs_reset(fail,0);size_t before=owner_inline_packet_drops(),consumed=99;
        int64_t result=12345;char *error=NULL;
        bool ok=II_PAIR((II_CAT(II_PAIR,_p0_w1_rows)){rows,2},&result,&consumed,&error);
        assert(consumed==(fail ? fail-1:4));
        assert(owner_inline_packet_drops()==before+consumed);
        assert(owner_inline_inputs_constructor_calls()==consumed/2);
        assert(owner_inline_inputs_calls()==(fail ? 0u:1u));
        if(fail){assert(!ok && error!=NULL && result==12345);owner_free_error(error);}
        else assert(ok && result==12 && error==NULL);
        owner_inline_inputs_reset(0,0);
        for(size_t i=0;i<4;++i){if(i>=consumed)assert(owner_inline_child_id(owner_inline_packet_ref(&packets[i]))==42);owner_drop_packet(&packets[i]);}
        assert(owner_inline_packet_drops()==before+4);
    }
}
#undef II_CAT
#undef II_CAT_RAW
