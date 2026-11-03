
/tmp/c-output-before-normal-v3/target/release/libc_collection_bench.so:     file format elf64-x86-64


Disassembly of section .init:

Disassembly of section .fini:

Disassembly of section .text:

0000000000016180 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_>:
   16180:	55                   	push   %rbp
   16181:	41 57                	push   %r15
   16183:	41 56                	push   %r14
   16185:	41 55                	push   %r13
   16187:	41 54                	push   %r12
   16189:	53                   	push   %rbx
   1618a:	48 83 ec 68          	sub    $0x68,%rsp
   1618e:	49 89 d6             	mov    %rdx,%r14
   16191:	48 89 7c 24 20       	mov    %rdi,0x20(%rsp)
   16196:	48 8b 36             	mov    (%rsi),%rsi
   16199:	4c 8b 3a             	mov    (%rdx),%r15
   1619c:	48 8d 7c 24 50       	lea    0x50(%rsp),%rdi
   161a1:	ff 15 69 c9 03 00    	call   *0x3c969(%rip)        # 52b10 <_DYNAMIC+0x2a8>
   161a7:	48 8b 44 24 50       	mov    0x50(%rsp),%rax
   161ac:	48 89 04 24          	mov    %rax,(%rsp)
   161b0:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
   161b5:	48 8b 5c 24 60       	mov    0x60(%rsp),%rbx
   161ba:	48 c7 44 24 38 00 00 	movq   $0x0,0x38(%rsp)
   161c1:	00 00 
   161c3:	48 c7 44 24 40 01 00 	movq   $0x1,0x40(%rsp)
   161ca:	00 00 
   161cc:	48 c7 44 24 48 00 00 	movq   $0x0,0x48(%rsp)
   161d3:	00 00 
   161d5:	48 85 db             	test   %rbx,%rbx
   161d8:	4c 89 74 24 30       	mov    %r14,0x30(%rsp)
   161dd:	4c 89 7c 24 28       	mov    %r15,0x28(%rsp)
   161e2:	0f 84 a4 00 00 00    	je     1628c <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x10c>
   161e8:	4c 8d 2c dd 00 00 00 	lea    0x0(,%rbx,8),%r13
   161ef:	00 
   161f0:	ff 15 b2 c8 03 00    	call   *0x3c8b2(%rip)        # 52aa8 <_DYNAMIC+0x240>
   161f6:	be 08 00 00 00       	mov    $0x8,%esi
   161fb:	4c 89 ef             	mov    %r13,%rdi
   161fe:	ff 15 ac c8 03 00    	call   *0x3c8ac(%rip)        # 52ab0 <_DYNAMIC+0x248>
   16204:	48 85 c0             	test   %rax,%rax
   16207:	0f 84 5b 01 00 00    	je     16368 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x1e8>
   1620d:	48 89 5c 24 08       	mov    %rbx,0x8(%rsp)
   16212:	48 89 44 24 10       	mov    %rax,0x10(%rsp)
   16217:	48 c7 44 24 18 00 00 	movq   $0x0,0x18(%rsp)
   1621e:	00 00 
   16220:	45 31 f6             	xor    %r14d,%r14d
   16223:	48 8d 6c 24 08       	lea    0x8(%rsp),%rbp
   16228:	eb 33                	jmp    1625d <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0xdd>
   1622a:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
   16230:	48 8b 44 24 10       	mov    0x10(%rsp),%rax
   16235:	4a 89 1c f8          	mov    %rbx,(%rax,%r15,8)
   16239:	49 ff c7             	inc    %r15
   1623c:	4c 89 7c 24 18       	mov    %r15,0x18(%rsp)
   16241:	48 8b 5c 24 48       	mov    0x48(%rsp),%rbx
   16246:	48 83 fb ff          	cmp    $0xffffffffffffffff,%rbx
   1624a:	74 34                	je     16280 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x100>
   1624c:	48 ff c3             	inc    %rbx
   1624f:	48 89 5c 24 48       	mov    %rbx,0x48(%rsp)
   16254:	49 83 c6 08          	add    $0x8,%r14
   16258:	4d 39 f5             	cmp    %r14,%r13
   1625b:	74 4a                	je     162a7 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x127>
   1625d:	4b 8b 1c 34          	mov    (%r12,%r14,1),%rbx
   16261:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
   16266:	4c 3b 7c 24 08       	cmp    0x8(%rsp),%r15
   1626b:	75 c3                	jne    16230 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0xb0>
   1626d:	48 89 ef             	mov    %rbp,%rdi
   16270:	e8 4b db ff ff       	call   13dc0 <_RNvMs4_NtCscHgRw1M2fX5_5alloc7raw_vecINtB5_6RawVecNCNCNvCsbJkkCGhgRyB_18c_collection_bench15bench_make_many00E8grow_oneBS_>
   16275:	eb b9                	jmp    16230 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0xb0>
   16277:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
   1627e:	00 00 
   16280:	48 8d 7c 24 38       	lea    0x38(%rsp),%rdi
   16285:	e8 f6 db ff ff       	call   13e80 <_RNvMs4_NtCscHgRw1M2fX5_5alloc7raw_vecINtB5_6RawVecNCNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers00E8grow_oneBS_>
   1628a:	eb c0                	jmp    1624c <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0xcc>
   1628c:	48 c7 44 24 08 00 00 	movq   $0x0,0x8(%rsp)
   16293:	00 00 
   16295:	48 c7 44 24 10 08 00 	movq   $0x8,0x10(%rsp)
   1629c:	00 00 
   1629e:	48 c7 44 24 18 00 00 	movq   $0x0,0x18(%rsp)
   162a5:	00 00 
   162a7:	48 8b 34 24          	mov    (%rsp),%rsi
   162ab:	48 85 f6             	test   %rsi,%rsi
   162ae:	74 12                	je     162c2 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x142>
   162b0:	48 c1 e6 03          	shl    $0x3,%rsi
   162b4:	ba 08 00 00 00       	mov    $0x8,%edx
   162b9:	4c 89 e7             	mov    %r12,%rdi
   162bc:	ff 15 ce c7 03 00    	call   *0x3c7ce(%rip)        # 52a90 <_DYNAMIC+0x228>
   162c2:	4c 8b 7c 24 08       	mov    0x8(%rsp),%r15
   162c7:	4c 8b 64 24 10       	mov    0x10(%rsp),%r12
   162cc:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
   162d1:	4d 39 f7             	cmp    %r14,%r15
   162d4:	76 40                	jbe    16316 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x196>
   162d6:	49 c1 e7 03          	shl    $0x3,%r15
   162da:	4d 85 f6             	test   %r14,%r14
   162dd:	74 3c                	je     1631b <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x19b>
   162df:	4e 8d 2c f5 00 00 00 	lea    0x0(,%r14,8),%r13
   162e6:	00 
   162e7:	ba 08 00 00 00       	mov    $0x8,%edx
   162ec:	4c 89 e7             	mov    %r12,%rdi
   162ef:	4c 89 fe             	mov    %r15,%rsi
   162f2:	4c 89 e9             	mov    %r13,%rcx
   162f5:	ff 15 a5 c7 03 00    	call   *0x3c7a5(%rip)        # 52aa0 <_DYNAMIC+0x238>
   162fb:	48 89 c3             	mov    %rax,%rbx
   162fe:	48 85 c0             	test   %rax,%rax
   16301:	75 2e                	jne    16331 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x1b1>
   16303:	bf 08 00 00 00       	mov    $0x8,%edi
   16308:	4c 89 ee             	mov    %r13,%rsi
   1630b:	ff 15 87 c7 03 00    	call   *0x3c787(%rip)        # 52a98 <_DYNAMIC+0x230>
   16311:	e9 75 01 00 00       	jmp    1648b <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x30b>
   16316:	4c 89 e3             	mov    %r12,%rbx
   16319:	eb 16                	jmp    16331 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x1b1>
   1631b:	bb 08 00 00 00       	mov    $0x8,%ebx
   16320:	ba 08 00 00 00       	mov    $0x8,%edx
   16325:	4c 89 e7             	mov    %r12,%rdi
   16328:	4c 89 fe             	mov    %r15,%rsi
   1632b:	ff 15 5f c7 03 00    	call   *0x3c75f(%rip)        # 52a90 <_DYNAMIC+0x228>
   16331:	4d 85 f6             	test   %r14,%r14
   16334:	49 0f 44 de          	cmove  %r14,%rbx
   16338:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
   1633d:	48 89 18             	mov    %rbx,(%rax)
   16340:	4c 89 70 08          	mov    %r14,0x8(%rax)
   16344:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
   16349:	48 c7 00 ff ff ff ff 	movq   $0xffffffffffffffff,(%rax)
   16350:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
   16355:	48 89 48 08          	mov    %rcx,0x8(%rax)
   16359:	48 83 c4 68          	add    $0x68,%rsp
   1635d:	5b                   	pop    %rbx
   1635e:	41 5c                	pop    %r12
   16360:	41 5d                	pop    %r13
   16362:	41 5e                	pop    %r14
   16364:	41 5f                	pop    %r15
   16366:	5d                   	pop    %rbp
   16367:	c3                   	ret
   16368:	bf 08 00 00 00       	mov    $0x8,%edi
   1636d:	4c 89 ee             	mov    %r13,%rsi
   16370:	ff 15 22 c7 03 00    	call   *0x3c722(%rip)        # 52a98 <_DYNAMIC+0x230>
   16376:	e9 10 01 00 00       	jmp    1648b <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x30b>
   1637b:	49 89 c6             	mov    %rax,%r14
   1637e:	ba 08 00 00 00       	mov    $0x8,%edx
   16383:	4c 89 e7             	mov    %r12,%rdi
   16386:	4c 89 fe             	mov    %r15,%rsi
   16389:	eb 59                	jmp    163e4 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x264>
   1638b:	49 89 c6             	mov    %rax,%r14
   1638e:	48 83 3c 24 00       	cmpq   $0x0,(%rsp)
   16393:	74 55                	je     163ea <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x26a>
   16395:	48 8b 34 24          	mov    (%rsp),%rsi
   16399:	48 c1 e6 03          	shl    $0x3,%rsi
   1639d:	ba 08 00 00 00       	mov    $0x8,%edx
   163a2:	4c 89 e7             	mov    %r12,%rdi
   163a5:	eb 3d                	jmp    163e4 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x264>
   163a7:	49 89 c6             	mov    %rax,%r14
   163aa:	eb 3e                	jmp    163ea <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x26a>
   163ac:	49 89 c6             	mov    %rax,%r14
   163af:	48 83 3c 24 00       	cmpq   $0x0,(%rsp)
   163b4:	74 16                	je     163cc <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x24c>
   163b6:	48 8b 34 24          	mov    (%rsp),%rsi
   163ba:	48 c1 e6 03          	shl    $0x3,%rsi
   163be:	ba 08 00 00 00       	mov    $0x8,%edx
   163c3:	4c 89 e7             	mov    %r12,%rdi
   163c6:	ff 15 c4 c6 03 00    	call   *0x3c6c4(%rip)        # 52a90 <_DYNAMIC+0x228>
   163cc:	48 8b 74 24 08       	mov    0x8(%rsp),%rsi
   163d1:	48 85 f6             	test   %rsi,%rsi
   163d4:	74 14                	je     163ea <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x26a>
   163d6:	48 8b 7c 24 10       	mov    0x10(%rsp),%rdi
   163db:	48 c1 e6 03          	shl    $0x3,%rsi
   163df:	ba 08 00 00 00       	mov    $0x8,%edx
   163e4:	ff 15 a6 c6 03 00    	call   *0x3c6a6(%rip)        # 52a90 <_DYNAMIC+0x228>
   163ea:	4c 89 f7             	mov    %r14,%rdi
   163ed:	ff 15 fd c6 03 00    	call   *0x3c6fd(%rip)        # 52af0 <_DYNAMIC+0x288>
   163f3:	49 89 c6             	mov    %rax,%r14
   163f6:	49 89 d7             	mov    %rdx,%r15
   163f9:	ff 15 a9 c6 03 00    	call   *0x3c6a9(%rip)        # 52aa8 <_DYNAMIC+0x240>
   163ff:	bf 17 00 00 00       	mov    $0x17,%edi
   16404:	be 01 00 00 00       	mov    $0x1,%esi
   16409:	ff 15 a1 c6 03 00    	call   *0x3c6a1(%rip)        # 52ab0 <_DYNAMIC+0x248>
   1640f:	48 85 c0             	test   %rax,%rax
   16412:	74 67                	je     1647b <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x2fb>
   16414:	0f 10 05 23 2c ff ff 	movups -0xd3dd(%rip),%xmm0        # 903e <GCC_except_table151+0x12da>
   1641b:	0f 11 00             	movups %xmm0,(%rax)
   1641e:	48 b9 70 61 6e 69 63 	movabs $0x64656b63696e6170,%rcx
   16425:	6b 65 64 
   16428:	48 89 48 0f          	mov    %rcx,0xf(%rax)
   1642c:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
   16431:	48 c7 01 17 00 00 00 	movq   $0x17,(%rcx)
   16438:	48 89 41 08          	mov    %rax,0x8(%rcx)
   1643c:	48 c7 41 10 17 00 00 	movq   $0x17,0x10(%rcx)
   16443:	00 
   16444:	49 8b 07             	mov    (%r15),%rax
   16447:	48 85 c0             	test   %rax,%rax
   1644a:	74 05                	je     16451 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x2d1>
   1644c:	4c 89 f7             	mov    %r14,%rdi
   1644f:	ff d0                	call   *%rax
   16451:	49 8b 77 08          	mov    0x8(%r15),%rsi
   16455:	48 85 f6             	test   %rsi,%rsi
   16458:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
   1645d:	0f 84 f6 fe ff ff    	je     16359 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x1d9>
   16463:	49 8b 57 10          	mov    0x10(%r15),%rdx
   16467:	4c 89 f7             	mov    %r14,%rdi
   1646a:	48 89 c3             	mov    %rax,%rbx
   1646d:	ff 15 1d c6 03 00    	call   *0x3c61d(%rip)        # 52a90 <_DYNAMIC+0x228>
   16473:	48 89 d8             	mov    %rbx,%rax
   16476:	e9 de fe ff ff       	jmp    16359 <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x1d9>
   1647b:	bf 01 00 00 00       	mov    $0x1,%edi
   16480:	be 17 00 00 00       	mov    $0x17,%esi
   16485:	ff 15 0d c6 03 00    	call   *0x3c60d(%rip)        # 52a98 <_DYNAMIC+0x230>
   1648b:	0f 0b                	ud2
   1648d:	48 89 c3             	mov    %rax,%rbx
   16490:	49 8b 77 08          	mov    0x8(%r15),%rsi
   16494:	48 85 f6             	test   %rsi,%rsi
   16497:	74 23                	je     164bc <_RINvCslWy1mbd6r6o_18prebindgen_runtime8boundaryuNCNvCsbJkkCGhgRyB_18c_collection_bench18bench_make_numbers0EBO_+0x33c>
   16499:	49 8b 57 10          	mov    0x10(%r15),%rdx
   1649d:	4c 89 f7             	mov    %r14,%rdi
   164a0:	ff 15 ea c5 03 00    	call   *0x3c5ea(%rip)        # 52a90 <_DYNAMIC+0x228>
   164a6:	48 89 df             	mov    %rbx,%rdi
   164a9:	e8 62 9c 03 00       	call   50110 <_Unwind_Resume@plt>
   164ae:	48 89 c3             	mov    %rax,%rbx
   164b1:	4c 89 f7             	mov    %r14,%rdi
   164b4:	4c 89 fe             	mov    %r15,%rsi
   164b7:	e8 e4 07 00 00       	call   16ca0 <_RINvNtCsc36rpYXAlPq_4core3ptr9drop_glueINtNtCscHgRw1M2fX5_5alloc5boxed3BoxDNtNtB4_3any3AnyNtNtB4_6marker4SendEL_EECsbJkkCGhgRyB_18c_collection_bench>
   164bc:	48 89 df             	mov    %rbx,%rdi
   164bf:	e8 4c 9c 03 00       	call   50110 <_Unwind_Resume@plt>
   164c4:	ff 15 2e c6 03 00    	call   *0x3c62e(%rip)        # 52af8 <_DYNAMIC+0x290>
   164ca:	ff 15 00 c6 03 00    	call   *0x3c600(%rip)        # 52ad0 <_DYNAMIC+0x268>

Disassembly of section .plt:
