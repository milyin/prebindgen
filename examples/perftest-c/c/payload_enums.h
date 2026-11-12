/* Real C callers: transferred enum fields are moved without copying payloads. */
static perftest_wire_event wire_event_return(void *context, perftest_wire_event value) {
    const uint8_t **original = context;
    assert(value.tag == perftest_wire_event_Packet);
    perftest_buffer_view bytes = bytes_view(&value.payload.Packet.bytes);
    assert(bytes.len == 3 && bytes.data[2] == 3);
    *original = bytes.data;
    return value;
}
static perftest_wire_owned_event wire_owned_return(void *context, perftest_wire_owned_event value) {
    (void)context;
    assert(value.tag == perftest_wire_owned_event_Pair);
    return value;
}
static perftest_wire_decision wire_result_return(void *context, perftest_wire_notice value) {
    (void)context;
    assert(value.tag == perftest_wire_notice_Ok);
    perftest_wire_decision returned = {.tag = perftest_wire_decision_Ok};
    returned.payload.Ok.v0 = 123;
    return returned;
}
static void check_payload_enums(void) {
    perftest_wire_event event = {.tag = perftest_wire_event_Count};
    event.payload.Count.v0 = UINT64_MAX;
    perftest_wire_event echoed = perftest_wire_event_echo(&event);
    assert(event.tag == -1 && echoed.tag == perftest_wire_event_Count);
    assert(echoed.payload.Count.v0 == UINT64_MAX);
    perftest_drop_wire_event(&event);
    perftest_drop_wire_event(&echoed);
    const uint8_t *original = NULL;
    perftest_wire_event returned = perftest_wire_event_callback((perftest_wire_event_callback_callback){.context=&original,.run=wire_event_return});
    assert(bytes_view(&returned.payload.Packet.bytes).data == original);
    perftest_wire_event taken = perftest_take_wire_event(&returned);
    assert(returned.tag == -1);
    perftest_drop_wire_event(&taken);
    perftest_drop_wire_event(&taken);
    uint64_t before = perftest_callback_return_resource_live();
    perftest_wire_owned_event owned = perftest_wire_owned_new();
    assert(perftest_callback_return_resource_live() == before + 2);
    perftest_wire_owned_event moved = perftest_wire_owned_echo(&owned);
    assert(owned.tag == -1 && moved.tag == perftest_wire_owned_event_Pair);
    perftest_drop_wire_owned_event(&moved);
    assert(perftest_callback_return_resource_live() == before);
    moved = perftest_wire_owned_callback((perftest_wire_owned_callback_callback){.run=wire_owned_return});
    perftest_drop_wire_owned_event(&moved);
    assert(perftest_callback_return_resource_live() == before);
    perftest_wire_decision decision = perftest_wire_result_callback((perftest_wire_result_callback_callback){.run=wire_result_return});
    assert(decision.tag == perftest_wire_decision_Ok && decision.payload.Ok.v0 == 123);
    perftest_wire_decision decision_echo = perftest_wire_result_echo(&decision);
    assert(decision.tag == -1 && decision_echo.payload.Ok.v0 == 123);
    perftest_drop_wire_decision(&decision_echo);
}

static perftest_wire_checked_event wire_checked_return(void *context) {
    (void)context;
    perftest_wire_checked_event value = {.tag=perftest_wire_checked_event_Data};
    value.payload.Data.first = perftest_callback_return_resource_new(1);
    value.payload.Data.marker = -1;
    value.payload.Data.last = perftest_callback_return_resource_new(2);
    return value;
}
static void check_payload_failures(void) {
    uint64_t before = perftest_callback_return_resource_live();
    size_t accepted = 0;
    int64_t result = 0;
    char *error = NULL;
    assert(!perftest_wire_checked_callback((perftest_wire_checked_callback_callback){.run=wire_checked_return},&result,&accepted,&error));
    assert(accepted == 1 && error != NULL);
    perftest_free_error(error);
    assert(perftest_callback_return_resource_live() == before);
    error = NULL;
    perftest_wire_checked_event output;
    assert(!perftest_wire_checked_new(-2,&output,&error));
    assert(error != NULL);
    perftest_free_error(error);
    assert(perftest_callback_return_resource_live() == before);
    int64_t count = 0; uint64_t x = 0; double y = 0;
    int32_t tag = perftest_wire_bare_echo(perftest_WIRE_BARE_PACKET_TAG,0,19,2.5,&count,&x,&y);
    assert(tag == perftest_WIRE_BARE_PACKET_TAG && count == 0 && x == 19 && y == 2.5);
    perftest_wire_empty empty={.tag=perftest_wire_empty_Tuple};
    perftest_wire_empty echoed=perftest_wire_empty_echo(empty);
    assert(echoed.tag == perftest_wire_empty_Tuple);
}

static void check_optional_payloads_and_rows(void) {
    /* The None payload's owner bits are deliberately invalid. They must never
       be read as a Vec or dropped, whether passed in or dropped directly. */
    perftest_wire_optional_event absent;
    memset(&absent,0xa5,sizeof(absent));
    absent.tag=perftest_wire_optional_event_Bytes;
    absent.payload.Bytes.payload_v0=0;
    absent.payload.Bytes.tail=NULL;
    perftest_wire_optional_event echoed=perftest_wire_optional_echo(&absent);
    assert(absent.tag==-1 && echoed.payload.Bytes.payload_v0==0);
    perftest_drop_wire_optional_event(&echoed);
    memset(&absent,0xa5,sizeof(absent));
    absent.tag=perftest_wire_optional_event_Bytes;
    absent.payload.Bytes.payload_v0=0;
    absent.payload.Bytes.tail=NULL;
    perftest_drop_wire_optional_event(&absent);
    uint64_t before=perftest_callback_return_resource_live();
    perftest_wire_owned_rows_new_ret_w0_rows rows=perftest_wire_owned_rows_new();
    assert(rows.len==4 && perftest_callback_return_resource_live()==before+4);
    perftest_wire_owned_rows_p0_w1_row input[4];
    for(size_t i=0;i<rows.len;++i) input[i].v0=&rows.data[i].v0;
    perftest_wire_owned_rows_ret_w0_rows moved=perftest_wire_owned_rows((perftest_wire_owned_rows_p0_w1_rows){input,4});
    for(size_t i=0;i<rows.len;++i) assert(rows.data[i].v0.tag==-1);
    perftest_wire_owned_rows_new_ret_w0_rows_free(rows);
    for(size_t i=0;i<moved.len;++i) perftest_drop_wire_owned_event(&moved.data[i].v0);
    perftest_wire_owned_rows_ret_w0_rows_free(moved);
    assert(perftest_callback_return_resource_live()==before);
}
