dump(unsigned long):
        sub     rsp, 40
        mov     rsi, rdi
        movabs  r10, -3689348814741910323
        mov     BYTE PTR [rsp+20], 10
        lea     rcx, [rsp+19]
        lea     r8, [rsp+21]
.L2:
        mov     rax, rsi
        mov     r9, r8
        mul     r10
        mov     rax, rsi
        sub     r9, rcx
        shr     rdx, 3
        lea     rdi, [rdx+rdx*4]
        add     rdi, rdi
        sub     rax, rdi
        add     eax, 48
        mov     BYTE PTR [rcx], al
        mov     rax, rsi
        mov     rsi, rdx
        mov     rdx, rcx
        sub     rcx, 1
        cmp     rax, 9
        ja      .L2
        sub     rdx, r8
        mov     edi, 1
        lea     rsi, [rsp+21+rdx]
        mov     rdx, r9
        call    write
        add     rsp, 40
        ret