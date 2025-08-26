
# For loops
 - they're simple If checks with the iteration as the first instructions
 - at the end there is a Jump with negative offset
```
Jump( skip_offset=-198 ) size=3 offset=288 opcode=23
Jump( skip_offset=-302 ) size=3 offset=291 opcode=23
```

- **odd stuff to figure out** bigger jump offset than its own offset:
  ```js
  Jump( skip_offset=-302 ) size=3 offset=291 opcode=23
  ```

# nested getter calls
```js
Breakpoint( source_line=86  is_set=false ) size=6 offset=0 opcode=12
Context( skip_offset=33 ) size=3 offset=6 opcode=35
Context( skip_offset=15 ) size=3 offset=9 opcode=35
This() size=1 offset=12 opcode=40
FinalFunc( skip_offset=11  source_line=86  function=CGameplayEntity::GetInventory ) size=13 offset=13 opcode=28
ParamEnd() size=1 offset=26 opcode=32
VirtualFunc( source_line=86  function=AddMoney  skip_offset=12 ) size=9 offset=27 opcode=29
IntConst( value=5 ) size=5 offset=36 opcode=4
ParamEnd() size=1 offset=41 opcode=32
Nop() size=1 offset=42 opcode=0
```

should result in
```js
this.GetInventory().AddMoney();
```

But at the moment it results in two lines:
```js
this.GetInventory();
AddMoney()
```

# constructor lifetime
- [ ] at the moment the lifetime is not decompiled, but the next instruction is actually the lifetime of the constructor. It can be seen by the extra `this` following constructor calls in the emited code.
