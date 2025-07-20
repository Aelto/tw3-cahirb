#[derive(Debug)]
enum RTTITypes {
    Simple = 0,
    Enum = 1,
    Class = 2,
    Array = 3, // dynamic array
    StaticArray = 4,
    NativeArray = 5,
    Pointer = 6,
    Handle = 7,
    SoftHandle = 8,
    BitField = 9,
    Void = 10,
    Fundamental = 11,
}
