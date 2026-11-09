#define EI_CAT_RAW(a,b) a##b
#define EI_CAT(a,b) EI_CAT_RAW(a,b)
#define EI_NAME(name) EI_CAT(EI_PREFIX,name)
static void EI_NAME(_check_enum_inputs)(void) {
    const int modes[]={0,3};
    for(size_t m=0;m<sizeof(modes)/sizeof(*modes);++m) {
        int mode=modes[m];
        EI_NAME(_owned_resource) *owners[3];
        EI_NAME(_enum_inputs_take_p0_w1_row) rows[3];
        for(size_t i=0;i<3;++i) {
            owners[i]=EI_NAME(_collection_resource_new)(mode==3 && i==1 ? -2 : (int64_t)i+1);
            rows[i]=(EI_NAME(_enum_inputs_take_p0_w1_row)){owners[i],i==0 ? -3 : i==1 ? 17 : 0};
        }
        EI_NAME(_enum_inputs_reset)();
        size_t before=EI_NAME(_owned_resource_drops)(),consumed=99;
        int64_t result=12345;char *error=NULL;
        bool ok=EI_NAME(_enum_inputs_take)((EI_NAME(_enum_inputs_take_p0_w1_rows)){rows,3},&result,&consumed,&error);
        size_t expected=mode==0 ? 3 : 2;
        assert(consumed==expected && EI_NAME(_owned_resource_drops)()==before+expected);
        assert(EI_NAME(_enum_inputs_constructors)()==(mode==0 ? 3u : 2u));
        assert(EI_NAME(_enum_inputs_calls)()==(mode==0 ? 1u : 0u));
        if(mode==0)assert(ok && result==20 && error==NULL);
        else {assert(!ok && result==12345 && error!=NULL);EI_NAME(_free_error)(error);}
        for(size_t i=consumed;i<3;++i) {
            assert(EI_NAME(_option_resource_id)(owners[i])==(int64_t)i+1);
            EI_NAME(_drop_owned_resource)(owners[i]);
        }
        assert(EI_NAME(_owned_resource_drops)()==before+3);
    }
    EI_NAME(_owned_resource) *shared=EI_NAME(_collection_resource_new)(7), *receiver=EI_NAME(_collection_resource_new)(100);
    EI_NAME(_enum_inputs_borrow_p1_w1_row) borrowed[]={{shared,-1},{shared,17}};
    EI_NAME(_enum_inputs_reset)();
    size_t before=EI_NAME(_owned_resource_drops)();int64_t result=12345;char *error=NULL;
    assert(EI_NAME(_enum_inputs_borrow)(receiver,(EI_NAME(_enum_inputs_borrow_p1_w1_rows)){borrowed,2},&result,&error));
    assert(result==132 && error==NULL && EI_NAME(_enum_inputs_constructors)()==2);
    assert(EI_NAME(_owned_resource_drops)()==before);
    EI_NAME(_enum_inputs_reset)();size_t consumed=99;
    assert(EI_NAME(_enum_inputs_optional)(0,(EI_NAME(_enum_inputs_optional_p0_w3_rows)){(void *)1,SIZE_MAX},&result,&consumed,&error));
    assert(result==-1 && consumed==0 && EI_NAME(_enum_inputs_constructors)()==0);
    EI_NAME(_enum_inputs_optional_p0_w3_row) optional[]={
        {NULL,42},{EI_NAME(_collection_resource_new)(1),-1},{EI_NAME(_collection_resource_new)(2),17}
    };
    assert(EI_NAME(_enum_inputs_optional)(1,(EI_NAME(_enum_inputs_optional_p0_w3_rows)){optional,3},&result,&consumed,&error));
    assert(result==21 && consumed==2 && error==NULL && EI_NAME(_enum_inputs_constructors)()==2);
    EI_NAME(_enum_inputs_alternatives_p0_w1_row) alternatives[]={
        {0,EI_NAME(_collection_resource_new)(1),-3,(void *)1,42},
        {1,(void *)1,42,shared,17},
        {1,(void *)1,42,shared,-1}
    };
    EI_NAME(_enum_inputs_reset)();
    assert(EI_NAME(_enum_inputs_alternatives)((EI_NAME(_enum_inputs_alternatives_p0_w1_rows)){alternatives,3},&result,&consumed,&error));
    assert(result==30 && consumed==1 && error==NULL && EI_NAME(_enum_inputs_constructors)()==3);
    assert(EI_NAME(_option_resource_id)(shared)==7 && EI_NAME(_option_resource_id)(receiver)==100);
    EI_NAME(_drop_owned_resource)(shared);EI_NAME(_drop_owned_resource)(receiver);
}
#undef EI_NAME
#undef EI_CAT
#undef EI_CAT_RAW
