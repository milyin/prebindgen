package io.prebindgen.covertest
import io.prebindgen.covertest.payloadenums.*
fun checkPayloadEnums() {
    for(mode in listOf(null,WireRecoveryMode.Heartbeat,WireRecoveryMode.PeriodicQueries(999uL))) {
        val config=WireRecoveryConfig(mode,123uL)
        check(wireRecoveryEcho(config)==config)
        check(wireRecoveryEcho(config.copy(retention_period=null))==config.copy(retention_period=null))
    }
    val timestamps=listOf(
        WireTimestampStackRecord(WireInterceptionPoint.Send,WireInstrumentationTimestamp.Uhlc(WireTimestamp(ULong.MAX_VALUE,byteArrayOf(0,-1)))),
        WireTimestampStackRecord(WireInterceptionPoint.Receive,WireInstrumentationTimestamp.Custom(byteArrayOf(7,8))),
        WireTimestampStackRecord(WireInterceptionPoint.Route,WireInstrumentationTimestamp.Uhlc(WireTimestamp(0uL,byteArrayOf())))
    )
    check(wireTimestamps(timestamps)==timestamps)
    val liveBefore=io.prebindgen.covertest.callbackreturns.callbackReturnResourceLive()
    val composite=wireCompositeOwnedNew() as WireCompositeOwnedEvent.Data
    check(composite.detail==WireEvent.Count(9uL))
    composite.close()
    check(io.prebindgen.covertest.callbackreturns.callbackReturnResourceLive()==liveBefore)
    val owned=wireOwnedNew() as WireOwnedEvent.Pair
    val moved=wireOwnedEcho(owned) as WireOwnedEvent.Pair
    check(owned.first.isClosed && owned.second.isClosed)
    check(!moved.first.isClosed && !moved.second.isClosed)
    moved.close()
    val returnedOwned=wireOwnedCallback({ event -> check(event is WireOwnedEvent.Pair);WireOwnedEvent.Pair(event.first.take(),event.second.take()) }) as WireOwnedEvent.Pair
    returnedOwned.close()
    val ownedRows=wireOwnedRowsNew()
    val movedRows=wireOwnedRows(ownedRows)
    for(row in movedRows) if(row is WireOwnedEvent.Pair) row.close()
    val optionalRows=listOf(
        WireNicheEvent.Empty,
        WireNicheEvent.Data(null,null),
        WireNicheEvent.Data(WireOptionalOwner(false,io.prebindgen.covertest.callbackreturns.callbackReturnResourceNew(3)),null),
        WireNicheEvent.Data(WireOptionalOwner(true,io.prebindgen.covertest.callbackreturns.callbackReturnResourceNew(4)),io.prebindgen.covertest.callbackreturns.callbackReturnResourceNew(5))
    )
    val optionalMoved=wireNicheRows(optionalRows)
    check(optionalMoved[0]==WireNicheEvent.Empty)
    check(optionalMoved[1]==WireNicheEvent.Data(null,null))
    check(!(optionalMoved[2] as WireNicheEvent.Data).small!!.enabled)
    check((optionalMoved[3] as WireNicheEvent.Data).small!!.enabled)
    optionalMoved.forEach { it.close() }
    for(row in optionalRows) if(row is WireNicheEvent.Data) {
        check(row.small?.resource?.isClosed!=false && row.wide?.isClosed!=false)
    }
    check(io.prebindgen.covertest.callbackreturns.callbackReturnResourceLive()==liveBefore)
    val first=io.prebindgen.covertest.callbackreturns.callbackReturnResourceNew(1)
    val last=io.prebindgen.covertest.callbackreturns.callbackReturnResourceNew(2)
    expectEnumFailure { wireCheckedCallback({ WireCheckedEvent.Data(first,-1,last) },onError={throw AssertionError(it)}) }
    check(first.isClosed && last.isClosed)
    check(io.prebindgen.covertest.callbackreturns.callbackReturnResourceLive()==liveBefore)
    expectEnumFailure { wireCheckedNew(-2,onError={throw AssertionError(it)}) }
    check(io.prebindgen.covertest.callbackreturns.callbackReturnResourceLive()==liveBefore)
    for(empty in listOf(WireEmpty.Unit,WireEmpty.Tuple,WireEmpty.Named)) check(wireEmptyEcho(empty)==empty)
    check(WIRE_EMPTY_UNIT_TAG==0 && WIRE_EMPTY_TUPLE_TAG==1 && WIRE_EMPTY_NAMED_TAG==2)
    wireReplyEmit(false,{ tag,sample,payload,errorPayload,encoding,replier ->
        check(tag==WIRE_REPLY_RESULT_SAMPLE_TAG && replier==33L)
        check(requireNotNull(payload).contentEquals(byteArrayOf(8,9)))
        check(errorPayload==null && encoding==0)
        requireNotNull(sample).close()
    },onError={throw AssertionError(it)})
    wireReplyEmit(true,{ tag,sample,payload,errorPayload,encoding,replier ->
        check(tag==WIRE_REPLY_RESULT_ERROR_TAG && replier==null)
        check(sample==null && payload==null && encoding==7)
        check(requireNotNull(errorPayload).contentEquals(byteArrayOf(0,-1,3)))
    },onError={throw AssertionError(it)})
    check(io.prebindgen.covertest.callbackreturns.callbackReturnResourceLive()==liveBefore)
    wireBareEcho(WIRE_BARE_PACKET_TAG,0,17uL,2.5) { tag,count,x,y -> check(tag==WIRE_BARE_PACKET_TAG && count==0L && x==17uL && y==2.5) }
    check(wireSelfNameEcho(WireSelfName.WireSelfName(9uL))==WireSelfName.WireSelfName(9uL))
    val nested=listOf(WireNestedDecision.Ok(WireDecision.Ok(1)),WireNestedDecision.Ok(WireDecision.Err("inner")),WireNestedDecision.Err("outer"))
    for(value in nested) check(wireNestedEcho(value)==value)
    check(wireNestedOutcome(1,onError={throw AssertionError(it)})==WireDecision.Err("inner"))
    check(wireNestedOutcome(2,onError={ message -> check(message=="outer");WireDecision.Ok(88) })==WireDecision.Ok(88))
    var sourceError=false
    wireErrorOutcome(onError={ error -> check(error==WireErrorKind.Code(73));check(wireBenchTotal(WireBench.Count(3uL))==3uL);sourceError=true })
    check(sourceError)
    var borrowed:WireEvent?=null
    wireEventBorrowed({ event -> borrowed=event })
    check(borrowed==WireEvent.Packet(99uL,byteArrayOf(4,5)))
    val values=listOf(WireEvent.Idle,WireEvent.Count(ULong.MAX_VALUE),WireEvent.Packet(17uL,byteArrayOf(0,-1,42)),WireEvent.EmptyTuple,WireEvent.EmptyRecord)
    for(value in values) check(wireEventEcho(value)==value)
    check(wireEventRows(values)==values)
    check(wireEventRowsCallback({ rows -> check(rows==listOf(WireEvent.Idle,WireEvent.Packet(ULong.MAX_VALUE,byteArrayOf(8,9)),WireEvent.Count(19uL)));values })==values)
    val returned=wireEventCallback({ value ->
        check(value==WireEvent.Packet(ULong.MAX_VALUE,byteArrayOf(1,2,3)))
        WireEvent.Count(23uL)
    })
    check(returned==WireEvent.Count(23uL))
    check(wireResultEcho(WireDecision.Ok(-7))==WireDecision.Ok(-7))
    check(wireResultEcho(WireDecision.Err("binary\u0000error"))==WireDecision.Err("binary\u0000error"))
    check(wireResultCallback({ notice -> check(notice==WireNotice.Ok(Unit));WireDecision.Err("callback result") })==WireDecision.Err("callback result"))
}

private fun expectEnumFailure(call:()->Unit) {
    try {call()} catch(_:IllegalArgumentException) {return}
    error("expected enum conversion failure")
}
