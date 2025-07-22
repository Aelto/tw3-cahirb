
# Decompiling notes
```js
function setbetter_flow_choice_weight(choice_description: string, choice_weight: int) {
  var better_flow_choice: MCM_BetterFlowWeight;

  better_flow_choice.description = choice_description;
  better_flow_choice.weight = choice_weight;
  parent.better_flow_weights.PushBack(better_flow_choice);
}
```
turns into:
```js
Breakpoint( source_line=612  is_set=false )
StructMember( property=(MCM_BetterFlowWeight)description )
LocalVar( name=better_flow_choice )
ParamVar( name=choice_description )
Breakpoint( source_line=613  is_set=false )
StructMember( property=(MCM_BetterFlowWeight)weight )
LocalVar( name=better_flow_choice )
ParamVar( name=choice_weight )
Breakpoint( is_set=false  source_line=614 )
ArrayPushBack( type=array:2,0,MCM_BetterFlowWeight )
Context( skip_offset=11 )
ObjectVar( property=(MCM_RandomDialogPicker)better_flow_weights )
LocalVar( name=better_flow_choice )
```

# setting a struct field
```js
StructMember( property=(MCM_BetterFlowWeight)weight ) // field name
LocalVar( name=better_flow_choice ) // where struct is stored, variable name
ParamVar( name=choice_weight ) // value
```

# accessing `parent` field
`parent` seems to be `Context( skip_offset=11 )`

# array pushing
`parent.field.pushback()` turns into

```js
ArrayPushBack( type=array:2,0,MCM_BetterFlowWeight )

Context( skip_offset=11 )
ObjectVar( property=(MCM_RandomDialogPicker)better_flow_weights )

LocalVar( name=better_flow_choice )
```


# calling this.method()
```js
this.setbetter_flow_choice_weight(GetLocStringById(520972), 200);
```
```js
Breakpoint( is_set=false  source_line=604 )
Context( skip_offset=35 )
VirtualFunc( function=setbetter_flow_choice_weight  skip_offset=31  source_line=604 )
FinalFunc( source_line=604  skip_offset=16  function=::GetLocStringById )
IntConst( value=520972 )
IntConst( value=200 )
```

# bool comparisons
```js
protected function isAxiiAction(choice: SSceneChoice): bool {
  return choice.dialogAction == DialogAction_AXII
      || choice.dialogAction == DialogAction_PERSUASION;
}

Return() size=1 offset=0 opcode=33
  Breakpoint( source_line=253  is_set=false ) size=6 offset=1 opcode=12
  FinalFunc( skip_offset=124  source_line=253  function=LogicOr_Bool_Bool ) size=13 offset=7 opcode=28
    FinalFunc( skip_offset=52  source_line=253  function=Equal_Int32_Int32 ) size=13 offset=20 opcode=28
      EnumToInt( type=EDialogActionIcon ) size=9 offset=33 opcode=99
        StructMember( property=(SSceneChoice)dialogAction ) size=9 offset=42 opcode=34
          ParamVar( name=choice ) size=5 offset=51 opcode=16
      EnumToInt( type=EDialogActionIcon ) size=9 offset=56 opcode=99
        IntConst( value=2 ) size=5 offset=65 opcode=4
    ParamEnd() size=1 offset=70 opcode=32
    Skip( skip_offset=55 ) size=3 offset=71 opcode=25
    FinalFunc( skip_offset=52  source_line=253  function=Equal_Int32_Int32 ) size=13 offset=74 opcode=28
      EnumToInt( type=EDialogActionIcon ) size=9 offset=87 opcode=99
        StructMember( property=(SSceneChoice)dialogAction ) size=9 offset=96 opcode=34
          ParamVar( name=choice ) size=5 offset=105 opcode=16
      EnumToInt( type=EDialogActionIcon ) size=9 offset=110 opcode=99
        IntConst( value=32 ) size=5 offset=119 opcode=4
    ParamEnd() size=1 offset=124 opcode=32
  ParamEnd() size=1 offset=125 opcode=32
Nop() size=1 offset=126 opcode=0
```
- First it calls a `FinalFunc` then it passes it two parameters
- on multiple comparisons like here, calls are nested so it really depends on the
  parsing being able to parse functions calls like `EnumToInt( type=EDialogActionIcon ) IntConst( value=2 )`
- the `FinalFunc` for example has an `offset=20` and a `skip_offset=52` which is more or less the offset of the last `Skip` offset + 1
- for things without a `skip_offset` it seems to be from from `offset` to `offset+size` of that same instruction. So it takes all instructions as long as `offset <= offset+size`
  - For example
    ```js
    EnumToInt( type=EDialogActionIcon ) size=9 offset=33 opcode=99 // <-- 33+9 = 42
      StructMember( property=(SSceneChoice)dialogAction ) size=9 offset=42 opcode=34
    ```