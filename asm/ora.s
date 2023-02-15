; asmx example for ORA

	.cpu 6502
	org $0200
	
	lda #$00
	ldx #$fe
	ldy #$fd
	
	ora #$12
	ora #$cd
	ora $39
	ora $40,X
	ora $41,Y
	ora $0230
	ora $0230,X
	ora $0230,Y
	ora ($42,X)
	ora ($43),Y

	org $fffc
	dw $0200
	